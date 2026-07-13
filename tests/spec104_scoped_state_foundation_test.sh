#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 "$ROOT/tests/spec104_singleton_inventory_gate.py"
jq -e '.schema == "focusa.spec104.singleton_inventory.v1" and (.entries | length > 0)' \
  "$ROOT/config/spec104-scoped-state-inventory.json" >/dev/null
jq -e '.properties.schema.const == "focusa.scoped_result.v1" and ."$defs".scopeRef.properties.scope_kind.enum == ["project","host"]' \
  "$ROOT/config/scoped-state.schema.json" >/dev/null
rg -q 'pub struct ScopeRef' "$ROOT/crates/focusa-core/src/scoped_state.rs"
rg -q 'pub struct WorkstreamKey' "$ROOT/crates/focusa-core/src/scoped_state.rs"
rg -q 'pub struct AttachmentKey' "$ROOT/crates/focusa-core/src/scoped_state.rs"
rg -q 'pub struct ScopedCrdtRecord' "$ROOT/crates/focusa-core/src/scoped_state.rs"
rg -q 'pub struct ScopedResultEnvelope' "$ROOT/crates/focusa-core/src/scoped_state.rs"
rg -q 'export interface ScopedResultEnvelope' "$ROOT/apps/pi-extension/src/scoped-state.ts"
rg -q 'renderScopedResultHuman' "$ROOT/apps/pi-extension/src/scoped-state.ts"

cd "$ROOT"
npx --yes tsx tests/spec104_scoped_state_runtime_test.mts
cargo test -p focusa-core scoped_state --lib
printf 'PASS: Spec104 typed scoped state foundation and inventory gate\n'
