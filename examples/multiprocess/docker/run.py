"""Run two inventory containers against a live host-side P observer."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid

COMMON = Path(__file__).resolve().parents[1] / "common"
sys.path.insert(0, str(COMMON))
from harness import run_scenario, TIMEOUT


class DockerBackend:
    def __init__(self, image):
        self.image = image
        self.prefix = "prs-inventory-" + uuid.uuid4().hex[:12] + "-"

    def container(self, name):
        return self.prefix + name

    def start(self, name, database, events, broken, stderr):
        # Docker forwards only a container's stdin, stdout and stderr, and a pipe cannot
        # cross into the VM that Docker Desktop and similar tools run containers in. So the
        # worker writes events to its stderr, which `docker run` copies into the pipe. A
        # crash traceback then reaches the observer, which fails the run with a parser error.
        args = ["docker", "run", "--interactive", "--name", self.container(name),
                "--network", "none", "--mount", "type=bind,src=%s,dst=/data" % database.parent,
                self.image, "--db", "/data/" + database.name, "--name", name, "--events", "/dev/stderr"]
        if broken:
            args.append("--allow-oversell")
        return subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=events, text=True)

    def alive(self, name, process):
        state = subprocess.run(["docker", "inspect", "--format", "{{.State.Running}}", self.container(name)],
                               capture_output=True, text=True, timeout=TIMEOUT)
        return process.poll() is None and state.returncode == 0 and state.stdout.strip() == "true"

    def cleanup(self, name, process):
        # Killing only `docker run` would risk orphaning its container. Remove the
        # named container first; also reap the CLI even if Docker cleanup fails.
        try:
            removed = subprocess.run(["docker", "rm", "--force", self.container(name)],
                                     capture_output=True, text=True, timeout=TIMEOUT)
            if removed.returncode and not self.absent(name, process):
                raise RuntimeError("container removal failed: " + removed.stderr)
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=TIMEOUT)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=TIMEOUT)

    def absent(self, name, process):
        # `docker inspect` errors can mean daemon failure, so establish daemon
        # health before accepting absence as evidence of successful cleanup.
        subprocess.run(["docker", "info", "--format", "{{.ServerVersion}}"],
                       check=True, capture_output=True, text=True, timeout=TIMEOUT)
        result = subprocess.run(["docker", "container", "ls", "--all", "--quiet", "--filter",
                                 "name=^/" + self.container(name) + "$"],
                                check=True, capture_output=True, text=True, timeout=TIMEOUT)
        return not result.stdout.strip() and process.poll() is not None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prs", default=os.environ.get("PRS_BIN", "prs"))
    parser.add_argument("--image", default="prs-inventory-example:local")
    parser.add_argument("--scenario", choices=("healthy", "oversell"), default="healthy")
    parser.add_argument("--artifacts", type=Path, default=None)
    args = parser.parse_args()
    if args.artifacts is None:
        args.artifacts = Path(tempfile.mkdtemp(prefix="prs-docker-")) / args.scenario
    try:
        result = run_scenario(DockerBackend(args.image), args.prs, args.artifacts, args.scenario)
    except Exception as exc:
        print("integration failed: " + str(exc), file=sys.stderr)
        return 2
    return result["observer_process_exit"]


if __name__ == "__main__":
    sys.exit(main())
