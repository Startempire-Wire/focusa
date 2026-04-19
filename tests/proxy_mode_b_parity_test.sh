#!/bin/bash
# SPEC 51 runtime behavior contract: proxy adapters must preserve operator-first minimal-slice behavior.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"
if ! command -v "$CARGO_BIN" >/dev/null 2>&1; then
  if [ -x "/root/.cargo/bin/cargo" ]; then
    CARGO_BIN="/root/.cargo/bin/cargo"
  fi
fi
HOME_DIR="${HOME:-$(cd ~ && pwd)}"
CARGO_HOME="${CARGO_HOME:-${HOME_DIR}/.cargo}"
RUSTUP_HOME="${RUSTUP_HOME:-${HOME_DIR}/.rustup}"
TARGET_DIR="${FOCUSA_CARGO_TARGET_DIR:-/tmp/focusa-target}"

FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC 51: Mode B proxy parity runtime behavior test ==="

if CARGO_HOME="$CARGO_HOME" RUSTUP_HOME="$RUSTUP_HOME" "$CARGO_BIN" test \
  --manifest-path "${REPO_ROOT}/Cargo.toml" -p focusa-core --locked --target-dir "$TARGET_DIR" \
  focus_relevant_request_gets_minimal_slice_not_full_focus_dump >/dev/null 2>&1; then
  log_pass "OpenAI proxy preserves minimal-slice behavior"
else
  log_fail "OpenAI proxy minimal-slice behavior regression"
fi

if CARGO_HOME="$CARGO_HOME" RUSTUP_HOME="$RUSTUP_HOME" "$CARGO_BIN" test \
  --manifest-path "${REPO_ROOT}/Cargo.toml" -p focusa-core --locked --target-dir "$TARGET_DIR" \
  anthropic_process_request_injects_minimal_slice >/dev/null 2>&1; then
  log_pass "Anthropic proxy preserves minimal-slice behavior"
else
  log_fail "Anthropic proxy minimal-slice behavior regression"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]
