//! The `prs-compat` binary: a drop-in replacement for upstream's `p` tool
//! ([`prs_compat::run`]) on the native file system and console.

use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = prs_cli::NativeHost::new();
    let code = prs_compat::run(&args, &mut host);
    let _ = io::stdout().flush();
    std::process::exit(code);
}
