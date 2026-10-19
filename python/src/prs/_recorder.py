"""The recorder both modes share: collects events, runs `prs observe`, builds reports."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, Tuple

from . import _config, _emit
from ._binary import find_prs
from ._emit import Event

#: Events shown in a failure report, counting back from the one that broke the spec.
REPORT_EVENTS = 20

LogParser = Callable[[str], Optional[dict]]


class PrsError(Exception):
    """prs could not run the check: bad configuration, unknown spec, P type errors, or
    events whose payload does not match the spec's event declaration."""


class SpecViolation(AssertionError):
    """A P spec was broken by the recorded events."""

    def __init__(
        self,
        spec: str,
        message: str,
        key: str = "",
        events: Sequence[Event] = (),
        *,
        location: Optional[str] = None,
        liveness: bool = False,
        earlier: int = 0,
        output_dir: Optional[str] = None,
        others: Sequence["SpecViolation"] = (),
    ):
        #: Name of the broken spec.
        self.spec = spec
        #: The spec's assert message (or, for liveness, which hot state it ended in).
        self.message = message
        #: Event key (stream) the violation happened on.
        self.key = key
        #: Events of that key up to and including the violating one (the last ones only).
        self.events = list(events)
        #: `file:line` of the failed assert in the P sources.
        self.location = location
        #: The spec ended in a hot state (a liveness rule was still waiting).
        self.liveness = liveness
        #: Events of this key before `events` that are not shown.
        self.earlier = earlier
        #: Folder with prs's full output.
        self.output_dir = output_dir
        #: Further violations found by the same check.
        self.others = list(others)
        super().__init__(self.report())

    def report(self) -> str:
        lines = []
        if self.liveness:
            lines.append(f"P spec {self.spec} was still waiting when the events ended: {self.message}")
        else:
            lines.append(f"P spec {self.spec} was broken: {self.message}")
        lines.append("")
        lines.append(f'  Events (key "{self.key}"):' if self.key else "  Events:")
        if self.earlier:
            lines.append(f"     ... {self.earlier} earlier event(s)")
        rows = [(_payload_text(e.payload), e) for e in self.events]
        w_event = max([len(e.event) for _, e in rows] + [0])
        w_payload = min(max([len(p) for p, _ in rows] + [0]), 48)
        for i, (p, e) in enumerate(rows):
            n = self.earlier + i + 1
            row = f"    {n:>3}  {e.event:<{w_event}}  {p:<{w_payload}}  {e.src or ''}"
            if e.at:
                row += "  (time set with at=)"
            if i == len(rows) - 1:
                row += "  <- events ended here" if self.liveness else "  <- broke the spec here"
            lines.append(row.rstrip() if i < len(rows) - 1 else row)
        if not rows:
            lines.append("    (none)")
        lines.append("")
        if self.location:
            lines.append(f"  Spec: {self.location}")
        if self.output_dir:
            lines.append(f"  prs output: {self.output_dir}")
        if self.others:
            lines.append("")
            lines.append(f"  {len(self.others)} more violation(s):")
            for o in self.others:
                key = f' (key "{o.key}")' if o.key else ""
                lines.append(f"    {o.spec}{key}: {o.message}")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.report()


class Recorder:
    """Collects the events emitted while it is active and checks them against P specs.

    ``specs`` names the specs to check; none means every spec in the spec files.
    ``spec_files`` defaults to ``[tool.prs] spec_files`` in pyproject.toml.
    """

    def __init__(
        self,
        specs: Iterable[str] = (),
        *,
        spec_files: Optional[Sequence[str]] = None,
        exclude: Iterable[str] = (),
        liveness: bool = False,
        output_root: Optional[str] = None,
        coverage: bool = False,
        strict: bool = False,
    ):
        self.specs: List[str] = list(specs)
        self.spec_files = list(spec_files) if spec_files is not None else None
        self.exclude: List[str] = list(exclude)
        self.liveness = liveness
        self.strict = strict
        self.coverage = coverage
        self.coverage_report: Optional[dict] = None
        #: Specs whose violations this recorder ignores (broken on purpose inside
        #: `expect_violation`).
        self.ignored: set = set()
        self._events: List[Event] = []
        self._logs: List[Tuple[str, Optional[LogParser]]] = []
        self._dirs: List[str] = []
        self._owned: List[str] = []
        self._output_root = output_root
        self._active = False
        self._prev_env: Optional[str] = None
        self._child_dir: Optional[str] = None

    # ---- collecting ----------------------------------------------------------------

    def start(self) -> "Recorder":
        """Starts collecting: events emitted in this process, and in processes started
        from now on (through the `PRS_EVENTS_DIR` environment variable)."""
        if self._active:
            return self
        self._child_dir = tempfile.mkdtemp(prefix="prs-events-")
        self._owned.append(self._child_dir)
        self._dirs.append(self._child_dir)
        self._prev_env = os.environ.get(_emit.EVENTS_DIR_ENV)
        os.environ[_emit.EVENTS_DIR_ENV] = self._child_dir
        _emit._recorders.append(self)
        self._active = True
        return self

    def stop(self) -> None:
        """Stops collecting. Events child processes wrote are passed on to the recorders
        still active, so an enclosing test-wide check sees them too."""
        if not self._active:
            return
        self._active = False
        if self in _emit._recorders:
            _emit._recorders.remove(self)
        if self._prev_env is None:
            os.environ.pop(_emit.EVENTS_DIR_ENV, None)
        else:
            os.environ[_emit.EVENTS_DIR_ENV] = self._prev_env
        if _emit._recorders and self._child_dir:
            for outer in _emit._recorders:
                outer._dirs.append(self._child_dir)
            # the outermost recorder deletes the folder when it is done
            _emit._recorders[0]._owned.append(self._child_dir)
            self._owned.remove(self._child_dir)

    def __enter__(self) -> "Recorder":
        return self.start()

    def __exit__(self, *exc: Any) -> None:
        self.stop()

    def _add(self, ev: Event) -> None:
        self._events.append(ev)

    def add_log(self, path: str, parse: Optional[LogParser] = None) -> None:
        """Also check the events in the log file at `path`, read when the check runs.

        ``parse`` turns one line into an event or ``None``: a dict like ``emit``'s
        arguments, ``{"event": "eEnter", "key": "L1", "at": when, "client": "c1"}``
        (``at`` is required so events can be merged by time). Without ``parse`` the file
        must hold prs's JSON lines."""
        self._logs.append((os.fspath(path), parse))

    def add_events_dir(self, path: str) -> None:
        """Also check the JSON-lines files in folder `path`, e.g. one mounted into a
        container whose service writes events there."""
        self._dirs.append(os.fspath(path))

    # ---- looking at events -----------------------------------------------------------

    @property
    def events(self) -> List[Event]:
        """Every event collected so far: emitted here, by child processes and in logs,
        in time order."""
        evs = list(self._events) + [e for _, e in self._external_lines()]
        evs.sort(key=lambda e: e.time_ns)
        return evs

    def events_named(self, name: str) -> List[Any]:
        """The payloads of the events called `name`, in order."""
        return [e.payload for e in self.events if e.event == name]

    def count(self, name: str) -> int:
        return sum(1 for e in self.events if e.event == name)

    def _external_lines(self) -> List[Tuple[str, Event]]:
        """(JSON line, event) for events written by other processes and in logs."""
        out: List[Tuple[str, Event]] = []
        for d in self._dirs:
            if not os.path.isdir(d):
                continue
            for name in sorted(os.listdir(d)):
                p = os.path.join(d, name)
                if not os.path.isfile(p):
                    continue
                with open(p, encoding="utf-8", errors="replace") as f:
                    for line in f:
                        line = line.strip()
                        if line:
                            out.append((line, _event_of_line(line)))
        for path, parse in self._logs:
            with open(path, encoding="utf-8", errors="replace") as f:
                for lineno, line in enumerate(f, 1):
                    if parse is None:
                        line = line.strip()
                        if line:
                            out.append((line, _event_of_line(line)))
                        continue
                    d = parse(line.rstrip("\n"))
                    if d is None:
                        continue
                    ev = _event_from_dict(d, f"{path}:{lineno}", what=f"the parser of {path}")
                    out.append((ev.to_json(), ev))
        return out

    # ---- checking --------------------------------------------------------------------

    def check(self) -> None:
        """Runs prs over the events; raises SpecViolation if a spec was broken."""
        __tracebackhide__ = True
        violations = self.violations()
        if violations:
            first = violations[0]
            first.others = violations[1:]
            first.args = (first.report(),)
            raise first

    def violations(self) -> List[SpecViolation]:
        """Runs prs over the events and returns the violations found (empty if none)."""
        lines = [e.to_json() for e in self._events] + [line for line, _ in self._external_lines()]
        try:
            if not lines and not self.strict:
                return []  # nothing happened: every spec still holds
            return self._run(lines)
        finally:
            self._cleanup_owned()

    def _cleanup_owned(self) -> None:
        if self._active:
            return
        for d in self._owned:
            shutil.rmtree(d, ignore_errors=True)
        self._owned.clear()

    def _files(self) -> List[str]:
        if self.spec_files is not None:
            files = [os.path.abspath(f) for f in self.spec_files]
        else:
            files = [os.path.abspath(f) for f in _config.load(os.getcwd()).files()]
        if not files:
            raise PrsError(
                "No P spec files to check: set spec_files under [tool.prs] in pyproject.toml, "
                "or pass spec_files=."
            )
        return files

    def _run(self, lines: List[str]) -> List[SpecViolation]:
        __tracebackhide__ = True
        files = self._files()
        workdir = tempfile.mkdtemp(prefix="prs-check-", dir=self._output_root)
        events_dir = os.path.join(workdir, "events")
        os.makedirs(events_dir)
        with open(os.path.join(events_dir, "events.jsonl"), "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
        out_dir = os.path.join(workdir, "out")
        report_path = os.path.join(workdir, "report.json")
        cmd = [find_prs(), "observe", "-pf", *files]
        if self.specs:
            cmd += ["--spec", *self.specs]
        else:
            cmd += ["--all-specs"]
            if self.exclude:
                cmd += ["--exclude-specs", *self.exclude]
        cmd += ["-l", events_dir, "-o", out_dir, "--json-report", report_path]
        if self.liveness:
            cmd.append("--check-hot")
        if self.coverage:
            cmd.append("--coverage")
        if self.strict:
            cmd.append("--strict")
        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=workdir)
        output = proc.stdout.decode("utf-8", errors="replace")
        with open(os.path.join(workdir, "prs-output.txt"), "w", encoding="utf-8") as f:
            f.write(" ".join(cmd) + "\n\n" + output)
        if self.coverage and os.path.isfile(report_path):
            with open(report_path, encoding="utf-8") as f:
                self.coverage_report = json.load(f)
            if any("coverage" not in spec for spec in self.coverage_report.get("specs", [])):
                raise PrsError("prs did not report monitor coverage; use a binary supporting observe --coverage")
        if proc.returncode == 0:
            shutil.rmtree(workdir, ignore_errors=True)
            return []
        if not os.path.isfile(report_path):
            raise PrsError(_command_error(output, workdir))
        with open(report_path, encoding="utf-8") as f:
            report = json.load(f)
        return self._violations_of(report, workdir)

    def _violations_of(self, report: dict, workdir: str) -> List[SpecViolation]:
        __tracebackhide__ = True
        violations: List[SpecViolation] = []
        bad_lines: Dict[str, str] = {}
        validation_errors: List[str] = []
        for s in report.get("specs", []):
            spec = s["spec"]
            for err in s.get("errors", []):
                kind = err.get("kind")
                if kind == "validation":
                    validation_errors.append(err.get("message", "Strict observation failed"))
                    continue
                if kind == "parser":
                    bad_lines.setdefault(err.get("message", ""), err.get("cause") or "")
                    continue
                if spec in self.ignored:
                    continue
                violations.append(_violation(spec, err, workdir))
        if bad_lines:
            raise PrsError(_payload_error(bad_lines, workdir))
        if validation_errors and not violations:
            raise PrsError("\n".join(validation_errors) + f"\nOutput: {workdir}")
        if not violations:
            shutil.rmtree(workdir, ignore_errors=True)
        return violations


def _violation(spec: str, err: dict, workdir: str) -> SpecViolation:
    message = err.get("message") or ""
    location = None
    liveness = False
    m = re.match(r"Assertion failure: (.+?:\d+):\d+ (.*)$", message, re.S)
    if m:
        location, message = _rel(m.group(1), workdir), m.group(2)
    else:
        m = re.match(r".*detected liveness bug in hot state '(.*)' at the end of program execution\.$", message)
        if m:
            liveness = True
            message = f"it ended in hot state '{m.group(1)}'"
        elif message in ("", "null"):
            message = err.get("kind", "error")
    events = [_event_of_line(e.get("log_line") or "{}") for e in err.get("events", [])]
    for ev, raw in zip(events, err.get("events", [])):
        ev.event = ev.event or raw.get("event", "")
    earlier = max(0, len(events) - REPORT_EVENTS)
    return SpecViolation(
        spec,
        message,
        err.get("key") or "",
        events[earlier:],
        location=location,
        liveness=liveness,
        earlier=earlier,
        output_dir=workdir,
    )


def _event_of_line(line: str) -> Event:
    try:
        d = json.loads(line)
    except ValueError:
        return Event(event="?", payload=line, key="", time_ns=0)
    if not isinstance(d, dict):
        return Event(event="?", payload=d, key="", time_ns=0)
    ev = Event.from_json(d)
    ts = d.get("timestamp")
    ev.time_ns = _parse_time(ts)
    return ev


def _parse_time(ts: Any) -> int:
    if isinstance(ts, int):
        return ts * 1_000_000  # prs reads integers as milliseconds
    if isinstance(ts, str):
        if ts.isdigit():
            return int(ts) * 1_000_000
        m = re.match(r"(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)(?:\.(\d+))?Z$", ts)
        if m:
            import calendar
            import time as _time

            secs = calendar.timegm(_time.strptime(m.group(1), "%Y-%m-%dT%H:%M:%S"))
            frac = (m.group(2) or "").ljust(9, "0")[:9]
            return secs * 1_000_000_000 + int(frac)
        try:
            from datetime import datetime

            return _emit.to_ns(datetime.fromisoformat(ts.replace("Z", "+00:00")))
        except ValueError:
            return 0
    return 0


def _event_from_dict(d: dict, src: str, what: str) -> Event:
    if not isinstance(d, dict) or not isinstance(d.get("event"), str):
        raise PrsError(f"{what} must return a dict with an 'event' name, or None; got {d!r}")
    d = dict(d)
    name = d.pop("event")
    key = d.pop("key", None)
    src = d.pop("src", None) or src
    if "at" in d:
        t = _emit.to_ns(d.pop("at"))
    elif "timestamp" in d:
        t = _parse_time(d.pop("timestamp"))
    else:
        raise PrsError(f"{what} must give each event a time ('at'), so events can be merged by time; got {d!r}")
    if "payload" in d:
        payload = d.pop("payload")
        if d:
            raise PrsError(f"{what}: give either 'payload' or payload fields, not both; got {d!r}")
    else:
        payload = d or None
    return Event(name, payload, _emit._key_str(key), t, False, src)


def events_from_dicts(events: Sequence[Any]) -> List[Event]:
    """Hand-written events for `check_events`: dicts like ``{"event": "eEnter",
    "payload": {"client": "c1"}}`` (``key`` and ``at`` optional) or ``Event`` objects.
    Events without a time are ordered as listed."""
    out: List[Event] = []
    for i, d in enumerate(events, 1):
        if isinstance(d, Event):
            out.append(d)
            continue
        d = dict(d)
        if "at" not in d and "timestamp" not in d:
            d["at"] = i
        d.setdefault("src", f"events[{i - 1}]")
        ev = _event_from_dict(d, f"events[{i - 1}]", what="check_events")
        ev.seq = i
        out.append(ev)
    return out


def _payload_text(p: Any) -> str:
    if p is None:
        return ""
    if isinstance(p, dict):
        return " ".join(f"{k}={_short(v)}" for k, v in p.items())
    return _short(p)


def _short(v: Any) -> str:
    s = v if isinstance(v, str) else json.dumps(v, default=str)
    return s if len(s) <= 40 else s[:37] + "..."


def _rel(path_loc: str, base: str) -> str:
    """`path:line` from prs (relative to `base`, where it ran), relative to the cwd."""
    path, _, line = path_loc.rpartition(":")
    path = os.path.normpath(os.path.join(base, path))
    try:
        rel = os.path.relpath(path)
        if not rel.startswith(".."):
            path = rel
    except ValueError:
        pass
    return f"{path}:{line}"


def _command_error(output: str, workdir: str) -> str:
    lines = output.splitlines()
    if lines and lines[0].startswith("Failed parsing"):
        msg = []
        for line in lines[1:]:
            if line.startswith("Usage:"):
                break
            msg.append(line)
        text = "\n".join(msg).strip()
    else:
        text = output.strip()
    return f"prs could not check the events:\n{text}\n\nprs output: {workdir}"


def _payload_error(bad: Dict[str, str], workdir: str) -> str:
    lines = ["Some events do not match their declaration in the P spec files:"]
    for line, cause in list(bad.items())[:10]:
        ev = _event_of_line(line)
        where = f" ({ev.src})" if ev.src else ""
        lines.append(f"  {ev.event}{where}: {cause or 'payload does not match'}")
        lines.append(f"    payload: {json.dumps(ev.payload, default=str)}")
    if len(bad) > 10:
        lines.append(f"  ... and {len(bad) - 10} more")
    lines.append("")
    lines.append(f"prs output: {workdir}")
    return "\n".join(lines)
