---
name: focusa-docs-maintenance
description: "Use when updating Focusa public docs, skills, evidence files, README links, version snapshot language, or release-proof documentation."
---

# Focusa Docs Maintenance Playbook

Use when updating Focusa public docs, skills, evidence files, README links, version snapshot language, or release-proof documentation.

## Progressive disclosure

Read `references/01-focusa-docs-maintenance-runbook.md` for the public, agent, skill, runbook, machine-contract, and all-Pi-tool coverage matrix.

## Start here

1. Load the main Focusa skill if you need the whole system model: `/skill:focusa`.
2. Read the focused tool doc: `docs/focusa-tools/README.md`.
3. Prefer canonical Focusa state over transcript memory.
4. Preserve proof as evidence refs, not pasted logs.

## Primary docs

- Focused tools: `docs/focusa-tools/README.md`
- Tool index: `docs/focusa-tools/README.md`
- Operator guide: `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`
- Live release proof: `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`

## Safety rules

- Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as recovery states, not success.
- Use Workpoint resume/checkpoint around compaction, context overflow, model switch, fork, or risky release work.
- Use writer-status/preflight before mutating work-loop state.
- Do not describe Focusa as complete or frozen; use current snapshot/version language.


## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-docs-maintenance` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: hand-authored; no automatic sibling-body injection
- supersession: none
