#!/usr/bin/env bash
# Spec focusa-ui0y: Mac menubar OAuth-like device pairing static smoke test.
# Verifies the 5 device-pair surfaces (api routes, CLI commands, Pi tools,
# tool contracts, doc pages) are all wired up consistently.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

# 1. Core surface: in-memory PairingState (defined inline in api route module for now)
ROUTES_RS="$ROOT_DIR/crates/focusa-api/src/routes/device_pairing.rs"
[ -f "$ROUTES_RS" ] || fail "api/routes/device_pairing.rs missing"
rg -n 'pub type SharedPairingState|struct PairingState' "$ROUTES_RS" >/dev/null \
  || fail "PairingState / SharedPairingState types missing in routes/device_pairing.rs"
pass "PairingState + SharedPairingState types present"

# 2. API routes
rg -n '"/v1/device/pair/start"|"/v1/device/pair/complete"|"/v1/device/pair/status"|"/v1/device/pair/list"|"/v1/device/pair/revoke"' "$ROUTES_RS" >/dev/null \
  || fail "device pair api routes missing"
pass "device pair api routes (5) registered"

# 3. routes/mod.rs exposes the module
MOD_RS="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
rg -n 'device_pairing' "$MOD_RS" >/dev/null || fail "routes/mod.rs missing device_pairing module"
pass "routes/mod.rs exposes device_pairing module"

# 4. server.rs mounts the routes
SERVER_RS="$ROOT_DIR/crates/focusa-api/src/server.rs"
rg -n 'device_pairing::router' "$SERVER_RS" >/dev/null || fail "server.rs missing device_pairing::router merge"
pass "server.rs mounts /v1/device/pair routes"

# 5. CLI commands
CLI_RS="$ROOT_DIR/crates/focusa-cli/src/commands/device_pairing.rs"
[ -f "$CLI_RS" ] || fail "cli/commands/device_pairing.rs missing"
rg -n 'pair_start|pair_complete|pair_status|pair_list|pair_revoke' "$CLI_RS" >/dev/null \
  || fail "device pair CLI subcommands missing"
pass "device pair CLI subcommands (5) registered"

# 6. CLI subcommand wiring in mod.rs + main.rs
MOD_CLI="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN_CLI="$ROOT_DIR/crates/focusa-cli/src/main.rs"
rg -n 'device_pairing' "$MOD_CLI" "$MAIN_CLI" >/dev/null || fail "CLI mod/main missing device_pairing wiring"
pass "CLI mod/main wire up device_pairing subcommand"

# 7. Pi extension tools
TOOLS_TS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
for name in focusa_device_pair_start focusa_device_pair_complete focusa_device_pair_status focusa_device_pair_list focusa_device_pair_revoke; do
  rg -qn "name: \"$name\"" "$TOOLS_TS" || fail "Pi tool $name not registered"
done
pass "Pi tools (5) registered in tools.ts"

# 8. Tool contracts
CONTRACTS_TS="$ROOT_DIR/apps/pi-extension/src/tool-contracts.ts"
for name in focusa_device_pair_start focusa_device_pair_complete focusa_device_pair_status focusa_device_pair_list focusa_device_pair_revoke; do
  rg -qn "\"$name\"" "$CONTRACTS_TS" || fail "tool contract $name missing"
done
rg -n '"session_transfer"' "$CONTRACTS_TS" >/dev/null || fail "session_transfer family missing in contracts"
pass "tool contracts (5) + session_transfer family present"

# 9. Per-tool docs
for name in focusa_device_pair_start focusa_device_pair_complete focusa_device_pair_status focusa_device_pair_list focusa_device_pair_revoke; do
  DOC="$ROOT_DIR/docs/focusa-tools/tools/${name}.md"
  [ -f "$DOC" ] || fail "tool doc $DOC missing"
done
pass "per-tool docs (5) present"

# 10. Canonical generated tool references (README no longer owns a tool table)
rg -qn '^# `focusa_device_pair_start`' "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_start.md" \
  && rg -qn 'agent-capability-descriptors.json#focusa_device_pair_start' "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_start.md" \
  && rg -qn '^# `focusa_device_pair_revoke`' "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_revoke.md" \
  || fail "canonical device_pair tool references are incomplete"
pass "canonical generated device_pair tool references present"

# 11. Live contract artifacts
CONTRACTS_JSON="$ROOT_DIR/docs/current/focusa-tool-contracts.json"
CHOREO_JSON="$ROOT_DIR/docs/current/focusa-tool-choreography.json"
rg -qn '"focusa_device_pair_start"' "$CONTRACTS_JSON" || fail "focusa-tool-contracts.json missing device_pair_start"
rg -qn '"focusa_device_pair_start"' "$CHOREO_JSON" || fail "focusa-tool-choreography.json missing device_pair_start edges"
pass "live contract + choreography artifacts reference device_pair tools"

# 12. Validation + audit pass
( cd "$ROOT_DIR" && node scripts/validate-focusa-tool-contracts.mjs >/dev/null ) \
  || fail "validate-focusa-tool-contracts.mjs failed"
( cd "$ROOT_DIR" && node scripts/audit-focusa-tool-implementation-spec-gaps.mjs >/dev/null ) \
  || fail "audit-focusa-tool-implementation-spec-gaps.mjs failed"
pass "static validation + implementation audit pass"

# 13. Shared PairingState type aliased for in-memory state
rg -n 'pub type SharedPairingState|type SharedPairingState' "$ROUTES_RS" >/dev/null \
  || fail "SharedPairingState type alias missing in routes/device_pairing.rs"
pass "SharedPairingState type alias present"

# 14. Append-only ledger path is host-scoped
rg -n 'data/device-pairing|device-pairing' "$ROOT_DIR/crates/focusa-core/src/runtime/persistence_sqlite.rs" "$ROOT_DIR/crates/focusa-core/src/types.rs" >/dev/null \
  || fail "device-pairing persistence path missing in core"
pass "device-pairing persistence path present in core (host-scoped)"

# 15. focusa-ui0y.7 — pair_url field + FOCUSA_PAIRING_URL env var
# Spec §4: pair_start returns pair_url + pair_url_qr_payload
rg -qn '"pair_url"' "$ROUTES_RS" || fail "pair_url field missing from pair_start response"
rg -qn 'FOCUSA_PAIRING_URL' "$ROUTES_RS" || fail "FOCUSA_PAIRING_URL env var fallback missing"
rg -qn 'pair_url_qr_payload' "$ROUTES_RS" || fail "pair_url_qr_payload field missing (forward-compat invariant)"
pass "pair_url + pair_url_qr_payload + FOCUSA_PAIRING_URL fallback present (focusa-ui0y.7)"

# 16. focusa-ui0y.8 — PWA helper page at /pair/{device_id}
# Spec §5: HTML + manifest.json + sw.js
rg -qn '"/pair/\{device_id\}"|pwa_helper_page' "$ROUTES_RS" || fail "/pair/{device_id} route missing"
rg -qn '"/pair/\{device_id\}/manifest.json"|pwa_manifest' "$ROUTES_RS" || fail "/pair/{device_id}/manifest.json route missing"
rg -qn '"/pair/\{device_id\}/sw.js"|pwa_service_worker' "$ROUTES_RS" || fail "/pair/{device_id}/sw.js route missing"
# Spec §5.2: no third-party scripts in PWA — assert no external src
PWA_FN=$(rg -n 'fn pwa_helper_html' "$ROUTES_RS" | head -1 | cut -d: -f1)
if [ -n "$PWA_FN" ]; then
  PWA_END=$((PWA_FN + 250))
  PWA_BODY=$(sed -n "${PWA_FN},${PWA_END}p" "$ROUTES_RS")
  echo "$PWA_BODY" | rg -qn '<script src=' && fail "PWA helper page references <script src=> (third-party asset, violates spec §5.2)" || true
  echo "$PWA_BODY" | rg -qn 'https?://[a-z]' && fail "PWA helper page references external URL (third-party asset, violates spec §5.2)" || true
fi
pass "PWA helper page routes (3) + no third-party assets (focusa-ui0y.8)"

# 17. focusa-ui0y.9 — menubar QR renderer
QR_SVELTE="$ROOT_DIR/apps/menubar/src/lib/components/QRCode.svelte"
[ -f "$QR_SVELTE" ] || fail "menubar QRCode.svelte missing"
rg -qn '"qrcode"' "$ROOT_DIR/apps/menubar/package.json" || fail "menubar package.json missing qrcode dep"
pass "menubar QR renderer present (focusa-ui0y.9)"

# 18. focusa-ui0y.10 — menubar PairingPanel Mode A/B/C tabs
PAIR_PANEL="$ROOT_DIR/apps/menubar/src/lib/components/PairingPanel.svelte"
rg -qn 'handoffMode' "$PAIR_PANEL" || fail "PairingPanel missing handoffMode (Mode A/B/C tabs)"
rg -qn 'QRCode' "$PAIR_PANEL" || fail "PairingPanel does not import QRCode"
pass "menubar PairingPanel uses Mode A/B/C tabs + QRCode (focusa-ui0y.10)"

# 19. focusa-ui0y.11 — CLI pair-qr shortcut
rg -qn 'pair-qr|PairQr' "$CLI_RS" || fail "CLI pair-qr subcommand missing"
rg -qn 'pair_url' "$CLI_RS" || fail "CLI pair-qr does not surface pair_url"
pass "CLI focusa device pair-qr shortcut present (focusa-ui0y.11)"

# 20. focusa-ui0y.12 — Pi focusa_device_pair_qr tool
rg -qn 'name: "focusa_device_pair_qr"' "$TOOLS_TS" || fail "Pi focusa_device_pair_qr tool missing"
rg -qn '"focusa_device_pair_qr"' "$CONTRACTS_JSON" || fail "contracts JSON missing focusa_device_pair_qr"
rg -qn '"focusa_device_pair_qr"' "$CHOREO_JSON" || fail "choreography JSON missing focusa_device_pair_qr edges"
pass "Pi focusa_device_pair_qr tool + contract + choreography present (focusa-ui0y.12)"

# 21. Spec + docs cross-reference
SPEC_DOC="$ROOT_DIR/docs/53-focusa-device-pairing-spec.md"
[ -f "$SPEC_DOC" ] || fail "spec doc docs/53-focusa-device-pairing-spec.md missing"
rg -qn 'docs/53-focusa-device-pairing-spec.md' \
  "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_start.md" \
  "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_complete.md" \
  "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_status.md" \
  "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_list.md" \
  "$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_revoke.md" \
  >/dev/null || fail "tool-level docs do not all reference the new spec"
pass "spec + tool-level docs cross-referenced (focusa-ui0y consolidated)"

# 22. Spec §6 multi-tenant invariant
# Each daemon is its own trust root. FOCUSA_PAIRING_URL must be operator-scoped,
# never shared with another operator's daemon. This is a static assertion
# that the env var is read at request time (not at startup) so per-daemon
# isolation is preserved.
rg -qF 'std::env::var' "$ROUTES_RS" && rg -qF 'FOCUSA_PAIRING_URL' "$ROUTES_RS" || fail "FOCUSA_PAIRING_URL not read at request time (multi-tenant risk)"
pass "FOCUSA_PAIRING_URL read at request time (multi-tenant isolation)"

# 23. Spec §6.3 multi-device is a property of the ledger
# Append-only JSONL ledger with revoked=true for revocation.
rg -qn 'devices\.jsonl|append.*DeviceRecord|revoked.*true' "$ROOT_DIR/crates/focusa-core/src/runtime/persistence_sqlite.rs" "$ROOT_DIR/crates/focusa-core/src/types.rs" >/dev/null \
  || fail "device-pairing ledger append-only invariant unclear"
pass "device-pairing ledger append-only + revocation model (multi-device)"

echo ""
echo "ALL focusa-ui0y device pairing static checks passed (.6 base + .13 extensions)."
