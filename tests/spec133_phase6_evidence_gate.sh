#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}

required_evidence=(
  docs/evidence/spec133-phase6-1-focusa-authority-integration-2026-07-23.md
  docs/evidence/spec133-phase6-2-context-authority-ontology-proof-2026-07-23.md
  docs/evidence/spec133-phase6-3-checkpoint-cadence-proof-2026-07-23.md
  docs/evidence/spec133-phase6-4-completion-evidence-proof-2026-07-23.md
  docs/evidence/spec133-phase6-5-receipts-closure-proof-2026-07-23.md
  docs/evidence/spec133-phase6-6-transfer-prediction-metacog-proof-2026-07-23.md
)
for evidence in "${required_evidence[@]}"; do
  test -s "$evidence" || {
    echo "missing Phase 6 evidence: $evidence" >&2
    exit 1
  }
done

cargo test -q -p focusa-core silent_session_authority
cargo test -q -p focusa-core silent_session_authorization
cargo test -q -p focusa-core silent_session_bootstrap
cargo test -q -p focusa-core silent_session_checkpoint_policy
cargo test -q -p focusa-core silent_session_completion
cargo test -q -p focusa-core silent_session_receipts
cargo test -q -p focusa-core silent_session_continuation
cargo test -q -p focusa-core silent_session_reconstruction
cargo test -q -p focusa-session-runner mutation_posix
cargo test -q -p focusa-api silent_sessions

printf '%s\n' 'PASS: Spec133 Phase 6 authority, checkpoint, evidence, receipt, transfer, learning, and reconstruction matrix'
