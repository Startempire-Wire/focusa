# docs/09-memory.md — Minimal Memory (Semantic + Procedural) (MVP)

## Purpose
Memory in MVP exists to:
- retain a small set of stable facts/preferences (semantic)
- encode stable behavioral preferences as rules (procedural)
- support Focus Gate and prompt assembly without token bloat

MVP memory is deliberately minimal and bounded.

## Memory Types (MVP)
1. Semantic Memory (facts/preferences)
2. Procedural Memory (rules / habits)
3. Decay mechanism

Not in MVP:
- episodic store
- schema emergence
- meta-memory

## Data Model

### MemoryState
- `semantic: HashMap<String, SemanticRecord>` (keyed by stable key)
- `procedural: HashMap<String, RuleRecord>`
- `version: u64`

### SemanticRecord
Fields:
- `key: String` (e.g., `user.response_style`)
- `value: String` (short)
- `created_at`
- `updated_at`
- `source: "user"|"worker"|"manual"`
- `confidence: f32` (optional; default 1.0 for user-set)
- `ttl: Option<duration>` (optional)
- `tags: Vec<String>`

MVP keys to support:
- `user.response_style` (e.g., concise steps)
- `project.name` (optional)
- `env.preferences` (optional)

### RuleRecord (Procedural)
Fields:
- `id: String` (stable rule id)
- `rule: String` (compact imperative)
- `weight: f32` (internal)
- `reinforced_count: u32`
- `last_reinforced_at`
- `scope: RuleScope`
- `enabled: bool`

### RuleScope
- `global`
- `frame:<frame_id>`
- `project:<name>` (optional; later)

## Memory Operations

### UpsertSemantic
- set or update a semantic record
- emit `memory.semantic_upserted`

### ReinforceRule
- increase rule weight
- emit `memory.rule_reinforced`

### DecayTick
Periodic:
- `rule.weight *= 0.99`
- if weight below threshold and not reinforced in long time -> disable or remove (configurable)
- emit `memory.decay_tick`

## How Memory Enters Prompt Assembly
Rule: memory must not become prompt bloat.

### Semantic Injection (MVP)
Only include whitelisted keys in prompt:
- response style
- explicit project constraints

Serialize as compact:
`PREFS: user.response_style=concise_steps`

### Procedural Injection (MVP)
Procedural rules are injected as “operating constraints”:

Example:
`RULES: Prefer concise bullet steps; avoid verbosity.`

Cap:
- at most 5 rules injected
- order by weight descending and scope relevance to active frame

## How Memory Is Created (MVP)
Memory can be created via:
- explicit user/CLI command (primary)
- background worker extraction (optional, conservative)
  - only extract stable preferences if repeated and confirmed

MVP default:
- only explicit user/CLI writes semantic memory
- worker may propose candidate memory updates via Focus Gate (suggest_pin_memory)

## Persistence
- `~/.focusa/state/memory.json`

## Acceptance Tests
- memory writes persist across restart
- prompt assembly includes only whitelisted semantic keys
- procedural rules bounded, ordered, and scoped
- decay tick reduces weights over time

---

# UPDATE

# docs/09-memory.md (UPDATED) — Trust Model

## Memory Trust Rules (Explicit)

1. Memory is **opt-in**
2. Memory writes require:
   - explicit user command OR
   - user-confirmed candidate promotion
3. Workers may only *suggest* memory

---

## Pinned Memory
Pinned memory:
- immune to decay
- always eligible for prompt inclusion (within caps)

---

## Non-Goals (Explicit)
- No automatic personality drift
- No silent preference learning
- No speculative inference
