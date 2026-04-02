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
      "focusaApiBaseUrl": "http://127.0.0.1:4777/v1",
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
- `FOCUSA_PI_API_BASE_URL=http://127.0.0.1:4777/v1`
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

On `agent_end` hook, LLM extraction (MiniMax M2.7, ≤500 tok, 2s timeout) parses the turn for work meta:

| Work Meta | Destination | Method |
|---|---|---|
| Decision | Mem0 | `wb memory inject "$DECISION"` with `source:pi, surface:pi` |
| Decision | Wiki | `wb wiki create --path ops/decisions/$DATE --tags decision,pi` |
| Fact | Mem0 | `wb memory inject "$FACT"` with `source:pi, category:technical` |
| Failure | Mem0 + Focusa | `wb memory inject "FAILURE: $DETAIL"` + focus state update |
| Learning | Mem0 | `wb memory inject "LEARNED: $INSIGHT"` with `source:pi, category:learning` |

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

All references to `:4777` in this spec should read `:8787`.

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
