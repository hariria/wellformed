#!/usr/bin/env bash
set -euo pipefail

run_rust=0
run_typescript=0
run_address=0
run_external=0

usage() {
  cat <<'EOF'
Usage: scripts/release-preflight.sh [--rust] [--typescript] [--address] [--external] [--all]

Default with no flags: --rust --typescript

Options:
  --rust        Run Rust format, MSRV, clippy, rustdoc, audit, tests, and package checks
  --typescript  Run TypeScript install, audit, lint, typecheck, build, tests, and package smoke
  --address     Check the optional Rust address feature using pkg-config libpostal paths
  --external    Verify public GitHub/docs/security endpoints
  --all         Run all checks, including address and external endpoint verification
  --help        Show this help
EOF
}

if [[ $# -eq 0 ]]; then
  run_rust=1
  run_typescript=1
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rust)
      run_rust=1
      ;;
    --typescript)
      run_typescript=1
      ;;
    --address)
      run_address=1
      ;;
    --external)
      run_external=1
      ;;
    --all)
      run_rust=1
      run_typescript=1
      run_address=1
      run_external=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_command() {
  local command="$1"
  local install_hint="${2:-}"

  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: required command not found: $command" >&2
    if [[ -n "$install_hint" ]]; then
      echo "$install_hint" >&2
    fi
    exit 1
  fi
}

run() {
  echo "+ $*"
  "$@"
}

run_rust_checks() {
  require_command cargo
  require_command rustup
  require_command cargo-audit "install with: cargo install cargo-audit --locked --version 0.22.1"

  run rustup toolchain install 1.93.0
  run cargo +1.93.0 check --workspace --all-targets --locked
  run cargo fmt --check
  run cargo clippy --workspace --all-targets -- -D warnings
  echo "+ RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps"
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
  run cargo audit
  run cargo test --workspace
  run cargo package -p wellformed-validate --allow-dirty
  echo "+ cargo package -p wellformed --allow-dirty --no-verify --list >/dev/null"
  cargo package -p wellformed --allow-dirty --no-verify --list >/dev/null
  echo "+ cargo package -p wellformed-macros --allow-dirty --no-verify --list >/dev/null"
  cargo package -p wellformed-macros --allow-dirty --no-verify --list >/dev/null
}

run_typescript_checks() {
  local nvm_script="${NVM_DIR:-$HOME/.nvm}/nvm.sh"
  if [[ ! -s "$nvm_script" ]]; then
    echo "error: nvm is required for TypeScript release checks" >&2
    echo "set NVM_DIR or install nvm, then rerun this script" >&2
    exit 1
  fi

  echo "+ nvm use && pnpm TypeScript release checks"
  (
    # shellcheck disable=SC1090
    source "$nvm_script"
    cd "$repo_root"
    nvm use
    require_command pnpm "install with: corepack enable pnpm"
    run pnpm --dir typescript install --frozen-lockfile
    run pnpm --dir typescript audit --audit-level moderate
    run pnpm --dir typescript lint
    run pnpm --dir typescript types:check
    run pnpm --dir typescript build
    run pnpm --dir typescript --filter wellformed-ts test
    run pnpm --dir typescript test:package

    echo "+ nvm use 18 && (cd typescript/packages/wellformed && npm run test:package)"
    nvm use 18
    cd "$repo_root/typescript/packages/wellformed"
    npm run test:package
  )
}

run_address_check() {
  require_command cargo
  require_command pkg-config "install pkg-config and native libpostal development files"

  echo "+ BINDGEN_EXTRA_CLANG_ARGS=\"\$(pkg-config --cflags libpostal)\" LIBRARY_PATH=\"\$(pkg-config --variable=libdir libpostal):\${LIBRARY_PATH:-}\" cargo check -p wellformed --features address"
  BINDGEN_EXTRA_CLANG_ARGS="$(pkg-config --cflags libpostal)" \
    LIBRARY_PATH="$(pkg-config --variable=libdir libpostal):${LIBRARY_PATH:-}" \
    cargo check -p wellformed --features address
}

cd "$repo_root"

if [[ "$run_rust" -eq 1 ]]; then
  run_rust_checks
fi

if [[ "$run_typescript" -eq 1 ]]; then
  run_typescript_checks
fi

if [[ "$run_address" -eq 1 ]]; then
  run_address_check
fi

if [[ "$run_external" -eq 1 ]]; then
  run bash scripts/verify-public-release.sh
fi

echo "release preflight completed"
