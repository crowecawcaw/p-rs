#!/usr/bin/env python3
"""Verify an eval image, including source plugin discovery and checker use."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import tempfile

DEFAULT_P_IMAGE = "snappea-eval:py312-pytest842-pysnappea-source-0.1.0"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", default=DEFAULT_P_IMAGE)
    parser.add_argument(
        "--checker", type=Path,
        default=Path(__file__).resolve().parents[3] / 'target' / 'release' / 'snappea',
        help="host checker binary mounted into the container at /opt/snappea",
    )
    args = parser.parse_args()
    checker = args.checker.resolve()
    if not checker.is_file():
        parser.error(f"checker binary not found: {checker}")

    with tempfile.TemporaryDirectory(prefix="snappea-image-smoke-") as temp:
        project = Path(temp)
        (project / "smoke-positive.p").write_text(
            (Path(__file__).resolve().parent / "container" / "smoke-positive.p").read_text()
        )
        (project / "test_smoke.py").write_text(
            "from snappea import check_spec, emit, expect_violation\n"
            "\ndef test_positive_check_spec():\n"
            "    with check_spec('Positive', spec_files=['smoke-positive.p']):\n"
            "        emit('eAllowed', value=1)\n"
            "\ndef test_expected_negative_check_spec():\n"
            "    with expect_violation('Positive', spec_files=['smoke-positive.p'], match='value must be positive'):\n"
            "        with check_spec('Positive', spec_files=['smoke-positive.p']):\n"
            "            emit('eAllowed', value=0)\n"
        )

        command = [
            "docker", "run", "--rm", "--network", "none",
            "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
            "--user", f"{os.getuid()}:{os.getgid()}", "--read-only",
            "--tmpfs", "/tmp:rw,size=128m", "--memory", "768m",
            "--mount", f"type=bind,src={checker},dst=/opt/snappea,readonly",
            "--mount", f"type=bind,src={project},dst=/workspace/smoke,readonly",
            "--workdir", "/workspace/smoke",
            "--env", "PATH=/opt:/usr/local/bin:/usr/bin:/bin",
            args.image, "/bin/sh", "-ec",
            "python -c \"import importlib.metadata as m, snappea; eps=m.entry_points(group='pytest11'); "
            "assert any(e.name == 'snappea' and e.value == 'snappea.pytest_plugin' for e in eps); "
            "print('snappea import:', snappea.__file__); print('pytest entry point: registered')\" "
            "&& pytest --help | grep -q -- '--snappea' "
            "&& python -m pytest -q -p no:cacheprovider",
        ]
        subprocess.run(command, check=True)
    print(f"Smoke passed for {args.image}; checker mounted read-only from {checker}")


if __name__ == "__main__":
    main()
