#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE="$ROOT/apps/pi-extension/src/state.ts"
PERSISTENCE="$ROOT/apps/pi-extension/src/persistence.ts"
SESSION="$ROOT/apps/pi-extension/src/session.ts"

rg -q 'focusa\.compaction_persistence_anchor\.v1' "$STATE" "$PERSISTENCE"
rg -q 'NATIVE_ANCHOR_MAX_BYTES = 8 \* 1024' "$STATE" "$PERSISTENCE"
rg -q 'PROJECT_SWITCH_ANCHOR_MAX_BYTES = 2 \* 1024' "$STATE" "$PERSISTENCE"
rg -q 'semanticPersistenceDigest' "$STATE" "$PERSISTENCE"
rg -q 'created_at|createdAt|timestamp' "$STATE" "$PERSISTENCE"
rg -q 'fsyncSync\(descriptor\)' "$STATE" "$PERSISTENCE"
rg -q 'renameSync\(temporary, target\)' "$STATE" "$PERSISTENCE"
rg -q 'PERSISTENCE_SIDECAR_GENERATIONS = 3' "$STATE" "$PERSISTENCE"
rg -q 'loadPersistedRecoveryState' "$STATE" "$SESSION"
rg -q 'pendingPersistAnchor' "$STATE" "$PERSISTENCE"
rg -q 'persistProjectSwitchLedgerAnchor' "$STATE" "$PERSISTENCE"

if rg -q 'appendEntry\("focusa-state", payload\)' "$STATE"; then
  echo 'legacy full-state native append remains' >&2
  exit 1
fi
if rg -q 'appendEntry\("focusa-wbm-state", payload\)' "$STATE"; then
  echo 'legacy full-state WBM native append remains' >&2
  exit 1
fi

cd "$ROOT"
npx --yes tsx tests/spec130_bounded_persistence_runtime_test.mts
printf 'PASS: Spec 130 bounded persistence static/runtime contract\n'
