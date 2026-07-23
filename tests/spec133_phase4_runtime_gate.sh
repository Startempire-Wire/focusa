#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"

export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}

required_evidence=(
  docs/evidence/spec133-phase4-1-posix-controlled-stop-proof-2026-07-23.md
  docs/evidence/spec133-phase4-2-control-capability-proof-2026-07-23.md
  docs/evidence/spec133-phase4-3-retry-reconnect-adoption-proof-2026-07-23.md
  docs/evidence/spec133-phase4-4-reboot-recovery-proof-2026-07-23.md
  docs/evidence/spec133-phase4-5-resource-admission-proof-2026-07-23.md
  docs/evidence/spec133-phase4-6-failure-envelope-proof-2026-07-23.md
)
for evidence in "${required_evidence[@]}"; do
  test -s "$evidence" || {
    echo "missing Phase 4 evidence: $evidence" >&2
    exit 1
  }
done

cargo test -q -p focusa-session-runner process_posix
cargo test -q -p focusa-session-runner adoption
cargo test -q -p focusa-core silent_session_retry
cargo test -q -p focusa-core silent_session_recovery
cargo test -q -p focusa-core silent_session_resources
cargo test -q -p focusa-core bounded_capture_reports_backpressure_without_blocking_or_truth_loss
cargo test -q -p focusa-core silent_session_failure

printf '%s\n' 'PASS: Spec133 Phase 4 supervision, recovery, resource, backpressure, and failure matrix'
