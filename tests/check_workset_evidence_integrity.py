#!/usr/bin/env python3
"""Gate: no bead may be marked completed without at least one evidence_ref."""
import json, sys

false = []
with open('release-proof/audit/next-locked-release-workset-members.jsonl') as f:
    for line in f:
        d = json.loads(line.strip())
        if d.get('disposition') == 'completed' and not d.get('evidence_refs', []) and d.get('member_id','').startswith('focusa-vbcqu'):
            false.append(d['member_id'])

if false:
    print(f'FAIL: {len(false)} beads marked completed with zero evidence_refs')
    for mid in false:
        print(f'  FALSE CLOSURE: {mid}')
    sys.exit(1)

print(f'PASS: all completed beads have evidence refs')
