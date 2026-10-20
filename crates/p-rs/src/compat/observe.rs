//! `p observe`: upstream's PObserve command line, run by [`crate::driver::observe`].

use crate::checker::observe::Parser;
use crate::host::Host;

pub use crate::driver::observe::{
    EXIT_ASSERTION, EXIT_CMDLINE_ERROR, EXIT_INTERNAL_ERROR, EXIT_PARSE_LOG_ERROR, EXIT_SUCCESS,
};

/// Runs `p observe` with the built-in parsers.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    run_with(args, host, Vec::new())
}

/// Runs `p observe` with additional parsers (selected by `--parser <name>`). When `--parser`
/// is not given and exactly one extra parser is passed, that one is used.
pub fn run_with(args: &[String], host: &mut dyn Host, parsers: Vec<Box<dyn Parser>>) -> i32 {
    crate::driver::observe::run(args, host, parsers, "p observe")
}
