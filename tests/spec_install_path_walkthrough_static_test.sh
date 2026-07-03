#!/usr/bin/env bash
# spec_install_path_walkthrough_static_test.sh
#
# Static guard for Spec 112 §15A.6 PATH automation + first-install walkthrough.
# Backward compatibility: existing --persist-path/--no-persist-path and shell
# family flags remain accepted. Scope/safety: PATH writes are idempotent and
# restricted to shell rc targets, not arbitrary files.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT_DIR/crates/focusa-cli/src/commands/install.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# Existing PATH flags retained
grep -q 'pub persist_path: bool' "$INSTALL" \
  || fail "missing --persist-path flag field"
grep -q 'pub no_persist_path: bool' "$INSTALL" \
  || fail "missing --no-persist-path flag field"
grep -q 'conflicts_with = "persist_path"' "$INSTALL" \
  || fail "--no-persist-path must conflict with --persist-path"
pass "PATH persistence flags retained and mutually exclusive"

# Shell rc detection supports bash/zsh/fish paths
grep -q 'pub fn detect_shell_rc_targets' "$INSTALL" \
  || fail "missing detect_shell_rc_targets"
for marker in '.bashrc' '.zshrc' 'config/fish/config.fish'; do
  grep -q "$marker" "$INSTALL" \
    || fail "shell rc detection missing marker: $marker"
done
pass "shell rc detection covers bash/zsh/fish"

# PATH write is idempotent: existing PATH/.local/bin marker returns without duplicate append
grep -q 'content.contains(".local/bin") && content.contains("PATH")' "$INSTALL" \
  || fail "persist_path_to_rc missing idempotency guard"
grep -q 'std::fs::write(rc,' "$INSTALL" \
  || fail "persist_path_to_rc missing explicit rc write"
pass "PATH persistence is idempotent and writes only selected rc file"

# Walkthrough envelope exists and has six-step card markers
grep -q 'pub struct FirstInstallWalkthrough' "$INSTALL" \
  || fail "missing FirstInstallWalkthrough envelope"
grep -q 'pub fn build_first_install_walkthrough' "$INSTALL" \
  || fail "missing build_first_install_walkthrough"
for marker in 'verify install' 'focusa start' 'doctor' 'pair' 'docs'; do
  grep -qi "$marker" "$INSTALL" \
    || fail "walkthrough missing marker: $marker"
done
pass "first-install walkthrough includes verify/start/doctor/pair/docs guidance"

# Runtime phase wires PATH then walkthrough into JSON envelope
grep -q 'detect_shell_rc_targets()' "$INSTALL" \
  || fail "execute_real_install missing detect_shell_rc_targets call"
grep -q 'persist_path_to_rc(&rc, &line)' "$INSTALL" \
  || fail "execute_real_install missing persist_path_to_rc call"
grep -q 'first_install_walkthrough' "$INSTALL" \
  || fail "install JSON envelope missing first_install_walkthrough"
pass "PATH automation and walkthrough are wired into install flow"

echo "✓ All install PATH/walkthrough static checks passed"