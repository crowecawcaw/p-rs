//! Type-checks every regression test under the given roots and compares the outcome with
//! the directory category (`Correct`/`DynamicError` must type-check, `StaticError` must not).
//! Prints mismatches and, with `-v`, every diagnostic. Exits with status 1 on any mismatch.
use std::path::Path;

#[path = "../tests/common/corpus.rs"]
mod corpus;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verbose = args.iter().any(|a| a == "-v");
    let roots: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if roots.is_empty() {
        eprintln!("usage: typecheck_corpus [-v] <regression test root>...");
        std::process::exit(2);
    }
    let (mut ok, mut mismatched, mut param_ok, mut param_bad) = (0, 0, 0, 0);
    for root in roots {
        for t in corpus::run(Path::new(root)) {
            if t.ok {
                ok += 1;
                if verbose {
                    println!("OK   {} ({})", t.dir.display(), t.category);
                }
            } else {
                mismatched += 1;
                println!(
                    "MISMATCH {} ({}): {}",
                    t.dir.display(),
                    t.category,
                    if t.failed { "failed" } else { "passed" }
                );
            }
            if verbose || !t.ok {
                for m in &t.messages {
                    println!("     {m}");
                }
            }
            if let Some((want, got)) = &t.params {
                if want == got {
                    param_ok += 1;
                } else {
                    param_bad += 1;
                    println!("PARAM MISMATCH {}", t.dir.display());
                    for w in want.iter().filter(|w| !got.contains(w)) {
                        println!("     missing  {w}");
                    }
                    for g in got.iter().filter(|g| !want.contains(g)) {
                        println!("     extra    {g}");
                    }
                }
            }
        }
    }
    println!("ok={ok} mismatched={mismatched} param_ok={param_ok} param_bad={param_bad}");
    if mismatched > 0 || param_bad > 0 {
        std::process::exit(1);
    }
}
