#!/usr/bin/env python3
"""Records what upstream `p` prints and writes on a fixed set of runs, for
crates/snappea/tests/compat_check_upstream.rs to compare snappea-compat against.

    UPSTREAM_P="dotnet <P checkout>/Bld/Drops/Release/Binaries/net8.0/p.dll" \
    scripts/record-upstream-goldens.py [--jobs N] [id prefix ...]

Build upstream at the commit in tests/upstream/goldens/UPSTREAM (`Bld/build.sh`). Each case
copies a directory of tests/upstream into a fresh folder and runs a list of steps there.
Steps that only upstream runs (`p compile`) set things up; the others run on both tools and
are recorded. Results go to tests/upstream/goldens/cases/<id>/:

    case.json        the steps, their arguments for each tool and upstream's exit codes
    <step>.out.txt   standard output     <step>.err.txt   standard error
    outputs.txt      every file the run wrote (PCheckerOutput, ...)
    files/<path>     the contents of those files, except .sci; files over 128 KB as
                     files/<path>.sha256, the SHA-256 of their normalized contents

Upstream prints compiler warnings in `p compile`, snappea-compat when it loads the sources in
`check`: the warnings of the compile step are put first in the recorded output of the check
steps. Console output and files are normalized with tests/upstream/goldens/normalize.tsv. With id
prefixes, only matching cases are recorded and the others are kept.
"""

import concurrent.futures
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
UPSTREAM = os.path.normpath(os.path.join(HERE, "..", "tests", "upstream"))
GOLDENS = os.path.join(UPSTREAM, "goldens")
CASES = os.path.join(GOLDENS, "cases")
SEED = "42"
TIMEOUT = 600
LARGE = 128 * 1024

# Runs upstream cannot finish: the action loops forever without a scheduling point and
# upstream hangs, using memory until it is killed (snappea-compat reports a bug).
EXCLUDED = {"regression/Liveness/DynamicError/InfiniteLoopInAtomicBlock"}

# Files whose contents are not compared (only that they are written): .NET serialized
# coverage, which snappea-compat writes in a reduced form.
NOT_COMPARED = (".sci",)


def load_rules():
    drops, masks = [], []
    with open(os.path.join(GOLDENS, "normalize.tsv"), encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            kind, *rest = line.split("\t")
            if kind == "drop":
                drops.append(re.compile(rest[0]))
            elif kind == "mask":
                # literal, except ${n} for a capture group (Rust's syntax)
                repl = re.sub(r"\$\{(\d+)\}", r"\\g<\1>", rest[1].replace("\\", "\\\\"))
                masks.append((re.compile(rest[0]), repl))
            else:
                sys.exit(f"normalize.tsv: unknown rule {kind!r}")
    return drops, masks


DROPS, MASKS = load_rules()


def normalize(text, workdir, console):
    text = text.removeprefix("﻿").replace(workdir, "<dir>")
    if console:
        text = "\n".join(l for l in text.split("\n") if not any(d.search(l) for d in DROPS))
    for pattern, repl in MASKS:
        text = pattern.sub(repl, text)
    return text


def p_files(d):
    out = []
    for root, dirs, files in os.walk(d):
        dirs.sort()
        out += [os.path.relpath(os.path.join(root, f), d) for f in sorted(files) if f.endswith(".p")]
    return out


def step(name, p, snappea):
    return {"name": name, "p": p, "snappea": snappea}


def setup(p):
    return {"p": p}


def check_step(name, project, args):
    return step(name, ["check", *args], ["check", *project, *args])


def regression_cases():
    root = os.path.join(UPSTREAM, "RegressionTests")
    for suite in sorted(os.listdir(root)):
        for category in ("Correct", "DynamicError", "StaticError"):
            cdir = os.path.join(root, suite, category)
            if not os.path.isdir(cdir):
                continue
            for test in sorted(os.listdir(cdir)):
                tdir = os.path.join(cdir, test)
                files = p_files(tdir)
                if not files:
                    continue
                project = ["-pf", *files, "-pn", re.sub(r"[^A-Za-z0-9_]", "_", test)]
                case = {"id": f"regression/{suite}/{category}/{test}", "source": os.path.relpath(tdir, UPSTREAM)}
                if category == "StaticError":
                    case["steps"] = [step("compile", ["compile", *project], ["check", *project])]
                else:
                    steps = [setup(["compile", *project])]
                    if "param" in test.lower():
                        steps.append(check_step("list-tests", project, ["--list-tests"]))
                    steps.append(check_step("check", project, ["-s", "10", "--max-steps", "1000", "--seed", SEED]))
                    case["steps"] = steps
                yield case


def tutorial_cases():
    root = os.path.join(UPSTREAM, "Tutorial")
    for dirpath, dirs, files in sorted(os.walk(root)):
        dirs.sort()
        rel = os.path.relpath(dirpath, root)
        # Advanced/: PVerifier projects. PriorityQueue: foreign code in C#. Both unsupported.
        if rel.split(os.sep)[0] in ("Common", "Advanced", "PriorityQueue"):
            continue
        if any(f.endswith(".pproj") for f in files):
            # the test cases are only known once upstream lists them (see run_case)
            yield {
                "id": f"tutorial/{rel}",
                "source": "Tutorial",
                "cwd": rel,
                "steps": [setup(["compile"]), step("list-tests", ["check", "--list-tests"], ["check", "--list-tests"])],
                "each_test": ["-s", "100", "--seed", SEED],
            }


def option_cases():
    """The command line: help, errors, replay, the other strategies and options."""
    bug2 = "RegressionTests/Feature1SMLevelDecls/DynamicError/bug2"
    proj = ["-pf", "bug2.p", "-pn", "bug2"]
    yield {"id": "cli/no-command", "source": None, "steps": [step("run", [], [])]}
    yield {"id": "cli/unknown-command", "source": None, "steps": [step("run", ["bogus"], ["bogus"])]}
    yield {"id": "cli/check-help", "source": None, "steps": [step("run", ["check", "--help"], ["check", "--help"])]}
    yield {"id": "cli/check-bad-option", "source": None,
           "steps": [step("run", ["check", "--bogus"], ["check", "--bogus"])]}
    yield {"id": "cli/merge-scenario-coverage-empty", "source": None,
           "steps": [step("run", ["merge-scenario-coverage"], ["merge-scenario-coverage"])]}

    def bug2_case(name, *args):
        return {"id": f"cli/{name}", "source": bug2,
                "steps": [setup(["compile", *proj]), check_step("check", proj, list(args))]}

    yield {"id": "cli/illegal-project-name", "source": bug2,
           "steps": [step("compile", ["compile", "-pf", "bug2.p", "-pn", "bug-2"],
                          ["check", "-pf", "bug2.p", "-pn", "bug-2"])]}
    yield bug2_case("verbose", "-v", "--seed", SEED)
    yield bug2_case("outdir", "-o", "out", "--seed", SEED)
    yield bug2_case("unknown-test-case", "-tc", "Nope")
    yield bug2_case("xml-trace", "--xml-trace", "--seed", SEED)
    yield bug2_case("xml-trace-bad-value", "--xml-trace", "trace.xml")
    yield {"id": "cli/replay", "source": bug2, "steps": [
        setup(["compile", *proj]),
        check_step("check", proj, ["--seed", SEED]),
        check_step("replay", proj, ["--replay", "PCheckerOutput/BugFinding/bug2_0_0.schedule"]),
    ]}
    yield {"id": "cli/history", "source": bug2, "steps": [
        setup(["compile", *proj]),
        check_step("first", proj, ["--seed", SEED]),
        check_step("second", proj, ["--seed", SEED]),
        check_step("third", proj, ["--seed", SEED]),
    ]}
    scov = "RegressionTests/Feature1SMLevelDecls/Correct/ScenarioCoverageBasic"
    sproj = ["-pf", "ScenarioCoverageBasic.p", "-pn", "S"]
    yield {"id": "cli/merge-scenario-coverage", "source": scov, "steps": [
        setup(["compile", *sproj]),
        check_step("check", sproj, ["-s", "5", "--seed", SEED]),
        step("merge", ["merge-scenario-coverage", "PCheckerOutput"], ["merge-scenario-coverage", "PCheckerOutput"]),
    ]}

    # strategies and exploration options, on a tutorial test case with a bug (not PEx mode:
    # upstream runs it on a Java build of the model, `p compile --mode pex`)
    strategies = {
        "pct": ["--sch-pct", "3"],
        "fairpct": ["--sch-fairpct", "3"],
        "probabilistic": ["--sch-probabilistic", "2"],
        "pos": ["--sch-pos"],
        "feedback": ["--sch-feedback"],
        "feedbackpct": ["--sch-feedbackpct", "10"],
        "feedbackpos": ["--sch-feedbackpos"],
        "explore": ["--explore"],
        "max-steps": ["--max-steps", "20", "--fail-on-maxsteps"],
        "max-steps-two": ["--max-steps", "20", "40"],
    }
    for name, args in strategies.items():
        for tc in ("tcSingleClient", "tcAbstractServerMultipleClients"):
            yield {"id": f"options/{name}/{tc}", "source": "Tutorial", "cwd": "1_ClientServer", "steps": [
                setup(["compile"]),
                step("check", ["check", "-tc", tc, "-s", "20", "--seed", SEED, *args],
                     ["check", "-tc", tc, "-s", "20", "--seed", SEED, *args]),
            ]}


def all_cases():
    yield from regression_cases()
    yield from tutorial_cases()
    yield from option_cases()


def snapshot(d):
    out = set()
    for root, dirs, files in os.walk(d):
        for f in files:
            out.add(os.path.relpath(os.path.join(root, f), d))
    return out


def generated(path):
    parts = path.split(os.sep)
    return parts[0] == "PGenerated" or "obj" in parts or "bin" in parts


def run(cmd, cwd):
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None, b"", b""
    return r.returncode, r.stdout, r.stderr


def run_case(case, upstream_p):
    with tempfile.TemporaryDirectory(prefix="pgold") as tmp:
        work = os.path.join(tmp, "w")
        if case["source"]:
            shutil.copytree(os.path.join(UPSTREAM, case["source"]), work)
        else:
            os.makedirs(work)
        cwd = os.path.join(work, case.get("cwd", ""))
        before = snapshot(cwd)
        steps = list(case["steps"])
        recorded, outputs, warnings = [], {}, ""
        i = 0
        while i < len(steps):
            s = steps[i]
            code, out, err = run(upstream_p + s["p"], cwd)
            if code is None:
                return case["id"], f"timeout in {s['p']}"
            if "name" not in s:
                if code != 0:
                    return case["id"], f"setup {s['p']} exited {code}:\n{out.decode(errors='replace')}"
                text = normalize(out.decode("utf-8", "replace"), work, True)
                warnings = "".join(l + "\n" for l in text.split("\n") if l.startswith("[Warning:]"))
                i += 1
                continue
            out = normalize(out.decode("utf-8", "replace"), work, True)
            if s["p"][:1] == ["check"]:
                out = warnings + out
            err = normalize(err.decode("utf-8", "replace"), work, True)
            recorded.append({**s, "exit_code": code, "out": out, "err": err})
            if s["name"] == "list-tests" and "each_test" in case:
                tests = out.split("test cases (total ")[1].split("\n")[1:] if "test cases (total " in out else []
                for tc in [t for t in tests if t and not t.startswith(("~~", "."))]:
                    args = ["-tc", tc, *case["each_test"]]
                    steps.append(step(f"check-{tc}", ["check", *args], ["check", *args]))
            i += 1
        for rel in sorted(snapshot(cwd) - before):
            if generated(rel):
                continue
            with open(os.path.join(cwd, rel), "rb") as f:
                data = f.read()
            outputs[rel] = None if rel.endswith(NOT_COMPARED) else normalize(data.decode("utf-8", "replace"), work, False)

    d = os.path.join(CASES, case["id"])
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    meta = {k: case[k] for k in ("source", "cwd") if case.get(k)}
    meta["steps"] = [{"name": r["name"], "p": r["p"], "snappea": r["snappea"], "exit_code": r["exit_code"]}
                     for r in recorded]
    with open(os.path.join(d, "case.json"), "w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2)
        f.write("\n")
    for r in recorded:
        for kind in ("out", "err"):
            with open(os.path.join(d, f"{r['name']}.{kind}.txt"), "w", encoding="utf-8", newline="") as f:
                f.write(r[kind])
    with open(os.path.join(d, "outputs.txt"), "w", encoding="utf-8") as f:
        f.write("".join(p.replace(os.sep, "/") + "\n" for p in outputs))
    for rel, text in outputs.items():
        if text is None:
            continue
        dst = os.path.join(d, "files", rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        data = text.encode("utf-8")
        if len(data) > LARGE:
            dst, data = dst + ".sha256", (hashlib.sha256(data).hexdigest() + "\n").encode()
        with open(dst, "wb") as f:
            f.write(data)
    return case["id"], None


def main():
    args = sys.argv[1:]
    jobs = os.cpu_count() or 1
    if args[:1] == ["--jobs"]:
        jobs, args = int(args[1]), args[2:]
    upstream_p = shlex.split(os.environ.get("UPSTREAM_P") or sys.exit("set UPSTREAM_P to the upstream p command"))
    cases = [c for c in all_cases()
             if c["id"] not in EXCLUDED and (not args or any(c["id"].startswith(a) for a in args))]
    if not args:
        shutil.rmtree(CASES, ignore_errors=True)
    failed = []
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        for n, (cid, problem) in enumerate(pool.map(lambda c: run_case(c, upstream_p), cases), 1):
            print(f"[{n}/{len(cases)}] {cid}" + (f": SKIPPED, {problem}" if problem else ""), flush=True)
            if problem:
                failed.append(cid)
    print(f"recorded {len(cases) - len(failed)} cases, skipped {len(failed)}")


if __name__ == "__main__":
    main()
