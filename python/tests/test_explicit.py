"""Explicit mode: check_spec, expect_violation, check_events."""

import os
import subprocess
import sys
import threading
from datetime import datetime, timezone

import pytest
from app.locks import LockServer
from conftest import SPEC

from prs import PrsError, Recorder, SpecViolation, check_events, check_spec, emit, expect_violation

FILES = [SPEC]


class Clock:
    def __init__(self):
        self.t = 0.0

    def __call__(self):
        return self.t


def test_correct_server_passes():
    clock = Clock()
    with check_spec("MutualExclusion", spec_files=FILES) as rec:
        server = LockServer(lease=1, clock=clock)
        server.acquire("c1")
        clock.t = 2
        server.acquire("c2")
    assert rec.events_named("eEnter") == [{"client": "c1"}, {"client": "c2"}]
    assert rec.count("eExit") == 1


def test_lease_bug_is_reported_on_the_with_line():
    clock = Clock()
    with pytest.raises(SpecViolation) as info:
        with check_spec("MutualExclusion", spec_files=FILES):
            server = LockServer(lease=1, clock=clock, buggy_expiry=True)
            server.acquire("c1")
            clock.t = 2
            server.acquire("c2")
    v = info.value
    assert isinstance(v, AssertionError)
    assert v.spec == "MutualExclusion"
    assert v.message == "c2 entered while c1 is inside"
    assert v.key == "L1"
    assert [e.event for e in v.events] == ["eEnter", "eEnter"]
    assert v.events[-1].payload == {"client": "c2"}
    assert v.events[-1].src.endswith("locks.py:27")
    assert v.location == os.path.relpath(SPEC) + ":19"
    text = str(v)
    assert text.startswith("P spec MutualExclusion was broken: c2 entered while c1 is inside")
    assert "<- broke the spec here" in text
    assert os.path.isdir(v.output_dir)


def test_all_specs_by_default_and_liveness_is_opt_in():
    server = LockServer()
    with check_spec(spec_files=FILES):
        server.acquire("c1")
        server.acquire("c2")  # queued: never granted
    with pytest.raises(SpecViolation) as info:
        with check_spec(spec_files=FILES, liveness=True):
            server.acquire("c3")
    assert info.value.spec == "RequestsGranted"
    assert info.value.liveness
    assert "ended in hot state 'Waiting'" in str(info.value)
    assert "<- events ended here" in str(info.value)


def test_keys_are_independent_streams():
    with check_spec("MutualExclusion", spec_files=FILES):
        emit("eEnter", key="L1", client="c1")
        emit("eEnter", key="L2", client="c2")


def test_expect_violation_on_emitted_events():
    with expect_violation("MutualExclusion", match="c2 entered", spec_files=FILES) as ev:
        emit("eEnter", client="c1")
        emit("eEnter", client="c2")
    assert ev.violation.message == "c2 entered while c1 is inside"


def test_expect_violation_fails_when_the_spec_holds():
    with pytest.raises(AssertionError, match="expected P spec MutualExclusion to be broken, but it held over 1 event"):
        with expect_violation("MutualExclusion", spec_files=FILES):
            emit("eEnter", client="c1")


def test_expect_violation_checks_the_message():
    with pytest.raises(AssertionError, match="does not match 'c3'"):
        with expect_violation("MutualExclusion", match="c3", spec_files=FILES):
            emit("eEnter", client="c1")
            emit("eEnter", client="c2")


def test_check_events_tests_the_spec_itself():
    with expect_violation("MutualExclusion", match="c2 entered", spec_files=FILES):
        check_events(
            [
                {"event": "eEnter", "payload": {"client": "c1"}},
                {"event": "eEnter", "payload": {"client": "c2"}},
            ],
            "MutualExclusion",
            spec_files=FILES,
        )
    check_events(
        [{"event": "eEnter", "client": "c1"}, {"event": "eExit", "client": "c1"}],
        "MutualExclusion",
        spec_files=FILES,
    )


def test_check_events_reports_list_positions():
    with pytest.raises(SpecViolation) as info:
        check_events(
            [{"event": "eEnter", "client": "c1"}, {"event": "eExit", "client": "c2"}],
            "MutualExclusion",
            spec_files=FILES,
        )
    assert info.value.events[-1].src == "events[1]"
    assert info.value.message == "c2 left, but c1 holds the lock"


def test_nested_recorders_both_see_events():
    with check_spec("MutualExclusion", spec_files=FILES) as outer:
        emit("eEnter", client="c1")
        emit("eExit", client="c1")
        with check_spec("MutualExclusion", spec_files=FILES) as inner:
            emit("eEnter", client="c2")
            emit("eExit", client="c2")
        assert inner.count("eEnter") == 1
    assert outer.count("eEnter") == 2 and outer.count("eExit") == 2


def test_at_overrides_the_time():
    t0 = datetime(2026, 10, 18, tzinfo=timezone.utc)
    with check_spec("MutualExclusion", spec_files=FILES) as rec:
        # logged out of order; times say exit came first
        emit("eEnter", client="c2", at=t0.replace(second=2))
        emit("eExit", client="c1", at=t0.replace(second=1))
        emit("eEnter", client="c1", at=t0)
    assert [e.event for e in rec.events] == ["eEnter", "eExit", "eEnter"]
    with pytest.raises(SpecViolation) as info:
        with check_spec("MutualExclusion", spec_files=FILES):
            emit("eEnter", client="c1", at=2)
            emit("eEnter", client="c2", at=1)  # a logical clock: c2 entered first
    assert info.value.message == "c1 entered while c2 is inside"
    assert "(time set with at=)" in str(info.value)


def test_payload_mismatch_is_a_clear_error():
    with pytest.raises(PrsError) as info:
        with check_spec("MutualExclusion", spec_files=FILES):
            emit("eEnter", who="c1")
    msg = str(info.value)
    assert "do not match their declaration" in msg
    assert "missing field 'client'" in msg
    assert "test_explicit.py" in msg


def test_unknown_spec_is_a_clear_error():
    with pytest.raises(PrsError, match="SPECIFICATION_SUPPLIER not found"):
        with check_spec("NoSuchSpec", spec_files=FILES):
            emit("eEnter", client="c1")


def test_no_events_skips_the_check():
    with check_spec("NoSuchSpec", spec_files=["nope.p"]):
        pass


def test_events_from_threads_are_kept():
    with expect_violation("MutualExclusion", spec_files=FILES):
        t = threading.Thread(target=lambda: emit("eEnter", client="t1"))
        t.start()
        t.join()
        emit("eEnter", client="main")


def test_events_from_child_processes(monkeypatch):
    code = "from prs import emit; emit('eEnter', key='L9', client='child')"
    monkeypatch.setenv("PYTHONPATH", os.pathsep.join(sys.path))
    with expect_violation("MutualExclusion", match="parent entered while child", spec_files=FILES) as ev:
        subprocess.run([sys.executable, "-c", code], check=True)  # inherits PRS_EVENTS_DIR
        emit("eEnter", key="L9", client="parent")
    assert ev.violation.events[0].src == "<string>:1"
    assert ev.violation.events[0].pid != os.getpid()


def test_existing_logs(tmp_path):
    log = tmp_path / "server.log"
    log.write_text(
        "2026-10-18T00:00:01Z INFO granted lock L1 to c1\n"
        "2026-10-18T00:00:02Z INFO heartbeat\n"
        "2026-10-18T00:00:03Z INFO granted lock L1 to c2\n"
    )

    def parse(line):
        parts = line.split()
        if parts[2:4] != ["granted", "lock"]:
            return None
        return {"event": "eEnter", "key": parts[4], "at": datetime.fromisoformat(parts[0].replace("Z", "+00:00")),
                "client": parts[6]}

    rec = Recorder(["MutualExclusion"], spec_files=FILES)
    rec.add_log(str(log), parse)
    assert rec.count("eEnter") == 2
    with pytest.raises(SpecViolation) as info:
        rec.check()
    assert info.value.events[-1].src == f"{log}:3"


def test_positional_payloads():
    with check_spec("MutualExclusion", spec_files=FILES) as rec:
        emit("eEnter", {"client": "c1"}, key="L1")
    assert rec.events_named("eEnter") == [{"client": "c1"}]
    with check_spec("MutualExclusion", spec_files=FILES), pytest.raises(TypeError):
        emit("eEnter", {"client": "c1"}, client="c1")


@pytest.mark.skipif(not hasattr(os, "fork"), reason="needs fork")
def test_events_from_forked_processes():
    import multiprocessing

    ctx = multiprocessing.get_context("fork")
    with expect_violation("MutualExclusion", match="parent entered while forked", spec_files=FILES):
        p = ctx.Process(target=emit, args=("eEnter",), kwargs={"key": "F", "client": "forked"})
        p.start()
        p.join()
        emit("eEnter", key="F", client="parent")


def test_strict_requires_events_without_changing_default():
    with check_spec("MutualExclusion", spec_files=FILES):
        pass
    with pytest.raises(PrsError, match="checked no relevant events"):
        with check_spec("MutualExclusion", spec_files=FILES, strict=True):
            pass
    with check_spec("MutualExclusion", spec_files=FILES, strict=True):
        emit("eEnter", client="a")
    with pytest.raises(PrsError, match="RequestsGranted.*checked no relevant events"):
        check_events([{"event": "eEnter", "payload": {"client": "a"}}], spec_files=FILES, strict=True)


def test_strict_keeps_sibling_safety_violation_primary():
    with pytest.raises(SpecViolation, match="entered while"):
        with check_spec(spec_files=FILES, strict=True):
            emit("eEnter", client="a")
            emit("eEnter", client="b")
