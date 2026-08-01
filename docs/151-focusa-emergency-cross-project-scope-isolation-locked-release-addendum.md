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
6. No fallback may cross `ProjectRootKey`; a prior-continuity fallback within the same verified project must be explicit, separately labeled, advisory-only, and non-authoritative.

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

`session_id` and `session_identity` are rolling temporal metadata, not stable project authority. They may corroborate provenance or detect stale sessions but MUST NOT independently authorize or reject a trajectory. Stable ownership is `ScopeRef(ProjectIdentity) + continuity_id + WorkingSubpathId`. A verified worktree may map to its canonical project root only through ProjectIdentity evidence. A raw cwd string is not a canonical project root.

A canonical packet may be injected only when all of the following are true:

```text
packet.project_root_key == active.project_root_key
packet.workstream_key == active.workstream_key
packet.working_subpath_id is compatible with active.working_subpath_id
packet.trajectory_id belongs to the same atomic trajectory revision
packet.scope_status == verified
packet.stale == false
packet.scope_conflict_reason == none
```

A prior-continuity fallback may remain available when `packet.project_root_key == active.project_root_key`, but it must carry its source continuity, remain separately labeled advisory context, and never be merged field-by-field with the current trajectory or treated as action authority.

---

## 4. Normative requirements

| ID | Requirement |
|---|---|
| ESI-001 | Every Trajectory read, cache, preload, render, and injection MUST require exact `ProjectRootKey` plus `WorkstreamKey`. |
| ESI-002 | Working-subpath resolution MUST use verified ProjectIdentity `ScopeRef`; raw cwd equality, path hashing, rolling `session_id`, or rolling `session_identity` MUST NOT grant stable authority. |
| ESI-003 | HLT, MLG, STG, waypoints, active gap, evidence refs, and trajectory id MUST be loaded and validated as one atomic scoped revision. |
| ESI-004 | The renderer MUST reject mixed-field packets assembled from different trajectories, revisions, projects, or workstreams. |
| ESI-005 | Missing or empty project/workstream identity MUST suppress injection and return a typed scope failure. |
| ESI-006 | `allow_prior_project_trajectory` MUST remain supported, but candidates MUST match the exact verified `ProjectRootKey`, carry source continuity, remain advisory-only, and be rendered separately from canonical context. |
| ESI-007 | Same-project prior-continuity recovery MAY enter automatic tool-result context only as an explicitly labeled advisory fallback; it remains non-canonical and cannot grant action authority until promoted. |
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
| ESI-021 | Work Loop writer leases, preflight, mutation, checkpoint, and context updates MUST derive the identical stable scope key; first-writer bootstrap MUST NOT depend circularly on an already-held lease. |
| ESI-022 | Begin-session, preload, resume, compaction, model-switch, fork, reconnect, and recovery-sidecar injection MUST independently revalidate stable scope immediately before prompt visibility. |
| ESI-023 | Rolling `session_id` changes MUST NOT repartition or authorize project/workstream stores; session identity remains metadata attached to the stable scope store. |
| ESI-024 | Every trajectory-consuming tool family MUST reject or quarantine a packet that lacks a verified stable scope receipt bound to its trajectory id and source revision. |
| ESI-025 | Scope remediation MUST preserve every valid feature, projection, and fallback; implementations MUST refactor ownership and verification rather than delete functionality as a safety shortcut. |

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

### 5.1 Affected architecture and tool families

The change is cross-cutting and MUST cover:

- ProjectIdentity, scope resolution, working-subpath mapping, and typed scope stores;
- Trajectory define/view/assess/propose/checkpoint/resume/history and HLT fallback;
- Workpoint checkpoint/resume/evidence composition and ECS trajectory attachments;
- Work Loop writer status/control/context/checkpoint/select-next lease authority;
- Context Cognition, awareness, preload, bootstrap, session transfer, and recovery packets;
- begin-session, resume, compaction, model switch, fork, reconnect, and sidecar restore;
- prediction, metacognition, ontology, evidence, browser diagnostics, and report surfaces that render trajectory-linked context;
- Pi tool-result decoration and every tool family receiving common injected context;
- CLI, API, Canvas, TUI, menubar, browser bridge, and silent-session projections.

### 5.2 Lease-bug relationship

The broad-cwd and rolling-session partition defects overlap with `focusa-mzqsa`, but trajectory isolation does not by itself close the Work Loop lease bug. Closure requires separate proof that first-writer bootstrap acquires exact scoped authority without circular lease dependency and that preflight and mutation resolve identical stable scope keys.

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
| prior same-project trajectory, explicit recovery | advisory candidate | render only as separately labeled non-authoritative fallback |
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

## 11. Implementation checkpoint — stable ontology and PRE ownership

The locked working subpath now enforces these additional boundaries:

- mutable ontology events and persisted proposal, verification, working-set refresh, and delta records carry optional stable `WorkstreamKey`; `None` is legacy replay state only;
- ontology mutation routes, generic PRE submission/list/resolution, and constitution proposal ingress require verified ProjectIdentity root plus continuity scope before mutation or disclosure;
- reducer lookup and mutation use `(WorkstreamKey, record/object/link identity)` so duplicate ids in different projects cannot cross-promote, reject, verify, or refresh;
- serialized ontology objects and links carry the same workstream receipt; scoped projections recognize `workstream.root_scope.root_path` plus `workstream.continuity_id`;
- unowned legacy mutable records remain deserializable but are quarantined from scoped projections; immutable `global_schema` objects remain visible;
- adversarial reducer proof covers duplicate proposal/object ids in two workstreams through propose, promote, and failed verification transitions.

Current proof: core `731/731`; API `430/430`. This checkpoint advances `ESI-006`, `ESI-007`, `ESI-008`, `ESI-010`, `ESI-013`, and `ESI-018` but does not settle them until migration receipts, all client surfaces, runtime reload, and end-to-end alternation gates pass.

## 12. Implementation checkpoint — granular event-sourced migration

The locked working subpath now provides `/v1/ontology/scope-migrations` with scoped `dry_run`, `apply`, `status`, and `rollback` actions:

- dry-run returns bounded record kind/hash/ref candidates without exposing legacy payload content;
- apply requires migration-level and per-record evidence, rejects ambiguous hashes, and emits one reducer event;
- the reducer preserves each unowned source unchanged, creates a target-WorkstreamKey clone, and appends an immutable apply receipt containing source and clone hashes;
- repeated apply events with the same migration id are idempotent;
- rollback verifies every target clone remains byte-identical, removes only exact clone hashes, preserves legacy sources, and appends a separate rollback receipt;
- duplicate selections, missing evidence, absent sources, ambiguous sources, foreign-workstream receipts, repeated rollback under a different id, and mutated clones fail closed;
- `global_schema` records are excluded from migration candidates.

Current proof: core `733/733`; API `431/431`. Migration client parity, interruption/restart integration, and installed-runtime proof remain locked-release blockers.
