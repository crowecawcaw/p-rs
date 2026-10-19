# prs-test

Supported in-process P monitor integration for native Rust tests. Compile a spec,
connect its recorder to existing application events, and explicitly check after setup,
the test body, and cleanup. No CLI subprocess, custom parser, or generated P model is
required. The runnable [inventory examples](../../examples/rust) show explicit checks,
a shared checked fixture, and proptest sequences.

These crates are currently distributed from source. In another repository, pin the
repository Git revision in your dev-dependency; the published `p-rs` crate is a placeholder.

```toml
[dev-dependencies]
prs-test = { git = "https://github.com/crowecawcaw/p-rs", rev = "<commit>" }
```

```rust
use prs_test::{json, Model};

let model = Model::compile(&[("inventory.p", r#"
    event Stock: int;
    spec Inventory observes Stock {
        start state Ready {
            on Stock do (n: int) { assert n >= 0, "negative stock"; }
        }
    }
"#)], "Inventory")?;
let recording = model.recording();
let sink = recording.sink();
// Connect this sink before seeding your application. Publish after commit.
sink.emit("coffee", "Stock", json!(3))?;
recording.check()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Model` owns the source map and compiled declarations. Reuse it between tests (for
example through `OnceLock`); semantic compilation happens once. Interpreter bytecode
preparation currently repeats at each check. Every recording has fresh monitor state,
with one monitor instance per partition key. Strict mode is on by default: zero relevant
verified events fails. Opt out with `.strict(false)` when intentional. `.check_hot(true)`
separately rejects monitor instances left in hot states; strictness does not enable
liveness checking.

`EventSink` is cloneable, `Send`, and `Sync`. It buffers owned JSON data under a mutex;
P values are constructed only on the checking thread. Both the timestamp assignment
and insertion happen under the lock. `emit` records current epoch milliseconds;
`emit_at` accepts explicit epoch milliseconds for deterministic or timed tests. Events
are checked in emission order, and decreasing timestamps for a key fail. Coordinate
application operations and publish after they commit; a sink lock does not make the
application's transactions atomic.

Payloads follow `prs observe` JSON conversion: scalar JSON for scalar events, objects
or positional arrays for named tuples, arrays for ordinary tuples, and `null` for unit
events. Names absent from the P model and events not observed by the selected spec
are ignored before payload conversion, as in `prs observe`. This lets applications
connect a shared event stream. An invalid payload on a relevant event fails at `check`.
Strict mode detects a trace with no verified relevant events; it cannot detect partially
missing events or every typo when other relevant events remain. Use explicit counts or
application assertions when a test expects a particular event. Emission itself returns
a lifecycle error if checking has already closed the recorder.

Finish cleanup and join producer threads before calling `Recording::check`. It consumes
the recording and closes its cloned sinks; later emissions return errors. There is no
assertion in `Drop`. `Model::with_recording` is a small closure helper: perform setup,
the body, and explicit cleanup inside the closure. It checks on normal return; an
application panic propagates unchanged without a secondary check during unwinding.

`check` returns a `CheckReport`, or a `CheckError` containing the failed report, readable
assertion/parser diagnostics and replay, and owned JSON event history through `events()`.
The failed report remains accessible through `error.report()` for coverage reporting.
Ordinary application assertions still matter: a monitor cannot see uninstrumented state
changes, and these buffered checks do not stop producer threads while they run.

## Optional suite coverage

`Suite` unions coverage through the core report API. Add either result before propagating
its failure; never let a reporting error hide the original application/monitor error.

```rust,ignore
let checked = recording.check();
let report = match &checked {
    Ok(report) => report,
    Err(error) => error.report(),
};
let coverage = suite.add(report)
    .and_then(|_| suite.write_json("target/coverage/model.json"));
println!("{}", suite.render_text());
if let Err(error) = &coverage { eprintln!("coverage reporting failed: {error}"); }
checked?; // Preserve the monitor failure if reporting also failed.
coverage?;
```

The JSON format is the same as `prs coverage merge-observe`. Output directories are
created, incompatible source versions are rejected atomically, and failed observation
metadata remains visible. Coverage reports measure observed monitor states and handlers,
not full model paths or schedules. Completion for this fully consumed in-process check
is not currently represented by the core `Report`; suite output labels it unknown.

Aggregation is explicit. It retains coverage sets and distinct observation metadata,
including errors, but not complete successful event traces. Use it for the ordinary
shared fixture. Generated tests should check and discard each case and shrink attempt,
retaining the final failure through the property-testing framework instead of collecting
all case reports in a suite. A `Suite` is not an automatic cross-process global: use one
per test target/process, and merge saved reports with `prs coverage merge-observe` if
multiple Cargo test binaries collect coverage.
