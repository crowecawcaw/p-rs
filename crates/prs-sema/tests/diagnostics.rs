//! Type errors on small programs.

fn errors(src: &str) -> Vec<String> {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("t.p", src);
    match prs_sema::compile(&sm, &prs_sema::Options::default()) {
        Ok(_) => Vec::new(),
        Err(prs_sema::CompileError::Analysis(a)) => a.messages(&sm),
        Err(e) => panic!("not a type error: {e:?}"),
    }
}

#[test]
fn format_indices_wrap_like_csharp_int() {
    // 4294967296 wraps to 0, so one argument is expected.
    assert_eq!(errors(r#"fun f() { print format("{4294967296}", 1); }"#), Vec::<String>::new());
    // 99999999999999999999 wraps to 1661992959.
    assert_eq!(
        errors(r#"fun f() { print format("{99999999999999999999}", 1); }"#),
        ["[t.p:1:17] goto, function or constructor call expected 1661992960 arguments, got 1"]
    );
}

#[test]
fn pairwise_needs_two_parameters() {
    let src = "param x: int;
               machine Main { start state S { } }
               test param (x in [1, 2]) pairwise tc [main=Main]: { Main };";
    assert_eq!(errors(src), ["[t.p:3:41] invalid twise number at t.p:3:16: twise number 2 is greater than 1."]);
}
