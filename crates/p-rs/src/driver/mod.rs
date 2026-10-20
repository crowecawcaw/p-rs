//! The commands' work, on a [`Host`]: finding and type-checking a project's P sources
//! ([`project`]), checking and replaying test cases ([`check`]), checking logs against spec
//! monitors ([`observe`]) and merging scenario coverage ([`scenario`]).
//!
//! The command lines parse their options into these modules' configurations: `prs`
//! (`prs::cli`) with its own options, and `prs-compat` (`prs::compat`) with upstream P's.
//! Console messages and output files are the same for both.

pub mod check;
pub mod observe;
mod pproj;
pub mod project;
pub mod scenario;
mod sci;

use std::path::Path;

use crate::host::{self, Color, Host, Stream};

/// Process exit requested (`Environment.Exit(code)`); the problem has been reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit(pub i32);

/// `Error.ReportAndExit`: `Error: {msg}` on stderr, exit code 1.
pub(crate) fn report_and_exit(host: &mut dyn Host, msg: &str) -> Exit {
    report(host, msg);
    Exit(1)
}

/// `Error.Report`: `Error: {msg}` on stderr.
fn report(host: &mut dyn Host, msg: &str) {
    host.write(Stream::Err, "Error: ", Some(Color::Red));
    host.write(Stream::Err, msg, Some(Color::Yellow));
    host.write(Stream::Err, "\n", None);
}

/// Writes an output file the command has announced (`..... Writing {path}`), `path` being
/// relative to the current directory or absolute. A failure ends the command with
/// `Error: ...` naming the file and exit code 1, as an uncaught I/O exception ends upstream's.
fn write_output(host: &mut dyn Host, path: &str, contents: &str) -> Result<(), Exit> {
    let full = host::full_path(&host.cwd(), Path::new(path));
    host.write_file(&full, contents)
        .map_err(|e| report_and_exit(host, &format!("Could not write file '{}': {}", path, e)))
}

/// `{value:0.##}`.
fn format_0_00(v: f64) -> String {
    let s = format!("{:.2}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}
