"""Check P specs against events emitted by your code during its tests.

Application code calls :func:`emit`. Tests are checked either by the pytest plugin
(every test, configured under ``[tool.prs]`` in pyproject.toml) or explicitly with
:func:`check_spec`, :func:`expect_violation` and :func:`check_events`.
"""

from __future__ import annotations

import re
from typing import Any, Optional, Sequence

from ._binary import find_prs
from ._emit import Event, emit
from ._recorder import PrsError, Recorder, SpecViolation, events_from_dicts

# show `prs.SpecViolation` in tracebacks, not the private module
for _cls in (Recorder, Event, SpecViolation, PrsError):
    _cls.__module__ = "prs"

__all__ = [
    "emit",
    "check_spec",
    "expect_violation",
    "check_events",
    "Recorder",
    "Event",
    "SpecViolation",
    "PrsError",
    "find_prs",
]


class check_spec:
    """Checks the events emitted inside the block against P specs::

        with check_spec("MutualExclusion") as rec:
            server.acquire("c1")
            server.acquire("c2")

    No spec names means every spec in the spec files. Raises :class:`SpecViolation`
    (an ``AssertionError``) at the end of the block, so the test fails on this line.
    ``liveness=True`` also fails if a spec is left in a hot state.
    ``strict=True`` requires relevant events for every selected spec.
    """

    def __init__(self, *specs: str, spec_files: Optional[Sequence[str]] = None, liveness: bool = False, strict: bool = False):
        self.recorder = Recorder(specs, spec_files=spec_files, liveness=liveness, strict=strict)

    def __enter__(self) -> Recorder:
        return self.recorder.start()

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        __tracebackhide__ = True
        self.recorder.stop()
        if exc_type is None:
            self.recorder.check()
        else:
            self.recorder._cleanup_owned()


class expect_violation:
    """The block must break `spec`, like ``pytest.raises``::

        with expect_violation("MutualExclusion", match="c2 entered"):
            server.acquire("c1")
            server.acquire("c2")

    The block passes if a :class:`SpecViolation` of `spec` is raised inside it (by
    :func:`check_events` or a nested :func:`check_spec`) or if the events emitted inside
    it break the spec. ``match`` is a regular expression searched for in the spec's
    message. Enclosing checks, such as the pytest plugin's, ignore `spec` for the rest of
    the test. The violation is available as ``.violation`` afterwards.
    """

    def __init__(
        self,
        spec: str,
        match: Optional[str] = None,
        *,
        spec_files: Optional[Sequence[str]] = None,
        liveness: bool = False,
    ):
        self.spec = spec
        self.match = match
        self.recorder = Recorder([spec], spec_files=spec_files, liveness=liveness)
        self.violation: Optional[SpecViolation] = None

    def __enter__(self) -> "expect_violation":
        from . import _emit

        for outer in _emit._recorders:
            outer.ignored.add(self.spec)
        self.recorder.start()
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        __tracebackhide__ = True
        self.recorder.stop()
        found: Optional[SpecViolation] = None
        if exc_type is not None:
            if isinstance(exc, SpecViolation):
                found = next((v for v in [exc, *exc.others] if v.spec == self.spec), None)
            if found is None:
                self.recorder._cleanup_owned()
                return False  # some other failure: let it through
        else:
            found = next((v for v in self.recorder.violations() if v.spec == self.spec), None)
        if found is None:
            n = len(self.recorder.events)
            raise AssertionError(f"expected P spec {self.spec} to be broken, but it held over {n} event(s)")
        if self.match is not None and not re.search(self.match, found.message):
            raise AssertionError(
                f"P spec {self.spec} was broken, but its message {found.message!r} "
                f"does not match {self.match!r}"
            )
        self.violation = found
        return True


def check_events(
    events: Sequence[Any],
    *specs: str,
    spec_files: Optional[Sequence[str]] = None,
    liveness: bool = False,
    strict: bool = False,
) -> None:
    """Checks a hand-written list of events: unit tests for a spec itself::

        check_events([
            {"event": "eEnter", "payload": {"client": "c1"}},
            {"event": "eEnter", "payload": {"client": "c2"}},
        ], "MutualExclusion")

    Events take ``event`` and ``payload`` (or payload fields directly, as with
    :func:`emit`) and optionally ``key`` and ``at``; without ``at`` they are checked in
    list order. These events do not reach any enclosing check.
    """
    __tracebackhide__ = True
    rec = Recorder(specs, spec_files=spec_files, liveness=liveness, strict=strict)
    rec._events.extend(events_from_dicts(events))
    rec.check()
