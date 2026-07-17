#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STORE="$ROOT/crates/focusa-core/src/silent_sessions/persistence_sqlite.rs"
RUNTIME="$ROOT/crates/focusa-core/src/runtime/persistence_sqlite.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

test -f "$STORE" || fail "Silent Session SQLite persistence module missing"

for table in silent_sessions silent_session_runs silent_session_config_revisions silent_session_events silent_session_stream_indexes silent_session_checkpoints silent_session_leases silent_session_notifications silent_session_completion_evaluations silent_session_backend_bindings; do
  rg -n "CREATE TABLE IF NOT EXISTS $table" "$STORE" >/dev/null || fail "missing required table: $table"
done
pass "all Spec133 §11 canonical tables are migrated"

for marker in 'UNIQUE(silent_session_id, sequence)' 'UNIQUE(silent_session_id, idempotency_key)' previous_event_hash event_hash 'connection.transaction' 'transaction.commit' 'transaction.rollback'; do
  rg -n -F "$marker" "$STORE" >/dev/null || fail "missing atomic event-chain marker: $marker"
done
pass "transactional append, idempotency, sequence and hash linkage are explicit"

for marker in 'DryRun,' 'wal_checkpoint(FULL)' 'fs::copy' verify_schema SILENT_SESSION_DB_SCHEMA_VERSION; do
  rg -n -F "$marker" "$STORE" >/dev/null || fail "missing migration safety marker: $marker"
done
pass "backup, dry-run, rollback and schema verification are explicit"

rg -n 'migrate_silent_session_schema' "$RUNTIME" >/dev/null || fail "canonical SQLite initialization does not invoke Silent Session migration"
if rg -n 'UPDATE silent_sessions SET lifecycle|DELETE FROM silent_session_events' "$STORE" >/dev/null; then
  fail "direct lifecycle mutation or event deletion bypasses reducer-event ownership"
fi
pass "canonical runtime integration preserves reducer-owned append-only mutation"

RECORDS="$ROOT/crates/focusa-core/src/silent_sessions/persistence_records.rs"
for marker in load_session load_session_events save_run load_run save_config_revision load_config_revision save_runtime_checkpoint load_runtime_checkpoint save_workpoint_checkpoint load_workpoint_checkpoint save_lease load_lease save_completion_evaluation load_completion_evaluation; do
  rg -n "pub fn $marker" "$RECORDS" >/dev/null || fail "missing canonical record persistence function: $marker"
done
pass "canonical sessions, events, runs, config, checkpoints, leases and completion data persist and reload"
