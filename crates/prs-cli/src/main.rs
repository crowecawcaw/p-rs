//! The `prs` binary: [`prs_cli::run`] on the native file system and console.

use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = prs_cli::NativeHost::new();
    let code = prs_cli::run(&args, &mut host);
    let _ = io::stdout().flush();
    std::process::exit(code);
}
