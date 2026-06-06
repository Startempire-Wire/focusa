#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

log() { printf '\n==> %s\n' "$*"; }

log "Spec98 project scope / bleed runtime proofs"
bun tests/spec98_pi_scope_cache_switch_handling_runtime_test.mts
bun tests/pi_project_root_inference_test.mts
bun tests/pi_session_project_switch_ledger_runtime_test.mts
bun tests/current_ask_project_override_runtime_test.mts

log "Spec98 scoped active/write guard proofs"
tests/spec98_workpoint_trajectory_active_scope_static_test.py
tests/spec98_focus_stack_state_scope_static_test.py

log "Spec98 partition contract proofs"
tests/spec98_project_workstream_partition_contract_test.py
tests/spec98_work_loop_execution_partition_static_test.py
tests/spec98_unscoped_canonical_inventory_static_test.py

log "Spec98 CRDT/event-store foundation proofs"
tests/spec98_crdt_event_store_wiring_static_test.py
${CARGO:-cargo} test -p focusa-core sync::crdt

log "Build gates"
npm --prefix apps/pi-extension run check
${CARGO:-cargo} check

log "Known gaps retained (not claimed complete)"
printf '%s\n' \
  '- scoped writer-claim map migration remains pending; active_writer is legacy-global migration fallback' \
  '- SQLite CRDT schema migration and same-root sync import reconciliation remain pending' \
  '- multi-daemon same-root sync proof remains pending until CRDT production migration lands'

log "PASS: Spec98 runtime bleed + CRDT regression proof suite completed"
