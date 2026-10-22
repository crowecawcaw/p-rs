//! The `snappea` binary: [`snappea::cli::run`] on the native file system and console.

use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = snappea::cli::NativeHost::new();
    let code = snappea::cli::run(&args, &mut host);
    let _ = io::stdout().flush();
    std::process::exit(code);
}
