#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAIN="$ROOT/crates/focusa-cli/src/main.rs"
MOD="$ROOT/crates/focusa-cli/src/commands/mod.rs"
SRC="$ROOT/crates/focusa-cli/src/commands/update.rs"
API="$ROOT/crates/focusa-api/src/routes/update.rs"
API_MOD="$ROOT/crates/focusa-api/src/routes/mod.rs"
API_SERVER="$ROOT/crates/focusa-api/src/server.rs"
SPEC="$ROOT/docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f "$SRC" ]] || fail "missing update command module"
rg -q 'pub mod update;' "$MOD" || fail "update module not exported"
rg -q 'Update\(commands::update::UpdateCmd\)' "$MAIN" || fail "main CLI missing Update command"
rg -q 'Commands::Update\(cmd\) => commands::update::run\(cmd, cli\.json\)' "$MAIN" || fail "main CLI missing update dispatch"
rg -q 'enum UpdateCmd' "$SRC" || fail "missing UpdateCmd enum"
rg -q 'Status\(UpdateStatusArgs\)' "$SRC" || fail "missing update status subcommand"
rg -q 'Check\(UpdateStatusArgs\)' "$SRC" || fail "missing update check subcommand"
rg -q 'read_only: true' "$SRC" || fail "update status must be read-only"
rg -q 'mutations_performed: false' "$SRC" || fail "update status must not mutate"
rg -q 'Policy\(UpdatePolicyCmd\)|UpdatePolicySetArgs|FOCUSA_UPDATE_POLICY|default_policy_from_license' "$SRC" || fail "missing CLI update policy show/set/defaults"
rg -q 'auto_apply_allowed.*false|auto_apply_allowed: false' "$SRC" || fail "CLI policy must keep auto-apply disabled in this slice"
rg -q 'focusa-daemon --version starts the server|--version as startup input' "$SRC" || fail "daemon version probe safety note missing"
rg -q 'auto-apply remains disabled|release manifest eligibility/signature/provenance is required' "$SRC" || fail "missing auto-apply guard warning"
rg -q 'cli|daemon|tui' "$SRC" || fail "inventory must include CLI/daemon/TUI"
[[ -f "$API" ]] || fail "missing update API route module"
rg -q 'pub mod update;' "$API_MOD" || fail "API routes module does not export update"
rg -q 'routes::update::router\(\)' "$API_SERVER" || fail "API server does not merge update router"
rg -q '/v1/update/status|/v1/update/check' "$API" || fail "missing update status/check API routes"
rg -q '/v1/update/policy|update_policy_set|default_policy_from_license' "$API" || fail "missing update policy API route/defaults"
rg -q 'read_only.*true|mutations_performed.*false' "$API" || fail "API update routes must be read-only/no-mutation"
rg -q 'cli|daemon|tui' "$API" || fail "API inventory must include CLI/daemon/TUI"
rg -q 'focusa update status --json|focusa update check --channel dev --json|focusa update status' "$SPEC" || fail "Spec128 update CLI surface missing"
rg -q 'dev_mode.*automatic|evaluation notify-only|update policy' "$SPEC" || fail "Spec128 policy/dev_mode requirements missing"

pass "Spec128 read-only update status/check CLI surface present"
