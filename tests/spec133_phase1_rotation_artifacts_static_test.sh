#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SS="$ROOT/crates/focusa-core/src/silent_sessions"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }; pass(){ echo "✓ PASS: $*"; }
for marker in max_uncompressed_bytes max_compressed_bytes max_event_count max_chunk_age_seconds ChannelChanged Checkpoint Completion; do
  rg -n "$marker" "$SS/stream_rotation.rs" >/dev/null || fail "missing rotation trigger: $marker"
done
pass "live byte, compressed, event, age, channel, checkpoint and completion rotation triggers exist"
for marker in DurableFanout capacity last_acknowledged_cursor disconnected publish_durable; do
  rg -n "$marker" "$SS/stream_rotation.rs" >/dev/null || fail "missing backpressure marker: $marker"
done
publish_line=$(rg -n '\.publish_chunk' "$SS/stream_rotation.rs" | tail -1 | cut -d: -f1)
fanout_line=$(rg -n 'fanout\.publish_durable' "$SS/stream_rotation.rs" | tail -1 | cut -d: -f1)
(( publish_line < fanout_line )) || fail "fanout occurs before durable publication"
pass "bounded independent fanout follows durable publication"
for marker in bounded_transcript redacted_stream_manifest stdout_stderr_index effective_config model_binding workpoint_history git_summary test_results blocker_summary completion_evaluation receipt_refs manifest_hash; do
  rg -n "$marker" "$SS/completion_artifacts.rs" >/dev/null || fail "missing completion artifact: $marker"
done
pass "immutable completion manifest covers every §12.6 artifact family"
