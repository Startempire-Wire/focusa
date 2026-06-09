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

## 12) QN Addendum: Non-Lazy HLT Inference (2026-06-08)

### Problem
HLT ladder reuse became lazy because Focusa inferred from missing verified state. When current verified state is absent or stale, Focusa fell back to stale ladder lines, causing:
- Repeated non-meaningful HLT/MLG/STG entries
- Scope confusion (wrong project context persisted)
- Agents operating without explicit intent

### Root Cause
Focusa did not enforce a "verified state gate" before HLT inference. The system allowed inference from:
- Unverified or missing project_root
- Stale continuity packets (e.g., agent runtime paths like `/root/pi-mono`)
- Empty/missing evidence refs
- Missing `current_ask` context

### Non-Lazy HLT Flow (Required Pattern)

**Rule: No ladder mutation unless explicit intent is passed.**

#### Step 1: Lock Scope First (Single Source)
```bash
focusa project identity --project-root /path/to/project
git -C /path/to/project status --short > /tmp/recent-files.txt
git -C /path/to/project diff --name-only > /tmp/changed-files.txt
```

#### Step 2: Build Evidence Packet (from repo, CLI)
```bash
# Add your own checks: node/php/css lint + TODO scan + test outputs
focusa trajectory assess \
  --project-root /path/to/project \
  --observed-state "explicit evidence summary..."
```

#### Step 3: Set HLT/derived Goals Explicitly (No Inference Fallback)
```bash
focusa trajectory define-goal \
  --project-root /path/to/project \
  --goal-source operator \
  --operator-confirmed \
  --long-term-goal "What the project desires to be..." \
  --desired-end-state "Deterministic, file-backed audit closure..." \
  --mid-level-goal "Reconcile and harden implementation..." \
  --short-term-goal "Close current gap with CLI-verified evidence..." \
  --current-state "Verified scope=/path/...; evidence captured in /tmp/..." \
  --waypoint "Waypoint 1 description" \
  --waypoint "Waypoint 2 description"
```

#### Step 4: Verify It Stuck to Your Inputs
```bash
focusa trajectory view --project-root /path/to/project --json \
  | jq '.trajectory | {long_term_goal, mlg, stg, waypoints, active_gap}'
```

### Implementation Requirements

1. **Verified State Gate**: Before HLT inference, require:
   - Verified `project_root` (not agent runtime path, not broad root)
   - Explicit `current_ask` or `mission` present
   - Evidence refs captured OR explicit operator override

2. **No Lazy Fallback**: If verified state is missing:
   - Do NOT infer HLT/MLG/STG from stale ladder lines
   - Return `active_gap: "missing_verified_state"` with guidance to capture state
   - Emit warning: "Cannot infer goals without verified project scope and evidence"

3. **Scope Isolation**:
   - Agent runtime paths (`/root/pi-mono`, `/.claude/`, `/.letta/`, etc.) are NEVER project scope
   - See Spec98 for full agent runtime blocklist

4. **Evidence-Backed Inference**: Stronger inference allowed for:
   - STG/current-state reasoning with evidence tags
   - MLG gaps with captured proof

5. **Explicit Override**: Operator can override with `--operator-confirmed` flag to bypass verified state gate when intent is explicitly provided.

### Smart Pattern Summary
Explicit values + explicit current_state + explicit waypoints = non-lazy HLT. Without `--current-state` and explicit goals, Focusa must NOT use fallback text and must NOT "repeat" non-meaningful ladder lines.

---

## 12) References

- `docs/00-glossary.md`
- `docs/96-trajectory-projection-and-daemon-stability-spec.md`
- `docs/98-project-root-crdt-reconciliation-foundation-spec.md`
- `docs/current/FOCUSA_AUTHORITY_TAXONOMY_GENERATED.md`
- `crates/focusa-core/src/types.rs`
- `crates/focusa-api/src/routes/project.rs`
