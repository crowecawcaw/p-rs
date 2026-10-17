//! `p check --workers`: schedules explored by several threads.

use std::path::Path;

use prs_compat::host::{Host, Job, MemHost, Threads};

fn run_threads<'a>(jobs: Vec<Job<'a>>, main: &mut (dyn FnMut() + 'a)) {
    std::thread::scope(|s| {
        for job in jobs {
            s.spawn(job);
        }
        main();
    });
}

/// A host with threads; four workers by default.
fn host() -> MemHost {
    let mut h = MemHost::new("/w");
    h.threads = Some(Threads { available: 4, run: run_threads });
    h.add_file(
        "coins.p",
        "machine M {\n\
           start state S {\n\
             entry {\n\
               var n: int;\n\
               while (n < 6) {\n\
                 if ($) { n = n + 1; } else { goto Done; }\n\
               }\n\
               assert false, \"six heads\";\n\
             }\n\
           }\n\
           state Done { }\n\
         }\n\
         test tcCoins [main=M]: { M };\n",
    );
    h
}

fn run(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let mut all = vec!["check", "coins.p", "-pn", "Coins"];
    all.extend_from_slice(args);
    let args: Vec<String> = all.iter().map(|s| s.to_string()).collect();
    prs_compat::run(&args, host)
}

fn read(h: &MemHost, path: &str) -> String {
    h.read_to_string(Path::new(path)).unwrap_or_else(|e| panic!("{}: {}", path, e))
}

/// Every file under `dir`, with contents.
fn files(h: &MemHost, dir: &str) -> Vec<(String, String)> {
    h.files_under(dir).iter().map(|p| (p.display().to_string(), h.read_to_string(p).unwrap())).collect()
}

#[test]
fn one_worker_is_a_host_without_threads() {
    for args in
        [&["-s", "500", "--seed", "3"][..], &["-s", "20", "--seed", "1", "--sch-pct", "2"], &["--explore", "-s", "300"]]
    {
        let mut threads = host();
        let mut plain = host();
        plain.threads = None;
        let mut with_flag = args.to_vec();
        with_flag.extend(["--workers", "1"]);
        assert_eq!(run(&mut threads, &with_flag), run(&mut plain, args));
        assert_eq!(threads.stdout, plain.stdout);
        assert_eq!(files(&threads, "/w"), files(&plain, "/w"));
        // and so does `--workers 4` on a host without threads
        with_flag.pop();
        with_flag.push("4");
        let mut plain4 = host();
        plain4.threads = None;
        run(&mut plain4, &with_flag);
        assert_eq!(plain4.stdout, plain.stdout);
    }
}

#[test]
fn workers_find_a_bug_that_replays() {
    for seed in ["1", "2", "3", "4", "5"] {
        let mut h = host();
        assert_eq!(run(&mut h, &["-s", "5000", "--seed", seed]), 1, "{}", h.stdout);
        assert!(h.stdout.contains("... Checker is using 4 workers.\n"), "{}", h.stdout);
        assert!(h.stdout.contains("Checker found a bug."), "{}", h.stdout);
        let dir = "PCheckerOutput/BugFinding";
        let txt = read(&h, &format!("{}/Coins_0_0.txt", dir));
        assert!(txt.contains("<ErrorLog> Assertion Failed: coins.p:8:1 six heads"), "{}", txt);
        // the trace ends with the statistics of all workers, as the console reports them
        let explored = h.stdout.lines().find(|l| l.starts_with("..... Explored") && l.ends_with("schedules")).unwrap();
        assert!(txt.contains(&explored.replace(".....", "<StrategyLog>")), "{}\n{}", explored, txt);
        let summary = read(&h, &format!("{}/Coins_pchecker_summary.txt", dir));
        let n: usize = explored.split(' ').nth(2).unwrap().parse().unwrap();
        assert!(summary.contains(&format!("\nschedules:{}\n", n)), "{}", summary);
        assert!(n < 5000);
        assert_eq!(run(&mut h, &["--replay", &format!("{}/Coins_0_0.schedule", dir)]), 0, "{}", h.stdout);
        assert!(h.stdout.contains("... Reproduced 1 bug."), "{}", h.stdout);
    }
}

#[test]
fn workers_share_the_schedule_count() {
    let mut h = host();
    // a check without bug: no six heads in a row
    h.add_file("coins.p", &read(&h, "coins.p").replace("n < 6", "n < 60"));
    assert_eq!(run(&mut h, &["-s", "1003", "--seed", "9"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 1003 schedules\n"), "{}", h.stdout);
    // progress is numbered across workers, each number once
    let progress: Vec<&str> = h.stdout.lines().filter(|l| l.starts_with("..... Schedule #")).collect();
    let mut sorted = progress.clone();
    sorted.sort_by_key(|l| l[16..].parse::<usize>().unwrap());
    sorted.dedup();
    assert_eq!(sorted.len(), 28, "{:?}", progress);
    assert!(read(&h, "PCheckerOutput/BugFinding/Coins_pchecker_summary.txt").contains("\nschedules:1003\n"));
    // the coverage of all workers: every event handled
    assert!(read(&h, "PCheckerOutput/BugFinding/Coins.coverage.txt").starts_with("Total event coverage: 100.0%"));
    // fewer schedules than workers
    assert_eq!(run(&mut h, &["-s", "2", "--seed", "9"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("... Checker is using 2 workers.\n"), "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 2 schedules\n"), "{}", h.stdout);
}

#[test]
fn some_runs_use_one_worker() {
    let mut h = host();
    for args in [
        &["-s", "1"][..],
        &["-s", "100", "--sch-feedback"],
        &["-s", "100", "--sch-rl"],
        &["--mode", "pex"],
        &["-s", "5", "-v"],
    ] {
        run(&mut h, args);
        assert!(!h.stdout.contains("workers."), "{:?}: {}", args, h.stdout);
    }
    assert_eq!(run(&mut h, &["-s", "5", "--workers", "0"]), 1);
    assert!(h.stderr.contains("The option '--workers' needs at least 1 worker."), "{}", h.stderr);
}

#[test]
fn explore_numbers_bugs_across_workers() {
    let mut h = host();
    assert_eq!(run(&mut h, &["-s", "2000", "--seed", "5", "--explore"]), 1, "{}", h.stdout);
    let bugs: Vec<&str> = h.stdout.lines().filter(|l| l.contains(" triggered bug #")).collect();
    let found = h.stdout.lines().find(|l| l.starts_with("..... Found ") && l.ends_with(" bugs.")).unwrap();
    assert_eq!(found, format!("..... Found {} bugs.", bugs.len()), "{}", h.stdout);
    assert!(bugs.iter().any(|l| !l.ends_with("[task-0]")), "{:?}", bugs);
    assert!(h.stdout.contains("..... Explored 2000 schedules\n"), "{}", h.stdout);
}
