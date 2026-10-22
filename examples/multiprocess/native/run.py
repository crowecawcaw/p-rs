#!/usr/bin/env python3
"""Supervise two local inventory processes and one live P observer."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

COMMON = Path(__file__).resolve().parents[1] / "common"
sys.path.insert(0, str(COMMON))
from harness import run_scenario


class SubprocessBackend:
    """The launch/cleanup adapter to replace for your own service executables."""

    def start(self, name, database, events, broken, stderr):
        # The worker inherits the pipe's write end and opens it by its /dev/fd path.
        command = [sys.executable, "-u", str(COMMON / "worker.py"),
                   "--db", str(database), "--name", name, "--events", "/dev/fd/%d" % events]
        if broken:
            command.append("--allow-oversell")
        return subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                stderr=stderr, text=True, pass_fds=[events])

    def alive(self, name, process):
        return process.poll() is None

    def cleanup(self, name, process):
        if process.poll() is None:
            process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)

    def absent(self, name, process):
        return process.poll() is not None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snappea", default=os.environ.get("SNAPPEA_BIN", "snappea"),
                        help="checker binary (default: SNAPPEA_BIN or snappea on PATH)")
    parser.add_argument("--scenario", choices=("healthy", "oversell"), default="healthy")
    parser.add_argument("--artifacts", type=Path,
                        help="new output directory; existing directories are never overwritten")
    args = parser.parse_args()
    snappea_bin = shutil.which(args.snappea)
    if snappea_bin is None:
        parser.error("cannot find checker; build snappea and pass --snappea /absolute/path/to/snappea")
    artifacts = args.artifacts
    if artifacts is None:
        parent = Path(__file__).resolve().parent / "artifacts"
        parent.mkdir(exist_ok=True)
        artifacts = Path(tempfile.mkdtemp(prefix="run-", dir=parent)) / args.scenario
    result = run_scenario(SubprocessBackend(), str(Path(snappea_bin).resolve()), artifacts, args.scenario)
    # Preserve the checker's real status for an external test runner. The fault
    # scenario validates its exact assertion and cleanup before returning 1.
    return result["observer_process_exit"]


if __name__ == "__main__":
    raise SystemExit(main())
