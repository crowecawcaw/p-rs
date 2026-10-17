#!/usr/bin/env python3
"""Vendors upstream's small PObserve regression logs with golden outputs of upstream PObserve.

    UPSTREAM_POBSERVE=... UPSTREAM_JARS=regression.jar \
    scripts/vendor-pobserve-goldens.py <P checkout>/Src/PObserve/PObserveRegressionTesting/src

Writes tests/upstream/PObserve/: the regression spec (spec/), the run parameters
(params/), every log under 8 KB (logs/), and for each log, with and without
--assumeInputsAreSorted, upstream's exit code, console output and output files
(expected/<log>[.sorted]/), normalized as in compare-pobserve.py.
"""

import importlib.util
import os
import shutil
import sys
import tempfile

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("cmp", os.path.join(here, "compare-pobserve.py"))
cmp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cmp)

LIMIT = 8 * 1024


def small(path):
    if os.path.isdir(path):
        return all(os.path.getsize(os.path.join(path, f)) < LIMIT for f in os.listdir(path))
    return os.path.getsize(path) < LIMIT


def main():
    src = os.path.abspath(sys.argv[1])
    out = os.path.join(here, "..", "tests/upstream/PObserve")
    shutil.rmtree(out, ignore_errors=True)
    specdir = os.path.join(src, "main/java/pobserve/testing/spec")
    os.makedirs(os.path.join(out, "spec"))
    shutil.copy(os.path.join(specdir, "p_specs.pproj"), os.path.join(out, "spec"))
    shutil.copytree(os.path.join(specdir, "pSpecs"), os.path.join(out, "spec/pSpecs"))
    shutil.copytree(os.path.join(src, "resources/logs/params"), os.path.join(out, "params"))
    up = cmp.shlex.split(os.environ["UPSTREAM_POBSERVE"])
    jars = os.path.abspath(os.environ["UPSTREAM_JARS"])
    logs = os.path.join(src, "resources/logs/unit-test-logs")
    for suite in ("happy-logs", "error-logs"):
        for case in range(2, 8):
            p = cmp.params(src, case)
            d = os.path.join(logs, suite, f"test_case_{case}_logs")
            for entry in sorted(os.listdir(d)):
                path = os.path.join(d, entry)
                if not small(path):
                    continue
                rel = os.path.join(suite, f"test_case_{case}_logs", entry)
                dst = os.path.join(out, "logs", rel)
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                (shutil.copytree if os.path.isdir(path) else shutil.copy)(path, dst)
                for flag in ([], ["--assumeInputsAreSorted"]):
                    with tempfile.TemporaryDirectory() as work:
                        uo = os.path.join(work, "out")
                        args = ["--jars", jars, "-o", uo, "--spec", p["--specName"], "--parser", p["--parser"], "-l", path] + flag
                        code, text = cmp.run(up + args, work)
                        exp = os.path.join(out, "expected", rel + (".sorted" if flag else ""))
                        os.makedirs(exp)
                        open(os.path.join(exp, "exit_code"), "w").write(f"{code}\n")
                        open(os.path.join(exp, "console.txt"), "w").write(cmp.normalize_console(text, uo))
                        for n, t in cmp.read_outputs(uo).items():
                            open(os.path.join(exp, n), "w").write(cmp.normalize_file(n, t))
                        print(code, rel, "sorted" if flag else "", flush=True)


if __name__ == "__main__":
    main()
