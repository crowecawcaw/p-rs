// Node smoke test: node smoke.mjs path/to/pnext_wasm.wasm
import { readFile } from "node:fs/promises";
import { loadP } from "./pnext.mjs";

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
let r = p.run(["compile", "-pf", "pingpong.p", "-pn", "PingPong"], { "/work/pingpong.p": model });
if (r.exitCode !== 0) throw new Error(r.stdout + r.stderr);
r = p.run(["check", "-s", "10"], r.files, { seed: 1 });
process.stdout.write(r.stdout);
if (r.exitCode !== 1 || !r.stdout.includes("Assertion Failed")) throw new Error("expected a bug");
console.log("smoke test OK");
