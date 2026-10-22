#!/usr/bin/env python3
"""Reproduce the frozen-v1 eCommit reachability-sentinel probe.

This research helper copies two checked-in model fixtures, inserts one unique
``assert false`` into the first ``on eCommit do`` handler, and runs both the
baseline and probe with the supplied historical Snappea binary. It does not
modify the fixtures or any feature code.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


HERE = Path(__file__).resolve().parent
FIXTURE_ROOT = HERE.parent / "private" / "calibration-p" / "reachability"
MARKER = "reachability sentinel: eCommit"
HANDLER_RE = re.compile(r"\bon\s+eCommit\s+do\b")
INVALID_RE = re.compile(
    r"(?:\[Parser Error:?\]|parse error|syntax error|type(?:checking)? error|"
    r"compilation error|failed to compile)",
    re.IGNORECASE,
)

FIXTURES = (
    {
        "name": "vacuous",
        "test": "tcAtomicBundlePublish",
        "expected": "not_reached_within_budget",
        "description": "luna-p-05 frozen-v1 model; BundleStore eCommit handler is unreachable",
    },
    {
        "name": "nonvacuous",
        "test": "tcAtomicPublish",
        "expected": "marker_hit",
        "description": "luna-p-03 frozen-v1 model; CommitChain observes Server eCommit",
    },
)


def _skip_trivia(source: str, pos: int) -> int:
    """Skip whitespace and P line/block comments from pos."""
    n = len(source)
    while pos < n:
        if source[pos].isspace():
            pos += 1
        elif source.startswith("//", pos):
            end = source.find("\n", pos + 2)
            pos = n if end < 0 else end + 1
        elif source.startswith("/*", pos):
            end = source.find("*/", pos + 2)
            if end < 0:
                raise ValueError("unterminated block comment in handler")
            pos = end + 2
        else:
            return pos
    return pos


def _statement_end(source: str, pos: int) -> int:
    """Find a declaration's semicolon, ignoring comments and quoted strings."""
    quote: str | None = None
    escaped = False
    while pos < len(source):
        char = source[pos]
        if quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            pos += 1
        elif source.startswith("//", pos):
            end = source.find("\n", pos + 2)
            pos = len(source) if end < 0 else end + 1
        elif source.startswith("/*", pos):
            end = source.find("*/", pos + 2)
            if end < 0:
                raise ValueError("unterminated block comment in local declaration")
            pos = end + 2
        elif char in ('"', "'"):
            quote = char
            pos += 1
        elif char == ";":
            return pos + 1
        else:
            pos += 1
    raise ValueError("local variable declaration has no terminating semicolon")


def _matching_brace(source: str, opening: int) -> int:
    """Return the matching closing brace, ignoring braces in comments/strings."""
    depth = 0
    pos = opening
    quote: str | None = None
    escaped = False
    while pos < len(source):
        char = source[pos]
        if quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            pos += 1
        elif source.startswith("//", pos):
            end = source.find("\n", pos + 2)
            pos = len(source) if end < 0 else end + 1
        elif source.startswith("/*", pos):
            end = source.find("*/", pos + 2)
            if end < 0:
                raise ValueError("unterminated block comment while finding handler")
            pos = end + 2
        elif char in ('"', "'"):
            quote = char
            pos += 1
        elif char == "{":
            depth += 1
            pos += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return pos
            pos += 1
        else:
            pos += 1
    raise ValueError("handler body has no matching closing brace")


def insert_handler_sentinel(source: str) -> tuple[str, int]:
    """Insert into the first eCommit handler after its leading local declarations."""
    match = HANDLER_RE.search(source)
    if not match:
        raise ValueError("fixture has no `on eCommit do` handler")
    opening = source.find("{", match.end())
    if opening < 0:
        raise ValueError("eCommit handler has no body")
    closing = _matching_brace(source, opening)

    # P requires local declarations to precede statements. Keep consuming the
    # handler's initial `var ...;` declarations, allowing comments between them.
    cursor = opening + 1
    while True:
        token = _skip_trivia(source, cursor)
        if re.match(r"var\b", source[token:closing]):
            cursor = _statement_end(source, token)
            if cursor > closing:
                raise ValueError("local declaration extends beyond handler body")
        else:
            break

    indent_start = source.rfind("\n", 0, cursor) + 1
    line_prefix = source[indent_start:cursor]
    indent = re.match(r"[ \t]*", line_prefix).group(0)  # type: ignore[union-attr]
    insertion = f'\n{indent}assert false, "{MARKER}";'
    return source[:cursor] + insertion + source[cursor:], source.count("\n", 0, cursor) + 2


def classify(result: subprocess.CompletedProcess[str], timeout: bool = False) -> str:
    output = result.stdout + "\n" + result.stderr
    if timeout:
        return "timeout"
    if result.returncode == 0:
        return "pass"
    if INVALID_RE.search(output):
        return "invalid"
    return "other_failure"


def run_check(
    binary: Path, source: Path, testcase: str, output_dir: Path, schedules: int, timeout_sec: int
) -> dict[str, Any]:
    command = [
        str(binary),
        "check",
        str(source),
        testcase,
        "-n",
        str(schedules),
        "-j",
        "1",
        "-o",
        str(output_dir),
    ]
    try:
        result = subprocess.run(command, text=True, capture_output=True, timeout=timeout_sec, check=False)
        status = classify(result)
        return {
            "command": command,
            "returncode": result.returncode,
            "status": status,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout.decode(errors="replace") if isinstance(exc.stdout, bytes) else (exc.stdout or "")
        stderr = exc.stderr.decode(errors="replace") if isinstance(exc.stderr, bytes) else (exc.stderr or "")
        return {
            "command": command,
            "returncode": None,
            "status": "timeout",
            "stdout": stdout,
            "stderr": stderr,
        }


def probe_fixture(
    binary: Path, output_dir: Path, fixture: dict[str, str], schedules: int, timeout_sec: int
) -> dict[str, Any]:
    name = fixture["name"]
    original = FIXTURE_ROOT / f"{name}.p"
    if not original.is_file():
        raise FileNotFoundError(f"missing frozen fixture: {original}")

    target = output_dir / name
    target.mkdir(parents=True, exist_ok=True)
    baseline_source = target / "baseline.p"
    sentinel_source = target / "sentinel.p"
    shutil.copyfile(original, baseline_source)
    original_text = original.read_text(encoding="utf-8")
    mutated, insertion_line = insert_handler_sentinel(original_text)
    sentinel_source.write_text(mutated, encoding="utf-8")

    baseline = run_check(binary, baseline_source, fixture["test"], target / "baseline-output", schedules, timeout_sec)
    sentinel = run_check(binary, sentinel_source, fixture["test"], target / "sentinel-output", schedules, timeout_sec)
    marker_hit = MARKER in sentinel["stdout"] or MARKER in sentinel["stderr"]
    # Snappea may write the assertion details only to the generated bug report.
    bug_text = target / "sentinel-output" / fixture["test"] / "bug.txt"
    if bug_text.is_file():
        marker_hit = marker_hit or MARKER in bug_text.read_text(encoding="utf-8", errors="replace")
    marker_hit = marker_hit and sentinel['returncode'] == 1 and sentinel['status'] == 'other_failure'
    if marker_hit:
        sentinel["status"] = "marker_hit"

    if baseline["status"] != "pass":
        outcome = "baseline_" + baseline["status"]
    elif sentinel["status"] == "invalid":
        outcome = "invalid"
    elif sentinel["status"] == "timeout":
        outcome = "timeout"
    elif marker_hit:
        outcome = "marker_hit"
    elif sentinel["status"] == "pass":
        outcome = "not_reached_within_budget"
    else:
        outcome = "other_failure"

    return {
        "fixture": name,
        "description": fixture["description"],
        "expected_outcome": fixture["expected"],
        "testcase": fixture["test"],
        "event": "eCommit",
        "source": str(original),
        "source_sha256": hashlib.sha256(original.read_bytes()).hexdigest(),
        "insertion_line": insertion_line,
        "marker": MARKER,
        "schedules": schedules,
        "baseline": baseline,
        "sentinel_run": sentinel,
        "marker_hit": marker_hit,
        "outcome": outcome,
        "replay_schedule": str(bug_text.with_name("bug.schedule")) if marker_hit else None,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path, help="path to historical snappea executable")
    parser.add_argument("--output-dir", required=True, type=Path, help="directory for copied sources, logs, traces, and JSON")
    parser.add_argument("--schedules", type=int, default=100, help="schedule bound for each run (default: 100)")
    parser.add_argument("--timeout", type=int, default=30, help="per-run timeout in seconds (default: 30)")
    args = parser.parse_args()

    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"binary does not exist: {binary}")
    if args.schedules < 1 or args.timeout < 1:
        parser.error("--schedules and --timeout must be positive")
    out = args.output_dir.resolve()
    if out.exists() and (not out.is_dir() or any(out.iterdir())):
        parser.error('--output-dir must be new or empty; choose a fresh directory to avoid stale bug artifacts')
    out.mkdir(parents=True, exist_ok=True)

    results = [probe_fixture(binary, out, fixture, args.schedules, args.timeout) for fixture in FIXTURES]
    document = {
        "schema_version": 1,
        "binary": str(binary),
        "schedules": args.schedules,
        "timeout_seconds_per_run": args.timeout,
        "results": results,
    }
    json_path = out / "reachability-probe.json"
    json_path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(document, indent=2))
    print(f"JSON: {json_path}", file=sys.stderr)
    return 0 if all(item["outcome"] == item["expected_outcome"] for item in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
