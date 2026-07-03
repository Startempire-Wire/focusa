#!/usr/bin/env bash
# spec_focusa_112_shell_ps1_slim_static_test.sh
#
# Static guard for focusa-112-shell-slim + focusa-112-ps1-slim:
# the bash and PowerShell installers must shrink to thin bootstrappers
# that download `focusa` and `exec` `focusa install --target=auto`.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SH="$ROOT_DIR/scripts/install-focusa.sh"
PS1="$ROOT_DIR/scripts/install-focusa.ps1"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# ---- Shell installer ----
[ -f "$SH" ] || fail "install-focusa.sh missing"
LINES_SH=$(wc -l < "$SH")
if [ "$LINES_SH" -gt 100 ]; then
  fail "install-focusa.sh: $LINES_SH lines (must be <= 100 per §15A.4)"
fi
pass "install-focusa.sh: $LINES_SH lines (≤ 100)"

# Bash syntax (no need to run)
bash -n "$SH" || fail "install-focusa.sh: bash -n syntax check failed"
pass "install-focusa.sh: bash syntax OK"

# Must exec `focusa install` at the end (no logic beyond bootstrap).
# The exec line may pass ARGS[@] which contains 'install --target=...'
# so we check for the exec line + the install arg in ARGS.
grep -qE '^exec .*focusa.*"' "$SH" \
  || fail "install-focusa.sh: must end with 'exec \$BIN_DIR/focusa ...'"
grep -qE 'install.*--target=' "$SH" \
  || fail "install-focusa.sh: ARGS must include 'install --target=...'"
pass "install-focusa.sh: ends with exec focusa install"

# No systemd/launchd heredoc (logic moved to Rust)
if grep -q '\[Unit\]\|ExecStart\|launchctl bootout\|systemctl --user enable' "$SH"; then
  fail "install-focusa.sh: must not contain systemd/launchd rendering (delegated to Rust service::module)"
fi
pass "install-focusa.sh: no systemd/launchd rendering (delegated to Rust service module)"

# No license validate logic (delegated to Rust registry_validate)
if grep -q 'wpuiai-ai-cloud.*license.*validate\|post_license_validate' "$SH"; then
  fail "install-focusa.sh: must not call WP REST license validate (delegated to Rust)"
fi
pass "install-focusa.sh: no license validate logic (delegated to Rust registry_validate)"

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
if [ "$LINES_PS" -gt 100 ]; then
  fail "install-focusa.ps1: $LINES_PS lines (must be <= 100 per §15A.4)"
fi
pass "install-focusa.ps1: $LINES_PS lines (≤ 100)"

# Must end with focusa.exe exec
grep -qE '&\s*\(Join-Path.*focusa\.exe' "$PS1" \
  || fail "install-focusa.ps1: must end with exec focusa.exe"
pass "install-focusa.ps1: ends with exec focusa.exe"

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

echo "✓ All focusa-112-shell-slim + ps1-slim static checks passed"