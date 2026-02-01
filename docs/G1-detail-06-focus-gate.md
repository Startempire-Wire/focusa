# docs/06-focus-gate.md — Focus Gate (RAS-Inspired) Specification (MVP)

## Purpose
Focus Gate is a **pre-conscious salience filter** that continuously shapes what is eligible to rise into priorities.

Key properties:
- advisory (never authoritative)
- fast (must not block hot path)
- organic (priorities emerge via repeated relevance, not fixed rules)
- model-agnostic (does not use transformer “attention” concept)

## Inputs/Outputs

### Inputs
A Focus Gate ingests **signals** from:
- adapter: user inputs, assistant outputs, errors, tool logs
- daemon: state changes (frame pushed/popped)
- worker: classification results, summary deltas, anomaly findings
- memory: reinforcement/decay ticks

### Outputs
Focus Gate produces **candidates** that can be surfaced to users:
- suggested next frame
- suggested issue to resolve
- suggested memory to pin
- suggested artifact to fetch

Focus Gate outputs are:
- read-only suggestions
- optionally accompanied by an internal “surface pressure” metric (not called attention)

## Core Concept: Candidate “Surface Pressure”
A candidate has a latent value called “surface pressure” (internal metric) that increases with:
- persistence (repeated occurrence)
- goal alignment to active frame or near ancestors
- risk signals (errors, contradictions)
- novelty spikes
and decreases with:
- suppression
- completion/resolution
- decay over time

The pressure is used to order candidates.

## Data Model

### Signal
Fields:
- `id: SignalId`
- `ts: timestamp`
- `origin: "adapter"|"worker"|"daemon"|"cli"|"gui"`
- `kind: SignalKind`
- `frame_context: Option<FrameId>` (active at time of signal)
- `summary: String` (short, <= 200 chars)
- `payload_ref: Option<HandleRef>` (if large; store in ECS)
- `tags: Vec<String>` (optional)

### SignalKind (MVP)
- `user_input`
- `assistant_output`
- `tool_output`
- `error`
- `warning`
- `artifact_changed`
- `repeated_pattern`
- `deadline_tick` (optional)
- `manual_pin` (user explicitly flags something)

### Candidate
Fields:
- `id: CandidateId`
- `created_at`
- `updated_at`
- `kind: CandidateKind`
- `label: String` (user-facing)
- `origin_signal_ids: Vec<SignalId>`
- `related_frame_id: Option<FrameId>`
- `state: CandidateState`
- `pressure: f32` (internal)
- `last_seen_at`
- `times_seen: u32`
- `suppressed_until: Option<timestamp>`
- `resolution: Option<String>` (when completed/dismissed)

### CandidateKind (MVP)
- `suggest_push_frame`
- `suggest_resume_frame`
- `suggest_check_artifact`
- `suggest_fix_error`
- `suggest_pin_memory`

### CandidateState
- `latent`
- `surfaced`
- `suppressed`
- `resolved`

## Focus Gate Algorithm (MVP Deterministic Heuristics)

### Step 1: Normalize signals
On ingest:
- if `payload_ref` missing and payload is large -> store to ECS -> set payload_ref
- derive tags:
  - error class
  - file path hints
  - tool name hints
- create a “fingerprint” for dedupe:
  - hash(kind + normalized summary + frame_context + key tags)

### Step 2: Candidate matching or creation
If fingerprint matches an existing candidate:
- increment `times_seen`
- update `last_seen_at`
- increase `pressure` by `Δp`
Else:
- create new candidate with base pressure.

### Step 3: Pressure update rules
Pressure update uses additive factors:

Base increments (example defaults):
- `user_input`: +0.6
- `tool_output`: +0.5
- `assistant_output`: +0.2
- `warning`: +0.7
- `error`: +1.2
- `repeated_pattern`: +0.8
- `manual_pin`: +2.0

Modifiers:
- goal alignment:
  - if related_frame == active: ×1.3
  - if related_frame in stack path: ×1.1
  - else ×0.8
- recency:
  - if within last 5 min: +0.3
- risk:
  - if error/warning: +0.4

Suppression:
- if suppressed_until in future: do not surface (still track but do not show).

Decay:
- on periodic tick, apply `pressure *= 0.98` (configurable) for non-manual candidates.
- if pressure below threshold and not seen in long time -> drop candidate (optional; or archive).

### Step 4: Surfacing
Candidates become “surfaced” when:
- pressure >= `SURFACE_THRESHOLD` (default 2.2)
AND
- not suppressed
AND
- not resolved

Surfacing does NOT change focus stack. It only:
- emits event `gate.candidate_surfaced`
- returns candidate in API/UI lists

### Step 5: User actions
Users can:
- suppress candidate (set suppressed_until)
- resolve candidate (state=resolved)
- convert candidate to a focus frame push (CLI command triggers HEC push)
  - This is important: conversion is a conscious action, not automatic.

## API Behavior
- `/v1/focus-gate/candidates`: returns sorted by:
  - `state` surfaced first
  - `pressure` descending
  - `last_seen_at` descending

- `/v1/focus-gate/ingest-signal`: must be fast (<5ms typical). If ECS store needed, store async and return 202 if needed.

## UI Mapping (MVP)
- Menubar shows:
  - number of surfaced candidates
  - top candidate label (optional)
- No auto-popup interruptions.

## Persistence
Persist candidate list with bounded size:
- Keep last N candidates (default 200)
- Persist to `~/.focusa/state/focus_gate.json`

## Acceptance Tests
- Duplicate signals merge into same candidate.
- Pressure increases predictably with repeated occurrences.
- Suppression hides candidate for the set period.
- Decay reduces stale candidates over time.
- No candidate action changes focus stack without explicit CLI/API call.

---

# UPDATE

# docs/06-focus-gate.md (UPDATED) — Pinning & Time Signals

## New Capability: Pinning

### Pinned Flag
Candidates, memory entries, and ASCC sections may be pinned.

Pinned items:
- ignore decay
- are always eligible for surfacing
- have minimum pressure floor

### Candidate (Updated)
Add:
- `pinned: bool`

### CLI
- `focusa gate pin <candidate_id>`
- `focusa gate unpin <candidate_id>`

---

## Time as a First-Class Signal (Added)

### Temporal Signals
- `inactivity_tick`
- `long_running_frame`
- `deadline_tick`

### Derived Heuristics (MVP)
- Frame open > N minutes → signal
- Candidate resurfacing over long interval → boost
- Explicit user deadline → hard signal

### Pressure Effects
- Long-running + unresolved increases pressure slowly
- Time decay slowed for pinned items

---

## Invariants
- Time signals never auto-switch focus
- Time only increases *eligibility*, not authority
