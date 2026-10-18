"""`emit`: the one function application code imports.

It has no dependencies beyond the standard library and does nothing unless a test is
recording: in production it costs a list check and an environment lookup.
"""

from __future__ import annotations

import enum
import itertools
import json
import os
import sys
import threading
import time
from dataclasses import dataclass, field
from datetime import date, datetime, timezone
from typing import Any, Optional, Union

EVENTS_DIR_ENV = "PRS_EVENTS_DIR"

_NO_PAYLOAD: Any = object()


@dataclass
class Event:
    """One emitted event."""

    event: str
    payload: Any
    key: str
    #: Nanoseconds since the epoch (or the logical time passed as `at`).
    time_ns: int
    #: `True` when the time came from `at=` rather than the clock.
    at: bool = False
    #: `file:line` of the `emit` call (or `path:line` of a parsed log line).
    src: Optional[str] = None
    #: Order of emission within this process.
    seq: int = 0
    #: Process that emitted it.
    pid: int = field(default_factory=os.getpid)

    def to_json(self) -> str:
        """The JSON line `prs observe` reads, plus fields used for reports."""
        d: dict = {
            "key": self.key,
            "timestamp": iso_timestamp(self.time_ns),
            "event": self.event,
            "payload": self.payload,
            "src": self.src,
            "seq": self.seq,
            "pid": self.pid,
        }
        if self.at:
            d["at"] = True
        return json.dumps(d, default=_json_default, separators=(",", ":"))

    @staticmethod
    def from_json(d: dict) -> "Event":
        return Event(
            event=d.get("event", ""),
            payload=d.get("payload"),
            key=_key_str(d.get("key")),
            time_ns=0,
            at=bool(d.get("at")),
            src=d.get("src"),
            seq=d.get("seq", 0) or 0,
            pid=d.get("pid", 0) or 0,
        )


# Recorders currently collecting events in this process, innermost last. A plain list,
# not a contextvar, so that events from threads started inside a test are kept.
_recorders: list = []
_lock = threading.Lock()
_seq = itertools.count(1)
_last_ns = 0
_file = None
_file_pid = 0
_file_dir = None


def _after_fork_in_child() -> None:
    # A forked child (multiprocessing on Linux) inherits the parent's recorders, but its
    # events would stay in its own copy of them: it writes to PRS_EVENTS_DIR instead.
    global _file
    _recorders.clear()
    _file = None


if hasattr(os, "register_at_fork"):
    os.register_at_fork(after_in_child=_after_fork_in_child)


def emit(event: str, payload: Any = _NO_PAYLOAD, /, *, key: Any = None, at: Union[datetime, int, None] = None, **fields: Any) -> None:
    """Record that `event` happened, for P specs to check.

    The payload is either the keyword arguments (``emit("eEnter", client="c1")``) or one
    positional value, for events whose payload is not a named tuple or whose field names
    clash with ``key``/``at`` (``emit("eClose", 7)``, ``emit("ePut", {"key": "k"})``).

    ``key`` splits events into independent streams, each checked by its own spec
    instance. ``at`` overrides the event time: a ``datetime``, or an integer of epoch
    nanoseconds (any increasing counter works). Without a recording test this does nothing.
    """
    if not _recorders:
        events_dir = os.environ.get(EVENTS_DIR_ENV)
        if not events_dir:
            return
    else:
        events_dir = None
    if payload is _NO_PAYLOAD:
        payload = fields if fields else None
    elif fields:
        raise TypeError("emit() takes either a positional payload or keyword fields, not both")
    ev = _make_event(event, payload, key, at, sys._getframe(1))
    if events_dir is None:
        for r in tuple(_recorders):
            r._add(ev)
    else:
        _write_to_dir(events_dir, ev)


def _make_event(event: str, payload: Any, key: Any, at: Any, frame: Any) -> Event:
    global _last_ns
    if not isinstance(event, str):
        raise TypeError(f"emit(): the event name must be a string, not {type(event).__name__}")
    src = f"{_short_path(frame.f_code.co_filename)}:{frame.f_lineno}" if frame is not None else None
    with _lock:
        if at is None:
            # never earlier than the previous event, so a clock step back keeps emit order
            now = max(time.time_ns(), _last_ns)
            _last_ns = now
            t, overridden = now, False
        else:
            t, overridden = to_ns(at), True
        return Event(event, payload, _key_str(key), t, overridden, src, next(_seq))


def _write_to_dir(events_dir: str, ev: Event) -> None:
    """Appends to this process's own file in `events_dir`, so writes never interleave."""
    global _file, _file_pid, _file_dir
    line = ev.to_json() + "\n"
    with _lock:
        pid = os.getpid()
        if _file is None or _file_pid != pid or _file_dir != events_dir:
            if _file is not None and _file_pid == pid:
                _file.close()
            os.makedirs(events_dir, exist_ok=True)
            _file = open(os.path.join(events_dir, f"events-{pid}.jsonl"), "a", encoding="utf-8")
            _file_pid, _file_dir = pid, events_dir
        _file.write(line)
        _file.flush()


def to_ns(at: Union[datetime, date, int, float]) -> int:
    """Epoch nanoseconds of an `at=` value. Naive datetimes are local time, as in Python."""
    if isinstance(at, bool):
        raise TypeError("emit(at=...) takes a datetime or an integer, not a bool")
    if isinstance(at, int):
        return at
    if isinstance(at, datetime):
        if at.tzinfo is None:
            at = at.astimezone()
        delta = at - datetime(1970, 1, 1, tzinfo=timezone.utc)
        return (delta.days * 86_400 + delta.seconds) * 1_000_000_000 + delta.microseconds * 1_000
    if isinstance(at, float):
        raise TypeError("emit(at=...) takes a datetime or integer nanoseconds; got a float (seconds?)")
    raise TypeError(f"emit(at=...) takes a datetime or an integer, not {type(at).__name__}")


def iso_timestamp(ns: int) -> str:
    """ISO-8601 with nanoseconds, which prs orders exactly (bare integers read as ms)."""
    secs, nanos = divmod(ns, 1_000_000_000)
    return time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime(secs)) + f".{nanos:09d}Z"


def _key_str(key: Any) -> str:
    if key is None:
        return ""
    return key if isinstance(key, str) else str(key)


def _json_default(o: Any) -> Any:
    if isinstance(o, enum.Enum):
        return o.name
    if isinstance(o, (set, frozenset, tuple)):
        return list(o)
    if isinstance(o, (datetime, date)):
        return o.isoformat()
    if hasattr(o, "__dataclass_fields__"):
        return {k: getattr(o, k) for k in o.__dataclass_fields__}
    return str(o)


_cwd = os.getcwd()


def _short_path(path: str) -> str:
    try:
        rel = os.path.relpath(path, _cwd)
    except ValueError:  # another drive on Windows
        return path
    return path if rel.startswith("..") else rel
