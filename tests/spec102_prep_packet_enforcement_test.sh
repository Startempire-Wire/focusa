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
missing = []
for item in items:
    status = item.get('status')
    title = item.get('title') or ''
    notes = item.get('notes') or ''
    if status in {'in_progress', 'closed'} and 'prep_packet:' not in notes:
        # The parent-generated docs/spec preservation bead may be completed by audit proof later, but once in_progress it still needs prep.
        missing.append(item)
print(f'checked_child_beads={len(items)}')
print(f'in_progress_or_closed={sum(1 for i in items if i.get("status") in {"in_progress", "closed"})}')
print(f'missing_prep_packets={len(missing)}')
if missing:
    print('MISSING_PREP_PACKETS:')
    for item in missing:
        print(f"- {item.get('id')} {item.get('status')} {item.get('title')}")
    sys.exit(1)
print('SPEC102 prep packet enforcement: PASS')
PY
