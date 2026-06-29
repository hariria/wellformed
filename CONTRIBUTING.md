# Contributing

Thanks for contributing to wellformed.

## Before You Start

- Open an issue (bug/feature) before large changes.
- Keep pull requests scoped and reviewable.
- Add tests for behavioral changes.
- Update docs for public-facing API changes.

## Local Setup

Install Rust, `nvm`, pnpm, and `cargo-audit` first:

```bash
rustup toolchain install 1.93.0
cargo install cargo-audit --locked --version 0.22.1
nvm install
corepack enable pnpm
```

```bash
bash scripts/release-preflight.sh
```

This runs the Rust and TypeScript release checks, including Rust 1.93.0 MSRV,
linting, rustdoc, dependency audits, tests, package checks, docs build, packed
npm package smoke tests, and the Node 18 minimum-runtime smoke test.

If you change address parsing predicates, also install native libpostal
headers/libraries plus parser data and run:

```bash
bash scripts/release-preflight.sh --address
```

The extra environment variables are needed for libpostal installs whose headers
are visible through `pkg-config` but not through Clang's default include path,
such as Homebrew on Apple Silicon.

## Commit Style

- Use focused commits by concern (runtime, docs, benchmarks, etc.).
- Prefer conventional-style subjects, e.g.:
  - `feat(rust): ...`
  - `feat(ts): ...`
  - `docs(...): ...`
  - `perf(...): ...`

## Pull Requests

- Include:
  - what changed
  - why it changed
  - risks and compatibility notes
  - validation steps run locally

## Release Notes

- TypeScript packages:
  - add a Changeset in `typescript/.changeset/` for user-facing changes
- Rust crates:
  - update `CHANGELOG.md` for release-worthy changes

## Dependency Updates

Dependabot opens grouped weekly PRs for Rust, TypeScript, and GitHub Actions
dependencies. Treat dependency PRs like code changes: review release notes for
runtime-impacting packages and run the same local checks listed above before
merging.

## Conduct

By participating, you agree to the `CODE_OF_CONDUCT.md`.
