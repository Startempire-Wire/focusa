#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUTO="$ROOT/apps/pi-extension/src/auto-compaction.ts"
INDEX="$ROOT/apps/pi-extension/src/index.ts"

rg -q 'PROACTIVE_COMPACTION_RESERVE_FRACTION = 0\.1' "$AUTO"
rg -q 'pi\.on\("agent_end"' "$AUTO"
rg -q 'pi\.on\("agent_settled"' "$AUTO"
rg -q 'pi\.on\("session_compact"' "$AUTO"
rg -q 'ctx\.getContextUsage\(\)' "$AUTO"
rg -q 'ctx\.compact\(' "$AUTO"
rg -q 'evaluationTimer = setTimeout' "$AUTO"
rg -q 'registerAutoCompaction\(pi\)' "$INDEX"
if rg -q 'as any|as unknown as' "$AUTO"; then
  echo 'FAIL: automatic compaction uses an unsafe context cast' >&2
  exit 1
fi

cd "$ROOT"
npx --yes tsx tests/spec130_auto_compaction_runtime_test.mts
printf 'PASS: Spec 130 automatic compaction static/runtime contract\n'
