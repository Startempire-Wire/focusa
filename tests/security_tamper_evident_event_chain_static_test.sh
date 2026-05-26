#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PERSIST="$ROOT_DIR/crates/focusa-core/src/runtime/persistence_sqlite.rs"
TEST="$ROOT_DIR/crates/focusa-core/src/runtime/persistence_sqlite_test.rs"
DOC="$ROOT_DIR/docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md"
[[ -f "$DOC" ]] || { echo "missing tamper-evident event chain doc" >&2; exit 1; }

for marker in \
  "CREATE TABLE IF NOT EXISTS event_hash_chain" \
  "payload_sha256" \
  "previous_hash" \
  "event_chain_hash" \
  "latest_event_hash_checkpoint" \
  "GENESIS"; do
  if ! grep -Fq "$marker" "$PERSIST"; then
    echo "persistence missing tamper-evident marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "sqlite_event_hash_chain_links_appended_events" \
  "SELECT chain_index, previous_hash, event_hash" \
  "assert_eq!(rows[1].1, rows[0].2)"; do
  if ! grep -Fq "$marker" "$TEST"; then
    echo "persistence test missing hash-chain marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "STRIDE repudiation" \
  "event_hash_chain" \
  "verification route/CLI"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "tamper-evident doc missing marker: $marker" >&2
    exit 1
  fi
done

echo "✓ tamper-evident event chain static markers present"
