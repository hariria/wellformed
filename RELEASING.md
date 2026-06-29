# Releasing

This repository has separate release flows for TypeScript and Rust.

## Preflight

- Use the documented Node runtime (`nvm use`) before TypeScript release checks.
- Use Rust 1.82 or newer before Rust release checks.
- The local preflight script allows uncommitted package metadata changes so it
  can run during release preparation. Commit release changes before publishing.
- Confirm public package metadata points at the final repository URL before publishing:
  - npm: `repository`, `homepage`, and `bugs` in `typescript/packages/wellformed/package.json`
  - crates.io: `repository` in each Rust crate manifest
- Confirm `README.md`, package README files, and docs links point at the public docs and issue tracker.
- Enable GitHub private vulnerability reporting or add a monitored security contact to `SECURITY.md`.
- Prefer npm Trusted Publishing with provenance for public npm releases.
  Configure the package's npm Trusted Publisher to the final public GitHub
  repository and release workflow before relying on OIDC-based publishing.

## External URL Verification

Before publishing, confirm the public repository and docs endpoints resolve. For
the currently staged metadata, this should succeed:

```bash
bash scripts/release-preflight.sh --external
```

Set `WELLFORMED_REPO` or `WELLFORMED_DOCS_URL` to test a different public
repository or docs domain. If you use a monitored security contact instead of
GitHub private vulnerability reporting, add it to `SECURITY.md` and run with
`WELLFORMED_SECURITY_CONTACT_CONFIRMED=1`.

If the public repository or docs domain changes, update package manifests,
`SECURITY.md`, GitHub issue template contact links, package READMEs, and
AI-agent docs before publishing.

## TypeScript (Changesets)

TypeScript packages are managed from `typescript/`.

1. Add a changeset for user-facing changes:

```bash
cd typescript
pnpm changeset
```

2. Create release versions and changelogs:

```bash
pnpm run version
```

3. Build and verify:

```bash
bash scripts/release-preflight.sh --typescript
```

4. Publish (when approved):

```bash
pnpm run release
```

For automated npm releases, prefer npm Trusted Publishing over long-lived npm
tokens. The authorized GitHub Actions workflow needs `id-token: write`; npm
generates provenance for public packages published from trusted public
repositories. For manual releases, use an npm account with 2FA and verify the
published tarball metadata after publishing.

## Rust (Current Recommendation)

Rust crates are currently released manually.

1. Update versions in:
- `wellformed/Cargo.toml`
- `wellformed-macros/Cargo.toml`
- `wellformed-validate/Cargo.toml`
- workspace references as needed

2. Update `CHANGELOG.md`.

3. Verify:

```bash
bash scripts/release-preflight.sh --rust
```

If the release changes address parsing predicates or the `address` feature,
verify it on a machine with native libpostal installed:

```bash
bash scripts/release-preflight.sh --address
```

Those environment variables are required for libpostal installs whose headers
are visible through `pkg-config` but not through Clang's default include path,
such as Homebrew on Apple Silicon.

4. Dry-run and publish in dependency order. `wellformed` will not package from a clean crates.io index until `wellformed-validate` is published, and `wellformed-macros` will not package until `wellformed` is published.

```bash
cargo publish --dry-run -p wellformed-validate
cargo publish -p wellformed-validate

cargo publish --dry-run -p wellformed
cargo publish -p wellformed

cargo publish --dry-run -p wellformed-macros
cargo publish -p wellformed-macros
```

## Future Rust Automation (Optional)

If you want automation later, `release-plz` is the cleanest option for workspace crates and can generate release PRs + changelog updates.
