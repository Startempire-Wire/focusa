# docs/19-intentional-cache-busting.md — Intentional Cache Busting Triggers (AUTHORITATIVE)

This document defines the triggers and procedures for intentionally breaking:
- internal assembly caches (C1/C2)
- provider KV/prompt cache (C3)

The goal is to ensure prompt caching never creates:
- staleness
- salience collapse
- authority mismatch
- “intelligence per token” degradation

---

## 0. Canonical Principle

> **Cache busting is a correctness feature.**  
> We break caches to preserve meaning.

---

## 1. Categories of Bust Triggers

### Category A — Fresh Evidence Arrived (Always Bust)
When new information arrives that could materially change decisions.

Examples:
- user attaches a new file / diff / snippet
- tool output includes new errors / stack traces
- repo HEAD changes
- new Beads task selected / unblocked
- new constraints explicitly stated by user

Bust:
- C1 prompt assembly
- C1 context pack compilation
- C2 retrieval rankings
- C3 provider KV (if it would exclude the new evidence)

---

### Category B — Authority Boundary Changed (Always Bust)
When the permission posture changes.

Examples:
- autonomy level changes
- human approval required toggled
- new policy constraints loaded
- task authority context changes (Beads scope shift)

Bust:
- all C1/C2
- always rebuild prompt from Focus State

---

### Category C — Compaction / Summary Inserted (Always Bust)
When CLT compaction or anchored summaries are inserted.

Reason:
- summaries change the minimal context representation
- stale prefix reuse can reintroduce outdated material

Bust:
- C1 prompt assembly
- C1 context pack compilation
- C3 provider KV if compaction changed the prefix block content

---

### Category D — Staleness Risk Detected (Should Bust)
When cached scaffolding likely encodes outdated assumptions.

Signals:
- UFI rises over last N interactions for the same task
- repeated “rephrase” / “no that’s not what I meant”
- repeated wrong-file edits in coding workflows
- agent references old artifacts despite new attachments

Bust:
- C1/C2 always
- C3 optionally (if stable prefix is biasing behavior)

---

### Category E — Salience Collapse Detected (Should Bust)
When prompt becomes long but low-signal.

Signals:
- token budget pressure increases
- high ratio of “history” to “current state”
- Focus Gate indicates low confidence due to diluted relevance
- frequent retrieval misses followed by manual user correction

Response:
- force compaction
- rebuild prompt minimalistically
- accept cache miss

Bust:
- C1/C2
- C3 if it incentivizes append-only behavior

---

### Category F — Provider Cache Mismatch / TTL / Routing (May Bust)
Provider caches may expire unpredictably.

Signals:
- unexpected prefill latency spikes
- inconsistent cache-hit telemetry (when available)
- harness switched implicitly

Response:
- do not attempt to preserve provider cache at the expense of correctness
- rebuild deterministic prompt anyway

Bust:
- none required; treat as opportunistic

---

## 2. “Busting” Procedures (Implementation)

### 2.1 Internal Cache Bust (C1/C2)
- Delete entries keyed by:
  - focus_state_hash
  - retrieval_policy_version
  - token_budget
- Or mark them invalid by incrementing:
  - `focus_state_revision`
  - `prompt_assembly_revision`

Preferred: revision bump (audit-friendly).

---

### 2.2 Provider KV Cache Bust (C3)
Provider caching is black-box. Busting means:
- altering the prefix in a controlled way OR
- bypassing reliance on prefix reuse

Approved methods:
1) **Prefix refresh marker** (deterministic)
   - include a stable “refresh nonce” segment that changes only when required
2) **Strict separation**
   - keep static scaffolding minimal
   - move dynamic content outside cached prefix
3) **Forced repack**
   - reassemble prompt with a new ordering when correctness requires

Note:
- Never insert random noise
- Must remain deterministic and auditable

---

## 3. UI/CLI Surface

### CLI
- `focusa cache status`
- `focusa cache bust --reason=<...>`
- `focusa cache policy show`

### GUI (Menubar)
- “Freshness mode” toggle: Prefer correctness (default ON)
- Cache hit/miss telemetry (read-only)
- Recent bust reasons (navigable)

---

## 4. Telemetry & Accountability

Each bust event records:
- timestamp
- category (A–F)
- reason
- impacted cache classes
- recompute cost

This becomes evidence for:
- performance tuning
- CS synthesis
- autonomy scoring interpretation

---

## 5. Canonical Rule

> **We bust caches when the system risks being wrong, stale, or miscalibrated — even if it costs tokens.**
