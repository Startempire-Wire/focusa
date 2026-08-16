# Spec 104 — Typed Scoped Runtime and Singleton Elimination

Status: strict implementation spec
Scope: Focusa core, API, CLI, Pi extension, `focusa_*` Pi tools, menubar Mac app, awareness/bootstrap adapters, packets, ledgers, eval/bench/proof surfaces, tests, docs, and migration discipline
Authority: architectural execution spec; downstream implementation must still satisfy Spec 107 proof/closure discipline

## 0. Numbering + status note

- `docs/104-*.md` did not previously exist in this repo.
- `tests/spec104_deep_focusa_surface_sweep.py` already exists as a test/eval surface; Spec 104 now owns the numbered document and that test becomes supporting implementation evidence rather than numbering authority.
- Specs 109, 110, and 111 exist as numbered docs, but current repo evidence does **not** prove they are fully implemented yet. Spec 104 defines the required runtime/state contract regardless of 109–111 implementation status.

## 0.1 Purpose of this spec

Spec 98 established that singleton daemon `current` / `active` / `last` authority was the foundational implementation mistake.

Spec 99 established that many surfaces still behave as if singleton state is canonical, with scope guards added later as patches.

Spec 104 is the repo-wide execution spec that turns that diagnosis into a complete implementation path:

```text
No canonical authority from global current/active/last state.
Every canonical surface typed.
Every mutable authority surface scoped.
Every ambiguity blocked instead of silently adopted.
```

This is not just a project-root correction. It is the complete anti-singleton program across:

- daemon/runtime,
- API routes,
- CLI,
- Pi runtime,
- Pi `focusa_*` tools,
- menubar,
- awareness/bootstrap adapters,
- benchmark/eval/proof surfaces,
- tests/docs/contracts.

## 1. Normative basis

This spec extends and makes strict:

- **Spec 98** — canonical Focusa authority must not depend on daemon-global singleton state
- **Spec 99** — implementation still centers many subsystems around singleton `current` / `active` / `last` fields
- **Spec 100** — typed advisory packets must carry scope + authority fields
- **Spec 102** — primary scope and continuity semantics must be explicit and stable
- **Spec 105** — no durable project-aware mutation without verified scope; broad roots block deterministically
- **Spec 106** — canonical vocabulary / hierarchy discipline; no weakening of precision
- **Spec 107** — spec-first and proof-first lifecycle
- **Spec 108** — unsafe-root and scope-conflict awareness surfaces
- **Spec 109** — agent-first API typed-contract direction
- **Spec 110** — Pi reminder behavior
- **Spec 111** — bootstrap/delivery must compile from verified authority surfaces
- **Spec 113 / 114** — benchmark/eval/proof integrity requires anti-bleed runtime behavior

If any later implementation or document reintroduces singleton canonical authority, this spec wins.

## 2. Problem statement

The current codebase still contains mutable global state and singleton-shaped runtime behavior across multiple surfaces.

The problem is **not** merely that some `static` values exist. The problem is that some mutable global state can influence:

- project identity,
- workstream identity,
- Workpoint / Trajectory / Focus State continuation,
- Pi bootstrap / compaction / reload behavior,
- `focusa_*` tool request payloads,
- benchmark/eval/proof integrity,
- or cross-project / cross-session adoption.

### 2.1 Current codebase reality

Observed singleton evidence includes:

- API globals (`APP_STATE`, prediction/metacog/turn caches, workpoint idempotency cache, project identity payload cache, snapshots store)
- runtime infra globals (resource mode / bounded response tracking)
- Pi extension global runtime state `S`
- one-slot remembered project root cache
- session restore logic that rehydrates prior identity / trajectory / workpoint shadow state
- tool-layer helpers that auto-inject persisted project identity into `focusa_*` tool requests
- menubar and adapter surfaces that reconstruct scope by merging multiple string fields

### 2.2 Why this matters now

Spec 104 must solve this before later product surfaces become trust sinks:

- host/server-wide scope support,
- menubar continuity control,
- agent bootstrap packets,
- benchmark ON/OFF runs,
- public-safe proof bundles.

If singleton authority remains, later features become cleaner wrappers over an untrustworthy substrate.

## 3. Core thesis

Move from:

```text
global mutable runtime + scope guards + recovery patches
```

to:

```text
typed scope keys + explicit authority envelopes + scoped state partitions + append-only ledgers + blocked ambiguity
```

The goal is **not** “zero `static` anywhere.”

The goal is:

1. zero **authority-bearing** singleton state,
2. zero untyped canonical scope,
3. zero fallback from ambiguous root/session into prior active authority,
4. zero silent cross-scope adoption in Pi / API / CLI / menubar / benchmark surfaces.

## 4. Definitions

### 4.1 Authority-bearing singleton

A mutable process-wide or adapter-wide state holder that can influence canonical authority, continuation, mutation routing, identity routing, evidence linkage, benchmark outcome, or cross-scope adoption.

### 4.2 Runtime-infrastructure singleton

A mutable process-wide holder that does **not** define canonical project/workstream authority, but still affects runtime behavior.

Examples:

- rate-limit buckets,
- resource-mode hysteresis,
- pressure tracking,
- shared HTTP clients.

These are tolerated only if explicitly classified and quarantined from canonical authority.

### 4.3 Typed scope

A machine-readable scope object whose schema is explicit, stable, and required, not implied by cwd, transcript tail, or prior active state.

### 4.4 Canonical scope

The explicit typed scope required for canonical read/write authority. Absence or ambiguity must produce a blocked envelope, never fallback global state.

`continuity_id` is never canonical authority by itself. It is secondary workstream metadata under a verified typed root/scope key. Any surface that can change `continuity_id` without changing the verified root must continue to resolve the same canonical project/host scope.

### 4.5 Authority-bearing singleton eradication rules

1. No daemon-global canonical authority.
2. No one-slot remembered root authority.
3. No unkeyed mutable cache for project/workpoint/trajectory/focus/prediction/metacognition authority.
4. No Pi `focusa_*` tool may silently adopt prior active project/workpoint/trajectory.
5. Globals may exist only for:
   - immutable constants,
   - process health,
   - peer registry,
   - telemetry / pressure accounting,
   - or explicitly classified non-authority runtime infra.
6. Every mutable canonical surface must be keyed by typed scope.
7. Every ambiguity across scopes must block with exact next action.

## 5. Required target model

### 5.1 Universal typed scope family

Spec 98 introduced `ProjectRootKey`, `WorkstreamKey`, and `AttachmentKey`. Spec 104 generalizes that into a cross-surface type family.

Minimum required shape:

```rust
ScopeKind = Project | Host

ScopeRef {
  scope_kind: ScopeKind,
  scope_id: String,
  root_path: PathBuf,
  canonical_name: String,
  fingerprint: String,
}

ProjectRootKey = ScopeRef(scope_kind=Project)
WorkstreamKey = ProjectRootKey + continuity_id
AttachmentKey = WorkstreamKey + instance_id + session_id + attachment_id
```

#### 5.1.1 Continuity semantics

`continuity_id` is a **workstream discriminator**, not root authority.

Required:

- canonical project/host identity must survive `continuity_id` churn,
- caches may partition by `continuity_id` only under a verified typed root/scope key,
- no API/CLI/Pi/menubar/adapter surface may adopt or switch canonical scope based on `continuity_id` alone,
- `workspace_id` convenience fields must not be implemented as `continuity_id` authority aliases,
- if Pi sessions regenerate or shift `continuity_id`, canonical project binding must remain tied to verified root/scope identity.

Not allowed:

- `continuity_id` as sole cache key for canonical authority,
- `continuity_id` as sole restore key for Workpoint/Trajectory/Focus State,
- `continuity_id` as sole benchmark-arm/session identity,
- `continuity_id`-only matching in consumer surfaces.

#### 5.1.2 Trajectory semantics

Trajectory must be rock solid under the typed scope model.

Required:

- HLT primary scope remains the verified project/host root, not `continuity_id`,
- `continuity_id` may select the current workstream trajectory view, but may not redefine canonical project trajectory authority,
- generic bootstrap HLT text is degraded placeholder content, not valid HLT authority,
- trajectory remains advisory route guidance while Workpoint remains immediate continuation authority,
- a valid historical HLT under the same verified root outranks generic bootstrap text,
- trajectory restore/resume paths must require typed root/scope match before any continuity/workstream matching,
- Pi, menubar, adapters, and bootstrap packets must not surface generic HLT as if it were canonical project intent.

Not allowed:

- regenerating or replacing HLT merely because `continuity_id` changed,
- `continuity_id`-only trajectory restore,
- daemon-global `active Trajectory` or adapter-global `lastTrajectoryClarity` as canonical truth,
- generic `Maintain and improve <project> within verified project scope` bootstrap text treated as real HLT,
- Workpoint/current_focus silently manufacturing MLG/STG from an invalid or generic HLT.

Equivalent typed TS/JSON shapes must exist for:

- API request/response envelopes,
- CLI payloads,
- Pi `focusa_*` tool payloads,
- menubar requests,
- awareness/bootstrap adapters,
- benchmark/eval/proof ledgers.

### 5.2 First-class host scope

This spec must support legitimate server-wide work without weakening strict scoping.

Therefore:

- `/root` remains unsafe as a **project** root,
- `/root` may become valid only as a **host** scope,
- host scope must not masquerade as project scope,
- host scope must be typed, verified, and blocked on ambiguity,
- a broad-root bypass flag is explicitly out-of-scope.

#### 5.2.1 Host identity shape

Minimum required host shape:

```rust
HostIdentity {
  scope_kind: Host,
  scope_id: String,          // e.g. host:<fingerprint>
  root_path: PathBuf,        // e.g. /root
  host_label: String,        // operator-stable human label
  owner_user: String,        // e.g. root
  fingerprint: String,       // derived from stable host evidence
  marker_path: Option<PathBuf>,
}
```

Host fingerprint must be derived from stable host evidence such as:

- hostname,
- machine-id or equivalent host identity source,
- root path,
- operator-confirmed host label,
- explicit host marker content when present.

#### 5.2.2 Host binding protocol

Safe root/host binding must follow this sequence:

1. broad cwd such as `/root` is detected,
2. project-scope binding stays blocked,
3. operator explicitly chooses host scope **or** a valid host marker supplies sufficient confidence,
4. host identity is built and verified,
5. canonical scope is recorded as `scope_kind=host`, never as `scope_kind=project`,
6. any convenience cache/profile for host scope is stored separately from project-root memory and must never populate `persisted_project_root`.

Required:

- explicit host bind/verify path in API/CLI/Pi surfaces,
- dedicated host marker format, e.g. `.focusa-host.json`,
- dedicated host cache/profile path separate from project-root cache,
- host scope packets and resumes must carry `scope_kind=host`,
- host scope may carry `continuity_id`, but only as secondary workstream metadata.

Not allowed:

- writing `/root` into project-root remembered cache,
- bypassing the project safety gate and pretending host scope is project scope,
- auto-adopting host scope from cwd alone without sufficient verification,
- treating host scope as authority for child repo/project scopes.

#### 5.2.3 Host vs project precedence

Required:

- verified project scope wins when the current target is a verified project and the operator has not explicitly pinned host scope,
- explicit pinned host scope wins only for server-wide work and must not silently absorb child project work,
- transitions between host scope and project scope must be explicit and visible,
- host-scoped Workpoint/Trajectory/Focus State must never be resumed into a project-scoped session or vice versa.

#### 5.2.4 Cross-surface host binding awareness

Every surface that can accept, emit, display, persist, recover, or infer scope must be aware of host binding as a first-class possibility.

Required:

- every scope-carrying schema supports `scope_kind=host` as well as `scope_kind=project`,
- every recovery/bootstrap path can distinguish host scope from project scope,
- every consumer surface can render host scope without coercing it into `project_root` semantics,
- every route/tool/client that accepts scope can reject invalid project/host coercion explicitly,
- every persistence/cache layer distinguishes host-scoped state from project-scoped state.

Not allowed:

- assuming every scope is a project root,
- dropping `scope_kind` on the floor and reconstructing only `project_root`,
- coercing host scope into project-root fields for convenience,
- using host scope as implicit fallback when project scope is ambiguous.

### 5.3 Packet / envelope contract

Every canonical-capable packet or envelope must carry explicit scope + authority fields.

Every scope-carrying packet or envelope must be able to represent both project scope and host scope without lossy coercion.

Minimum envelope shape:

```json
{
  "scope": {
    "scope_kind": "project|host",
    "scope_id": "...",
    "root_path": "..."
  },
  "authority": {
    "status": "canonical|advisory|blocked|degraded",
    "why": "..."
  },
  "continuity_id": "...",
  "session_identity": {"...": "..."}
}
```

No packet may depend on transcript tail, process-global active state, or adapter-local remembered root as canonical truth.

## 6. No omission / no deferral / no partial acceptance rule

Spec 104 must be exhaustive and strict.

### 6.1 Nothing omitted

Every mutable global or singleton-shaped state surface must be explicitly classified as one of:

- **eliminate**,
- **scope-key**,
- **infra allowlist**,
- **consumer to migrate**.

No “probably harmless” mutable global is allowed to remain undocumented.

### 6.2 No deferral as accepted state

Implementation sequencing is allowed. Partial acceptance is not.

That means:

- phases define execution order only,
- no phase completion is product acceptance,
- no omitted surface is acceptable because another phase will handle it later,
- no documentation may imply Spec 104 is satisfied while any Annex A/B/C item remains open.

### 6.3 Full-closure rule

Spec 104 is not implemented, accepted, or claimable as complete until:

- every canonical-surface requirement in §8 is satisfied,
- every required test class in §9 passes,
- every Annex A remediation row is completed,
- every Annex B scope-carrying surface is accounted for in runtime/contracts/tests,
- every Annex C mutable-global / singleton-evidence item is either eliminated, scope-keyed, or explicitly allowlisted as non-authority infra.

### 6.4 Bead-ready decomposition rule

Every future implementation bead derived from Spec 104 must cite:

- one or more Annex A row IDs,
- the migration phase,
- the acceptance tests required,
- exact dependencies and completion proof.

## 7. Implementation order

Implementation order is part of the spec, not optional process advice.

Phases are sequencing only. They do not authorize partial compliance claims.

### Phase 0 — complete inventory and classification

Produce the full singleton evidence matrix with:

- file/symbol,
- surface,
- mutable or immutable,
- authority-bearing or infra-only,
- current risk,
- target replacement,
- acceptance tests.

Machine enforcement artifacts:

- `config/spec104-scoped-state-inventory.json` is the classified source inventory.
- `tests/spec104_singleton_inventory_gate.py` fails on unknown or stale singleton/non-scoped markers.
- `tests/spec104_singleton_inventory_gate.py --closure` fails while any remediation remains open.
- `crates/focusa-core/src/scoped_state.rs`, `apps/pi-extension/src/scoped-state.ts`, and `config/scoped-state.schema.json` define Rust/TS/JSON contract parity.

### Phase 1 — establish typed scope model

Required before broad surface refactors:

- universal typed scope family,
- explicit host-vs-project distinction,
- host identity schema + host binding protocol,
- explicit rule that `continuity_id` is secondary to verified root/scope identity,
- shared scope envelope shapes for Rust + TS/JSON,
- contract vocabulary update.

### Phase 2 — remove authority-bearing canonical singletons

Priority order:

1. API authority caches/stores
2. Pi runtime singleton authority
3. remembered-root behavior
4. resume/bootstrap/adoption shadow state
5. `focusa_*` tool bridge hidden persisted-identity injection

### Phase 3 — migrate bridge and consumer surfaces

Priority order:

1. Pi tool runtime + compaction/awareness/commands
2. menubar Mac app
3. awareness/bootstrap adapters
4. CLI project/host split

### Phase 4 — classify and quarantine infra globals

- rate limits,
- resource mode,
- bounded route telemetry,
- shared clients,
- immutable projections.

These are allowed only if explicitly quarantined from canonical authority.

### Phase 5 — tests, contracts, and benchmark integrity

- static singleton audits,
- packet schema parity tests,
- scope mismatch tests,
- benchmark contamination tests,
- proof lineage tests.

## 8. Surface-by-surface requirements

### 8.1 focusa-core and scoped state engines

Required:

- canonical state under typed project/workstream/attachment scope keys,
- reducers reject unscoped canonical writes,
- append-only ledgers and materialized read models scope-keyed,
- any `active_*` pointer exists only inside scoped state,
- trajectory state preserves the Workpoint-vs-Trajectory authority split: Trajectory advisory, Workpoint immediate authority,
- HLT persistence remains keyed primarily by verified root/scope and must survive `continuity_id` churn under the same root,
- core types and reducers must be host-aware as well as project-aware.

This includes, at minimum:

- Focus State,
- Focus Stack,
- Work-loop,
- reducer,
- runtime persistence,
- reference store / replay surfaces,
- utility card / awareness substrate data models,
- sync/CRDT state models.

Not allowed:

- project adoption from session id alone,
- continuity id without typed root authority,
- project_root-only hacks that cannot generalize to host scope,
- host scope represented as a malformed project root.

### 8.2 API routes and middleware

Required:

- every canonical route accepts or derives verified typed scope,
- blocked envelope on missing/ambiguous scope,
- route contracts distinguish advisory vs canonical vs blocked,
- mutable caches that affect canonical behavior are scope-keyed or removed,
- API routes and middleware must accept and preserve host scope without coercing it into project-only fields.

This includes, at minimum:

- route families for awareness, call-stack, commands, context cognition, DXUX, ECS, focus/focus-state, health/status, ontology, predictions, project identity/verify/card, proposals, proxy, reflex, session, snapshots/tree, sync, trajectory, traverse, visual workflow, work-loop, and workpoint,
- middleware for auth, error envelopes, JSON guards, mutation rate limits, and any route-scope enforcement layer.

Not allowed:

- fallback to daemon-global current/active/last,
- fallback from ambiguous `/root` to prior active project,
- unscoped materialization of prediction/metacog/workpoint/trajectory/focus/work-loop authority,
- middleware that strips or loses `scope_kind`.

### 8.3 CLI command families

Required:

- explicit typed scope or local profile selection that is convenience only,
- separate project-scope vs host-scope commands,
- no remembered-root authority,
- CLI help/output/contracts must surface host scope as first-class, not as a project-root exception.

This includes, at minimum:

- project, trajectory, workpoint, focus, HLT, context cognition, awareness, call-stack, action, env/onboard, pair/pairing, and any command that can derive or pass canonical scope.

### 8.4 Pi runtime and `focusa_*` tools

This surface is mandatory and must not be forgotten.

Required:

- replace singleton `S` authority behavior with typed scoped runtime stores/services,
- every canonical-capable `focusa_*` tool payload carries explicit scope/authority/session identity fields,
- bootstrap/reminder/awareness/compaction/reload/turn flows become typed-scope-based,
- `focusa_agent_prompt`, `focusa_utility_card`, bootstrap cards, and post-compaction cards derive only from typed bootstrap/resume packets,
- MCP-facing prompt/context/handoff surfaces in the Pi plugin carry typed scope/authority or are explicitly marked non-canonical,
- local persisted bridge state becomes explicitly non-canonical,
- no Pi path may treat `continuity_id` alone as sufficient to recover canonical scope or continuation authority,
- no Pi path may restore `lastTrajectoryClarity` or other trajectory shadow state as canonical truth without verified typed root/scope match,
- Pi runtime and `focusa_*` tools must be explicitly host-binding-aware and must not collapse host scope into project-root-only behavior.

This includes, at minimum:

- `state.ts`, `session.ts`, `tools.ts`, `tool-contracts.ts`, `awareness.ts`, `awareness-substrate.ts`, `commands.ts`, `compaction.ts`, `turns.ts`, `config.ts`, `wbm.ts`, `polish.ts`.

### 8.5 Menubar Mac app

Required:

- stop reconstructing scope by mixed-source string fallback,
- consume typed scope packets/contexts,
- stop using `project_root` as fake `cwd`,
- use explicit typed session/adapter identity envelopes,
- render and act on host scope as first-class, not as a malformed project scope.

This includes both:

- the Svelte/UI scope consumers, and
- the native Tauri bridge/runtime layer.

### 8.6 TUI and other read consumers

Required:

- read-only consumers must still preserve typed scope semantics,
- Focus State / Focus Stack / Work-loop views must never synthesize scope from stale or mixed-source data,
- host scope must be displayable and recoverable without coercion into project-root-only language.

This includes, at minimum:

- `crates/focusa-tui` API/view/app surfaces,
- any future read-only clients or diagnostics viewers.

### 8.7 Awareness / bootstrap adapters

Required:

- no hardcoded project-root defaults,
- no env-only continuity authority,
- prepended context compiled from typed bootstrap packets,
- agent bootstrap cards and post-compaction cards compiled from typed bootstrap/resume packets only,
- blocked/advisory behavior on ambiguity,
- host scope must be represented explicitly in adapter config/request/packet shapes.

### 8.8 Bench / eval / proof

Required:

- typed run scope,
- typed arm scope,
- typed proof lineage,
- zero hidden singleton contamination in ON/OFF runs,
- benchmark/eval/proof surfaces must remain aware that runs may occur under host scope as well as project scope.

### 8.9 Docs, contracts, and inventories are also surfaces

Required:

- scope-carrying surface inventories stay current,
- contract registries stay in sync with runtime payloads,
- docs and tests must explicitly cover host-vs-project semantics,
- no scope-carrying file remains uncatalogued.


## 9. Required tests

### 9.1 Static audits

Static tests must fail if:

- a new authority-bearing mutable singleton is introduced,
- `focusa_*` tool payloads omit required scope/authority fields,
- contract docs drift from runtime payload shape,
- remembered-root authority is restored,
- consumer surfaces hardcode roots or merge scope ambiguously,
- a scope-carrying file exists in repo but is absent from the Spec 104 surface inventory.

### 9.2 Live/runtime tests

Must prove:

1. broad cwd such as `/root` cannot adopt prior project authority,
2. project mismatch produces blocked deterministic envelope,
3. host scope does not masquerade as project scope,
4. broad cwd such as `/root` offers explicit host-binding recovery instead of project fallback,
5. verified child project scope does not silently merge into host scope and host scope does not silently absorb child project scope,
6. multiple projects remain active without bleed,
7. multiple workstreams under same project remain isolated,
8. Pi `focusa_*` tools do not silently inherit stale scope,
9. trajectory HLT remains stable across `continuity_id` churn under the same verified root,
10. generic bootstrap HLT is surfaced as degraded placeholder, never canonical trajectory authority,
11. every scope-carrying surface preserves `scope_kind=host` without coercing it into project-root-only semantics,
12. menubar requests do not synthesize wrong scope from mixed sources,
13. compaction/reload/bootstrap do not restore singleton authority,
14. `continuity_id` churn within a Pi session or across resumes does not rebind canonical scope by itself,
15. benchmark ON/OFF runs do not share hidden scoped state.

### 9.3 Regression test families

At minimum:

- `tests/spec104_deep_focusa_surface_sweep.py`
- `tests/spec96_session_identity_envelope_static_test.sh`
- `tests/spec102_project_identity_mismatch_semantics_test.sh`
- Spec 105 scope/doability/drift tests
- Spec 109 envelope/contract tests
- Spec 111 bootstrap packet tests
- Spec 113/114 benchmark contamination tests

## 10. Acceptance criteria

This spec is accepted only when:

1. no authority-bearing singleton remains on any canonical surface,
2. all canonical-capable packets carry typed scope + authority,
3. Pi `focusa_*` tools are scope-truthful and singleton-free in authority behavior,
4. no surface depends solely on `continuity_id` for canonical authority,
5. trajectory remains advisory while Workpoint remains immediate continuation authority,
6. HLT remains root-scoped and stable across `continuity_id` churn under the same verified root,
7. generic bootstrap HLT remains degraded placeholder content and never masquerades as canonical project intent,
8. menubar and adapter surfaces consume typed scope instead of mixed-source fallback strings,
9. every scope-carrying surface is host-binding-aware as well as project-binding-aware,
10. ambiguous scope blocks instead of falling back,
11. broad-root server-wide work is solved through first-class typed host scope, not a bypass flag,
12. host scope and project scope have explicit precedence and transition rules,
13. benchmark/eval/proof evidence demonstrates no cross-scope contamination,
14. no scope-carrying surface remains uncatalogued in Spec 104 annexes,
15. no Annex A/B/C item remains open at acceptance time,
16. repo tests/docs/contracts agree on the typed scoped model.

## 11. Non-goals

This spec does **not** require:

- eliminating immutable constants,
- eliminating every process-wide infra helper on day one,
- forcing every infra helper into one storage mechanism,
- pretending 109–111 are already complete,
- weakening strict scoping just to support `/root` quickly.

## 12. Annex A — Remediation matrix

### A. Canonical runtime / API authority surfaces

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| API-01 | daemon app runtime | `crates/focusa-api/src/server.rs:329` | `APP_STATE: OnceLock<Arc<AppState>>` holds global app runtime | hidden coupling between canonical route behavior and process-global state | app runtime may remain, but canonical scope/authority must be request-local typed context | P0 | concurrent requests for two different scopes never affect each other; no route depends on global “current” app authority |
| API-02 | project identity | `crates/focusa-api/src/routes/project.rs:1781` | `PROJECT_IDENTITY_PAYLOAD_CACHE` caches identity payloads | stale or under-keyed identity cache can misroute scope authority | cache keyed by typed identity envelope / `ScopeRef`, never used as canonical fallback | P0 | rapidly alternating roots do not return cached wrong identity; stale cache cannot survive mismatch |
| API-03 | Workpoint continuation | `crates/focusa-api/src/routes/workpoint.rs:37` | `WORKPOINT_IDEMPOTENCY_CACHE` is process-global | idempotency path can accidentally cross scopes if key too weak | idempotency keyed by typed scope + workpoint/action id | P0 | same idempotency key under two scopes produces isolated records |
| API-04 | predictions | `crates/focusa-api/src/routes/predictions.rs:72` | `PREDICTION_CACHE` is process-global | prediction read/write bleed across projects/workstreams | scope-keyed prediction store/read model | P0 | predictions created in one scope never appear in another without explicit shared scope |
| API-05 | metacognition | `crates/focusa-api/src/routes/metacognition.rs:500` | `METACOG_STORE` is process-global | learning signals bleed across unrelated scopes | scope-keyed metacog store or durable scoped ledger/read model | P0 | retrieval for project A cannot see project B learning unless explicitly shared |
| API-06 | turn dedupe | `crates/focusa-api/src/routes/turn.rs:27` | `RECENT_COMPLETED_TURNS` is global | recent-turn suppression can affect wrong session/workstream | scope/session-keyed runtime correlation | P0 | repeated turn ids in different scopes do not collide |
| API-07 | snapshots | `crates/focusa-api/src/routes/snapshots.rs:89` | `SNAPSHOTS` global in-memory snapshot map | snapshot ids and restore lineage can bleed across scopes | scope-keyed snapshot store/index | P0 | restore only sees snapshots belonging to current typed scope |
| API-08 | device pairing runtime | `crates/focusa-api/src/routes/device_pairing.rs:455` | `STATE: OnceLock<SharedPairingState>` | valid runtime singleton but could drift into authority if reused carelessly | explicitly classify as runtime infra, not project/workstream authority | P2 | pairing state changes do not affect project identity / workpoint / trajectory authority |
| API-09 | ontology read index | `crates/focusa-api/src/routes/ontology.rs:1712` | `ONTOLOGY_READ_INDEX` caches read model globally | stale/global ontology index if ontology becomes scope-relative | scope-partition or scoped invalidation rules | P1 | ontology reads vary only by explicit scope, never by prior request |
| API-10 | proxy client | `crates/focusa-api/src/routes/proxy.rs:39` | `UPSTREAM_CLIENT` shared client | infra singleton; low risk but must be classified | keep as infra-only singleton | P3 | static audit allows only infra-allowlisted globals like shared HTTP clients |
| API-12 | compaction packet hot cache | `crates/focusa-api/src/routes/compaction.rs:72` | `STORE: OnceLock<Mutex<VecDeque<_>>>` caches bounded recent packets while SQLite remains durable truth | in-memory cache could be mistaken for replay authority | keep bounded cache advisory; packet replay and restart recovery must use durable scoped storage | P2 | packet survives restart through SQLite; cache eviction cannot remove durable packet |
| API-11 | mutation rate limit | `crates/focusa-api/src/middleware/rate_limit.rs:23` | `MUTATION_BUCKETS` global buckets
| BND-01 | resource mode / pressure tracking | `crates/focusa-api/src/routes/bounded.rs:14-24,143-146` | `TEST_PRESSURE_THRESHOLD_KB`, `RUNTIME_RESOURCE_MODE_OVERRIDE`, `RESOURCE_MODE_LAST_OBSERVED`, `RESOURCE_MODE_TRANSITIONS`, `RESOURCE_MODE_TRANSITION_OMITTED`, `RESOURCE_MODE_HYSTERESIS`, `PRESSURE_LAST_ACTIVE`, `PRESSURE_TRACKED_BUCKETS` are process-global | resource mode and pressure metrics are runtime infra affecting canonical route behavior; globals survive across scopes | scope-keyed runtime mode/pressure stores; transient values only; no canonical decision depends on global mode without typed scope | P2 | resource mode under one root never redefines behavior for another root | | caller/route throttling might be confused with project authority | classify as infra-only; no canonical scope dependence | P3 | throttling never changes project/workstream resolution outcome |

### B. Runtime infra globals to quarantine, not ignore

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| INF-01 | bounded runtime | `crates/focusa-api/src/routes/bounded.rs:14` | `TEST_PRESSURE_THRESHOLD_KB` | runtime pressure globals remain unclassified | explicit infra-only classification | P3 | static audit marks this as allowlisted infra |
| INF-02 | bounded runtime | `.../bounded.rs:16` | `RUNTIME_RESOURCE_MODE_OVERRIDE` | could become hidden authority if reused | explicit runtime service, never canonical scope source | P3 | resource-mode override never changes project identity result |
| INF-03 | bounded runtime | `.../bounded.rs:18` | `RESOURCE_MODE_LAST_OBSERVED` | global observation memory | infra-only | P3 | canonical packets don’t vary due to prior mode observation |
| INF-04 | bounded runtime | `.../bounded.rs:20` | `RESOURCE_MODE_TRANSITIONS` | transition ring global | infra-only | P3 | no canonical route consumes transition ring as authority |
| INF-05 | bounded runtime | `.../bounded.rs:22` | `RESOURCE_MODE_TRANSITION_OMITTED` | global omitted-counter | infra-only | P3 | omitted-counter never affects scope outcome |
| INF-06 | bounded runtime | `.../bounded.rs:23` | `RESOURCE_MODE_HYSTERESIS_STATE` | process-global hysteresis state | infra-only runtime service | P3 | scope results independent of hysteresis history |
| INF-07 | bounded runtime | `.../bounded.rs:142,144,145` | pressure/response-size globals | pressure history can taint prompts if not quarantined | infra-only telemetry service | P3 | awareness/warnings may vary, canonical authority may not |
| INF-08 | Context retrieval model cache | `crates/focusa-core/src/runtime/context_retrieval.rs:31` | `FASTEMBED_MODEL` caches one immutable embedding model behind a mutex | process-local model reuse could be mistaken for Context authority | keep as infra-only compute cache; canonical Context and retrieval evidence remain exact-scope reducer state | P3 | alternating project scopes may reuse model weights but never vectors, sources, claims, or retrieval authority |
| INF-09 | Generated capability registries | `crates/focusa-api/src/routes/agent_capabilities.rs` | immutable `include_str!` adapter/Spec141 capability and Agent Card registries | immutable metadata could be mistaken for mutable capability authority | keep generated values immutable and process-shared; permission, scope, execution, receipt, and mutation authority remain request-scoped | P3 | alternating scopes see identical metadata while every capability execution remains exact-scope and permission checked |
| INF-10 | Generated MCP projection | `crates/focusa-api/src/routes/mcp.rs` | immutable `include_str!` MCP tool projection | projection cache could be mistaken for call authority | keep projection immutable and process-shared; each MCP call routes through scoped REST auth/permission/idempotency/receipt enforcement | P3 | projection equality across scopes does not permit cross-scope action or state reuse |
| INF-11 | Temporal fallback clock | `crates/focusa-core/src/temporal_clock.rs` | `MONOTONIC_ORIGIN` is a process-local immutable origin used only by non-Unix fallback sampling | process-local clock origin could be mistaken for temporal/project authority | keep as infra-only clock calibration; canonical temporal claims remain evidence/scoped | P3 | cross-platform clock compilation and Spec104 inventory prove no scope or instruction authority |
| INF-12 | Entitlement route metadata and denial guidance | `crates/focusa-api/src/middleware/entitlement.rs` | `ENTITLEMENT_METADATA` and `GUIDANCE` cache validated immutable embedded contracts | immutable metadata cache could be mistaken for mutable entitlement authority | keep process-shared metadata immutable; every decision still consumes the request's signed entitlement snapshot and exact route | P3 | alternating scopes reuse metadata but produce decisions only from their own signed snapshot and route |
| INF-13 | Compaction policy controller store lock | `crates/focusa-api/src/routes/compaction_policy_store.rs` | `STORE_LOCK` serializes process-local file-store access | synchronization primitive could be mistaken for shared policy authority | retain lock only for atomic I/O; scoped records and receipts remain the authority | P3 | concurrent scopes cannot read or overwrite each other's controller record |
| INF-14 | Compaction policy status store lock | `crates/focusa-api/src/routes/compaction_policy.rs` | `STORE_LOCK` serializes process-local policy status persistence | synchronization primitive could be mistaken for current-scope authority | retain lock only for atomic I/O; typed scope key remains mandatory for every policy record | P3 | concurrent scope writes remain isolated despite sharing the lock |
| INF-15 | Embedded entitlement policy registry | `crates/focusa-license/src/entitlement_policy.rs` | `REGISTRY` caches one digest-validated immutable policy artifact | process cache could be mistaken for a mutable grant source | keep immutable registry process-shared; signed snapshots and authority sequence remain request-specific | P3 | registry reuse cannot create, extend, or transfer an entitlement |
| INF-16 | Embedded denial UX catalog | `crates/focusa-license/src/denial_ux.rs` | `CATALOG` caches one immutable presenter catalog | UX metadata could be mistaken for denial/grant authority | keep catalog presentation-only and fail closed on unknown codes; entitlement decision remains upstream | P3 | catalog reuse only changes safe wording, never authorization outcome |

### C. Pi extension runtime singleton hub

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| PI-01 | Pi runtime state | `apps/pi-extension/src/state.ts:180+` | `export const S = { ... }` is the adapter-global mutable hub | canonical behavior can depend on hidden prior session/project state | split into typed scoped runtime stores/services | P0 | new session in broad cwd cannot inherit prior project/workpoint/trajectory from singleton `S` |
| PI-02 | root resolution | `state.ts:184` | `lastProjectRootResolution` kept in global `S` | prior root bias affects later project adoption | typed scoped resolution object, not singleton shadow | P0 | project-root resolution depends only on current evidence, not previous session |
| PI-03 | active workpoint | `state.ts:241-242` | `activeWorkpointPacket` / `activeWorkpointSummary` in singleton memory | immediate continuation authority lives outside scoped durable model | scoped workpoint packet/cache only | P0 | wrong-scope resume packet cannot appear after session switch |
| PI-04 | trajectory shadow | `state.ts:243` | `lastTrajectoryClarity` in singleton memory | trajectory guidance can bleed across roots or be mistaken for canonical trajectory authority | scope-keyed trajectory cache/read model; advisory only until verified typed root/scope match | P0 | trajectory for root A never shows when root B active; advisory trajectory never overrides Workpoint authority |
| PI-05 | identity shadow | `state.ts:244-245` | `lastProjectIdentity` / `lastProjectVerify` in singleton memory | hidden prior identity reused by tool calls and awareness | typed scoped identity cache only | P0 | prior verified identity cannot influence mismatched cwd/session |
| PI-06 | report/evidence shadow | `state.ts:246` | `latestReportSummary` cached globally | stale report handle can masquerade as current truth | scoped evidence handle cache | P1 | report handle only rendered when scope match holds |
| PI-07 | project switch memory | `state.ts:259` | `projectSwitchLedger` in singleton state | advisory history can drift into authority | keep advisory only; never authoritative | P1 | project-switch ledger cannot cause auto-adoption |
| PI-08 | remembered root file | `state.ts:2060` | `~/.pi/agent/focusa-project-root.json` one-slot root cache | one remembered root acts like implicit binding | remove as authority; explicit typed scope/profile if retained | P0 | starting Pi in `/root` after working in Focusa never auto-adopts Focusa |
| PI-09 | root remember helper | `state.ts:2096` | `rememberProjectRoot(...)` writes singleton root | singleton carryover path | remove / replace with non-authority convenience profile | P0 | no write to one-slot remembered root on normal startup |
| PI-10 | root adoption | `state.ts:2169-2173` | `adoptPiProjectRoot(...)` sets `S.sessionCwd`, updates resolution, calls remember | central auto-adoption path | explicit typed scope adoption only | P0 | adoption requires verified scope match, not prior remembered state |
| PI-11 | session identity builder | `state.ts:2209` | `buildFocusaSessionIdentity(...)` builds request identity from local state + persisted hints | every `focusa_*` tool can inherit stale authority | typed `ScopeRef` + `SessionIdentityEnvelope` independent of stale singleton fields | P1 | tool requests include correct scope only when explicit typed scope exists |
| PI-12 | persisted project hint injection | `state.ts:2228` | appends `persisted_project_root` into identity query | hidden prior root influences current request | no singleton persisted-root injection | P0 | project identity tool cannot silently send prior root |
| PI-13 | continuity restore | `state.ts:2379+` | `adoptPersistedContinuityForSession(...)` restores continuity + workpoint packet | prior session/workpoint authority can return; `continuity_id` may shift across Pi sessions and must not become sole restore authority | strict typed root/scope + session/workstream match; `continuity_id` only secondary under verified root | P0 | persisted continuity ignored on scope/session mismatch; changed `continuity_id` alone never rebinds canonical scope |
| PI-14 | frame recovery | `state.ts:2450+` | `adoptWorkpointScopeForFrameRecovery(...)` can recover scope from packet | packet-driven fallback can recreate stale active authority | typed packet scope only, with blocked mismatch | P1 | frame recovery blocks on mismatched root/continuity/session |
| PI-15 | local persisted bridge state | `state.ts:2654+,2703` | `persistState()` + `appendEntry("focusa-state", payload)` | local bridge log can be replayed as hidden authority | adapter cache only, never canonical truth | P1 | replayed local state cannot override daemon-scoped canonical packets |

### D. Pi extension consumer / dependency files

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| PI-C01 | session bootstrap | `apps/pi-extension/src/session.ts:396` | builds `session_identity` from `buildFocusaSessionIdentity(...)` | startup inherits stale scope if builder stale | bootstrap from typed scoped authority only | P1 | session start from cold state creates correct typed identity or blocks |
| PI-C02 | session lifecycle | `session.ts:557,617,807,851` | resets scoped state, then re-adopts persisted continuity/workpoint | resume path can resurrect old authority | resume only from canonical typed packets | P1 | switching session/frame never revives prior workpoint without exact scope match |
| PI-C03 | restore logic | `session.ts:600+` | restores `lastProjectIdentity`, `lastTrajectoryClarity`, etc. into `S` | old state shadows current session; trajectory shadow may reappear under changed `continuity_id` | scoped restore pipeline only; no continuity-only trajectory restore | P1 | post-reload scope always derived from current packet/identity, not singleton shadow; changed `continuity_id` alone never restores stale trajectory |
| PI-C04 | awareness | `apps/pi-extension/src/awareness.ts:25-35,45` | renders from `S.sessionCwd`, `S.continuityId`, `S.lastProjectRootResolution`, `S.lastTrajectoryClarity`, `S.lastProjectIdentity`, `S.lastProjectVerify`, `S.activeWorkpointSummary` | awareness can surface stale authority | awareness renders from typed scoped packets/read models only | P1 | awareness card on mismatch shows blocked scope, not stale project |
| PI-C05 | compaction | `apps/pi-extension/src/compaction.ts:104,108,125,126,129,138` | compaction instructions inject `project_root`, `continuityId`, `lastTrajectoryClarity`, scope verdict from singleton state | compaction/recovery may anchor on stale singleton shadow | compaction derived from canonical typed resume packets + explicit blocked envelopes | P1 | compaction after scope switch never emits prior project root |
| PI-C06 | polish/reports | `apps/pi-extension/src/polish.ts:110` | references `S.activeWorkpointPacket` for report metadata | report can bind to stale workpoint | derive from scoped workpoint packet only | P2 | final report metadata absent unless scoped workpoint valid |
| PI-C07 | config/prompts | `apps/pi-extension/src/config.ts:304` | prompt surfaces configurable via `project_root,project_verify,workpoint,trajectory` labels | prompt composition can remain stringly-typed | typed prompt/packet surface identifiers | P2 | static audit rejects missing typed scope/authority fields in injected surfaces |

### E. Pi `focusa_*` tool layer

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| TOOL-01 | persisted identity bridge | `apps/pi-extension/src/tools.ts:1214+` | `persistedProjectIdentityFields()` reads `S.lastProjectIdentity` | prior identity injected into current tool calls | remove singleton-based persisted-identity authority path | P0 | project identity tool call contains no hidden prior root |
| TOOL-02 | persisted identity query | `tools.ts:1228` | `appendPersistedProjectIdentityQuery(...)` mutates query from singleton state | stale identity silently biases API | explicit typed scope/session envelope only | P0 | mismatch test proves no auto-appended persisted scope |
| TOOL-03 | project identity tool | `tools.ts:2988+` | `focusa_project_identity` uses `S.sessionCwd`, auto-appends persisted fields | tool runtime inherits singleton state | tool consumes typed `ScopeRef`/identity envelope only | P1 | `focusa_project_identity` from broad cwd returns blocked, no stale project |
| TOOL-04 | query injection callsite | `tools.ts:3013` | persisted identity appended for identity request | stale scope injection | remove auto-append | P0 | identity query contains only explicit fields |
| TOOL-05 | query injection callsite | `tools.ts:3117` | persisted identity appended for verify request | stale scope injection | remove auto-append | P0 | verify query contains only explicit fields |
| TOOL-06 | session identity body builder | `tools.ts:3577,3644,3672,3701,3727,3787,3870,3955,4067,4106,7312` | multiple tool bodies attach `session_identity` built from singleton-derived state | many `focusa_*` tools can inherit stale scope | central typed scope/session builder with no singleton authority | P1 | all canonical-capable tool calls fail closed on missing/mismatched scope |
| TOOL-07 | tool contract registry | `apps/pi-extension/src/tool-contracts.ts:549+` and broader registry | contracts describe tools but don’t alone guarantee singleton-free runtime scope | docs/contracts can drift from runtime behavior | require explicit scope + authority fields in contract schema for canonical tools | P2 | static contract test fails if canonical-capable tool omits scope/authority |
| TOOL-08 | agent prompt tool | `apps/pi-extension/src/tools.ts:2869`, `apps/pi-extension/src/tool-contracts.ts:549` | `focusa_agent_prompt` returns bootstrap guidance from plugin/runtime state | bootstrap guidance can surface stale scope, stale identity, or singleton-shadow authority | `focusa_agent_prompt` derives only from typed scoped bootstrap packets and blocked/advisory envelopes | P1 | broad cwd or mismatched scope makes `focusa_agent_prompt` return blocked/advisory scope, never stale project guidance |
| TOOL-09 | utility card / post-compaction card tool | `apps/pi-extension/src/tools.ts:2891`, `apps/pi-extension/src/tool-contracts.ts:1589`, `crates/focusa-core/src/utility_card.rs` | `focusa_utility_card` renders bootstrap and post-compaction cards | post-compaction guidance can revive stale root/continuity/trajectory shadows | utility-card output derives only from typed bootstrap/resume packets and explicit blocked envelopes | P1 | post-compaction card after scope switch never emits prior project root or stale trajectory authority |
| TOOL-10 | Pi plugin prompt/command export surfaces | `apps/pi-extension/src/commands.ts`, `apps/pi-extension/src/config.ts`, `apps/pi-extension/src/tools.ts` | plugin command/prompt surfaces export context through stringly labels and runtime helpers | MCP or other external bridge consumers can receive singleton-derived scope without typed authority envelope | exported prompt/context/handoff payloads require typed scope/authority or explicit non-canonical marking | P2 | static/live audit proves Pi-plugin-exported MCP/handoff surfaces never emit singleton-derived scope |
| TOOL-11 | Pi session transfer / rollover bridge | `apps/pi-extension/src/tools.ts`, `crates/focusa-api/src/routes/project.rs` | session transfer previously inferred continuity from process/project shadows | static continuity can cross projects or impersonate target attachment authority | require explicit typed source/target `WorkstreamKey`, target session/workpoint refs, and verified transition receipt | P0 | rotating-continuity test proves no derived continuity and target resume must be verified |

### F. Menubar Mac app surface

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| MEN-01 | project context helper | `apps/menubar/src/lib/projectContext.svelte.ts:1-29` | extracts `projectRoot`, `continuityId`, `sessionId`, `workItemId` by fishing through mixed snapshot fields | mixed-source fallback can reassemble stale/ambiguous scope | consume one typed `ScopeContext` object | P2 | helper returns typed scope only from canonical scoped packet |
| MEN-02 | page bootstrap | `apps/menubar/src/routes/+page.svelte:43-54` | derives root/continuity from `projectIdentityRecord`, `projectIdentityRaw`, `activeWorkpointRecord`, `state.session.*` | ambiguous fallback precedence | one typed source of truth | P2 | page load under mismatch shows blocked state, not merged fallback |
| MEN-03 | API bridge | `apps/menubar/src/lib/api.ts:211-229` | builds requests with `session_frame_key`, `project_root`, `cwd: ctx.projectRoot`, `workspace_id: ctx.continuityId || ctx.projectRoot || 'menubar'`, `continuity_id`, `pi_session_id` | overloaded `workspace_id`; fake `cwd`; stringly-typed scope; risk of `continuity_id` acting like scope authority | typed scope envelope; no `cwd=project_root` proxy; `workspace_id` and `continuity_id` never treated as root authority aliases | P2 | menubar request payload validated against typed scope schema; changing `continuity_id` alone does not retarget canonical scope |
| MEN-04 | body/context merge | `apps/menubar/src/lib/api.ts:246-248` | merges body scope with ctx scope | hidden precedence / ambiguity | explicit precedence rules in typed envelope | P2 | body/ctx mismatch yields blocked request, not silent merge |
| MEN-05 | workpoint action layer | `apps/menubar/src/lib/actions/workpointActions.svelte.ts:92,106,133` | forwards `continuity_id` and scope strings | action layer tied to current string scope model | typed `ScopeContext` prop contract | P2 | action calls carry typed scope and reject mismatches |
| MEN-06 | WorkLoop peek | `apps/menubar/src/lib/components/WorkLoopPeek.svelte:33-36,65` | compares loop project root to current project root | current root derived from fallback strings | typed scope comparison only | P2 | stale loop authority flagged correctly from typed scope mismatch |
| MEN-07 | Workpoint peek | `apps/menubar/src/lib/components/WorkpointPeek.svelte:33-36,58,114-115,135,148` | consumes root/continuity/session/work item from current helper and packet | depends on mixed-source helper | typed workpoint scope object | P2 | component renders only when typed scope valid |
| MEN-08 | Trajectory/Cockpit/Context peeks | `TrajectoryPeek.svelte:48`, `CockpitView.svelte:64,80,110`, `ContextCognitionPeek.svelte:10-13,44-50` | reads `project_root`, `continuity_id`, even hardcoded sample query (`/v1/context-cognition?project_root=/home/wirebot/focusa`) | UI examples and views can encode stringly or hardcoded scope assumptions | typed scope props and dynamic verified scope only | P2 | no hardcoded root remains in menubar components/tests |

### G. Awareness / external adapter surface

| ID | Surface | File:line | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---:|---|---|---|---|---|
| AWA-01 | adapter config | `apps/focusa-awareness/index.ts:19-20` | raw `projectRoot?` / `continuityId?` config | loose string scope config | typed bootstrap packet input | P2 | adapter startup validates typed scope schema |
| AWA-02 | hardcoded default root | `apps/focusa-awareness/index.ts:33` | default `projectRoot` hardcoded to `/data/wirebot/users/verious` | hidden authority default | remove hardcoded project root | P1 | startup without explicit scope yields advisory/blocked card, not default root |
| AWA-03 | ambient continuity env | `apps/focusa-awareness/index.ts:34` | default `continuityId` from env | hidden ambient scope; risk of `continuity_id` being treated as sufficient identity | explicit typed session/workstream packet under verified root/scope | P1 | env-only continuity cannot create or switch canonical scope |
| AWA-04 | prompt rendering | `apps/focusa-awareness/index.ts:51` | emits `project_root` / `continuity_id` strings into prepended context | stringly scope display from raw config | render from typed scope envelope | P2 | prepended context matches canonical typed scope or warns blocked |
| AWA-05 | request builder | `apps/focusa-awareness/index.ts:67-73,103+` | sends `workspace_id`, `session_id`, `project_root`, optional `continuity_id`; `before_agent_start` injects prompt | loose/overloaded scope wiring | typed bootstrap/awareness request envelope | P2 | adapter requests rejected on missing/ambiguous typed scope |

### H. Core / persistence / model alignment dependencies

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| CORE-01 | scope types | `focusa-core` types / reducer / persistence surfaces | many good `project_root + continuity_id` patterns exist, but not universal typed `ScopeKind` / `ScopeRef` | APIs/adapters keep reinventing local string scope models | universal typed scope family shared across Rust + TS/JSON | P1 | static tests prove packet/type parity across core/API/Pi |
| CORE-02 | host scope gap | project-root-centric model | no first-class host scope for legitimate server-wide work | pressure to weaken broad-root safety | explicit `host` scope kind, separate from project scope | P1 | `/root` remains blocked as project scope, accepted only through explicit host scope path |

### I. Root / host binding remediation surfaces

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| HOST-01 | broad-root safety gate | `crates/focusa-cli/src/commands/scope.rs:3-23` | `/root` and other broad roots are blocked for project scope | pressure to weaken safety gate via bypass flag | preserve project gate; add separate host bind path | P1 | `/root` remains blocked as project scope while host bind route is available |
| HOST-02 | project CLI path | `crates/focusa-cli/src/commands/project.rs:197-204` | project commands validate `cwd`, `project_root`, `persisted_project_root` only | no first-class host binding path; temptation to overload project commands | add explicit host identity/bind/verify command family or typed scope argument | P1 | CLI can bind/verify host scope without ever treating `/root` as project root |
| HOST-03 | Pi broad-cwd startup | `apps/pi-extension/src/session.ts:634-638`, `apps/pi-extension/src/state.ts:2060,2169-2173` | broad cwd drives project-root prompt/remembered-root behavior | Pi may keep reaching for project-root memory when operator really wants server-wide host scope | add explicit host bootstrap branch and separate host cache/profile path | P1 | starting Pi in `/root` can enter host-scope workflow without project-root cache pollution |
| HOST-04 | menubar scope bridge | `apps/menubar/src/lib/api.ts:211-229`, `apps/menubar/src/lib/projectContext.svelte.ts:1-29` | menubar assumes project-root-centric scope fields | no first-class host-scope packet handling | add typed `scope_kind=host` handling in menubar request/packet model | P2 | menubar can display and act on host scope without fabricating project_root semantics |
| HOST-05 | awareness/bootstrap adapter | `apps/focusa-awareness/index.ts:33-34,67-73` | adapter config assumes `projectRoot` + `continuityId` string inputs | no typed host bootstrap path; hardcoded root defaults are unsafe | add host-aware typed bootstrap request/packet shape | P2 | adapter can surface server-wide host context without hardcoded project-root defaults |
| HOST-06 | core scope model | `focusa-core` scope types + reducer/persistence surfaces | current model is project-root-centric | host work risks being forced through project-root abstractions | add first-class `ScopeKind=Host` and host identity semantics | P1 | host-scoped state persists and restores separately from project-scoped state |

### J. Bench / eval / proof dependency surfaces

| ID | Surface | File / spec | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| BEN-01 | implementation order | `docs/current/NEW_SPECS_IMPLEMENTATION_ORDER.md` | 109–114 rollout depends on later waves | benchmark rollout may land on top of stale singleton behavior | Spec 104 gates benchmark trust surfaces | P3 | no benchmark/promotion proof accepted until singleton contamination tests pass |
| BEN-02 | benchmark integrity | Specs 113/114 | ON/OFF benchmark promise depends on no hidden bleed | prior session/root/project state contaminates arms | typed run scope + arm scope + proof lineage | P3 | repeated ON/OFF runs from clean starts produce reproducible isolated results |
| BEN-03 | public proof | Spec 114 proof surfaces | public-safe proof can be derived from contaminated internal state | false public claims with impure lineage | immutable typed proof lineage from scoped eval ledger | P3 | every public benchmark claim links to typed run/proof snapshot lineage |

### K. Trajectory-specific hardening requirements

| ID | Surface | File / spec | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| TRAJ-01 | HLT scoping | `docs/102-trajectory-ladder-consolidated-spec.md:44-58` | HLT is documented as root-scoped, not continuity-scoped | implementation may still drift toward continuity-shaped restore or regeneration | preserve HLT as root-scoped authority under typed scope model | P1 | changing `continuity_id` under same verified root does not regenerate or replace HLT |
| TRAJ-02 | generic bootstrap HLT | `docs/102-trajectory-ladder-consolidated-spec.md:12-20` | generic bootstrap HLT is explicitly degraded placeholder | generic text can mislead agents if surfaced as authority | keep placeholder degraded; require valid historical/operator-defined HLT for canonical trajectory authority | P1 | generic bootstrap HLT always carries degraded flags and never satisfies long-term-goal authority gates |
| TRAJ-03 | projection semantics | `docs/96-trajectory-projection-and-daemon-stability-spec.md:48-66` | trajectory projection is a derived navigation view over existing primitives | projection layer can be mistaken for canonical execution authority | preserve trajectory as derived/advisory; Workpoint stays immediate authority | P1 | trajectory route guidance cannot mutate execution authority without Workpoint path |
| TRAJ-04 | Pi trajectory shadow restore | `apps/pi-extension/src/state.ts:243`, `apps/pi-extension/src/session.ts:606-610` | Pi restores `lastTrajectoryClarity` into singleton shadow state | stale trajectory may survive scope/continuity churn | scoped advisory cache only; no singleton restore authority | P0 | restarted Pi session with changed continuity or root never reuses stale trajectory as canonical truth |

### L. Focus State / Focus Stack / Work-loop authority surfaces

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| FS-01 | Focus State reducer/update | `crates/focusa-api/src/routes/focus.rs:1-26`, `apps/pi-extension/src/tool-contracts.ts:370-585` | Focus stack routes expose `POST /v1/focus/update` and multiple FocusState reducer/update tool surfaces | focus updates can target wrong project/host scope if scope is implicit or stale | every focus update/push/pop/set-active operation requires typed scope and preserves host/project distinction | P0 | Focus State writes in host scope never mutate project scope and vice versa |
| FS-02 | Focus Stack active frame | `crates/focusa-api/src/routes/focus.rs:1-26`, `docs/98-project-root-crdt-reconciliation-foundation-spec.md:208` | active frame/stack behavior is part of scoped state model but not yet explicitly remediated in Annex A | stale active frame can become hidden continuation authority | active frame ids exist only inside typed scoped state and never in adapter-global shadow memory | P0 | active frame restore after reload stays within exact typed scope or blocks |
| WL-01 | Work-loop writer control | `crates/focusa-api/src/routes/work_loop.rs:1-27`, `apps/pi-extension/src/tool-contracts.ts:597-806` | work-loop writer/current-task control exists as its own family | writer ownership or current task can cross host/project boundary if not explicitly scope-bound | writer, current task, checkpoints, and status are keyed by typed scope/workstream and reject cross-scope resumes | P0 | work-loop writer for host scope cannot control project-scoped loop and vice versa |
| WL-02 | Work-loop consumer surfaces | `crates/focusa-tui/src/main.rs:52-58`, `apps/menubar/src/lib/components/WorkLoopPeek.svelte:33-36,65` | read surfaces expose Work-loop state but do not yet have explicit Annex A remediation | stale loop state can be displayed as current authority | consumers display typed scope and advisory/blocked state explicitly | P2 | TUI/menubar show blocked or stale loop mismatch instead of stale current task |

### M. API middleware and route-family scope carriers

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| MW-01 | auth middleware | `crates/focusa-api/src/middleware/auth.rs` | auth layer sits in front of canonical scope-carrying routes | middleware could authenticate while dropping or distorting typed scope | auth preserves typed scope fields and never invents canonical scope | P1 | authenticated requests with host/project scope preserve exact scope fields end-to-end |
| MW-02 | error envelope middleware | `crates/focusa-api/src/middleware/error_envelope.rs` | error envelopes wrap route failures | blocked/mismatch scope details may be lost or flattened | error envelopes preserve typed blocked/advisory/canonical scope semantics | P1 | scope mismatch errors return machine-readable blocked envelopes with intact scope metadata |
| MW-03 | JSON guard / payload middleware | `crates/focusa-api/src/middleware/json_guard.rs` | request validation sits in front of typed payloads | malformed/partial scope payloads may degrade into ambiguous state | JSON guard rejects malformed scope packets strictly | P1 | malformed `scope_kind=host/project` payloads fail before route execution |
| MW-04 | scope-carrying route families | `crates/focusa-api/src/routes/awareness.rs`, `call_stack.rs`, `commands.rs`, `context_cognition.rs`, `dxux.rs`, `ecs.rs`, `health.rs`, `ontology.rs`, `proposals.rs`, `proxy.rs`, `reflex.rs`, `session.rs`, `sync.rs`, `traverse.rs`, `visual_workflow.rs`, `utility.rs` | many route families carry or render scope but were only inventoried, not remediated | hidden stringly scope behavior can survive outside high-profile routes | every scope-carrying route family must accept/preserve typed host/project scope or return blocked | P2 | surface sweep proves no route family drops `scope_kind` or reconstructs implicit scope |

### N. TUI and native menubar bridge surfaces

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| TUI-01 | focusa-tui API/view layer | `crates/focusa-tui/src/api.rs:1-26`, `crates/focusa-tui/src/main.rs:52-58`, `crates/focusa-tui/src/views/focus_state.rs` | TUI consumes `/v1/state/dump` and presents FocusState / FocusStack / WorkLoop tabs | read-only UI can still present stale or mis-scoped authority if scope is not explicit | TUI renders typed scope, host/project distinction, and blocked mismatch state explicitly | P2 | TUI opened on mismatched scope shows blocked/advisory state, never stale current project |
| MBN-01 | menubar native Tauri bridge | `apps/menubar/src-tauri/src/main.rs:19-27,130,167,198` | native bridge holds `BRIDGE_COMPLETIONS` and `BRIDGE_LISTENERS` in global memory | native bridge state may become unclassified singleton residue and hide scope bugs | classify as infra or refactor; any scope-bearing bridge messages must preserve typed scope and host awareness | P2 | native bridge callbacks/listeners cannot leak or coerce host/project scope |

### O. Tests / docs / contract enforcement surfaces

| ID | Surface | File / area | Current behavior | Failure mode | Target design | Phase | Acceptance test |
|---|---|---|---|---|---|---|---|
| DOC-01 | singleton audit | `tests/spec104_deep_focusa_surface_sweep.py` | existing spec104 test number exists but doc was missing | no repo-wide anti-singleton enforcement yet | make it a hard static/live singleton-surface sweep | P3 | test fails on new authority-bearing global |
| DOC-02 | session identity tests | `tests/spec96_session_identity_envelope_static_test.sh` | identity envelope exists but not full de-singletonization | envelopes can still sit atop singleton state | extend to require no singleton authority dependencies | P3 | test fails if packets omit typed scope/authority |
| DOC-03 | mismatch semantics | `tests/spec102_project_identity_mismatch_semantics_test.sh` | mismatch semantics exist | stale helper layers may still auto-adopt | extend to Pi/tools/menubar/adapters/TUI | P3 | mismatch test covers all surfaces |
| DOC-04 | tool contracts | `apps/pi-extension/src/tool-contracts.ts` | contract docs exist | runtime may drift from contract | add scope/authority contract checks for canonical tools | P3 | static contract audit blocks missing fields |
| DOC-05 | scope hard stop | `docs/current/CROSS_PROJECT_SCOPE_HARD_STOP.md` | hard-stop intent exists | implementation may still use singleton shadows | align hard-stop doc + runtime + tests | P3 | live scope-conflict path matches documented blocked envelope |

### 12.1 Annex B — Scope-carrying surface inventory

The following files were observed carrying scope, scope-adjacent authority, or typed scope payload fields and are therefore explicitly in scope for Spec 104. This annex exists so no surface is silently omitted.

### B.1 Apps — awareness / adapters / menubar / Pi

#### Awareness / adapters

- `apps/focusa-awareness/index.ts`

#### Menubar Svelte/UI

- `apps/menubar/src/app.d.ts`
- `apps/menubar/src/lib/actions/workpointActions.svelte.ts`
- `apps/menubar/src/lib/api.ts`
- `apps/menubar/src/lib/projectContext.svelte.ts`
- `apps/menubar/src/lib/stores/diagnostics.svelte.ts`
- `apps/menubar/src/lib/stores/focus-canvas.svelte.ts`
- `apps/menubar/src/lib/stores/focus.svelte.ts`
- `apps/menubar/src/lib/stores/gate.svelte.ts`
- `apps/menubar/src/lib/stores/pairing.svelte.ts`
- `apps/menubar/src/lib/stores/runtime.svelte.ts`
- `apps/menubar/src/lib/stores/toast.svelte.ts`
- `apps/menubar/src/lib/types/focus-canvas.ts`
- `apps/menubar/src/lib/types/focus.ts`
- `apps/menubar/src/routes/+layout.ts`
- `apps/menubar/src/lib/canvas/AsccPanel.svelte`
- `apps/menubar/src/lib/canvas/FocusCanvas.svelte`
- `apps/menubar/src/lib/canvas/Timeline.svelte`
- `apps/menubar/src/lib/components/AddPeerModal.svelte`
- `apps/menubar/src/lib/components/CockpitView.svelte`
- `apps/menubar/src/lib/components/ContextCognitionPeek.svelte`
- `apps/menubar/src/lib/components/FirstRunWizard.svelte`
- `apps/menubar/src/lib/components/FocusView.svelte`
- `apps/menubar/src/lib/components/GatePanel.svelte`
- `apps/menubar/src/lib/components/PairingPanel.svelte`
- `apps/menubar/src/lib/components/ProofPeek.svelte`
- `apps/menubar/src/lib/components/QRCode.svelte`
- `apps/menubar/src/lib/components/Settings.svelte`
- `apps/menubar/src/lib/components/SyncPanel.svelte`
- `apps/menubar/src/lib/components/ToastContainer.svelte`
- `apps/menubar/src/lib/components/ToolsRegistryPeek.svelte`
- `apps/menubar/src/lib/components/TrajectoryPeek.svelte`
- `apps/menubar/src/lib/components/WorkLoopPeek.svelte`
- `apps/menubar/src/lib/components/WorkpointPeek.svelte`
- `apps/menubar/src/routes/+layout.svelte`
- `apps/menubar/src/routes/+page.svelte`
- `apps/menubar/src/routes/canvas/+page.svelte`

#### Menubar native / Tauri bridge

- `apps/menubar/src-tauri/src/main.rs`

#### Pi extension runtime + tool layer

- `apps/pi-extension/src/awareness-substrate.ts`
- `apps/pi-extension/src/awareness.ts`
- `apps/pi-extension/src/commands.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/src/config.ts`
- `apps/pi-extension/src/polish.ts`
- `apps/pi-extension/src/session.ts`
- `apps/pi-extension/src/state.ts`
- `apps/pi-extension/src/tool-contracts.ts`
- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/wbm.ts`
- `apps/pi-extension/src/index.ts`
- `docs/focusa-tools/tools/focusa_agent_prompt.md`
- `docs/focusa-tools/tools/focusa_utility_card.md`

### B.2 API — middleware and route families

#### Middleware

- `crates/focusa-api/src/middleware/error_envelope.rs`
- `crates/focusa-api/src/middleware/json_guard.rs`
- `crates/focusa-api/src/middleware/rate_limit.rs`
- `crates/focusa-api/src/middleware/mod.rs`
- `crates/focusa-api/src/middleware/route_scope.rs`
- `crates/focusa-api/src/middleware/auth.rs`

#### Route families

- `crates/focusa-api/src/routes/agent_capabilities.rs`
- `crates/focusa-api/src/routes/agent_reminder.rs`
- `crates/focusa-api/src/routes/ascc.rs`
- `crates/focusa-api/src/routes/attachments.rs`
- `crates/focusa-api/src/routes/autonomy.rs`
- `crates/focusa-api/src/routes/awareness.rs`
- `crates/focusa-api/src/routes/bloatgaurd.rs`
- `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- `crates/focusa-api/src/routes/bounded.rs`
- `crates/focusa-api/src/routes/call_stack.rs`
- `crates/focusa-api/src/routes/capabilities.rs`
- `crates/focusa-api/src/routes/capabilities_extra.rs`
- `crates/focusa-api/src/routes/clt.rs`
- `crates/focusa-api/src/routes/commands.rs`
- `crates/focusa-api/src/routes/compaction.rs`
- `crates/focusa-api/src/routes/constitution.rs`
- `crates/focusa-api/src/routes/context_cognition.rs`
- `crates/focusa-api/src/routes/deck.rs`
- `crates/focusa-api/src/routes/device_pairing.rs`
- `crates/focusa-api/src/routes/dxux.rs`
- `crates/focusa-api/src/routes/ecs.rs`
- `crates/focusa-api/src/routes/env.rs`
- `crates/focusa-api/src/routes/events.rs`
- `crates/focusa-api/src/routes/events_sqlite.rs`
- `crates/focusa-api/src/routes/events_stream.rs`
- `crates/focusa-api/src/routes/focus.rs`
- `crates/focusa-api/src/routes/gate.rs`
- `crates/focusa-api/src/routes/health.rs`
- `crates/focusa-api/src/routes/info.rs`
- `crates/focusa-api/src/routes/instances.rs`
- `crates/focusa-api/src/routes/llms_txt.rs`
- `crates/focusa-api/src/routes/license.rs`
- `crates/focusa-api/src/routes/memory.rs`
- `crates/focusa-api/src/routes/metacognition.rs`
- `crates/focusa-api/src/routes/mcp.rs`
- `crates/focusa-api/src/routes/ontology.rs`
- `crates/focusa-api/src/routes/pairing_store.rs`
- `crates/focusa-api/src/routes/device_pairing.rs`
- `crates/focusa-api/src/routes/permissions.rs`
- `crates/focusa-api/src/routes/predictions.rs`
- `crates/focusa-api/src/routes/preload.rs`
- `crates/focusa-api/src/routes/project.rs`
- `crates/focusa-api/src/routes/proposals.rs`
- `crates/focusa-api/src/routes/proxy.rs`
- `crates/focusa-api/src/routes/reflection.rs`
- `crates/focusa-api/src/routes/reflex.rs`
- `crates/focusa-api/src/routes/release.rs`
- `crates/focusa-api/src/routes/resource.rs`
- `crates/focusa-api/src/routes/rfm.rs`
- `crates/focusa-api/src/routes/session.rs`
- `crates/focusa-api/src/routes/skills.rs`
- `crates/focusa-api/src/routes/snapshots.rs`
- `crates/focusa-api/src/routes/sse.rs`
- `crates/focusa-api/src/routes/subagent.rs`
- `crates/focusa-api/src/routes/sync.rs`
- `crates/focusa-api/src/routes/sync_receive.rs`
- `crates/focusa-api/src/routes/sync_transfer.rs`
- `crates/focusa-api/src/routes/telemetry.rs`
- `crates/focusa-api/src/routes/threads.rs`
- `crates/focusa-api/src/routes/tokens.rs`
- `crates/focusa-api/src/routes/training.rs`
- `crates/focusa-api/src/routes/trajectory.rs`
- `crates/focusa-api/src/routes/traverse.rs`
- `crates/focusa-api/src/routes/trust.rs`
- `crates/focusa-api/src/routes/turn.rs`
- `crates/focusa-api/src/routes/turn_recent.rs`
- `crates/focusa-api/src/routes/utility.rs`
- `crates/focusa-api/src/routes/update.rs`
- `crates/focusa-api/src/routes/uxp.rs`
- `crates/focusa-api/src/routes/visual_workflow.rs`
- `crates/focusa-api/src/routes/work_loop.rs`
- `crates/focusa-api/src/routes/work_items.rs`
- `crates/focusa-api/src/routes/workpoint.rs`
- `crates/focusa-api/src/routes/mod.rs`


#### Post-baseline API routes

- `crates/focusa-api/src/routes/agent_runtime.rs`
- `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- `crates/focusa-api/src/routes/agent_runtime_migration.rs`
- `crates/focusa-api/src/routes/agent_runtime_studio.rs`
- `crates/focusa-api/src/routes/agent_runtime_tests.rs`
- `crates/focusa-api/src/routes/browser_interop.rs`
- `crates/focusa-api/src/routes/context_claims.rs`
- `crates/focusa-api/src/routes/context_sources.rs`
- `crates/focusa-api/src/routes/interview_sessions.rs`
- `crates/focusa-api/src/routes/interview_strategy.rs`
- `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- `crates/focusa-api/src/routes/prediction_authority.rs`
- `crates/focusa-api/src/routes/project_bootstrap.rs`
- `crates/focusa-api/src/routes/project_bootstrap_support.rs`
- `crates/focusa-api/src/routes/project_genesis.rs`
- `crates/focusa-api/src/routes/project_genesis_support.rs`
- `crates/focusa-api/src/routes/project_genesis_tests.rs`
- `crates/focusa-api/src/routes/provider_execution.rs`
- `crates/focusa-api/src/routes/role_profiles.rs`
- `crates/focusa-api/src/routes/silent_sessions.rs`
- `crates/focusa-api/src/routes/silent_sessions_adopt.rs`
- `crates/focusa-api/src/routes/silent_sessions_authorize.rs`
- `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- `crates/focusa-api/src/routes/silent_sessions_config_mutation.rs`
- `crates/focusa-api/src/routes/silent_sessions_config_mutation_test.rs`
- `crates/focusa-api/src/routes/silent_sessions_config_read.rs`
- `crates/focusa-api/src/routes/silent_sessions_contract.rs`
- `crates/focusa-api/src/routes/silent_sessions_control.rs`
- `crates/focusa-api/src/routes/silent_sessions_create.rs`
- `crates/focusa-api/src/routes/silent_sessions_input.rs`
- `crates/focusa-api/src/routes/silent_sessions_input_test.rs`
- `crates/focusa-api/src/routes/silent_sessions_lifecycle.rs`
- `crates/focusa-api/src/routes/silent_sessions_observe.rs`
- `crates/focusa-api/src/routes/silent_sessions_projection.rs`
- `crates/focusa-api/src/routes/silent_sessions_restart.rs`
- `crates/focusa-api/src/routes/silent_sessions_retention.rs`
- `crates/focusa-api/src/routes/silent_sessions_retention_export.rs`
- `crates/focusa-api/src/routes/spec_workbench.rs`
- `crates/focusa-api/src/routes/task_plans.rs`
- `crates/focusa-api/src/routes/temporal.rs`
- `crates/focusa-api/src/routes/temporal_advanced.rs`
- `crates/focusa-api/src/routes/work_rail.rs`
- `crates/focusa-api/src/routes/workspace_artifacts.rs`

### B.3 CLI command families

- `crates/focusa-cli/src/api_client.rs`
- `crates/focusa-cli/src/main.rs`
- `crates/focusa-cli/src/commands/action.rs`
- `crates/focusa-cli/src/commands/about.rs`
- `crates/focusa-cli/src/commands/audit.rs`
- `crates/focusa-cli/src/commands/autonomy.rs`
- `crates/focusa-cli/src/commands/awareness.rs`
- `crates/focusa-cli/src/commands/binary.rs`
- `crates/focusa-cli/src/commands/bloatgaurd.rs`
- `crates/focusa-cli/src/commands/cache.rs`
- `crates/focusa-cli/src/commands/call_stack.rs`
- `crates/focusa-cli/src/commands/claim.rs`
- `crates/focusa-cli/src/commands/cleanup.rs`
- `crates/focusa-cli/src/commands/compaction.rs`
- `crates/focusa-cli/src/commands/clt.rs`
- `crates/focusa-cli/src/commands/codesign.rs`
- `crates/focusa-cli/src/commands/constitution.rs`
- `crates/focusa-cli/src/commands/context_cognition.rs`
- `crates/focusa-cli/src/commands/continue_work.rs`
- `crates/focusa-cli/src/commands/contribute.rs`
- `crates/focusa-cli/src/commands/deck.rs`
- `crates/focusa-cli/src/commands/daemon.rs`
- `crates/focusa-cli/src/commands/debug.rs`
- `crates/focusa-cli/src/commands/device_pairing.rs`
- `crates/focusa-cli/src/commands/doctor.rs`
- `crates/focusa-cli/src/commands/dxux.rs`
- `crates/focusa-cli/src/commands/ecs.rs`
- `crates/focusa-cli/src/commands/env.rs`
- `crates/focusa-cli/src/commands/export.rs`
- `crates/focusa-cli/src/commands/first_mission.rs`
- `crates/focusa-cli/src/commands/focus.rs`
- `crates/focusa-cli/src/commands/gate.rs`
- `crates/focusa-cli/src/commands/help.rs`
- `crates/focusa-cli/src/commands/hlt.rs`
- `crates/focusa-cli/src/commands/init.rs`
- `crates/focusa-cli/src/commands/install.rs`
- `crates/focusa-cli/src/commands/intro.rs`
- `crates/focusa-cli/src/commands/license.rs`
- `crates/focusa-cli/src/commands/lineage.rs`
- `crates/focusa-cli/src/commands/memory.rs`
- `crates/focusa-cli/src/commands/metacognition.rs`
- `crates/focusa-cli/src/commands/onboard.rs`
- `crates/focusa-cli/src/commands/ontology.rs`
- `crates/focusa-cli/src/commands/pair.rs`
- `crates/focusa-cli/src/commands/pairing_cycle_test.rs`
- `crates/focusa-cli/src/commands/pairing_dashboard.rs`
- `crates/focusa-cli/src/commands/pairing_doctor.rs`
- `crates/focusa-cli/src/commands/pairing_email_link.rs`
- `crates/focusa-cli/src/commands/pairing_transport.rs`
- `crates/focusa-cli/src/commands/pairing_wizard.rs`
- `crates/focusa-cli/src/commands/predict.rs`
- `crates/focusa-cli/src/commands/preload.rs`
- `crates/focusa-cli/src/commands/project.rs`
- `crates/focusa-cli/src/commands/proposals.rs`
- `crates/focusa-cli/src/commands/recover.rs`
- `crates/focusa-cli/src/commands/reflection.rs`
- `crates/focusa-cli/src/commands/release.rs`
- `crates/focusa-cli/src/commands/resource.rs`
- `crates/focusa-cli/src/commands/rfm.rs`
- `crates/focusa-cli/src/commands/runtime.rs`
- `crates/focusa-cli/src/commands/scope.rs`
- `crates/focusa-cli/src/commands/scope_resolver.rs`
- `crates/focusa-cli/src/commands/service.rs`
- `crates/focusa-cli/src/commands/setup.rs`
- `crates/focusa-cli/src/commands/skills.rs`
- `crates/focusa-cli/src/commands/telemetry.rs`
- `crates/focusa-cli/src/commands/threads.rs`
- `crates/focusa-cli/src/commands/tokens.rs`
- `crates/focusa-cli/src/commands/trajectory.rs`
- `crates/focusa-cli/src/commands/traverse.rs`
- `crates/focusa-cli/src/commands/turns.rs`
- `crates/focusa-cli/src/commands/tui.rs`
- `crates/focusa-cli/src/commands/update.rs`
- `crates/focusa-cli/src/commands/utility.rs`
- `crates/focusa-cli/src/commands/uninstall.rs`
- `crates/focusa-cli/src/commands/upgrade.rs`
- `crates/focusa-cli/src/commands/walkthrough.rs`
- `crates/focusa-cli/src/commands/work_item.rs`
- `crates/focusa-cli/src/commands/workflow.rs`
- `crates/focusa-cli/src/commands/workpoint.rs`
- `crates/focusa-cli/src/commands/wrap.rs`
- `crates/focusa-cli/src/commands/mod.rs`
#### Post-baseline CLI commands

- `crates/focusa-cli/src/commands/agent_runtime.rs`
- `crates/focusa-cli/src/commands/install_e6_failure_matrix_tests.rs`
- `crates/focusa-cli/src/commands/pi_launch.rs`
- `crates/focusa-cli/src/commands/pi_launch_migration.rs`
- `crates/focusa-cli/src/commands/release_master.rs`
- `crates/focusa-cli/src/commands/silent.rs`
- `crates/focusa-cli/src/commands/silent_render.rs`
- `crates/focusa-cli/src/commands/temporal.rs`
- `crates/focusa-cli/src/commands/update_trust.rs`

### B.4 Core / TUI / model surfaces

#### focusa-core

- `crates/focusa-core/src/adapters/acp.rs`
- `crates/focusa-core/src/adapters/anthropic.rs`
- `crates/focusa-core/src/adapters/letta.rs`
- `crates/focusa-core/src/adapters/mod.rs`
- `crates/focusa-core/src/adapters/openai.rs`
- `crates/focusa-core/src/adapters/passthrough.rs`
- `crates/focusa-core/src/ascc.rs`
- `crates/focusa-core/src/autonomy/mod.rs`
- `crates/focusa-core/src/awareness.rs`
- `crates/focusa-core/src/bloatgaurd.rs`
- `crates/focusa-core/src/bonjour.rs`
- `crates/focusa-core/src/cache/mod.rs`
- `crates/focusa-core/src/claim_gate.rs`
- `crates/focusa-core/src/clt/mod.rs`
- `crates/focusa-core/src/constitution/mod.rs`
- `crates/focusa-core/src/dxux.rs`
- `crates/focusa-core/src/expression/budget.rs`
- `crates/focusa-core/src/expression/engine.rs`
- `crates/focusa-core/src/expression/mod.rs`
- `crates/focusa-core/src/expression/serializer.rs`
- `crates/focusa-core/src/focus/frame.rs`
- `crates/focusa-core/src/focus/mod.rs`
- `crates/focusa-core/src/focus/stack.rs`
- `crates/focusa-core/src/focus/state.rs`
- `crates/focusa-core/src/gate/candidates.rs`
- `crates/focusa-core/src/gate/focus_gate.rs`
- `crates/focusa-core/src/gate/mod.rs`
- `crates/focusa-core/src/intuition/aggregation.rs`
- `crates/focusa-core/src/intuition/engine.rs`
- `crates/focusa-core/src/intuition/mod.rs`
- `crates/focusa-core/src/intuition/signals.rs`
- `crates/focusa-core/src/lib.rs`
- `crates/focusa-core/src/license.rs`
- `crates/focusa-core/src/memory/mod.rs`
- `crates/focusa-core/src/memory/procedural.rs`
- `crates/focusa-core/src/memory/semantic.rs`
- `crates/focusa-core/src/permissions/mod.rs`
- `crates/focusa-core/src/pre/mod.rs`
- `crates/focusa-core/src/pre/resolution.rs`
- `crates/focusa-core/src/reducer.rs`
- `crates/focusa-core/src/reference/artifact.rs`
- `crates/focusa-core/src/reference/gc.rs`
- `crates/focusa-core/src/reference/mod.rs`
- `crates/focusa-core/src/reference/store.rs`
- `crates/focusa-core/src/replay/mod.rs`
- `crates/focusa-core/src/scope_safety.rs`
- `crates/focusa-core/src/rfm/mod.rs`
- `crates/focusa-core/src/runtime/daemon.rs`
- `crates/focusa-core/src/runtime/event_bus.rs`
- `crates/focusa-core/src/runtime/events.rs`
- `crates/focusa-core/src/runtime/mod.rs`
- `crates/focusa-core/src/runtime/persistence.rs`
- `crates/focusa-core/src/runtime/persistence_sqlite.rs`
- `crates/focusa-core/src/runtime/persistence_sqlite_test.rs`
- `crates/focusa-core/src/skills/mod.rs`
- `crates/focusa-core/src/sync/crdt.rs`
- `crates/focusa-core/src/sync/mod.rs`
- `crates/focusa-core/src/telemetry/mod.rs`
- `crates/focusa-core/src/threads/mod.rs`
- `crates/focusa-core/src/training/mod.rs`
- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/update.rs`
- `crates/focusa-core/src/utility_card.rs`
- `crates/focusa-core/src/work_item/mod.rs`
- `crates/focusa-core/src/work_item/adapter.rs`
- `crates/focusa-core/src/work_item/adapters/bd.rs`
- `crates/focusa-core/src/work_item/adapters/mod.rs`
- `crates/focusa-core/src/work_item/adapters/none.rs`
- `crates/focusa-core/src/work_item/audit.rs`
- `crates/focusa-core/src/work_item/evidence.rs`
- `crates/focusa-core/src/work_item/lifecycle.rs`
- `crates/focusa-core/src/work_item/policy.rs`
- `crates/focusa-core/src/work_item/scope_safety.rs`
- `crates/focusa-core/src/work_item/storage.rs`
- `crates/focusa-core/src/work_item/types.rs`
- `crates/focusa-core/src/uxp/mod.rs`
- `crates/focusa-core/src/workers/executor.rs`
- `crates/focusa-core/src/workers/mod.rs`
- `crates/focusa-core/src/workers/priority_queue.rs`
- `crates/focusa-core/src/workers/queue.rs`
#### Post-baseline focusa-core surfaces

- `crates/focusa-core/src/agent_runtime_constitution.rs`
- `crates/focusa-core/src/agent_runtime_constitution_authority.rs`
- `crates/focusa-core/src/agent_runtime_constitution_authority_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_compiler.rs`
- `crates/focusa-core/src/agent_runtime_constitution_compiler_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_enforcement.rs`
- `crates/focusa-core/src/agent_runtime_constitution_enforcement_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_lifecycle.rs`
- `crates/focusa-core/src/agent_runtime_constitution_lifecycle_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_migration.rs`
- `crates/focusa-core/src/agent_runtime_constitution_migration_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_orchestrator.rs`
- `crates/focusa-core/src/agent_runtime_constitution_orchestrator_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_store.rs`
- `crates/focusa-core/src/agent_runtime_constitution_store_test.rs`
- `crates/focusa-core/src/agent_runtime_constitution_test.rs`
- `crates/focusa-core/src/agent_runtime_instruction_integrity.rs`
- `crates/focusa-core/src/agent_runtime_instruction_integrity_scenario_test.rs`
- `crates/focusa-core/src/connector_auth.rs`
- `crates/focusa-core/src/connectors.rs`
- `crates/focusa-core/src/epistemic_conformance.rs`
- `crates/focusa-core/src/epistemic_fusion.rs`
- `crates/focusa-core/src/epistemic_memory_lifecycle.rs`
- `crates/focusa-core/src/epistemic_primitives.rs`
- `crates/focusa-core/src/epistemic_security.rs`
- `crates/focusa-core/src/google_drive_connector.rs`
- `crates/focusa-core/src/install_lifecycle.rs`
- `crates/focusa-core/src/metacognitive_learning.rs`
- `crates/focusa-core/src/outcome_resolution.rs`
- `crates/focusa-core/src/prediction.rs`
- `crates/focusa-core/src/prediction_advanced.rs`
- `crates/focusa-core/src/prediction_authority.rs`
- `crates/focusa-core/src/prediction_authority_ledger.rs`
- `crates/focusa-core/src/prediction_authority_storage.rs`
- `crates/focusa-core/src/prediction_authority_tests.rs`
- `crates/focusa-core/src/prediction_calibration.rs`
- `crates/focusa-core/src/prediction_migration.rs`
- `crates/focusa-core/src/prediction_profiles.rs`
- `crates/focusa-core/src/prediction_scoring.rs`
- `crates/focusa-core/src/prediction_scoring_algorithms.rs`
- `crates/focusa-core/src/provider_execution.rs`
- `crates/focusa-core/src/release_adapters.rs`
- `crates/focusa-core/src/release_adapters_test.rs`
- `crates/focusa-core/src/release_calibration.rs`
- `crates/focusa-core/src/release_calibration_test.rs`
- `crates/focusa-core/src/release_cycle.rs`
- `crates/focusa-core/src/release_cycle_test.rs`
- `crates/focusa-core/src/release_intelligence.rs`
- `crates/focusa-core/src/release_ledger.rs`
- `crates/focusa-core/src/release_ledger_test.rs`
- `crates/focusa-core/src/release_orchestrator.rs`
- `crates/focusa-core/src/release_orchestrator_test.rs`
- `crates/focusa-core/src/release_planner.rs`
- `crates/focusa-core/src/release_protocol.rs`
- `crates/focusa-core/src/runtime/context_retrieval.rs`
- `crates/focusa-core/src/runtime/interview_strategy.rs`
- `crates/focusa-core/src/runtime/persistence_actor.rs`
- `crates/focusa-core/src/scoped_state.rs`
- `crates/focusa-core/src/silent_session.rs`
- `crates/focusa-core/src/silent_session_authority.rs`
- `crates/focusa-core/src/silent_session_authorization.rs`
- `crates/focusa-core/src/silent_session_bootstrap.rs`
- `crates/focusa-core/src/silent_session_checkpoint_policy.rs`
- `crates/focusa-core/src/silent_session_completion.rs`
- `crates/focusa-core/src/silent_session_config.rs`
- `crates/focusa-core/src/silent_session_continuation.rs`
- `crates/focusa-core/src/silent_session_failure.rs`
- `crates/focusa-core/src/silent_session_integration.rs`
- `crates/focusa-core/src/silent_session_launch.rs`
- `crates/focusa-core/src/silent_session_notifications.rs`
- `crates/focusa-core/src/silent_session_protocol.rs`
- `crates/focusa-core/src/silent_session_receipts.rs`
- `crates/focusa-core/src/silent_session_reconstruction.rs`
- `crates/focusa-core/src/silent_session_recovery.rs`
- `crates/focusa-core/src/silent_session_reducer.rs`
- `crates/focusa-core/src/silent_session_resources.rs`
- `crates/focusa-core/src/silent_session_retry.rs`
- `crates/focusa-core/src/silent_session_scheduler.rs`
- `crates/focusa-core/src/silent_session_stream.rs`
- `crates/focusa-core/src/silent_session_wizard.rs`
- `crates/focusa-core/src/silent_session_workspace.rs`
- `crates/focusa-core/src/silent_session_writer.rs`
- `crates/focusa-core/src/silent_sessions/authorization.rs`
- `crates/focusa-core/src/silent_sessions/authorization_persistence.rs`
- `crates/focusa-core/src/silent_sessions/authorization_test.rs`
- `crates/focusa-core/src/silent_sessions/capability_catalog.rs`
- `crates/focusa-core/src/silent_sessions/cognitive_governance.rs`
- `crates/focusa-core/src/silent_sessions/completion_artifacts.rs`
- `crates/focusa-core/src/silent_sessions/concurrency_governance.rs`
- `crates/focusa-core/src/silent_sessions/config.rs`
- `crates/focusa-core/src/silent_sessions/config_resolution.rs`
- `crates/focusa-core/src/silent_sessions/config_resolution_test.rs`
- `crates/focusa-core/src/silent_sessions/config_revision.rs`
- `crates/focusa-core/src/silent_sessions/config_revision_test.rs`
- `crates/focusa-core/src/silent_sessions/event_protocol.rs`
- `crates/focusa-core/src/silent_sessions/failure_envelope.rs`
- `crates/focusa-core/src/silent_sessions/harness_adapter.rs`
- `crates/focusa-core/src/silent_sessions/harness_adapter_test.rs`
- `crates/focusa-core/src/silent_sessions/identity.rs`
- `crates/focusa-core/src/silent_sessions/launch_manifest.rs`
- `crates/focusa-core/src/silent_sessions/launch_manifest_test.rs`
- `crates/focusa-core/src/silent_sessions/legacy_import.rs`
- `crates/focusa-core/src/silent_sessions/legacy_import_test.rs`
- `crates/focusa-core/src/silent_sessions/mod.rs`
- `crates/focusa-core/src/silent_sessions/model_safety.rs`
- `crates/focusa-core/src/silent_sessions/operator_experience.rs`
- `crates/focusa-core/src/silent_sessions/persistence_records.rs`
- `crates/focusa-core/src/silent_sessions/persistence_sqlite.rs`
- `crates/focusa-core/src/silent_sessions/persistence_sqlite_test.rs`
- `crates/focusa-core/src/silent_sessions/persistence_usage.rs`
- `crates/focusa-core/src/silent_sessions/pi_rpc_adapter.rs`
- `crates/focusa-core/src/silent_sessions/platform_backends.rs`
- `crates/focusa-core/src/silent_sessions/process_supervision.rs`
- `crates/focusa-core/src/silent_sessions/recovery_policy.rs`
- `crates/focusa-core/src/silent_sessions/resource_admission.rs`
- `crates/focusa-core/src/silent_sessions/retention.rs`
- `crates/focusa-core/src/silent_sessions/runner_client.rs`
- `crates/focusa-core/src/silent_sessions/runner_protocol.rs`
- `crates/focusa-core/src/silent_sessions/runner_protocol_test.rs`
- `crates/focusa-core/src/silent_sessions/runner_security.rs`
- `crates/focusa-core/src/silent_sessions/runner_security_test.rs`
- `crates/focusa-core/src/silent_sessions/runtime_control.rs`
- `crates/focusa-core/src/silent_sessions/secure_fs.rs`
- `crates/focusa-core/src/silent_sessions/state_machine.rs`
- `crates/focusa-core/src/silent_sessions/stream_codec.rs`
- `crates/focusa-core/src/silent_sessions/stream_recovery.rs`
- `crates/focusa-core/src/silent_sessions/stream_rotation.rs`
- `crates/focusa-core/src/silent_sessions/stream_storage.rs`
- `crates/focusa-core/src/silent_sessions/stream_storage_test.rs`
- `crates/focusa-core/src/silent_sessions/types.rs`
- `crates/focusa-core/src/software_domain.rs`
- `crates/focusa-core/src/temporal.rs`
- `crates/focusa-core/src/temporal_authority.rs`
- `crates/focusa-core/src/temporal_claims.rs`
- `crates/focusa-core/src/temporal_clock.rs`
- `crates/focusa-core/src/temporal_conformance.rs`
- `crates/focusa-core/src/temporal_deadline.rs`
- `crates/focusa-core/src/temporal_forecast.rs`
- `crates/focusa-core/src/temporal_forecast_evaluation.rs`
- `crates/focusa-core/src/temporal_foundation.rs`
- `crates/focusa-core/src/temporal_full_tests.rs`
- `crates/focusa-core/src/temporal_high_consequence.rs`
- `crates/focusa-core/src/temporal_integrity.rs`
- `crates/focusa-core/src/temporal_ledger.rs`
- `crates/focusa-core/src/temporal_operations.rs`
- `crates/focusa-core/src/temporal_platform.rs`
- `crates/focusa-core/src/temporal_progress.rs`
- `crates/focusa-core/src/temporal_release_gate.rs`
- `crates/focusa-core/src/temporal_tests.rs`
- `crates/focusa-core/src/tool_result.rs`
- `crates/focusa-core/src/work_item/scheduler.rs`
- `crates/focusa-core/src/working_subpath.rs`

#### focusa-tui

- `crates/focusa-tui/src/api.rs`
- `crates/focusa-tui/src/app.rs`
- `crates/focusa-tui/src/main.rs`
- `crates/focusa-tui/src/theme.rs`
- `crates/focusa-tui/src/views/autonomy.rs`
- `crates/focusa-tui/src/views/cache.rs`
- `crates/focusa-tui/src/views/constitution.rs`
- `crates/focusa-tui/src/views/contribution.rs`
- `crates/focusa-tui/src/views/events.rs`
- `crates/focusa-tui/src/views/focus_stack.rs`
- `crates/focusa-tui/src/views/focus_state.rs`
- `crates/focusa-tui/src/views/gate.rs`
- `crates/focusa-tui/src/views/intuition.rs`
- `crates/focusa-tui/src/views/lineage.rs`
- `crates/focusa-tui/src/views/metrics.rs`
- `crates/focusa-tui/src/views/mod.rs`
- `crates/focusa-tui/src/views/proposals.rs`
- `crates/focusa-tui/src/views/references.rs`
- `crates/focusa-tui/src/views/rfm.rs`
- `crates/focusa-tui/src/views/skills.rs`
- `crates/focusa-tui/src/views/telemetry.rs`
- `crates/focusa-tui/src/views/training.rs`
- `crates/focusa-tui/src/views/uxp.rs`
- `crates/focusa-tui/src/views/work_loop.rs`
#### Docs / contracts / tool doc surfaces

- `docs/focusa-tools/tools/focusa_agent_prompt.md`
- `docs/focusa-tools/tools/focusa_utility_card.md`
- `docs/focusa-tools/tools/focusa_tool_doctor.md`
- `apps/pi-extension/src/tool-contracts.ts`

### 12.2 Annex C — Mutable-global / singleton-evidence inventory

The following files were observed containing mutable global or singleton-shaped behavior and must either map to Annex A remediation rows or be explicitly allowlisted as infra-only.

### C.1 Native / adapter / bridge layers

- `apps/menubar/src-tauri/src/main.rs`
- `apps/pi-extension/src/state.ts`
- `apps/pi-extension/src/tools.ts`

### C.2 API / server layers

- `crates/focusa-api/src/middleware/rate_limit.rs`
- `crates/focusa-api/src/middleware/mod.rs`
- `crates/focusa-api/src/middleware/route_scope.rs`
- `crates/focusa-api/src/routes/bounded.rs`
- `crates/focusa-api/src/routes/device_pairing.rs`
- `crates/focusa-api/src/routes/metacognition.rs`
- `crates/focusa-api/src/routes/ontology.rs`
- `crates/focusa-api/src/routes/predictions.rs`
- `crates/focusa-api/src/routes/project.rs`
- `crates/focusa-api/src/routes/proxy.rs`
- `crates/focusa-api/src/routes/snapshots.rs`
- `crates/focusa-api/src/routes/turn.rs`
- `crates/focusa-api/src/routes/workpoint.rs`
- `crates/focusa-api/src/routes/mod.rs`
- `crates/focusa-api/src/server.rs`

Every file in this annex must be assigned one of:

- authority-bearing singleton to eliminate,
- scope-keyed store to refactor,
- infra-only allowlisted global,
- consumer-side singleton shadow to remove.

## 13. One-line definition

Spec 104 turns Focusa into a typed scoped runtime whose canonical authority no longer depends on singleton state anywhere that can affect trust, continuation, implementation correctness, or proof.
