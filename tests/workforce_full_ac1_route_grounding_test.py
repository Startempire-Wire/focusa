#!/usr/bin/env python3
import json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
subprocess.run(['python3','scripts/generate-agent-route-classification.py','--check'],cwd=R,check=True)
report=json.loads((R/'docs/contracts/spec141/generated-capability-v2/route-classification.json').read_text())
live={item['path'] for item in report['routes']}; unreachable={item['path'] for item in report['unreachable_declared_routes']}
assert '/v1/silent-sessions/{session_id}/approvals' in live
for path in ['/v1/worksets','/v1/callgraphs','/v1/silent-sessions/fanout']:
 assert path not in live, f'dead route grounded as live: {path}'
 assert path in unreachable, f'dead declaration not reported: {path}'
source=(R/'scripts/generate-agent-route-classification.py').read_text()
for marker in ['reachable_modules','reachable_sources','unreachable_declarations','server_source']:
 assert marker in source
assert report['route_count']==len(live)
assert len(unreachable)>=3
print(f'PASS: {len(live)} reachable routes grounded; {len(unreachable)} unreachable declarations quarantined')
