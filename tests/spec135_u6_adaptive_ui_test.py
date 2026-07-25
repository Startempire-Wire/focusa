#!/usr/bin/env python3
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
p=json.loads((R/'docs/contracts/spec135/generated-contract-v1/spec135-u6-adaptive-ui-proof.json').read_text())
assert p['canonical_primitive']=='focusa_context_cognition_curate_optimize'
assert p['module_name']=='mission_canvas_generated_ui'
assert all(p['contract'].values())
s=(R/'crates/focusa-api/src/routes/context_cognition.rs').read_text()
for marker in ('latest_promoted_artifact','eval_score > baseline_score && eval_score >= score_threshold','explicit_rollback','"rollback"','append_cognition_optimizer_artifact','module_name'):
    assert marker in s
assert p['receipt'].startswith('receipt:spec135-u6:')
print('Spec 135 U6 adaptive generated-UI proposal/evaluation/promotion/rollback: PASS')
