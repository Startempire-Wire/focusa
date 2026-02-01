# docs/05-focus-stack-hec.md — Focus Stack (HEC) Specification (MVP)

## Purpose
The Focus Stack (HEC: Hierarchical Execution Context) provides a **structured hierarchy of work** so Focusa can:
- maintain stable focus over long sessions,
- isolate subtasks without exploding linear context,
- provide deterministic prompt assembly grounded in the active frame,
- enable organic priority surfacing via Focus Gate without auto-hijack.

HEC is the authoritative source of “what we are focused on.”

## Core Concepts

### Frame
A Focus Frame is a node in a tree/stack representing a unit of work.

Each frame has:
- a stable ID
- a parent pointer (except root)
- a goal/label (human-readable)
- state sections used by ASCC
- metadata for prompt selection

### Stack vs Tree
Internally, frames form a tree. The active path from root to active frame is the “stack”.

MVP only needs:
- push new child frame under current active
- pop back to parent
- set active to any existing frame on the active path (optional)
- list stack

No cross-branch jumping in MVP (keep simple and deterministic).

## Data Model

### FrameId
- String UUIDv7 (preferred) or ULID.

### FrameStatus
Enum:
- `active`
- `paused`
- `completed`
- `archived`

MVP behavior:
- only one `active` frame
- parent frames on stack path are `paused` when a child is active

### FrameRecord
Fields:
- `id: FrameId`
- `parent_id: Option<FrameId>`
- `created_at: ts`
- `updated_at: ts`
- `status: FrameStatus`
- `title: String` (short)
- `goal: String` (one sentence)
- `tags: Vec<String>` (optional)
- `priority_hint: Option<String>` (optional; not numeric)
- `ascc_checkpoint_id: Option<String>` (anchor pointer; see ASCC)
- `stats: FrameStats` (optional)
- `handles: Vec<HandleRef>` (references used in this frame; see ECS)
- `constraints: Vec<String>` (optional, short constraints relevant to this frame)

### FrameStats (MVP minimal)
- `turn_count: u64`
- `last_turn_id: Option<String>`
- `last_token_estimate: Option<u32>`

### FocusStackState
Fields:
- `root_id: FrameId`
- `active_id: FrameId`
- `frames: HashMap<FrameId, FrameRecord>`
- `stack_path_cache: Vec<FrameId>` (derived, cached for fast reads)
- `version: u64` (monotonic; increments on mutation)

## Operations

### PushFrame
Creates a new child frame under the current active frame.
Inputs:
- `title`
- `goal`
- `constraints?`
- `tags?`

Rules:
- New frame becomes `active`.
- Previous active becomes `paused`.
- Emit events:
  - `focus.frame_pushed`
  - `focus.active_changed`

### PopFrame
Returns focus to parent frame.
Rules:
- Current active frame status becomes:
  - `completed` if explicitly completed by user OR if pop called with `complete=true`
  - otherwise `paused`
- Parent becomes `active`.
- Emit events:
  - `focus.frame_popped`
  - `focus.active_changed`

MVP: Provide two commands:
- `pop` (pause current)
- `complete` (complete and pop)

### SetActiveFrame (MVP optional)
Allows selecting an ancestor frame on the current active path.
Rules:
- Only allowed if `target_id` is in current stack path.
- Children frames below target become `paused`.
- Target becomes `active`.
- Emit `focus.active_changed`

This prevents branch-hopping complexity.

### Read Operations
- `get_active_frame()`
- `get_stack_path()`: root -> active
- `get_frame(frame_id)`
- `list_frames(status?)` (optional)

## Frame Lifecycle Invariants
1. Exactly one active frame exists.
2. Active frame is always reachable from root via parent pointers.
3. Stack path is contiguous: root -> ... -> active.
4. Child frames cannot be active if parent is not in stack path.

## How HEC Influences Prompt Assembly
Prompt assembly always includes:
- Active frame ASCC checkpoint slots.
- Optionally includes parent ASCC checkpoints (bounded by budget).
- Never includes siblings or unrelated frames in MVP.

HEC provides:
- “focus proximity”:
  - active = 0
  - parent = 1
  - grandparent = 2
Used by prompt assembler’s inclusion heuristics.

## How HEC Interacts With Focus Gate
Focus Gate can propose:
- “candidate: push frame suggestion”
- “candidate: resume ancestor”
But Focus Gate never mutates HEC.

In MVP, Focus Gate candidates are surfaced to UI/CLI only.

## Persistence
Persist `FocusStackState` on every mutation (debounced).
File:
- `~/.focusa/state/focus_stack.json`

## Acceptance Tests
- Push/pop maintains invariants.
- Stack path correct across nested pushes.
- Active frame always valid after restart.
- Events emitted exactly once per mutation.

---

# UPDATE

# docs/05-focus-stack-hec.md (UPDATED) — Completion Semantics

## Frame Completion Semantics (Added)

### FrameCompletionReason
Enum:
- `goal_achieved`
- `blocked`
- `abandoned`
- `superseded`
- `error`

### FrameRecord (Updated)
Add:
- `completed_at: Option<timestamp>`
- `completion_reason: Option<FrameCompletionReason>`

### CLI Extensions
- `focusa focus complete --reason goal_achieved`
- default reason: `goal_achieved`

### ASCC Integration
Completion reason MUST be reflected in:
- `recent_results`
- or `failures` (if error/blocked)

### Focus Gate Integration
Completion reason contributes signals:
- `blocked` → raises surface pressure on related candidates
- `abandoned` → suppress related candidates
