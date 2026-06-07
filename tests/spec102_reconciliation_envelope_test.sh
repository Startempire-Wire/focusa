#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
PI_CARD="apps/pi-extension/src/awareness.ts"
API_CARD="crates/focusa-api/src/routes/awareness.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$PI_CARD" "$API_CARD"; do
  for term in RECONCILIATION_ENVELOPE surface_states resolution authority_for_next_action supporting_context blocked_or_stale_surfaces next_repair_tool; do
    rg -F "$term" "$file" >/dev/null || fail "$file missing reconciliation term $term"
  done
  pass "$file declares reconciliation envelope terms"
done

python3 - <<'PY'
from pathlib import Path
for path in ['apps/pi-extension/src/awareness.ts','crates/focusa-api/src/routes/awareness.rs']:
    src = Path(path).read_text()
    if 'RECONCILIATION_ENVELOPE' not in src:
        raise SystemExit(f'{path} missing envelope')
    if not any(marker in src for marker in ['reconciliationEnvelope = [', 'reconciliationEnvelope = reconciliationActive ?', 'reconciliation_envelope = if']):
        raise SystemExit(f'{path} missing conditional envelope construction')
    if not any(marker in src for marker in ['...reconciliationEnvelope', 'reconciliation_envelope.clone()']):
        raise SystemExit(f'{path} missing envelope injection')
print('✓ PASS: reconciliation envelope is conditional and injected')
PY

# Live API: mismatched continuity should show envelope; canonical active continuity should keep happy path calm.
if curl -fsS --max-time 10 'http://127.0.0.1:8787/v1/health' >/dev/null 2>&1; then
  curl -fsS --max-time 15 'http://127.0.0.1:8787/v1/awareness/card?project_root=/home/wirebot/focusa&continuity_id=definitely-missing-spec102-cont' > /tmp/spec102-reconcile-conflict.json
  jq -e '.rendered_card | contains("RECONCILIATION_ENVELOPE") and contains("next_repair_tool") and contains("authority_for_next_action")' /tmp/spec102-reconcile-conflict.json >/dev/null \
    || fail 'conflict path missing reconciliation envelope'
  pass 'conflict path renders one reconciliation envelope'
  KEY="spec102-reconcile-happy-test-$$"
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"project_root\":\"/home/wirebot/focusa\",\"continuity_id\":\"$KEY\",\"session_id\":\"$KEY\",\"mission\":\"Spec102 reconciliation happy fixture\",\"next_slice\":\"Continue calmly\",\"canonical\":true,\"idempotency_key\":\"$KEY\"}" \
    'http://127.0.0.1:8787/v1/workpoint/checkpoint' >/tmp/spec102-reconcile-happy-wp.json
  curl -fsS --max-time 15 "http://127.0.0.1:8787/v1/awareness/card?project_root=/home/wirebot/focusa&continuity_id=$KEY" >/tmp/spec102-reconcile-happy.json
  jq -e '.workpoint_canonical == true and (.rendered_card | contains("RECONCILIATION_ENVELOPE") | not)' /tmp/spec102-reconcile-happy.json >/dev/null \
    || fail 'canonical happy path should omit reconciliation envelope details'
  pass 'canonical happy path omits reconciliation envelope details'
fi

echo "SPEC102 reconciliation envelope test: PASS"
