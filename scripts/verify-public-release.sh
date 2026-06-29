#!/usr/bin/env bash
set -euo pipefail

repo="${WELLFORMED_REPO:-hariria/wellformed}"
docs_url="${WELLFORMED_DOCS_URL:-https://wellformed.net}"

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

check_url() {
  local url="$1"
  echo "checking ${url}"
  curl -fsSIL "$url" >/dev/null
}

require_command gh "install GitHub CLI and authenticate with: gh auth login"
require_command curl

echo "checking GitHub repository ${repo}"
if ! gh repo view "$repo" --json nameWithOwner,isPrivate,url >/dev/null; then
  echo "error: GitHub repository ${repo} is not reachable" >&2
  exit 1
fi

is_private="$(gh repo view "$repo" --json isPrivate --jq '.isPrivate')"
if [[ "$is_private" != "false" ]]; then
  echo "error: GitHub repository ${repo} must be public before publishing" >&2
  exit 1
fi

security_status="$(
  gh api "repos/${repo}" \
    --jq '.security_and_analysis.private_vulnerability_reporting.status // "unavailable"'
)"

if [[ "$security_status" != "enabled" ]]; then
  if [[ "${WELLFORMED_SECURITY_CONTACT_CONFIRMED:-}" == "1" ]]; then
    echo "private vulnerability reporting is ${security_status}; using confirmed security contact override"
  else
    echo "error: private vulnerability reporting is ${security_status}" >&2
    echo "enable GitHub private vulnerability reporting or set WELLFORMED_SECURITY_CONTACT_CONFIRMED=1 after adding a monitored contact to SECURITY.md" >&2
    exit 1
  fi
fi

check_url "$docs_url"
check_url "${docs_url%/}/docs"
check_url "${docs_url%/}/llms.txt"

echo "public release endpoints verified"
