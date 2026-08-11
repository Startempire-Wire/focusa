#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUTO="$ROOT/apps/pi-extension/src/auto-compaction.ts"
COMPACTION="$ROOT/apps/pi-extension/src/compaction.ts"
CONFIG="$ROOT/apps/pi-extension/src/config.ts"
COMMANDS="$ROOT/apps/pi-extension/src/commands.ts"
INDEX="$ROOT/apps/pi-extension/src/index.ts"

rg -q 'PROACTIVE_COMPACTION_RESERVE_FRACTION = 0\.1' "$AUTO"
rg -q 'PROACTIVE_COMPACTION_TRIGGER_FRACTION = 0\.7' "$AUTO"
rg -q 'PROACTIVE_COMPACTION_ABSOLUTE_TOKEN_CAP = 256_000' "$AUTO"
rg -q 'pi\.on\("agent_end"' "$AUTO"
rg -q 'pi\.on\("agent_settled"' "$AUTO"
rg -q 'pi\.on\("session_compact"' "$AUTO"
rg -q 'ctx\.getContextUsage\(\)' "$AUTO"
rg -q 'ctx\.compact\(' "$AUTO"
rg -q 'retryTimer = setTimeout' "$AUTO"
rg -U -q 'registerAutoCompaction\(\s*pi,\s*\(\) =>\s*proactiveCompactionPolicy\(getAttachmentRuntime\(\)\.cfg\),\s*\(\) =>\s*getAttachmentRuntime\(\)\.cfg\s*\)' "$INDEX"
for key in autoCompactionEnabled autoCompactionTokenCap autoCompactionReserveTokens autoCompactionReservePct autoCompactionCooldownMs; do
  rg -q "$key" "$CONFIG"
  rg -q "$key" "$COMMANDS"
done
rg -q 'prepareRuntime\(runtimeFor\(ctx, event\)\)' "$INDEX"
rg -q 'if \(!target\.cfg && bootstrap\.cfg\) target\.cfg = bootstrap\.cfg' "$INDEX"
if rg -q 'as any|as unknown as' "$AUTO"; then
  echo 'FAIL: automatic compaction uses an unsafe context cast' >&2
  exit 1
fi
rg -q 'requestCoordinatedCompaction\(ctx' "$COMPACTION"
if rg -q 'ctx\.compact\(' "$COMPACTION"; then
  echo 'FAIL: compaction.ts bypasses the process-wide compaction coordinator' >&2
  exit 1
fi

cd "$ROOT"
NODE_OPTIONS='--experimental-default-type=module' npx --yes tsx tests/spec130_auto_compaction_runtime_test.mts
printf 'PASS: Spec 130 automatic compaction static/runtime contract\n'
