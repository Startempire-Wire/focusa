#!/usr/bin/env python3
import json,re,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
for command in [
 ['python3','scripts/generate-agent-route-classification.py','--check'],
 ['bun','scripts/generate-agent-capability-descriptors.ts','--check'],
 ['bun','scripts/generate-agent-tool-docs.ts','--check'],
 ['node','scripts/validate-focusa-tool-contracts.mjs'],
]: subprocess.run(command,cwd=R,check=True)
routes=json.loads((R/'docs/contracts/spec141/generated-capability-v2/route-classification.json').read_text())
live={row['path'] for row in routes['routes']}
registry=json.loads((R/'docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json').read_text())
assert registry['capability_count']==146
assert registry['assignable_capability_count']==140
assert registry['unavailable_capability_count']==6
unavailable={row['tool_names']['pi'] for row in registry['descriptors'] if not row['availability']['assignable']}
expected_unavailable={'focusa_workset_projection','focusa_callgraph_validate','focusa_callgraph_observe','focusa_credentials_verify','focusa_cockpit_projection','focusa_fast_forward'}
assert unavailable==expected_unavailable, (unavailable,expected_unavailable)
for descriptor in registry['descriptors']:
 for route in descriptor['tool_names']['rest']:
  assert route['path'].split('?')[0] in live, (descriptor['tool_names']['pi'],route)
 for permission in descriptor['permissions']:
  path=permission['route'].split(' ',1)[1].split('?')[0]
  assert path in live, (descriptor['tool_names']['pi'],path)
pi_names={row['name'] for row in json.loads((R/'docs/contracts/spec141/generated-capability-v2/pi-tools.json').read_text())['tools']}
assert not (pi_names & unavailable)
assert len(pi_names)==140
rest=json.loads((R/'docs/contracts/spec141/generated-capability-v2/rest-agent-operations.json').read_text())['operations']
assert rest and all(row['path'].split('?')[0] in live for row in rest)
assert any(row['path']=='/v1/silent-sessions/{session_id}/approvals' for row in rest)
contracts=(R/'apps/pi-extension/src/tool-contracts.ts').read_text()
assert not re.search(r'api_routes:\s*\[\s*"/',contracts), 'bare methodless API route remains'
tools=(R/'apps/pi-extension/src/tools.ts').read_text()
for name in expected_unavailable:
 block=tools.split(f'name: "{name}"',1)[1].split('pi.registerTool',1)[0]
 assert 'capability is unavailable because its daemon router is not registered' in block, name
for name in ['focusa_bg_run','focusa_bg_run_many','focusa_bg_status']:
 descriptor=next(row for row in registry['descriptors'] if row['tool_names']['pi']==name)
 assert descriptor['availability']['assignable'] is True
 assert descriptor['tool_names']['rest']==[]
 assert descriptor['tool_names']['cli'], name
print('PASS: 140 grounded assignable capabilities; 6 dead-router tools fail closed and are unassignable')
