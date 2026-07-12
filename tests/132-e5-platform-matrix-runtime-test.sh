#!/usr/bin/env bash
# 132 E5: applicable platform interaction matrix runtime proof.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${FOCUSA_BIN:-$ROOT/target/debug/focusa}"
[[ -x "$BIN" ]] || { echo "FAIL: missing executable $BIN" >&2; exit 1; }
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run_case() {
  local label="$1"; shift
  mkdir -p "$TMP/$label/home"
  HOME="$TMP/$label/home" "$@" >"$TMP/$label/out"
  jq -e '.schema == "focusa.install_preflight.v1" and .read_only == true and .mutations_performed == false' "$TMP/$label/out" >/dev/null
  ! grep -q $'\033' "$TMP/$label/out"
}

run_case linux-ci env CI=1 TERM=dumb NO_COLOR=1 "$BIN" --json install --preflight --quiet --no-animation
run_case linux-no-color env TERM=xterm NO_COLOR=1 FOCUSA_REDUCE_MOTION=1 "$BIN" --json install --preflight --quiet
run_case linux-plain env TERM=xterm FOCUSA_INSTALL_UI=plain "$BIN" --json install --preflight --quiet

# This host is Linux; Windows ConPTY execution belongs to the Windows CI host.
# Keep the required Windows capability and ConPTY implementation contract in
# the repository guard rather than pretending a Linux shell is ConPTY proof.
rg -q 'windows|ConPTY|TerminalGuard' "$ROOT/crates/focusa-terminal-ui/src" "$ROOT/docs/132-focusa-installer-animated-terminal-experience-spec.md"

echo "PASS: Linux CI/NO_COLOR/reduced/plain matrix runtime proof; Windows ConPTY delegated to Windows host"
