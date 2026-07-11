#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
SPEC="$ROOT/docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md"

if ! command -v rg >/dev/null 2>&1; then
  rg() { grep -E "$@"; }
fi

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

rg -q 'pub preflight: bool' "$INSTALL" || fail "install args missing --preflight"
rg -q 'pub no_animation: bool' "$INSTALL" || fail "install args missing --no-animation"
rg -q 'pub quiet: bool' "$INSTALL" || fail "install args missing --quiet"
rg -q 'pub assume_yes: bool' "$INSTALL" || fail "install args missing --assume-yes"
rg -q 'focusa.install_preflight.v1' "$INSTALL" || fail "missing install preflight schema"
rg -q 'InstallPreflightReport|PreflightSystem|PreflightDependency|DependencyInstallOffer|TerminalUxPreflight' "$INSTALL" || fail "missing preflight report structs"
rg -q 'detect_preflight_system|detect_dependencies|install_hint|terminal_ux_preflight' "$INSTALL" || fail "missing preflight detection functions"
rg -q 'auto_install_performed: false|requires_explicit_consent: true' "$INSTALL" || fail "preflight must not auto-install dependencies"
rg -q 'curl|python3|sha256sum|tar' "$INSTALL" || fail "required bootstrap dependency checks missing"
rg -q 'NO_COLOR|CI|--no-animation|non_interactive_terminal' "$INSTALL" || fail "terminal intro fallback rules missing"
rg -q 'Installer first-run and system environment preflight|Missing dependency handling|Intro and terminal UX' "$SPEC" || fail "Spec128 installer preflight sections missing"

pass "Spec128 installer preflight/dependency/terminal UX scaffold present"
