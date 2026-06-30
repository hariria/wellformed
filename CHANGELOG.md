# Changelog

All notable changes to this repository are documented here.

The format is based on Keep a Changelog and this project follows Semantic Versioning.

## [Unreleased]

### Added

- Open-source readiness scaffolding for docs, governance, CI, security reporting, and release process docs.
- Production guidance for serialized schemas, runtime validation, AI-assisted integration, Rust runtime use, and package publishing.
- TypeScript package smoke coverage for packed npm tarballs, including ESM, CommonJS, subpath exports, and TypeScript consumer checks.
- CI and release checks for dependency audits, TypeScript package tarball smoke tests, and Rust leaf-crate packaging.
- Framework-neutral Rust form helpers and `form_schema!` codegen for generated field metadata, validation helpers, and serializable form state.

### Changed

- Rust `CodegenOptions::default()` now generates structs and validation only. Generated Axum/API code requires explicitly setting `generate_api: true`, and PDF render handlers require `generate_pdf_handlers: true` plus the rendering dependencies in the consuming application.

### Fixed

- TypeScript schema parsing now validates top-level `definitions` as a map, matching the documented IR and Rust serde representation.
- TypeScript runtime validation now resolves `ref` schemas through top-level `definitions` and reports missing references or cycles instead of accepting unvalidated pass-through values.
- Rust codegen now derives Rust field identifiers from schema keys instead of presentation labels, keeping label-only changes from becoming Rust API breaks.
- Generated Rust API request DTOs and OpenAPI field schemas now also derive field names from schema keys instead of presentation labels.

## [0.1.0] - 2026-02-07

### Added

- Initial workspace crates and packages:
  - Rust: `wellformed`, `wellformed-macros`, `wellformed-validate`
  - TypeScript: `wellformed` package and docs/playground app

### Notes

- TypeScript package-level changelog/release notes are managed via Changesets.
