#!/usr/bin/env bash
# Regression for host-compatible focusa binary selection and explicit override ordering.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"

if [[ -z "${FOCUSA_BIN+x}" ]]; then
  echo "FAIL: FOCUSA_BIN must be set to a portable focusa fixture binary" >&2
  exit 1
fi
if [[ -z "$FOCUSA_BIN" ]]; then
  echo "FAIL: FOCUSA_BIN must not be empty for portable fixture" >&2
  exit 1
fi
if [[ ! -x "$FOCUSA_BIN" ]]; then
  echo "FAIL: expected an executable portable focusa fixture at FOCUSA_BIN=$FOCUSA_BIN" >&2
  exit 1
fi
PORTABLE_FIXTURE="$FOCUSA_BIN"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/workspace/target/debug" "$TMP/workspace/target/release"

# 1) executable-format-compatible but runtime-incompatible binary in the default debug slot.
cp /bin/ls "$TMP/workspace/target/debug/focusa"
chmod +x "$TMP/workspace/target/debug/focusa"

# 2) valid portable binary candidate in default release slot.
cp "$PORTABLE_FIXTURE" "$TMP/workspace/target/release/focusa"

# 3) valid explicit override fixture.
mkdir -p "$TMP/workspace/explicit"
cp "$PORTABLE_FIXTURE" "$TMP/workspace/explicit/focusa"

unset FOCUSA_BIN
DEFAULT_BIN="$(focusa_resolve_test_cli_binary "$TMP/workspace")"
[[ "$DEFAULT_BIN" == "$TMP/workspace/target/release/focusa" ]] || fail "runtime-incompatible default candidate was not skipped"
pass "runtime-incompatible executable format-compatible candidate is rejected by runtime check"
focusa_print_binary_evidence "$DEFAULT_BIN"

FOCUSA_BIN="$TMP/workspace/explicit/focusa"
EXPLICIT_BIN="$(focusa_resolve_test_cli_binary "$TMP/workspace")"
[[ "$EXPLICIT_BIN" == "$FOCUSA_BIN" ]] || fail "explicit FOCUSA_BIN override was not honored"
pass "explicit FOCUSA_BIN override was honored before default candidates"
focusa_print_binary_evidence "$EXPLICIT_BIN"

FOCUSA_BIN="/tmp/does-not-exist/focusa"
missing_log="$TMP/focusa-bin-selection-missing"
if focusa_resolve_test_cli_binary "$TMP/workspace" >"$missing_log" 2>&1; then
  fail "missing explicit FOCUSA_BIN should fail selection"
fi
if ! grep -q 'explicit FOCUSA_BIN' "$missing_log"; then
  fail "explicit missing candidate error was not surfaced"
fi
pass "missing explicit FOCUSA_BIN fails fast"

unset FOCUSA_BIN
missing_candidates_log="$TMP/focusa-bin-selection-no-candidates"
if focusa_resolve_test_cli_binary "$TMP/missing" >"$missing_candidates_log" 2>&1; then
  fail "missing candidates should fail selection"
fi
if ! grep -q 'no host-compatible focusa binary found' "$missing_candidates_log"; then
  fail "missing-candidates fixture did not surface expected error"
fi
pass "missing candidate fixture is rejected"

FOCUSA_BIN="$TMP/workspace/explicit/focusa"
VALID_BIN="$(focusa_resolve_test_cli_binary "$TMP/workspace")"
[[ "$VALID_BIN" == "$FOCUSA_BIN" ]] || fail "explicit valid portable candidate was not selected"
pass "valid portable binary candidate is selected"
focusa_print_binary_evidence "$VALID_BIN"

echo "PASS: portable binary-selection regression checks complete"
