# Checking logs with `prs observe`

`prs observe` checks events recorded by a running system against the spec monitors of
a P project. It replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve)
and follows its options, console output, `PObserve-<date>/` output and exit codes
(0 ok, 1 command line, 2 spec violation, 3 internal error, 4 parser errors only).

The difference is where the spec comes from: there are no jars. The P sources are
type-checked in memory, from the `.pproj` in the current folder, `-pp/--pproj <file>`
or `-pf/--pfiles <files>`.

```sh
prs observe --spec BankBalanceCorrect -l logs/                 # JSON lines
prs observe --spec BankBalanceCorrect -l trace.json -i JSON    # a P JSON trace
prs observe -pf PSpec/Bank.p --spec BankBalanceCorrect -l app.log --keys 7 9
```

## Log formats

Parsers turn log entries into events. Two are built in:

* `JsonLines` (default for `-i TEXT`): one JSON object per line,
  `{"key": "7", "timestamp": 1700000000000, "event": "eWithDraw", "payload": {...}}`.
  The timestamp is epoch milliseconds or ISO-8601. Payloads are matched against the
  event's payload type. `-pc '{"key": "ctx.account", "event": "type"}'` remaps the
  fields (dotted paths).
* `PTrace` (default for `-i JSON`): the `.trace.json` written by `prs check`, checking
  its sends and announcements.

## Custom parsers

Other log formats need a parser written in Rust: implement
`prs_checker::observe::Parser` and run `prs_compat::observe::run_with(args, host, parsers)`.
`crates/prs-cli/examples/observe_regression.rs` is a worked example; it ports
upstream's regression parsers.
