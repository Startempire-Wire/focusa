#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOMAIN="$ROOT/crates/focusa-core/src/silent_sessions"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

for file in mod.rs identity.rs config.rs types.rs; do
  test -f "$DOMAIN/$file" || fail "missing Silent Session domain file: $file"
done
pass "domain module split is present"

for object in SilentSession SilentSessionRun SilentSessionConfigRevision SilentSessionEvent RuntimeCheckpoint SilentSessionWorkpointCheckpoint SilentSessionLease CompletionEvaluation; do
  rg -n "(struct|enum) $object" "$DOMAIN" >/dev/null || fail "missing canonical object: $object"
done
pass "all Spec133 §8 canonical objects are typed"

for version in SILENT_SESSION_SCHEMA_VERSION CONFIG_SCHEMA_VERSION EVENT_SCHEMA_VERSION DAEMON_RUNNER_PROTOCOL_VERSION HARNESS_ADAPTER_PROTOCOL_VERSION PROCESS_BACKEND_PROTOCOL_VERSION STREAM_CHUNK_FORMAT_VERSION RECEIPT_MAPPING_VERSION; do
  rg -n "$version" "$DOMAIN/types.rs" >/dev/null || fail "missing independent version: $version"
done
pass "all Spec133 §27 independent versions are declared"

rg -n 'Uuid::now_v7' "$DOMAIN/identity.rs" >/dev/null || fail "UUIDv7 generation missing"
rg -n 'project_root.*continuity_id|SilentSessionAuthority' "$DOMAIN/identity.rs" >/dev/null || fail "project+continuity authority missing"
rg -n 'RunGeneration' "$DOMAIN/identity.rs" >/dev/null || fail "stable run generation missing"
pass "UUIDv7 identity, authority boundary, and run generation are explicit"

if rg -n 'std::fs|tokio::|rusqlite|reqwest|std::process::Command' "$DOMAIN" >/dev/null; then
  fail "domain-only slice contains persistence, process, transport, or I/O behavior"
fi
pass "domain slice contains facts only"
