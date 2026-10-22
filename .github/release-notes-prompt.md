# Release-note instructions

You are writing release notes for **snappea** (repo `snappea`), a Rust implementation
of the P modeling language: the `snappea` command-line tool (`snappea check` explores a
model, `snappea observe` checks recorded events against P specs), `snappea-compat` (a
drop-in for upstream P's `p`), the `snappea` crate (`snappea::emit`, `snappea::test`),
helpers for Python, Go and Java, and a browser playground.

You get a context document listing every change that landed on `main` in this
release: merged PRs (title, description and their commits) and commits pushed
directly. Decide which changes a **user** would notice and describe them.

## Read the PR description, not just the title

A title often names the vehicle ("UX review fixes", "close the gaps") while
the user-visible change is described in the body or in the commit list. Read
each one through before deciding.

## Include

- New or changed `snappea` / `snappea-compat` commands, options, output, trace formats, exit codes
- P language support: syntax, type-checking rules, diagnostics
- Model-checker behavior: scheduling strategies, bug finding, liveness,
  deadlock detection, replay
- Fixes where behavior now matches upstream P where it previously did not
- New or changed public API in the `snappea` crate or the Python, Go and Java helpers
- Playground features and fixes that users can see
- Breaking changes and deprecations
- Performance improvements a user would notice

## Exclude

- CI, GitHub Actions, release tooling, Dependabot, lockfile and dependency bumps
- Formatting, lint and clippy cleanups, internal refactors with no behavior change
- Tests, test fixtures, corpus/comparison scripts
- Documentation-only changes
- Version bump commits ("Release Snap Pea vX.Y.Z")

A PR that mixes test work with a behavior fix: leave out the tests, include the fix.

## Style

- Write from the user's point of view: what they can now do, or what was wrong
  and now works.
- For bug fixes, name the symptom: what happened before and what happens now.
- For breaking changes, say what users must change.
- Say where a change applies when it's specific (`snappea check`, the Go helper, the
  playground, a scheduling strategy).
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
