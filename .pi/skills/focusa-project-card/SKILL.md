---
name: focusa-project-card
description: Use for project bootstrap sequencing, advisory prediction, outcomes, and inferred Workpoints.
---
# focusa-project-card

## Trigger
Use when the request directly concerns project bootstrap sequencing, advisory prediction, outcomes, and inferred Workpoints.

## Non-trigger
Use the narrower owning skill when the request is primarily release, browser, resource, security, or implementation work.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: route to `focusa-project-scope` for ambiguous roots; `focusa-troubleshooting` for daemon failures
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; Workpoint/Trajectory/daemon contracts retain canonical authority
- workflow: `focusa-project-scope` → `focusa-project-card` → `focusa-workpoint` → `focusa-evidence-outcomes`

## Compatibility
- minimum contract: `focusa.tool_affordance_catalog.v1`
- source: hand-authored focused routing skill
- supersession: none

## Rules
1. Search/select the narrowest Focusa tool.
2. Read before mutation; require typed scope and confirmation where contracted.
3. Return bounded evidence and executable recovery, never transcript blobs.
