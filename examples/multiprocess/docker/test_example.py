"""Exercise actual containers; an intentional fault must fail before producer EOF."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent


class InventoryContainers(unittest.TestCase):
    def test_healthy_and_overselling_workers(self):
        root = Path(os.environ.get("EXAMPLE_ARTIFACTS") or tempfile.mkdtemp(prefix="prs-docker-"))
        for scenario, expected_exit in (("healthy", 0), ("oversell", 1)):
            with self.subTest(scenario=scenario):
                artifacts = root / scenario
                command = [sys.executable, str(ROOT / "run.py"), "--scenario", scenario,
                           "--artifacts", str(artifacts)]
                completed = subprocess.run(command, text=True, capture_output=True, timeout=180)
                print(completed.stdout, end="")
                print(completed.stderr, end="", file=sys.stderr)
                self.assertEqual(completed.returncode, expected_exit)
                result = json.loads((artifacts / "result.json").read_text())
                self.assertTrue(result["workers_removed"])
                self.assertEqual(result["emitters"], ["worker-a", "worker-b"])
                self.assertEqual(result["input_complete"], scenario == "healthy")
                self.assertEqual(result["workers_alive_at_failure"], scenario == "oversell")
                self.assertEqual(result["observer_process_exit"], expected_exit)


if __name__ == "__main__":
    unittest.main()
