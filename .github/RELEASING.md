# Publishing p-rs

The `Publish p-rs` workflow uses crates.io trusted publishing. No crates.io API
token needs to be stored in GitHub secrets.

The crates.io configuration for `p-rs` trusts:

- Repository: `crowecawcaw/p-rs`
- Workflow: `release.yml`
- GitHub environment: `release`

The environment allows the `main` branch and tags matching `v*`.

## Release a version

1. Update the version in `crates/p-rs/Cargo.toml` and refresh `Cargo.lock`.
2. Commit those changes to `main` and push them.
3. Create and push a matching version tag, for example:

   ```sh
   git tag v0.0.2
   git push origin v0.0.2
   ```

The workflow checks that the tag matches the package version and points to a
commit in `main`, verifies the package, obtains a short-lived crates.io token,
and publishes only `p-rs`. The authentication action revokes its token afterward.

## Check the configuration without publishing

Run `Publish p-rs` manually from the Actions tab on `main`, or use:

```sh
gh workflow run release.yml --ref main
```

Manual runs verify the package and the OIDC token exchange without uploading a
new version. Version `0.0.1` was already published during initial setup; do not
push its tag as a new release.
