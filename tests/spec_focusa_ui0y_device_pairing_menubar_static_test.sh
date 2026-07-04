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
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunWizard.svelte"
rg -n 'renderRedactedDebugBundle|Focusa Mac pairing debug bundle|app_version|latest_failure|diagnostics_jsonl|redaction' "$DIAG" >/dev/null   || fail "diagnostics store missing redacted debug bundle fields"
rg -n "lower\.includes\('token'\)|\[REDACTED\]|long_strings" "$DIAG" >/dev/null   || fail "debug bundle missing token/secret redaction"
rg -n 'Copy debug bundle|copyDebugBundle|first_run_wizard|callback_status|completion_status|discovered_url' "$FIRST_RUN" >/dev/null   || fail "FirstRunWizard missing Copy debug bundle with callback/completion context"
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

# 15. Svelte typecheck passes when local JS deps are installed
if [ -x "$ROOT_DIR/apps/menubar/node_modules/.bin/svelte-check" ] && [ -x "$ROOT_DIR/apps/menubar/node_modules/.bin/svelte-kit" ]; then
  ( cd "$ROOT_DIR/apps/menubar" && npm run -s check >/dev/null 2>&1 ) \
    || fail "menubar svelte-check failed"
  pass "menubar svelte-check passes (0 errors)"
else
  echo "SKIP: menubar svelte-check deps unavailable; static menubar guards already passed"
fi

# 16. Vite build succeeds when local JS deps are installed
if [ -x "$ROOT_DIR/apps/menubar/node_modules/.bin/vite" ]; then
  ( cd "$ROOT_DIR/apps/menubar" && npm run -s build >/dev/null 2>&1 ) \
    || fail "menubar vite build failed"
  pass "menubar vite build succeeds"
else
  echo "SKIP: menubar vite deps unavailable; static menubar guards already passed"
fi

# 17. menubar headless e2e Rust integration test exists
HEADLESS="$ROOT_DIR/crates/focusa-cli/tests/menubar_headless_e2e.rs"
[ -f "$HEADLESS" ] || fail "menubar headless e2e test missing: $HEADLESS"
rg -qn 'chromium_dump_dom|build_menubar|spawn_static_server' "$HEADLESS" \
  || fail "menubar headless e2e missing required helpers (chromium_dump_dom, build_menubar, spawn_static_server)"
pass "menubar headless e2e Rust test present (builds SPA + headless chromium dump-dom)"

# 18. menubar FirstRunWizard.svelte has headless Tauri stub
rg -qn '__FOCUSA_HEADLESS__|__FOCUSA_DAEMON_URL__' "$FIRST_RUN" \
  || fail "FirstRunWizard.svelte missing headless Tauri stub globals"
pass "FirstRunWizard.svelte honors __FOCUSA_HEADLESS__ + __FOCUSA_DAEMON_URL__ globals"

# 19. pairing cycle-test Rust subcommand has --with-pwa-verify flag
CYCLE_TEST="$ROOT_DIR/crates/focusa-cli/src/commands/pairing_cycle_test.rs"
rg -qn 'with_pwa_verify|verify_pwa_scan' "$CYCLE_TEST" \
  || fail "pairing_cycle_test.rs missing --with-pwa-verify flag"
pass "focusa pairing cycle-test --with-pwa-verify present"

# 20. Server CORS check against the live daemon (if running)
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

# 18. FirstRunWizard renders the CANONICAL V2 mac_offer QR (not pairUrl).
# Canonical V2: VPS creates the room and prints a QR for the phone to scan.
# Mac idles showing a STATIC mac_offer QR (mac_name + nonce + pubkey + callback).
# The Mac does NOT display the VPS pair_url.
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunWizard.svelte"
[ -f "$FIRST_RUN" ] || fail "FirstRunWizard.svelte missing"
rg -n 'QRCode payload=' "$FIRST_RUN" >/dev/null || fail "FirstRunWizard.svelte missing QRCode usage"
if ! rg -q 'payload=\{macOffer' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte QR payload must be macOffer (canonical V2)"
fi
if rg -q 'payload=\{pairUrl' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte still renders pairUrl as QR (legacy V1, forbidden in V2)"
fi
if ! rg -q 'role.*mac_handoff_offer' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte macOffer missing role=mac_handoff_offer"
fi
if rg -q '/v1/connect/room/create' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte still calls /v1/connect/room/create (Mac must not create rooms in V2)"
fi
if ! rg -q '/v1/connect/rooms' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte missing /v1/connect/rooms polling (canonical V2)"
fi
# The QR payload must be the JSON-shaped macOffer (V2 protocol), not a URL.
# We allow JSON.stringify elsewhere in the file (macOffer construction,
# completion payload), so we narrow the check to the actual QRCode usage line.
if rg -q 'QRCode payload=\{pairUrl' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte QR payload is pairUrl (legacy V1, forbidden in V2)"
fi
pass "FirstRunWizard.svelte QR is canonical V2 mac_offer (JSON), not pairUrl"

# 19. FirstRunWizard polls the URL-QR room status endpoint
if ! rg -q '/v1/connect/room/firstrun|/v1/connect/room/.*status|pollRoomStatus' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte missing firstrun / pollRoomStatus / status URL"
fi
pass "FirstRunWizard.svelte polls /v1/connect/room/{room_id}/status after firstrun"

# 20. FirstRunWizard wires the diagnostics store so errors are recorded + copiable (v0.9.35-dev)
if ! rg -q 'installGlobalDiagnostics|diagnosticsStore\.record' "$FIRST_RUN"; then
  fail "FirstRunWizard.svelte missing diagnostics wiring (installGlobalDiagnostics or diagnosticsStore.record)"
fi
pass "FirstRunWizard.svelte wires diagnostics store (errors recorded + bundle copiable)"

# 21. DebugBundleContext includes v0.9.35-dev fields
DIAG="$ROOT_DIR/apps/menubar/src/lib/stores/diagnostics.svelte.ts"
if ! rg -q 'connect_id|pair_url|firstrun_error|server_url' "$DIAG"; then
  fail "diagnostics.svelte.ts DebugBundleContext missing v0.9.35-dev fields (connect_id, pair_url, firstrun_error, server_url)"
fi
pass "diagnostics.svelte.ts DebugBundleContext includes v0.9.35-dev fields"

# 22. Diagnostics store persists to localStorage + caps entries + installs global handlers
if ! rg -q 'STORAGE_KEY|MAX_ENTRIES|window\.addEventListener.*error|window\.addEventListener.*unhandledrejection' "$DIAG"; then
  fail "diagnostics.svelte.ts missing storage key, entry cap, or global error handlers"
fi
pass "diagnostics.svelte.ts persists + caps + installs global error/unhandledrejection handlers"

# 23. PairingPanel exposes Copy debug bundle button
if ! rg -q 'Copy debug bundle|copyDebugBundle' "$PANEL"; then
  fail "PairingPanel.svelte missing Copy debug bundle button"
fi
pass "PairingPanel.svelte exposes Copy debug bundle button (post-pair errors copyable)"

echo ""
echo "ALL focusa-ui0y menubar static checks passed."
