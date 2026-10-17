# Releasing p-rs

Releases are cut by the manually triggered **Release** workflow
(`.github/workflows/release.yml`). Nothing publishes on a tag push.

## Cut a release

Actions → **Release** → Run workflow, on `main`, and pick a level:

| level   | effect                                        |
| ------- | --------------------------------------------- |
| patch   | 0.0.1 → 0.0.2                                  |
| minor   | 0.0.1 → 0.1.0                                  |
| major   | 0.0.1 → 1.0.0                                  |
| release | publish the version already in `Cargo.toml`    |

Or from a terminal: `gh workflow run release.yml --ref main -f level=patch`.

The workflow:

1. **Preflight**: fmt, clippy, tests and `cargo publish --dry-run` on `main`.
2. **Bump and tag**: bumps `crates/p-rs/Cargo.toml` and `Cargo.lock`, commits
   `Release p-rs vX.Y.Z` to `main`, tags `vX.Y.Z` and pushes both atomically.
   If `main` moved since the run started, the push fails and nothing is published.
3. **Publish**: publishes `p-rs` to crates.io. A version already on crates.io is
   skipped, so a run that failed after tagging can be finished with `level=release`.
4. **GitHub release**: creates the release for the tag with notes generated from
   the merged PRs.

Only the `p-rs` crate is published. The `pnext-*` crates are not.

## Requirements

- The `github-actions[bot]` must be able to push to `main`. If `main` gets branch
  protection, allow GitHub Actions to bypass it (or the bump step will fail).
- crates.io trusted publishing for `p-rs` trusts:
  - Repository: `crowecawcaw/p-rs`
  - Workflow: `release.yml`
  - Environment: `release`

  The `release` environment must allow the `main` branch. No crates.io token is
  stored in GitHub; the auth action gets a short-lived token and revokes it after.
