// JavaScript wrapper for pnext_wasm.wasm (browser or Node.js, ES module).
//
//   import { loadP } from "./pnext.mjs";
//   const p = await loadP(wasmBytesOrResponse);
//   let r = p.run(["compile", "-pf", "model.p"], { "/work/model.p": source });
//   r = p.run(["check", "-tc", "tcMain", "-s", "1000"], r.files, { seed: 42 });
//   console.log(r.stdout, r.exitCode);

export async function loadP(source) {
  const imports = { env: { pnext_now: () => performance.now() / 1000 } };
  const { instance } =
    source instanceof Response || (typeof source?.then === "function")
      ? await WebAssembly.instantiateStreaming(source, imports)
      : await WebAssembly.instantiate(source, imports);
  const ex = instance.exports;
  const enc = new TextEncoder();
  const dec = new TextDecoder();

  function call(request) {
    const input = enc.encode(JSON.stringify(request));
    const ptr = ex.pnext_alloc(input.length);
    new Uint8Array(ex.memory.buffer, ptr, input.length).set(input);
    const out = ex.pnext_run(ptr, input.length);
    ex.pnext_free(ptr, input.length);
    const len = new DataView(ex.memory.buffer).getUint32(out, true);
    const json = dec.decode(new Uint8Array(ex.memory.buffer, out + 4, len));
    ex.pnext_result_free(out);
    return JSON.parse(json);
  }

  return {
    /** Runs `p <args>` over an in-memory file system ({absolutePath: text}). */
    run(args, files = {}, { cwd = "/work", seed } = {}) {
      return call({ args, files, cwd, seed: seed ?? Math.floor(Math.random() * 2 ** 32) });
    },
  };
}
