# docs/07-ascc.md — Anchored Structured Context Checkpointing (ASCC) (MVP)

## Purpose
ASCC maintains a **persistent structured summary** per focus frame that:
- replaces linear chat history in prompts,
- updates incrementally using anchors,
- preserves high-fidelity task continuity.

ASCC is the primary “meaning-dense compression” mechanism.

## Design Principles
1. Structure over prose.
2. Delta updates, not full re-summarization.
3. Anchors prevent drift and ensure continuity.
4. Each focus frame has its own checkpoint (scope matters).

## Structured Checkpoint Schema (MVP)

### CheckpointId
- stable per frame: `frame_id` or a derived id
- each update produces a new revision id (monotonic integer)

### CheckpointRecord
Fields:
- `frame_id: FrameId`
- `revision: u64`
- `updated_at`
- `anchor_turn_id: String` (last processed turn)
- `sections: AsccSections`
- `breadcrumbs: Vec<HandleRef>` (optional handles to external artifacts)
- `confidence: AsccConfidence` (optional; MVP can omit)
- `history: Vec<AsccDeltaMeta>` (bounded; optional)

### AsccSections (fixed slots)
- `intent`: String (1–3 sentences)
- `current_focus`: String (1–3 sentences)
- `decisions`: Vec<String> (bullets; each <= 160 chars)
- `artifacts`: Vec<ArtifactLine>` (typed lines, see below)
- `constraints`: Vec<String>` (short)
- `open_questions`: Vec<String>`
- `next_steps`: Vec<String>`
- `recent_results`: Vec<String>` (short outputs or references)
- `failures`: Vec<String>` (what failed and why)
- `notes`: Vec<String>` (misc, bounded)

### ArtifactLine
- `kind: "file"|"diff"|"log"|"url"|"handle"|"other"`
- `label: String`
- `ref: Option<HandleRef>`
- `path_or_id: Option<String>`

MVP: store large artifact details in ECS and reference via handle.

## Anchor Model
Anchors are “turn ids” emitted by adapter:
- each user prompt/assistant response pair is a `turn_id`.
ASCC only summarizes content up to `turn_id`.

`anchor_turn_id` in checkpoint = last applied turn.

## Update Pipeline (MVP)

### Inputs for ASCC Update
- `frame_id`
- `turn_id`
- `raw_user_input` (small)
- `assistant_output` (small or handle)
- `tool_outputs` (handles)
- `events` relevant to this frame
- optionally: extracted facts/preferences from worker

### Delta Summarization Rule
When a new turn arrives:
1. Determine “delta content” = only new items since last anchor.
2. Summarize delta into structured slots.
3. Merge into existing checkpoint using deterministic merge rules.

MVP summarization mechanism:
- can be LLM-assisted OR rule-based.
Because Focusa must be harness-agnostic, MVP can start with:
- rule-based extraction (regex for file paths/errors)
- plus optional call to a cheap summarizer model if configured (NOT required)

You must implement a pluggable summarizer interface:
- `Summarizer::summarize_delta(existing_checkpoint, delta_input) -> delta_sections`

## Merge Rules (Deterministic)
For each slot:

- `intent`:
  - if empty -> set from delta
  - else update only if delta contains explicit intent change marker

- `current_focus`:
  - update with the latest concise statement (replace)

- `decisions`:
  - append new unique bullets; dedupe by normalized text
  - cap length (default 30 items)

- `artifacts`:
  - append new artifact lines; dedupe by (kind + path_or_id + label)
  - cap length (default 50 lines)

- `constraints`:
  - append unique constraints; cap 30

- `open_questions`:
  - append unique; if question is answered in delta, remove it (simple match heuristic)
  - cap 20

- `next_steps`:
  - replace with latest suggested steps derived from active frame state
  - cap 15

- `recent_results`:
  - keep last 10 results, newest first

- `failures`:
  - append failure bullets; cap 20

- `notes`:
  - append; cap 20; decay oldest first

Always update:
- `revision += 1`
- `anchor_turn_id = turn_id`
- `updated_at = now`

Emit event:
- `ascc.delta_applied`

## Prompt Serialization (for Prompt Assembly)
ASCC must serialize into a compact slot form (no fluff):

Example serialization (messages format):
System:
- “You are operating within Focusa.”
User:
- `FOCUS_FRAME: <title>`
- `INTENT: ...`
- `CURRENT_FOCUS: ...`
- `DECISIONS: ...`
- `ARTIFACTS: ...`
- `CONSTRAINTS: ...`
- `OPEN_QUESTIONS: ...`
- `NEXT_STEPS: ...`

MVP: Provide two serializers:
- `to_string_compact()`
- `to_messages_slots()`

## Persistence
Checkpoint per frame stored in:
- `~/.focusa/ascc/<frame_id>.json`

Optionally keep a bounded revision history:
- `~/.focusa/ascc/<frame_id>/revisions/<rev>.json` (non-MVP)

MVP: only current checkpoint required.

## Acceptance Tests
- Delta updates never overwrite unrelated slots.
- Anchors advance monotonically.
- Prompt serialization stable and bounded.
- Large tool outputs become handles (never inline).

---

# UPDATE

# docs/07-ascc.md (UPDATED) — Pinning & Degradation Hooks

## ASCC Section Pinning (Added)

### Pinned Sections
Any ASCC section may be marked pinned.

Pinned sections:
- cannot be dropped during prompt degradation
- are immune to slot-priority eviction

### Section Metadata
Each slot now has:
- `pinned: bool`
- `last_updated_at`

---

## Prompt Degradation Hooks (Added)

ASCC must expose:
- `to_digest()` → ultra-compact fallback summary

Used only when:
- prompt budget cannot be satisfied

---

## Invariants
- ASCC degradation is explicit
- ASCC never silently truncates pinned sections
