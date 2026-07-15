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

# Existing command surface checks
rg -q 'pub preflight: bool' "$INSTALL" || fail "install args missing --preflight"
rg -q 'pub no_animation: bool' "$INSTALL" || fail "install args missing --no-animation"
rg -q 'pub quiet: bool' "$INSTALL" || fail "install args missing --quiet"
rg -q 'pub assume_yes: bool' "$INSTALL" || fail "install args missing --assume-yes"

# Preflight schema / envelopes
rg -q 'focusa\.install_preflight\.v1' "$INSTALL" || fail "missing install preflight schema"
rg -q 'InstallPreflightReport|PreflightSystem|PreflightDependency|DependencyInstallOffer|TerminalUxPreflight' "$INSTALL" || fail "missing preflight report structs"
rg -q 'detect_preflight_system\(|detect_dependencies\(|install_hint\(|terminal_ux_preflight\(' "$INSTALL" || fail "missing preflight detection functions"

# New Spec112/128 environment inventory fields are mandatory in system object
rg -q 'distro: String' "$INSTALL" || fail "missing distro field in PreflightSystem"
rg -q 'os_version: String' "$INSTALL" || fail "missing os_version field in PreflightSystem"
rg -q 'kernel: String' "$INSTALL" || fail "missing kernel field in PreflightSystem"
rg -q 'libc: String' "$INSTALL" || fail "missing libc field in PreflightSystem"
rg -q 'path_targets: Vec<PathTargetSummary>' "$INSTALL" || fail "missing path_targets inventory"
rg -q 'existing_surfaces: Vec<ExistingSurface>' "$INSTALL" || fail "missing existing_surfaces inventory"
rg -q 'cpu: String' "$INSTALL" || fail "missing CPU inventory"
rg -q 'memory: String' "$INSTALL" || fail "missing memory inventory"
rg -q 'disk: String' "$INSTALL" || fail "missing disk inventory"
rg -q 'network: NetworkInventory' "$INSTALL" || fail "missing network inventory"
rg -q 'tls: TlsInventory' "$INSTALL" || fail "missing TLS inventory"
rg -q 'proxy: ProxyInventory' "$INSTALL" || fail "missing proxy inventory"
rg -q 'daemon_health: DaemonHealthInventory' "$INSTALL" || fail "missing daemon health inventory"
rg -q 'license_override: LicenseOverrideInventory' "$INSTALL" || fail "missing license/dev override inventory"
rg -q 'update_policy: UpdatePolicyInventory' "$INSTALL" || fail "missing update-policy inventory"
rg -q 'compatibility: CompatibilityInventory' "$INSTALL" || fail "missing compatibility classification"

# Detection helpers for Spec112/128 inventory dimensions
rg -q 'fn detect_distro_version\(|fn detect_kernel_version\(|fn detect_libc\(' "$INSTALL" || fail "missing OS/library detector helpers"
rg -q 'fn detect_cpu_summary\(|fn detect_memory_summary\(|fn detect_disk_summary\(' "$INSTALL" || fail "missing hardware resource detectors"
rg -q 'fn detect_network_summary\(|fn detect_tls_inventory\(|fn detect_proxy_inventory\(' "$INSTALL" || fail "missing net/cert/proxy detectors"
rg -q 'fn detect_daemon_health\(|fn detect_license_override\(|fn detect_update_policy\(' "$INSTALL" || fail "missing service/licensing/update inventory detectors"
rg -q 'fn classify_compatibility\(' "$INSTALL" || fail "missing compatibility classification function"

# Preflight dependency behavior expectations
rg -q 'auto_install_performed: false|requires_explicit_consent: true' "$INSTALL" || fail "preflight must not auto-install dependencies"
rg -q 'curl|python3|sha256sum|tar' "$INSTALL" || fail "required bootstrap dependency checks missing"
rg -q 'NO_COLOR|CI|--no-animation|non_interactive_terminal' "$INSTALL" || fail "terminal intro fallback rules missing"

# Spec references still expected by this preflight work stream
rg -q 'Installer first-run and system environment preflight|Missing dependency handling|Intro and terminal UX' "$SPEC" || fail "Spec128 installer preflight sections missing"

pass "Spec128 installer preflight inventory struct, detectors, and compatibility checks present"
