# Spec 158 Companion 02 — Persistence, Migration, Replay, and Quarantine

**Status:** normative companion to Spec 158  
**Parent:** `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`

---

## 1. Persistence target

The mixed global cognition snapshot is not an acceptable permanent authority.

Canonical persistence SHALL provide:

- Scope/Project records;
- stable Workstream records;
- Workstream-scoped event streams;
- Workstream-scoped snapshots/projections;
- Continuity lineage inside the Workstream;
- Attachment and workspace-binding records;
- event head, causal metadata and projection version;
- independent export and replay for one Workstream;
- explicit quarantine for unresolved legacy data.

Illustrative relational shape:

```text
projects(
  project_root_key,
  identity_json,
  project_hlt_json,
  projection_version
)

workstreams(
  project_root_key,
  workstream_id,
  created_at,
  status,
  legacy_thread_id,
  migration_provenance_json
)

workstream_events(
  project_root_key,
  workstream_id,
  sequence,
  event_id,
  event_type,
  event_json,
  actor_ref,
  attachment_id,
  causal_parent,
  idempotency_key,
  recorded_at
)

workstream_snapshots(
  project_root_key,
  workstream_id,
  sequence,
  state_json,
  projection_version,
  checksum,
  created_at
)

attachments(
  attachment_id,
  project_root_key,
  workstream_id,
  continuity_id,
  instance_id,
  session_id,
  workspace_binding_id,
  status
)

legacy_quarantine(
  quarantine_id,
  source_kind,
  source_ref,
  reason_code,
  candidate_workstreams_json,
  evidence_json,
  repair_status
)
```

Exact storage technology may vary. Ownership and replay guarantees may not.

---

## 2. Global snapshot disposition

The legacy global snapshot may be retained only as:

- immutable forensic migration evidence;
- rollback input during a bounded cutover window;
- a parity comparison source;
- an explicit legacy export.

It SHALL NOT remain a writable canonical source after cutover.

Permanent dual writes to both global and Workstream stores are forbidden.

---

## 3. Migration inventory

Before modifying storage, produce a field-and-surface ledger covering:

```text
field or store
current owner
read paths
write paths
fallback paths
cache keys
serialization format
candidate Scope
candidate Workstream mapping
ambiguity conditions
migration action
parity proof
rollback action
removal gate
```

The inventory includes:

- core structs and reducer events;
- SQLite tables and snapshots;
- API response caches;
- Pi local/scoped state and recovery sidecars;
- Workpoint/Trajectory packets;
- Work Loop and Silent Session stores;
- Context, memory, ontology, Evidence and claim stores;
- menubar/TUI/Mission Canvas read models;
- remote host, worktree and device bindings;
- idempotency and lease keys;
- export/training/telemetry paths.

---

## 4. Workstream ID generation and mapping

Existing records using `(project_root, continuity_id)`, Thread IDs, Session IDs or global active pointers require explicit mapping.

A Workstream mapping is acceptable only when supported by sufficient evidence, such as:

- one unique legacy durable workspace record;
- stable project identity;
- compatible Workpoint and Trajectory lineage;
- consistent attachment/session history;
- compatible event ancestry;
- explicit operator or migration manifest confirmation.

The mapping record SHALL retain provenance:

```json
{
  "schema": "focusa.workstream_migration_mapping.v1",
  "source_refs": ["legacy-thread:...", "continuity:..."],
  "scope_ref": {"project_root_key": "..."},
  "workstream_id": "ws_...",
  "confidence": "proven",
  "evidence_refs": ["..."],
  "approved_by": "migration-rule|operator",
  "created_at": "..."
}
```

Similarity alone is never enough to assign canonical ownership.

---

## 5. Quarantine law

Ambiguous, conflicting, foreign or unverified legacy records SHALL enter quarantine.

Quarantine guarantees:

- no canonical mutation;
- no prompt-visible canonical augmentation;
- no automatic merge;
- no silent assignment to a default Workstream;
- no data loss;
- bounded evidence and candidate information;
- explicit operator repair or deterministic migration rule;
- audit and Receipt for resolution.

Required reason classes include:

```text
missing_scope
missing_workstream_identity
multiple_candidate_workstreams
conflicting_project_roots
conflicting_thread_lineage
continuity_collision
session_only_identity
foreign_host_or_worktree
invalid_causal_history
corrupt_snapshot
unsupported_projection_version
```

---

## 6. Shadow materialization and parity

Migration proceeds subsystem by subsystem.

A shadow phase may:

1. read legacy canonical state;
2. materialize the proposed Workstream partition;
3. replay scoped events;
4. compare bounded projections;
5. record mismatches;
6. block cutover on unexplained differences.

Shadow output is advisory until cutover. It must not produce a second independently mutable canonical truth.

Parity comparison SHALL distinguish:

- expected differences caused by removal of foreign/global fallback;
- migration bugs;
- ambiguous legacy state requiring quarantine;
- intentionally deprecated data;
- serialization-only differences.

---

## 7. Subsystem cutover order

Recommended order:

```text
1. Workpoint + tactical Trajectory
2. Focus Stack + Focus State
3. Work Loop + writer leases + temporal authority
4. Silent Sessions + runner/Attachment state
5. Context + memory + ontology
6. Evidence + claims + references
7. API/CLI/MCP/Pi/UI read models
8. export/training/telemetry
```

Each cutover requires:

- backup;
- migration preview;
- shadow parity report;
- rollback rehearsal;
- write-path switch;
- read-path switch;
- fallback removal;
- post-cutover replay proof;
- deprecation/cleanup issue.

---

## 8. Replay law

A Workstream replay SHALL be deterministic from:

```text
project identity/version
Workstream creation/migration record
ordered Workstream event stream
projection version
referenced immutable artifacts
```

Replay must not require:

- another Workstream’s state;
- daemon-global active/current fields;
- current UI selection;
- latest cache contents;
- transcript tail;
- network availability for already-canonical facts.

Provider-opaque execution objects may be accelerators or evidence sources, but they are not the canonical reducer history.

---

## 9. Cache, idempotency and lease keys

Every authority-sensitive key SHALL begin with exact Scope + Workstream identity.

Illustrative keys:

```text
cache:{project_root_key}:{workstream_id}:{domain}:{revision}
idempotency:{project_root_key}:{workstream_id}:{operation}:{key}
lease:{project_root_key}:{workstream_id}:{partition}
recovery:{project_root_key}:{workstream_id}:{workspace_binding_id}:{attachment_id}
```

A cache may use additional Attachment or Session narrowing. It may never omit the Workstream owner where canonical cognition is involved.

---

## 10. Export and portability

Workstream export SHALL be self-contained and deterministic.

It includes:

- Workstream identity and Scope;
- migration provenance where applicable;
- event stream or equivalent canonical history;
- latest verified snapshot;
- Continuity lineage;
- Attachment references;
- Workpoints and tactical Trajectory;
- scoped Context/memory/ontology references;
- Evidence/Receipt/claim/reference edges;
- projection and schema versions;
- checksum manifest.

Cross-Workstream aggregation is an explicit export mode and never the default.

---

## 11. Backup and rollback

Before migration:

- back up the global snapshot and database;
- record schema/version/checksum;
- verify restore in a separate target;
- preserve release and binary compatibility metadata.

Rollback SHALL restore a complete compatible set. It must not combine a new reducer with an incompatible old persistence projection or vice versa.

Rollback is a bounded safety mechanism, not a reason to preserve indefinite dual authority.

---

## 12. Closure gates

Persistence migration is complete only when:

- every canonical cognitive write targets one Workstream partition;
- every canonical read resolves one Workstream partition;
- per-Workstream replay passes;
- ambiguous data is quarantined;
- legacy snapshots are immutable forensic artifacts;
- global cognitive writes are disabled;
- global cognitive reads/fallbacks are removed;
- independent Workstream export passes;
- backup and rollback rehearsal passes;
- no permanent dual canonical write path remains.
