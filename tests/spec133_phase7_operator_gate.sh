#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}
export FOCUSA_PROJECT_ROOT=${FOCUSA_PROJECT_ROOT:-$ROOT}
export FOCUSA_CONTINUITY_ID=${FOCUSA_CONTINUITY_ID:-spec133-phase7-operator-gate}

required_evidence=(
  docs/evidence/spec133-phase7-1-dashboard-proof-2026-07-23.md
  docs/evidence/spec133-phase7-2-live-views-controls-proof-2026-07-23.md
  docs/evidence/spec133-phase7-3-notification-proof-2026-07-23.md
  docs/evidence/spec133-phase7-4-wizard-context-proof-2026-07-23.md
  docs/evidence/spec133-phase7-5-pi-menubar-projection-proof-2026-07-23.md
)
for evidence in "${required_evidence[@]}"; do
  test -s "$evidence" || {
    echo "missing Phase 7 evidence: $evidence" >&2
    exit 1
  }
done

cargo test -q -p focusa-core silent_session_notifications
cargo test -q -p focusa-core silent_session_wizard
cargo test -q -p focusa-api silent_sessions
cargo test -q -p focusa-cli commands::silent
pnpm --dir apps/menubar check
pnpm --dir apps/menubar test
npm --prefix apps/pi-extension run typecheck
npm --prefix apps/pi-extension run test:menu-audit
npm --prefix apps/pi-extension run test:interaction-mode

printf '%s\n' 'PASS: Spec133 Phase 7 dashboard, views, controls, notifications, wizard, Pi, menubar, and bounded-context matrix'
