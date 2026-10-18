#!/usr/bin/env python3
"""Compares `p observe` with upstream PObserve on upstream's PObserve regression logs.

    UPSTREAM_POBSERVE="java -cp PObserveCommons-1.0.0.jar:PObserve-1.0.0.jar pobserve.PObserve" \
    UPSTREAM_JARS=regression.jar \
    PRS_OBSERVE_BIN=target/release/examples/observe_regression \
    scripts/compare-pobserve.py <P checkout>/Src/PObserve/PObserveRegressionTesting/src [filter]

UPSTREAM_JARS is a jar built from the regression project (its generated spec classes plus
the TestCase parsers). PRS_OBSERVE_BIN is `p observe` with the Rust ports of those parsers
(`cargo build --release -p prs-cli --example observe_regression`).

Every log file and log directory of the happy and error suites runs through both tools,
with and without --assumeInputsAreSorted. Compared: exit code, console output, and every
output file (replay logs, parser/unknown error files, metrics). Normalized away: colors,
output paths, the jar/project configuration line, timings, and Java stack-trace frames
(upstream checks keys in parallel, so error blocks are compared as a set).
"""

import os
import re
import shlex
import subprocess
import sys
import tempfile

ANSI = re.compile(r"\x1b\[[0-9;]*m")


def params(src, case):
    p = {}
    with open(os.path.join(src, "resources/logs/params", f"test_case_{case}_params.txt")) as f:
        for line in f:
            if ":" in line:
                k, v = line.split(":", 1)
                p[k.strip()] = v.strip()
    return p


def normalize_console(text, outdir):
    text = ANSI.sub("", text).replace(outdir, "<out>")
    lines = []
    for l in text.split("\n"):
        if re.match(r"  (outputDir|supplierJars|pproj|pfiles) +=", l):
            continue
        l = re.sub(r"Total time taken = \S+ min", "Total time taken = <t> min", l)
        lines.append(l)
    text = "\n".join(lines)
    # error blocks between rules, compared as a set
    rule = "-------------------------------------\n"
    m = re.search(r"(?s)(-------------------------------------\n.*-------------------------------------\n)\n?Total time", text)
    if m:
        body = m.group(1)
        blocks = [b for b in body.split(rule) if b.strip()]
        blocks = sorted(re.sub(r"(UnknownError|ParserError)_\d+", r"\1_N", b) for b in blocks)
        text = text[: m.start(1)] + "".join(rule + b + rule for b in blocks) + text[m.end(1):]
    return text


def normalize_file(name, text):
    if name == "PObserveMetrics.txt":
        text = re.sub(r"Total time taken = \S+ min", "Total time taken = <t> min", text)
        out = []
        for l in text.split("\n"):
            # per-event rows: name, verified count, avg time, max time
            m = re.match(r"^([A-Za-z_]\S* +\d+) +[\d.]+ +\d+ *$", l)
            if m:
                l = m.group(1) + " <avg> <max>"
            out.append(l)
        return "\n".join(out)
    if name.startswith(("UnknownError", "ParserError")):
        # drop Java stack frames
        return "\n".join(l for l in text.split("\n") if not l.startswith("\tat ") and not l.startswith("\t... "))
    return text


def run(cmd, cwd):
    p = subprocess.run(cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    return p.returncode, p.stdout


def read_outputs(d):
    files = {}
    for n in sorted(os.listdir(d)):
        with open(os.path.join(d, n), errors="replace") as f:
            files[n] = f.read()
    # numbering of error files follows the (parallel) report order upstream
    for prefix in ("UnknownError_", "ParserError_"):
        group = sorted(v for k, v in files.items() if k.startswith(prefix))
        for k in [k for k in files if k.startswith(prefix)]:
            del files[k]
        for i, v in enumerate(group):
            files[f"{prefix}{i}.txt"] = v
    return files


def main():
    src = os.path.abspath(sys.argv[1])
    filt = sys.argv[2] if len(sys.argv) > 2 else ""
    up = shlex.split(os.environ["UPSTREAM_POBSERVE"])
    jars = os.path.abspath(os.environ["UPSTREAM_JARS"])
    pn = [os.path.abspath(os.environ.get("PRS_OBSERVE_BIN", "target/release/examples/observe_regression"))]
    pproj = os.path.join(src, "main/java/pobserve/testing/spec/p_specs.pproj")
    logs = os.path.join(src, "resources/logs/unit-test-logs")
    total = same = 0
    failures = []
    for suite in ("happy-logs", "error-logs"):
        for case in range(2, 8):
            p = params(src, case)
            d = os.path.join(logs, suite, f"test_case_{case}_logs")
            for entry in sorted(os.listdir(d)):
                for sorted_flag in ([], ["--assumeInputsAreSorted"]):
                    label = f"{suite}/{entry}{' sorted' if sorted_flag else ''}"
                    if filt not in label:
                        continue
                    total += 1
                    common = ["--spec", p["--specName"], "--parser", p["--parser"], "-l", os.path.join(d, entry)] + sorted_flag
                    with tempfile.TemporaryDirectory() as work:
                        uo, po = os.path.join(work, "up"), os.path.join(work, "pn")
                        ucode, uout = run(up + ["--jars", jars, "-o", uo] + common, work)
                        pcode, pout = run(pn + ["-pp", pproj, "-o", po] + common, work)
                        problems = []
                        if ucode != pcode:
                            problems.append(f"exit code {ucode} vs {pcode}")
                        uc, pc = normalize_console(uout, uo), normalize_console(pout, po)
                        if uc != pc:
                            problems.append("console differs")
                        uf, pf = read_outputs(uo), read_outputs(po)
                        if set(uf) != set(pf):
                            problems.append(f"files {sorted(set(uf) ^ set(pf))}")
                        for n in sorted(set(uf) & set(pf)):
                            if normalize_file(n, uf[n]) != normalize_file(n, pf[n]):
                                problems.append(f"{n} differs")
                        if problems:
                            failures.append((label, problems))
                            if os.environ.get("KEEP"):
                                keep = os.path.join(os.environ["KEEP"], label.replace("/", "_").replace(" ", "_"))
                                os.makedirs(keep, exist_ok=True)
                                for n, t in (("up.txt", uc), ("pn.txt", pc)):
                                    open(os.path.join(keep, n), "w").write(t)
                                subprocess.run(["cp", "-r", uo, po, keep])
                        else:
                            same += 1
                    print(f"{'ok  ' if not problems else 'FAIL'} [exit {ucode}] {label}" + (f": {'; '.join(problems)}" if problems else ""), flush=True)
    print(f"\n{same}/{total} runs identical")
    sys.exit(0 if same == total else 1)


if __name__ == "__main__":
    main()
