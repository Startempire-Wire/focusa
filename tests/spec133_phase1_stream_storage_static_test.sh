#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SS="$ROOT/crates/focusa-core/src/silent_sessions"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

for file in event_protocol.rs stream_codec.rs stream_storage.rs secure_fs.rs stream_storage_test.rs; do
  test -f "$SS/$file" || fail "missing stream protocol/storage file: $file"
done

for marker in 'from_mode(0o700)' 'mode(0o600)' no_follow_flag symlink_metadata create_new 'fs::rename' sync_all; do
  rg -n -F "$marker" "$SS/secure_fs.rs" >/dev/null || fail "missing secure filesystem marker: $marker"
done
pass "0700/0600, NOFOLLOW, symlink rejection and same-directory atomic publication are explicit"

for marker in compress_chunk decompress_chunk STREAM_CHUNK_CODEC_VERSION sha256_hex ChecksumMismatch; do
  rg -n "$marker" "$SS/stream_codec.rs" "$SS/stream_storage.rs" >/dev/null || fail "missing compression/checksum marker: $marker"
done
pass "versioned compression and corruption detection are explicit"

for marker in StreamCursor STREAM_CURSOR_VERSION CursorChecksumMismatch read_after CursorRunMismatch; do
  rg -n "$marker" "$SS/stream_codec.rs" "$SS/stream_storage.rs" >/dev/null || fail "missing cursor/reconnect marker: $marker"
done
pass "opaque restart-stable cursors and resumable reads are explicit"

for channel in Stdout Stderr StructuredHarnessEvents AssistantText ThinkingText ToolCalls ToolOutput FocusaControlEvents OperatorInput SystemDiagnostics; do
  rg -n "$channel" "$SS/event_protocol.rs" >/dev/null || fail "missing output channel: $channel"
done
for family in session.created config.resolved model.effective harness.connected agent.working stream.stdout tool.started project_identity.verified writer_lease.acquired resource.sample process.exited; do
  rg -n -F "$family" "$SS/event_protocol.rs" >/dev/null || fail "missing event family: $family"
done
pass "all §12 output channels and event families are represented"

for marker in silent_session_stream_indexes connection.transaction chunk_sequence last_event_sequence redaction_applied MIGRATION_V2_SQL; do
  rg -n "$marker" "$SS/stream_storage.rs" "$SS/persistence_sqlite.rs" >/dev/null || fail "missing durable index marker: $marker"
done
if rg -n 'std::process|tokio::process|Command::new|\bpty\b|\bprovider\b' "$SS/stream_storage.rs" "$SS/stream_codec.rs" >/dev/null; then
  fail "stream persistence slice drifted into process, PTY or provider scope"
fi
pass "transactional indexes, monotonic sequencing and redaction boundary are present without scope drift"
