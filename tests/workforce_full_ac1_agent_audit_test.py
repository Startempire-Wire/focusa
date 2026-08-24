#!/usr/bin/env python3
import json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
result=subprocess.run(['python3','scripts/audit-agent-first-tool-surfaces.py','--strict'],cwd=R,text=True,capture_output=True)
assert result.returncode==0, result.stdout+result.stderr
report=json.loads(result.stdout)
assert report['findings']==[]
metrics=report['metrics']
assert metrics['route_classifier_passed'] is True
assert metrics['api_route_paths']==metrics['classified_api_route_paths']==617
assert metrics['declared_api_route_paths']>metrics['api_route_paths']
assert metrics['mcp_exposed_tools']==metrics['mcp_expected_callable_tools']
source=(R/'scripts/audit-agent-first-tool-surfaces.py').read_text()
for marker in ['assignable_descriptors','unavailable_descriptors','invalid_unavailable_descriptors','route_classifier']:
 assert marker in source
assert 'mcp_names != expected_mcp_names' in source
print('PASS: strict agent audit accepts only reachable, assignable capability surfaces')
