# 44 — Pi × Focusa Integration Spec (Proxy-First, Extension-Thin)

Status: Draft for implementation
Owner: Focusa + Pi integration
Goal: Better long-session compaction + optional smarter model routing without cognitive drift

---

## 1) Executive Summary

Use **Focusa as the single cognitive authority**.

- Focusa Proxy/Daemon handles: ASCC, ECS, Focus Gate, prompt assembly, degradation, telemetry.
- Pi extension handles: UX glue + operator controls + observability.
- Pi does **not** maintain parallel memory/compaction state.

Result:
- Better compaction quality
- Lower context failures
- Cross-harness continuity (Pi, Claude Code, Codex, Letta)
- Optional stronger upstream model usage through Focusa routing

---

## 2) Objectives

1. Improve compaction fidelity in Pi sessions.
2. Prevent context-loss regressions in long sessions.
3. Keep behavior deterministic/auditable.
4. Enable model/provider upgrades via Focusa proxy path.
5. Avoid duplication between Pi and Focusa cognition.

## 3) Non-Goals

- No autonomous hidden memory writes.
- No random/opaque prompt mutations.
- No Pi-side clone of ASCC/ECS.

---

## 4) Canonical Design Principles

1. **Single source of cognition**: Focusa owns cognitive state.
2. **Deterministic prompt assembly**: fixed slots, fixed degradation order.
3. **Handles over blobs**: large artifacts externalized (ECS), never inlined by default.
4. **Fail-safe passthrough**: if Focusa unavailable, harness continues.
5. **Policy over convenience**: mutation only via explicit commands.

---

## 5) Architecture

## 5.1 Components

- Pi harness (interactive coding)
- Focusa adapter/proxy (`focusa wrap -- pi` or HTTP proxy mode)
- Focusa daemon (Capabilities API)
- Optional Pi extension (`focusa-pi-bridge.ts`) for UX controls only

## 5.2 Authority Split

### Focusa owns
- ASCC checkpointing (10-slot structure)
- ECS artifact storage + handle refs
- Focus Gate pressure/candidate surfacing
- Prompt assembly + budget degradation
- Telemetry + lineage

### Pi extension owns
- Display Focusa status in Pi UI
- Manual commands: pin/suppress/checkpoint/rehydrate (submitted via Focusa Commands API)
- Early warning notifications in session
- Local fallback helpers when proxy unavailable

### Write-path rule (authoritative)
- Preferred mutation path: `POST /v1/commands/submit` (auditable command envelope)
- Compatibility path (where exposed): direct helper endpoints (e.g. focus-gate pin/suppress aliases)
- No hidden writes from extension code

---

## 6) Integration Modes

1. **Mode A (recommended initially):**
   `focusa wrap -- pi`

2. **Mode B (advanced):**
   Pi -> Focusa HTTP proxy -> upstream provider

3. **Pi extension mode (optional):**
   API-first client; CLI fallback only for diagnostics.

---

## 7) Compaction Strategy (Best Practice)

## 7.1 Primary Compaction (Focusa)

- ASCC delta updates anchored by turn ID
- Fixed slots:
  - intent
  - current_focus
  - decisions
  - artifacts
  - constraints
  - open_questions
  - next_steps
  - recent_results
  - failures
  - notes

## 7.2 Continuous Pruning (lean session mode)

Compaction is not only a single large event. Use a continuous hygiene loop:

1. Per turn: enforce output discipline (truncate + externalize large blobs)
2. Every N turns: micro-compact stale low-value turn history
3. Keep pinned/critical ASCC sections intact
4. Reserve full compaction for threshold crossings or overflow recovery

This keeps sessions lean while preserving continuity.

## 7.3 Threshold-Driven Auto-Compaction (extension policy)

Policy tiers (configurable):
- Warn tier: 50% context usage
- Auto-compact tier: 70% context usage
- Hard tier: 85% context usage (compact + suggest fork/handoff)

Notes:
- compute usage from current token usage vs active model context window
- enforce cooldown + max compactions/hour to avoid thrash

## 7.4 ECS Offload Rules

Externalize when content exceeds either threshold:
- bytes > 8KB, or
- estimated tokens > 800

Prompt includes only handles:
`[HANDLE:<kind>:<id> "label"]`

## 7.5 Degradation Order (mandatory)

If prompt budget exceeded:
1. Drop parent frames beyond depth limit
2. Drop non-pinned ASCC slots
3. Replace ASCC with digest
4. Truncate user input with explicit marker
5. Abort (last resort)

No silent truncation.

---

## 8) Prompt Mutation Policy (Smarter Models, Safely)

Allowed:
- Deterministic slot composition
- Provider compatibility mutations only when explicitly enabled
- Explicit cache-bust markers when correctness requires

Forbidden:
- Random dynamic shaping
- Hidden intent rewrites
- Silent memory mutation

Compatibility toggle:
- `FOCUSA_PROXY_COMPAT_MODE=false` default
- `true` enables provider-specific sanitization/retry shims

---

## 9) Model Routing / "Smarter" Execution

Two safe paths:

1. **Context-first intelligence**
   - Keep same model
   - Improve output through better Focusa-assembled context

2. **Upstream model routing**
   - Point Focusa proxy to stronger provider/model
   - Keep harness unchanged

Rule: routing choices are explicit, logged, reversible.

---

## 10) Pi Extension Spec (Thin Bridge)

Name: `focusa-pi-bridge.ts`

## 10.1 API usage
- Primary: HTTP Capabilities API (`/v1/...`)
- Fallback: `focusa` CLI for diagnostics only

## 10.2 Extension Config Surface

Required config keys:
- `warnPct` (default 50)
- `compactPct` (default 70)
- `hardPct` (default 85)
- `cooldownMs` (default 180000)
- `maxCompactionsPerHour` (default 8)
- `minTurnsBetweenCompactions` (default 3)
- `compactInstructions` (default: preserve intent/decisions/constraints/next_steps/failures)
- `externalizeThresholdBytes` (default 8192)
- `externalizeThresholdTokens` (default 800)
- `focusaApiTimeoutMs` (default 5000)
- `fallbackMode` (`passthrough` default)
- `emitMetrics` (default true)

## 10.3 Commands (Pi)
- `/focusa-status`
- `/focusa-pin <candidate_id>`
- `/focusa-suppress <candidate_id> [duration]`
- `/focusa-checkpoint`
- `/focusa-rehydrate <handle_id> [max_tokens]`
- `/focusa-gate-explain <candidate_id>`
- `/focusa-stack`

Command mapping should prefer Capabilities API semantics and IDs; CLI fallback may map to:
- `focusa gate ...`
- `focusa state stack`
- `focusa ecs rehydrate ...`

## 10.4 UI widgets
- Focus pressure indicator
- Degraded-context badge
- Active frame + thesis summary snippet
- Compaction tier badge (warn/auto/hard)

## 10.5 Guardrails
- No local cognitive DB in extension
- No custom compactor that diverges from Focusa
- Read-heavy, command-light
- No hidden writes or silent context mutation

---

## 11) Reliability + Failure Handling

Failure modes:
1. Focusa unavailable before turn start
2. Focusa fails during assemble
3. Focusa drops mid-stream

Behavior (faithful to proxy specs):
1. passthrough to normal Pi behavior
2. emit visible warning
3. keep working; no hard stop
4. retain audit event for outage window

Recovery:
- auto-reconnect with backoff
- soft resync (state/info, recent candidates)
- never replay writes blindly; re-submit with idempotency keys where supported

---

## 12) Security & Policy

- Localhost-only API by default
- Bearer token required
- Capability-scoped tokens
- Commands require policy validation
- Auditable command IDs for all writes

---

## 13) Observability & KPIs

Track before/after:

1. context overflow incidents per 100 turns
2. retry rate due to provider/internal 5xx
3. compaction frequency and compaction regret
4. token usage per task
5. successful long-session completion rate
6. user interruption count for lost-context recovery

Success criteria:
- 50%+ drop in overflow/retry incidents in long sessions
- measurable reduction in manual “remind context” interventions

---

## 14) Rollout Plan

Phase 1 (quick win)
- run Pi through `focusa wrap -- pi`
- enable telemetry + strict assembly

Phase 2 (stability)
- tune budgets/degradation
- validate ECS thresholds on real coding sessions

Phase 3 (UX)
- add thin Pi extension commands/widgets

Phase 4 (smarter routing)
- route selected workloads to stronger upstream model via Focusa
- compare quality/cost metrics

---

## 15) Test Plan

1. Long-session stress test (200+ turns)
2. Large tool-output test (forced ECS handles)
3. Budget exceed test (verify exact degradation order)
4. Focusa outage test (passthrough correctness)
5. Cross-harness continuity test (Pi -> Claude Code -> Pi)
6. Threshold policy test: 50/70/85 tiers trigger expected actions
7. Continuous pruning test: session token growth remains bounded without frequent full compaction

---

## 16) Implementation Checklist

- [ ] Confirm Focusa daemon health + auth token flow
- [ ] Add Pi wrapper startup script
- [ ] Enable deterministic prompt assembly settings
- [ ] Validate ECS offload thresholds in live usage
- [ ] Build thin Pi bridge extension (API-first)
- [ ] Add telemetry dashboard panels
- [ ] Run A/B sessions and document outcomes

---

## 17) Final Decision

For your goal (“better compaction + smarter model utilization”), the aligned architecture is:

**Focusa proxy-first + thin Pi extension.**

Not extension-only.
Not duplicate cognition.

---

## 18) Extension Config Schema (single-doc authoritative)

Use project-local config in `.pi/settings.json` under `extensions.focusaPiBridge` (or env vars).

```json
{
  "extensions": {
    "focusaPiBridge": {
      "enabled": true,
      "warnPct": 50,
      "compactPct": 70,
      "hardPct": 85,
      "cooldownMs": 180000,
      "maxCompactionsPerHour": 8,
      "minTurnsBetweenCompactions": 3,
      "compactInstructions": "Preserve intent, decisions, constraints, next_steps, failures. Prefer handles over blobs.",
      "externalizeThresholdBytes": 8192,
      "externalizeThresholdTokens": 800,
      "focusaApiBaseUrl": "http://127.0.0.1:8787/v1",
      "focusaApiTimeoutMs": 5000,
      "fallbackMode": "passthrough",
      "emitMetrics": true,
      "autoSuggestForkPct": 90,
      "autoSuggestHandoffAfterNCompactions": 3
    }
  }
}
```

Validation rules:
- `0 < warnPct < compactPct < hardPct < 100`
- `cooldownMs >= 30000`
- `maxCompactionsPerHour >= 1`
- `externalizeThresholdBytes >= 2048`
- `externalizeThresholdTokens >= 200`

---

## 19) Env Var Mapping (override layer)

Environment overrides for automation/ops:

- `FOCUSA_PI_ENABLED=true|false`
- `FOCUSA_PI_WARN_PCT=50`
- `FOCUSA_PI_COMPACT_PCT=70`
- `FOCUSA_PI_HARD_PCT=85`
- `FOCUSA_PI_COOLDOWN_MS=180000`
- `FOCUSA_PI_MAX_COMPACTIONS_PER_HOUR=8`
- `FOCUSA_PI_MIN_TURNS_BETWEEN_COMPACTIONS=3`
- `FOCUSA_PI_COMPACT_INSTRUCTIONS="..."`
- `FOCUSA_PI_EXTERNALIZE_BYTES=8192`
- `FOCUSA_PI_EXTERNALIZE_TOKENS=800`
- `FOCUSA_PI_API_BASE_URL=http://127.0.0.1:8787/v1`
- `FOCUSA_PI_API_TIMEOUT_MS=5000`
- `FOCUSA_PI_FALLBACK_MODE=passthrough`
- `FOCUSA_PI_EMIT_METRICS=true`
- `FOCUSA_PI_AUTO_SUGGEST_FORK_PCT=90`
- `FOCUSA_PI_AUTO_SUGGEST_HANDOFF_AFTER=3`

Precedence:
1. Env vars
2. `.pi/settings.json`
3. extension defaults

---

## 20) Auto-Compaction Tier Behavior (alignment-correct)

Per `turn_end`:

1. compute `usagePct = currentTokens / contextWindow * 100`
2. apply cooldown / rate limit guard
3. tier actions:

- `usagePct >= warnPct` and `< compactPct`
  - notify only (no write action)

- `usagePct >= compactPct` and `< hardPct`
  - submit Focusa command to compact/refresh context package (authoritative path)

- `usagePct >= hardPct`
  - submit Focusa compaction command
  - show strong warning badge
  - suggest `/fork` or handoff command

Authoritative write path:
- `POST /commands/submit`

Local fallback (strictly optional):
- use `ctx.compact()` only when Focusa is unreachable **and** `fallbackMode` explicitly allows local compaction
- fallback compaction is marked degraded and never treated as canonical Focusa cognition

Guardrails:
- never compact twice within `cooldownMs`
- never exceed `maxCompactionsPerHour`
- respect `minTurnsBetweenCompactions`

---

## 21) Continuous Pruning Loop (alignment-correct)

Keep session lean continuously while preserving Focusa authority:

1. **Per tool result**
   - if output > thresholds: externalize to Focusa Reference/ECS path and replace with handle reference
2. **Per turn**
   - extension may hide/truncate display noise in Pi UI, but does not mutate canonical cognition
3. **Periodic micro-compact** (every N turns; default 5)
   - requested via Focusa command/API, not extension-owned summarization
   - preserve pinned/critical ASCC slots
4. **Threshold compact fallback**
   - full compaction at configured tiers via Focusa command path

Design intent:
- smooth token curve over time
- fewer emergency compactions
- stable long sessions
- zero split-brain between Pi and Focusa

---

## 22) API/Command Mapping Examples

### 22.1 Read calls (API-first)
- `GET /state/current`
- `GET /state/stack`
- `GET /gate/scores`
- `GET /gate/explain?candidate_id=<id>`

### 22.2 Write calls (preferred envelope)
- `POST /commands/submit`

Example (pin candidate):
```json
{
  "command": "gate.pin",
  "args": { "candidate_id": "cand_123" },
  "idempotency_key": "pin-cand_123-20260317T1300Z"
}
```

Example (suppress candidate):
```json
{
  "command": "gate.suppress",
  "args": { "candidate_id": "cand_123", "duration": "10m" },
  "idempotency_key": "suppress-cand_123-20260317T1301Z"
}
```

### 22.3 CLI fallback examples
- `focusa gate explain <candidate_id>`
- `focusa state stack --json`
- `focusa ecs rehydrate <handle_id> --max-tokens 300`

---

## 23) Tuning Presets

### 23.1 Balanced (default)
- `warnPct=50`, `compactPct=70`, `hardPct=85`
- `cooldownMs=180000`
- `maxCompactionsPerHour=8`

### 23.2 Aggressive (very long sessions)
- `warnPct=40`, `compactPct=60`, `hardPct=75`
- `cooldownMs=120000`
- `maxCompactionsPerHour=12`

### 23.3 Conservative (preserve full conversational flow)
- `warnPct=60`, `compactPct=80`, `hardPct=92`
- `cooldownMs=300000`
- `maxCompactionsPerHour=5`

---

## 24) Minimal Implementation Pseudocode

```ts
onTurnEnd(ctx) {
  const usage = ctx.getContextUsage();
  if (!usage || !usage.tokens) return;

  const window = getActiveModelContextWindow(ctx); // from model registry/config
  if (!window) return;

  const pct = (usage.tokens / window) * 100;
  if (isRateLimited()) return;

  if (pct >= hardPct) {
    submitFocusaCommand("context.compact", {
      tier: "hard",
      instructions: compactInstructions,
      idempotency_key: makeKey("hard", pct),
    });
    notify("hard", pct);
    suggestForkOrHandoff();
    return;
  }

  if (pct >= compactPct) {
    submitFocusaCommand("context.compact", {
      tier: "auto",
      instructions: compactInstructions,
      idempotency_key: makeKey("auto", pct),
    });
    notify("compact", pct);
    return;
  }

  if (pct >= warnPct) {
    notify("warn", pct);
  }
}
```

Fallback branch (optional, explicit): if Focusa unavailable and fallback mode allows, run local `ctx.compact()` and tag as non-canonical fallback.

---

## 25) Acceptance Addendum (config + behavior)

Must pass:
1. Config validation rejects invalid tier ordering.
2. Tier behavior exactly matches `warn/compact/hard` policy.
3. Continuous pruning reduces mean context growth slope.
4. No hidden mutation paths; all writes auditable.
5. Passthrough behavior remains functional under Focusa outage.
6. Extension performs no canonical summarization/memory writes outside Focusa command/API path.
7. Local fallback compaction (if enabled) is visibly marked non-canonical and reconciled on reconnect.

---

## 26) Reality Check — Current Gaps (Observed)

This section captures current observed behavior vs intended architecture.

Observed in live environment:
1. Focusa runtime active with high event volume, but Pi integration path appears inconsistent.
2. Turn telemetry shows frequent `prompt_tokens=null` / `completion_tokens=null` at turn level.
3. Focus Gate candidate list often empty despite ingested warning/user signals.
4. Stack discipline not consistently automatic (active frame may be missing until manually pushed).
5. CLT/turn payload quality includes noisy terminal/control-sequence artifacts in historical entries.
6. Large directive payloads are ingested as raw user input, likely diluting salience quality.

Risk:
- System appears operational but does not consistently deliver intended cognitive governance outcomes for Pi sessions.

Immediate containment:
- Run controlled validation sessions via `focusa wrap -- pi` only.
- Require active frame + beads mapping at session start.
- Add payload normalization guard before signal/CLT ingestion.

---

## 27) Spec Fidelity Audit Plan (Code vs Spec)

Objective:
- Determine implementation faithfulness to original Focusa specs before further extension work.

Scope:
- Docs baseline: `docs/INDEX.md` canonical set + G1 detail docs.
- Code baseline: `crates/focusa-core`, `crates/focusa-api`, `crates/focusa-cli`, adapter paths.

Method:
1. Build a requirement matrix from canonical docs:
   - Focus Stack invariants
   - ASCC slot/update/merge invariants
   - ECS handle/offload/rehydration invariants
   - Prompt assembly/degradation invariants
   - Focus Gate pressure/surfacing invariants
   - command/audit/write-path invariants
2. Map each requirement to code location(s).
3. Classify each as:
   - ✅ implemented + tested
   - ⚠️ partial / behavior drift
   - ❌ missing
4. Produce evidence (file:path + function + test reference).
5. Prioritize remediation by impact on Pi integration.

Deliverable:
- Single audit report section in this doc with:
  - fidelity score by subsystem
  - top drift items
  - required fixes before extension GA

Exit gate before extension GA:
- No critical drift in Focus Stack, ASCC, ECS, prompt assembly, or write-path auditability.
- Controlled Pi run demonstrates deterministic context governance end-to-end.



---

## 28) Session Scoping Rule (AUTHORITATIVE)

**All Focusa API calls from the Pi extension are scoped to the current Pi session.**

The extension does not access, display, or act on:
- Wirebot's operator model (Pairing Engine drift, RABIT, DISC, emotional features)
- Wirebot's scoreboard (ships, season, lanes, score)
- Wirebot's trust metrics (fabrication count, corrections, compliance)
- Wirebot's Context Core state (interruptibility, circadian, RescueTime)
- Wirebot's Mem0 memories (operator facts, preferences, cross-session recall)
- Wirebot's Kaizen reflections
- Wirebot's SOUL.md or behavioral doctrine

Focusa serves multiple agents. Each gets its own session boundary. Pi sees only Pi's cognition:
- Pi's Focus State (intent, decisions, constraints, failures for this session)
- Pi's Focus Stack (frames pushed during this session)
- Pi's ARI score (autonomy earned by this session's turn quality)
- Pi's Thread Thesis (what this Pi session is about)
- Pi's procedural rules (scoped to active frame)
- Pi's semantic memory (project-scoped facts)
- Pi's ECS handles (artifacts created or referenced in this session)
- Pi's gate candidates (surfaced during this session)
- Pi's events (emitted during this session)

**Exception: `/wbm` (Wirebot Mode)**

When the operator explicitly activates `/wbm on`, the extension bridges Wirebot context INTO the Pi session and catalogues Pi's work meta BACK to Wirebot's systems. See §29.

Without `/wbm`, Pi and Wirebot are completely isolated sessions on the same Focusa daemon.

---

## 29) Wirebot Mode (`/wbm`) — Cross-Surface Identity Bridge

**Wirebot is one person across all surfaces.** Pi sessions are Wirebot working with different hands. The work must come home.

`/wbm` is a Pi slash command that optionally attaches Wirebot's context and catalogues work back.

### Activation

```
/wbm on          → Inject Wirebot context + start cataloguing
/wbm off         → Return to pure Pi mode
/wbm status      → Show injection state + catalogued items count
/wbm deep        → Also fetch Mem0 memories + wiki decisions (~1500 tok)
/wbm flush       → Force-catalogue accumulated work meta now
/wbm decisions   → Show decisions catalogued this session
/wbm ships       → Show git ships auto-detected by scoreboard during this session
```

### Inbound Context (read-only, ~500 tokens)

Injected into Pi's system prompt via `before_agent_start` hook:

| Source | API | Timeout | Fallback |
|---|---|---|---|
| Operator state | `GET :7400/me` | 2s | "Context Core: unavailable" |
| Objectives | Read `objectives.yaml` | 1s | Skip |
| Drift + season | `GET :8100/v1/score` (auth'd) | 2s | Skip |
| Active Focusa frame | `GET :8787/v1/focus/stack` | 2s | Skip |
| SOUL.md pillars | Read file (first ~500 chars) | 1s | Static fallback |
| Recent wiki decisions | `wb wiki search "tag:decision" --limit 3` | 3s | Skip |

Injected format:
```
[WIREBOT MODE ACTIVE]
Operator: Verious (Pacific), mode=agent_coding, interruptibility=very_low
Time: 03:22 Thursday, phase=sleeping, quiet hours
Drift: 14 (disconnected)
Season: S1 0W-21L, streak 21d
P1: TEP Book — "Publishable 12-chapter manuscript"
Active Frame: [from Focusa if present]
Pillars: Human first > Calm > Rigor > Radical Truth > Deep Clarity
Banned: No helplessness, no fabrication, no ask-back when context exists
```

### Outbound Cataloguing (work meta → Wirebot systems)

On `agent_end` hook, the extension receives `event.messages` — the **full conversation from this prompt cycle** (not just the last exchange). This is the live extraction path:

1. Collect all messages from `event.messages`
2. Filter: keep user + assistant messages with >50 chars (skip tool noise)
3. If >20 messages: chunk into windows of 15-20 turns
4. For each chunk: call MiniMax M2.7 for structured extraction (≤500 tok, 2s timeout)
5. Extracted memories route through `QueueMemoryForApproval()` into WINS portal queue
6. Operator reviews/approves in WINS portal → delivery to all 5 sinks (Mem0, MEMORY.md, fact YAML, wiki, Letta)

**This replaces the need for a cold Pi session JSONL parser for all future sessions.** Historical .jsonl backfill is a separate one-time batch job.

| Work Meta | WINS Queue | After Approval → Sinks |
|---|---|---|
| Decision | `QueueMemoryForApproval()` with `source_type:pi_session` | Mem0 + MEMORY.md + fact YAML + wiki + Letta |
| Fact | `QueueMemoryForApproval()` with `source_type:pi_session` | All 5 sinks |
| Failure | `QueueMemoryForApproval()` + Focusa focus state update | All 5 sinks + Focus State |
| Learning | `QueueMemoryForApproval()` with `source_type:pi_session` | All 5 sinks |

**Ships are NOT manually catalogued.** The scoreboard already auto-detects ships from git (GitHub webhooks + git discovery scanning `/root`, `/home`, `/data`). Pi commits code → scoreboard detects WHAT shipped → `/wbm` catalogues WHY.

### Write Safety

- All writes go through `wb` CLI (auditable, rate-limited)
- Mem0 writes use promotion pipeline — not raw injection
- Wiki writes are schema-validated
- All items tagged `source:pi` + `surface:pi` for provenance
- LLM extraction failure → skip cataloguing for this turn (no partial writes)
- Operator can disable cataloguing: `/wbm on --no-catalogue`

### What This Enables

Without `/wbm`: Pi does 8 hours of coding. Wirebot knows nothing. Decisions vanish.
With `/wbm`: Wirebot recalls "yesterday in a Pi session we decided X" — even though Wirebot wasn't coding.

---

## 30) Metacognitive Awareness (Organism Spec Alignment)

Per `UNIFIED_ORGANISM_SPEC.md §11`, Focusa performs background LLM-backed metacognitive work after each turn:
- LLM extraction (replacing regex workers)
- Post-turn evaluation (quality + consistency check)
- Thread Thesis refinement (every Nth turn)
- Anticipatory queries (pre-fetch for next turn)

The Pi extension should be **aware** of this background work:

### UX Indicators

| Indicator | When | Display |
|---|---|---|
| "🧠 thinking..." | Background workers running after response sent | Subtle footer status |
| "📝 extracted N decisions" | Extraction worker completed | Brief notification |
| "🎯 thesis updated" | Thread Thesis refined | Footer status update |
| "⚠️ quality flag" | Post-turn evaluation found issues | Visible warning |

### Rules

- These indicators are **informational only** — the extension does not control metacognitive work
- Background work runs on Focusa daemon, not in the extension
- Extension polls `GET /v1/events/stream` (SSE) for real-time awareness
- If background work fails, the extension shows nothing (fail silent)

---

## 31) Port Correction

All references to `:8787` in this spec should read `:8787`.

Focusa daemon canonical bind: `127.0.0.1:8787`

Update in §18 config schema:
```json
"focusaApiBaseUrl": "http://127.0.0.1:8787/v1"
```

Update in §19 env vars:
```
FOCUSA_PI_API_BASE_URL=http://127.0.0.1:8787/v1
```

---

## 32) Pi Harness Type

The Focusa adapter (`crates/focusa-core/src/adapters/letta.rs`) should add an explicit `Pi` variant to `HarnessType`:

```rust
pub enum HarnessType {
    Letta,
    ClaudeCode,
    CodexCli,
    GeminiCli,
    Pi,              // ← NEW
    Generic(String),
}
```

Pi-specific capabilities to declare:
- `session_management: true` (Pi has `/compact`, `/fork`, session persistence)
- `tool_output_structured: true` (Pi captures tool results with structured metadata)
- `slash_commands: true` (Pi supports `/command` registration)
- `extension_api: true` (Pi has full extension lifecycle hooks)

This enables Focusa to tailor behavior for Pi sessions (e.g., different compaction strategies, awareness of Pi session management).

---

## 33) Novel Integration Opportunities (Pi Extension API Deep Audit)

Audited 2026-04-02 against full Pi extension docs (2232 lines, extensions.md) and all Focusa specs. These are integration points the original spec missed or underutilized.

### 33.1 Focusa-Owned Compaction via `session_before_compact`

**Pi's compaction** generates a generic LLM summary when context exceeds threshold.
**Focusa's ASCC** maintains a structured 10-slot state (intent, decisions, constraints, failures, next_steps, etc.) that is FAR richer.

**These are parallel summarization systems that don't know about each other.**

**Integration:** Use `session_before_compact` to replace Pi's compaction with Focusa's ASCC:

```typescript
pi.on("session_before_compact", async (event, ctx) => {
  try {
    const ascc = await fetch("http://127.0.0.1:8787/v1/ascc/state", {
      signal: AbortSignal.timeout(3000)
    }).then(r => r.json());
    
    const summary = [
      `# Focusa Cognitive Summary`,
      `## Intent\n${ascc.focus_state?.intent || "none"}`,
      `## Current Focus\n${ascc.focus_state?.current_state || "none"}`,
      `## Decisions Made\n${(ascc.focus_state?.decisions || []).map(d => `- ${d}`).join("\n") || "none"}`,
      `## Active Constraints\n${(ascc.focus_state?.constraints || []).map(c => `- ${c}`).join("\n") || "none"}`,
      `## Failures Encountered\n${(ascc.focus_state?.failures || []).map(f => `- ${f}`).join("\n") || "none"}`,
      `## Next Steps\n${(ascc.focus_state?.next_steps || []).map(n => `- ${n}`).join("\n") || "none"}`,
      `## Open Questions\n${(ascc.focus_state?.open_questions || []).map(q => `- ${q}`).join("\n") || "none"}`,
    ].join("\n\n");
    
    return {
      compaction: {
        summary,
        firstKeptEntryId: event.preparation.firstKeptEntryId,
        tokensBefore: event.preparation.tokensBefore,
      }
    };
  } catch {
    // Focusa unavailable — fall through to Pi's default compaction
    return;
  }
});
```

**Why this matters:** Decisions never vanish on compaction because they live in Focus State. Pi's compaction becomes Focusa-aware.

**Fallback:** If Focusa is down, return undefined → Pi compacts normally.

### 33.2 Per-LLM-Call Context Injection via `context` Event

**Gap:** `before_agent_start` fires once per user prompt. But `context` fires before EVERY LLM call — including after each tool call in a multi-tool turn.

**Integration:** Inject live Focus State into every LLM call, not just the first:

```typescript
pi.on("context", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  const state = await fetchFocusaState(); // cached with 5s TTL
  if (!state) return;
  
  // Prepend Focus State as first message
  const focusaMsg = {
    role: "user" as const,
    content: [{ type: "text" as const, text: `[FOCUSA LIVE STATE]\nFrame: ${state.title}\nDecisions: ${state.decisions?.join("; ")}\nConstraints: ${state.constraints?.join("; ")}` }],
    timestamp: Date.now(),
  };
  
  return { messages: [focusaMsg, ...event.messages] };
});
```

**Why this matters:** After a tool modifies a file, the Focus State's `current_focus` may have updated from a background worker. The next LLM call should see the updated state, not the stale one from `before_agent_start`.

### 33.3 Automatic ECS Externalization via `tool_result`

**Gap:** Doc 44 §21 describes periodic externalization. But Pi's `tool_result` hook can modify results **per tool call** — cleaner and more immediate.

**Integration:**

```typescript
pi.on("tool_result", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  const textContent = event.content
    ?.filter(c => c.type === "text")
    .map(c => c.text)
    .join("") || "";
  
  if (textContent.length > 8192) { // ECS threshold
    try {
      const handle = await fetch("http://127.0.0.1:8787/v1/ecs/store", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          kind: "text",
          label: `${event.toolName}-output-${Date.now()}`,
          content: textContent
        }),
        signal: AbortSignal.timeout(3000),
      }).then(r => r.json());
      
      return {
        content: [{
          type: "text",
          text: `[Output externalized to Focusa ECS: HANDLE:${handle.id} "${event.toolName} output" (${textContent.length} bytes)]\nUse /focusa-rehydrate ${handle.id} to retrieve full content.`
        }],
      };
    } catch {
      // ECS unavailable — pass through full content
      return;
    }
  }
});
```

### 33.4 Tool Usage Tracking for Autonomy via `tool_call`

**Integration:** Feed tool usage patterns into Focusa's Intuition Engine for pattern detection (repeated failures, tool-hopping, excessive writes):

```typescript
pi.on("tool_call", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  // Fire-and-forget — don't block tool execution
  fetch("http://127.0.0.1:8787/v1/gate/ingest-signal", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      signal_type: "tool_usage",
      summary: `Pi tool: ${event.toolName}`,
      metadata: { tool: event.toolName, session: "pi" }
    }),
    signal: AbortSignal.timeout(1000),
  }).catch(() => {}); // Truly fire-and-forget
});
```

### 33.5 Mode B Without Proxy via `before_provider_request`

> **⚠️ SUPERSEDED by §36.6:** This injection mode is DISABLED when `context` event injection (§33.2) is active. They are mutually exclusive. Use §33.2 (lighter, per-call refresh) as the primary injection path. This section is retained as documentation of the alternative approach only.

**Gap:** Doc 44 describes Mode A (focusa wrap) and Mode B (HTTP proxy). But Pi's `before_provider_request` enables Mode B **without a proxy** — Focusa's Expression Engine injects context directly into the LLM request payload.

**Integration:**

```typescript
pi.on("before_provider_request", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  try {
    const assembled = await fetch("http://127.0.0.1:8787/v1/prompt/assemble", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        turn_id: `pi-${Date.now()}`,
        raw_user_input: lastUserInput || "",
        format: "string"
      }),
      signal: AbortSignal.timeout(2000),
    }).then(r => r.json());
    
    if (assembled?.assembled && event.payload?.messages) {
      // Find system message and append Focusa context
      const sysMsg = event.payload.messages.find(m => m.role === "system");
      if (sysMsg) {
        sysMsg.content += "\n\n" + assembled.assembled;
      }
    }
  } catch {
    // Focusa unavailable — send request as-is
  }
  
  return event.payload;
});
```

**This is the highest-performance integration mode.** No proxy overhead, no `focusa wrap`, no Mode A shims. Pi calls the model directly, Focusa injects its cognition into every request.

### 33.6 Focusa as Registered Pi Provider via `pi.registerProvider`

**Integration:** Register Focusa's proxy as a selectable Pi model provider:

```typescript
pi.registerProvider("focusa", {
  baseUrl: "http://127.0.0.1:8787/proxy/v1",
  apiKey: "FOCUSA_AUTH_TOKEN",
  api: "openai-completions",
  models: [{
    id: "kimi-via-focusa",
    name: "Kimi K2.5 (via Focusa cognitive proxy)",
    reasoning: false,
    input: ["text"],
    contextWindow: 262144,
    maxTokens: 8192,
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
  }]
});
```

User can then `/model` to switch between direct and Focusa-proxied calls.

### 33.7 Session State Persistence via `pi.appendEntry`

**Gap:** Current spec doesn't persist Focusa state across Pi session restarts.

**Integration:** Save Focusa session reference in Pi session for resumability:

```typescript
// Save on WBM activation and periodically
function persistFocusaState() {
  pi.appendEntry("focusa-wbm-state", {
    sessionId: focusaSessionId,
    frameId: activeFrameId,
    wbmEnabled,
    cataloguedDecisions: [...cataloguedDecisions],
    cataloguedFacts: [...cataloguedFacts],
  });
}

// Restore on session resume
pi.on("session_start", async (_event, ctx) => {
  for (const entry of ctx.sessionManager.getEntries()) {
    if (entry.type === "custom" && entry.customType === "focusa-wbm-state") {
      const data = entry.data;
      if (data.wbmEnabled) {
        wbmEnabled = true;
        focusaSessionId = data.sessionId;
        cataloguedDecisions = data.cataloguedDecisions || [];
        cataloguedFacts = data.cataloguedFacts || [];
        // Attempt to resume Focusa session
      }
    }
  }
});
```

**Why this matters:** When you `/resume` a Pi session, `/wbm` state auto-restores. No need to re-activate.

### 33.8 Clean Session Close via `session_shutdown`

```typescript
pi.on("session_shutdown", async (_event, ctx) => {
  if (!focusaSessionActive) return;
  
  // Final flush of accumulated work meta
  await flushWorkMeta();
  
  // Persist final state
  persistFocusaState();
  
  // Close Focusa session
  await fetch("http://127.0.0.1:8787/v1/session/close", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reason: "pi_session_ended" }),
    signal: AbortSignal.timeout(2000),
  }).catch(() => {});
});
```

### 33.9 wb CLI via `pi.exec` Instead of HTTP fetch

For `wb` CLI calls, use `pi.exec` instead of `fetch`:

```typescript
async function wbWikiSearch(query: string): Promise<any> {
  const result = await pi.exec("wb", ["wiki", "search", query, "--format", "json"], {
    timeout: 5000
  });
  if (result.code !== 0) return null;
  try { return JSON.parse(result.stdout); } catch { return null; }
}

async function wbMemoryInject(text: string, source: string): Promise<boolean> {
  const result = await pi.exec("wb", ["memory", "inject", text], { timeout: 5000 });
  return result.code === 0;
}
```

**Why this is better:** `wb` handles its own auth, config, retries, and error formatting. More reliable than direct HTTP. Works even if service ports change.

### 33.10 Compaction Guidance (Softer Alternative to §33.1)

If full compaction replacement (§33.1) is too aggressive, inject Focusa state as **custom instructions** to guide Pi's native compaction:

```typescript
pi.on("session_before_compact", async (event, ctx) => {
  try {
    const state = await fetchFocusaState();
    if (!state) return;
    
    const instructions = [
      "PRESERVE these Focus State elements in the compaction summary:",
      `Intent: ${state.intent}`,
      `Decisions: ${state.decisions?.join("; ")}`,
      `Constraints: ${state.constraints?.join("; ")}`,
      `Failures: ${state.failures?.join("; ")}`,
      "DO NOT lose any decisions or constraints during compaction.",
      "Summarize tool outputs but keep decision rationale intact.",
    ].join("\n");
    
    return { customInstructions: instructions };
  } catch {
    return; // Fall through to default
  }
});
```

### 33.11 Recommended Integration Architecture

Based on these findings, the recommended Pi×Focusa integration uses **3 layers**, not 2:

```
Layer 1: Provider-level (§33.5 or §33.6)
  → Focusa context injected into every LLM request
  → No proxy, no wrap, pure extension

Layer 2: Event-level (§33.1–33.4, §33.7–33.8)
  → Compaction guided by Focusa ASCC
  → Tool results auto-externalized to ECS
  → Tool usage tracked for autonomy
  → Session state persisted for resumability

Layer 3: Command-level (doc 44 §10, §29)
  → /wbm for Wirebot cross-surface bridge
  → /focusa-* for manual inspection
  → /focusa-rehydrate for ECS retrieval
```

This replaces the need for `focusa wrap -- pi` entirely. The extension IS the integration.

---

## 34) Second-Pass Audit — Route Corrections and Untapped Capabilities

Audited 2026-04-02: Full re-read of Pi extension docs (extensions.md, session.md, compaction.md, skills.md, prompt-templates.md, rpc.md, sdk.md) + verified every Focusa API route against actual code.

### 34.1 Route Name Corrections

| Doc 44 Uses | Actual Route | Fix |
|---|---|---|
| `/v1/gate/ingest-signal` (§33.4) | `/v1/focus-gate/ingest-signal` OR `/v1/gate/signal` | Update all references |

All other routes in doc 44 verified correct against live code.

### 34.2 Focusa Capabilities Not Yet Used by Pi Extension

These are implemented, tested, live API routes that the Pi extension spec doesn't reference:

#### A. Instance Registration (`/v1/instances/connect`)
Pi should register itself as a Focusa instance on startup:
```typescript
pi.on("session_start", async (_event, ctx) => {
  await fetch(":8787/v1/instances/connect", {
    method: "POST",
    body: JSON.stringify({ kind: "Gui" })  // or new "Pi" kind after focusa-ygb
  });
});
pi.on("session_shutdown", async () => {
  await fetch(":8787/v1/instances/disconnect", { method: "POST" });
});
```
This gives Focusa visibility into which agents are currently connected. Multi-device sync (§43) uses instance IDs for ownership.

#### B. Turn Tracking (`/v1/turn/start`, `/v1/turn/append`, `/v1/turn/complete`)
Pi extension should notify Focusa of every turn lifecycle — not just session start/close. This enables:
- Per-turn telemetry (tokens, latency)
- ASCC delta extraction per turn
- CLT lineage tracking
- Autonomy scoring per turn

```typescript
pi.on("turn_start", async (event, ctx) => {
  await fetch(":8787/v1/turn/start", {
    method: "POST",
    body: JSON.stringify({
      turn_id: `pi-${event.turnIndex}-${Date.now()}`,
      adapter_id: "pi-extension",
      harness_name: "pi",
      timestamp: new Date().toISOString()
    })
  }).catch(() => {});
});

pi.on("turn_end", async (event, ctx) => {
  const userMsg = event.message?.role === "assistant" ? "" : event.message?.content;
  const assistantMsg = event.message?.role === "assistant" 
    ? event.message.content?.filter(c => c.type === "text").map(c => c.text).join("") 
    : "";
  
  await fetch(":8787/v1/turn/complete", {
    method: "POST",
    body: JSON.stringify({
      turn_id: `pi-${event.turnIndex}-${Date.now()}`,
      assistant_output: assistantMsg?.slice(0, 4000) || "",
      artifacts: [],
      errors: []
    })
  }).catch(() => {});
});
```

**This is critical.** Without turn tracking, Focusa can't:
- Score autonomy from Pi turns
- Run workers on Pi output
- Build ASCC deltas from Pi conversations
- Track CLT lineage for Pi sessions

#### C. Focus State Updates (`/v1/focus/update`)
When Pi makes significant progress (completes a task, makes a decision), the extension should update Focusa Focus State directly:
```typescript
async function updateFocusState(delta: any) {
  await fetch(":8787/v1/focus/update", {
    method: "POST",
    body: JSON.stringify(delta)
  }).catch(() => {});
}
```
Currently only `/wbm` cataloguing does this. But even without `/wbm`, if Focusa is connected, Focus State should reflect Pi's work.

#### D. Intuition Signal Submission (`/v1/intuition/submit`)
Pi can submit signals directly to the Intuition Engine — not just via the Focus Gate ingestion path. Useful for:
- Pi detects repeated compilation errors → signal
- Pi session running >2 hours → long-running signal
- Pi making many small edits to same file → possible churn signal

#### E. State Explanation (`/v1/state/explain`)
The `/focusa-gate-explain` command in §10.3 maps to `/v1/gate/explain`. But there's also `/v1/state/explain` which explains the last decision made. This should be exposed as `/focusa-explain-decision`.

#### F. CLT Lineage (`/v1/lineage/*`)
Pi sessions should be trackable in CLT. When the Pi extension submits turns, CLT nodes are created automatically. The extension could expose:
- `/focusa-lineage` → show lineage path from current CLT head to root
- Useful for "how did we get here?" provenance queries

#### G. Focusa Skill (NOT extension)
Pi has a skills system (`/skill:name`). A Focusa SKILL.md could be created at `~/.pi/skills/focusa/SKILL.md` that:
- Documents how to use Focusa CLI within Pi
- Provides agent instructions for Focus State management
- Auto-loads when agent needs metacognitive guidance

This is SEPARATE from the extension. The skill gives the LLM knowledge about Focusa. The extension gives Pi programmatic access.

```
~/.pi/skills/focusa/SKILL.md
---
name: focusa
description: Focusa cognitive runtime — metacognition, focus management, decision tracking. Use when managing complex multi-step tasks or needing to preserve decisions across compaction.
---
# Focusa Meta-Cognition

When working on complex tasks, use Focusa to track your cognitive state:

## Check Status
```bash
curl -s http://127.0.0.1:8787/v1/focus/stack | jq .
```

## Record a Decision
```bash
curl -X POST http://127.0.0.1:8787/v1/focus/update \
  -H 'Content-Type: application/json' \
  -d '{"decisions": ["Chose X because Y"]}'
```
...
```

#### H. Prompt Template for Focusa-Aware Work
Create `.pi/prompts/focusa-context.md`:
```markdown
---
description: Start work with Focusa cognitive context loaded
---
Before starting: check Focusa status and load cognitive context.
1. Run: curl -s http://127.0.0.1:8787/v1/focus/stack | jq .
2. Run: curl -s http://127.0.0.1:8787/v1/ascc/state | jq .
3. Review the active focus frame, decisions, and constraints.
4. Proceed with the task, recording decisions in Focusa as you go.

$ARGUMENTS
```

### 34.3 Pi Session Format Understanding for Historical Extraction

From session.md, Pi sessions are JSONL with entry types:
- `session` (header)
- `message` (with `AgentMessage` subtypes: user, assistant, toolResult, custom, compactionSummary, branchSummary, bashExecution)
- `compaction`
- `branch_summary`
- `custom` (extension state)
- `model_change`
- `thinking_level_change`
- `label`

**For historical Pi session extraction (bead focusa-d8i):**
The parser must handle ALL these types, not just user+assistant. Specifically:
- `message` with `role: "assistant"` → extract decisions/learnings from text content
- `message` with `role: "toolResult"` → extract errors, file paths, command outputs
- `compaction` → `summary` field already contains a condensed session summary — extract from this directly (much cheaper than re-processing all messages)
- `branch_summary` → `summary` field contains abandoned branch context — may contain decisions that were superseded

**Novel insight:** Compaction summaries are ALREADY LLM-generated summaries. The historical extraction pipeline should process these FIRST — they're pre-digested and contain the highest-signal content. Processing raw messages is the fallback for pre-compaction content.

### 34.4 Pi SDK Direct Embedding — Alternative to Extension

Pi's SDK (`@mariozechner/pi-coding-agent`) allows creating `AgentSession` objects programmatically. This means Focusa could potentially:
- Spawn Pi as a sub-agent for specific tasks
- Control Pi sessions from Focusa daemon directly
- Use Pi's tool execution engine from Focusa workers

This is a FUTURE capability, not needed for the current extension. But worth noting in the architecture for Phase 6+.

### 34.5 Pi RPC Mode — External Control

Pi's RPC mode (`pi --mode rpc`) allows JSON protocol control over stdin/stdout. This means:
- A Focusa adapter could control Pi via RPC rather than extension hooks
- Focusa daemon could start `pi --mode rpc` as a subprocess and send prompts

This is an alternative to Mode A (`focusa wrap -- pi`). The extension approach (§33) is better for our use case because it's lighter and doesn't require subprocess management. But RPC mode is available if needed.

### 34.6 Updated Architecture Recommendation

After two full passes, the recommended integration architecture is:

```
┌──────────────────────────────────────────────────────┐
│  Pi Extension (focusa-pi-bridge.ts)                  │
│                                                      │
│  Layer 1: Provider Integration                       │
│    before_provider_request → inject Focusa context   │
│    OR registerProvider("focusa") for proxy mode      │
│                                                      │
│  Layer 2: Lifecycle Integration                      │
│    session_start → /v1/session/start + /v1/instances │
│    turn_start/end → /v1/turn/start + /v1/turn/complete│
│    session_before_compact → Focusa ASCC summary      │
│    session_shutdown → flush + close                  │
│                                                      │
│  Layer 3: Real-Time Integration                      │
│    context → inject live Focus State per LLM call    │
│    tool_result → auto-externalize to ECS             │
│    tool_call → feed Intuition Engine                 │
│    agent_end → extract work meta (/wbm)              │
│                                                      │
│  Layer 4: State Persistence                          │
│    appendEntry → save Focusa state in Pi session     │
│    session_start → restore from entries              │
│                                                      │
│  Layer 5: Commands & UX                              │
│    /wbm on|off|status|deep|flush|decisions|ships     │
│    /focusa-status|stack|pin|suppress|checkpoint      │
│    /focusa-rehydrate|gate-explain|explain-decision   │
│    /focusa-lineage                                   │
│    Status bar: 🧠 + ARI badge + WBM indicator       │
│                                                      │
│  Layer 6: Knowledge (Skill + Prompt Template)        │
│    ~/.pi/skills/focusa/SKILL.md                      │
│    .pi/prompts/focusa-context.md                     │
└──────────────────────────────────────────────────────┘
```

This is the definitive 6-layer architecture. Previous versions had 3 layers — this is more complete.

---

## 35) Third-Pass Audit — OBVIOUS Gaps

Audited 2026-04-02: Re-read all Pi docs + Focusa docs looking specifically for obvious missing pieces that would prevent the extension from actually working.

### 35.1 WHO PUSHES THE INITIAL FOCUS FRAME? (CRITICAL)

The entire Focusa cognitive layer requires an active Focus Frame. Without one:
- Focus State is empty
- ASCC has nothing to checkpoint
- Expression Engine has nothing to inject
- Decisions have nowhere to be recorded
- Constraints don't exist

**Doc 44 §26 already flagged this:** "Stack discipline not consistently automatic (active frame may be missing until manually pushed)."

**But no spec section solves it.**

**Required: Auto-frame on session start.**

```typescript
pi.on("session_start", async (_event, ctx) => {
  // Check if Focusa has an active frame
  const stack = await fetch(":8787/v1/focus/stack").then(r => r.json()).catch(() => null);
  
  if (!stack?.active_frame_id) {
    // Derive frame from project context
    const cwd = ctx.cwd;
    const projectName = cwd.split("/").pop() || "unknown";
    
    // Check for beads in project
    let beadsId = "pi-session";
    try {
      const result = await pi.exec("bd", ["ready"], { timeout: 3000 });
      if (result.stdout.trim()) beadsId = result.stdout.trim().split(/\s/)[0];
    } catch {}
    
    // Push frame
    await fetch(":8787/v1/focus/push", {
      method: "POST",
      body: JSON.stringify({
        title: `Pi: ${projectName}`,
        goal: `Work on ${projectName}`,
        beads_issue_id: beadsId,
        constraints: [],
        tags: ["pi", projectName]
      })
    }).catch(() => {});
  }
});
```

**Or:** Prompt the user on first turn:
```typescript
pi.on("before_agent_start", async (event, ctx) => {
  if (!hasActiveFrame && !promptedForFrame) {
    promptedForFrame = true;
    return {
      message: {
        customType: "focusa-pi-bridge",
        content: "No active focus frame. What are you working on? (I'll create a Focusa frame for tracking)",
        display: true,
      }
    };
  }
});
```

### 35.2 THE LLM DOESN'T KNOW TO USE FOCUSA (CRITICAL)

Injecting Focus State data (`[FOCUSA LIVE STATE]`) is necessary but not sufficient. The LLM also needs **behavioral instructions** telling it what to DO with that data.

**Required: System prompt augmentation.**

```typescript
pi.on("before_agent_start", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  const focusaInstructions = `
## Focusa Cognitive Governance (Active)
You are operating within Focusa, a cognitive runtime that preserves focus and decisions.

RULES:
- When you make a significant decision, state it clearly as "DECISION: ..."
- Check the CONSTRAINTS section below before acting — do not violate them
- When something fails, note it as "FAILURE: ..."  
- When you discover a new constraint, note it as "CONSTRAINT: ..."
- The DECISIONS listed below were made in this session — do not contradict them without explanation
- If context was compacted, the Focus State below is your source of truth for what was decided
`;

  return {
    systemPrompt: event.systemPrompt + "\n" + focusaInstructions,
  };
});
```

Without this, the LLM sees Focus State data but has no instruction to respect constraints, record decisions, or avoid contradicting prior work.

### 35.3 USER WORKFLOW IS UNDEFINED (HIGH)

The spec defines what the extension CAN do but never describes the user's actual experience:

**Recommended default workflow:**

```
1. User runs: pi
2. Extension loads, checks Focusa health
3. If no active frame:
   a. Check project beads (bd ready) for current task
   b. If task found → auto-push frame with task title/goal
   c. If no task → show status bar "No focus frame — use /focusa-push to set one"
4. If /wbm previously active (from appendEntry state) → auto-restore
5. Show status bar: "🧠 Focusa" or "🧠 WBM" or "⚪ Focusa (no frame)"
6. User works normally — extension handles everything in background
7. On compaction → Focusa ASCC preserves decisions
8. On session end → work extracted and catalogued
```

**No manual steps required except:**
- `/wbm on` if user wants Wirebot context (opt-in)
- `/focusa-push "title"` if auto-frame detection fails

### 35.4 LOCAL DECISION SHADOW FOR OFFLINE COMPACTION (HIGH)

When Focusa is down, compaction loses decisions. The extension should maintain a local shadow:

```typescript
let localDecisions: string[] = [];
let localConstraints: string[] = [];
let localFailures: string[] = [];

// Collect from assistant messages (simple heuristic)
pi.on("agent_end", async (event, ctx) => {
  for (const msg of event.messages || []) {
    if (msg.role === "assistant") {
      const text = msg.content?.filter(c => c.type === "text").map(c => c.text).join("") || "";
      // Extract DECISION: lines
      for (const line of text.split("\n")) {
        if (line.match(/^DECISION:|^Decided:|decided to|chose to|going with/i)) {
          localDecisions.push(line.trim());
        }
        if (line.match(/^CONSTRAINT:|^FAILURE:|failed|error:/i)) {
          localFailures.push(line.trim());
        }
      }
    }
  }
});

// Use local shadow when Focusa is down during compaction
pi.on("session_before_compact", async (event, ctx) => {
  // Try Focusa ASCC first
  const ascc = await fetchFocusaASCC();
  if (ascc) return { compaction: buildFocusaSummary(ascc) };
  
  // Fallback: use local shadow
  if (localDecisions.length > 0 || localFailures.length > 0) {
    return {
      customInstructions: `PRESERVE these in the summary:\nDECISIONS: ${localDecisions.join("; ")}\nFAILURES: ${localFailures.join("; ")}`
    };
  }
});
```

### 35.5 TOKEN COUNTS MISSING FROM TURN TRACKING (MEDIUM)

Pi's `AssistantMessage.usage` has exact token counts. The turn/complete call should include them:

```typescript
pi.on("turn_end", async (event, ctx) => {
  if (event.message?.role === "assistant" && event.message.usage) {
    await fetch(":8787/v1/turn/complete", {
      method: "POST",
      body: JSON.stringify({
        turn_id: currentTurnId,
        assistant_output: extractText(event.message),
        prompt_tokens: event.message.usage.input,
        completion_tokens: event.message.usage.output,
        artifacts: [],
        errors: []
      })
    }).catch(() => {});
  }
});
```

Without this, Focusa telemetry shows `prompt_tokens: null` for all Pi turns.

### 35.6 FILE TRACKING FROM COMPACTION (MEDIUM)

Pi tracks `readFiles[]` and `modifiedFiles[]` during compaction. Feed to Focusa:

```typescript
pi.on("session_compact", async (event, ctx) => {
  if (event.compactionEntry?.details) {
    const { readFiles, modifiedFiles } = event.compactionEntry.details;
    if (modifiedFiles?.length > 0) {
      await fetch(":8787/v1/focus/update", {
        method: "POST",
        body: JSON.stringify({
          artifacts: modifiedFiles.map(f => `file:${f}`),
          notes: [`Session compacted. Modified: ${modifiedFiles.join(", ")}`]
        })
      }).catch(() => {});
    }
  }
});
```

### 35.7 OPERATOR CORRECTION DETECTION (MEDIUM)

```typescript
pi.on("input", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  const text = typeof event.text === "string" ? event.text : "";
  const lower = text.toLowerCase();
  
  // Detect corrections
  const isCorrection = lower.match(/^no[,.]?\s|that'?s wrong|i already|not what i|you misunderstood|wrong approach|undo|revert/);
  
  if (isCorrection) {
    // Record as Focus State failure
    fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({ failures: [`Operator correction: ${text.slice(0, 200)}`] })
    }).catch(() => {});
    
    // If /wbm on, also feed trust metrics
    if (wbmEnabled) {
      pi.exec("wb", ["trust", "set", "--corrections", "+1"]).catch(() => {});
    }
  }
  
  return { action: "continue" };
});
```

### 35.8 SESSION NAME FROM FOCUS FRAME (LOW)

```typescript
// When Focusa frame is active, set Pi session name to match
async function syncSessionName() {
  const stack = await fetchFocusaStack();
  if (stack?.active_frame_id) {
    const frame = stack.stack?.frames?.find(f => f.id === stack.active_frame_id);
    if (frame?.title) {
      pi.setSessionName(`🧠 ${frame.title}`);
    }
  }
}
```

Users see meaningful names in `/resume` instead of first-message snippets.

### 35.9 /WBM CONFIG — SCOREBOARD TOKEN (LOW)

The `/wbm` feature needs the scoreboard token. Where does it come from?

**Options (in order of preference):**
1. `process.env.SCOREBOARD_TOKEN` — already set in many service envs
2. Read from `/run/wirebot/scoreboard.env` — runtime secret file
3. Config in `.pi/settings.json` under `extensions.focusaPiBridge.scoreboardToken`
4. Prompt user on first `/wbm on` if not found

The extension should try options 1-2 automatically. Never hardcode tokens.

### 35.10 Summary: What Was Obvious But Missing

| # | Gap | Severity | Why It's Obvious |
|---|---|---|---|
| 1 | No auto-frame on session start | **CRITICAL** | Without a frame, Focusa has no Focus State. Everything is inert. |
| 2 | LLM has no Focusa behavioral instructions | **CRITICAL** | Data without instructions is noise. LLM must know to record decisions. |
| 3 | User workflow never described | **HIGH** | Spec says what extension CAN do, not what happens when user starts Pi. |
| 4 | No local decision shadow for offline compaction | **HIGH** | Focusa down = decisions lost on compact. Trivial local fix. |
| 5 | Token counts not passed to Focusa | **MEDIUM** | Pi has exact tokens. Focusa telemetry shows null. Easy bridge. |
| 6 | File tracking not fed to Focusa | **MEDIUM** | Pi tracks modified files. Focusa artifacts[] stays empty. |
| 7 | Operator corrections not detected | **MEDIUM** | "No that's wrong" should become a failure + trust metric. |
| 8 | Session name not synced | **LOW** | /resume shows message snippets, not frame titles. |
| 9 | /wbm token sourcing undefined | **LOW** | Extension needs scoreboard token but spec doesn't say where. |

---

## 36) Fourth-Pass Audit — Adapter Contract Compliance + Data Flow Verification

Audited 2026-04-02: Cross-checked doc 44 against Focusa adapter contract (G1-detail-04-proxy-adapter.md), Pi session lifecycle (session.md, compaction.md), and traced every data flow for double-injection risks.

### 36.1 Missing: Streaming via turn/append (ADAPTER CONTRACT)

Focusa's adapter spec (G1-detail-04) defines 4 required endpoints:
1. `POST /v1/turn/start` — §34.2B covers ✅
2. `POST /v1/turn/append` — **NOT in any section** ❌
3. `POST /v1/turn/complete` — §34.2B covers ✅
4. `POST /v1/prompt/assemble` — §33.5 covers ✅

Pi streams responses via `message_update` events (token-by-token). The extension should forward streaming chunks to Focusa for real-time visibility:

```typescript
pi.on("message_update", async (event, ctx) => {
  if (!focusaEnabled || !currentTurnId) return;
  if (event.assistantMessageEvent?.type === "text_delta") {
    fetch(":8787/v1/turn/append", {
      method: "POST",
      body: JSON.stringify({
        turn_id: currentTurnId,
        chunk: event.assistantMessageEvent.delta
      })
    }).catch(() => {}); // Fire-and-forget
  }
});
```

This is REQUIRED. turn/append enables real-time ASCC delta extraction, SSE streaming to Agent Audit, and fulfills the adapter contract.

### 36.2 Missing: Error Signals to Focus Gate

When Pi tool calls fail (`isError: true`) or the model errors (`stopReason: "error"`), these should become Focusa Intuition signals:

```typescript
pi.on("tool_result", async (event, ctx) => {
  if (event.isError && focusaEnabled) {
    fetch(":8787/v1/focus-gate/ingest-signal", {
      method: "POST",
      body: JSON.stringify({
        signal_type: "tool_error",
        summary: `Tool ${event.toolName} failed: ${event.content?.[0]?.text?.slice(0, 200)}`,
      })
    }).catch(() => {});
  }
});

pi.on("message_end", async (event, ctx) => {
  if (event.message?.role === "assistant" && event.message.stopReason === "error") {
    fetch(":8787/v1/focus-gate/ingest-signal", {
      method: "POST",
      body: JSON.stringify({
        signal_type: "model_error",
        summary: `Model error: ${event.message.errorMessage || "unknown"}`,
      })
    }).catch(() => {});
  }
});
```

### 36.3 Missing: User Input Signals to Focus Gate

The adapter contract requires signaling "user input received" to Focus Gate. This helps the Intuition Engine track activity cadence (inactivity detection):

```typescript
pi.on("agent_start", async (_event, ctx) => {
  if (!focusaEnabled) return;
  fetch(":8787/v1/focus-gate/ingest-signal", {
    method: "POST",
    body: JSON.stringify({
      signal_type: "user_input",
      summary: "User input received in Pi session",
    })
  }).catch(() => {});
});
```

### 36.4 Missing: Focusa Session Resume on Pi /resume (HIGH)

When user runs `/resume` in Pi, the extension restores WBM state from `appendEntry` (§33.7). But it currently starts a NEW Focusa session instead of resuming the saved one.

```typescript
pi.on("session_start", async (_event, ctx) => {
  let savedSessionId: string | null = null;
  
  // Check for saved Focusa state
  for (const entry of ctx.sessionManager.getEntries()) {
    if (entry.type === "custom" && entry.customType === "focusa-wbm-state") {
      savedSessionId = entry.data?.sessionId;
    }
  }
  
  if (savedSessionId) {
    // Check if that Focusa session is still valid
    const status = await fetch(`:8787/v1/status`).then(r => r.json()).catch(() => null);
    if (status?.session?.session_id === savedSessionId) {
      // Same session still active — just reconnect
      focusaSessionId = savedSessionId;
      return;
    }
  }
  
  // Start new Focusa session
  const result = await fetch(":8787/v1/session/start", { method: "POST", ... });
  focusaSessionId = result?.session_id;
});
```

### 36.5 Missing: Pi /fork and /tree Handling

When Pi forks (`session_fork`) or navigates tree (`session_tree`), Focus State should reflect the branch point, not current state.

```typescript
pi.on("session_fork", async (event, ctx) => {
  // Focus State may contain decisions from AFTER the fork point.
  // For correctness, we should snapshot Focus State at the fork point.
  // Log note AND snapshot current Focus State for reconciliation.
  if (focusaEnabled) {
    fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({
        notes: [`Pi session forked from ${event.previousSessionFile}. Focus State may contain post-fork decisions.`]
      })
    }).catch(() => {});
  }
});

pi.on("session_tree", async (event, ctx) => {
  // Similar: navigating to a different branch may invalidate current Focus State
  if (focusaEnabled) {
    fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({
        notes: [`Pi session tree navigation. Focus State may need reconciliation.`]
      })
    }).catch(() => {});
  }
});
```

**Required:** Focusa must support Focus State snapshots per CLT node for branch-aware state restoration. The extension must snapshot state on fork and restore on tree navigation. Both extension and Focusa core changes needed.

### 36.6 CRITICAL: Injection Layering Rules (Prevents Double-Injection)

Doc 44 has FOUR context injection mechanisms that could fire simultaneously:
1. `before_agent_start` — behavioral instructions + state (§35.2)
2. `context` — live Focus State per LLM call (§33.2)
3. `before_provider_request` — Focusa assembled prompt (§33.5)
4. `session_before_compact` — compaction instructions (§33.10)

**If all active: Focus State injected 3× per turn. Token waste + confusion.**

**AUTHORITATIVE LAYERING RULES:**

| Hook | Purpose | Active? | Content |
|---|---|---|---|
| `before_agent_start` | Behavioral instructions (ONE TIME per prompt) | **YES — always** | Focusa rules: "record decisions as DECISION:, check constraints..." |
| `context` | Live Focus State refresh (PER LLM CALL) | **YES — always** | Compact state: intent + decisions + constraints (≤500 tok) |
| `before_provider_request` | Full assembled prompt injection | **NO — disabled by default** | Only enable if NOT using `context` hook (Mode B alternative) |
| `session_before_compact` | Compaction preservation | **YES — on compact only** | ASCC full state as compaction summary or instructions |

**Rule: `context` and `before_provider_request` are MUTUALLY EXCLUSIVE. Use context (lighter, per-call refresh). Disable provider request injection.**

### 36.7 Missing: Context Budget Communication

Focusa's Expression Engine has `max_prompt_tokens` (default 6000). Pi's context may only have 2000 tokens of headroom. If Focusa injects 6000 tokens, Pi overflows.

```typescript
pi.on("context", async (event, ctx) => {
  const usage = ctx.getContextUsage();
  if (!usage || !focusaEnabled) return;
  
  // Calculate available headroom
  const model = ctx.model;
  const contextWindow = model?.contextWindow || 200000;
  const headroom = contextWindow - usage.tokens - 16384; // reserve for response
  
  // Request Focusa assemble within budget
  const maxFocusaTokens = Math.min(Math.max(headroom * 0.2, 200), 2000);
  // Inject at most maxFocusaTokens of Focusa context
  
  const state = await fetchFocusaState();
  if (!state) return;
  
  const truncatedState = truncateFocusState(state, maxFocusaTokens);
  return { messages: [focusaMessage(truncatedState), ...event.messages] };
});
```

**Without this, Focusa can push Pi into context overflow, triggering unnecessary compaction.**

---

## 37) Purpose Verification — Final Holistic Review

Reviewed 2026-04-02: Verified the spec delivers on Focusa's core invariant, the organism's higher-level goals, the JARVIS vision, and uses Pi's full capability.

### 37.1 Purpose Alignment Verification

| Purpose | Delivered? | How |
|---|---|---|
| "Meaning lives in Focus State, not in conversation" | ✅ | §33.1 ASCC compaction, §35.1 auto-frame, §35.2 behavioral instructions, §35.4 local shadow |
| Single organism across all surfaces | ✅ | §29 /wbm two-way bridge, WINS portal approval flow, 5-sink delivery |
| JARVIS 7 domains for Pi surface | ✅ | All 7 mapped in organism spec §9.11 |
| Pi gets smarter over time | ✅ | If beads implemented: procedural rules, Mem0, wiki, ARI, thesis |
| Pi → Context Core feedback | ❌ | **NEW GAP:** Pi doesn't tell Context Core it's active |
| Cross-surface real-time events | ⚠️ | §30 SSE for metacognitive indicators, but not for cross-surface decision notifications |

### 37.2 CRITICAL: Metacognition Tools (Not Text Markers)

**The single most impactful change remaining.**

Current §35.2 tells the LLM to write `DECISION:` text markers, then extracts via regex/LLM. This is fragile — LLMs forget, format varies, extraction misses content.

**Register metacognition TOOLS the LLM can call directly:**

```typescript
pi.registerTool({
  name: "focusa_decide",
  label: "Record Decision",
  description: "Record a decision made during this work session. Call this whenever you make a significant choice.",
  promptSnippet: "Record decisions, constraints, and failures for cognitive tracking",
  promptGuidelines: [
    "When you make a significant decision (choosing an approach, selecting a library, architectural choice), call focusa_decide.",
    "When you discover a constraint (must handle X, cannot use Y), call focusa_constraint.",
    "When something fails (build error, test failure, wrong approach), call focusa_failure.",
  ],
  parameters: Type.Object({
    decision: Type.String({ description: "What was decided" }),
    rationale: Type.String({ description: "Why this was chosen" }),
    alternatives: Type.Optional(Type.Array(Type.String(), { description: "What alternatives were considered" })),
  }),
  async execute(toolCallId, params, signal, onUpdate, ctx) {
    // Record in Focusa Focus State
    await fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({ decisions: [`${params.decision} (because: ${params.rationale})`] })
    }).catch(() => {});
    
    // If /wbm active, also queue for WINS portal
    if (wbmEnabled) {
      await pi.exec("wb", ["memory", "inject", `DECISION: ${params.decision}. Rationale: ${params.rationale}`]).catch(() => {});
    }
    
    localDecisions.push(`${params.decision} (${params.rationale})`);
    
    return {
      content: [{ type: "text", text: `✅ Decision recorded: ${params.decision}` }],
      details: { decision: params.decision, rationale: params.rationale },
    };
  },
});

pi.registerTool({
  name: "focusa_constraint",
  label: "Record Constraint",
  description: "Record a constraint discovered during work.",
  parameters: Type.Object({
    constraint: Type.String({ description: "What constraint was found" }),
    reason: Type.Optional(Type.String({ description: "Why this constraint exists" })),
  }),
  async execute(toolCallId, params) {
    await fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({ constraints: [params.constraint] })
    }).catch(() => {});
    localConstraints.push(params.constraint);
    return { content: [{ type: "text", text: `⚠️ Constraint recorded: ${params.constraint}` }] };
  },
});

pi.registerTool({
  name: "focusa_failure",
  label: "Record Failure",
  description: "Record a failure encountered during work.",
  parameters: Type.Object({
    failure: Type.String({ description: "What failed" }),
    impact: Type.Optional(Type.String({ description: "Impact of the failure" })),
  }),
  async execute(toolCallId, params) {
    await fetch(":8787/v1/focus/update", {
      method: "POST",
      body: JSON.stringify({ failures: [params.failure] })
    }).catch(() => {});
    localFailures.push(params.failure);
    return { content: [{ type: "text", text: `❌ Failure recorded: ${params.failure}` }] };
  },
});
```

**Why this is better than text markers:**
- Structured JSON, not free-text parsing
- Tool calls are guaranteed in Pi's session format (type: "toolCall" in AssistantMessage)
- `promptGuidelines` injects tool-specific instructions when tool is active
- `promptSnippet` adds to Available Tools section
- Tool result details are persisted via `appendEntry` and survive compaction
- Historical extraction can find tool calls reliably (`toolName: "focusa_decide"`)
- LLM sees `✅ Decision recorded` confirmation — positive reinforcement

**This replaces §35.2 behavioral instructions as the primary decision capture mechanism.** §35.2 instructions remain as backup for when LLM doesn't call the tool.

### 37.3 Widget: Persistent Focus State Display

```typescript
function updateFocusWidget(ctx) {
  const frame = currentFrame;
  const dCount = localDecisions.length;
  const cCount = localConstraints.length;
  const fCount = localFailures.length;
  
  const lines = [
    `🧠 ${frame?.title || "No frame"} ${wbmEnabled ? "⚡WBM" : ""}`,
    `📋 ${dCount} decisions · ${cCount} constraints · ${fCount} failures`,
  ];
  
  ctx.ui.setWidget("focusa", lines, { placement: "belowEditor" });
}
```

Always visible below the editor. Operator sees cognitive state without typing commands.

### 37.4 Keyboard Shortcuts

```typescript
pi.registerShortcut("ctrl+shift+f", {
  description: "Toggle Focusa status overlay",
  handler: async (ctx) => {
    // Show/hide detailed Focusa state
  },
});

pi.registerShortcut("ctrl+shift+w", {
  description: "Toggle Wirebot Mode",
  handler: async (ctx) => {
    wbmEnabled = !wbmEnabled;
    ctx.ui.notify(wbmEnabled ? "⚡ WBM ON" : "WBM OFF", "info");
  },
});
```

### 37.5 CLI Flags

```typescript
pi.registerFlag("wbm", {
  description: "Start with Wirebot Mode enabled",
  type: "boolean",
  default: false,
});

pi.registerFlag("focusa", {
  description: "Enable Focusa cognitive integration",
  type: "boolean",
  default: true,
});

// On session start
if (pi.getFlag("--wbm")) {
  wbmEnabled = true;
}
if (pi.getFlag("--focusa") === false) {
  focusaEnabled = false;
}
```

`pi --wbm` starts with Wirebot Mode on. `pi --no-focusa` disables integration.

### 37.6 Custom Message Renderer

```typescript
pi.registerMessageRenderer("focusa-state", (message, options, theme) => {
  const { expanded } = options;
  let text = theme.fg("dim", "🧠 ") + theme.fg("muted", "Focusa Context");
  if (expanded) {
    text += "\n" + theme.fg("dim", message.content);
  }
  return new Text(text, 0, 0);
});
```

Focusa-injected messages get collapsed styling — visible but not noisy.

### 37.7 Session Switch Handling

```typescript
pi.on("session_before_switch", async (event, ctx) => {
  // Flush Focusa state before switching sessions
  await flushWorkMeta();
  persistFocusaState();
  
  if (focusaSessionActive) {
    await fetch(":8787/v1/session/close", {
      method: "POST",
      body: JSON.stringify({ reason: "pi_session_switch" })
    }).catch(() => {});
  }
});
```

### 37.8 Model Change Tracking

```typescript
pi.on("model_select", async (event, ctx) => {
  if (!focusaEnabled) return;
  
  const modelName = `${event.model.provider}/${event.model.id}`;
  // Inform Focusa of model change for telemetry
  fetch(":8787/v1/focus/update", {
    method: "POST",
    body: JSON.stringify({ notes: [`Model changed to ${modelName}`] })
  }).catch(() => {});
});
```

### 37.9 Pi → Context Core Feedback

```typescript
// On session start, tell Context Core Pi is active
pi.on("session_start", async () => {
  pi.exec("wb", ["me", "--set", "pi_active=true"]).catch(() => {});
});

pi.on("session_shutdown", async () => {
  pi.exec("wb", ["me", "--set", "pi_active=false"]).catch(() => {});
});
```

Context Core then knows Pi is running, which affects operator state calculations.

### 37.10 Cross-Surface Event Notifications via SSE

```typescript
// Subscribe to Focusa SSE for cross-surface events
let sseConnection: EventSource | null = null;

function connectSSE() {
  // Use fetch with streaming instead of EventSource (Node.js compatible)
  fetch(":8787/v1/events/stream", { signal: AbortSignal.timeout(0) })
    .then(response => {
      const reader = response.body?.getReader();
      // Process SSE events...
      // On FocusStateUpdated from another surface:
      //   ctx.ui.notify("Wirebot recorded: DECISION: Use bind mounts", "info");
    }).catch(() => {
      // Reconnect with backoff
    });
}
```

### 37.11 Remaining Gaps After All Passes

| # | Gap | Severity | Status |
|---|---|---|---|
| 1 | Metacognition tools (focusa_decide/constraint/failure) | **HIGH** | NEW — §37.2 |
| 2 | Focus State widget (always visible) | **MEDIUM** | NEW — §37.3 |
| 3 | Keyboard shortcuts (Ctrl+Shift+F/W) | **LOW** | NEW — §37.4 |
| 4 | CLI flags (--wbm, --focusa) | **LOW** | NEW — §37.5 |
| 5 | Custom message renderer (collapsed Focusa blocks) | **LOW** | NEW — §37.6 |
| 6 | Session switch flush | **MEDIUM** | NEW — §37.7 |
| 7 | Model change tracking | **LOW** | NEW — §37.8 |
| 8 | Pi → Context Core activity signal | **MEDIUM** | NEW — §37.9 |
| 9 | Cross-surface SSE notifications | **MEDIUM** | NEW — §37.10 |

**All previous findings (§33-§36) confirmed valid and properly cross-referenced.**

---

## 38) Final Review — Minor Gaps and Edge Case Fixes

### 38.1 Local Decision Shadow Must Trim After ASCC Write

Edge case: Very long Pi sessions (days) with multiple compactions. `localDecisions`, `localConstraints`, `localFailures` arrays grow unbounded.

**Fix:** After successful Focusa ASCC write (confirmed via §33.1 `session_before_compact` returning compaction), clear the local shadow:

```typescript
pi.on("session_before_compact", async (event, ctx) => {
  const ascc = await fetchFocusaASCC();
  if (ascc) {
    const result = { compaction: buildFocusaSummary(ascc), ... };
    // ASCC write succeeded — Focusa now owns these decisions
    localDecisions = [];
    localConstraints = [];
    localFailures = [];
    return result;
  }
  
  // Focusa down — keep local shadow, inject as instructions
  // (do NOT clear — these are the only copy)
  return { customInstructions: buildLocalShadowInstructions() };
});
```

**Rule:** Clear local shadow only when Focusa ASCC is confirmed written. Never clear when falling back to local shadow.

### 38.2 /wbm HTTP Fallback When wb CLI Not Available (Multi-Machine)

Edge case: Pi runs on Mac, Focusa runs on VPS via Tailscale. `wb` CLI is only installed on VPS. `pi.exec("wb", [...])` fails on Mac.

**Fix:** All /wbm `wb` calls must have an HTTP fallback:

```typescript
async function wbWikiSearch(query: string): Promise<any> {
  // Try wb CLI first (fast, handles auth)
  try {
    const result = await pi.exec("wb", ["wiki", "search", query, "--format", "json"], { timeout: 5000 });
    if (result.code === 0) return JSON.parse(result.stdout);
  } catch {}
  
  // Fallback: direct HTTP to Wiki.js GraphQL
  try {
    const wikiToken = process.env.WIKI_TOKEN || await readFile("/data/wirebot/secrets/wiki-api-token", "utf-8").catch(() => "");
    if (!wikiToken) return null;
    const res = await fetch("http://127.0.0.1:7325/graphql", {
      method: "POST",
      headers: { "Authorization": `Bearer ${wikiToken.trim()}`, "Content-Type": "application/json" },
      body: JSON.stringify({ query: `{ pages { search(query: "${query}", locale: "en") { results { path title } } } }` }),
      signal: AbortSignal.timeout(5000),
    });
    return await res.json();
  } catch { return null; }
}

async function wbMemoryInject(text: string): Promise<boolean> {
  // Try wb CLI first
  try {
    const result = await pi.exec("wb", ["memory", "inject", text], { timeout: 5000 });
    if (result.code === 0) return true;
  } catch {}
  
  // Fallback: direct HTTP to scoreboard
  try {
    const token = process.env.SCOREBOARD_TOKEN || "";
    const res = await fetch("http://127.0.0.1:8100/v1/memory/queue", {
      method: "POST",
      headers: { "Authorization": `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify({ memory_text: text, source_type: "pi_session", confidence: 0.85 }),
      signal: AbortSignal.timeout(5000),
    });
    return res.ok;
  } catch { return false; }
}
```

**Rule:** Every `wb` CLI call must have a direct HTTP fallback. The extension works on any machine with network access to the services, not just the VPS.

**Config:** When Pi is on Mac, set in `.pi/settings.json`:
```json
{
  "extensions": {
    "focusaPiBridge": {
      "focusaApiBaseUrl": "http://100.x.x.x:8787/v1",
      "scoreboardUrl": "http://100.x.x.x:8100",
      "wikiUrl": "http://100.x.x.x:7325",
      "contextCoreUrl": "http://100.x.x.x:7400"
    }
  }
}
```

### 38.3 Disable Focusa Tools When Daemon Down

When Focusa is unreachable, `focusa_decide`/`focusa_constraint`/`focusa_failure` tools should be disabled to avoid confusing the LLM with tools that can't execute:

```typescript
let focusaToolsActive = true;

async function checkFocusaHealth() {
  try {
    const res = await fetch(focusaBaseUrl + "/health", { signal: AbortSignal.timeout(2000) });
    const data = await res.json();
    const isUp = data?.ok === true;
    
    if (isUp && !focusaToolsActive) {
      // Focusa came back — re-enable tools
      const allTools = pi.getAllTools();
      const currentActive = pi.getActiveTools().map(t => t.name);
      pi.setActiveTools([...currentActive, "focusa_decide", "focusa_constraint", "focusa_failure"]);
      focusaToolsActive = true;
    } else if (!isUp && focusaToolsActive) {
      // Focusa went down — disable tools
      const currentActive = pi.getActiveTools().map(t => t.name);
      pi.setActiveTools(currentActive.filter(n => !n.startsWith("focusa_")));
      focusaToolsActive = false;
      ctx.ui.notify("Focusa unavailable — metacognition tools disabled", "warn");
    }
  } catch {
    if (focusaToolsActive) {
      const currentActive = pi.getActiveTools().map(t => t.name);
      pi.setActiveTools(currentActive.filter(n => !n.startsWith("focusa_")));
      focusaToolsActive = false;
    }
  }
}
```

**Rule:** Check health on session_start and periodically (every 60s). Disable tools when down. Re-enable when back. The LLM never sees tools it can't use.
