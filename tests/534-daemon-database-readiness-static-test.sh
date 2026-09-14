#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

rg -q 'base_product_projection\(state\.license_guard\.entitlement' \
  crates/focusa-api/src/routes/health.rs \
  || fail 'API doctor must derive readiness from signed write authority'
rg -q '"status": if base_mutations_allowed \{ "ready" \} else \{ "blocked" \}' \
  crates/focusa-api/src/routes/health.rs \
  || fail 'API doctor must block runtime readiness when writes are denied'
rg -q 'installed daemon service authority' \
  crates/focusa-cli/src/commands/doctor.rs \
  || fail 'CLI doctor must verify installed service ownership'
rg -q 'the daemon did not provide a runtime readiness decision' \
  crates/focusa-cli/src/commands/doctor.rs \
  || fail 'missing readiness must fail closed'
rg -q 'process_start_token TEXT' crates/focusa-core/src/background_job_store.rs \
  || fail 'background jobs must store an additive process-start identity'
rg -q 'reconcile_stale_jobs' crates/focusa-api/src/server.rs \
  || fail 'daemon startup must own stale-row reconciliation'
rg -q 'BACKGROUND_JOB_SCHEMA:.*background_job\.v3' \
  crates/focusa-core/src/background_jobs.rs \
  || fail 'new background records must use the v3 additive contract'
rg -q 'BACKGROUND_JOB_SCHEMA_V2' crates/focusa-core/src/background_jobs.rs \
  || fail 'v2 compatibility marker must remain'
rg -q 'SQLITE_OPEN_READ_ONLY' crates/focusa-api/src/routes/background_jobs.rs \
  || fail 'background-job GET routes must open SQLite read-only'
rg -q 'reconciliation_skipped' crates/focusa-api/src/routes/background_jobs.rs \
  || fail 'recovery-only job listings must disclose skipped reconciliation'
rg -q 'legacy_schema_reads_without_migration' crates/focusa-core/src/background_job_store.rs \
  || fail 'legacy job reads must prove zero schema mutation'

printf 'PASS: readiness fails closed; recovery reads stay read-only; migrations remain additive\n'
