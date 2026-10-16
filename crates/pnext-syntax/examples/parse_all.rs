//! Parses every `.p` file under the given directories and reports failures.
use std::path::Path;

fn main() {
    let mut ok = 0;
    let mut failed = Vec::new();
    for root in std::env::args().skip(1) {
        walk(Path::new(&root), &mut ok, &mut failed);
    }
    for (f, e) in &failed {
        println!("FAIL {}: {}", f, e);
    }
    println!("ok={} failed={}", ok, failed.len());
}

fn walk(p: &Path, ok: &mut usize, failed: &mut Vec<(String, String)>) {
    if p.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(p).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for e in entries {
            walk(&e, ok, failed);
        }
    } else if p.extension().map(|e| e == "p").unwrap_or(false) {
        let text = String::from_utf8_lossy(&std::fs::read(p).unwrap()).to_string();
        let mut sm = pnext_syntax::SourceMap::new();
        let id = sm.add(p.display().to_string(), text.clone());
        match pnext_syntax::parse(id, &text) {
            Ok(_) => *ok += 1,
            Err(e) => {
                let loc = sm.location(e.span);
                failed.push((p.display().to_string(), format!("{} {}", loc, e.message)));
            }
        }
    }
}
