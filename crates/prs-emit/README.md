# prs-emit

Emit application events once, then check them against P specs in tests (with
[`prs-test`](../prs-test)) and in production (with `prs observe`). Depends only on
`serde` / `serde_json`.

```rust
use prs_emit::{emit, json};

// After the reservation commits:
emit("eReserved", "coffee", json!({"order": "o1", "units": 2}));
```

`emit(event, key, payload)` takes the P event name, the partition key (anything
`Display`; each key gets its own spec instance) and a `Serialize` payload: `json!`, a
`#[derive(Serialize)]` struct, a scalar, or `()` for none. `emit_at(time, ...)` sets the
time (a `SystemTime`, or `i64` epoch nanoseconds for a logical clock). The call site is
recorded as `src` and shown in failure reports.

Where `emit` sends events:

| Process | Events go to |
| --- | --- |
| A test with a `prs-test` recording | that recording (`prs_emit::test_hook`) |
| Started by a test (`PRS_EVENTS_DIR` set) | its own `<name>-<pid>-<random>.jsonl` in that folder |
| After `prs_emit::set_sink(...)` | the sink: JSON lines `prs observe` reads |
| Otherwise | nowhere: one atomic load |

```rust,no_run
use prs_emit::Emitter;
use std::time::Duration;

prs_emit::set_sink(Emitter::file("/var/log/app/events/inventory.jsonl")?);
// or Emitter::stdout(), Emitter::writer(w), Emitter::callback(|event| ...)
// ... at shutdown:
prs_emit::close(); // writes what is queued; prs_emit::flush(timeout) only waits
# Ok::<(), std::io::Error>(())
```

```sh
prs observe -pf specs/inventory.p --spec InventoryCorrect -l '/var/log/app/events/*.jsonl'
```

`set_sink` replaces the earlier sink (`Emitter::off()` turns it off) and does nothing
in a process a test started. [Testing and production](../../docs/production.md) covers
deploying this.

An `Emitter` is also a value: code that takes one (instead of calling the global
`emit`) gets `recording.emitter()` in tests and a production `Emitter` otherwise;
clones share a destination. `Emitter::callback(f)` calls `f` with each `Event` on the
emitting thread, under the emitter's lock, so `f` must be quick; the other production
sinks write from a background thread.

Emitting never panics, returns an error, or blocks on I/O. Production writers queue up
to 16,384 lines and drop events when the queue is full or a write fails;
`prs_emit::dropped()` (or `Emitter::dropped()`) counts them. When off, `emit` costs one atomic load, but the
payload is still built: guard costly payloads with `is_on()`. The events folder writer
appends each line synchronously, so a child's events are on disk when it exits.

Lines follow the [events folder format](../../docs/observe.md#events-folder-format):
`timestamp` is ISO-8601 with nanoseconds, from a wall clock that never goes back within a
process, plus `src`, `seq` and `pid`.
