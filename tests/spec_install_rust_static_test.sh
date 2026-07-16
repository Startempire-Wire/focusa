#!/usr/bin/env bash
# spec_install_rust_static_test.sh
#
# Spec 112 §15A.5 static guard: install logic lives in Rust (`focusa install`),
# shell/PowerShell scripts are thin bootstrappers only.
#
# Backward compatibility: curl|bash and irm|iex wrappers still download a
# `focusa` binary and execute `focusa install`; existing --target/--dry-run/
# license/eval flags remain accepted.
#
# Scope enforcement: install must keep explicit target allowlist and must not
# silently treat cross-arch/cross-os targets as safe.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_RS="$ROOT_DIR/crates/focusa-cli/src/commands/install.rs"
SH="$ROOT_DIR/scripts/install-focusa.sh"
PS1="$ROOT_DIR/scripts/install-focusa.ps1"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# Rust owns the real installer
grep -q 'Focusa install — single Rust orchestrator' "$INSTALL_RS" \
  || fail "install.rs missing single Rust orchestrator header"
grep -q 'pub struct InstallArgs' "$INSTALL_RS" \
  || fail "install.rs missing InstallArgs"
grep -q 'pub enum InstallTarget' "$INSTALL_RS" \
  || fail "install.rs missing InstallTarget allowlist enum"
pass "Rust install orchestrator exists with InstallArgs + InstallTarget"

# Backward-compatible flags retained
for flag in 'pub target: InstallTarget' 'pub dry_run: bool' 'pub license_key: Option<String>' 'pub eval: bool'; do
  grep -q "$flag" "$INSTALL_RS" \
    || fail "install.rs missing backward-compatible flag field: $flag"
done
pass "existing install flags retained (--target/--dry-run/--license-key/--eval)"

# Target allowlist is explicit and matches current public CLI surface.
# Future CI assets (musl/windows-arm64) may expand build matrix, but the
# install CLI must not accept arbitrary target strings.
for target in Auto Linux Darwin WindowsX64 WindowsArm64; do
  grep -q "$target" "$INSTALL_RS" \
    || fail "InstallTarget missing explicit variant: $target"
done
pass "InstallTarget allowlist is explicit (auto/linux/darwin/windows-x64/windows-arm64)"

# Cross-platform safety path must exist
for marker in 'fn resolve_target' 'std::env::consts::OS' 'std::env::consts::ARCH'; do
  grep -q "$marker" "$INSTALL_RS" \
    || fail "install.rs missing platform/target resolution marker: $marker"
done
pass "install target resolution markers present (no silent cross-arch install)"

# Checksum verification retained in Rust and scripts
grep -qi 'SHA256SUMS' "$INSTALL_RS" \
  || fail "install.rs missing SHA256SUMS verification reference"
grep -qi 'checksum mismatch' "$INSTALL_RS" \
  || fail "install.rs missing checksum mismatch failure path"
grep -qi 'checksum mismatch' "$SH" \
  || fail "install-focusa.sh missing checksum mismatch failure path"
grep -qi 'checksum mismatch' "$PS1" \
  || fail "install-focusa.ps1 missing checksum mismatch failure path"
pass "checksum verification failure paths retained in Rust + thin scripts"

# Thin bootstrapper contract: scripts download focusa, then delegate to Rust install.
# Bash invokes directly (not exec) so its EXIT trap can roll back partial clean installs.
for marker in 'ARGS=(install --target="$RUST_TARGET"' 'if "$BIN_DIR/focusa" "${ARGS[@]}"; then' \
  'Rust install orchestrator failed (exit ${status})'; do
  grep -qF "$marker" "$SH" \
    || fail "install-focusa.sh missing rollback-aware delegate marker: $marker"
done
grep -q '@("install", "--target=$ResolvedTarget"' "$PS1" \
  || fail "install-focusa.ps1 missing thin-delegate install args"
grep -q '& $Focusa @Args' "$PS1" \
  || fail "install-focusa.ps1 missing focusa install execution"
pass "shell/PowerShell installers delegate to focusa install"

python3 - "$SH" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
assignment = text.index('RELEASE_TAG="${SELECTED%%')
first_expansion = text.index('$RELEASE_TAG')
assert assignment < first_expansion, "RELEASE_TAG is expanded before initialization under set -u"
pattern = re.compile(
    r'if "\$BIN_DIR/focusa" "\$\{ARGS\[@\]\}"; then\s+'
    r'BOOTSTRAP_SUCCESS=1\s+exit 0\s+else\s+status=\$\?\s+'
    r'err "Rust install orchestrator failed \(exit \$\{status\}\)"\s+exit "\$status"',
    re.MULTILINE,
)
assert pattern.search(text), "orchestrator failure status is not preserved through an explicit else branch"
PY
pass "bootstrapper initializes release state and preserves nonzero orchestrator exits"

# Shell scripts must not embed service manager heredocs anymore (logic belongs in Rust service module)
if grep -qE 'cat >.*systemd|LaunchAgent|plist|systemctl --user enable' "$SH"; then
  fail "install-focusa.sh still appears to embed service install logic"
fi
if grep -qE 'New-Service|sc.exe create|LaunchAgent|plist' "$PS1"; then
  fail "install-focusa.ps1 still appears to embed service install logic"
fi
pass "thin scripts do not embed service manager install logic"

# Installer surface remains additive/compatible: new Rust command still exposes dry-run + channel
grep -q 'pub channel:' "$INSTALL_RS" \
  || fail "install.rs missing --channel support"
grep -q 'default_value = "auto"' "$INSTALL_RS" \
  || fail "install --target default must remain auto"
pass "install default target remains auto and channel support exists"

echo "✓ All Spec 112 install Rust static checks passed"