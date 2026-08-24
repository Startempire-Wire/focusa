#!/usr/bin/env python3
from pathlib import Path
import json
R=Path(__file__).resolve().parents[1]
s=(R/'crates/focusa-api/src/routes/silent_sessions_config_read.rs').read_text()
registry=json.loads((R/'docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json').read_text())
assignable={d['tool_names']['pi'] for d in registry['descriptors'] if d['availability']['assignable']}
roles=['planner','researcher','builder','reviewer','operator']
for role in roles:
 assert f'"role_id": "{role}"' in s
 assert f'"preset_id": "{role}"' in s
assert s.count('"approval_required": true') >= 5
assert '"grants_permissions": false' in s
for capability in ['focusa_trajectory_assess','focusa_context_cognition','focusa_silent_sessions','focusa_call_stack_verify','focusa_bg_status']:
 assert capability in assignable
assert 'generated_assignable_capability_registry' in s
assert 'workforce-role-profiles-v1' in s and 'workforce-role-presets-v1' in s
print('PASS: five grounded role profiles and five approval-bound presets')
