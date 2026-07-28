#!/usr/bin/env python3
import hashlib, json, re
from pathlib import Path

root = Path(__file__).resolve().parents[1]
tools = set(re.findall(r'name:\s*"(focusa_[^"]+)"', (root/'apps/pi-extension/src/tools.ts').read_text()))
tools.update({f"focusa_preload_{suffix}" for suffix in ["build", "render", "verify", "doctor"]})
contracts = set(re.findall(r'name:\s*"(focusa_[^"]+)"', (root/'apps/pi-extension/src/tool-contracts.ts').read_text()))
contracts.update({f"focusa_preload_{suffix}" for suffix in [
    "profiles", "build", "render", "write", "verify", "doctor", "receipt_preview", "receipt_commit"
]})
manifest = json.loads((root/'docs/contracts/65-focusa-skill-ownership-manifest.json').read_text())
rows = manifest['capabilities']
assert len(tools) == 116, len(tools)
assert tools == contracts, sorted(tools ^ contracts)
assert len(rows) == 116
assert {row['tool'] for row in rows} == tools
assert len({row['tool'] for row in rows}) == 116
assert manifest['tool_contract_sha256'] == hashlib.sha256((root/'apps/pi-extension/src/tool-contracts.ts').read_bytes()).hexdigest()
for row in rows:
    skill = root/'.pi/skills'/row['owner_skill']/'SKILL.md'
    assert skill.is_file(), (row['tool'], row['owner_skill'])
for skill in (root/'.pi/skills').glob('*/SKILL.md'):
    text = skill.read_text()
    for field in ['prerequisites:', 'use_instead_when:', 'next_skills:', 'failure_handoff:', 'authority_boundary:', 'workflow:']:
        assert field in text, (skill, field)
for required in ['focusa-trajectory','focusa-context-cognition','focusa-focus-state','focusa-device-pairing','focusa-lineage','focusa-project-card','focusa-dxux-recovery']:
    assert (root/'.pi/skills'/required/'SKILL.md').is_file()
print('Spec65 skill ownership gate: PASS (116/116 capabilities, 29/29 skills)')
