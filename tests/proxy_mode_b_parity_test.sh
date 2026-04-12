#!/bin/bash
# SPEC 51: Mode B proxy parity test
# Enforces operator-first minimal-slice behavior in both proxy adapters.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OPENAI_RS="${REPO_ROOT}/crates/focusa-core/src/adapters/openai.rs"
ANTHROPIC_RS="${REPO_ROOT}/crates/focusa-core/src/adapters/anthropic.rs"
CARGO_BIN="${CARGO_BIN:-/root/.cargo/bin/cargo}"
CARGO_HOME="${CARGO_HOME:-/tmp/focusa-cargo-home}"
RUSTUP_HOME="${RUSTUP_HOME:-/root/.rustup}"
TARGET_DIR="${FOCUSA_CARGO_TARGET_DIR:-/tmp/focusa-target}"

FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC 51: Mode B proxy parity test ==="

if rg -n "build_operator_first_slice\(" "$OPENAI_RS" "$ANTHROPIC_RS" >/dev/null 2>&1; then
  log_pass "Both proxy adapters use shared operator-first minimal-slice builder"
else
  log_fail "Proxy adapters do not share operator-first minimal-slice builder"
fi

if rg -n "assemble_from\(input\)" "$OPENAI_RS" "$ANTHROPIC_RS" >/dev/null 2>&1; then
  log_fail "Legacy full-context assemble_from injection remains in proxy adapters"
else
  log_pass "Legacy full-context assemble_from injection removed from proxy adapters"
fi

if CARGO_HOME="$CARGO_HOME" RUSTUP_HOME="$RUSTUP_HOME" "$CARGO_BIN" test \
  --manifest-path "${REPO_ROOT}/Cargo.toml" -p focusa-core --locked --target-dir "$TARGET_DIR" \
  focus_relevant_request_gets_minimal_slice_not_full_focus_dump >/dev/null 2>&1; then
  log_pass "OpenAI proxy minimal-slice unit test passes"
else
  log_fail "OpenAI proxy minimal-slice unit test failed"
fi

if CARGO_HOME="$CARGO_HOME" RUSTUP_HOME="$RUSTUP_HOME" "$CARGO_BIN" test \
  --manifest-path "${REPO_ROOT}/Cargo.toml" -p focusa-core --locked --target-dir "$TARGET_DIR" \
  anthropic_process_request_injects_minimal_slice >/dev/null 2>&1; then
  log_pass "Anthropic proxy minimal-slice unit test passes"
else
  log_fail "Anthropic proxy minimal-slice unit test failed"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]
