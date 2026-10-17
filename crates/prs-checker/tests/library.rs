//! The example in `docs/library.md`, kept compiling and running.

use prs_checker::engine::{Checker, CheckerOptions};

const MODEL: &str = "
    event ePing;
    machine Main {
        start state Init {
            entry { send this, ePing; }
            on ePing do { assert false, \"pinged\"; }
        }
    }
    test tcMain [main = Main]: { Main };
";

#[test]
fn library_example() {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("model.p", MODEL);
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    let tc = analysis.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
    let checker = Checker::new(&analysis.program);
    let opts = CheckerOptions { iterations: 1000, seed: 42, ..Default::default() };
    let result = checker.check(&tc, &opts, &mut Vec::<String>::new()).expect("runs");
    assert_eq!(result.report.num_bugs, 1);
}

#[test]
fn observe_spec_by_name() {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("spec.p", "event eTick; spec MySpec observes eTick { start state S { on eTick do {} } }");
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    assert!(prs_checker::observe::Spec::new(&analysis.program, "MySpec").is_ok());
    assert!(prs_checker::observe::Spec::new(&analysis.program, "Missing").is_err());
}
