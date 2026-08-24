#!/usr/bin/env python3
import json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
subprocess.run(['bun','scripts/generate-agent-tool-docs.ts','--check'],cwd=R,check=True)
result=subprocess.run(['node','scripts/audit-focusa-tool-implementation-spec-gaps.mjs','--json'],cwd=R,text=True,capture_output=True)
assert result.returncode==0, result.stdout+result.stderr
report=json.loads(result.stdout)
assert report['failures']==[]
assert report['tool_count']==report['contract_count']==146
registry=json.loads((R/'docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json').read_text())
for descriptor in registry['descriptors']:
 doc=(R/descriptor['docs_ref']).read_text()
 assert f"Result envelope: `{descriptor['result_envelope']}`" in doc
 if not descriptor['availability']['assignable']:
  assert 'capability is unavailable because its daemon router is not registered' in doc
source=(R/'scripts/audit-focusa-tool-implementation-spec-gaps.mjs').read_text()
for mapping in ["bg: 'crates/focusa-cli/src/commands/bg.rs'","silent: 'crates/focusa-cli/src/commands/silent.rs'","'agent-runtime': 'crates/focusa-cli/src/commands/agent_runtime.rs'"]:
 assert mapping in source
print('PASS: 146 generated tool docs and grounded implementation audit agree')
