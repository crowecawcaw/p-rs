"""Exercise both runnable scenarios, including the external runner's exit code."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent


class InventoryProcessesTest(unittest.TestCase):
    def run_scenario(self, scenario, expected_status):
        parent = Path(os.environ.get("MULTIPROCESS_ARTIFACTS", HERE / "artifacts"))
        parent.mkdir(parents=True, exist_ok=True)
        artifacts = Path(tempfile.mkdtemp(prefix="test-", dir=parent)) / scenario
        completed = subprocess.run(
            [sys.executable, str(HERE / "run.py"), "--scenario", scenario,
             "--artifacts", str(artifacts)],
            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=90,
        )
        print(completed.stdout, end="", flush=True)
        self.assertEqual(completed.returncode, expected_status, completed.stdout)
        # A crash also returns 1. Require the summary written only after the
        # harness verifies the exact violation and successful worker cleanup.
        result = json.loads((artifacts / "result.json").read_text())
        self.assertTrue(result["workers_removed"])
        self.assertEqual(result["emitters"], ["worker-a", "worker-b"])
        self.assertEqual(result["workers_alive_at_failure"], scenario == "oversell")
        self.assertEqual(result["input_complete"], scenario == "healthy")

    def test_healthy(self):
        self.run_scenario("healthy", 0)

    def test_oversell_fails_before_workers_exit(self):
        self.run_scenario("oversell", 1)


if __name__ == "__main__":
    unittest.main()
