//! Drop-in compatibility with the upstream P command line tools.
//!
//! [`run`] reproduces `p check` and `p merge-scenario-coverage` from `PCommandLine`, and
//! [`observe`] the PObserve command line: the same options, help text, project-file
//! handling, console messages, output files and exit codes. This module is only the
//! adapter: it parses upstream's options and runs the commands of [`crate::driver`], on
//! top of the pure-Rust type checker ([`crate::sema`]) and checker ([`crate::checker`]).
//! There is no `p compile`: the commands read the P sources directly. Everything goes
//! through a [`Host`], so the commands also run in-memory (tests, browsers).
//!
//! Nothing outside this module depends on it (`tests/layering.rs` checks this).
//!
//! Known differences from upstream are listed in the repository README.

mod argparse;
pub mod check;
pub mod observe;
pub mod scenario;

use crate::host::{Color, Host, HostExt};

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
