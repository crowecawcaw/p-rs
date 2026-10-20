//! `p merge-scenario-coverage [directory]`, run by [`crate::driver::scenario`].

use std::path::Path;

use crate::driver::scenario::{merge_dir, FILE_SUFFIX};
use crate::host::{full_path, Color, Host, HostExt, Stream};

/// `p merge-scenario-coverage [directory]`.
pub fn run_merge(args: &[String], host: &mut dyn Host) -> i32 {
    // upstream has no help and looks for a directory named `--help`
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        host.out_line("usage: p merge-scenario-coverage [directory]");
        host.out_line("");
        host.out_line(&format!(
            "Merges the scenario coverage reports (*{}) under the directory, recursively, into one report. The directory defaults to the current one.",
            FILE_SUFFIX
        ));
        return 0;
    }
    let dir = args.first().cloned().unwrap_or_else(|| host.cwd().to_string_lossy().to_string());
    let full = full_path(&host.cwd(), Path::new(&dir));
    if !host.is_dir(&full) {
        host.out_colored_line(&format!("Directory not found: {}", dir), Color::Red);
        return 0;
    }
    let report = merge_dir(host, &full);
    host.write(Stream::Out, &report, None);
    0
}
