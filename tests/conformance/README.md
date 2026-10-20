# Helper conformance corpus

Shared cases every language helper replays through its own L1/L2 API (see
[docs/language-support.md](../../docs/language-support.md#conformance-corpus)). A helper
conforms when every case passes with no skips.

`crates/p-rs/tests/cli_conformance.rs` replays the same cases through `prs observe`
directly (part of `cargo test --workspace`), so the corpus always agrees with the core.

## Layout

```
<case>/
  spec.p          P declarations and spec machines; the only spec file
  events.jsonl    events emitted in the test process, in emit order
  children/*.jsonl  optional: files child processes wrote to PRS_EVENTS_DIR
  expect.json     options and expected outcome
```

Lines use the [events folder format](../../docs/observe.md#events-folder-format).

## Replaying a case

1. Configure a recording with `spec_files = [<case>/spec.p]` and the case's `options`.
   An omitted option takes the helper's default (below).
2. For each line of `events.jsonl`, in order, call `emit_at(timestamp, event, key,
   payload)`. `payload` is the JSON value as given (an object, a single value, or
   `null`).
3. Copy each `children/*.jsonl` file verbatim into the recording's events folder, or
   pass the `children` folder to `add_events_dir`.
4. Check, then compare the result with `expect`.

Skip a case whose `requires` names a feature the helper lacks, and print the reason.
Do not skip anything else.

## `expect.json`

```json
{
  "description": "One sentence: what the case shows.",
  "requires": ["events_dir"],
  "options": {
    "specs": ["InventoryCorrect"],
    "exclude": [],
    "strict": true,
    "check_hot": false,
    "unknown_events": "error"
  },
  "expect": {
    "outcome": "violation",
    "spec": "InventoryCorrect",
    "key": "p1",
    "message": "exceeds available stock",
    "liveness": false,
    "coverage": {
      "InventoryCorrect": {
        "states": { "covered": 1, "total": 1 },
        "handlers": { "covered": 2, "total": 2 }
      }
    }
  }
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `description` | yes | What the case shows. |
| `requires` | no | Features the case needs: `events_dir` (children files), `unknown_events` (the policy option). |
| `options.specs` | no | Specs to check. Default: every spec in `spec.p`. |
| `options.exclude` | no | Specs left out of the default selection. Default: none. |
| `options.strict` | no | Default: `true` when `specs` is given, otherwise `false`. |
| `options.check_hot` | no | Default: `false`. |
| `options.unknown_events` | no | `ignore`, `warn` or `error`. Default: `error`. |
| `expect.outcome` | yes | `pass`, `violation` (a `SpecViolation`) or `setup_error` (a `SetupError`). |
| `expect.spec` | violation | The violated spec. |
| `expect.key` | no | The violating key. |
| `expect.message` | violation: yes; setup_error: no | Substring of the error message. |
| `expect.liveness` | no | `true` for a hot-state (`check_hot`) violation. Default: `false`. |
| `expect.kind` | setup_error | `validation` (strict), `unknown_event`, `parser`, or another kind from the spec. |
| `expect.coverage` | no | Per spec, any of `states` / `handlers` `covered` / `total` counts. The check enables coverage when present. |

Unknown keys are errors, so a misspelled field fails the CLI runner instead of being
ignored.

## Adding a case

Keep cases small: one behaviour each, a few events, ISO-8601 timestamps with
nanoseconds. Run `cargo test -p p-rs --test cli_conformance` to check it against the
core. A case for a core feature that is not implemented yet carries a `requires` entry
until it lands; add the feature name to `KNOWN_FEATURES` in the runner.
