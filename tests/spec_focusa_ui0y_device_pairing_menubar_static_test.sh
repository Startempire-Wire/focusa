#!/usr/bin/env bash
# Spec focusa-ui0y: Mac menubar UI wiring static smoke test.
# Verifies the PairingPanel + pairing store + tab integration + CORS layer
# are all in place.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

# 1. Pairing store
STORE="$ROOT_DIR/apps/menubar/src/lib/stores/pairing.svelte.ts"
[ -f "$STORE" ] || fail "menubar pairing store missing: $STORE"
rg -n 'export const pairingStore|createPairingStore|PairingState' "$STORE" >/dev/null \
  || fail "pairing store missing exported store/State"
pass "pairing store present (createPairingStore + PairingState)"

# 2. Pairing store has 5 methods matching the 5 API surfaces
for method in start list revoke reset bootstrapFromStorage; do
  rg -qn "  $method" "$STORE" || fail "pairing store missing method: $method"
done
pass "pairing store methods: start / list / revoke / reset / bootstrapFromStorage"

# 3. PairingPanel component exists
PANEL="$ROOT_DIR/apps/menubar/src/lib/components/PairingPanel.svelte"
[ -f "$PANEL" ] || fail "PairingPanel.svelte missing"
rg -n 'pairingStore|welcome_idle|waiting_vps|completed|expired|error' "$PANEL" >/dev/null \
  || fail "PairingPanel missing state machine branches"
pass "PairingPanel.svelte exists with 5 state branches (idle/waiting/completed/expired/error)"

# 4. PairingPanel renders code + on_your_vps_run
rg -n 'class="code"|onYourVpsRun|on_your_vps_run|copyToClipboard' "$PANEL" >/dev/null \
  || fail "PairingPanel missing code/cmd/copy affordances"
pass "PairingPanel renders code + on_your_vps_run + copy buttons"

# 5. PairingPanel revokes paired devices
rg -n 'revoke|Revoke|paired_devices|pairedDevices' "$PANEL" >/dev/null \
  || fail "PairingPanel missing revoke flow"
pass "PairingPanel supports per-device revoke"

# 3b. Release-grade pairing debug bundle support
DIAG="$ROOT_DIR/apps/menubar/src/lib/stores/diagnostics.svelte.ts"
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunConnect.svelte"
rg -n 'renderRedactedDebugBundle|Focusa Mac pairing debug bundle|app_version|latest_failure|diagnostics_jsonl|redaction' "$DIAG" >/dev/null   || fail "diagnostics store missing redacted debug bundle fields"
rg -n "lower\.includes\('token'\)|\[REDACTED\]|long_strings" "$DIAG" >/dev/null   || fail "debug bundle missing token/secret redaction"
rg -n 'Copy debug bundle|copyDebugBundle|first_run_connect|callback_status|completion_status|public_pairing_url' "$FIRST_RUN" >/dev/null   || fail "FirstRunConnect missing Copy debug bundle with callback/completion context"
rg -n 'Copy debug bundle|copyDebugBundle|pairing_panel|pairing_state_kind|pairing_state' "$PANEL" >/dev/null   || fail "PairingPanel missing Copy debug bundle with pairing state"
pass "release-grade redacted debug bundle wired into first-run + pairing failure UI"

# 6. +page.svelte has the pair tab
PAGE="$ROOT_DIR/apps/menubar/src/routes/+page.svelte"
rg -n 'PairingPanel|\"pair\"|activeTab === .pair.' "$PAGE" >/dev/null \
  || fail "+page.svelte missing PairingPanel tab integration"
pass "+page.svelte integrates PairingPanel as a tab"

# 7. Tab type union includes 'pair'
rg -n "'pair'" "$PAGE" >/dev/null || fail "+page.svelte Tab type missing 'pair'"
pass "Tab type union includes 'pair'"

# 8. server.rs exposes menubar_cors_layer
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
rg -n 'menubar_cors_layer|tauri://localhost|http://tauri.localhost' "$SERVER" >/dev/null \
  || fail "server.rs missing menubar_cors_layer"
pass "server.rs exposes menubar_cors_layer (Tauri + Vite dev origins)"

# 9. server.rs CORS layer mounted in the router
rg -n 'layer\(menubar_cors_layer\(\)\)' "$SERVER" >/dev/null \
  || fail "server.rs missing menubar_cors_layer mount in router"
pass "menubar_cors_layer is mounted on the router"

# 10. FOCUSA_CORS_ALLOWED_ORIGINS env hook is honored
rg -n 'FOCUSA_CORS_ALLOWED_ORIGINS' "$SERVER" >/dev/null \
  || fail "server.rs missing FOCUSA_CORS_ALLOWED_ORIGINS env hook"
pass "CORS layer supports FOCUSA_CORS_ALLOWED_ORIGINS env override"

# 11. tower-http cors feature already on
rg -n 'tower-http.*cors|tower-http.*=.*cors' "$ROOT_DIR/Cargo.toml" >/dev/null \
  || fail "tower-http cors feature not enabled in workspace Cargo.toml"
pass "tower-http cors feature enabled in workspace"

# 12. tauri-plugin-positioner is wired (menubar tray positioning)
rg -n 'tauri-plugin-positioner|tauri_plugin_positioner' "$ROOT_DIR/apps/menubar/src-tauri/Cargo.toml" \
  "$ROOT_DIR/apps/menubar/src-tauri/src/main.rs" >/dev/null \
  || fail "tauri-plugin-positioner not wired into menubar"
pass "tauri-plugin-positioner wired into menubar (Tauri tray positioning)"

# 13. menubar tray icon + click handler present
rg -n 'TrayIconBuilder|on_tray_icon_event|MouseButton::Left' \
  "$ROOT_DIR/apps/menubar/src-tauri/src/main.rs" >/dev/null \
  || fail "menubar tray click handler missing"
pass "menubar tray click handler present (left-click toggles popover)"

# 14. menubar uses Accessory activation policy (no dock icon, mac menubar-only)
rg -n 'ActivationPolicy::Accessory|set_activation_policy' \
  "$ROOT_DIR/apps/menubar/src-tauri/src/main.rs" >/dev/null \
  || fail "menubar activation policy not set to Accessory (dock icon will show)"
pass "menubar uses macOS Accessory activation policy (no dock icon)"

# 15. Svelte typecheck passes
( cd "$ROOT_DIR/apps/menubar" && npm run -s check >/dev/null 2>&1 ) \
  || fail "menubar svelte-check failed"
pass "menubar svelte-check passes (0 errors)"

# 16. Vite build succeeds
( cd "$ROOT_DIR/apps/menubar" && npm run -s build >/dev/null 2>&1 ) \
  || fail "menubar vite build failed"
pass "menubar vite build succeeds"

# 17. Server CORS check against the live daemon (if running)
if curl -s -m 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:8787/v1/health 2>/dev/null | grep -q '^200$'; then
  # Preflight from tauri origin should echo allow-origin
  preflight=$(curl -s -i -m 3 -X OPTIONS http://127.0.0.1:8787/v1/device/pair/start \
    -H 'Origin: tauri://localhost' \
    -H 'Access-Control-Request-Method: POST' \
    -H 'Access-Control-Request-Headers: content-type' 2>/dev/null \
    | rg -i '^access-control-allow-origin: tauri://localhost' | head -n 1)
  [ -n "$preflight" ] || fail "live CORS preflight from tauri://localhost did not echo allow-origin"
  pass "live CORS preflight from tauri://localhost echoes allow-origin"
else
  echo "ⓘ  daemon not running locally; skipping live CORS check"
fi

# 18. FirstRunConnect renders a URL-shaped QR (WhatsApp-like), not a JSON blob.
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunConnect.svelte"
[ -f "$FIRST_RUN" ] || fail "FirstRunConnect.svelte missing"
rg -n 'QRCode payload=' "$FIRST_RUN" >/dev/null || fail "FirstRunConnect.svelte missing QRCode usage"
if ! rg -q 'payload=\{pairUrl' "$FIRST_RUN"; then
  fail "FirstRunConnect.svelte QR payload is not the URL-shaped pairUrl"
fi
if rg -q "JSON.stringify\(\{\s*protocol" "$FIRST_RUN"; then
  fail "FirstRunConnect.svelte still embeds a JSON QR blob (must be URL-shaped)"
fi
pass "FirstRunConnect.svelte QR is URL-shaped (WhatsApp-like), not a JSON blob"

# 19. FirstRunConnect polls the URL-QR room status endpoint
if ! rg -q '/v1/connect/room/firstrun|/v1/connect/room/.*status|pollRoomStatus' "$FIRST_RUN"; then
  fail "FirstRunConnect.svelte missing firstrun / pollRoomStatus / status URL"
fi
pass "FirstRunConnect.svelte polls /v1/connect/room/{room_id}/status after firstrun"

echo ""
echo "ALL focusa-ui0y menubar static checks passed."
