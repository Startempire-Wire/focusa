# docs/08-ecs.md — Externalized Context Store (ECS) & Handles (MVP)

## Purpose
ECS prevents token waste by storing bulky content externally and replacing it in prompts with compact typed handles.

ECS is “lossless compression by indirection.”

## What ECS Stores (MVP)
- tool outputs (logs, search results, command output)
- diffs / patches
- large assistant responses
- user-provided attachments (where available)
- any blob above size/token threshold

ECS does NOT attempt semantic retrieval in MVP.
It is a key-value store by handle id with metadata + file content.

## Handle
A handle is the only thing that appears in prompts.

### HandleId
- UUIDv7 or hash-based id (sha256 prefix)
Preferred: uuidv7 for uniqueness + store sha256 in metadata.

### HandleKind (MVP)
- `log`
- `diff`
- `text`
- `json`
- `url`
- `file_snapshot`
- `other`

### HandleRef (prompt-safe)
Fields:
- `id: HandleId`
- `kind: HandleKind`
- `label: String` (short)
- `size: u64` (bytes)
- `sha256: String` (hex)
- `created_at`

Prompt representation:
`[HANDLE:<kind>:<id> "<label>"]`

## Storage Layout
Root: `~/.focusa/ecs/`

- `objects/` — immutable content-addressed blobs (optional in MVP)
- `handles/` — metadata json by id
- `index.json` — small index (id -> metadata)

MVP simplest:
- store blob at `objects/<id>`
- store metadata at `handles/<id>.json`
- update `index.json` (debounced)

## StoreArtifact Operation
Input:
- `kind`
- `label`
- `content_bytes` or `content_string`
- optional `content_type`
- `origin` + `correlation_id` + `frame_id`

Process:
1. compute `sha256`
2. generate `id`
3. write blob file
4. write metadata file
5. update index
6. emit `ecs.artifact_stored`

Return:
- HandleRef

## ResolveHandle Operation
Input:
- handle id
Output:
- metadata + content (streaming ok)

API:
- GET `/v1/ecs/resolve/:handle_id`

CLI:
- `focusa ecs cat <handle_id>`
- `focusa ecs meta <handle_id>`

## Threshold Policy (MVP)
Config:
- `ecs.externalize_bytes_threshold` default 8KB
- `ecs.externalize_token_estimate_threshold` default 800 tokens

If either exceeded, externalize.

## Prompt Inclusion Policy
In prompts, include handles only.
If model must see the content, provide an explicit retrieval step:
- user/harness can ask Focusa to “rehydrate handle” as needed:
  - Focusa returns content snippet or full content depending on budget.

MVP:
- add command: `focusa ecs rehydrate <id> --max-tokens N`
- rehydration returns:
  - first N tokens + trailing summary line with “truncated; fetch more if needed”

## Security and Privacy
Local-only storage.
Config can disable storing raw transcripts:
- if enabled, store only ASCC + handles for tool outputs.

## Garbage Collection (MVP Minimal)
- keep everything by default
- optional config: delete blobs older than N days
- ensure index consistency on startup (repair pass)

## Acceptance Tests
- storing same content results in distinct handles but same sha (ok)
- resolving returns exact bytes written
- large blobs never appear inline in prompt assembly
- index repair works when index missing

---

# UPDATE

# docs/08-ecs.md (UPDATED) — Session & Trust Rules

## ECS Session Scoping (Added)

- Every handle includes `session_id`
- Cross-session resolution forbidden by default
- Explicit override required

---

## Human Pinning (Added)

Pinned handles:
- never garbage collected
- always shown in ECS listings
- surfaced preferentially in Focus Gate

---

## Security Invariant
ECS must never:
- auto-inline content
- fetch remote data
- mutate stored artifacts
