//! `p observe` with the parsers of upstream's PObserve regression tests registered, for
//! comparing against upstream PObserve (`scripts/compare-pobserve.sh`).
//!
//!   cargo run --release -p pnext-cli --example observe_regression -- --spec IncreasingInts \
//!       --parser TestCase4Parser -pp tests/upstream/PObserve/p_specs.pproj -l some.log

#[path = "../../pnext-compat/tests/support/pobserve_parsers.rs"]
mod pobserve_parsers;

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = pnext_cli::NativeHost::new();
    let code = pnext_compat::observe::run_with(&args, &mut host, pobserve_parsers::all());
    let _ = std::io::stdout().flush();
    std::process::exit(code);
}
