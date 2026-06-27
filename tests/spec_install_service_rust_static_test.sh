#!/usr/bin/env bash
# First-class Rust install-service guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

MOD="$ROOT_DIR/crates/focusa-cli/src/commands/service.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"

rg -n 'pub struct InstallServiceArgs|pub async fn run|enum ServiceManager|render_systemd_unit|render_launchd_plist|focusa-daemon.service|com.startempire.focusa-daemon' "$MOD" >/dev/null \
  || fail "service.rs missing first-class systemd + launchd rendering"
rg -n 'Commands::InstallService\(args\) => commands::service::run\(args' "$MAIN" >/dev/null \
  || fail "main.rs not dispatching focusa install-service"
pass "focusa install-service is wired in Rust core"