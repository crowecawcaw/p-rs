/// <reference lib="webworker" />
// Runs the WebAssembly build of the `p` tool. Each request starts from the user's
// files in a fresh in-memory file system rooted at /work.
import { loadP, type PTool, type PResult } from "../../crates/prs-wasm/js/prs.mjs";
import wasmUrl from "./generated/prs.wasm?url";
import type { CheckResult, CompileResult, Diagnostic, Files, Request, WorkerReply } from "./protocol";

const ROOT = "/work";
const PROJECT = "Playground";
let tool: Promise<PTool> | undefined;

function p(): Promise<PTool> {
  tool ??= loadP(fetch(wasmUrl));
  return tool;
}

function toFs(files: Files): Files {
  const out: Files = {};
  for (const [k, v] of Object.entries(files)) out[`${ROOT}/${k}`] = v;
  return out;
}

function pFiles(files: Files): string[] {
  return Object.keys(files).filter((f) => f.endsWith(".p")).sort();
}

const ERR_LOC = /\[([^\]\s]+\.p):(\d+):(\d+)\]\s*(.*)/;
const PARSE_ERR = /\[([^\]\s]+\.p)\]\s*parse error: line (\d+):(\d+)\s*(.*)/;

function parseDiagnostics(text: string): Diagnostic[] {
  const out: Diagnostic[] = [];
  for (const line of text.split("\n")) {
    const m = PARSE_ERR.exec(line) ?? ERR_LOC.exec(line);
    if (!m) continue;
    const parse = m.length === 5 && line.includes("parse error");
    out.push({
      file: m[1].replace(`${ROOT}/`, ""),
      line: Number(m[2]),
      // Parser columns are 0-based, checker columns 1-based.
      col: parse ? Number(m[3]) + 1 : Number(m[3]),
      message: m[4],
    });
  }
  return out;
}

function stripBoilerplate(s: string): string {
  return s
    .split("\n")
    .filter((l) => !l.startsWith("~~ [PTool]"))
    .join("\n")
    .trimEnd();
}

let cache: { key: string; out: PResult; tests: string[]; console: string; diagnostics: Diagnostic[] } | undefined;

async function compile(files: Files) {
  const key = JSON.stringify(files);
  if (cache?.key === key) return cache;
  const t = await p();
  const out = t.run(["compile", "-pf", ...pFiles(files), "-pn", PROJECT], toFs(files), { seed: 0 });
  const console = stripBoilerplate(out.stdout + out.stderr);
  let tests: string[] = [];
  if (out.exitCode === 0) {
    const list = t.run(["check", "--list-tests"], out.files, { seed: 0 });
    const lines = list.stdout.split("\n");
    const start = lines.findIndex((l) => l.startsWith(".. List of test cases"));
    if (start >= 0) {
      for (const l of lines.slice(start + 1)) {
        if (l.startsWith(".") || l.startsWith("~~") || !l.trim()) break;
        tests.push(l.trim());
      }
    }
  }
  cache = { key, out, tests, console, diagnostics: parseDiagnostics(console) };
  return cache;
}

async function handle(req: Request): Promise<CompileResult | CheckResult> {
  const t0 = performance.now();
  const c = await compile(req.files);
  if (req.kind === "compile") {
    return {
      kind: "compile",
      ok: c.out.exitCode === 0,
      console: c.console,
      diagnostics: c.diagnostics,
      tests: c.tests,
      millis: performance.now() - t0,
    };
  }
  if (c.out.exitCode !== 0) {
    return {
      kind: "check", ok: false, bug: false, exitCode: c.out.exitCode, console: c.console,
      diagnostics: c.diagnostics, testCase: req.testCase, millis: performance.now() - t0,
    };
  }
  const args = ["check", "-tc", req.testCase, "-s", String(req.schedules), "--seed", String(req.seed), ...req.extraArgs];
  const r = (await p()).run(args, c.out.files, { seed: req.seed });
  const dir = `${ROOT}/PCheckerOutput/${req.testCase}/BugFinding/`;
  const find = (suffix: string) =>
    Object.entries(r.files).find(([k]) => k.startsWith(dir) && k.endsWith(suffix))?.[1];
  const console = `$ p ${args.join(" ")}\n` + stripBoilerplate(r.stdout + r.stderr);
  return {
    kind: "check",
    ok: true,
    bug: r.exitCode === 1,
    exitCode: r.exitCode,
    console,
    diagnostics: parseDiagnostics(console),
    testCase: req.testCase,
    traceJson: find("_0_0.trace.json"),
    traceText: find("_0_0.txt"),
    schedule: find("_0_0.schedule"),
    millis: performance.now() - t0,
  };
}

self.onmessage = async (e: MessageEvent<{ id: number; req: Request }>) => {
  const { id, req } = e.data;
  let reply: WorkerReply;
  try {
    reply = { id, result: await handle(req) };
  } catch (err) {
    // A trap (e.g. a Rust panic) poisons the instance; start over next time.
    tool = undefined;
    cache = undefined;
    reply = { id, error: String(err) };
  }
  (self as unknown as DedicatedWorkerGlobalScope).postMessage(reply);
};
