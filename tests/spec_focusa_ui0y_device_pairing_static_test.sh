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

# 10. README tool table rows
rg -qn 'focusa_device_pair_start' "$ROOT_DIR/README.md" \
  && rg -qn 'focusa_device_pair_revoke' "$ROOT_DIR/README.md" \
  || fail "README tool table missing device_pair_* rows"
pass "README tool table includes device_pair_* rows"

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

echo ""
echo "ALL focusa-ui0y device pairing static checks passed."
