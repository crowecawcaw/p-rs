//! Deeply nested models: up to `MAX_NESTING` levels they are parsed, checked and
//! run; deeper ones are a parse error, never a stack overflow.

use prs_compat::host::MemHost;
use prs_syntax::MAX_NESTING;

/// Runs `p check` on `model` on a stack of the size the WebAssembly build has
/// (1 MiB) or, for the much larger frames of a debug build, a native main
/// thread's (8 MiB). Returns the exit code and the console output.
fn check(model: String) -> (i32, String) {
    let size = if cfg!(debug_assertions) { 8 << 20 } else { 1 << 20 };
    let run = move || {
        let mut h = MemHost::new("/w");
        h.add_file("m.p", &model);
        let args: Vec<String> = ["check", "m.p", "-s", "2"].iter().map(|s| s.to_string()).collect();
        let code = prs_compat::run(&args, &mut h);
        (code, h.stdout)
    };
    std::thread::Builder::new().stack_size(size).spawn(run).unwrap().join().unwrap()
}

/// A model whose start state runs `body`, with `x: int` and `s: seq[int]` in scope.
fn model(body: &str) -> String {
    format!(
        "machine Main {{
           var x: int; var s: seq[int];
           fun f(a: int): int {{ return a; }}
           start state S {{ entry {{ s += (0, 0); {body} }} }}
         }}"
    )
}

/// The models of `deep_models_are_checked`, `k` levels deep.
fn shapes(k: usize) -> Vec<(&'static str, String)> {
    let r = |s: &str, n: usize| s.repeat(n);
    vec![
        ("parentheses", model(&format!("x = {}1{}; assert x == 1;", r("(", k), r(")", k)))),
        ("sum", model(&format!("x = 1{}; assert x == {};", r(" + 1", k), k + 1))),
        ("casts", model(&format!("x = 1{}; assert x == 1;", r(" as int", k)))),
        ("negations", model(&format!("x = {}1; assert x == 1;", r("- -", k / 2)))),
        ("calls", model(&format!("x = {}1{}; assert x == 1;", r("f(", k), r(")", k)))),
        ("indexing", model(&format!("x = {}0{}; assert x == 0;", r("s[", k / 2), r("]", k / 2)))),
        ("blocks", model(&format!("{}x = 1;{} assert x == 1;", r("{", k), r("}", k)))),
        ("ifs", model(&format!("{}x = 1; assert x == 1;", r("if (x == 0) ", k)))),
        ("else ifs", model(&format!("{}x = 2; assert x == 2;", r("if (x == 1) x = 3; else ", k)))),
        ("whiles", model(&format!("{}x = 1; assert x == 1;", r("while (x == 0) ", k)))),
        (
            "types",
            format!(
                "machine Main {{ var v: {}int{}; start state S {{ entry {{ assert sizeof(v) == 0; }} }} }}",
                r("seq[", k),
                r("]", k)
            ),
        ),
    ]
}

#[test]
fn deep_models_are_checked() {
    // The statement holding the nesting, the expression in it and, for the casts,
    // the type of the innermost one take up the other three levels.
    for (shape, src) in shapes(MAX_NESTING - 3) {
        let (code, out) = check(src);
        assert_eq!(code, 0, "{shape}: {out}");
        assert!(out.contains("Found 0 bugs."), "{shape}: {out}");
    }
}

#[test]
fn deeper_models_are_a_parse_error() {
    let too_deep = format!("nesting is too deep (more than {MAX_NESTING} levels)");
    let mut shapes = shapes(2000);
    shapes.push(("long sum", model(&format!("x = 1{};", " + 1".repeat(20_000)))));
    for (shape, src) in shapes {
        let (code, out) = check(src);
        assert_eq!(code, 1, "{shape}: {out}");
        assert!(out.contains("[Parser Error:] [m.p] parse error: line "), "{shape}: {out}");
        assert!(out.contains(&too_deep), "{shape}: {out}");
    }
    // Reported at the start of the first expression past the limit: the statement
    // and the expression assigned take two levels, the one in each parenthesis one more.
    let src = model(&format!("x = {}1{};", "(".repeat(2000), ")".repeat(2000)));
    let col = src.lines().nth(3).unwrap().find("x = (").unwrap() + 4 + MAX_NESTING - 1;
    let (_, out) = check(src);
    assert!(out.contains(&format!("parse error: line 4:{col} {too_deep}")), "{out}");
}
