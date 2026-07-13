#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE="$ROOT/apps/pi-extension/src/state.ts"
TOOLS="$ROOT/apps/pi-extension/src/tools.ts"

python3 - "$STATE" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
start=s.index("export async function ensurePiFrame(")
end=s.index("export async function rescopePiFrameFromCurrentAsk", start)
body=s[start:end]
recover=body.index("scopedWorkpointFrameRecoveryCwd()")
block=body.index('event_type: "pi_frame_creation_blocked_unconfirmed_project_root"')
if recover >= block:
    raise SystemExit("canonical Workpoint recovery still occurs after broad-cwd rejection")
PY

rg -q 'resolveFocusWriteProjectRoot\(process\.cwd\(\), cachedCwd \|\| liveCwd\)' "$TOOLS"
rg -q '"workpoint_checkpoint_tool"' "$TOOLS"
rg -q '"workpoint_resume_tool"' "$TOOLS"
rg -q 'source: "canonical_session_workpoint_recovery"' "$STATE"

cd "$ROOT"
npx --yes tsx tests/focusa_decide_scope_recovery_runtime_test.mts
printf 'PASS: focusa_decide scope recovery static/runtime contract\n'
