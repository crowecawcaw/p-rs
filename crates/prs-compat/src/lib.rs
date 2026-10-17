//! Drop-in compatibility with the upstream P command line tool.
//!
//! Reproduces `p check` from `PCommandLine`: the same options, help text, project-file
//! handling, console messages, output files and exit codes, on top of the pure-Rust type
//! checker ([`prs_sema`]) and checker ([`prs_checker`]). There is no `p compile`: `p check`
//! reads the P sources directly. Everything goes
//! through a [`Host`](host::Host), so the commands also run in-memory (tests, browsers).
//!
//! Known differences from upstream are listed in the crate README.

pub mod argparse;
pub mod check;
pub mod host;
pub mod observe;
pub mod pproj;
pub mod project;
pub mod scenario;
pub mod sci;

use host::{Color, Host, HostExt, Stream};

/// Process exit requested (`Environment.Exit(code)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit(pub i32);

/// `Error.ReportAndExit`: `Error: {msg}` on stderr, exit code 1.
pub fn report_and_exit(host: &mut dyn Host, msg: &str) -> Exit {
    report(host, msg);
    Exit(1)
}

/// `Error.Report`: `Error: {msg}` on stderr.
pub fn report(host: &mut dyn Host, msg: &str) {
    host.write(Stream::Err, "Error: ", Some(Color::Red));
    host.write(Stream::Err, msg, Some(Color::Yellow));
    host.write(Stream::Err, "\n", None);
}

/// The version reported by `p --version`.
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (prs)");

/// `CommandLine.Main`: runs the `p` tool with `args` (without the program name) and returns
/// the process exit code.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    // PObserve is its own tool upstream: its output has no P tool banner
    if args.first().map(|a| a.to_lowercase()).as_deref() == Some("observe") {
        return observe::run(&args[1..], host);
    }
    let code = match args.first().map(|a| a.to_lowercase()) {
        None => {
            print_command_help(host);
            0
        }
        Some(cmd) => match cmd.as_str() {
            "compile" => report_and_exit(
                host,
                "There is no `compile` command: prs checks P sources directly. Run `p check` (it reads the *.pproj or *.p files).",
            )
            .0,
            "check" => check::run(&args[1..], host),
            "merge-scenario-coverage" => scenario::run_merge(&args[1..], host),
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
                    &format!("Expected (check | observe | merge-scenario-coverage) as the command input but received `{}`", args[0]),
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
pub fn format_0_00(v: f64) -> String {
    let s = format!("{:.2}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}
