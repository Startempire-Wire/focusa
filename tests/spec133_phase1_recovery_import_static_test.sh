#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SS="$ROOT/crates/focusa-core/src/silent_sessions"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }; pass(){ echo "✓ PASS: $*"; }
for marker in MigrationMode::DryRun recover_registered_run IndexRebuilt Quarantined degraded manifest.chunk_hash recovery/quarantine; do
  rg -n "$marker" "$SS/stream_recovery.rs" >/dev/null || fail "missing recovery contract: $marker"
done
pass "migration, registered-root audit, hash verification, rebuild, quarantine and degraded recovery are explicit"
for marker in expected_uid allowed_log_roots legacy_unverified aliases source_registry_hash LegacyImportMap SilentSessionId::new SilentSessionRunId::new; do
  rg -n "$marker" "$SS/legacy_import.rs" >/dev/null || fail "missing legacy-import contract: $marker"
done
pass "legacy ownership/path safety, stable UUID mapping, aliases and unverified posture are explicit"
rg -n 'must-not-exist|!command_marker.exists' "$SS/legacy_import_test.rs" >/dev/null || fail "missing no-command-execution proof"
pass "legacy command non-execution has a regression proof"
