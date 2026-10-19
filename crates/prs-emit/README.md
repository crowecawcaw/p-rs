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

Where events go is set once, by the process-wide [`Emitter`]:

| Emitter | Writes to | Use |
| --- | --- | --- |
| `Emitter::from_env()` (default) | `PRS_EVENTS_DIR` if set, else nothing | child processes started by a test |
| `Emitter::file(path)?`, `stdout()`, `writer(w)` | JSON lines, from a background thread | production |
| `Emitter::callback(f)` | your function | adapters, e.g. to `tracing` |
| `recording.emitter()` (`prs-test`) | an in-process test recording | tests |

```rust,no_run
prs_emit::set_global(prs_emit::Emitter::file("/var/log/app/events.jsonl")?).ok();
// ... at shutdown:
prs_emit::flush();
# Ok::<(), std::io::Error>(())
```

```sh
prs observe -pf specs/inventory.p --spec InventoryCorrect -l /var/log/app/events.jsonl
```

Code that tests construct with a recording takes an `Emitter` value instead of using the
global one; clones share a destination.

Emitting never panics, returns an error, or blocks on I/O. Production writers queue up
to 16,384 lines and drop events when the queue is full or a write fails;
`Emitter::dropped()` counts them. When off, `emit` costs one atomic load, but the
payload is still built: guard costly payloads with `is_on()`. The events folder writer
appends each line synchronously, so a child's events are on disk when it exits.

Lines follow the [events folder format](../../docs/observe.md#events-folder-format):
`timestamp` is ISO-8601 with nanoseconds, from a wall clock that never goes back within a
process, plus `src`, `seq` and `pid`.
