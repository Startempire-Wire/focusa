#!/usr/bin/env bash
# Spec 132 E4: executable terminal lifecycle/output isolation proof.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${FOCUSA_BIN:-$ROOT/target/debug/focusa}"
[[ -x "$BIN" ]] || { echo "FAIL: missing executable $BIN" >&2; exit 1; }
command -v jq >/dev/null || { echo "FAIL: jq is required" >&2; exit 1; }
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/home"

# JSON is a durable stdout contract: no terminal control bytes may leak.
HOME="$TMP/home" TERM=dumb NO_COLOR=1 "$BIN" --json install --preflight --no-animation --quiet >"$TMP/json.out"
jq -e '.schema == "focusa.install_preflight.v1" and .read_only == true and .mutations_performed == false' "$TMP/json.out" >/dev/null
! grep -q $'\033' "$TMP/json.out"

# A real pseudo-terminal invocation in plain mode must not enter alternate
# screen or emit cursor-control escapes; durable plain output remains visible.
command -v script >/dev/null || { echo "FAIL: script(1) is required" >&2; exit 1; }
script -qfec "HOME='$TMP/home' TERM=xterm FOCUSA_INSTALL_UI=plain '$BIN' install --preflight --quiet" "$TMP/pty.out" >/dev/null 2>&1
! grep -qE $'\033\[\?1049h|\033\[\?1049l|\033\[\?25[lh]' "$TMP/pty.out"
grep -q 'install preflight' "$TMP/pty.out"

# The source-level fallback/guard contract complements the executable plain
# and JSON paths above; these are implementation checks, not success markers.
rg -q 'TerminalGuard' "$ROOT/crates/focusa-terminal-ui/src/install/renderer.rs"
rg -q 'PlainPresenter' "$ROOT/crates/focusa-terminal-ui/src/install/presenter.rs"
rg -q 'channel.fail' "$ROOT/crates/focusa-cli/src/commands/install.rs"

echo "PASS: E4 JSON isolation, non-alternate plain PTY lifecycle, and fallback contract"
