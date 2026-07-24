#!/usr/bin/env python3
import json
from pathlib import Path
C=Path(__file__).resolve().parents[1]/'docs/contracts/spec135/generated-contract-v1'
p=json.loads((C/'spec135-alpha5-alpha6-proof.json').read_text())
assert p['status']=='passed'
for ref in p['alpha5']['proof_refs']:
    assert json.loads((C/ref).read_text())['status']=='passed'
assert p['alpha5']['receipt'].startswith('receipt:spec135-alpha5:')
assert p['alpha6']['receipt'].startswith('receipt:spec135-alpha6:')
assert len(p['alpha6']['remote_evidence_refs'])==3
print('Spec 135 Alpha 5-6 artifact refresh and portable Work Surfaces: PASS')
