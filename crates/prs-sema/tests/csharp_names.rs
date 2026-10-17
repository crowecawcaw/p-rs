//! Generated class names follow upstream's shared name pool.

fn machines(src: &str) -> Vec<(String, String)> {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("t.p".to_string(), src.to_string());
    let a = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    a.program.machines.iter().map(|m| (m.name.clone(), m.cs_name.clone())).collect()
}

#[test]
fn field_claims_later_machine_name() {
    let got = machines(
        "machine Timer { var client: machine; start state Init { } }
         machine client { var Timer: int; start state Init { } }
         machine Other { start state Init { entry { var Third: int; } } }
         machine Third { start state Init { } }",
    );
    let cs: Vec<&str> = got.iter().map(|(_, c)| c.as_str()).collect();
    // `Timer` is generated before the field of the same name; `Third` after the local.
    assert_eq!(cs, ["Timer", "client_1", "Other", "Third_1"]);
}

#[test]
fn global_function_params_come_first() {
    let got = machines(
        "fun F(Main: int) { }
         machine Main { start state Init { } }",
    );
    assert_eq!(got[0].1, "Main_1");
}

#[test]
fn action_names_share_the_pool() {
    // Matches the methods upstream generates for this program: a field named `Anon` takes
    // the plain name, so anonymous handlers start at `Anon_1`.
    let mut sm = prs_syntax::SourceMap::new();
    sm.add(
        "t.p".to_string(),
        "machine Other { var Anon: int; start state Init { entry { } } }
         machine Main { start state Init { entry Foo; on e1 do { } } state S2 { entry { } } fun Foo() { } }
         event e1;"
            .to_string(),
    );
    let a = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    let mut names: Vec<String> = a.program.funs.iter().map(|f| f.action_name()).collect();
    names.sort();
    assert_eq!(names, ["Anon_1", "Anon_2", "Anon_3", "_Foo"]);
}
