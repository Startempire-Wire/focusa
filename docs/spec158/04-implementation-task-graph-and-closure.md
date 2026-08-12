# Spec 158 Companion 04 — Implementation Task Graph, Integration, and Closure

**Status:** normative companion to Spec 158  
**Parent:** `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`  
**Machine graph:** `docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`

---

## 1. Execution rules

- Preserve before modifying.
- Replacement precedes removal.
- No broad rebase of divergent Mission Canvas branches before unique-work inventory.
- No permanent dual canonical authority.
- No claim of singleton elimination until end-to-end closure passes.
- Every task has explicit inputs, outputs, dependencies, proof, rollback and cleanup.
- Work may parallelize only when ownership boundaries are stable and shared contracts are frozen.

---

## 2. Critical path

```text
PRESERVE
  -> INVENTORY GLOBAL COGNITION
  -> WORKSTREAM IDENTITY
  -> SCOPE ROUTER / REDUCER ENVELOPE
  -> SCOPED PERSISTENCE
  -> WORKPOINT + TRAJECTORY CUTOVER
  -> FOCUS STACK + FOCUS STATE CUTOVER
  -> WORK LOOP + SILENT SESSION CUTOVER
  -> CONTEXT/EVIDENCE/ONTOLOGY CUTOVER
  -> CLIENT CONTRACT CUTOVER
  -> MISSION CANVAS CORE EXTRACTION
  -> DESKTOP CONTROL PLANE
  -> PI WORK SURFACE
  -> FULL DESKTOP MIGRATION
  -> LEGACY REMOVAL AND CLOSURE
```

---

## 3. Phase graph

### Phase A — Preservation and authority freeze

Deliverables:

- current worktree preservation checkpoint;
- branch/local unique-work report;
- Mission Canvas migration ledger;
- test baseline;
- repository-wide global cognitive selector inventory;
- stop notices and agent bootstrap updates;
- no-new-global-authority gate.

No cleanup or decomposition work begins before preservation proof.

### Phase B — Identity and routing foundation

Deliverables:

- WorkstreamId and WorkstreamKey;
- ScopeRef and ProjectRootKey normalization;
- Continuity separated from Workstream identity;
- AttachmentKey and WorkspaceBindingId;
- WorkstreamContext extractor;
- ScopeRouter;
- Workstream event envelope;
- fail-closed ambiguity behavior;
- compatibility mapping schemas.

### Phase C — Persistence and migration infrastructure

Deliverables:

- Workstream event/snapshot storage;
- migration inventory tooling;
- mapping and quarantine stores;
- backup and restore proof;
- shadow materialization harness;
- parity report schema;
- rollback procedure.

### Phase D — Cognitive subsystem cutovers

Cut over in order:

1. Workpoints and tactical Trajectory;
2. Focus Stack and Focus State;
3. Work Loop, writer leases and temporal state;
4. Silent Sessions and runner state;
5. Context, memory and ontology;
6. Evidence, claims and references.

Each cutover has:

- shadow parity;
- write switch;
- read switch;
- fallback removal;
- replay proof;
- contamination tests;
- rollback rehearsal.

### Phase E — Client and schema cutover

Deliverables:

- Workstream-aware REST/OpenAPI;
- CLI/MCP/Pi tool schema alignment;
- generated-client regeneration;
- capability registry alignment;
- old compatibility routes with explicit deprecation and no fallback;
- exact Workstream echo in Results and Receipts.

### Phase F — Mission Canvas extraction

Deliverables:

- WorkSurface projection gains WorkstreamKey;
- session inventory gains WorkstreamKey;
- Pi TUI-independent `mission-canvas-core`;
- shared bounded read models;
- generated DTO inputs;
- Pi compatibility adapter remains functional;
- menubar duplicate projections inventoried and reduced.

### Phase G — Focusa Desktop vertical slice

Deliverables:

- Tauri/SvelteKit shell;
- shared workspace registry and command palette;
- Workstream-aware Context Control;
- Mission Deck;
- Mission Canvas current-work projection;
- Work Rail;
- truthful Evidence projection;
- daemon discovery, entitlement and updater state;
- semantic Desktop state endpoint.

### Phase H — GUI/CLI/agent control plane

Deliverables:

- Desktop manifest/status/state/events;
- Desktop presenter and operation Receipts;
- `focusa desktop` CLI;
- workspace/subsection/object addressing;
- Work Surface commands;
- agent tool projection;
- parity and blocked/recovery tests.

### Phase I — Embedded Pi Work Surface

Deliverables:

- cross-platform PTY;
- pinned Pi runtime distribution;
- automatic Focusa extension loading;
- exact Workstream Attachment binding;
- process survival and resize;
- split/detach/restoration;
- separate headless RPC execution adapter.

### Phase J — Full capability migration and Focusa.work

Deliverables:

- C.R.I.S.T. generated UI;
- Context/Role/Trajectory workspaces;
- Sessions, contention and approvals;
- Evidence/Receipts/history/reports;
- Documents and Research;
- Silent Session/UIAI Work Surfaces;
- hosted, connected-local and self-hosted web adapters.

### Phase K — Removal and closure

Deliverables:

- global cognitive writes disabled;
- global cognitive reads/fallbacks removed;
- singleton fields removed;
- global snapshot demoted to immutable forensic artifact;
- obsolete Thread surfaces removed or compatibility-bounded;
- obsolete Pi rich presentation removed only after parity;
- public docs and claims updated;
- release, rollback and migration proof complete.

---

## 4. Parallelization boundaries

May proceed in parallel after contracts are frozen:

- Desktop product-neutral shell extraction;
- PTY technology evaluation;
- visual/UX work using synthetic Workstream fixtures;
- generated schema/client tooling;
- preservation and branch inventory;
- documentation contradiction audit.

Must not precede Workstream foundation:

- canonical Desktop mutation routes;
- shared Mission Canvas core extraction that freezes continuity-only identity;
- Work Surface restoration authority;
- persistent agent-control state;
- Workpoint/Trajectory Desktop mutation against global active state;
- broad Focusa.work runtime implementation.

---

## 5. Required issue/task metadata

Every decomposed implementation task SHALL state:

```text
task_id
phase
owner
status
depends_on
paths
current authority assumption
target Workstream owner
migration action
compatibility impact
proof commands
Evidence/Receipt outputs
rollback
cleanup/removal gate
```

Tasks that touch existing Mission Canvas branches also state the originating branch/commit and whether the code is local-only, branch-only or on main.

---

## 6. Required proof suites

### Static proof

- no forbidden new global cognitive fields;
- no new canonical `thread_id` outputs;
- no continuity-only canonical keys;
- all Work Surface schemas contain Workstream identity;
- all mutable Desktop/CLI/agent commands have one registry entry.

### Runtime isolation proof

Run two or more concurrent Workstreams and prove:

- Focus Stack isolation;
- Workpoint isolation;
- tactical Trajectory isolation;
- Work Loop isolation;
- writer lease isolation;
- Context/memory/ontology isolation;
- Evidence visibility and authority isolation;
- Pi augmentation isolation;
- Silent Session isolation;
- Desktop visual focus cannot cross-mutate.

### Persistence proof

- independent replay;
- restart restoration;
- migration mapping determinism;
- quarantine behavior;
- backup/restore;
- rollback rehearsal;
- no permanent dual writes.

### Client parity proof

For each command:

- GUI invoke/present;
- CLI invoke/present;
- Pi/agent tool invoke/present;
- same exact Workstream resolution;
- same blocked/recovery semantics;
- same Receipt class;
- semantic state verification.

### Packaging proof

- one-installer path;
- pinned compatible Pi runtime;
- genuine PTY;
- signed update/rollback;
- Desktop/daemon/CLI/Pi/extension/contract matrix.

---

## 7. Cleanup policy

Cleanup is a distinct task after replacement proof.

Do not mix foundational migration with:

- broad formatting;
- unrelated dependency upgrades;
- package-manager normalization;
- full lockfile regeneration;
- mass renaming without migration semantics;
- deletion of tests before replacement coverage;
- silent removal of compatibility routes.

A cleanup task requires:

- old path/behavior;
- replacement owner;
- proof of parity or intentional deprecation;
- release note;
- rollback considerations;
- no remaining consumers.

---

## 8. Closure report

The final closure report SHALL include:

```text
A. architecture delta
B. singleton field removal inventory
C. Thread -> Workstream migration inventory
D. global snapshot disposition
E. migration/quarantine statistics
F. per-Workstream replay proof
G. concurrent isolation proof
H. GUI/CLI/agent parity matrix
I. Mission Canvas preservation/migration ledger
J. Pi compatibility status
K. Desktop and Focusa.work status
L. compatibility and deprecation inventory
M. backup/rollback rehearsal
N. release and public-claim audit
O. remaining explicitly deferred work, if any
```

Spec 158 and the Desktop transition are not closed by unit tests alone. Closure requires end-to-end authority proof across reducer, persistence, Pi, API, CLI and UI.
