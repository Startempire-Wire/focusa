#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
EPIC_ID="${SPEC102_EPIC_ID:-focusa-pm2b}"
GATE_ID="${SPEC102_GATE_ID:-focusa-pm2b.43}"
ALLOW_GATE_SELF_OPEN="${SPEC102_ALLOW_GATE_SELF_OPEN:-1}"
TMP_JSON="${TMPDIR:-/tmp}/spec102-no-deferral-children.json"

cd "$ROOT_DIR"
bd --no-daemon list --parent "$EPIC_ID" --all --json --no-pager > "$TMP_JSON"

python3 - "$TMP_JSON" "$GATE_ID" "$ALLOW_GATE_SELF_OPEN" <<'PY'
import json, sys
path, gate_id, allow_self = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
items = json.load(open(path))
open_items = []
for item in items:
    status = item.get('status')
    item_id = item.get('id')
    if allow_self and item_id == gate_id and status in {'open', 'in_progress'}:
        continue
    if status != 'closed':
        open_items.append(item)
missing_proof = []
for item in items:
    notes = str(item.get('notes') or '')
    title = str(item.get('title') or '')
    item_id = item.get('id')
    if allow_self and item_id == gate_id:
        continue
    if item.get('status') == 'closed' and 'implementation_proof:' not in notes and 'golden real-life happy-path regression' not in title:
        missing_proof.append(item)
residual_risk = []
for item in items:
    notes = str(item.get('notes') or '')
    item_id = item.get('id')
    if allow_self and item_id == gate_id:
        continue
    if item.get('status') == 'closed' and 'residual_ui_risk: none' not in notes:
        residual_risk.append(item)
print(f'total_child_beads={len(items)}')
print(f'open_child_beads={len(open_items)}')
print(f'missing_proof_matrices={len(missing_proof)}')
print(f'residual_ui_risk_items={len(residual_risk)}')
if open_items:
    print('OPEN_ITEMS:')
    for item in open_items:
        print(f"- {item.get('id')} {item.get('status')} {item.get('title')}")
if missing_proof:
    print('MISSING_PROOF:')
    for item in missing_proof:
        print(f"- {item.get('id')} {item.get('title')}")
if residual_risk:
    print('RESIDUAL_UI_RISK_NOT_NONE:')
    for item in residual_risk:
        print(f"- {item.get('id')} {item.get('title')}")
if open_items or missing_proof or residual_risk:
    sys.exit(1)
print('SPEC102 no-deferral closure gate: PASS')
PY
