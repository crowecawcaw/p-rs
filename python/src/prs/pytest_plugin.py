"""Global mode: every test also checks the P specs against the events it emitted.

Loaded automatically when the package is installed (the ``pytest11`` entry point).
Configured under ``[tool.prs]`` in pyproject.toml::

    [tool.prs]
    spec_files = ["specs/*.p"]
    exclude = ["DraftSpec"]   # optional: specs not checked anywhere yet
    mode = "auto"             # optional: "auto" (every test) | "marked" | "off"

Per test: ``@pytest.mark.prs(skip=True)``, ``@pytest.mark.prs(skip=["DraftSpec"])``,
``@pytest.mark.prs(check_hot=True)``. The ``prs_events`` fixture gives the test's recorder.
"""

from __future__ import annotations

import contextlib
import json
import os
import subprocess
from pathlib import Path
from typing import Any, Generator, Iterator, List, Optional

import pytest

from . import _config
from ._binary import find_prs
from ._errors import SetupError
from ._recorder import Recorder

_RECORDER = pytest.StashKey[Optional[Recorder]]()
_CONFIG = pytest.StashKey[Optional[_config.Config]]()
_ERROR = pytest.StashKey[Optional[str]]()
_COVERAGE = pytest.StashKey[list]()
_COVERAGE_PATH = pytest.StashKey[Optional[Path]]()
_COVERAGE_SUMMARY = pytest.StashKey[tuple]()

MARK_KEYS = {"skip", "check_hot"}


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("prs", "checking P specs (p-rs)")
    group.addoption("--prs-strict", action="store_true", default=False,
                    help="Require relevant events for every selected P spec in each checked test.")
    group.addoption("--prs-coverage", metavar="PATH", default=None,
                    help="Print suite monitor coverage and merge it as JSON into PATH (or set PRS_COVERAGE).")
    group.addoption(
        "--prs",
        dest="prs_mode",
        choices=_config.MODES,
        default=None,
        help="which tests check the P specs: auto (all), marked, off. Overrides PRS_MODE and [tool.prs] mode.",
    )


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "prs(skip=False, check_hot=False): P spec checking for this test. skip=True turns it off, "
        "skip=[names] leaves out some specs, check_hot=True also fails on a spec left in a hot state.",
    )
    config.stash[_COVERAGE] = []
    config.stash[_ERROR] = None
    config.stash[_COVERAGE_PATH] = _coverage_path(config)
    try:
        cfg = _config.load(str(config.rootpath))
    except SetupError as e:
        config.stash[_CONFIG] = None
        config.stash[_ERROR] = str(e)
        return
    explicit = config.getoption("prs_mode") or os.environ.get(_config.MODE_ENV)
    if not cfg.spec_files and cfg.path is None and not explicit:
        cfg.mode = "off"  # no [tool.prs]: installed but not set up here
    else:
        cfg.mode = config.getoption("prs_mode") or cfg.mode or "auto"
    config.stash[_CONFIG] = cfg


def _coverage_path(config: pytest.Config) -> Optional[Path]:
    """`--prs-coverage` (relative to the rootdir), else `PRS_COVERAGE` (relative to the
    working directory)."""
    option = config.getoption("prs_coverage")
    if option:
        path = Path(option)
        return path if path.is_absolute() else config.rootpath / path
    env = os.environ.get(_config.COVERAGE_ENV)
    return Path(env).absolute() if env else None


def pytest_report_header(config: pytest.Config) -> Optional[str]:
    cfg = config.stash.get(_CONFIG, None)
    err = config.stash.get(_ERROR, None)
    if err:
        return f"prs: configuration error: {err}"
    if cfg is None or cfg.mode == "off":
        return None
    return f"prs: checking P specs from {', '.join(cfg.spec_files)} ({cfg.mode})"


def _artifacts_root(config: pytest.Config) -> Optional[str]:
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
    check_hot: Optional[bool] = None
    for m in reversed(marks):  # closest mark wins
        unknown = set(m.kwargs) - MARK_KEYS
        if unknown or m.args:
            raise pytest.UsageError(f"@pytest.mark.prs takes skip= and check_hot=; got {m.args or ''} {m.kwargs}")
        s = m.kwargs.get("skip", False)
        if s is True:
            return None
        if s:
            skip.extend([s] if isinstance(s, str) else s)
        if "check_hot" in m.kwargs:
            check_hot = bool(m.kwargs["check_hot"])
    specs = [s for s in cfg.specs if s not in skip]
    if cfg.specs and not specs:
        return None  # every configured spec skipped
    return {"specs": specs, "exclude": [*cfg.exclude, *skip], "check_hot": check_hot, "files": cfg.files(), "config": cfg}


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
        settings["specs"] if settings else (),
        spec_files=settings["files"] if settings else [],
        exclude=settings["exclude"] if settings else (),
        check_hot=settings["check_hot"] if settings else None,
        coverage=item.config.stash[_COVERAGE_PATH] is not None,
        strict=True if item.config.getoption("prs_strict") else None,
    )
    rec._cfg = settings["config"] if settings else None
    rec._default_artifacts = _artifacts_root(item.config)
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


@pytest.hookimpl(wrapper=True)
def pytest_runtest_teardown(item: pytest.Item) -> Generator[None, Any, Any]:
    rec = item.stash.get(_RECORDER, None)
    if rec is not None:  # setup failed before the test ran
        rec.stop()
        rec._cleanup_owned()
    result = yield
    if rec is not None and rec.late:
        # a thread started by the test emitted after the check (in teardown, say)
        raise rec.late[0]
    return result


@pytest.fixture
def prs_events(request: pytest.FixtureRequest) -> Recorder:
    """The recorder of this test: ``prs_events.count("eEnter")``,
    ``prs_events.events_named("eEnter")``, ``prs_events.add_log(path, parse)``,
    ``prs_events.env()`` for child processes."""
    rec = request.node.stash.get(_RECORDER, None)
    if rec is None:
        raise SetupError("prs_events: the prs plugin did not start a recorder for this test")
    return rec


@contextlib.contextmanager
def _locked(path: Path) -> Iterator[None]:
    """An exclusive lock on `path`.lock, held across processes (pytest-xdist workers,
    several pytest runs sharing PRS_COVERAGE)."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(f"{path}.lock", "a+b") as f:
        if os.name == "nt":
            import msvcrt

            while True:
                try:
                    f.seek(0)
                    msvcrt.locking(f.fileno(), msvcrt.LK_LOCK, 1)  # retries for 10 s
                    break
                except OSError:
                    continue
            try:
                yield
            finally:
                f.seek(0)
                msvcrt.locking(f.fileno(), msvcrt.LK_UNLCK, 1)
        else:
            import fcntl

            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)


@pytest.hookimpl(tryfirst=True)
def pytest_sessionfinish(session: pytest.Session, exitstatus: int) -> None:
    """Merge this run's coverage into the file in core, under a lock; report errors
    before pytest chooses its process exit code."""
    config = session.config
    path = config.stash.get(_COVERAGE_PATH, None)
    if path is None:
        return
    try:
        with _locked(path):
            reports = list(config.stash[_COVERAGE])
            if path.is_file() and path.stat().st_size:
                try:
                    reports.insert(0, json.loads(path.read_text(encoding="utf-8")))
                except ValueError as e:
                    raise SetupError(f"{path} is not a coverage JSON file ({e}); delete it to start over") from None
            result = subprocess.run(
                [find_prs(), "coverage", "merge-observe", "--stdin", "--json-report", str(path)],
                input=json.dumps(reports), text=True,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=60,
            )
        if result.returncode:
            raise SetupError(f"coverage merge exited {result.returncode}: {result.stdout}")
        config.stash[_COVERAGE_SUMMARY] = (True, result.stdout, path)
    except (SetupError, OSError, RuntimeError, subprocess.SubprocessError) as error:
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
