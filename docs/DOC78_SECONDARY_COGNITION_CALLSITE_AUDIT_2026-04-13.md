# Doc 78 Secondary/Background Cognition Call-Site Audit — 2026-04-13

Purpose:
- inventory real secondary/background cognition call sites
- distinguish heuristic vs model-backed vs mixed paths
- assess whether the current subconscious/background layer is structured or gibberish-prone

## Summary judgment

Current repo evidence shows:
- **one confirmed model-backed background/reflection path** in `crates/focusa-api/src/routes/reflection.rs`
- **multiple heuristic/autonomy substrate paths** in autonomy state, scheduler guardrails, confidence thresholds, repeated-recommendation suppression, and decay/checkpoint/governance infrastructure
- **no evidence yet of a broad multi-call-site structured subconscious layer** with several distinct high-quality prompt programs

So the honest current status is:
- not zero
- but still **thin and fragile**
- with real risk that later doc-78 language overstates how structured the background layer currently is

---

## Call-site inventory

### 1. Reflection LLM call
**File:** `crates/focusa-api/src/routes/reflection.rs`

**Key evidence:**
- comment at line ~475: `Call MiniMax M2.7 directly for reflection analysis.`
- POST to `https://api.minimax.io/v1/chat/completions`
- model: `MiniMax-M2.7`
- message format: single `{"role":"user","content": prompt}`

**Classification:**
- **model-backed**
- **background/secondary cognition candidate**
- **structured but still thin**

**Prompt structure:**
The prompt includes:
- recent event summary
- focus state summary
- semantic memory keys
- procedural rules
- required JSON schema for `observations`, `risks`, `recommendations`, `confidence`
- explicit focus areas: stale/stuck frames, repetitive patterns, memory gaps, contradictions, proactive opportunities

**Assessment:**
- better than gibberish because it asks for strict JSON and named analysis categories
- still somewhat fragile because:
  - single freeform user message, no separate system/user role design
  - broad summarization inputs may be low-signal or lossy
  - little evidence yet of deeper prompt-governance/eval around output quality

### 2. Reflection execution logic
**File:** `crates/focusa-api/src/routes/reflection.rs`

**Classification:**
- **mixed path**

**Why mixed:**
The final reflection result combines:
- heuristic baseline observations (`focus_stack_depth`, `event_count`)
- heuristic risks (e.g. `no_active_frame`)
- LLM observations/risks/recommendations when present
- fallback advisory recommendations when LLM output is absent
- heuristic confidence fallback (`0.82` if active frame else `0.66`)
- heuristic stop reasons (`repeated_recommendation_set`, `low_confidence`, `no_evidence_delta`, `single_iteration_complete`)

**Assessment:**
- not pure LLM cognition
- real structure comes from heuristic shell + optional LLM analysis
- quality depends heavily on whether the LLM succeeds and returns parseable JSON

### 3. Reflection scheduler / guardrails
**File:** `crates/focusa-api/src/routes/reflection.rs`

**Classification:**
- **heuristic-only**

**Evidence:**
- scheduler enable/disable config
- cooldown and per-window iteration limits
- low-confidence threshold
- no-delta minimum event delta
- repeated recommendation suppression

**Assessment:**
- this is background-execution governance, not cognition quality itself
- useful substrate for boundedness, but not proof of a rich subconscious layer

### 4. Autonomy state surfaces
**File:** `crates/focusa-api/src/routes/autonomy.rs`

**Classification:**
- **heuristic/state substrate**

**Evidence:**
- autonomy level exposure
- ARI score and dimensions
- recommendation from `focusa_core::autonomy::should_recommend_promotion`
- granted scope / TTL / history

**Assessment:**
- autonomy state exists
- but this is not itself a model-backed secondary cognition program
- should not be mistaken for doc-78 completion

### 5. Memory decay / checkpoint / governance hooks
**Files:**
- `crates/focusa-api/src/routes/commands.rs`
- related governance/proposal tests and routes

**Classification:**
- **heuristic/state substrate**

**Assessment:**
- relevant to persistence/boundedness/governance
- not themselves background reasoning call sites

---

## Gibberish-proneness assessment

### What reduces gibberish risk
- explicit JSON return contract
- named fields (`observations`, `risks`, `recommendations`, `confidence`)
- bounded categories of analysis
- parseability check with failure fallback
- heuristic shell around output consumption

### What still leaves fragility risk
- single prompt/program rather than multiple audited secondary programs
- single freeform user message shape
- uncertain quality of `events_summary` / focus summary inputs
- no strong evidence yet of replayed quality evals specifically for reflection output usefulness
- no clear prompt-version governance or anti-waffle quality gates

## Honest conclusion

The current background layer is **not pure gibberish theater**, because there is at least one structured model-backed reflection path with JSON-shape enforcement.

But it is also **not yet a mature structured subconscious layer**.
It is better described as:
- one real structured reflection call
- wrapped in heuristic guardrails/fallbacks
- with autonomy/governance substrate around it
- still needing stronger inventory, eval, and prompt-quality proof before later doc-78 claims can be treated as fully grounded

---

## Decomposition consequences

This audit most strongly justifies these doc-78 branches:
- `focusa-9l5h` call-site inventory
- `focusa-k50q` heuristic vs model-backed classification
- `focusa-rwlo` truthful trace/eval proof surfaces
- `focusa-adw9` reuse/extend/blocked/new mapping

It also suggests more decomposition is needed for:
- prompt-quality verification
- reflection usefulness evals
- operator-priority interaction between background reflection and current ask
