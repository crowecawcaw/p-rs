//! The `p` binary: [`pnext_compat::run`] on the native file system and console.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use pnext_compat::host::{Color, Host, Stream};

struct NativeHost {
    start: Instant,
    color_out: bool,
    color_err: bool,
}

impl Host for NativeHost {
    fn write(&mut self, stream: Stream, text: &str, color: Option<Color>) {
        let use_color = match stream {
            Stream::Out => self.color_out,
            Stream::Err => self.color_err,
        };
        let code = match (use_color, color) {
            (true, Some(Color::Red)) => Some("\x1b[31m"),
            (true, Some(Color::Yellow)) => Some("\x1b[33m"),
            (true, Some(Color::Cyan)) => Some("\x1b[36m"),
            _ => None,
        };
        let mut buf = String::new();
        if let Some(c) = code {
            buf.push_str(c);
        }
        buf.push_str(text);
        if code.is_some() {
            buf.push_str("\x1b[0m");
        }
        let _ = match stream {
            Stream::Out => io::stdout().lock().write_all(buf.as_bytes()),
            Stream::Err => io::stderr().lock().write_all(buf.as_bytes()),
        };
    }

    fn cwd(&self) -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read(path).map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    fn write_file(&mut self, path: &Path, contents: &str) -> io::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(path, contents)
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut v: Vec<PathBuf> = std::fs::read_dir(path)?.filter_map(|e| e.ok().map(|e| e.path())).collect();
        v.sort();
        Ok(v)
    }

    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir_all(path)
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    fn max_memory_gb(&self) -> f64 {
        // peak resident set size (Linux); 0 elsewhere
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmHWM:"))
                    .and_then(|l| l.split_whitespace().nth(1).and_then(|k| k.parse::<f64>().ok()))
            })
            .map(|kb| kb / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    }

    fn random_seed(&mut self) -> u32 {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let mut x = (t as u64) ^ ((std::process::id() as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15;
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
        x ^= x >> 33;
        x as u32
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = NativeHost {
        start: Instant::now(),
        color_out: io::stdout().is_terminal(),
        color_err: io::stderr().is_terminal(),
    };
    let code = pnext_compat::run(&args, &mut host);
    let _ = io::stdout().flush();
    std::process::exit(code);
}
