#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MAIN="$ROOT_DIR/apps/menubar/src-tauri/src/main.rs"
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunConnect.svelte"
PWA="$ROOT_DIR/crates/focusa-api/src/routes/device_pairing.rs"
TRANSPORT="$ROOT_DIR/scripts/phone-bridge-transport.sh"
GLOSSARY="$ROOT_DIR/docs/00-glossary.md"
PLAN="$ROOT_DIR/docs/54-focusa-pairing-room-plan.md"

assert_has() {
  local file="$1" pattern="$2" label="$3"
  if rg -n "$pattern" "$file" >/dev/null; then
    echo "✓ PASS: ${label}"
  else
    echo "✗ FAIL: ${label}" >&2
    echo "Missing pattern '${pattern}' in ${file}" >&2
    exit 1
  fi
}

assert_not_has() {
  local file="$1" pattern="$2" label="$3"
  if rg -n "$pattern" "$file" >/dev/null; then
    echo "✗ FAIL: ${label}" >&2
    echo "Found unexpected pattern '${pattern}' in ${file}" >&2
    exit 1
  else
    echo "✓ PASS: ${label}"
  fi
}

bash -n "$TRANSPORT"

assert_has "$MAIN" 'focusa_start_bridge_callback' 'Tauri exposes start_bridge_callback command'
assert_has "$MAIN" 'focusa_take_bridge_completion' 'Tauri exposes take_bridge_completion command'
assert_has "$MAIN" 'focusa-phone-bridge' 'Tauri callback handler route present'
assert_has "$MAIN" 'best_local_ip' 'Tauri resolves best local IP for callback'
assert_has "$MAIN" 'OnceLock' 'Tauri uses thread-safe OnceLock for callback state'
assert_has "$MAIN" 'bridge_completions' 'Tauri stores callback completions'
assert_has "$MAIN" 'TcpListener' 'Tauri uses TcpListener for callback server'
assert_has "$FIRST_RUN" 'focusa_start_bridge_callback' 'Mac frontend calls start_bridge_callback'
assert_has "$FIRST_RUN" 'focusa_take_bridge_completion' 'Mac frontend polls take_bridge_completion'
assert_has "$FIRST_RUN" 'callbackPollHandle' 'Mac frontend uses callback polling interval'
assert_has "$FIRST_RUN" 'mac_callback:' 'Mac frontend includes mac_callback in QR offer'
assert_has "$PWA" 'fetch.*mac_callback' 'PWA POSTs to mac_callback URL on approval'
assert_has "$PWA" 'lastOffer.mac_callback' 'PWA reads mac_callback from lastOffer'
assert_has "$TRANSPORT" 'private_or_tailscale' 'Transport resolver covers private/Tailscale routes'
assert_has "$TRANSPORT" 'Tailscale' 'Transport resolver mentions Tailscale'
assert_has "$TRANSPORT" 'Focusa Connect' 'Transport resolver validates Focusa Connect page'
assert_has "$TRANSPORT" 'Focusa-contained' 'Transport resolver is contained within Focusa'
assert_has "$TRANSPORT" 'never mutates a live webserver' 'Transport resolver is contained within Focusa'
assert_has "$GLOSSARY" 'Phone Bridge Flow' 'Glossary defines Phone Bridge Flow'
assert_has "$GLOSSARY" 'Mac Completion Payload' 'Glossary defines Mac Completion Payload'
assert_has "$GLOSSARY" 'Bridge Room' 'Glossary defines Bridge Room'
assert_has "$PLAN" 'callback listener' 'Phone Bridge plan covers callback listener'
assert_has "$PLAN" 'automatically' 'Phone Bridge plan covers automatic Mac completion'
assert_has "$PLAN" 'POSTs.*Mac Completion Payload' 'Phone Bridge plan covers automatic POST delivery'
assert_not_has "$TRANSPORT" 'cPanel\|whm\|httpd\.conf\|LiteSpeed\.conf' 'Transport resolver contains no live webserver mutation'

echo "Phone Bridge automatic callback static test: PASS"
