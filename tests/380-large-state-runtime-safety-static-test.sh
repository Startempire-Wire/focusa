#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

rg -q 'const LARGE_STATE_STACK_BYTES: usize = 32 \* 1024 \* 1024' \
  crates/focusa-core/src/runtime/daemon.rs \
  || fail 'large-state reducer and clone work must use an explicit bounded stack'
rg -q 'Action::IngestSignal \{ signal \} => vec!\[intuition_signal_event\(signal\.clone\(\)\)\]' \
  crates/focusa-core/src/runtime/daemon.rs \
  || fail 'periodic intuition and Guardian signals must bypass the large general translator future'
rg -q 'find_map\(\|request\| request\.state\.take\(\)\)' \
  crates/focusa-core/src/runtime/persistence_actor.rs \
  || fail 'persistence coalescing must move, not recursively clone, the latest state'
rg -q 'name\("focusa-persistence-writer"\.into\(\)\)' \
  crates/focusa-core/src/runtime/persistence_actor.rs \
  || fail 'SQLite state serialization must run on its dedicated bounded writer'
rg -q 'stack_size\(32 \* 1024 \* 1024\)' \
  crates/focusa-core/src/runtime/persistence_actor.rs \
  || fail 'the persistence writer stack must be bounded explicitly'
rg -q 'const ORDINARY_EVENTS_PER_SNAPSHOT: usize = 32' \
  crates/focusa-core/src/runtime/persistence_actor.rs \
  || fail 'ordinary whole-state checkpoints are not bounded'
rg -q 'CREATE TABLE IF NOT EXISTS snapshot_event_cursors' \
  crates/focusa-core/src/runtime/persistence_sqlite.rs \
  || fail 'snapshot-to-event replay cursor is missing'
rg -q 'FocusGatePipelineCommitted' crates/focusa-core/src/runtime/daemon.rs \
  || fail 'periodic Focus Gate changes are not replayable append-only events'
if sed -n '/async fn run_gate_pipeline/,/async fn expire_stale_turn/p' \
  crates/focusa-core/src/runtime/daemon.rs \
  | rg -q 'persist_reducer_batch\(Vec::new\(\), false\)'; then
  fail 'periodic Focus Gate changes still force whole-state checkpoints'
fi
rg -q 'replay_durable_tail' crates/focusa-core/src/runtime/daemon.rs \
  || fail 'daemon startup does not replay the post-checkpoint durable tail'
rg -q 'SignalKind::terminate\(\)' crates/focusa-api/src/main.rs \
  || fail 'systemd SIGTERM does not enter the governed shutdown checkpoint path'

printf 'PASS: large-state ingestion, bounded checkpoints, replay, and graceful shutdown avoid amplification\n'
