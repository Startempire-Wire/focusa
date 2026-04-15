# 79 Continuous Work Loop — Full BD Decomposition

Date: 2026-04-13  
Spec: `docs/79-focusa-governed-continuous-work-loop.md`  
Root epic: `focusa-3d7d` — **Implement Focusa-governed continuous work loop from spec 79**

This document records the full epic → BD → grandchild BD decomposition created in the project `.beads/` workspace.

Authority model for this decomposition:
- operator/spec author
- authoritative spec
- `bd` decomposition
- code composition
- app functionality

`bd` NEVER outranks spec.

---

## Root Epic

- `focusa-3d7d` — Implement Focusa-governed continuous work loop from spec 79

---

## Epic 1 — Authority, policy, and canonical loop substrate

- `focusa-3d7d.1` — Authority, policy, and canonical loop substrate
  - `focusa-3d7d.1.1` — Define work-loop core types and policy structs
  - `focusa-3d7d.1.2` — Add run identity and checkpoint identity types
  - `focusa-3d7d.1.3` — Add canonical continuous-work event family
  - `focusa-3d7d.1.4` — Encode authority/trust/delegation guard types
  - `focusa-3d7d.1.5` — Implement WorkLoopPolicy defaults and presets
    - `focusa-3d7d.1.5.1` — Define highest-value kernel preset values
    - `focusa-3d7d.1.5.2` — Define safe default pause/governance/destructive flags
    - `focusa-3d7d.1.5.3` — Define budget hierarchy fields and semantics
  - `focusa-3d7d.1.6` — Implement canonical loop event schema
    - `focusa-3d7d.1.6.1` — Add loop lifecycle events to core types
    - `focusa-3d7d.1.6.2` — Map loop events to existing TurnStarted/TurnCompleted surfaces
    - `focusa-3d7d.1.6.3` — Define event payloads for blocker, pause, resume, tranche completion
  - `focusa-3d7d.1.7` — Implement authority and delegation policy schema
    - `focusa-3d7d.1.7.1` — Encode operator/spec-author supremacy rules
    - `focusa-3d7d.1.7.2` — Encode no-LLM-decision-authority runtime checks
    - `focusa-3d7d.1.7.3` — Encode delegated authorship trust-level gating

---

## Epic 2 — Daemon supervisor kernel and BD work-graph execution

- `focusa-3d7d.2` — Daemon supervisor kernel and BD work-graph execution
  - `focusa-3d7d.2.1` — Add daemon work-loop state machine
  - `focusa-3d7d.2.2` — Implement BD ready-work discovery and selection
  - `focusa-3d7d.2.3` — Implement task/tranche/project advancement logic
  - `focusa-3d7d.2.4` — Implement blocked-task defer and alternate-ready-work traversal
  - `focusa-3d7d.2.5` — Implement supervisor lifecycle transitions
    - `focusa-3d7d.2.5.1` — Add Idle/Selecting/Preparing/Awaiting/Evaluating states
    - `focusa-3d7d.2.5.2` — Add pause/resume/stop/completed transition guards
    - `focusa-3d7d.2.5.3` — Add project-blocked vs task-blocked distinction
  - `focusa-3d7d.2.6` — Implement BD traversal substrate
    - `focusa-3d7d.2.6.1` — Read dependency-satisfied ready items from BD
    - `focusa-3d7d.2.6.2` — Claim/move selected item to in-progress state
    - `focusa-3d7d.2.6.3` — Advance automatically to next ready item without reprompt
  - `focusa-3d7d.2.7` — Implement spec-linked task packet assembly
    - `focusa-3d7d.2.7.1` — Resolve linked spec refs for selected BD item
    - `focusa-3d7d.2.7.2` — Assemble acceptance/verification/scope packet
    - `focusa-3d7d.2.7.3` — Reject execution when packet lacks authoritative spec grounding
  - `focusa-3d7d.2.8` — Implement blocker-aware project progression
    - `focusa-3d7d.2.8.1` — Retry self-recovery on blocked task when policy allows
    - `focusa-3d7d.2.8.2` — Defer blocked task and continue sibling ready work
    - `focusa-3d7d.2.8.3` — Escalate only when no valid ready work remains

---

## Epic 3 — Pi transport kernel and API control surface

- `focusa-3d7d.3` — Pi transport kernel and API control surface
  - `focusa-3d7d.3.1` — Implement Pi RPC/SDK session adapter
  - `focusa-3d7d.3.2` — Implement loop control API routes
  - `focusa-3d7d.3.3` — Implement status/checkpoint payloads
  - `focusa-3d7d.3.4` — Enforce API/daemon boundary invariants
  - `focusa-3d7d.3.5` — Implement transport session control
    - `focusa-3d7d.3.5.1` — Spawn/attach Pi RPC or SDK session
    - `focusa-3d7d.3.5.2` — Send prompts and forward abort requests
    - `focusa-3d7d.3.5.3` — Report session health, crash, timeout, and retry state
  - `focusa-3d7d.3.6` — Implement event stream ingestion
    - `focusa-3d7d.3.6.1` — Consume agent_start/turn_start/message_update events
    - `focusa-3d7d.3.6.2` — Consume turn_end/agent_end for continuation decisions
    - `focusa-3d7d.3.6.3` — Preserve ordered event delivery to daemon supervisor
  - `focusa-3d7d.3.7` — Implement work-loop API endpoints
    - `focusa-3d7d.3.7.1` — Add enable/pause/resume/stop endpoints
    - `focusa-3d7d.3.7.2` — Add status/checkpoint query endpoints
    - `focusa-3d7d.3.7.3` — Prevent API layer from mutating canonical state directly

---

## Epic 4 — Verification, completion, and anti-drift enforcement

- `focusa-3d7d.4` — Verification, completion, and anti-drift enforcement
  - `focusa-3d7d.4.1` — Implement verification-before-close enforcement
  - `focusa-3d7d.4.2` — Implement task-class verification matrix
  - `focusa-3d7d.4.3` — Implement anti-drift and replan triggers
  - `focusa-3d7d.4.4` — Implement completion semantics for task/tranche/project
  - `focusa-3d7d.4.5` — Implement verification orchestration by task class
    - `focusa-3d7d.4.5.1` — Run code-task verification tier
    - `focusa-3d7d.4.5.2` — Run doc/spec-task verification tier
    - `focusa-3d7d.4.5.3` — Run architecture/integration verification tier
  - `focusa-3d7d.4.6` — Implement spec-conformance close checks
    - `focusa-3d7d.4.6.1` — Block close when implementation deviates from spec
    - `focusa-3d7d.4.6.2` — Require acceptance criteria satisfaction before close
    - `focusa-3d7d.4.6.3` — Require BD transition recording after successful verification
  - `focusa-3d7d.4.7` — Implement anti-drift control paths
    - `focusa-3d7d.4.7.1` — Detect stale bead/spec mismatch
    - `focusa-3d7d.4.7.2` — Trigger replan when operator/spec-author amends spec
    - `focusa-3d7d.4.7.3` — Reject self-invented completion targets

---

## Epic 5 — Recovery, checkpoints, observability, and status

- `focusa-3d7d.5` — Recovery, checkpoints, observability, and status
  - `focusa-3d7d.5.1` — Implement checkpoint creation and persistence
  - `focusa-3d7d.5.2` — Implement resume-from-checkpoint behavior
  - `focusa-3d7d.5.3` — Implement visible work-loop status surfaces
  - `focusa-3d7d.5.4` — Implement blocker package quality contract
  - `focusa-3d7d.5.5` — Implement run identity model
    - `focusa-3d7d.5.5.1` — Assign project_run/tranche_run/task_run identities
    - `focusa-3d7d.5.5.2` — Associate worker session and checkpoint identities
    - `focusa-3d7d.5.5.3` — Expose identities through status and audit surfaces
  - `focusa-3d7d.5.6` — Implement checkpoint triggers and resume semantics
    - `focusa-3d7d.5.6.1` — Checkpoint on enable/verification/blocker/pause/switch
    - `focusa-3d7d.5.6.2` — Restore last safe re-entry prompt basis on resume
    - `focusa-3d7d.5.6.3` — Restore mission/constraints/verified deltas on resume
  - `focusa-3d7d.5.7` — Implement operator-visible status model
    - `focusa-3d7d.5.7.1` — Expose current work item and tranche/project status
    - `focusa-3d7d.5.7.2` — Expose last blocker, last checkpoint, and continuation reason
    - `focusa-3d7d.5.7.3` — Expose budget remaining and transport health
  - `focusa-3d7d.5.8` — Implement high-quality blocker packages
    - `focusa-3d7d.5.8.1` — Include spec requirement and recovery attempts in blocker package
    - `focusa-3d7d.5.8.2` — Include alternate-ready-work availability in blocker package
    - `focusa-3d7d.5.8.3` — Include exact operator decision and next action in blocker package

---

## Epic 6 — Safety, git/worktree discipline, and governance boundaries

- `focusa-3d7d.6` — Safety, git/worktree discipline, and governance boundaries
  - `focusa-3d7d.6.1` — Implement destructive/governance approval boundaries
  - `focusa-3d7d.6.2` — Implement git/worktree safety checks
  - `focusa-3d7d.6.3` — Implement no-second-writer / no-parallel-authority guards
  - `focusa-3d7d.6.4` — Implement operator interruption supersedence
  - `focusa-3d7d.6.5` — Implement git/worktree discipline
    - `focusa-3d7d.6.5.1` — Inspect git status/diff before meaningful work units
    - `focusa-3d7d.6.5.2` — Preserve unrelated worktree changes by default
    - `focusa-3d7d.6.5.3` — Forbid hard reset/clean/restore without approval
  - `focusa-3d7d.6.6` — Implement governance and approval gates
    - `focusa-3d7d.6.6.1` — Pause on destructive confirmation requirements
    - `focusa-3d7d.6.6.2` — Pause on governance-sensitive proposal decisions
    - `focusa-3d7d.6.6.3` — Pause on explicit operator/spec-author override
  - `focusa-3d7d.6.7` — Implement no-parallel-authority enforcement
    - `focusa-3d7d.6.7.1` — Prevent API from becoming second policy engine
    - `focusa-3d7d.6.7.2` — Prevent extension from becoming second cognitive authority
    - `focusa-3d7d.6.7.3` — Prevent LLM from self-granting decision authority

---

## Epic 7 — Practical quality improvements and deferred sophistication

- `focusa-3d7d.7` — Practical quality improvements and deferred sophistication
  - `focusa-3d7d.7.1` — Implement degraded-mode behavior
  - `focusa-3d7d.7.2` — Implement fidelity-first worker fallback/routing
  - `focusa-3d7d.7.3` — Implement richer trust/delegation automation
  - `focusa-3d7d.7.4` — Implement richer UX/status surfaces after kernel validation
  - `focusa-3d7d.7.5` — Implement degraded-mode execution paths
    - `focusa-3d7d.7.5.1` — Narrow scope under degraded conditions
    - `focusa-3d7d.7.5.2` — Increase checkpoint/verification cadence under degradation
    - `focusa-3d7d.7.5.3` — Continue safe ready work while deferring risky degraded work
  - `focusa-3d7d.7.6` — Implement advanced worker selection
    - `focusa-3d7d.7.6.1` — Select better-fit worker after repeated task-class failure
    - `focusa-3d7d.7.6.2` — Preserve spec fidelity across worker/model swaps
    - `focusa-3d7d.7.6.3` — Constrain routing heuristics to kernel authority rules
  - `focusa-3d7d.7.7` — Implement delegated-authorship automation
    - `focusa-3d7d.7.7.1` — Gate authorship delegation by explicit trust policy
    - `focusa-3d7d.7.7.2` — Record delegated amendments as authoritative changes
    - `focusa-3d7d.7.7.3` — Cascade delegated spec amendments through BD and implementation targets

---

## Recommended execution order

For immediate implementation:
1. Epic 1
2. Epic 2
3. Epic 3
4. Epic 4
5. Epic 5
6. Epic 6
7. Epic 7

Phase-1 kernel proof target is primarily covered by:
- Epic 1
- Epic 2
- Epic 3
- Epic 4
- Epic 5
- Epic 6

Epic 7 is mostly post-kernel sophistication.

---

## Notes

- This hierarchy is intentionally full-spec, not kernel-only.
- Spec remains supreme over every bead in this tree.
- If any BD item drifts from the authoritative spec, the BD item must be reconciled to spec, not vice versa.
