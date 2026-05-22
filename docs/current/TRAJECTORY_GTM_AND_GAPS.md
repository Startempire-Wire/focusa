# Focusa Trajectory GTM and Companion Gap Assessment

## Bottom line

Focusa is strong enough to be a high-value companion for power-user coding agents today, especially where compaction, handoff, evidence, and long-running project continuity matter.

It is not yet the default go-to framework for every agent type. The missing piece is not more raw memory; it is a simple, reliable, per-project Trajectory layer that every agent can consume before acting.

## North-star product frame

**Focusa is the per-project trajectory intelligence runtime for AI agents.**

It keeps any agent aligned to:

- the correct project,
- the real long-term goal,
- desired end state,
- current verified state,
- active gap,
- evidence and uncertainty,
- drift boundaries,
- and the next bounded Workpoint candidate.

Focusa should complement agents, not replace them. Agents do the work; Focusa keeps them oriented, continuous, evidence-grounded, and recoverable.

## Current strengths

- Durable Focus State beyond transcript memory.
- Workpoint continuation packets for compaction/resume/handoff.
- Evidence handles and bounded proof discipline.
- Tool contracts and live parity proof.
- Work-loop ownership/writer safety.
- Lineage/snapshot recovery primitives.
- Metacognition and prediction primitives.
- Low-memory reliability principle now encoded and audited.
- First live per-project Trajectory API and `focusa_trajectory_view` tool slice.

## Gaps before “go-to” status

| Gap | Why it matters | Needed next |
|---|---|---|
| Trajectory tools incomplete | Agents need one obvious first call and follow-up lifecycle tools. | Finish `define_goal`, `assess`, `propose_workpoint`, `checkpoint`, `resume`. |
| Focus Slice injection not trajectory-first | Models should not have to remember to call trajectory manually. | Pi Focus Slice now injects bounded ProjectIdentity + Trajectory summary per project; remaining work is cross-agent adapter parity and golden eval proof. |
| Cross-agent adapters uneven | Pi is best-supported; generic agents need thin, dead-simple entrypoints. | Ship adapter cards/prompts for Claude Code, OpenCode, Letta, CLI, MCP. |
| Onboarding/install still expert-oriented | Go-to frameworks win with fast setup and clear mental model. | One-command local install, sample project, quickstart, demo flow. |
| Golden evals missing | Need proof Focusa reduces drift and improves completion across agents. | Add with/without trajectory evals for compaction, project mismatch, long tasks. |
| Persistent per-project trajectory lifecycle incomplete | Current view composes state; durable goal supersession needs reducer-backed metadata. | Add reducer events and storage for accepted trajectory checkpoints. |
| Low-memory mode still early | Companion must be reliable under constrained dev environments. | Enforce caps/caches/store budgets and SLO tests. |
| Product message needs sharp wedge | “Cognitive runtime” is accurate but abstract. | Lead with per-project trajectory intelligence and agent continuity. |

## GTM message

### One-liner

Focusa keeps AI agents from losing the plot.

### Positioning sentence

Focusa is the per-project trajectory intelligence runtime that gives every AI agent the correct project, goal, verified state, evidence, drift boundaries, and next bounded move.

### Mission

Make AI agents reliable collaborators on long-running project work by giving them durable, project-scoped orientation and evidence-backed continuity across turns, tools, models, sessions, and handoffs.

### Pillars

1. **Orientation:** every agent knows which project it is in and what the project is trying to become.
2. **Continuity:** the next move survives compaction, restart, model switch, and handoff.
3. **Evidence:** progress is tied to proof, not vibes.
4. **Drift control:** stale/cross-project/context-mismatched signals are suppressed.
5. **Portability:** Focusa wraps existing agents instead of replacing them.
6. **Reliability:** low memory keeps the core trajectory alive; high memory enriches without hogging.

### Initial wedge

Developers using multiple coding agents on long-running repos.

Pain: agents forget context, confuse projects, lose goals after compaction, repeat work, and need constant steering.

Promise: install Focusa locally and every agent gets the same project trajectory, evidence, and next Workpoint before it acts.

## Product doctrine

- Per-project first; global memory only supports project-scoped trajectory.
- Trajectory is advisory orientation, not task authority.
- Workpoints remain canonical immediate continuation.
- Evidence is the completion currency.
- Operator steering wins.
- Low-memory reliability beats rich-context ambition.
