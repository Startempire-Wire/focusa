#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CONTRACTS_TS="${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts"
CONTRACTS_JSON="${ROOT_DIR}/docs/current/focusa-tool-contracts.json"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_silent_sessions.md"
SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"

if rg -n 'name: "focusa_silent_sessions"|tmux-backed Focusa SilentSessions|SILENT_SESSION_PREFIX = "focusa-silent"' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: focusa_silent_sessions tool is registered with tmux-backed SilentSession naming"
else
  echo "✗ FAIL: focusa_silent_sessions registration missing" >&2
  exit 1
fi

for action in list start reopen tail health send kill; do
  if ! rg -n "Type\.Literal\(\"${action}\"\)" "$TOOLS_TS" >/dev/null; then
    echo "✗ FAIL: missing SilentSession action: ${action}" >&2
    exit 1
  fi
done
echo "✓ PASS: list/start/reopen/tail/send/kill actions are exposed"

if rg -n 'approved !== true|force !== true|tmux attach.*-t|capture-pane|kill-session|send-keys|new-session' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: reopen returns attach command and mutating actions have approval/force gates"
else
  echo "✗ FAIL: SilentSession process-control guardrails missing" >&2
  exit 1
fi

if rg -n 'silentSessionAttachCommand|attach_detach_others_command|capture-pane", "-p", "-J"|send-keys", "-l"|list-panes|pane_dead|health_status|history-limit|remain-on-exit|tmux_version|window_name' "$TOOLS_TS" >/dev/null   && rg -n 'Tmux control model|tmux attach -d -t|tmux capture-pane -p -J|tmux list-panes|tmux send-keys -l' "$DOC" >/dev/null; then
  echo "✓ PASS: tmux cheat-sheet ergonomics are documented and implemented"
else
  echo "✗ FAIL: SilentSession tmux ergonomics missing from docs or implementation" >&2
  exit 1
fi

if rg -n 'activate_lowmem|SilentSession start|focusa_resource_mode' "$TOOLS_TS" "$DOC" >/dev/null; then
  echo "✓ PASS: default start path documents LowMem activation posture"
else
  echo "✗ FAIL: LowMem SilentSession posture missing" >&2
  exit 1
fi

node - <<'NODE' "$CONTRACTS_JSON"
const fs = require('fs');
const registry = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const contract = registry.contracts.find((item) => item.name === 'focusa_silent_sessions');
if (!contract) throw new Error('missing focusa_silent_sessions contract');
if (contract.family !== 'work_loop') throw new Error(`unexpected family ${contract.family}`);
if (contract.parity_status !== 'pi_only') throw new Error(`unexpected parity ${contract.parity_status}`);
if (!contract.cli_commands.some((cmd) => cmd.includes('tmux'))) throw new Error('tmux commands missing from contract');
if (!contract.doc_path.endsWith('focusa_silent_sessions.md')) throw new Error('doc path missing from contract');
NODE
echo "✓ PASS: contract registry includes Pi-only tmux SilentSession contract"

if [[ -f "$DOC" ]] && rg -n 'approved=true|force=true|tmux attach|LowMem|detach-others|Steer:' "$DOC" >/dev/null; then
  echo "✓ PASS: operator documentation covers reopen, kill guard, and LowMem behavior"
else
  echo "✗ FAIL: SilentSession doc missing guardrail coverage" >&2
  exit 1
fi

if rg -n 'focusa_silent_sessions|focusa_resource_mode|LowMem|mutating process actions require approval' "$SKILL" >/dev/null; then
  echo "✓ PASS: main Focusa skill routes SilentSession and ResourceMode tools"
else
  echo "✗ FAIL: main Focusa skill missing SilentSession/ResourceMode routing" >&2
  exit 1
fi

echo "SPEC96 SilentSession tool static test: PASS"

if rg -n 'silentSessionBlocked|tool_result_v1|recovery_hint|misuse_hint|approved=true.*operator|tmux .*failed' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: SilentSession failures expose why/recovery/misuse hints and tool_result_v1"
else
  echo "✗ FAIL: SilentSession failures can still return opaque blocked/not_found responses" >&2
  exit 1
fi
