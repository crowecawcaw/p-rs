"""Events from a non-Python process, written with the Go reference emitter."""

import os
import shutil
import subprocess

import pytest
from conftest import HERE, SPEC

from prs import emit, expect_violation

ROOT = os.path.dirname(os.path.dirname(HERE))  # the Go module root


@pytest.fixture(scope="module")
def go_service(tmp_path_factory):
    if shutil.which("go") is None:
        pytest.skip("go is not installed")
    exe = str(tmp_path_factory.mktemp("go") / "service")
    subprocess.run(["go", "build", "-o", exe, "./go/prsemit/example"], cwd=ROOT, check=True)
    return exe


def test_go_events_are_checked_with_python_events(go_service):
    with expect_violation("MutualExclusion", match="py entered while go is inside", spec_files=[SPEC]) as ev:
        subprocess.run([go_service, "go"], check=True)
        emit("eEnter", key="L1", client="py")
    first = ev.violation.events[0]
    assert first.payload == {"client": "go"}
    assert first.src == "main.go:15"
