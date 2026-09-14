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
[ "$LINES_SH" -ge 180 ] || fail "install-focusa.sh lost required verified delegation behavior ($LINES_SH lines)"
[ "$LINES_SH" -le 350 ] || fail "install-focusa.sh regained bootstrapper-owned product authority ($LINES_SH lines)"
pass "install-focusa.sh: $LINES_SH lines (bounded verified delegation surface)"

# Bash syntax (no need to run)
bash -n "$SH" || fail "install-focusa.sh: bash -n syntax check failed"
pass "install-focusa.sh: bash syntax OK"

# The Bash path delegates to the verified temporary Rust binary and preserves
# its failure status without issuing local entitlement state.
grep -qF 'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' "$SH" \
  || fail "install-focusa.sh: rollback-aware Rust handoff missing"
grep -qF 'restore_bootstrap_stash' "$SH" \
  || fail "install-focusa.sh: orchestrator recovery hint missing"
grep -qF 'ARGS=(install --target="$RUST_TARGET"' "$SH" \
  || fail "install-focusa.sh: ARGS must include Rust install target"
grep -qF 'trap cleanup EXIT INT TERM' "$SH" \
  || fail "install-focusa.sh: bounded temporary cleanup trap missing"
pass "install-focusa.sh: verified Rust handoff retained"

# No systemd/launchd heredoc (logic moved to Rust)
if grep -q '\[Unit\]\|ExecStart\|launchctl bootout\|systemctl --user enable' "$SH"; then
  fail "install-focusa.sh: must not contain systemd/launchd rendering (delegated to Rust service::module)"
fi
pass "install-focusa.sh: no systemd/launchd rendering (delegated to Rust service module)"

# Entitlement authority is Rust-owned; the bootstrapper rejects raw credentials.
grep -qF 'raw credentials and legacy registry overrides are forbidden' "$SH" \
  || fail "install-focusa.sh: raw credential rejection missing"
if grep -qE 'write_license_json|LICENSE_KEY=|CUSTOMER_EMAIL=' "$SH"; then
  fail "install-focusa.sh: local entitlement authority remains"
fi
pass "install-focusa.sh: signed authority delegation retained"

# Detects platform and architecture
grep -q 'OS=.*uname -s\|uname -s' "$SH" \
  || fail "install-focusa.sh: must detect OS via uname"
grep -q 'ARCH=.*uname -m\|uname -m' "$SH" \
  || fail "install-focusa.sh: must detect arch via uname"
pass "install-focusa.sh: detects platform + arch (uname)"

grep -Fq 'Darwin:arm64|Darwin:aarch64)' "$SH" \
  || fail "install-focusa.sh: native macOS arm64 must pass the architecture allowlist"
grep -Fq 'TRIPLE="aarch64-apple-darwin"' "$SH" \
  || fail "install-focusa.sh: macOS arm64 must select the Apple Silicon release triple"
pass "install-focusa.sh: accepts native Apple Silicon arm64"

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
[ "$LINES_PS" -ge 140 ] || fail "install-focusa.ps1 lost required verified delegation behavior ($LINES_PS lines)"
[ "$LINES_PS" -le 250 ] || fail "install-focusa.ps1 regained bootstrapper-owned product authority ($LINES_PS lines)"
pass "install-focusa.ps1: $LINES_PS lines (bounded verified delegation surface)"

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