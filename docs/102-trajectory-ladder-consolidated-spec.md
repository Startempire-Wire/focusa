# Trajectory Ladder Consolidation Spec

Scope: Focusa canonical orientation model (`docs/00-glossary.md`, `docs/96-trajectory-projection-and-daemon-stability-spec.md`, `docs/98-project-root-crdt-reconciliation-foundation-spec.md`, `crates/focusa-core/src/types.rs`).

## 1) Intent

Consolidate distributed references into one canonical spec for how Focusa represents, scopes, persists, and uses trajectory orientation.

This document defines **what Trajectory Ladder is**, **what it is not**, and how it must interact with Workpoint/ASCC/CLT.

## 2) Definition and status

1. **No standalone file called `Trajectory Ladder` exists**.
2. The concept is distributed:
   - `docs/00-glossary.md`: HLT/MLG/STG/Waypoint taxonomy and authority rule wording.
   - `docs/96-trajectory-projection-and-daemon-stability-spec.md`: projection model and daemon behavior.
   - `docs/98-project-root-crdt-reconciliation-foundation-spec.md`: binding/correctness and authority contract.
   - `crates/focusa-core/src/types.rs`: runtime carrier types and handoff role enum.

## 3) Canonical contract

- **Trajectory Ladder = advisory route context**, not a full execution authority.
- **Handoff role contract:** `trajectory` is `TrajectoryRouteGuidance` (advisory projection), while Workpoint remains immediate continuation authority.
  - `AsccFocusStateSlots` = semantic slots/cognition context.
  - `WorkpointContinuationAuthority` = canonical immediate next-action contract.
  - `CltLineageHistory` = append-only history and recovery.
  - `TrajectoryRouteGuidance` = HLT/MLG/STG/Waypoints guidance.

## 4) Persistence policy (must be scoped)

### 4.1 HLT persistence

- HLT is durable per `(project_root, continuity_id)`.
- Changed only by:
  - explicit operator steering, or
  - durable supersession evidence path.
- Scope mismatch must not allow canonical HLT writes.

### 4.2 HLT Ledger (append-only, scope-bounded)

- HLT changes are persisted to an append-only JSONL ledger: `{data_dir}/hlt-ledger/{project_root_hash}/hlt.jsonl`
- Each entry records: timestamp, event_id, lamport_ts, project_root, continuity_id, session_id, old_hlt, new_hlt, source, reason, evidence_refs
- Per Spec98/99: no singleton, scope-bounded by (project_root, continuity_id), CRDT-grade events
- API: `GET /v1/hlt/history?project_root=<path>&continuity_id=<id>&limit=50`
- Tool: `focusa_hlt_history` exposes ledger via API
- Ledger is **append-only** — old entries are never modified or deleted

## 4.3 Storage replacement (legacy)

- `hlt-ledger.md` style ad-hoc persistence is replaced by reducer-backed `TrajectoryProjectionRecord` and resumed injection paths.
- Long-lived orientation survives compaction only through canonical trajectory records, resumes, and verified project scope.

### 4.3 MLG/STG/Waypoints

- MLG/STG/Waypoints are route context and may be advisory/inferred when canonical certainty is lower.
- Promotion to authoritative trajectory path is through normal trajectory/Workpoint/evidence workflow.

## 5) Inference and confidence policy

From the projection contract:

- Minimal/light inference for HLT/goal only when needed and safe.
- Stronger inference is allowed for STG/current-state reasoning when evidence tags exist.
- No inference for approvals, scope changes, or destructive actions without explicit operator scope/approval.
- Supersession of HLT requires clear operator-confirmed path or durable evidence posture.

## 6) Scope and false authority controls

- All trajectory lookup/selection for projection or card output must be anchored to verified project scope.
- Do not treat cross-project records as canonical just because an ID happens to be active.
- Route to `ProjectIdentity`/`Trajectory View`/`Project Card` with explicit `project_root + continuity_id` authority checks before proceeding.

## 7) Clarity gate and continuation posture

- Before advancing work, run the route-consistent clarity checks:
  - project identity scope match
  - Focus State/Focu point continuity sanity
  - Workpoint readiness
  - ontology/evidence alignment
- When checks fail, prefer `verify_first` over direct execution.

## 8) Operational role boundaries

- **Focusa tool orchestration order (authoritative orientation):**
  1. `focusa_project_verify` / `focusa_project_identity`
  2. `focusa_trajectory_view` (or projected trajectory)
  3. Workpoint resume/resolution (`focusa_workpoint_resume` / `focusa_workpoint_checkpoint`)
  4. Evidence / prediction / metacog-informed action
- `focusa_project_card` remains advisory and bootstrap-oriented; it must not be used as canonical source for route-changing authority.

## 9) Agent-first visibility and alerts

- Any change in trajectory durability state that affects HLT continuity (new canonical HLT, continuity mismatch, or lost prior HLT list) must trigger a visible, high-priority warning in every active agent channel ("flashing light" equivalent).
- The warning is non-blocking but persistent until the agent either acknowledges it or writes a checkpoint that resolves scope clarity.
- Manual retrieval remains available and should be explicit, not default: `focusa_trajectory_view --project_root <path> --continuity_id <id> --mode summary` is not auto-invoked by default for routine turns.
- Agents must receive both `trajectory.durable_lifecycle.history` visibility hint and a short action hint (`focusa_trajectory_view` for historical HLT list) when auto-warning fires.

## 10) Failure mode to avoid

- **Wrong HLT/HLG in Project Card** = a scope-boundary violation in context selection, not a separate ladder subsystem bug.
- Fix is to enforce project-root-scoped trajectory/workpoint selection prior to reading ladder fields.

## 11) Acceptance criteria (for implementation and review)

1. A card call with verified scope must show the correct project HLT/trajectory context.
2. HLT changes cannot occur via inferred projection alone; must follow explicit operator or durable-supersession path.
3. Cross-project active IDs do not leak into project-local card output.
4. Tests prove scoped trajectory/workpoint selection prefers matching `project_root` over stale/foreign active IDs.
5. Trajectory route context remains separate from execution authority in docs and code comments/roles.
6. Critical HLT continuity changes emit visible agent alerts and keep manual HLT history retrieval opt-in.

## 12) References

- `docs/00-glossary.md`
- `docs/96-trajectory-projection-and-daemon-stability-spec.md`
- `docs/98-project-root-crdt-reconciliation-foundation-spec.md`
- `docs/current/FOCUSA_AUTHORITY_TAXONOMY_GENERATED.md`
- `crates/focusa-core/src/types.rs`
- `crates/focusa-api/src/routes/project.rs`
