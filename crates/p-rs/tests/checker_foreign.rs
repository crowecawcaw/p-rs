//! The PriorityQueue tutorial with its C# foreign code ported to Rust.
use std::any::Any;
use std::path::Path;
use std::rc::Rc;

use prs::checker::engine::{Checker, CheckerOptions};
use prs::checker::foreign::{foreign, ForeignCx, ForeignError, ForeignFunctions, ForeignValue};
use prs::checker::value::{NamedTuple, Value};

/// `tPriorityQueue`: elements with priorities, in insertion order.
#[derive(Debug, Clone, Default)]
struct PriorityQueue(Vec<(Value, i64)>);

impl ForeignValue for PriorityQueue {
    fn eq_value(&self, other: &dyn ForeignValue) -> bool {
        other.as_any().downcast_ref::<PriorityQueue>().map(|o| o.0 == self.0).unwrap_or(false)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn queue(v: &Value) -> PriorityQueue {
    match v {
        Value::Foreign(f) => f.as_any().downcast_ref::<PriorityQueue>().cloned().unwrap_or_default(),
        _ => PriorityQueue::default(),
    }
}

struct Functions;

impl ForeignFunctions for Functions {
    fn call(
        &self,
        name: &str,
        _owner: Option<&str>,
        args: &[Value],
        cx: &mut ForeignCx,
    ) -> Option<Result<Value, ForeignError>> {
        Some(match name {
            "CreatePriorityQueue" => {
                cx.log_line("Creating Priority Queue!");
                Ok(foreign(PriorityQueue::default()))
            }
            "AddElement" | "AddIntToQueue" => {
                let mut q = queue(&args[0]);
                q.0.push((args[1].clone(), args[2].as_int()));
                if name == "AddElement" {
                    cx.log_line("Adding Element in the Priority Queue!");
                }
                Ok(foreign(q))
            }
            "RemoveElement" => {
                let mut q = queue(&args[0]);
                let (i, _) = q.0.iter().enumerate().min_by_key(|(_, (_, p))| *p).unwrap();
                let (e, _) = q.0.remove(i);
                Ok(Value::NamedTuple(Rc::new(NamedTuple {
                    names: Rc::from(vec!["element".to_string(), "queue".to_string()]),
                    values: vec![e, foreign(q)],
                })))
            }
            "CountElement" => Ok(Value::Int(queue(&args[0]).0.len() as i64)),
            "ChooseElement" => {
                let q = queue(&args[0]);
                match cx.random_int(q.0.len() as i32) {
                    Ok(i) => {
                        cx.log_line(&format!("Choosing element at location: {}", i));
                        Ok(q.0[i as usize].0.clone())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => return None,
        })
    }
}

fn load() -> prs::sema::Analysis {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/upstream/Tutorial/PriorityQueue");
    let mut sm = prs::syntax::SourceMap::new();
    for sub in ["PSrc", "PTst"] {
        let mut files: Vec<_> = std::fs::read_dir(dir.join(sub)).unwrap().map(|e| e.unwrap().path()).collect();
        files.sort();
        for f in files {
            sm.add(f.display().to_string(), std::fs::read_to_string(&f).unwrap());
        }
    }
    prs::sema::compile(&sm, &prs::sema::Options::default()).unwrap_or_else(|_| panic!("compile"))
}

#[test]
fn priority_queue_with_rust_foreign_functions() {
    let a = load();
    let tc = a.program.test_cases().into_iter().find(|t| t.name == "tcCheck_PriorityQueue").unwrap();
    let checker = Checker::new(&a.program).with_foreign(Rc::new(Functions));
    let opts = CheckerOptions { iterations: 100, log_every_schedule: true, ..Default::default() };
    let r = checker.check(&tc, &opts, &mut Vec::<String>::new()).unwrap();
    assert_eq!(r.report.num_bugs, 0, "{:?}", r.report.bug_reports);
    let prints: Vec<&str> = r
        .last_log
        .iter()
        .filter_map(|l| l.strip_prefix("<PrintLog> "))
        .filter(|l| !l.contains("Queue") && !l.starts_with("Choosing"))
        .collect();
    assert_eq!(prints, ["--------------", "Hello", "World", "!!", "123", "--------------"]);
    assert!(r.last_log.iter().any(|l| l.starts_with("<RandomLog> Client(1) nondeterministically chose")));
}

#[test]
fn missing_foreign_function_is_reported() {
    let a = load();
    let tc = a.program.test_cases().into_iter().find(|t| t.name == "tcCheck_PriorityQueue").unwrap();
    let r = Checker::new(&a.program).check(&tc, &CheckerOptions::default(), &mut Vec::<String>::new()).unwrap();
    assert_eq!(r.report.num_bugs, 1);
    assert!(r.report.bug_reports[0].contains("System.NotImplementedException"), "{}", r.report.bug_reports[0]);
}

#[test]
fn random_int_with_a_negative_maximum_throws() {
    struct Negative;
    impl ForeignFunctions for Negative {
        fn call(
            &self,
            name: &str,
            _owner: Option<&str>,
            _args: &[Value],
            cx: &mut ForeignCx,
        ) -> Option<Result<Value, ForeignError>> {
            (name == "Pick").then(|| cx.random_int(-1).map(|i| Value::Int(i as i64)))
        }
    }
    let mut sm = prs::syntax::SourceMap::new();
    sm.add(
        "model.p",
        "fun Pick(): int;
         machine Main { start state S { entry { Pick(); } } }
         test tc [main=Main]: { Main };"
            .to_string(),
    );
    let a = prs::sema::compile(&sm, &prs::sema::Options::default()).unwrap_or_else(|_| panic!("compile"));
    let tc = a.program.test_cases().into_iter().next().unwrap();
    let checker = Checker::new(&a.program).with_foreign(Rc::new(Negative));
    let r = checker.check(&tc, &CheckerOptions::default(), &mut Vec::<String>::new()).unwrap();
    assert_eq!(r.report.num_bugs, 1);
    let bug = &r.report.bug_reports[0];
    assert!(bug.starts_with("Exception 'System.ArgumentOutOfRangeException' was thrown in Main(1)"), "{}", bug);
    assert!(bug.contains("maxValue ('-1') must be a non-negative value."), "{}", bug);
    assert!(bug.contains("from location 'System.Private.CoreLib'"), "{}", bug);
}
