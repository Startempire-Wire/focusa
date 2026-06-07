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
missing_proof=[]
missing_residual=[]
missing_refs=[]
for item in items:
    if item.get('status') != 'closed':
        continue
    notes = item.get('notes') or ''
    title = item.get('title') or ''
    if 'implementation_proof:' not in notes:
        missing_proof.append(item)
    if 'residual_ui_risk: none' not in notes or 'residual_authority_risk: none' not in notes:
        missing_residual.append(item)
    if not any(k in notes for k in ('test_refs:', 'evidence_refs:', 'code_refs:')):
        missing_refs.append(item)
print(f'checked_child_beads={len(items)}')
closed=[i for i in items if i.get('status')=='closed']
print(f'closed_child_beads={len(closed)}')
print(f'missing_proof_matrices={len(missing_proof)}')
print(f'missing_residual_risk_none={len(missing_residual)}')
print(f'missing_test_or_evidence_refs={len(missing_refs)}')
if missing_proof:
    print('MISSING_PROOF:')
    for i in missing_proof: print(f"- {i.get('id')} {i.get('title')}")
if missing_residual:
    print('MISSING_RESIDUAL_NONE:')
    for i in missing_residual: print(f"- {i.get('id')} {i.get('title')}")
if missing_refs:
    print('MISSING_REFS:')
    for i in missing_refs: print(f"- {i.get('id')} {i.get('title')}")
if missing_proof or missing_residual or missing_refs:
    sys.exit(1)
print('SPEC102 proof matrix enforcement: PASS')
PY
