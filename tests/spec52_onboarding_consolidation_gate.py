#!/usr/bin/env python3
import argparse, json
from pathlib import Path

root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(); parser.add_argument('--final',action='store_true'); args=parser.parse_args()
matrix=json.loads((root/'docs/contracts/52-focusa-onboarding-seam-matrix.json').read_text())
assert len(matrix['canonical_owners'])==9
assert len(matrix['call_stack'])==9
assert len(matrix['ranked_friction'])>=6
assert len(matrix['dead_road_decisions'])>=5
assert matrix['after_trace']['steps'] < matrix['before_trace']['steps']
assert matrix['after_trace']['repeated_prompts']==0
required=[15,47,48,49,50,53,54,59,64,65,66,87,88]
evidence=list((root/'docs/evidence/v0.9.135').glob('*.txt'))
for issue in required:
    assert any(path.name.startswith(f'{issue}-') for path in evidence), issue
assert 'requiredWriterLeaseHeaders()' not in (root/'apps/pi-extension/src/tools.ts').read_text().split('name: "focusa_workpoint_checkpoint"',1)[1].split('name: "focusa_workpoint_link_evidence"',1)[0]
assert 'sendUserMessage' not in (root/'apps/pi-extension/src/ota-activation.ts').read_text()
if args.final:
    assert (root/'apps/pi-extension/src/mission-canvas-v2.ts').is_file(), 'external #45/PR73 not integrated'
print(f"Spec52 onboarding consolidation: {'FINAL PASS' if args.final else 'PREFLIGHT PASS'}")
