# Release-note instructions

You are writing release notes for **p-rs** (repo `p-rs`), a pure Rust
implementation of the P modeling language toolchain: parser, type checker and
model checker, the `p` command-line tool, a WebAssembly build, and a browser
playground. Users write P models and run `p compile` / `p check`, embed the
crates as a library, or use the playground.

You get a context document listing every change that landed on `main` in this
release: merged PRs (title, description and their commits) and commits pushed
directly. Decide which changes a **user** would notice and describe them.

## Read the PR description, not just the title

A title often names the vehicle ("UX review fixes", "close the gaps") while
the user-visible change is described in the body or in the commit list. Read
each one through before deciding.

## Include

- New or changed `p` CLI commands, options, output, trace formats, exit codes
- P language support: syntax, type-checking rules, diagnostics
- Model-checker behavior: scheduling strategies, bug finding, liveness,
  deadlock detection, replay
- Fixes where behavior now matches upstream P where it previously did not
- New or changed public library API in the `prs-*` crates or the WASM/JS wrapper
- Playground features and fixes that users can see
- Breaking changes and deprecations
- Performance improvements a user would notice

## Exclude

- CI, GitHub Actions, release tooling, Dependabot, lockfile and dependency bumps
- Formatting, lint and clippy cleanups, internal refactors with no behavior change
- Tests, test fixtures, corpus/comparison scripts
- Documentation-only changes
- Version bump commits ("Release p-rs vX.Y.Z")

A PR that mixes test work with a behavior fix: leave out the tests, include the fix.

## Style

- Write from the user's point of view: what they can now do, or what was wrong
  and now works.
- For bug fixes, name the symptom: what happened before and what happens now.
- For breaking changes, say what users must change.
- Say where a change applies when it's specific (`p check`, the playground, a
  scheduling strategy).
- Backticks for commands, options, identifiers and P syntax.
- One or two sentences per entry. Merge related PRs into one entry.
- No PR numbers or commit hashes in `description`; they go in `reference`.

## Output

Write a JSON object with one key, `entries`: an array of objects with exactly
these string fields:

| Field | Value |
| --- | --- |
| `category` | one of `breaking`, `deprecations`, `features`, `bug fixes` |
| `description` | user-facing text, no PR refs |
| `reference` | `(#12)`, `(#12, #13)`, or a commit hash as ``(`abc1234`)`` |

```json
{
  "entries": [
    {
      "category": "features",
      "description": "The playground's trace diagram links each step to its line in the log.",
      "reference": "(#6)"
    }
  ]
}
```

If nothing is user-visible, write `{"entries": []}`. Don't pad the notes with
internal work.

The file must contain only the JSON object: no markdown fence, no commentary.
