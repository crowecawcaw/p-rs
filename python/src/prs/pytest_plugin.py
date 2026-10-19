"""Global mode: every test also checks the P specs against the events it emitted.

Loaded automatically when the package is installed (the ``pytest11`` entry point).
Configured under ``[tool.prs]`` in pyproject.toml::

    [tool.prs]
    spec_files = ["specs/*.p"]
    exclude = ["DraftSpec"]   # optional: specs not checked anywhere yet
    mode = "auto"             # optional: "auto" (every test) | "marked" | "off"

Per test: ``@pytest.mark.prs(skip=True)``, ``@pytest.mark.prs(skip=["DraftSpec"])``,
``@pytest.mark.prs(liveness=True)``. The ``prs_events`` fixture gives the test's recorder.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from typing import Any, Generator, List, Optional

import pytest

from ._binary import find_prs
from . import _config
from ._recorder import PrsError, Recorder

_RECORDER = pytest.StashKey[Optional[Recorder]]()
_CONFIG = pytest.StashKey[Optional[_config.Config]]()
_ERROR = pytest.StashKey[Optional[str]]()
_COVERAGE = pytest.StashKey[list]()
_COVERAGE_SUMMARY = pytest.StashKey[tuple]()


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("prs", "checking P specs (p-rs)")
    group.addoption("--prs-strict", action="store_true", default=False,
                    help="Require relevant events for every selected P spec in each checked test.")
    group.addoption("--prs-coverage", metavar="PATH", default=None,
                    help="Print suite monitor coverage and write its union as JSON to PATH.")
    group.addoption(
        "--prs",
        dest="prs_mode",
        choices=_config.MODES,
        default=None,
        help="which tests check the P specs: auto (all), marked, off. Overrides [tool.prs] mode.",
    )


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "prs(skip=False, liveness=False): P spec checking for this test. skip=True turns it off, "
        "skip=[names] leaves out some specs, liveness=True also fails on a spec left in a hot state.",
    )
    config.stash[_COVERAGE] = []
    config.stash[_ERROR] = None
    try:
        cfg = _config.load(str(config.rootpath))
    except _config.ConfigError as e:
        config.stash[_CONFIG] = None
        config.stash[_ERROR] = str(e)
        return
    mode = config.getoption("prs_mode") or cfg.mode
    if not cfg.spec_files and cfg.path is None and config.getoption("prs_mode") is None:
        mode = "off"  # no [tool.prs]: installed but not set up here
    cfg.mode = mode
    config.stash[_CONFIG] = cfg


def pytest_report_header(config: pytest.Config) -> Optional[str]:
    cfg = config.stash.get(_CONFIG, None)
    err = config.stash.get(_ERROR, None)
    if err:
        return f"prs: configuration error: {err}"
    if cfg is None or cfg.mode == "off":
        return None
    return f"prs: checking P specs from {', '.join(cfg.spec_files)} ({cfg.mode})"


def _output_root(config: pytest.Config) -> Optional[str]:
    """pytest's temporary folder for this run (kept for the last few runs), so prs's
    output for a failed test is easy to find; None falls back to the system's."""
    factory = getattr(config, "_tmp_path_factory", None)
    if factory is None:
        return None
    try:
        d = factory.getbasetemp() / "prs"
        d.mkdir(exist_ok=True)
        return str(d)
    except OSError:
        return None


def _settings(item: pytest.Item) -> Optional[dict]:
    """How to check `item`, or None to not check it."""
    cfg = item.config.stash.get(_CONFIG, None)
    if cfg is None or cfg.mode == "off":
        return None
    marks = list(item.iter_markers("prs"))
    if cfg.mode == "marked" and not marks:
        return None
    skip: List[str] = []
    liveness = False
    for m in reversed(marks):  # closest mark wins
        unknown = set(m.kwargs) - {"skip", "liveness"}
        if unknown or m.args:
            raise pytest.UsageError(f"@pytest.mark.prs takes skip= and liveness=; got {m.args or ''} {m.kwargs}")
        s = m.kwargs.get("skip", False)
        if s is True:
            return None
        if s:
            skip.extend([s] if isinstance(s, str) else s)
        if "liveness" in m.kwargs:
            liveness = bool(m.kwargs["liveness"])
    return {"exclude": [*cfg.exclude, *skip], "liveness": liveness, "files": cfg.files(), "strict": cfg.strict}


@pytest.hookimpl(tryfirst=True)
def pytest_runtest_setup(item: pytest.Item) -> None:
    """Starts recording before fixtures run, so events from setup count too."""
    err = item.config.stash.get(_ERROR, None)
    if err:
        raise pytest.UsageError(f"[tool.prs]: {err}")
    item.stash[_RECORDER] = None
    settings = _settings(item)
    wants_fixture = "prs_events" in getattr(item, "fixturenames", ())
    if settings is None and not wants_fixture:
        return
    rec = Recorder(
        spec_files=settings["files"] if settings else [],
        exclude=settings["exclude"] if settings else (),
        liveness=settings["liveness"] if settings else False,
        output_root=_output_root(item.config),
        coverage=bool(item.config.getoption("prs_coverage")),
        strict=bool(item.config.getoption("prs_strict")) or bool(settings and settings["strict"]),
    )
    rec._plugin_checks = settings is not None  # type: ignore[attr-defined]
    item.stash[_RECORDER] = rec.start()


@pytest.hookimpl(wrapper=True)
def pytest_runtest_call(item: pytest.Item) -> Generator[None, Any, Any]:
    """Checks the test's events when the test body is done, so a violation fails the
    test itself rather than its teardown."""
    try:
        result = yield
    except BaseException:
        rec = item.stash.get(_RECORDER, None)
        if rec is not None:
            rec.stop()
            rec._cleanup_owned()
        raise
    rec = item.stash.get(_RECORDER, None)
    if rec is not None:
        rec.stop()
        if getattr(rec, "_plugin_checks", False):
            __tracebackhide__ = True
            try:
                rec.check()
            finally:
                if rec.coverage_report is not None:
                    item.config.stash[_COVERAGE].append(rec.coverage_report)
        else:
            rec._cleanup_owned()
    return result


def pytest_runtest_teardown(item: pytest.Item) -> None:
    rec = item.stash.get(_RECORDER, None)
    if rec is not None:  # setup failed before the test ran
        rec.stop()
        rec._cleanup_owned()


@pytest.fixture
def prs_events(request: pytest.FixtureRequest) -> Recorder:
    """The recorder of this test: ``prs_events.count("eEnter")``,
    ``prs_events.events_named("eEnter")``, ``prs_events.add_log(path, parse)``."""
    rec = request.node.stash.get(_RECORDER, None)
    if rec is None:
        raise PrsError("prs_events: the prs plugin did not start a recorder for this test")
    return rec


@pytest.hookimpl(tryfirst=True)
def pytest_sessionfinish(session: pytest.Session, exitstatus: int) -> None:
    """Merge once in core; report errors before pytest chooses its process exit code."""
    config = session.config
    destination = config.getoption("prs_coverage")
    if not destination:
        return
    path = Path(destination)
    if not path.is_absolute():
        path = config.rootpath / path
    try:
        result = subprocess.run(
            [find_prs(), "coverage", "merge-observe", "--stdin", "--json-report", str(path)],
            input=json.dumps(config.stash[_COVERAGE]), text=True,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=30,
        )
        if result.returncode:
            raise PrsError(f"coverage merge exited {result.returncode}: {result.stdout}")
        config.stash[_COVERAGE_SUMMARY] = (True, result.stdout, path)
    except (PrsError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        config.stash[_COVERAGE_SUMMARY] = (False, str(error), path)
        session.exitstatus = pytest.ExitCode.TESTS_FAILED


def pytest_terminal_summary(terminalreporter: Any, exitstatus: int, config: pytest.Config) -> None:
    summary = config.stash.get(_COVERAGE_SUMMARY, None)
    if summary is None:
        return
    success, output, path = summary
    terminalreporter.section("Monitor coverage (union of completed test checks)")
    for line in output.splitlines():
        terminalreporter.write_line(line)
    if success:
        terminalreporter.write_line(f"JSON: {path}")
    else:
        terminalreporter.write_line("Monitor coverage reporting failed", red=True)
