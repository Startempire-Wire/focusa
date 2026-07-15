#!/usr/bin/env bash
# Runtime regression for focusa onboard JSON/quiet/noninteractive behavior.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

if cargo --version >/tmp/onboard-cargo-version.log 2>&1; then
  CARGO_CMD=(cargo)
elif RUSTUP_TOOLCHAIN=nightly cargo --version >/tmp/onboard-cargo-version.log 2>&1; then
  CARGO_CMD=(env RUSTUP_TOOLCHAIN=nightly cargo)
else
  echo "SKIP: cargo unavailable or no active toolchain; cannot execute onboard runtime checks" >&2
  cat /tmp/onboard-cargo-version.log 1>&2
  exit 0
fi

run_focusa() {
  (cd "$ROOT_DIR" && "${CARGO_CMD[@]}" run -q -p focusa-cli -- "$@")
}

# --json must print only one JSON document and no banner/prompt text.
json_output="$(run_focusa --json onboard --scope host --project-root "$ROOT_DIR" 2>/tmp/onboard-json-err.log)"
cat /tmp/onboard-json-err.log 1>&2
if ! jq -e '.scope == "host" and .project_root == ""' <<<"$json_output" >/dev/null; then
  fail "json output should be valid JSON with host scope and skipped project_root"
fi
json_docs=$(jq -c . <<<"$json_output" | wc -l)
if [[ "$json_docs" -ne 1 ]]; then
  fail "--json should emit exactly one JSON document"
fi
if grep -q "FOCUSA OPERATOR PREVIEW ONBOARDING\|Choose \[1-" <<<"$json_output"; then
  fail "--json output contains banner or picker prompt"
fi
pass "--json emits single clean JSON document"

# --quiet should suppress human-mode banner/pickers and summary output.
quiet_output="$(run_focusa --quiet onboard --scope host --project-root "$ROOT_DIR" 2>/tmp/onboard-quiet-err.log)"
cat /tmp/onboard-quiet-err.log 1>&2
if [[ -n "$quiet_output" ]]; then
  fail "--quiet output should be silent in human mode"
fi
pass "--quiet is quiet"

# Non-interactive stdin/stdout should skip scope picker regardless of prompt intent.
noninteractive_output="$(printf '' | run_focusa onboard --scope project --project-root "$ROOT_DIR" 2>/tmp/onboard-noninteractive-err.log)"
cat /tmp/onboard-noninteractive-err.log 1>&2
if grep -q "Choose \[1-" <<<"$noninteractive_output"; then
  fail "scope picker ran in non-interactive session"
fi
pass "TTY picker does not run noninteractively"

# Scope selection remains explicit in command output.
non_json_json_output="$(run_focusa --json onboard --scope project --project-root "$ROOT_DIR" 2>/tmp/onboard-project-err.log)"
cat /tmp/onboard-project-err.log 1>&2
project_root=$(jq -r '.project_root' <<<"$non_json_json_output")
if [[ "$project_root" != "$ROOT_DIR" ]]; then
  fail "explicit --project-root should be preserved in output"
fi
pass "scope selection remains explicit and safe"

echo "✓ Onboarding JSON/quiet/noninteractive runtime regression test passed"