//! Generated class names follow upstream's shared name pool.

fn machines(src: &str) -> Vec<(String, String)> {
    let mut sm = pnext_syntax::SourceMap::new();
    sm.add("t.p".to_string(), src.to_string());
    let a = pnext_sema::compile(&sm, &pnext_sema::Options::default()).expect("compiles");
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
