//! Prints each machine's generated class name and the sorted action names its states
//! reference, in the shape `scripts/csnames-upstream.sh` extracts from upstream's C#.
use pnext_sema::{HandlerAction, Options};

fn main() {
    let mut sm = pnext_syntax::SourceMap::new();
    for f in std::env::args().skip(1) {
        sm.add(f.clone(), std::fs::read_to_string(&f).expect("read"));
    }
    let a = pnext_sema::compile(&sm, &Options::default()).unwrap_or_else(|_| std::process::exit(1));
    let p = &a.program;
    let mut lines = Vec::new();
    for m in &p.machines {
        let mut acts = Vec::new();
        for &s in &m.states {
            let st = p.state(s);
            acts.extend(st.entry.map(|f| p.fun(f).action_name()));
            for h in &st.handlers {
                match h.action {
                    HandlerAction::Do(f) | HandlerAction::Goto { with: Some(f), .. } => {
                        acts.push(p.fun(f).action_name())
                    }
                    _ => {}
                }
            }
            acts.extend(st.exit.map(|f| p.fun(f).action_name()));
        }
        acts.sort();
        lines.push(format!("{}: {}", m.cs_name, acts.join(" ")));
    }
    lines.sort();
    for l in lines {
        println!("{l}");
    }
}
