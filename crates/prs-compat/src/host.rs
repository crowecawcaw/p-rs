//! The environment the commands run in: console, file system, clock and entropy.
//!
//! Commands never touch `std::fs`, `std::time` or `std::process` directly so that the crate
//! also runs in a browser. [`MemHost`] is an in-memory implementation for tests and wasm;
//! the `prs` binary supplies a native one.

use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Console stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// Console colors upstream uses (`ConsoleColor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Yellow,
    Cyan,
}

/// Everything the commands need from their environment. File system methods behave like
/// their `std::fs` namesakes unless documented otherwise.
pub trait Host {
    /// Writes console text (no newline added).
    fn write(&mut self, stream: Stream, text: &str, color: Option<Color>);
    /// The current directory (absolute).
    fn cwd(&self) -> PathBuf;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    /// Creates or replaces a file, creating its parent directories first.
    fn write_file(&mut self, path: &Path, contents: &str) -> io::Result<()>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    /// Entries (full paths) of a directory, sorted.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    /// Seconds since an arbitrary fixed point (for elapsed-time reporting).
    fn now(&self) -> f64;
    /// Peak memory use of the process in gigabytes, or 0 if unknown.
    fn max_memory_gb(&self) -> f64;
    /// A fresh random seed (used when `--seed` is not given).
    fn random_seed(&mut self) -> u32;
    /// Seconds since the Unix epoch (wall-clock time, for timestamped names), or 0 if unknown.
    fn unix_time(&self) -> f64;
}

/// Convenience console helpers.
pub trait HostExt: Host {
    fn out_line(&mut self, s: &str) {
        self.write(Stream::Out, s, None);
        self.write(Stream::Out, "\n", None);
    }
    fn out_colored_line(&mut self, s: &str, c: Color) {
        self.write(Stream::Out, s, Some(c));
        self.write(Stream::Out, "\n", None);
    }
}

impl<T: Host + ?Sized> HostExt for T {}

/// `Path.GetFullPath`: makes `p` absolute against `cwd` and resolves `.`/`..` lexically.
pub fn full_path(cwd: &Path, p: &Path) -> PathBuf {
    let joined = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    normalize(&joined)
}

/// Lexically resolves `.` and `..` components.
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// `Path.GetRelativePath(from, to)` for absolute, normalized paths.
pub fn relative_path(from: &Path, to: &Path) -> String {
    let f: Vec<_> = from.components().collect();
    let t: Vec<_> = to.components().collect();
    let common = f.iter().zip(t.iter()).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return to.to_string_lossy().to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..f.len() {
        parts.push("..".into());
    }
    for c in &t[common..] {
        parts.push(c.as_os_str().to_string_lossy().to_string());
    }
    if parts.is_empty() {
        ".".into()
    } else {
        parts.join("/")
    }
}

/// Whether a file name is hidden the way .NET reports it on Unix (leading dot).
pub fn is_hidden(p: &Path) -> bool {
    p.file_name().and_then(|n| n.to_str()).map(|n| n.starts_with('.')).unwrap_or(false)
}

#[derive(Debug, Clone)]
enum Node {
    Dir,
    File(String),
}

/// An in-memory host: a virtual file system plus captured console output. Memory use and
/// wall-clock time read as 0.
#[derive(Debug, Clone)]
pub struct MemHost {
    nodes: BTreeMap<PathBuf, Node>,
    pub cwd: PathBuf,
    pub stdout: String,
    pub stderr: String,
    /// [`Host::now`]; always 0 by default.
    pub clock: fn() -> f64,
    /// [`Host::random_seed`].
    pub seed: u32,
}

impl Default for MemHost {
    fn default() -> Self {
        MemHost::new("/work")
    }
}

impl MemHost {
    pub fn new(cwd: &str) -> MemHost {
        let mut h = MemHost {
            nodes: BTreeMap::new(),
            cwd: PathBuf::from(cwd),
            stdout: String::new(),
            stderr: String::new(),
            clock: || 0.0,
            seed: 0,
        };
        h.create_dir_all(Path::new(cwd)).expect("an empty file system has no files in the way");
        h
    }

    fn abs(&self, p: &Path) -> PathBuf {
        full_path(&self.cwd, p)
    }

    /// Adds a file (creating parent directories), path relative to the cwd or absolute.
    ///
    /// # Panics
    ///
    /// If the file cannot be written (a directory is in the way, or a file where a parent
    /// directory should be).
    pub fn add_file(&mut self, path: &str, contents: &str) {
        if let Err(e) = self.write_file(Path::new(path), contents) {
            panic!("cannot add {}: {}", path, e);
        }
    }

    /// All files under `dir` (absolute paths).
    pub fn files_under(&self, dir: &str) -> Vec<PathBuf> {
        let d = self.abs(Path::new(dir));
        self.nodes
            .iter()
            .filter(|(p, n)| matches!(n, Node::File(_)) && p.starts_with(&d))
            .map(|(p, _)| p.clone())
            .collect()
    }
}

fn not_found(p: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("Could not find a part of the path '{}'.", p.display()))
}

fn is_a_directory(p: &Path) -> io::Error {
    io::Error::other(format!("Is a directory: '{}'", p.display()))
}

fn not_a_directory(p: &Path) -> io::Error {
    io::Error::other(format!("Not a directory: '{}'", p.display()))
}

impl Host for MemHost {
    fn write(&mut self, stream: Stream, text: &str, _color: Option<Color>) {
        match stream {
            Stream::Out => self.stdout.push_str(text),
            Stream::Err => self.stderr.push_str(text),
        }
    }

    fn cwd(&self) -> PathBuf {
        self.cwd.clone()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        match self.nodes.get(&self.abs(path)) {
            Some(Node::File(s)) => Ok(s.clone()),
            Some(Node::Dir) => Err(is_a_directory(path)),
            None => Err(not_found(path)),
        }
    }

    fn write_file(&mut self, path: &Path, contents: &str) -> io::Result<()> {
        let p = self.abs(path);
        if let Some(Node::Dir) = self.nodes.get(&p) {
            return Err(is_a_directory(path));
        }
        if let Some(parent) = p.parent() {
            self.create_dir_all(parent)?;
        }
        self.nodes.insert(p, Node::File(contents.to_string()));
        Ok(())
    }

    fn is_file(&self, path: &Path) -> bool {
        matches!(self.nodes.get(&self.abs(path)), Some(Node::File(_)))
    }

    fn is_dir(&self, path: &Path) -> bool {
        matches!(self.nodes.get(&self.abs(path)), Some(Node::Dir))
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let d = self.abs(path);
        if !self.is_dir(&d) {
            return Err(not_found(path));
        }
        Ok(self.nodes.keys().filter(|p| p.parent() == Some(d.as_path())).cloned().collect())
    }

    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        let p = self.abs(path);
        let mut cur = PathBuf::new();
        for c in p.components() {
            cur.push(c.as_os_str());
            if let Some(Node::File(_)) = self.nodes.get(&cur) {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("File exists: '{}'", cur.display())));
            }
            self.nodes.entry(cur.clone()).or_insert(Node::Dir);
        }
        Ok(())
    }

    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()> {
        let p = self.abs(path);
        match self.nodes.get(&p) {
            Some(Node::Dir) => {
                self.nodes.retain(|k, _| !k.starts_with(&p));
                Ok(())
            }
            Some(Node::File(_)) => Err(not_a_directory(path)),
            None => Err(not_found(path)),
        }
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let f = self.abs(from);
        let t = self.abs(to);
        if !self.nodes.contains_key(&f) {
            return Err(not_found(from));
        }
        let moved: Vec<(PathBuf, Node)> =
            self.nodes.iter().filter(|(k, _)| k.starts_with(&f)).map(|(k, v)| (k.clone(), v.clone())).collect();
        self.nodes.retain(|k, _| !k.starts_with(&f));
        for (k, v) in moved {
            let rel = k.strip_prefix(&f).unwrap().to_path_buf();
            let dest = if rel.as_os_str().is_empty() { t.clone() } else { t.join(rel) };
            self.nodes.insert(dest, v);
        }
        Ok(())
    }

    fn now(&self) -> f64 {
        (self.clock)()
    }

    fn max_memory_gb(&self) -> f64 {
        0.0
    }

    fn random_seed(&mut self) -> u32 {
        self.seed
    }

    fn unix_time(&self) -> f64 {
        0.0
    }
}
