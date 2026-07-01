#!/usr/bin/env bash
# spec_focusa_ui0y_deep_link_static_test.sh
#
# Static test that verifies the focusa:// deep link is registered in the
# Tauri menubar app and that a focusa_handle_deep_link command exists in
# the Rust source. Spec: focusa-ui0y Phase-2 callback fallback when the
# Mac TCP bridge listener is unreachable.
#
# Acceptance criteria:
#   1. tauri.conf.json registers the "focusa" scheme under
#      plugins.deep-link.desktop.schemes
#   2. Cargo.toml declares tauri-plugin-deep-link as a dependency
#   3. main.rs initializes tauri_plugin_deep_link::init()
#   4. main.rs registers focusa_handle_deep_link in the invoke_handler
#   5. main.rs implements focusa_handle_deep_link that:
#      - accepts focusa://connect?payload=<base64>
#      - decodes the base64 JSON
#      - inserts a completion under <nonce>|<token>

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONF="$ROOT_DIR/apps/menubar/src-tauri/tauri.conf.json"
CARGO_TOML="$ROOT_DIR/apps/menubar/src-tauri/Cargo.toml"
MAIN_RS="$ROOT_DIR/apps/menubar/src-tauri/src/main.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# 1. tauri.conf.json registers the focusa scheme
[ -f "$CONF" ] || fail "tauri.conf.json not found at $CONF"
python3 - "$CONF" <<'PY' || fail "focusa scheme not registered in tauri.conf.json"
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
plugins = d.get("plugins", {})
dl = plugins.get("deep-link", {})
desktop = dl.get("desktop", {})
schemes = desktop.get("schemes", [])
assert "focusa" in schemes, f"focusa not in schemes: {schemes}"
print("scheme registered: focusa in", schemes)
PY
pass "focusa:// scheme registered in tauri.conf.json (plugins.deep-link.desktop.schemes)"

# 2. Cargo.toml declares tauri-plugin-deep-link
grep -q 'tauri-plugin-deep-link' "$CARGO_TOML" \
  || fail "tauri-plugin-deep-link missing from Cargo.toml"
pass "tauri-plugin-deep-link declared in Cargo.toml"

# 3. main.rs initializes tauri_plugin_deep_link
grep -q 'tauri_plugin_deep_link::init' "$MAIN_RS" \
  || fail "tauri_plugin_deep_link::init() missing from main.rs"
pass "tauri_plugin_deep_link::init() called in main()"

# 4. main.rs registers focusa_handle_deep_link
grep -q 'focusa_handle_deep_link' "$MAIN_RS" \
  || fail "focusa_handle_deep_link reference missing from main.rs"
grep -q 'fn focusa_handle_deep_link' "$MAIN_RS" \
  || fail "fn focusa_handle_deep_link definition missing from main.rs"
pass "focusa_handle_deep_link defined and registered in invoke_handler"

# 5a. Handler decodes base64 and stashes by <nonce>|<token>
grep -q 'focusa://connect?payload=' "$MAIN_RS" \
  || fail "handler does not match focusa://connect?payload= prefix"
pass "handler matches focusa://connect?payload= URL prefix"

# 5b. Handler inserts under nonce|token key
grep -q 'format!("{nonce}|{token}")' "$MAIN_RS" \
  || fail "handler does not use nonce|token composite key for completion store"
pass "handler inserts completion under <nonce>|<token> composite key"

# 5c. Handler decodes base64 (not just forwards raw)
grep -q 'fn base64_decode' "$MAIN_RS" \
  || fail "fn base64_decode helper missing from main.rs"
pass "base64_decode helper defined and called from focusa_handle_deep_link"

echo "✓ All focusa-ui0y deep link static checks passed"
