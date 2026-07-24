#!/usr/bin/env python3
"""SPEC135-U3 aggregate UIAI browser-eval closure proof."""
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
C=R/'docs/contracts/spec135/generated-contract-v1'
m=json.loads((C/'spec135-u3-browser-eval-matrix.json').read_text())
assert m['status']=='passed' and m['driver']=='UIAI Engine Eval'
assert len(m['generated_ui_eval_results'])>=13
for ref in m['generated_ui_eval_results']:
    result=json.loads((C/ref).read_text())
    assert result['status']=='passed', ref
for proof in m['remote_browser_and_recovery_proofs']:
    assert (R/proof['test']).is_file()
    assert proof['evidence_ref'].endswith(':passed')
assert any('no Playwright' in invariant for invariant in m['invariants'])
assert not (R/'playwright.config.ts').exists()
print('Spec 135 U3 UIAI browser evaluation matrix: PASS')
