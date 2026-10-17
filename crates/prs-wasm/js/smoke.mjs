// Node smoke test: node smoke.mjs path/to/prs_wasm.wasm
import { readFile } from "node:fs/promises";
import { loadP } from "./prs.mjs";

const p = await loadP(await readFile(process.argv[2]));
const model = `
event Ping; event Pong;
machine Main {
  var other: machine;
  start state Init { entry { other = new Echo(this); send other, Ping; } on Pong goto Done; }
  state Done { entry { assert false, "reached done"; } }
}
machine Echo { var m: machine; start state S { entry (x: machine) { m = x; } on Ping do { send m, Pong; } } }
`;
const r = p.run(["check", "-pf", "pingpong.p", "-pn", "PingPong", "-s", "10"], { "/work/pingpong.p": model }, { seed: 1 });
process.stdout.write(r.stdout);
const trace = Object.entries(r.files).find(([k]) => k.endsWith("_0_0.txt"))?.[1] ?? "";
if (r.exitCode !== 1 || !trace.includes("<ErrorLog> Assertion Failed")) throw new Error("expected a bug");
console.log("smoke test OK");
