# Spec 144 — Deterministic HLT Extrapolation Algorithm (Addendum to Spec 143)

**Status:** DRAFT — addendum to Spec 143 LOCKED ladder  
**Parent:** `docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md`  
**Authority:** deterministic, evidence-bounded inference — does not relax Spec 143 deliberation or fallback rules  
**Related beads:** `focusa-vbcqu` family  
**Surfaces:** daemon, TUI, Cockpit, UIAI Engine — single shared algorithm, per-surface rendering only  

## 1. Purpose

Turn the operator's 2-sentence HLT into a helper that **suggests** the next ladder rungs without claiming future state as done. No extrapolation may create a canonical MLG/STG/Waypoint/Workpoint; the helper only produces a **deterministic, ranked suggestion** that the operator or ladder owner later promotes through the existing Spec 143 write path.

Encapsulates the true ecosystem goal:

> *Bring the Focusa platform to healthy, entitled operation; bring Cockpit and UIAI Engine each to a working state on its own latest released, testable MVP, interoperating through Focusa. Advance each surface incrementally from its docs toward full functionality — promoting only increments proven runnable.*

## 2. Non-Goals

- No auto-creation of canonical Trajectory, tasks, or Workpoints.
- No combining of product surfaces into one artifact — Focusa, Cockpit, and UIAI Engine remain distinct; the algorithm keeps per-surface slices.
- No bypass of evidence, Focus Gate, or release gates.

## 3. Definitions

- **HLT** — operator-confirmed 2-sentence operative objective.
- **MLG** — mid-level goal that makes measurable progress on HLT within one release cycle.
- **STG** — short-term goal achievable in one work session.
- **Waypoint** — ordered, verifiable step (max 7, per this addendum).
- **Suggested Workpoint** — the single highest-ranked waypoint slice ready to execute next; still requires promotion to canonical Workpoint.

## 4. Inputs (all required for deterministic run)

1. Canonical `HLT` text + `project_id` (from ProjectIdentity).
2. Current evidence frame — last 20 verdicts/gaps, last signed release tag, convergence state (`cli/daemon/tui/pi-extension` staleness), daemon health.
3. Surface inventory — per-surface MVP version and `latest released tag` for Focusa, Cockpit, UIAI Engine (from `docs/current/RELEASE_*` + git tags).
4. Docs gap list — open spec gaps mapped to surfaces (from `docs/` + `Uiai-Cockpit-005` decomposition when present).
5. Prior ladder snapshot (if any) — for stability/determinism across runs.

All inputs hashed to `extrapolation_input_hash` for replay.

## 5. Deterministic Algorithm (5 steps, run verbatim each time)

Precondition: `hash(HLT + inputs) != prior_hash` → recompute; else return cached suggestion.

**Step 1 — Normalize HLT clauses**
- Split HLT sentence 1 into Focusa-clause and Cockpit/UIAI-clause; sentence 2 into iteration-clause. Fail-closed if clauses missing. Each clause maps to one surface bucket (`focusa` | `cockpit` | `uiai-engine`).

**Step 2 — Derive MLG (1, bounded 120 chars)**
- Rule: `MLG = highest-priority open docs gap that unblocks the earliest failing convergence/runnable check`. Ties broken by: `focusa health → cockpit → uiai-engine`, then lexical gap id. Deterministic lexical sort, no LLM sampling.

**Step 3 — Derive STG (1, bounded 100 chars)**
- Rule: `STG = smallest slice of MLG provable in one session` (single surface + single gate). Must reference one measurable proof predicate (`health ok`, `tests pass`, `signed artifact`, `installer runnable`).

**Step 4 — Derive Waypoints (1..7, ordered)**
- Partition STG into ordered verifiable steps. Hard caps:
  - 1 ≤ n ≤ 7 — if decomposition exceeds 7, keeper keeps first 6 + last ("ship") waypoint and emits overflow note.
  - Each waypoint: `{ordinal, surface, verb(≤3 words), proof_ref}`.
  - Cross-surface waypoints forbidden — one surface per waypoint.
  - Deterministic ordering: prerequisites first (identity → health → tests → signed → runnable → docs sync).
- Dedup and lexical-stable sort within dependency tier.

**Step 5 — Suggest Workpoint (0..1)**
- Rank waypoints by `unblocked ∧ smallest proof cost ∧ highest ecosystem leverage`. Winner becomes `suggested Workpoint` with `reason: which gap + which proof`. If none unblocked, suggestion is empty with `blocked_reason` surfaced, never fabricated.

Output envelope (`hlt_extrapolation_suggestion.v1`):
```json
{ "hlt_hash": "…", "mlg": "…", "stg": "…", "waypoints": [ …1..7 ], "suggested_workpoint": { "waypoint_ordinal": 1, "reason": "…" } | null, "input_hash": "…" }
```

Determinism: same inputs → byte-identical JSON. No timestamps or random seeds.

## 6. Surface Alignment

One algorithm, three renderings — no per-surface fork:
- **Daemon/TUI/CLI** — suggestion rendered in `focusa_trajectory_view` as advisory overlay, never canonical ladder.
- **Cockpit** — same suggestion consumed via shared helper for OTA/install slices.
- **UIAI Engine** — same helper for engine release proofs.
All read `input_hash` to know when suggestion is stale.

## 7. Promotion (how suggestion becomes real)

Suggestion → operator/TODO owner promotes explicitly via Spec 143 `Trajectory ladder write` path (HLT→MLG→STG commit) and Workpoint `create/promote`. Promotion creates the only auditable record; suggestions are never written to ledger.

## 8. Safety

- Never claims future state as present — wording uses "bring/advance/promote" not "is."
- Respects Focus Gate — if gate blocks, no suggestion beyond gate.
- Respects W0 lock — helper is read-only until Spec 143 W0 review closes; then it may run advisory-only.

## 9. Verification

- Doc: this addendum reviewed against Spec 143 ladder and glossary.
- Unit: deterministic golden — same HLT+inputs → identical output across runs; cap 7 enforced; one-surface-per-waypoint invariant.
- Integration: daemon + TUI + Cockpit render same suggestion from shared helper.
