#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}

required_evidence=(
  docs/evidence/spec133-phase5-1-writer-admission-proof-2026-07-23.md
  docs/evidence/spec133-phase5-2-workspace-strategy-proof-2026-07-23.md
  docs/evidence/spec133-phase5-3-scheduler-proof-2026-07-23.md
  docs/evidence/spec133-phase5-4-governed-integration-proof-2026-07-23.md
  docs/evidence/spec133-phase5-5-multisession-isolation-proof-2026-07-23.md
)
for evidence in "${required_evidence[@]}"; do
  test -s "$evidence" || {
    echo "missing Phase 5 evidence: $evidence" >&2
    exit 1
  }
done

cargo test -q -p focusa-core silent_session_writer
cargo test -q -p focusa-core silent_session_workspace
cargo test -q -p focusa-core silent_session_scheduler
cargo test -q -p focusa-core silent_session_integration
cargo test -q -p focusa-session-runner identity
cargo test -q -p focusa-session-runner mutation_posix

printf '%s\n' 'PASS: Spec133 Phase 5 writer, worktree, scheduler, integration, and isolation matrix'
