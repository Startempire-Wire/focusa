#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
EPIC_ID="${SPEC102_EPIC_ID:-focusa-pm2b}"
TMP_JSON="${TMPDIR:-/tmp}/spec102-proof-matrix-children.json"
cd "$ROOT_DIR"
bd --no-daemon list --parent "$EPIC_ID" --all --json --no-pager > "$TMP_JSON"
python3 - "$TMP_JSON" <<'PY'
import json, sys
items = json.load(open(sys.argv[1]))
required_fields = [
    'implementation_proof:',
    'bead_id:',
    'requirement_id:',
    'code_refs:',
    'test_refs:',
    'original_failure_before:',
    'fixed_failure_after:',
    'restored_happy_path_after:',
    'no_residual_ux_assertions:',
    'evidence_refs:',
    'residual_ui_risk: none',
    'residual_authority_risk: none',
]
missing=[]
for item in items:
    if item.get('status') != 'closed':
        continue
    notes = item.get('notes') or ''
    missing_fields = [field for field in required_fields if field not in notes]
    if missing_fields:
        missing.append((item, missing_fields))
print(f'checked_child_beads={len(items)}')
closed=[i for i in items if i.get('status')=='closed']
print(f'closed_child_beads={len(closed)}')
print(f'missing_proof_matrices={len(missing)}')
print('missing_residual_risk_none=0' if not any('residual_ui_risk: none' in fields or 'residual_authority_risk: none' in fields for _, fields in missing) else 'missing_residual_risk_none=1')
if missing:
    print('MISSING_PROOF_MATRIX_FIELDS:')
    for item, fields in missing:
        print(f"- {item.get('id')} {item.get('title')} missing={','.join(fields)}")
    sys.exit(1)
print('SPEC102 proof matrix enforcement: PASS')
PY
