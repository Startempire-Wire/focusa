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
  'database_bytes' \
  'wal_bytes' \
  'persist_checkpoint' \
  'append_events_checkpoint'; do
  grep -F "$marker" "$ACTOR" >/dev/null || fail "persistence actor missing: $marker"
done

grep -F 'attach_persistence_actor' "$DAEMON" >/dev/null || fail "daemon does not share persistence actor"
grep -F 'persist_reducer_batch' "$DAEMON" >/dev/null || fail "daemon reductions bypass persistence actor"
grep -F 'persistence_actor: Some(persistence_actor)' "$SERVER" >/dev/null || fail "API does not share persistence actor"
grep -F 'actor.metrics()' "$HEALTH" >/dev/null || fail "health omits persistence pressure metrics"
grep -F '.unwrap_or(1_000)' "$ROOT/crates/focusa-core/src/runtime/persistence_sqlite.rs" >/dev/null \
  || fail "hot CLT projection remains large enough to stall reducer state clones"
grep -F 'DEFAULT_HOT_HANDLE_LIMIT: usize = 2_048' "$ROOT/crates/focusa-core/src/reference/mod.rs" >/dev/null \
  || fail "ECS handle projection lacks a strict hot-state bound"
grep -F 'snapshot_payload_bytes()?' "$ACTOR" >/dev/null \
  || fail "snapshot_bytes still aliases allocated SQLite database bytes"
python3 - "$SERVER" <<'PY'
import pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text()
bind = text.index('tokio::net::TcpListener::bind(&bind_addr).await?')
rehydrate = text.index('tokio::spawn(async move {', bind)
call = text.index('rehydrate_pairing_state_from_ledger(&pairing_rehydrate_state).await', rehydrate)
assert bind < rehydrate < call, "API readiness must precede asynchronous pairing-ledger rehydration"
PY

if rg -n 'state\.persistence\.(save_state|append_event)' "$ROOT/crates/focusa-api/src/routes" --glob '!**/*test*'; then
  fail "API route performs synchronous SQLite persistence"
fi
if rg -n 'self\.persistence\.(save_state|append_event)' "$DAEMON"; then
  fail "daemon hot path performs synchronous SQLite persistence"
fi

echo "PASS: Spec130A bounded persistence actor, coalescing, telemetry, and hot-path guards"
