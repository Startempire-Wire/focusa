# Spec 151 — Focusa Emergency Cross-Project Scope Isolation Locked-Release Addendum

- **Status:** NORMATIVE EMERGENCY ADDENDUM — P0/SEVERE — MANDATORY LOCKED-RELEASE BLOCKER
- **Project:** Focusa
- **Created:** 2026-07-31
- **Owner:** Focusa Core / Pi Extension / Trajectory / Workpoint / Context Authority / Release Engineering
- **Incident bead:** `focusa-gkwt`
- **Interrupted bead:** `focusa-vbcqu.2.2.4` remains open and blocked by this incident
- **Parents:** Specs 98, 100, 104, 111, 125, 130, 137, 137A, 138, 138A, 140, 149
- **Precedence:** This addendum overrides any weaker fallback, cache reuse, prior-project reload, similarity, broad-cwd, degraded-mode, or best-effort behavior that can expose one project’s context inside another project’s agent session.
- **Closure relationship:** The locked release cannot be approved, tagged, deployed, published, or represented as scope-safe until every requirement and acceptance gate in this addendum is proven.

---

## 1. Emergency finding

During a verified Focusa Workpoint, Pi tool output repeatedly injected a foreign trajectory:

- expected project: `Focusa`
- verified root: `/home/wirebot/focusa`
- continuity: `focusa-v0.9.135-locked-14`
- foreign trajectory id: `trajectory:project-fnv1a64:62237f5c6a62200f:wire-pitch-trajectory-recover-20260…`
- foreign MLG/STG: governed Wire Pitch outlet-profile research
- observed mixed payload: Focusa HLT combined with Wire Pitch MLG, STG, and waypoints

This is a cross-project authority-boundary breach. Advisory labeling does not reduce severity because injected context can alter agent reasoning, tool choice, mutation targets, evidence interpretation, and release decisions.

## 2. Mandatory safety posture

1. Exact typed scope is the only selection authority.
2. Similarity, path hashes, trajectory text, HLT text, recency, broad cwd, process globals, or cache proximity never grant cross-project authority.
3. A context packet is atomic: HLT, MLG, STG, waypoints, gap, evidence, and trajectory id must originate from one scope-verified trajectory revision.
4. Missing, stale, ambiguous, conflicting, or unverifiable scope fails closed.
5. Suppressing trajectory injection is always safer than injecting a possibly foreign trajectory.
6. No fallback may silently cross `ProjectRootKey`, `WorkstreamKey`, or `WorkingSubpathId`.

---

## 3. Typed authority boundary

The canonical lookup and cache key is:

```text
TrajectoryContextKey {
  project_root_key: ProjectRootKey,
  workstream_key: WorkstreamKey,
  working_subpath_id: WorkingSubpathId,
}
```

`session_id` is temporal metadata, not project authority. A verified worktree may map to its canonical project root only through ProjectIdentity evidence. A raw cwd string is not a canonical project root.

A packet may be injected only when all of the following are true:

```text
packet.project_root_key == active.project_root_key
packet.workstream_key == active.workstream_key
packet.working_subpath_id is compatible with active.working_subpath_id
packet.trajectory_id belongs to the same atomic trajectory revision
packet.scope_status == verified
packet.stale == false
packet.scope_conflict_reason == none
```

---

## 4. Normative requirements

| ID | Requirement |
|---|---|
| ESI-001 | Every Trajectory read, cache, preload, render, and injection MUST require exact `ProjectRootKey` plus `WorkstreamKey`. |
| ESI-002 | Working-subpath resolution MUST use verified ProjectIdentity mapping; raw cwd equality or path hashing MUST NOT grant authority. |
| ESI-003 | HLT, MLG, STG, waypoints, active gap, evidence refs, and trajectory id MUST be loaded and validated as one atomic scoped revision. |
| ESI-004 | The renderer MUST reject mixed-field packets assembled from different trajectories, revisions, projects, or workstreams. |
| ESI-005 | Missing or empty project/workstream identity MUST suppress injection and return a typed scope failure. |
| ESI-006 | `allow_prior_project_trajectory` MUST remain explicit, advisory-only, separately labeled, and MUST NOT enter automatic tool-result context. |
| ESI-007 | Same-project prior-continuity recovery MUST require explicit policy plus verified root identity and MUST remain non-canonical until promoted. |
| ESI-008 | Cross-project fallback is prohibited for prompt injection, tool guidance, action authority, mutation planning, and evidence settlement. |
| ESI-009 | All process caches MUST be keyed by `TrajectoryContextKey`; singleton “last trajectory” state is prohibited. |
| ESI-010 | Project, cwd, worktree, continuity, fork, resume, compaction, reconnect, and model-switch transitions MUST invalidate incompatible cached context before the next tool result. |
| ESI-011 | Workpoint and Trajectory packets MUST be independently scope-validated before composition; one valid packet MUST NOT launder the other. |
| ESI-012 | Pi, CLI, API, Canvas, TUI, preload, awareness, browser bridge, silent sessions, and every tool family MUST enforce the same scope gate. |
| ESI-013 | Tool-result decoration MUST run the scope gate immediately before injection, not only when data is fetched. |
| ESI-014 | Degraded mode MUST suppress foreign or ambiguous context; it MUST NOT weaken scope matching. |
| ESI-015 | Rejected context MUST emit bounded metadata only: failure class, candidate scope hash/ids, active scope ids, and recovery route; foreign content MUST NOT be echoed. |
| ESI-016 | Legacy globally scoped records MUST be scanned individually and migrated only from convergent authoritative evidence; ambiguous records remain quarantined advisory. |
| ESI-017 | Migration MUST preserve immutable source records, produce per-record plans and receipts, and support dry-run, apply, resume, and rollback. |
| ESI-018 | Scope mismatch telemetry MUST be exact-scope, content-minimized, restart-safe, and incapable of becoming selection authority. |
| ESI-019 | Security tests MUST cover maliciously crafted packets whose HLT matches the active project while MLG/STG/waypoints belong to another project. |
| ESI-020 | Release proof MUST demonstrate zero foreign context across alternating projects and workstreams under cache, resume, compaction, reconnect, and concurrent execution pressure. |

---

## 5. Required implementation seams

1. **Canonical resolver:** one typed `TrajectoryContextKey` constructor from verified ProjectIdentity and continuity authority.
2. **Scoped store:** CRDT/reducer ownership keyed by exact project and workstream identity.
3. **Scoped cache:** no unkeyed last-value or process-global trajectory authority.
4. **Atomic packet validator:** validates identity and common revision lineage for every rendered field.
5. **Final injection firewall:** runs at the Pi/tool-output boundary immediately before prompt-visible decoration.
6. **Transition invalidator:** clears incompatible cache entries on project/workstream/subpath/session transitions.
7. **Migration service:** per-record evidence scanner, planner, dry-run, apply, receipt, quarantine, and rollback.
8. **Bounded diagnostics:** reports mismatch metadata without disclosing foreign project content.

## 6. Required adversarial matrix

The gate MUST exercise at least these pairings:

| Active | Candidate | Required result |
|---|---|---|
| same root, same workstream, same subpath | exact current revision | inject |
| same root, same workstream, compatible verified worktree | exact scoped revision | inject |
| same root, different workstream | any trajectory | reject |
| different root, same continuity string | any trajectory | reject |
| different root, different workstream | any trajectory | reject |
| broad cwd `/root` | cached project trajectory | reject until ProjectIdentity verifies exact root |
| valid Focusa HLT + foreign MLG/STG | mixed packet | reject |
| stale exact-scope packet | old revision | reject or render explicitly stale outside prompt authority |
| prior same-project trajectory, explicit recovery | advisory candidate | never automatic injection |
| missing scope fields | legacy record | quarantine; never infer |

Tests MUST alternate projects repeatedly and run concurrently to expose cache-key and last-value defects.

---

## 7. Migration acceptance

A legacy/global record may be promoted only when authoritative evidence converges on one exact `TrajectoryContextKey`. Acceptable evidence includes immutable creation receipts, exact Workpoint bindings, verified ProjectIdentity records, scoped event ancestry, and operator-confirmed correction. Similar wording, matching HLT, shared cwd ancestry, temporal proximity, and path hashes are insufficient.

Each migration receipt MUST include:

- source record id and immutable digest;
- old scope representation;
- proposed exact typed scope;
- evidence refs and confidence class;
- conflicts considered;
- decision: migrate, quarantine, or reject;
- actor, wall-clock timestamp, and causal event id;
- rollback reference;
- post-migration isolation verification.

## 8. Rollback and containment

Until all gates pass:

1. disable prompt-visible Trajectory injection whenever exact scope cannot be proven;
2. preserve Workpoint authority independently;
3. quarantine ambiguous legacy records without deletion;
4. retain immutable pre-migration snapshots and receipts;
5. if any cross-project test fails, restore the last known scope-safe snapshot and keep injection disabled.

Rollback MUST prefer loss of advisory convenience over any possibility of foreign context exposure.

---

## 9. Acceptance evidence

Required proof artifacts:

- requirement-to-code-test-evidence matrix for `ESI-001` through `ESI-020`;
- unit tests for typed key construction, atomic packet validation, and final injection firewall;
- integration tests across API, CLI, Pi, Canvas/TUI, preload, resume, compaction, reconnect, and browser/tool bridges;
- concurrency and cache-churn tests alternating at least two projects and two workstreams;
- legacy migration dry-run/apply/quarantine/rollback fixtures;
- regression fixture reproducing Focusa HLT mixed with Wire Pitch MLG/STG and proving rejection;
- release receipt showing no unresolved scope mismatch or foreign-context event.

## 10. Locked-release gate

The emergency addendum is complete only when:

1. all `ESI-*` rows are implemented and specifically evidenced;
2. `focusa-gkwt` acceptance criteria are satisfied;
3. the Wire Pitch reproduction is rejected before prompt injection;
4. no generic file-only or duplicated proof block is accepted as row-level evidence;
5. full core/API/CLI/Pi/Canvas/TUI and resume/compaction/reconnect suites pass;
6. legacy migration is proven granular, reversible, and fail-closed;
7. the operator approves lifting the P0 release block.

Until then: **locked release remains blocked; no tag, deploy, publish, or completion claim.**
