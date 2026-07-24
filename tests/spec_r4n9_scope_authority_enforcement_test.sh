#!/usr/bin/env bash
set -euo pipefail
STATE="apps/pi-extension/src/state.ts"

rg -n 'durable_project_write_authority' "$STATE" >/dev/null
rg -n 'action_authority_for_current_ask: true' "$STATE" >/dev/null
rg -n 'continue diagnosis; verify project scope before durable writes' "$STATE" >/dev/null
rg -n 'conversation=continue; durable_writes=' "$STATE" >/dev/null
if rg -n 'BLOCKED: scope conflict|EXECUTION BLOCKED|⛔' "$STATE" >/dev/null; then
  echo "FAIL: model-flow blocking wording remains" >&2
  exit 1
fi
( cd apps/pi-extension && npx tsc --noEmit )
echo "PASS: scope conflict preserves steering and gates only durable writes"
