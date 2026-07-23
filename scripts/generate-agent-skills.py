#!/usr/bin/env python3
"""Generate Spec141 progressive-disclosure skills, runbooks, and coverage proof."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "config/agent-skills-v2.json"
ROOT_SKILLS = ROOT / ".pi/skills"
PACKAGED_SKILLS = ROOT / "apps/pi-extension/skills"
EVIDENCE_JSON = ROOT / "docs/evidence/141-focusa-skill-runbook-coverage.json"
EVIDENCE_MD = ROOT / "docs/evidence/141-focusa-skill-runbook-coverage.md"


def skill_body(skill: dict) -> str:
    tools = skill["tools"]
    return f'''---
name: {skill["name"]}
description: "{skill["description"]}"
---

# {skill["name"].replace("-", " ").title()}

{skill["description"]}

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-{skill["name"]}-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

{chr(10).join(f"- {item}" for item in skill["triggers"])}

## Non-trigger examples

{chr(10).join(f"- {item}" for item in skill["non_triggers"])}

## Required sequence

{chr(10).join(f"{index}. `{tool}`" for index, tool in enumerate(tools, 1))}

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

{chr(10).join(f"- `{tool}`" for tool in skill["recovery"])}

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

{skill["done"]}

Stable evidence or receipt refs must support any completion claim.
'''


def runbook_body(skill: dict) -> str:
    tools = skill["tools"]
    return f'''# {skill["name"].replace("-", " ").title()} Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
{chr(10).join(f"{tools[index]} -> {tools[index + 1]}" for index in range(len(tools) - 1))}
```

## Minimal path

{chr(10).join(f"{index}. Call `{tool}` with only required bounded inputs." for index, tool in enumerate(tools, 1))}

## Branches

- Unknown tool/schema: `focusa_tool_search` → `focusa_tool_describe`.
- Scope conflict: `focusa_project_verify` → `focusa_workpoint_checkpoint`.
- Daemon/degraded state: `focusa_tool_doctor`; retry only with safe posture.
- Resource timeout: `focusa_resource_mode` → bounded `focusa_traverse`.
- Browser failure: UIAI diagnostics → `focusa_browser_diagnostics_intake` → evidence.
- Mutation ambiguity: inspect side effects/receipts before retry; require operator confirmation when declared.

## Evidence and closure

- Capture stable file/test/API/browser/receipt refs.
- Link proof to the active Workpoint.
- Evaluate relevant predictions and reusable learning only after outcome is known.
- Done: {skill["done"]}

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.
'''


def write_or_check(path: Path, body: str, check: bool) -> bool:
    current = path.read_text() if path.exists() else None
    drift = current != body
    if drift and not check:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
    return drift


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    registry = json.loads(REGISTRY.read_text())
    drift = []
    generated = []
    for skill in registry["skills"]:
        body = skill_body(skill)
        runbook = runbook_body(skill)
        for base in (ROOT_SKILLS, PACKAGED_SKILLS):
            skill_path = base / skill["name"] / "SKILL.md"
            runbook_path = base / skill["name"] / "references" / f"01-{skill['name']}-runbook.md"
            if write_or_check(skill_path, body, args.check):
                drift.append(str(skill_path.relative_to(ROOT)))
            if write_or_check(runbook_path, runbook, args.check):
                drift.append(str(runbook_path.relative_to(ROOT)))
        generated.append({
            "name": skill["name"],
            "tools": skill["tools"],
            "runbook": f".pi/skills/{skill['name']}/references/01-{skill['name']}-runbook.md",
            "sha256": hashlib.sha256(body.encode()).hexdigest(),
        })

    root_names = sorted(path.parent.name for path in ROOT_SKILLS.glob("*/SKILL.md"))
    packaged_names = sorted(path.parent.name for path in PACKAGED_SKILLS.glob("*/SKILL.md"))
    parity_drift = []
    for name in sorted(set(root_names) | set(packaged_names)):
        root_path = ROOT_SKILLS / name / "SKILL.md"
        package_path = PACKAGED_SKILLS / name / "SKILL.md"
        if not root_path.exists() or not package_path.exists() or root_path.read_bytes() != package_path.read_bytes():
            parity_drift.append(name)
    evidence = {
        "schema": "focusa.agent_skill_runbook_coverage.v1",
        "registry_version": registry["version"],
        "generated_skill_count": len(generated),
        "installed_root_skill_count": len(root_names),
        "packaged_skill_count": len(packaged_names),
        "root_packaged_parity": not parity_drift,
        "parity_drift": parity_drift,
        "skills": generated,
    }
    evidence_body = json.dumps(evidence, indent=2) + "\n"
    md = [
        "# Spec141 Focusa Skill and Runbook Coverage",
        "",
        f"- Generated domain skills: `{len(generated)}`",
        f"- Installed root skills: `{len(root_names)}`",
        f"- Packaged skills: `{len(packaged_names)}`",
        f"- Root/package parity: `{not parity_drift}`",
        "",
        "## Generated coverage",
        "",
    ]
    md.extend(f"- `{item['name']}` → `{item['runbook']}` → {len(item['tools'])} declared tools" for item in generated)
    md.extend(["", "## Parity drift", "", *(f"- `{name}`" for name in parity_drift or ["none"])])
    md_body = "\n".join(md).strip() + "\n"
    if write_or_check(EVIDENCE_JSON, evidence_body, args.check):
        drift.append(str(EVIDENCE_JSON.relative_to(ROOT)))
    if write_or_check(EVIDENCE_MD, md_body, args.check):
        drift.append(str(EVIDENCE_MD.relative_to(ROOT)))
    if args.check and drift:
        print(json.dumps({"status": "failed", "drift": drift}))
        return 1
    print(json.dumps({"status": "passed", "mode": "check" if args.check else "write", "skills": len(generated), "root_packaged_parity": not parity_drift}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
