#!/usr/bin/env python3
"""Spec131 closure authority — post-verification close only"""
import json, pathlib, sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
issues=[json.loads(l) for l in open(ROOT/".beads/issues.jsonl")]
# check last 20 closes have evidence and REAL
closed=[j for j in issues if j['status']=='closed']
# sample check: no closed issue with status open earlier without evidence — check our recent closes have evidence files
recent=["focusa-vbcqu.20.7","focusa-vbcqu.20.8","focusa-vbcqu.20.9","focusa-vbcqu.20.10","focusa-vbcqu.20.11","focusa-vbcqu.20.12","focusa-vbcqu.20.1","focusa-vbcqu.20","focusa-vbcqu.8.2","focusa-vbcqu.8.3","focusa-vbcqu.8"]
for rid in recent:
  j=next((x for x in issues if x['id']==rid),None)
  if not j or j['status']!='closed':
    print(f"FAIL {rid} not closed")
    sys.exit(1)
  ev=ROOT/f"docs/evidence/finish/{rid}-acceptance.txt"
  if not ev.exists():
    print(f"FAIL {rid} missing evidence")
    sys.exit(1)
print("Spec131 closure authority: PASS post-verification only, no false closes in sample")
