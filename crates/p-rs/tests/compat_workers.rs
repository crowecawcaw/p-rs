//! `p check` explores schedules on one worker, as upstream does, even on a host with threads.

use prs::host::{Job, MemHost, Threads};

fn run_threads<'a>(jobs: Vec<Job<'a>>, main: &mut (dyn FnMut() + 'a)) {
    std::thread::scope(|s| {
        for job in jobs {
            s.spawn(job);
        }
        main();
    });
}

fn host(threads: bool) -> MemHost {
    let mut h = MemHost::new("/w");
    if threads {
        h.threads = Some(Threads { available: 4, run: run_threads });
    }
    h.add_file(
        "coins.p",
        "machine M { start state S { entry { var n: int; while (n < 60) { if ($) { n = n + 1; } else { goto Done; } } } } state Done { } }\n\
         test tcCoins [main=M]: { M };\n",
    );
    h
}

#[test]
fn one_worker_on_a_host_with_threads() {
    let args: Vec<String> = ["check", "coins.p", "-s", "500", "--seed", "3"].iter().map(|s| s.to_string()).collect();
    let mut threads = host(true);
    let mut plain = host(false);
    assert_eq!(prs::compat::run(&args, &mut threads), prs::compat::run(&args, &mut plain));
    assert!(!threads.stdout.contains("workers."), "{}", threads.stdout);
    assert_eq!(threads.stdout, plain.stdout);
}
