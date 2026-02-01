# docs/11-prompt-assembly.md — Prompt Assembly & Token Discipline (MVP)

## Purpose
Prompt Assembly is the **critical path** of Focusa.

Its job:
> Construct the smallest possible prompt that preserves task fidelity and intent.

Prompt Assembly MUST be:
- deterministic
- bounded
- focus-aware
- harness-agnostic

---

## Inputs
Prompt Assembly consumes:
- active Focus Frame
- parent Focus Frames (bounded)
- ASCC checkpoints
- selected semantic memory
- selected procedural rules
- handles (ECS)
- raw user input
- harness formatting requirements

---

## Prompt Budgeting

### Budget Contract
Configurable per adapter:
- `max_prompt_tokens` (default 6000)
- `reserve_for_response` (default 2000)

Assembly must **never exceed** budget.

If budget is exceeded:
1. drop lowest-priority parent frames
2. drop non-essential ASCC slots
3. truncate rehydrated handles
4. fail only as last resort

Emit `prompt.assembled` with warnings.

---

## Slot-Based Structure (Canonical)

Prompt is assembled into **fixed slots** in this order:

1. SYSTEM HEADER
2. OPERATING RULES (procedural)
3. ACTIVE FOCUS FRAME
4. PARENT CONTEXT (optional, bounded)
5. ARTIFACT HANDLES
6. USER INPUT
7. EXECUTION DIRECTIVE

---

### 1. System Header
Static, short.

Example:
```
You are operating within Focusa, a cognitive runtime that maintains focus over time.
Follow the structured context below.
```

---

### 2. Operating Rules (Procedural Memory)
Include at most 5 rules.

Format:
```
RULES:
- Prefer concise, step-based responses.
- Avoid unnecessary verbosity.
```

Rules ordered by:
- scope relevance
- weight

---

### 3. Active Focus Frame (ASCC)
Serialize ASCC slots for active frame.

Format:
```
FOCUS FRAME: <title>
INTENT:
CURRENT_FOCUS:
DECISIONS:
ARTIFACTS:
CONSTRAINTS:
OPEN_QUESTIONS:
NEXT_STEPS:
```

Omit empty slots.

---

### 4. Parent Context (Optional)
Include up to N ancestors (default 2).

Rules:
- include only `intent`, `decisions`, `constraints`
- omit artifacts unless explicitly required

Format:
```
PARENT CONTEXT:
FRAME: <title>
INTENT:
DECISIONS:
```

---

### 5. Artifact Handles
List handles referenced in ASCC.

Format:
```
ARTIFACT REFERENCES:
- [HANDLE:diff:abcd1234 "auth_refresh.patch"]
- [HANDLE:log:efgh5678 "build_log"]
```

Never inline content here.

---

### 6. User Input
Raw, unmodified user input.

---

### 7. Execution Directive
Single sentence.

Example:
```
Respond with the next best step to advance the current focus.
```

---

## Formats Supported (MVP)
- `string`
- `chat_messages[]`

Adapter chooses format.

---

## Delta Injection Rule
Prompt Assembly must:
- reuse prior assembled prompt context
- inject **only changes** since last turn where possible
- avoid repeating static headers/rules unnecessarily

MVP implementation:
- cache last assembled prompt hash
- if unchanged sections → reuse string slices
(This can be optimized later; correctness first.)

---

## Handle Rehydration (Optional)
If user or harness explicitly requests:
- rehydrate handle with `max_tokens`
- include snippet under:
```
REHYDRATED ARTIFACT (TRUNCATED):
...
```

Never auto-rehydrate in MVP.

---

## Events
Emit:
- `prompt.assembled`
Payload:
- estimated_tokens
- budget_target
- dropped_sections[]
- handles_used[]

---

## Acceptance Tests
- Prompt size stabilizes across turns
- No duplicated ASCC content
- Handles never inline by default
- Budget enforcement deterministic

---

# UPDATE

# docs/11-prompt-assembly.md (UPDATED) — Explicit Degradation Strategy

## Prompt Degradation Strategy (MANDATORY)

When prompt exceeds budget, apply **exactly in this order**:

1. Drop parent frames beyond depth limit
2. Drop non-pinned ASCC slots
3. Replace ASCC with `checkpoint_digest`
4. Truncate user input with explicit marker
5. Abort with error (last resort)

---

## Events
Emit:
- `prompt.degraded`
Payload:
- reason
- steps_taken[]
- final_token_estimate

---

## CLI / UI Behavior
- CLI prints warning
- UI shows subtle degraded-state indicator

---

## Forbidden
- Silent truncation
- Random dropping
- Model-dependent heuristics
