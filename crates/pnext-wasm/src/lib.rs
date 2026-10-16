//! WebAssembly entry point for the P tool.
//!
//! The module exposes a tiny C ABI (no wasm-bindgen needed):
//!
//! * `pnext_alloc(len) -> ptr` / `pnext_free(ptr, len)` manage input buffers;
//! * `pnext_run(ptr, len) -> ptr` runs one request and returns a pointer to a result
//!   buffer laid out as a little-endian `u32` length followed by UTF-8 JSON;
//! * `pnext_result_free(ptr)` releases a result buffer.
//!
//! A request is JSON: `{"args": ["check", "-tc", "t"], "files": {"path": "text"},
//! "cwd": "/work", "seed": 42}`. The command runs against an in-memory file system holding
//! `files`; the response carries the exit code, console output and the complete file
//! system afterwards (so a following request can pass it back in, e.g. `compile` then
//! `check`). `js/pnext.mjs` wraps this for JavaScript.
//!
//! The host may import `env.pnext_now() -> f64` (seconds) for elapsed-time reporting;
//! when targeting WebAssembly it is required.

use std::collections::BTreeMap;
use std::path::Path;

use pnext_compat::host::{Host, MemHost};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub args: Vec<String>,
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub seed: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub files: BTreeMap<String, String>,
}

#[cfg(target_arch = "wasm32")]
mod clock {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn pnext_now() -> f64;
    }
    pub fn now() -> f64 {
        unsafe { pnext_now() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod clock {
    pub fn now() -> f64 {
        0.0
    }
}

/// A [`MemHost`] whose clock reads the embedder's time.
struct WasmHost(MemHost);

impl Host for WasmHost {
    fn write(&mut self, s: pnext_compat::host::Stream, t: &str, c: Option<pnext_compat::host::Color>) {
        self.0.write(s, t, c)
    }
    fn cwd(&self) -> std::path::PathBuf {
        self.0.cwd()
    }
    fn read_to_string(&self, p: &Path) -> std::io::Result<String> {
        self.0.read_to_string(p)
    }
    fn write_file(&mut self, p: &Path, c: &str) -> std::io::Result<()> {
        self.0.write_file(p, c)
    }
    fn is_file(&self, p: &Path) -> bool {
        self.0.is_file(p)
    }
    fn is_dir(&self, p: &Path) -> bool {
        self.0.is_dir(p)
    }
    fn read_dir(&self, p: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        self.0.read_dir(p)
    }
    fn create_dir_all(&mut self, p: &Path) -> std::io::Result<()> {
        self.0.create_dir_all(p)
    }
    fn remove_dir_all(&mut self, p: &Path) -> std::io::Result<()> {
        self.0.remove_dir_all(p)
    }
    fn rename(&mut self, a: &Path, b: &Path) -> std::io::Result<()> {
        self.0.rename(a, b)
    }
    fn now(&self) -> f64 {
        clock::now()
    }
    fn random_seed(&mut self) -> u32 {
        self.0.random_seed()
    }
}

/// Runs one request (usable natively too, e.g. for tests).
pub fn run(req: Request) -> Response {
    let cwd = req.cwd.unwrap_or_else(|| "/work".into());
    let mut mem = MemHost::new(&cwd);
    mem.seed = req.seed.unwrap_or(0);
    for (path, text) in &req.files {
        mem.add_file(path, text);
    }
    let mut host = WasmHost(mem);
    let exit_code = pnext_compat::run(&req.args, &mut host);
    let mem = host.0;
    let mut files = BTreeMap::new();
    for p in mem.files_under("/") {
        if let Ok(t) = mem.read_to_string(&p) {
            files.insert(p.to_string_lossy().to_string(), t);
        }
    }
    Response { exit_code, stdout: mem.stdout, stderr: mem.stderr, files }
}

/// Runs a JSON request and returns the JSON response.
pub fn run_json(input: &str) -> String {
    let resp = match serde_json::from_str::<Request>(input) {
        Ok(req) => run(req),
        Err(e) => Response {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("invalid request: {}\n", e),
            files: BTreeMap::new(),
        },
    };
    serde_json::to_string(&resp).expect("serializable")
}

#[no_mangle]
pub extern "C" fn pnext_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// # Safety
/// `ptr`/`len` must come from [`pnext_alloc`].
#[no_mangle]
pub unsafe extern "C" fn pnext_free(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
}

/// # Safety
/// `ptr..ptr+len` must be an allocated buffer holding UTF-8 JSON.
#[no_mangle]
pub unsafe extern "C" fn pnext_run(ptr: *const u8, len: usize) -> *mut u8 {
    let input = std::slice::from_raw_parts(ptr, len);
    let out = run_json(&String::from_utf8_lossy(input));
    let bytes = out.into_bytes();
    let mut buf = Vec::with_capacity(bytes.len() + 4);
    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&bytes);
    let mut buf = buf.into_boxed_slice();
    let p = buf.as_mut_ptr();
    std::mem::forget(buf);
    p
}

/// # Safety
/// `ptr` must come from [`pnext_run`].
#[no_mangle]
pub unsafe extern "C" fn pnext_result_free(ptr: *mut u8) {
    let len = u32::from_le_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)]) as usize + 4;
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_then_check_round_trip() {
        let mut files = BTreeMap::new();
        files.insert(
            "/work/m.p".to_string(),
            "machine Main { start state S { entry { assert false, \"no\"; } } }".to_string(),
        );
        let r = run(Request { args: vec!["compile".into(), "-pf".into(), "m.p".into()], files, cwd: None, seed: None });
        assert_eq!(r.exit_code, 0, "{}", r.stdout);
        let r = run(Request {
            args: vec!["check".into(), "PGenerated/PChecker/generatedOutput.pnext".into()],
            files: r.files,
            cwd: None,
            seed: Some(7),
        });
        assert_eq!(r.exit_code, 1, "{}", r.stdout);
        assert!(r.stdout.contains("Error: Assertion Failed: m.p:1:40 no"), "{}", r.stdout);
        assert!(r.files.keys().any(|k| k.ends_with(".trace.json")));
    }
}
