#!/usr/bin/env bash
# spec_focusa_112_shell_ps1_slim_static_test.sh
#
# Static guard for focusa-112-shell-slim + focusa-112-ps1-slim:
# the Bash and PowerShell installers must preserve their protected,
# rollback-aware bootstrap behavior while delegating installation to Rust.
# These surfaces are intentionally not line-count constrained: the Bash
# bootstrapper owns release/license/download transaction safety and the
# PowerShell bootstrapper owns the equivalent Windows handoff.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SH="$ROOT_DIR/scripts/install-focusa.sh"
PS1="$ROOT_DIR/scripts/install-focusa.ps1"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# ---- Shell installer ----
[ -f "$SH" ] || fail "install-focusa.sh missing"
LINES_SH=$(wc -l < "$SH")
[ "$LINES_SH" -ge 700 ] || fail "install-focusa.sh protected rollback-aware surface unexpectedly shrank ($LINES_SH lines)"
pass "install-focusa.sh: $LINES_SH lines (protected surface retained)"

# Bash syntax (no need to run)
bash -n "$SH" || fail "install-focusa.sh: bash -n syntax check failed"
pass "install-focusa.sh: bash syntax OK"

# The protected Bash path runs the Rust handoff (rather than exec) so its
# EXIT trap can remove clean-state partial mutations after failure.
grep -qF 'if "$BIN_DIR/focusa" "${ARGS[@]}"; then' "$SH" \
  || fail "install-focusa.sh: rollback-aware Rust handoff missing"
grep -qF 'Rust install orchestrator failed (exit ${status})' "$SH" \
  || fail "install-focusa.sh: orchestrator failure must reach rollback trap"
grep -qE 'install.*--target=' "$SH" \
  || fail "install-focusa.sh: ARGS must include Rust install target"
grep -qF 'trap cleanup_bootstrap_failure EXIT' "$SH" \
  || fail "install-focusa.sh: cleanup trap missing"
pass "install-focusa.sh: protected rollback-aware Rust handoff retained"

# No systemd/launchd heredoc (logic moved to Rust)
if grep -q '\[Unit\]\|ExecStart\|launchctl bootout\|systemctl --user enable' "$SH"; then
  fail "install-focusa.sh: must not contain systemd/launchd rendering (delegated to Rust service::module)"
fi
pass "install-focusa.sh: no systemd/launchd rendering (delegated to Rust service module)"

# The protected bootstrapper retains its license preflight contract and must
# fail closed for commercial installs without acceptance or a key.
grep -qF 'Commercial install requires --accept-license or a --license-key.' "$SH" \
  || fail "install-focusa.sh: commercial license gate missing"
grep -qF 'LICENSE_VALIDATE_PATH' "$SH" \
  || fail "install-focusa.sh: license registry validation contract missing"
pass "install-focusa.sh: license preflight contract retained"

# Detects platform and architecture
grep -q 'HOST_OS\|uname -s' "$SH" \
  || fail "install-focusa.sh: must detect OS via uname"
grep -q 'HOST_ARCH\|uname -m' "$SH" \
  || fail "install-focusa.sh: must detect arch via uname"
pass "install-focusa.sh: detects platform + arch (uname)"

# Verifies SHA256SUMS
grep -q 'SHA256SUMS\|sha256sum' "$SH" \
  || fail "install-focusa.sh: must verify SHA256SUMS"
pass "install-focusa.sh: verifies SHA256SUMS"

# Cites Spec 112 §15A.1 / §15A.4
grep -q 'Spec 112' "$SH" \
  || fail "install-focusa.sh: must cite Spec 112"
pass "install-focusa.sh: cites Spec 112 §15A"

# ---- PowerShell installer ----
[ -f "$PS1" ] || fail "install-focusa.ps1 missing"
LINES_PS=$(wc -l < "$PS1")
[ "$LINES_PS" -ge 180 ] || fail "install-focusa.ps1 protected Windows surface unexpectedly shrank ($LINES_PS lines)"
pass "install-focusa.ps1: $LINES_PS lines (protected surface retained)"

# PowerShell delegates the selected release to the Rust orchestrator and
# propagates its exit status; it does not own service installation.
grep -qF '& $Focusa @Args' "$PS1" \
  || fail "install-focusa.ps1: Rust orchestrator handoff missing"
grep -qF 'focusa install failed with exit code' "$PS1" \
  || fail "install-focusa.ps1: orchestrator failure must be surfaced"
pass "install-focusa.ps1: protected Rust handoff retained"

# No sc.exe registration logic (delegated to Rust service module)
if grep -qi 'sc\.exe create\|sc\.exe delete' "$PS1"; then
  fail "install-focusa.ps1: must not call sc.exe (delegated to Rust service::run_scm Phase 2.0)"
fi
pass "install-focusa.ps1: no sc.exe service registration (delegated to Rust)"

# Detects platform
grep -qi 'OSArchitecture\|Is64BitOperatingSystem\|windows-x64\|windows-arm64' "$PS1" \
  || fail "install-focusa.ps1: must detect platform+arch"
pass "install-focusa.ps1: detects platform + arch"

# Downloads focusa
grep -qi 'Invoke-WebRequest\|browser_download_url' "$PS1" \
  || fail "install-focusa.ps1: must download focusa"
pass "install-focusa.ps1: downloads focusa"

# Cites Spec 112
grep -q 'Spec 112' "$PS1" \
  || fail "install-focusa.ps1: must cite Spec 112"
pass "install-focusa.ps1: cites Spec 112 §15A"

# Executable Bash behavior: argument validation must fail before any state is
# created. This catches a broken bootstrapper, not merely source markers.
FIXTURE=$(mktemp -d)
trap 'rm -rf "$FIXTURE"' EXIT
set +e
HOME="$FIXTURE/home" "$SH" --not-a-real-option >"$FIXTURE/out" 2>&1
RC=$?
set -e
[ "$RC" -eq 64 ] || fail "install-focusa.sh: unknown option exited $RC, expected 64"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "install-focusa.sh: invalid option mutated install state"
pass "install-focusa.sh: invalid option fails closed without mutation"

echo "✓ Protected Bash + PowerShell bootstrapper contract checks passed"