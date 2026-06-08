# Agent DX/UX Merged Scope Spec

Status: planning
Scope: merged improvement scope for agent DX + UX after deep testing
Authority: planning/evaluation only; this spec does not authorize direct implementation by itself

## 0) Numbering + alignment note

- Numbered docs currently present: Spec100, Spec101, Spec102.
- Spec103/Spec104 currently exist as test/eval surfaces (`tests/spec103_*`, `tests/spec104_*`), not numbered docs.
- This document uses **Spec105** as the next merged scope doc per operator steering.

## 1) Core purpose and vision (non-negotiable)

Focusa remains:
- canonical cognition runtime (not transcript-memory hacks),
- project-scoped authority (`project_root + continuity_id`),
- evidence-first and verification-first,
- bounded contracts with explicit failure/recovery envelopes,
- recovery-first continuity system (checkpoint/resume survives compact/restart/handoff).

Any DX/UX improvement that weakens these is out-of-scope.

## 2) Explicit assumption from operator

Spec100 Context Cognition and Spec101 Bloatgaurd are **future-implementation targets**.
This scope aligns to them now but does not assume they are already complete in runtime.

## 3) Inputs merged into this scope

- Spec100 Context Cognition (`docs/100-context-cognition-spec.md`)
  - `ContextCognitionPacket` is advisory.
  - Context Curator/Context Compiler path is the target architecture.
- Spec101 Bloatgaurd (`docs/101-focusa-bloatgaurd-spec.md`)
  - leanness is a product invariant.
  - must reuse Spec100 context architecture, not create parallel engines.
- Spec102 Agent UX Composition + real-life testing (`docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md`)
  - practical UX/friction findings from live battery use.

## 4) What deep testing showed lacking (merged gaps)

1. Hot-path ambiguity (`accepted` vs truly materialized).
2. Mutation model drift (some routes dispatch-only, others direct-write).
3. Local-to-CI parity gaps (green local subsets still fail CI full strict lane).
4. Durability contract gaps (in-memory success without on-disk/restart proof).
5. Scope/authority churn during handoff/compaction transitions.
6. Hook/operator ergonomics (correct but too expert-heavy recovery path).
7. Too much reconstruction burden during blocked/degraded states.

## 5) Merged scope requirements

## 5.1 P0 (hard reliability + authority)

**DXUX-001: Canonical scope gate before durable writes**
- No durable project-aware mutation unless project identity is verified.
- If scope unverified or broad (`/root`, `/home`, etc.), route returns deterministic blocked envelope + exact next action.

**DXUX-002: Deterministic materialization contract**
- Mutating routes must declare one of:
  - `accepted_pending_materialization`, or
  - `materialized_canonical`.
- Never silent eventual consistency on hot paths.

**DXUX-003: One mutation model per route family**
- Route family is either daemon-dispatch authoritative OR direct serialized canonical writer.
- CI guardrail must fail mixed/ambiguous ownership.

**DXUX-004: CI parity as first-class preflight**
- One command target (`focusa preflight`) runs:
  - `cargo clippy --workspace -- -D warnings`,
  - strict spec gates,
  - canonical writer guardrail,
  - critical restart/compact/handoff checks.

**DXUX-005: Persistence triad proof for durability claims**
- Any route claiming persistence must pass: in-memory + on-disk artifact + restart restore.

## 5.2 P1 (agent cognition ergonomics)

**DXUX-006: Single continuation contract packet**
- Every continuation path should yield one compact packet with:
  - scope, authority, next action, blockers, evidence refs.

**DXUX-007: Machine-readable doability**
- Standard fields: `can_continue`, `blocked_reason_code`, `exact_next_action`.

**DXUX-008: Recovery explainability**
- Add `focusa explain <failure>` behavior:
  - root cause summary,
  - exact command sequence to recover,
  - confidence and assumptions.

**DXUX-009: Evidence-linked change policy**
- Key commits/task closes include explicit evidence citations (tests/docs/routes/handles).

## 5.3 P2 (high-polish UX)

**DXUX-010: Zero-ambiguity response layout**
- Every critical response: `status | authority | why | exact next action`.

**DXUX-011: Drift alarms**
- Warn when requested action exits verified project/continuity or contradicts active Workpoint boundary.

**DXUX-012: One-click compact/resume digest**
- Human+agent readable digest with canonical packet receipt and rehydrate refs.

## 6) Context Compiler + Bloatgaurd alignment matrix

| Concern | Spec100 anchor | Spec101 anchor | Spec105 requirement |
|---|---|---|---|
| Context engine ownership | ContextCognitionPacket + Context Curator | Reuse Spec100, no parallel engine | DXUX-003 |
| Token budget discipline | selected/excluded context, bounded packet | bloat budget + anti-dump posture | DXUX-002, DXUX-004 |
| Profiles/routines compatibility | advisory packet composition path | profile/routine surfaces | keep naming/path parity with existing profile_selector/routine_commands surfaces |
| Evidence/rehydrate fidelity | evidence refs + rehydrate refs | compact handles over raw dumps | DXUX-006, DXUX-009 |
| Operator trust | advisory vs canonical distinction | explicit anti-bloat contracts | DXUX-010 |

## 7) Best practices to enforce (currently uneven)

- golden-path contract tests per route: happy + failure taxonomy + recovery envelope,
- idempotency for mutating endpoints,
- docs/runtime/schema parity checks in CI,
- strict stale-signal hygiene and recency checks,
- rollback/undo guidance attached to risky operations,
- small-batch, evidence-linked changes.

## 8) What is still harder than necessary for agents

- too many active mental models at once (frame/session/workpoint/trajectory/continuity),
- mismatch states require manual reconstruction,
- hook failures expose internals without first-pass remediation,
- local “green enough” can still miss strict CI semantics.

## 9) DX and UX target outcomes

If this spec is implemented correctly, an agent should be able to:
1. bootstrap verified project scope quickly,
2. get exact next action without transcript archaeology,
3. recover from blocked states in one guided path,
4. run CI-equivalent preflight locally,
5. complete handoff with canonical packet + proof refs.

## 10) Acceptance criteria and metrics

- read-after-write consistency on hot paths: >= 99.9%,
- push→CI red caused by missing local parity checks: near zero,
- durability regressions (claimed persistent but restart-lost): zero,
- zero-to-safe-workpoint (verified scope + canonical continuation): < 90s target,
- zero-to-green local preflight path: < 5 min target for maintained repo.

## 11) Rollout sequence

Phase A (P0): DXUX-001..005
Phase B (P1): DXUX-006..009
Phase C (P2): DXUX-010..012

Each phase requires:
- route/test/doc parity evidence,
- no regression in authority boundaries,
- explicit failure envelope coverage.

## 12) Out of scope

- redefining Focusa core identity away from continuity/runtime authority,
- replacing Spec100/Spec101 with alternative context engines,
- UX-only polish that weakens determinism/evidence contracts.

## 13) References

- `docs/100-context-cognition-spec.md`
- `docs/101-focusa-bloatgaurd-spec.md`
- `docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md`
- `tests/spec102_profile_selector_routine_commands_test.sh`
- `tests/spec103_post_compaction_recovery_surfaces_test.sh`
- `tests/spec104_deep_focusa_surface_sweep.py`
