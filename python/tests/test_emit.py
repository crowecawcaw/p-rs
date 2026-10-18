import json
import os
from datetime import datetime, timedelta, timezone

import pytest

from prs import _emit, emit
from prs._recorder import _parse_time


def test_emit_does_nothing_without_a_recorder(monkeypatch, tmp_path):
    monkeypatch.delenv("PRS_EVENTS_DIR", raising=False)
    emit("eAnything", x=1)
    assert list(tmp_path.iterdir()) == []


def test_emit_writes_to_the_events_dir(monkeypatch, tmp_path):
    monkeypatch.setenv("PRS_EVENTS_DIR", str(tmp_path))
    emit("eThing", key=7, x={1, 2}, at=5)
    (f,) = tmp_path.iterdir()
    assert f.name == f"events-{os.getpid()}.jsonl"
    d = json.loads(f.read_text())
    assert d["key"] == "7"
    assert d["timestamp"] == "1970-01-01T00:00:00.000000005Z"
    assert d["event"] == "eThing"
    assert sorted(d["payload"]["x"]) == [1, 2]
    assert d["at"] is True
    assert d["src"].endswith("test_emit.py:19")
    _emit._file.close()
    _emit._file = None


def test_timestamps_round_trip():
    t = datetime(2026, 10, 18, 1, 2, 3, 456789, tzinfo=timezone.utc)
    ns = _emit.to_ns(t)
    assert _emit.iso_timestamp(ns) == "2026-10-18T01:02:03.456789000Z"
    assert _parse_time(_emit.iso_timestamp(ns + 1)) == ns + 1
    assert _emit.to_ns(t.astimezone(timezone(timedelta(hours=5)))) == ns
    with pytest.raises(TypeError, match="float"):
        _emit.to_ns(1.5)


def test_clock_never_goes_back():
    a = _emit._make_event("e", None, None, None, None)
    b = _emit._make_event("e", None, None, None, None)
    assert b.time_ns >= a.time_ns and b.seq > a.seq
