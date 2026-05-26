# Focusa Trajectory GTM and Companion Gap Assessment

## Bottom line

Focusa is strong enough to be a high-value companion for power-user coding agents today, especially where compaction, handoff, evidence, and long-running project continuity matter.

It is not yet the default go-to framework for every agent type. The missing piece is not more raw memory; it is a simple, reliable, per-project Trajectory layer that every agent can consume before acting.

## North-star product frame

**Focusa is the per-project trajectory intelligence runtime for AI agents.**

It keeps any agent aligned to:

- the correct project,
- the **HLT (High-Level Trajectory)** — the ultimate project direction,
- **MLGs (Mid-Level Goals)** derived from the HLT,
- **STGs (Short-Term Goals)** derived from the current HLT/MLG context,
- **Waypoints** that mark concrete progress along MLG/STG paths,
- desired end state,
- current verified state,
- active gap,
- evidence and uncertainty,
- drift boundaries,
- and the next bounded Workpoint candidate.

Official trajectory ladder: `HLT → MLG → STG → Waypoints → Workpoint`. HLT/MLG/STG/Waypoints steer the project; Workpoints remain the canonical immediate continuation contract. Once an agent/model knows the HLT, it should proactively derive the next MLG/STG/Waypoints and move toward the HLT instead of passively waiting or reacting turn-by-turn, unless an explicit risk/approval gate blocks action. Operator authority still wins: the model should defer to the operator while actively offering HLT-aligned Waypoints, STGs, and MLGs as optional route guidance.

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
| Trajectory tools need product hardening | API/Pi/CLI surfaces now cover view, define_goal, assess, propose_workpoint, checkpoint, resume, and DOD proof contracts; UX must stay obvious. | Keep one first-call path (`trajectory view`) and add demo flows showing the follow-up lifecycle. |
| Focus Slice injection is Pi-first | Pi now injects bounded ProjectIdentity, Trajectory, ResourceMode, and TOOL_AFFORDANCES; other adapters need equivalent cards. | Port the compact trajectory/affordance card to Claude Code, OpenCode, Letta, CLI, and MCP entrypoints. |
| Cross-agent adapters uneven | Pi is best-supported; generic agents need thin, dead-simple entrypoints. | Ship adapter cards/prompts for Claude Code, OpenCode, Letta, CLI, MCP. |
| Onboarding/install still expert-oriented | Go-to frameworks win with fast setup and clear mental model. | One-command local install, sample project, quickstart, demo flow. |
| Golden eval corpus needs expansion | Spec96 golden/static/runtime/stress proofs now exist; more real-world with/without trajectory evals would sharpen GTM claims. | Add longer compaction, project mismatch, long-task, and cross-agent eval scenarios. |
| Persistent per-project trajectory lifecycle needs metrics | Reducer-backed goal/checkpoint/state-delta/DOD records now exist; lifecycle quality metrics are still early. | Add rollups for evidence-linked completion, drift reduction, and operator-assistance rate. |
| Low-memory reliability needs ongoing SLOs | LowMem route, Focus Slice, traverse, Workpoint, and surgical-agent stress proofs pass. | Keep latency/health SLO tests in release gates and monitor Beads/daemon storage bloat. |
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
- HLT is the ultimate trajectory; MLGs, STGs, and Waypoints derive downward from it and steer bounded work.
- HLT knowledge creates a proactive planning obligation: derive the next MLG/STG/Waypoint and continue unless risk, destructive action, or operator approval boundaries block movement.
- Operator deference and proactive route-offering coexist: agents should not override the operator, but should keep suggesting useful HLT-aligned Waypoints, STGs, and MLGs.
- Workpoints remain canonical immediate continuation.
- Evidence is the completion currency.
- Operator steering wins.
- Low-memory reliability beats rich-context ambition.
