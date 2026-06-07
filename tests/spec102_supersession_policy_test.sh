#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
EPIC_ID="${SPEC102_EPIC_ID:-focusa-pm2b}"
TMP_JSON="${TMPDIR:-/tmp}/spec102-supersession-children.json"
cd "$ROOT_DIR"
bd --no-daemon list --parent "$EPIC_ID" --all --json --no-pager > "$TMP_JSON"
python3 - "$TMP_JSON" <<'PY'
import json, re, sys
items=json.load(open(sys.argv[1]))
quiet_gap=[]
bad_supersession=[]
required=['requirement_id:', 'bead_id:', 'reason:', 'operator_approval_ref:', 'replacement_requirement:', 'updated_spec_ref:', 'affected_tests:']
for item in items:
    notes=item.get('notes') or ''
    title=item.get('title') or ''
    status=item.get('status')
    if item.get('id') == 'focusa-pm2b.47':
        continue
    text=f"{title}\n{notes}".lower()
    has_sup='supersession:' in notes
    gap_words=any(w in text for w in ['not implemented', 'known gap', 'manual workaround', 'todo', 'later'])
    if status=='closed' and gap_words and not has_sup:
        quiet_gap.append(item)
    if has_sup:
        missing=[k for k in required if k not in notes]
        if missing:
            bad_supersession.append((item, missing))
print(f'checked_child_beads={len(items)}')
print(f'closed_quiet_gap_items={len(quiet_gap)}')
print(f'incomplete_supersession_records={len(bad_supersession)}')
if quiet_gap:
    print('QUIET_GAPS:')
    for i in quiet_gap: print(f"- {i.get('id')} {i.get('title')}")
if bad_supersession:
    print('BAD_SUPERSESSIONS:')
    for i, missing in bad_supersession: print(f"- {i.get('id')} missing={','.join(missing)} {i.get('title')}")
if quiet_gap or bad_supersession:
    sys.exit(1)
print('SPEC102 supersession policy: PASS')
PY
