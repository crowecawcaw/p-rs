#!/usr/bin/env python3
"""Build pinned, offline Docker images for the certified-bundle eval."""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import os
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "tools"
REPO = ROOT.parents[1]
VENDOR = ROOT / "vendor" / "image-build"
PYTHON_IMAGE = (
    "python:3.12-slim@sha256:"
    "cb33f7f771ecec2ef1225cf39b159591a040391bd73e4c01f00fdfc4c47611d2"
)


def wheel_record_hash(data: bytes) -> str:
    return "sha256=" + base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()


def make_source_wheel(destination: Path) -> None:
    """Package the checkout's Python modules without maturin or a Rust binary."""
    package = REPO / "python" / "src" / "snappea"
    dist_info = "pysnappea-0.1.0.dist-info"
    entries: dict[str, bytes] = {}
    for path in sorted(package.rglob("*")):
        if path.is_file() and "__pycache__" not in path.parts:
            entries[f"snappea/{path.relative_to(package).as_posix()}"] = path.read_bytes()
    entries[f"{dist_info}/METADATA"] = (
        "Metadata-Version: 2.1\n"
        "Name: pysnappea\nVersion: 0.1.0\n"
        "Summary: Check P specs against events from Python tests\n"
        "Requires-Python: >=3.9\n"
        "Classifier: Framework :: Pytest\n"
        "Classifier: Programming Language :: Python :: 3\n\n"
    ).encode()
    entries[f"{dist_info}/WHEEL"] = (
        "Wheel-Version: 1.0\nGenerator: build_eval_images.py\n"
        "Root-Is-Purelib: true\nTag: py3-none-any\n"
    ).encode()
    entries[f"{dist_info}/entry_points.txt"] = b"[pytest11]\nsnappea = snappea.pytest_plugin\n"
    record_path = f"{dist_info}/RECORD"
    out = io.StringIO(newline="")
    writer = csv.writer(out, lineterminator="\n")
    for name, data in [*entries.items(), (record_path, b"")]:
        writer.writerow((name, wheel_record_hash(data) if data else "", str(len(data)) if data else ""))
    entries[record_path] = out.getvalue().encode()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as wheel:
        for name, data in entries.items():
            wheel.writestr(name, data)


def run(*args: str) -> str:
    completed = subprocess.run(args, check=True, text=True, stdout=subprocess.PIPE)
    return completed.stdout.strip()


def image_id(tag: str) -> str:
    return run("docker", "image", "inspect", tag, "--format", "{{.Id}}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--wheelhouse", type=Path,
        default=Path(os.environ.get("SNAPPEA_EVAL_WHEELHOUSE", str(ROOT / 'vendor' / 'wheels'))),
        help="directory containing pytest==8.4.2 and its wheels",
    )
    args = parser.parse_args()
    wheelhouse = args.wheelhouse.resolve()
    if not wheelhouse.is_dir():
        parser.error(f"wheelhouse does not exist: {wheelhouse}")
    required = {"pytest-8.4.2-py3-none-any.whl", "pluggy-1.7.0-py3-none-any.whl",
                "iniconfig-2.3.1-py3-none-any.whl", "packaging-26.3-py3-none-any.whl",
                "pygments-2.21.0-py3-none-any.whl"}
    missing = sorted(name for name in required if not (wheelhouse / name).is_file())
    if missing:
        parser.error("wheelhouse is missing: " + ", ".join(missing))

    base_context = VENDOR / "base-context"
    p_context = VENDOR / "p-context"
    shutil.rmtree(VENDOR, ignore_errors=True)
    (base_context / "wheels").mkdir(parents=True)
    shutil.copy2(TOOLS / "container" / "Dockerfile.base", base_context / "Dockerfile")
    for name in sorted(required):
        shutil.copy2(wheelhouse / name, base_context / "wheels" / name)
    p_context.mkdir(parents=True)
    shutil.copy2(TOOLS / "container" / "Dockerfile.p", p_context / "Dockerfile")
    wheel = p_context / "pysnappea-0.1.0-py3-none-any.whl"
    make_source_wheel(wheel)

    base_tag = "snappea-eval:py312-pytest842-base"
    p_tag = "snappea-eval:py312-pytest842-pysnappea-source-0.1.0"
    run("docker", "build", "--network=none", "--tag", base_tag,
        "--build-arg", f"PYTHON_IMAGE={PYTHON_IMAGE}", str(base_context))
    base_id = image_id(base_tag)
    run("docker", "build", "--network=none", "--tag", p_tag,
        "--build-arg", f"BASE_IMAGE={base_tag}", str(p_context))
    p_id = image_id(p_tag)
    print(f"BASE_IMAGE_TAG={base_tag}\nBASE_IMAGE_ID={base_id}")
    print(f"P_IMAGE_TAG={p_tag}\nP_IMAGE_ID={p_id}")
    print(f"PYTHON_WHEEL_SHA256={hashlib.sha256(wheel.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
