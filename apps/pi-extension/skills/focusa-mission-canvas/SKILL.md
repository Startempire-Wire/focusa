---
name: focusa-mission-canvas
description: "Use for Workstream-aware Mission Canvas, Work Rail, CRIST generated UI, Desktop presentation, workspace artifacts, live refresh, and provider-neutral operation binding."
---

# Focusa Mission Canvas

## P0 transition stop line

Before Mission Canvas implementation, read:

1. `docs/agent/00-p0-transition-bootstrap.md`
2. `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
3. `docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
4. `docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`
5. `docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`

The complete rich Mission Canvas is no longer primarily owned by Pi TUI. Focusa Desktop is the primary rich application. Pi retains Work Rail, tools, exact Attachment binding, embedded/standalone terminal operation and a bounded compatibility Canvas.

Do not discard existing Mission Canvas work. Preserve it, correct it to exact Scope + Workstream identity, extract reusable semantic logic and keep replacement/parity proof before cleanup.

## Canonical identity

Mission Canvas operations require exact `ScopeRef + WorkstreamId` or an exact Attachment/Object reference that resolves uniquely to them.

- Workstream is durable cognition.
- Continuity is lineage inside a Workstream.
- Thread is legacy terminology.
- Session/Instance are runtime metadata.
- Work Surface is presentation identity only.
- UI focus, CWD, latest project and daemon-global active/current state are not authority.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-mission-canvas-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to load exact schemas only for selected tools.
4. Open linked specs/Evidence only when required.

## Trigger examples

- Mission Canvas
- Focusa Desktop
- Work Rail
- C.R.I.S.T.
- generated UI
- Work Surface
- workspace artifact
- Desktop presentation

## Non-trigger examples

- hand-coded parallel UI contract
- invented operation binding
- coordinate-click Desktop automation
- new rich Pi-only pane
- continuity-only canonical binding

## Required sequence

1. verify the transition preservation/checkpoint state when working from old Mission Canvas branches;
2. resolve exact Workstream and Attachment authority;
3. `focusa_call_stack_design`;
4. `focusa_context_cognition`;
5. `focusa_evidence_capture`;
6. `focusa_active_object_resolve`.

Operator steering, exact Workstream authority and canonical Workpoint remain higher priority than this default sequence.

## Desktop development rule

For the active MacBook transition worktree:

- commit locally; do not push directly to main/shared Mission Canvas branches;
- use one pinned Rust toolchain;
- keep the real SvelteKit app continuously previewable in browser;
- use UIAI Engine for browser proof;
- build/open the full Tauri shell at 5/25/50/75/100 percent;
- initiate the canonical release from the approved KnownHost host at 75 percent after operator approval;
- never commit private release-host details.

## Failure recovery

- `focusa_call_stack_verify`
- `focusa_tool_doctor`
- `focusa_workpoint_resume`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, ambiguous Workstream resolution, validation rejection and ambiguous side effects as recovery states—not completion.

## Routing metadata

- prerequisites: exact ScopeRef + WorkstreamId and typed Attachment where runtime scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; Workstream reducer and typed Workpoint/Trajectory contracts are canonical
- workflow: `focusa-project-scope` → `focusa-mission-canvas` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: hand-authored transition-aware core; packaged copy must remain byte-identical
- supersession: FOCUSA-TRANSITION-001 and Spec 158 govern conflicting older Pi-primary wording

## Done condition

Mission Canvas surfaces bind exact Workstream-owned canonical operations and durable Evidence without semantic drift. GUI, CLI and agent paths share stable identifiers and typed Results/Receipts.

Stable Evidence or Receipt refs support every completion claim.
