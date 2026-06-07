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
prep_required = [
    'prep_packet:', 'bead_id:', 'requirement_id:', 'target_routes_or_tools:', 'target_files:',
    'current_failing_output_ref:', 'current_happy_path_output_ref:', 'regression_test_name:',
    'clean_repair_assertions:', 'implementation_owner:', 'collision_check:', 'rollback_plan:',
]
proof_required = [
    'implementation_proof:', 'bead_id:', 'requirement_id:', 'code_refs:', 'test_refs:',
    'original_failure_before:', 'fixed_failure_after:', 'restored_happy_path_after:',
    'no_residual_ux_assertions:', 'evidence_refs:', 'residual_ui_risk: none',
    'residual_authority_risk: none',
]
missing_prep = []
missing_proof = []
residual_risk = []
for item in items:
    notes = str(item.get('notes') or '')
    item_id = item.get('id')
    if allow_self and item_id == gate_id:
        continue
    if item.get('status') in {'in_progress', 'closed'}:
        fields = [field for field in prep_required if field not in notes]
        if fields:
            missing_prep.append((item, fields))
    if item.get('status') == 'closed':
        fields = [field for field in proof_required if field not in notes]
        if fields:
            missing_proof.append((item, fields))
        if 'residual_ui_risk: none' not in notes or 'residual_authority_risk: none' not in notes:
            residual_risk.append(item)
print(f'total_child_beads={len(items)}')
print(f'open_child_beads={len(open_items)}')
print(f'missing_prep_packets={len(missing_prep)}')
print(f'missing_proof_matrices={len(missing_proof)}')
print(f'residual_ui_risk_items={len(residual_risk)}')
if open_items:
    print('OPEN_ITEMS:')
    for item in open_items:
        print(f"- {item.get('id')} {item.get('status')} {item.get('title')}")
if missing_prep:
    print('MISSING_PREP_PACKET_FIELDS:')
    for item, fields in missing_prep:
        print(f"- {item.get('id')} {item.get('title')} missing={','.join(fields)}")
if missing_proof:
    print('MISSING_PROOF_MATRIX_FIELDS:')
    for item, fields in missing_proof:
        print(f"- {item.get('id')} {item.get('title')} missing={','.join(fields)}")
if residual_risk:
    print('RESIDUAL_RISK_NOT_NONE:')
    for item in residual_risk:
        print(f"- {item.get('id')} {item.get('title')}")
if open_items or missing_prep or missing_proof or residual_risk:
    sys.exit(1)
print('SPEC102 no-deferral closure gate: PASS')
PY
