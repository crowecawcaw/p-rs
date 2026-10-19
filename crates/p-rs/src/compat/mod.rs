//! Drop-in compatibility with the upstream P command line tools.
//!
//! [`run`] reproduces `p check` and `p merge-scenario-coverage` from `PCommandLine`, and
//! [`observe`] the PObserve command line: the same options, help text, project-file
//! handling, console messages, output files and exit codes, on top of the pure-Rust type
//! checker ([`crate::sema`]) and checker ([`crate::checker`]). There is no `p compile`: the
//! commands read the P sources directly. Everything goes through a [`Host`], so
//! the commands also run in-memory (tests, browsers).
//!
//! [`check`], [`project`] and [`scenario`] also expose the steps of `p check` (resolving
//! sources, type-checking, checking one test case, replaying a schedule) for front ends
//! with their own command line, such as `prs`.
//!
//! Known differences from upstream are listed in the repository README.

mod argparse;
pub mod check;
pub mod host;
pub mod observe;
mod pproj;
pub mod project;
pub mod scenario;
mod sci;

use std::path::Path;

use host::{Color, Host, HostExt, Stream};

/// Process exit requested (`Environment.Exit(code)`); the problem has been reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit(pub i32);

/// `Error.ReportAndExit`: `Error: {msg}` on stderr, exit code 1.
fn report_and_exit(host: &mut dyn Host, msg: &str) -> Exit {
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

/// The version reported by `p --version`.
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (prs)");

/// `CommandLine.Main`: runs the `p` tool with `args` (without the program name) and returns
/// the process exit code.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    let code = match args.split_first() {
        None => {
            print_command_help(host);
            0
        }
        Some((cmd, rest)) => match cmd.to_lowercase().as_str() {
            // PObserve is its own tool upstream: its output has no P tool banner
            "observe" => return observe::run(rest, host),
            "check" => check::run(rest, host),
            "merge-scenario-coverage" => scenario::run_merge(rest, host),
            "version" | "-v" | "-version" | "--version" => {
                host.out_line(&format!("P version {}", VERSION));
                0
            }
            "help" | "-h" | "--help" => {
                print_command_help(host);
                0
            }
            _ => {
                host.out_colored_line(
                    &format!(
                        "Expected (check | observe | merge-scenario-coverage) as the command input but received `{}`",
                        cmd
                    ),
                    Color::Red,
                );
                print_command_help(host);
                0
            }
        },
    };
    host.out_line("~~ [PTool]: Thanks for using P! ~~");
    code
}

fn print_command_help(host: &mut dyn Host) {
    for l in [
        "================================================================================",
        "The P commandline tool supports the commands: check, observe and merge-scenario-coverage.\n",
        "usage:> p command options",
        "\t command : check                     run the P checker on the P sources",
        "\t           observe                   check recorded logs against spec monitors",
        "\t           merge-scenario-coverage   merge scenario coverage reports",
        "\t options:                             use `--help` or `-h` to learn more about the",
        "\t                                      corresponding command options",
        "\t -----------------------------------------------------------------------",
        "\t p check --help                       for P checker help",
        "\t p observe --help                     for P observe help",
        "================================================================================",
    ] {
        host.out_line(l);
    }
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
