#!/usr/bin/env python3
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
C=R/'docs/contracts/spec135/generated-contract-v1'
p=json.loads((C/'spec135-u4-u5-usability-friction-proof.json').read_text())
assert p['status']=='passed'
assert json.loads((C/'spec135-u3-browser-eval-matrix.json').read_text())['status']=='passed'
assert p['u5']['raw_user_content_retained'] is False
assert p['u5']['surveillance_authority'] is False
assert p['u5']['maximum_read_signals']==100
types=(R/'crates/focusa-core/src/types.rs').read_text()
route=(R/'crates/focusa-api/src/routes/uxp.rs').read_text()
executor=(R/'crates/focusa-core/src/workers/executor.rs').read_text()
for marker in ('signal_type','timestamp','session_id','weight_tier'): assert marker in types
assert 'raw_user_content' not in types
for marker in ('limit.clamp(1, 100)','raw_user_content_retained','surveillance_authority','focusa.ufi_bounded_view.v1'): assert marker in route
assert 'detect_ufi_signals' in executor
print('Spec 135 U4-U5 accessible UI and bounded friction learning: PASS')
