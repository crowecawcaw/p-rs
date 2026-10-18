// JavaScript wrapper for prs_wasm.wasm (browser or Node.js, ES module).
//
//   import { loadP } from "./prs.mjs";
//   const p = await loadP(wasmBytesOrResponse);
//   const r = p.run(["check", "model.p", "tcMain", "-n", "1000"], { "/work/model.p": source }, { seed: 42 });
//   console.log(r.stdout, r.exitCode);

export async function loadP(source) {
  const imports = { env: { prs_now: () => performance.now() / 1000 } };
  const { instance } =
    source instanceof Response || (typeof source?.then === "function")
      ? await WebAssembly.instantiateStreaming(source, imports)
      : await WebAssembly.instantiate(source, imports);
  const ex = instance.exports;
  const enc = new TextEncoder();
  const dec = new TextDecoder();

  function call(request) {
    const input = enc.encode(JSON.stringify(request));
    const ptr = ex.prs_alloc(input.length);
    new Uint8Array(ex.memory.buffer, ptr, input.length).set(input);
    const out = ex.prs_run(ptr, input.length);
    ex.prs_free(ptr, input.length);
    const len = new DataView(ex.memory.buffer).getUint32(out, true);
    const json = dec.decode(new Uint8Array(ex.memory.buffer, out + 4, len));
    ex.prs_result_free(out);
    return JSON.parse(json);
  }

  return {
    /** Runs `prs <args>` over an in-memory file system ({absolutePath: text}). */
    run(args, files = {}, { cwd = "/work", seed } = {}) {
      return call({ args, files, cwd, seed: seed ?? Math.floor(Math.random() * 2 ** 32) });
    },
  };
}
