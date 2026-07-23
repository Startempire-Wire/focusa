#!/usr/bin/env bash
# Spec130A / GitHub #12: SQLite persistence must stay off Tokio core workers.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ACTOR="$ROOT/crates/focusa-core/src/runtime/persistence_actor.rs"
DAEMON="$ROOT/crates/focusa-core/src/runtime/daemon.rs"
SERVER="$ROOT/crates/focusa-api/src/server.rs"
HEALTH="$ROOT/crates/focusa-api/src/routes/health.rs"

fail() { echo "FAIL: $*" >&2; exit 1; }

for marker in \
  'mpsc::channel::<PersistenceRequest>(PERSISTENCE_QUEUE_CAPACITY)' \
  'tokio::task::spawn_blocking' \
  'requests_coalesced_total' \
  'queue_depth_max' \
  'last_write_duration_ms' \
  'snapshot_bytes' \
  'wal_bytes' \
  'persist_checkpoint' \
  'append_events_checkpoint'; do
  grep -F "$marker" "$ACTOR" >/dev/null || fail "persistence actor missing: $marker"
done

grep -F 'attach_persistence_actor' "$DAEMON" >/dev/null || fail "daemon does not share persistence actor"
grep -F 'persist_reducer_batch' "$DAEMON" >/dev/null || fail "daemon reductions bypass persistence actor"
grep -F 'persistence_actor: Some(persistence_actor)' "$SERVER" >/dev/null || fail "API does not share persistence actor"
grep -F 'actor.metrics()' "$HEALTH" >/dev/null || fail "health omits persistence pressure metrics"

if rg -n 'state\.persistence\.(save_state|append_event)' "$ROOT/crates/focusa-api/src/routes" --glob '!**/*test*'; then
  fail "API route performs synchronous SQLite persistence"
fi
if rg -n 'self\.persistence\.(save_state|append_event)' "$DAEMON"; then
  fail "daemon hot path performs synchronous SQLite persistence"
fi

echo "PASS: Spec130A bounded persistence actor, coalescing, telemetry, and hot-path guards"
