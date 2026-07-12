#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${FOCUSA_BIN:-$ROOT/target/debug/focusa}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
[[ -x "$BIN" ]] || { echo "FAIL: FOCUSA_BIN is not executable: $BIN" >&2; exit 1; }

run_invalid() {
  local name="$1" value="$2" expected="$3"
  local err="$TMP/${name}.err"
  set +e
  env "${name}=${value}" HOME="$TMP/home" "$BIN" --json install --preflight --quiet >"$TMP/${name}.out" 2>"$err"
  local rc=$?
  set -e
  [[ "$rc" -ne 0 ]] || { echo "FAIL: ${name}=${value} unexpectedly succeeded" >&2; exit 1; }
  cat "$TMP/${name}.out" "$err" | grep -Fq "$expected" || { echo "FAIL: ${name}=${value} missing actionable error" >&2; cat "$TMP/${name}.out" "$err" >&2; exit 1; }
  [[ ! -e "$TMP/home/.focusa" ]] || { echo "FAIL: invalid ${name} mutated install root" >&2; exit 1; }
  echo "PASS: invalid ${name} fails before mutation"
}

run_invalid FOCUSA_INSTALL_UI bogus "invalid FOCUSA_INSTALL_UI"
run_invalid FOCUSA_INSTALL_SEED not-a-u64 "FOCUSA_INSTALL_SEED must be an unsigned 64-bit integer"
run_invalid FOCUSA_REDUCE_MOTION yes "FOCUSA_REDUCE_MOTION must be 0 or 1"

for ui in auto full mono reduced plain; do
  FOCUSA_INSTALL_UI="$ui" HOME="$TMP/home-valid-$ui" "$BIN" --json install --preflight --quiet >/dev/null
  echo "PASS: valid FOCUSA_INSTALL_UI=${ui} accepted"
done
FOCUSA_INSTALL_SEED=18446744073709551615 HOME="$TMP/home-valid-seed" "$BIN" --json install --preflight --quiet >/dev/null
echo "PASS: valid u64 seed accepted"
for motion in 0 1; do
  FOCUSA_REDUCE_MOTION="$motion" HOME="$TMP/home-valid-motion-$motion" "$BIN" --json install --preflight --quiet >/dev/null
  echo "PASS: valid FOCUSA_REDUCE_MOTION=${motion} accepted"
done

echo "Spec 132 installer animation environment validation: PASS"
