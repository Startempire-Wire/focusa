# docs/43-multi-device-sync.md — Multi-Device Local-First Sync (AUTHORITATIVE)

## Goal

Support **local-first** Focusa usage across multiple machines (e.g. MacBook + VPS) with:
- **bidirectional sync**
- deterministic, inspectable behavior
- no silent merges of cognitive state
- room for future **daisy chains** (peer relays)

This spec is designed to align with:
- `docs/40-instance-session-attachment-spec.md` (Instance/Session/Attachment)
- reducer determinism and event auditability

---

## Canonical Policy (MVP)

We adopt conflict policy **#2 + #5**:

### Policy #2 — Auto-import as observations only
Remote changes are imported as **observational events** by default.
- Observations may update:
  - telemetry
  - reference indexes
  - CLT nodes
  - artifacts/handles
- Observations MUST NOT directly mutate canonical Focus Stack / Focus State.

### Policy #5 — Per-thread ownership
Each Thread has an **owner machine_id**.
- Owner is the default canonical writer for that thread.
- Non-owner machines may:
  - observe
  - propose
  - attach as assistant/observer
- Owner may be changed only by explicit action (proposal → resolution).

---

## Definitions

### Machine
A physical device with a stable `machine_id`.
- MacBook, VPS, workstation.

### Peer
A reachable Focusa daemon endpoint configured for sync.

### Thread
A cognitive workspace (see thread specs). Threads are the unit of ownership.

### Observation
An imported event which is recorded and queryable but does not mutate canonical cognitive state.

### Proposal
A request to change canonical state (handled by PRE / proposal resolution engine).

---

## Identity & Matching

### Required IDs
Every event MUST include:
- `event_id` (uuidv7)
- `timestamp` (UTC)
- `machine_id`
- `instance_id`, `session_id` (when applicable)
- `thread_id` (when applicable)

### Repo / project matching
Sync MUST NOT guess by default.

MVP matching uses explicit identifiers:
- `workspace_id` (configured)
- `repo_signature` (derived from git remote URL + default branch + repo root fingerprint)

Heuristics/embeddings may be used for UI suggestions ONLY.

---

## Storage Model (Required)

Each machine runs its own Focusa daemon with local persistence:
- **SQLite** canonical DB for:
  - events
  - snapshots
  - UXP/UFI
  - telemetry
  - autonomy scoring indices
- **Filesystem** for ECS objects (blobs)

---

## Sync Transport

### Push/Pull (bidirectional)
Each peer sync consists of:
1. **pull** remote events since last cursor
2. **push** local events since last cursor
3. transfer missing ECS objects referenced by imported events

### Daisy chain readiness
Sync protocol MUST support forwarding:
- imported remote events keep their original `machine_id`
- peers MAY relay events without rewriting origin identity

---

## Cursors & Idempotency

Each peer connection maintains a cursor:
- `last_seen_event_id` OR `(last_seen_timestamp, event_id)`

Import must be idempotent:
- `event_id` is globally unique
- on conflict, ignore duplicates

---

## Conflict Semantics

### Canonical state
Only the thread owner’s events may directly mutate canonical focus state.

### Non-owner events
Non-owner events are imported as observations.
If they imply a desired state change, they become a **proposal**.

---

## Menubar UI Requirements (Intelligent)

Menubar is a window into cognition, but must also make sync legible without distraction.

Must show:
- local daemon status
- configured peers (MacBook/VPS)
- per-peer sync status:
  - last sync time
  - backlog counts (events pending)
  - errors
- per-thread ownership indicator:
  - owner machine
  - local attachment role (active/assistant/observer)
- conflicts surface as proposals count (no auto-merge)

Must allow:
- connect/disconnect peers
- trigger sync now
- view imported observations vs local canonical changes
- propose ownership transfer

---

## Non-Goals (MVP)

- real-time multi-master merging of Focus Stack
- CRDT merges for cognitive state
- automatic ownership changes
- cloud SaaS dependency

---

## Acceptance Criteria

- Two machines can run independently offline.
- After reconnect, events sync bidirectionally.
- Imported remote changes are visible (observations) without corrupting local invariants.
- Ownership prevents silent divergence of canonical focus state.
- UI makes it obvious what is local vs imported.
