#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CONTRACTS_JSON="${ROOT_DIR}/docs/current/focusa-tool-contracts.json"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_silent_sessions.md"
SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"

if rg -n 'name: "focusa_silent_sessions"|Daemon-native Spec133 Silent Session client' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: focusa_silent_sessions is registered as the daemon-native facade"
else
  echo "✗ FAIL: daemon-native SilentSession registration missing" >&2
  exit 1
fi

for action in list start reopen tail health send interrupt restart kill preflight watch pause resume config receipt capabilities; do
  rg -n "Type\.Literal\(\"${action}\"\)" "$TOOLS_TS" >/dev/null || {
    echo "✗ FAIL: missing SilentSession action: ${action}" >&2
    exit 1
  }
done
echo "✓ PASS: observation, lifecycle, control, config, receipt, and capability actions are exposed"

for marker in 'session_id' 'run_id' 'generation' 'approval_id' 'idempotency_key' 'daemon remains canonical authority' 'failure_class=process_control_failed'; do
  rg -n -F "$marker" "$TOOLS_TS" "$DOC" >/dev/null || {
    echo "✗ FAIL: missing daemon authority/process marker: ${marker}" >&2
    exit 1
  }
done
if rg -n 'exec\.Command|Command::new|tmux (attach|capture-pane|send-keys|kill-session|new-session)' "$TOOLS_TS" >/dev/null; then
  echo "✗ FAIL: Pi facade contains a legacy shell/tmux execution path" >&2
  exit 1
fi
echo "✓ PASS: exact identity, durable approval, idempotency, and daemon-only process control are enforced"

node - <<'NODE' "$CONTRACTS_JSON"
const fs = require('fs');
const registry = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const contract = registry.contracts.find((item) => item.name === 'focusa_silent_sessions');
if (!contract) throw new Error('missing focusa_silent_sessions contract');
if (contract.family !== 'work_loop') throw new Error(`unexpected family ${contract.family}`);
if (contract.parity_status !== 'full') throw new Error(`unexpected parity ${contract.parity_status}`);
if (!contract.cli_commands.includes('focusa silent')) throw new Error('daemon-native CLI parity missing');
if (!contract.api_routes.some((route) => route.includes('/v1/silent-sessions'))) throw new Error('daemon route parity missing');
if (!contract.doc_path.endsWith('focusa_silent_sessions.md')) throw new Error('doc path missing');
NODE
echo "✓ PASS: contract registry exposes full CLI/REST/Pi parity"

if rg -n 'approved=true|approval_id|idempotency|LowMem|interrupt|restart|process_control_failed' "$DOC" "$SKILL" >/dev/null; then
  echo "✓ PASS: operator documentation covers approval, recovery, and LowMem posture"
else
  echo "✗ FAIL: SilentSession documentation lacks daemon-native guardrails" >&2
  exit 1
fi

if rg -n 'silentSessionBlocked|tool_result_v1|recovery_hint|misuse_hint|process_control_failed' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: failures expose typed why/recovery/misuse guidance"
else
  echo "✗ FAIL: failures can still return opaque blocked/not_found responses" >&2
  exit 1
fi

echo "SPEC96 daemon-native SilentSession tool static test: PASS"
