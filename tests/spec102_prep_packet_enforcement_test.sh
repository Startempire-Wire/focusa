#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
EPIC_ID="${SPEC102_EPIC_ID:-focusa-pm2b}"
TMP_JSON="${TMPDIR:-/tmp}/spec102-prep-packet-children.json"
cd "$ROOT_DIR"
bd --no-daemon list --parent "$EPIC_ID" --all --json --no-pager > "$TMP_JSON"
python3 - "$TMP_JSON" <<'PY'
import json, sys
items = json.load(open(sys.argv[1]))
required_fields = [
    'prep_packet:',
    'bead_id:',
    'requirement_id:',
    'target_routes_or_tools:',
    'target_files:',
    'current_failing_output_ref:',
    'current_happy_path_output_ref:',
    'regression_test_name:',
    'clean_repair_assertions:',
    'implementation_owner:',
    'collision_check:',
    'rollback_plan:',
]
missing = []
for item in items:
    status = item.get('status')
    notes = item.get('notes') or ''
    if status in {'in_progress', 'closed'}:
        missing_fields = [field for field in required_fields if field not in notes]
        if missing_fields:
            missing.append((item, missing_fields))
print(f'checked_child_beads={len(items)}')
print(f'in_progress_or_closed={sum(1 for i in items if i.get("status") in {"in_progress", "closed"})}')
print(f'missing_prep_packets={len(missing)}')
if missing:
    print('MISSING_PREP_PACKET_FIELDS:')
    for item, fields in missing:
        print(f"- {item.get('id')} {item.get('status')} {item.get('title')} missing={','.join(fields)}")
    sys.exit(1)
print('SPEC102 prep packet enforcement: PASS')
PY
