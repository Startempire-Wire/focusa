# Wirebot Unified Organism Spec

**Status:** CANONICAL — ready for implementation  
**Goal:** All systems function as a single organism that grows smarter daily  
**Authors:** Opus (operational detail) + GPT5.4 (governance framework) — merged by Opus  
**Date:** 2026-04-02  
**Supersedes:** `INTEGRATION_SPEC.md` (Opus), `GPT5_4_UNIFIED_INTELLIGENCE_INTEGRATION_SPEC.md` (GPT5.4)

---

## 0. Design Principle

> **Live cognition, durable knowledge, retrievable memory, and task execution must be separated — but continuously reconciled.**

The target is not "many connected tools." The target is **one operating organism** with sensory surfaces, working memory, long-term memory, a knowledge graph, active cognition, task discipline, self-observation, feedback loops, and constrained self-improvement.

---

## 1. Canonical Authority Matrix

Every concern has exactly one owner. Cross-system overlap is a defect.

| Concern | Canonical System | What It Is NOT |
|---|---|---|
| Live intent / current meaning | **Focusa** (Focus State + Stack) | Not wiki, not Mem0 |
| Prompt shaping / context injection | **Focusa** (Expression Engine) | Not OpenClaw native |
| Durable reviewed knowledge | **Wiki.js / Obsidian vault** | Not Focusa, not Mem0 |
| Retrievable long-term memory | **Mem0** (Qdrant + Kuzu) | Not canonical policy |
| Agent-local continuity / persona | **Letta** | Not project truth |
| Task/work state | **Flow Mesh** | Not semantic memory |
| Operator state | **Context Core** | Not agent-owned |
| Agent behavioral doctrine | **SOUL.md** (human-authored) | Not auto-generated |
| Runtime projected constitution | **Focusa Constitution** (compiled from SOUL) | Not a wiki page |
| Orchestration surface | **wb CLI** | Not a storage system |
| Infrastructure health | **Guardian + wb health** | — |
| Audit trail | **Focusa event log + OpenClaw audit** | Append-only provenance |
| Mobile cognitive visibility | **Agent Audit UI** (audit.philoveracity.com) | PWA, read-only, never mutates |

---

## 2. Current State Audit

### 2.1 All Systems (verified live 2026-04-02)

| System | Port | Status | Connected To |
|--------|------|--------|--------------|
| OpenClaw Gateway | :18789 | ✅ Running | Wirebot agent runtime |
| Context Core | :7400 | ✅ Running | Operator state, session tracking |
| Scoreboard | :8100 | ✅ Running | OpenClaw, Context Core |
| Focusa | :8787 | ✅ Running | **NOTHING** (isolated) |
| Mem0 | :8200 | ✅ Running | memory-syncd only |
| Letta | :8283 | ✅ Running | memory-syncd only |
| memory-syncd | :8201 | ✅ Running | Mem0 ↔ Letta ↔ workspace |
| Wiki.js | :7325 | ✅ Running | Obsidian vault (one-way sync) |
| wiki-agent | — | ❌ **STOPPED** | Was: Wiki.js autonomous maintenance |
| Flow Mesh | — | ✅ Running | Task queue (100 backlog items) |
| UIAI Engine | :7456 | ✅ Running | Vision/screenshots |
| ntfy | :2586 | ✅ Running | Push notifications |
| Guardian | — | ✅ Running | Server health monitoring |
| wb CLI | — | ✅ 45+ commands | Orchestration layer |

### 2.2 Connection Map (Current Reality)

```
                    ┌──────────────┐
                    │   OPERATOR   │
                    │  (Obsidian)  │
                    └──────┬───────┘
                           │ git sync (Mac→VPS)
                    ┌──────▼───────┐     sync-vault-wiki
                    │   Obsidian   ├─────────────────────┐
                    │    Vault     │                      │
                    └──────────────┘                      │
                                                  ┌──────▼───────┐
    ┌──────────┐    ┌──────────────┐              │   Wiki.js    │
    │ Context  ├────┤   OpenClaw   │              │  (1056 pgs)  │
    │   Core   │    │   Gateway    │              └──────────────┘
    └──────────┘    └──────┬───────┘                     ╳
                           │ LLM calls                   ╳ NOT CONNECTED
                    ┌──────▼───────┐              ╳      ╳
                    │   Wirebot    │              ╳           ╳
                    │   (Agent)    │         ┌────╳───────┐
                    └──────┬───────┘         │   Focusa   │
                           │                 │  (8787)    │
                    ┌──────▼───────┐         └────────────┘
                    │  memory-     │              ISOLATED
                    │   syncd      │
                    └───┬──────┬───┘
                        │      │
                 ┌──────▼┐  ┌──▼──────┐
                 │ Mem0  │  │  Letta  │
                 │(8200) │  │ (8283)  │
                 └───────┘  └─────────┘
```

### 2.3 What's Broken (12 Disconnections)

| # | Gap | Impact |
|---|-----|--------|
| 1 | Focusa is isolated — no agent flows through it | Meta-cognition exists but isn't used |
| 2 | Wiki not queried during reasoning | Prior decisions/skills invisible |
| 3 | Mem0 not connected to Focusa | Can't remember across sessions |
| 4 | wiki-agent is STOPPED | Wiki decays, red links accumulate |
| 5 | Nightly enrichment not scheduled | Knowledge doesn't grow automatically |
| 6 | 952 orphan wiki pages (90%) | Graph traversal impossible |
| 7 | Engineering agents don't use wiki | Repeated mistakes |
| 8 | Session learnings don't flow to wiki | Knowledge doesn't accumulate |
| 9 | SOUL.md → Focusa constitution not automated | Doctrine drift |
| 10 | Context Core not connected to Focusa | No circadian/interruptibility awareness |
| 11 | Flow Mesh not connected to Focus Stack | Work tracking fragmented |
| 12 | Kaizen reflections not feeding wiki | Lessons don't persist |

### 2.4 Wiki Health (verified live)

| Metric | Value | Assessment |
|--------|-------|------------|
| Total pages | 1,056 | — |
| `/notes/` (knowledge graph) | 86 | Real graph |
| `/joplin-import/` | 571 | Dead weight |
| `/ai-chats/` | 225 | Unprocessed source material |
| Orphan pages | 952 (90%) | Critical |
| Pages with outbound links | 61 (6%) | Too sparse for traversal |
| Total link connections | 208 | Avg 0.2 per page |
| Stale >30d | 937 (88%) | — |
| wiki-agent | STOPPED | No autonomous maintenance |
| Enrichment timer | NONE | Not scheduled |

---

## 3. Target Architecture

```
Human / Operator
   ↓
Context Core ──────────────────┐
   ↓                           │ operator modulation
OpenClaw / Wirebot runtime     │
   ↓                           │
Focusa  ← live cognition spine ←┘
  ↓  ↓  ↓
Wiki  Mem0  Letta
  ↓    ↓      ↓
Knowledge  Recall  Local continuity
  \    |     /
     wb CLI facade
         ↓
   Flow Mesh / Scoreboard / Guardian / UIAI
```

**OpenClaw acts. Focusa thinks about the acting. Wiki remembers what should stay true. Mem0 recalls what might matter again. Letta carries local continuity. Context Core reflects the human reality. Flow Mesh disciplines the work. wb ties the surfaces together. Ontology prevents conceptual drift.**

---

## 4. Data Classes

All information must be classified. Misclassification is how systems corrupt each other.

| Class | Definition | Storage | Properties |
|---|---|---|---|
| **Event** | Atomic observed fact | Focusa event log, audit systems | Append-only, timestamped, provenance |
| **State** | Current condition of a subsystem | Subsystem-owned, mutable | Authoritative only within owner |
| **Memory** | Retrievable learned fact/pattern | Mem0 primary; Focusa bounded projection | Searchable, decayable, promotable |
| **Knowledge** | Durable, linked, reviewed truth | Wiki.js / Obsidian vault | Human-readable, schema-valid, linked |
| **Doctrine** | Behavioral principles | SOUL.md → Focusa constitution | Human-authored, projected at runtime |

---

## 5. Event Spine

### 5.1 Required Event Families

#### Session lifecycle
`session_started`, `session_closed`, `thread_attached`, `thread_detached`

#### Turn lifecycle
`turn_started`, `prompt_assembled`, `turn_completed`, `turn_failed`

#### Cognition
`focus_frame_pushed`, `focus_frame_resumed`, `focus_frame_completed`, `focus_state_updated`, `candidate_surfaced`, `candidate_suppressed`, `candidate_pinned`

#### Memory
`semantic_memory_upserted`, `procedural_rule_reinforced`, `memory_candidate_extracted`, `memory_promoted`, `memory_decay_tick`

#### Knowledge graph
`wiki_page_created`, `wiki_page_updated`, `decision_recorded`, `skill_added`, `link_created`, `graph_gap_detected`

#### Task/work
`task_selected`, `task_started`, `task_blocked`, `task_completed`

#### Operator modulation
`operator_state_changed`, `interruptibility_changed`, `circadian_phase_changed`

#### Governance
`constitution_reloaded`, `autonomy_score_changed`, `autonomy_level_changed`, `rfm_regeneration_triggered`

### 5.2 Transport Rule

No subsystem may silently mutate another subsystem's canonical state without an auditable event.

---

## 6. Promotion Pipeline

No raw model output goes directly into durable knowledge. All writes are gated.

### 6.1 Pipeline Stages

| Stage | Action |
|---|---|
| **Observe** | Collect events, turns, session captures, kaizen, workspace changes |
| **Extract** | Generate candidates: memory, decision, skill, ontology relation, project updates |
| **Validate** | Check provenance, novelty, contradiction, confidence, schema compliance, duplication |
| **Promote** | Route to target: Mem0 (recall), Wiki (knowledge), Focusa (procedural rule), Letta (continuity) |
| **Audit** | Record source event, reason, confidence, reviewer mode |

### 6.2 Write-Trust Levels

| Target | Allowed Write Mode |
|---|---|
| Telemetry / traces | Automatic |
| Session captures | Automatic |
| Mem0 candidate memory | Automatic after validation |
| Focusa semantic memory seeding | Automatic bounded projection |
| Procedural rule promotion | Thresholded or operator-approved |
| Wiki durable page creation | Thresholded + schema validated |
| Constitution changes | Human-approved only |
| Ontology schema changes | Human-approved only |

---

## 7. Contradiction Resolution

When systems disagree, precedence is:

1. Direct operator instruction
2. Active safety/constitution doctrine
3. Current Focus State constraints
4. Reviewed wiki decision pages
5. Current Context Core operator state
6. Validated Mem0 recall
7. Letta local continuity memory
8. Raw extraction candidates

**Contradictions must be logged, not silently resolved.**

---

## 8. Degraded Mode Matrix

| Failure | Fallback Behavior |
|---|---|
| **Focusa down** | OpenClaw direct passthrough; log cognition deficit |
| **Wiki down** | Use Mem0 + workspace + Letta; defer durable writes |
| **Mem0 down** | Use Wiki + Focusa bounded memory + Letta |
| **Letta down** | Continue with Wiki + Mem0 + Focusa |
| **Context Core down** | Use last-known operator state with TTL |
| **Flow Mesh down** | Local work shadow queue; reconcile later |
| **wiki-agent down** | Manual/sync paths continue; flag graph health degradation |
| **Sync timers down** | System continues; mark knowledge growth degraded |
| **Agent Audit down** | No mobile visibility; agent execution unaffected |

**No subsystem failure should fully halt agent execution unless safety requires it.**

---

## 9. Per-System Integration Rules

### 9.1 Focusa

**Focusa must remain backend-agnostic.** No dependence on Letta internals. Wrap via generic stdin/stdout or HTTP.

**Focusa owns live cognition only:**
- active frame, current intent, current constraints, current decisions-in-session, active artifact references, autonomy status

**Focusa must NOT become:** primary wiki, primary vector DB, canonical task DB.

**Memory policy:**
- Semantic: bounded, whitelisted, active-use only
- Procedural: stable rules with provenance, reinforcement history, decay, scope
- **No single event may become a global procedural rule**

**Skill surface rule:** Agents interact via inspection, explanation, and proposals. Never direct mutation outside sanctioned reducers.

### 9.2 Wiki / Vault

**Wiki is the reviewed knowledge graph.**

**Graph quality rules:**
- Every durable page needs ≥1 inbound MOC reference
- Every durable page needs ≥2 outbound semantic links
- Schema-valid metadata required
- Machine-generated pages need provenance notes

**Page classes:**
- Durable: `/notes/projects/*`, `/notes/skills/*`, `/notes/concepts/*`, `/notes/decisions/*`, `/notes/_mocs/*`
- Operational: `/ops/sessions/*`, `/ops/handoffs/*`, `/ops/inbox/*`, `/ops/journal/*`
- Archive: imports, raw chats, joplin dump

**Raw imports are not graph health.** Only linked, active knowledge pages count.

**Graph health KPIs (track daily/weekly):**
- Orphan ratio
- Stale critical page ratio
- Average inbound/outbound links per knowledge page
- MOC coverage by active project
- Skill coverage by active project
- Decision coverage by active project
- Unresolved red link count

### 9.3 Mem0

**Mem0 is recall memory, not policy authority.**

**Session-start seeding:**
1. Derive retrieval query from Focus State intent + task + project
2. Search Mem0 (`wb memory search`)
3. Validate top-N candidates
4. Project bounded, relevant items into Focusa semantic memory

**Session-end writeback:**
- Push candidate facts/learnings only after extraction/validation
- Attach metadata: source session, frame, project, confidence, promotion level

**Kuzu graph role:** Relation recall, entity linkage, project-skill-decision mappings. Not a substitute wiki.

### 9.4 Letta

**Letta stores agent-local continuity.** Persona blocks, narrative state, short/medium horizon statefulness.

**Letta must not become the hidden durable truth source.** If Letta learns something durable → promote to Mem0 and/or Wiki.

**Focusa × Letta:** Focusa injects runtime cognition into prompt layer. Letta remains harness/runtime, not cognition owner.

### 9.5 Context Core

**Operator state must modulate cognition every turn.**

Required fields: interruptibility, confidence, circadian phase, active focus mode, fatigue/overload signals.

Mapping examples:
- `interruptibility=very_low` → queue questions, avoid interruptions
- `late-night` → concise, low-churn output
- `operator overloaded` → reduce branchiness, emphasize execution

**Context Core modulates Focus State; it does not overwrite it.**

### 9.6 Flow Mesh

**Flow Mesh is canonical work graph.** Owns: task status, backlog, dependencies, queue order, completion state.

**Focusa ↔ Flow Mesh bridge:**
- Each active focus frame should map to a Flow Mesh task ID (or explicit no-task reason)
- Required mapping fields: `task_id`, `project_id`, `frame_id`, `session_id`
- Task completion does not silently close focus frame; event path required

### 9.7 wb CLI

**wb is the default agent-facing facade.** Already consolidates: wiki, focusa, memory, ontology, queue, soul, health, session, kaizen.

Direct service-to-service APIs are allowed when latency matters or no wb wrapper exists. But agent-facing workflows default to wb.

### 9.8 Ontology

**Ontology defines types, relations, and action contracts. It is NOT another wiki.**

Core entities: Agent, Role, Operator, Project, Task, Skill, Tool, Decision, Constraint, Memory, Session, Thread, Artifact, Objective, Policy

Core relations: `agent_has_skill`, `project_requires_skill`, `task_advances_project`, `task_advances_objective`, `decision_applies_to_project`, `decision_constrains_task`, `memory_supports_decision`, `artifact_evidences_decision`, `session_updates_project`, `operator_state_modulates_agent`

### 9.9 Wirebot Mode for Pi (`/wbm`) — Cross-Surface Identity

**Wirebot is one person across all surfaces.** Discord, OpenClaw, Pi sessions, Claude Code sessions, mobile — these are all Wirebot's hands. Work done in any surface must flow back to Wirebot's central nervous system.

`/wbm` is a Pi extension that makes Pi sessions part of Wirebot's unified consciousness.

#### Two-Way Bridge (Not Just Read)

```
                    ┌─────────────┐
     /wbm on        │   WIREBOT   │      /wbm catalogues work
   ┌───────────────►│   Central   │◄────────────────────┐
   │  context IN    │   Nervous   │   work meta OUT     │
   │                │   System    │                     │
   │                └─────────────┘                     │
   │                       │                            │
   │    ┌──────────────────┼──────────────────┐         │
   │    │                  │                  │         │
   │    ▼                  ▼                  ▼         │
   │  Mem0            Wiki.js           Scoreboard      │
   │  (memories)      (decisions)       (ships)         │
   │                                                    │
   │                                                    │
   └──────────── Pi Session ────────────────────────────┘
```

**IN (context injection):**
- Operator state from Context Core (mode, interruptibility, circadian)
- Objectives from `objectives.yaml` (P1/P2/P3 priorities)
- Drift + season from Scoreboard (alignment quality)
- Active Focusa frame (what's being worked on)
- SOUL.md pillars digest (behavioral doctrine)
- Recent wiki decisions (project context)

**OUT (work cataloguing):**
- Decisions made during Pi session → Mem0 + Wiki decision page
- Facts discovered → Mem0 memory
- Tasks completed → Scoreboard ship event
- Failures encountered → Mem0 + Focus State
- Files created/modified → Wiki project page update
- Learnings → Mem0 + wiki knowledge page candidate

#### How Cataloguing Works

The extension listens to Pi session events and extracts work metadata:

```typescript
pi.on("agent_end", async (event, ctx) => {
  if (!wbmEnabled) return;
  
  // Extract work meta from this agent turn
  const meta = await extractWorkMeta(event.messages);
  
  // Catalogue to Wirebot's systems
  if (meta.decisions.length > 0) {
    for (const d of meta.decisions) {
      await catalogueDecision(d);     // → Mem0 + Wiki
    }
  }
  if (meta.completions.length > 0) {
    for (const c of meta.completions) {
      await catalogueShip(c);         // → Scoreboard
    }
  }
  if (meta.facts.length > 0) {
    for (const f of meta.facts) {
      await catalogueMemory(f);       // → Mem0
    }
  }
  if (meta.failures.length > 0) {
    for (const f of meta.failures) {
      await catalogueFailure(f);      // → Mem0 + Focusa
    }
  }
});
```

#### Extraction Uses LLM (Not Regex)

Work meta extraction calls MiniMax M2.7 (cheap/fast) with:
- Input: the Pi session messages from this turn
- Prompt: "Extract decisions made, tasks completed, facts discovered, failures encountered. Return structured JSON."
- Budget: ≤500 tokens
- Timeout: 2s
- Fallback: skip cataloguing for this turn, try next turn

#### Cataloguing Destinations

| Work Meta | Destination | Method | Metadata |
|---|---|---|---|
| Decision | Mem0 | `wb memory inject "$DECISION"` | `source:pi, surface:pi, session:$ID` |
| Decision | Wiki | `wb wiki create --path ops/decisions/$DATE --tags decision,pi` | rationale, project link |
| Fact | Mem0 | `wb memory inject "$FACT"` | `source:pi, surface:pi, category:technical` |
| Failure | Mem0 | `wb memory inject "FAILURE: $DETAIL"` | `source:pi, surface:pi, category:failure` |
| Failure | Focusa | `POST :8787/v1/focus/state/update` | failures array |
| Learnings | Mem0 | `wb memory inject "LEARNED: $INSIGHT"` | `source:pi, surface:pi, category:learning` |

**Ships are NOT manually catalogued.** The scoreboard already auto-detects ships from git:
- GitHub releases → `PRODUCT_RELEASE` (10 points)
- Merged PRs → `FEATURE_SHIPPED` (6 points)
- Successful CI runs → `DEPLOY_SUCCESS` (8 points)
- Git discovery scans `/root`, `/home`, `/data` for new commits
- GitHub webhooks provide real-time detection

Pi sessions that commit code are **automatically detected as ships** by the existing git discovery + GitHub webhook system. `/wbm` does not need to duplicate this. Instead, `/wbm` catalogues the **meaning behind the commits** — the decisions, the reasoning, the failures, the learnings — which git can't see.

This means: Pi commits code → scoreboard auto-detects the ship → `/wbm` catalogues WHY that code was written and what was decided along the way → Wirebot knows both WHAT shipped and WHY.

#### Commands

```
/wbm on          → Activate: inject context + start cataloguing
/wbm off         → Deactivate: stop injection + cataloguing
/wbm status      → Show: what's injected + what's been catalogued this session
/wbm deep        → Deep mode: also fetch Mem0 memories + wiki decisions (~1500 tok)
/wbm flush       → Force-catalogue accumulated work meta now
/wbm decisions   → Show decisions catalogued this session
/wbm ships       → Show git ships auto-detected by scoreboard during this session
```

#### Why This Makes Wirebot Smarter

Without `/wbm`:
- Pi does 8 hours of coding work
- Wirebot knows nothing about it
- Operator has to re-explain everything
- Decisions made in Pi are lost
- Ships aren't logged
- The season stays 0W-21L

With `/wbm`:
- Pi's decisions flow to Mem0 → Wirebot recalls them tomorrow
- Pi's commits are auto-detected by Scoreboard git discovery → season record improves
- Pi's failures flow to Focusa → same mistake isn't repeated
- Pi's facts flow to Mem0 → cross-surface knowledge grows
- Wirebot can say "Yesterday in a Pi session we decided to use JWT with PKCE" — even though Wirebot wasn't the one coding

**Wirebot is one person. Pi sessions are Wirebot working with different hands. The work must come home.**

#### Write Safety

- All writes go through `wb` CLI (auditable, rate-limited)
- Mem0 writes use promotion pipeline (§6) — not raw injection
- Wiki writes are schema-validated
- Scoreboard ships require description (no empty events)
- All catalogued items tagged with `source:pi` + `surface:pi` for provenance
- Operator can disable cataloguing independently of context injection (`/wbm on --no-catalogue`)

---

### 9.10 Agent Audit UI + Mobile Access

**Agent Audit** (`audit.philoveracity.com`) is the organism's mobile surface. It is already:
- Running as a Go binary at `/opt/agent-audit/` on `:3020`
- Tunneled via Cloudflare to `audit.philoveracity.com`
- A PWA (installable on Samsung phone — `manifest.json` with standalone display, portrait, dark theme)
- Showing agent sessions, actions, errors, tool/model breakdowns
- Showing Focusa API compliance probe results (CI artifact)

**Current gap:** The audit UI shows only CI probe results for Focusa. It does NOT show live Focusa cognitive state.

**Required: Live Focusa panels in Agent Audit**

The audit UI must become the operator's mobile window into the organism's cognition. Add these panels by querying Focusa API at `:8787`:

| Panel | Focusa Endpoint | What It Shows |
|---|---|---|
| **Focus Stack** | `GET /v1/focus/stack` | Active frame title, goal, depth, parent chain |
| **Focus State** | `GET /v1/ascc/state` | Current intent, decisions, constraints, failures, next steps |
| **Autonomy** | `GET /v1/autonomy` | ARI score, AL level, 6 dimension breakdown |
| **Gate** | `GET /v1/gate/scores` | Surfaced candidates with pressure levels |
| **Thread Thesis** | `GET /v1/threads` | Primary intent, confidence, open questions |
| **Constitution** | `GET /v1/constitution/active` | Active principles + safety rules |
| **Reflection** | `GET /v1/reflect/status` | Last reflection result, scheduler status |
| **Telemetry** | `GET /v1/telemetry/tokens` | Token usage, per-task cost |
| **RFM** | `GET /v1/rfm` | Current RFM level, AIS score |
| **Events** | `GET /v1/events/stream` (SSE) | Live event tail |

**Implementation approach:**
- Agent Audit already polls services for health data
- Add Focusa as another polled service (`:8787`)
- Render panels in the existing Go template engine
- SSE event stream can power a live activity indicator
- All data is read-only — audit UI never mutates Focusa state

**Mobile UX rules:**
- Panels collapse to summary cards on phone (tap to expand)
- Focus Stack shows active frame title + ARI badge at top (always visible)
- Gate candidates show count badge — tap for detail
- Dark theme matches existing audit UI
- PWA offline: show last-known state with staleness indicator

**Why this is the right mobile strategy:**
- No new app to build — extend existing PWA
- Already tunneled, already installable on phone
- Same auth surface as existing audit
- Operator can check cognitive state from Samsung at any time
- No Tauri mobile build complexity

**Effort estimate:** ~4 hours (Go template panels + Focusa HTTP client + CSS)

### 9.11 JARVIS Protocol Integration Points

The JARVIS Plan (`/wirebot/jarvis-plan`) defines 7 capability domains for transforming Wirebot from reactive chatbot to autonomous sovereign agent. The organism spec must map to these domains.

#### JARVIS ↔ Organism Mapping

| JARVIS Domain | Organism System | Status | Gap |
|---|---|---|---|
| 1. System Management | wb CLI (45+ commands), Guardian, root exec | ✅ Infrastructure exists | Lacks initiative — Focusa Focus Gate should surface actions |
| 2. Real-Time Analysis | Context Core (sensors, predictions, RescueTime, calendar, solar) | ✅ **Rich data already flowing** | Not connected to Focusa — §9.5 closes this |
| 3. Orchestration | Flow Mesh (task queue), OpenClaw plugins | ✅ Partially built | Focus Stack ↔ Flow Mesh bridge (§9.6) closes this |
| 4. Health Monitoring | Context Core (RescueTime, screen time, productivity, circadian) | ✅ Data exists | Focusa should modulate behavior from health signals |
| 5. Security & Defense | Guardian, wb health, audit watcher | ✅ Running | Guardian alerts should feed Focusa Intuition Engine |
| 6. Cross-Domain Synthesis | Wiki + Mem0 + Focusa + Context Core | ✅ All running | Integration spec (§§9.1–9.8) closes this |
| 7. Resilience | Focusa Focus State, Mem0, Flow Mesh, memory-syncd | ✅ Persistence exists | Session resume (§12.2) + degraded mode (§8) closes this |

#### Context Core Is JARVIS Layer 2 (Real-Time Analysis)

Context Core already provides extraordinary operator awareness:
- **Location:** where the operator is (VPS, mobile, driving)
- **Mode:** `agent_coding` / `meeting` / `sleeping` / `driving`
- **Interruptibility:** `very_low` / `low` / `medium` / `high`
- **Calendar:** next event, free-until, today's schedule
- **Solar/Circadian:** sunrise/sunset, phase, day length
- **RescueTime:** productivity score, screen hours, social hours, distraction ratio
- **Predictions:** tomorrow's daylight, calendar busyness
- **Day plan:** time blocks with best-for labels (creative, deep work, meetings)
- **Policy:** `agent_should=queue_questions`, `ask_user_now=false`, allowed/forbidden actions

**This is the richest operator-state system in the entire stack.** Focusa MUST ingest this every turn (§9.5).

#### Will Wirebot Get Smarter Over Time?

Yes — through 6 compounding mechanisms:

| Mechanism | How It Works | Gets Smarter Because |
|---|---|---|
| **Procedural memory** | Rules reinforced by use, decay without use | Behavioral patterns converge on what works |
| **Mem0 accumulation** | Facts extracted from every session | Recall improves with more data |
| **Wiki graph growth** | Decisions, skills, knowledge pages accumulate | Richer context injected into prompts |
| **Autonomy calibration** | ARI score measured from turn quality | Earns more independence through evidence |
| **Thread Thesis refinement** | LLM refines "what is this really about" | Understanding deepens per session |
| **Reflection loop** | Periodic self-review of work quality | Self-corrects trajectory without operator input |
| **Kaizen reflections** | Post-session learnings promoted to wiki | Lessons persist across all future sessions |
| **Context Core predictions** | Rolling stats, EWMA, linear regression on operator patterns | Anticipation improves with data |

**The daily growth loop (§12) ensures these mechanisms run continuously, not just when triggered.**

#### JARVIS Phases → Organism Spec Alignment

| JARVIS Phase | Organism Spec Phase | Notes |
|---|---|---|
| Phase 0: Foundation | ✅ Done | SOUL.md, heartbeat, config |
| Phase 1: Smart heartbeat + work queue | Phase 0 + Phase 1 | wiki-agent, Focusa wiring, Flow Mesh bridge |
| Phase 2: Event sensors + triage | Phase 1 (§9.5) | Context Core → Focusa = JARVIS triage engine |
| Phase 3: Proactive check-ins | Phase 5 (§10.6) | Reflection loop + thesis = proactive awareness |
| Phase 4: Resilience | §8 Degraded Mode Matrix | Explicit fallbacks per subsystem |
| Phase 5: Multi-system orchestration | Phase 3 (wiki tools) | wiki_search/read/decide + Flow Mesh |
| Phase 6: Operator wellness | §9.5 Context Core | RescueTime, circadian, productivity already flowing |
| Phase 7: Personality + learning | §6 Promotion Pipeline | Memory → knowledge → procedural rules = personality growth |

#### MiniMax Model Upgrade

Upgrade background inference model from **MiniMax M2.5 → MiniMax M2.7** (newest):
- All internal metacognitive calls (§10)
- Mem0 entity extraction + graph construction
- Worker LLM extraction
- RFM microcell validators
- Reflection loop reasoning
- Nightly contradiction scan + graph gap detection

**Does NOT affect primary model.** Operator-facing calls stay on Kimi K2.5.

**Implementation:**
- Update Mem0 config: `llm.model` → `minimax-m2.7`
- Update Focusa worker LLM endpoint config
- Verify API compatibility (M2.7 should be drop-in)
- Test extraction quality improvement

#### Required Fallbacks (All Integration Points)

Every cross-system call must have an explicit fallback:

| Call | Timeout | Fallback |
|---|---|---|
| Focusa proxy → Kimi | 30s | OpenClaw direct to Kimi (bypass Focusa) |
| Focusa → Context Core | 2s | Use cached operator state with TTL |
| Focusa → Mem0 (session seed) | 3s | Skip seeding, use local semantic memory |
| Focusa → Wiki (context fetch) | 3s | Skip wiki context, use cached |
| Worker → MiniMax M2.7 | 2s | Fall back to regex heuristic |
| RFM microcell → MiniMax M2.7 | 2s | Skip validation, pass response through |
| Reflection → MiniMax M2.7 | 5s | Skip this reflection cycle |
| Session end → Mem0 writeback | 3s | Queue for retry, don't block close |
| Session end → Wiki writeback | 3s | Queue for retry, don't block close |
| Agent Audit → Focusa (panels) | 2s | Show stale data with staleness badge |
| vault → wiki sync | 300s | Skip cycle, retry next interval |
| wiki-agent cycle | 2700s | Abort cycle, retry next scheduled |

**Rule:** No integration failure may block the operator's response or crash any service.

---

## 10. Cognitive Hygiene — Forgetting, Decay, and Waste Elimination

An organism that never forgets becomes senile. Accumulation without pruning is hoarding, not intelligence. The system must **actively sharpen cognition** by eliminating waste, decaying irrelevance, and compacting history.

### 10.1 The Problem Today

| System | Waste | Size | Growing? | Pruning |
|---|---|---|---|---|
| Focusa event log | 35,171 events, full turn content stored | **420MB** | ~1K events/day | ❌ None |
| Focusa snapshots | State saved on every mutation | 10MB | Yes | ❌ None |
| Wiki.js | 952 orphan pages, 937 stale | 1,056 pages | Yes (enrichment) | ❌ wiki-agent stopped |
| Mem0 | No memories yet — but no retention policy | 0 | Will grow | ❌ None planned |
| Focusa semantic memory | TTL field exists but never enforced | 3 records | Slow | ❌ TTL ignored |
| CLT | Append-only, no compaction | Growing | Yes | ❌ Never compacts |
| Sessions | JSONL files accumulate | Varies | Yes | Manual only (`wb session prune`) |

### 10.2 Retention Policies (REQUIRED)

Every data store must have an explicit retention policy:

| Store | Hot (instant access) | Warm (queryable) | Cold (archived) | Delete |
|---|---|---|---|---|
| **Focusa events** | Last 7 days | 8-90 days (indexed) | 91-365 days (compressed) | >365 days: delete payload, keep metadata |
| **Focusa snapshots** | Last 24 hours (every mutation) | 2-30 days (daily snapshots only) | 31-90 days (weekly) | >90 days: keep only monthly |
| **Wiki T1 (active)** | Always in active graph | — | — | Never delete; demote to T2 after 90d untouched |
| **Wiki T2 (reference)** | Queryable, not auto-injected | — | — | Never delete; demote to T3 after 90d untouched |
| **Wiki T3 (archive)** | Exists, not maintained | — | — | Never delete; invisible to cognition |
| **Wiki T4 (raw)** | Exists, unprocessed | — | — | Never delete; candidate for reduction pipeline |
| **Wiki T5 (quarantine)** | Flagged for operator | — | — | Never auto-delete; operator decides |
| **Mem0 memories** | Last 90 days or recalled in last 30 | 91-365 days | — | >365 days without recall: candidate for removal |
| **Focusa semantic memory** | Enforce TTL if set | — | — | Expired TTL → remove on next decay tick |
| **Focusa procedural rules** | Weight > 0.1 | Weight 0.01-0.1 (dormant) | — | Weight < 0.01: remove |
| **CLT nodes** | Last 1,000 nodes (navigable) | Older: compacted summary nodes | — | Never delete, but compact aggressively |
| **Sessions** | Last 10 sessions | Older: pruned to 512KB | — | `wb session prune` runs nightly |

### 10.3 Event Log Compaction (Most Urgent)

The event log is **420MB and growing**. 228MB is raw `turn_started` content. 134MB is full assistant output in `turn_completed`.

**Implementation:**
1. **Immediate:** Strip large payloads from events older than 7 days
   - Keep: event type, timestamp, turn_id, frame_id, token counts
   - Remove: raw_user_input, assistant_output, correlation_id content
   - Estimated savings: ~350MB (80% of current size)
2. **Daily:** Run SQLite `VACUUM` after compaction
3. **Weekly:** Archive events older than 90 days to compressed JSONL export
4. **Ongoing:** `turn_completed` events should store a **summary hash**, not full content
   - Full content goes to ECS Reference Store (where it belongs)
   - Event stores handle reference only

### 10.4 Memory Decay Pipeline

**Procedural rules** (already have decay, needs threshold):
- Current: `weight *= 0.99` every 30s, but rules never removed
- Required: if `weight < 0.01` for 7+ days → remove rule
- Required: if `weight < 0.1` for 30+ days and never reinforced → remove rule

**Semantic memories** (needs TTL enforcement):
- Current: TTL field exists but never checked
- Required: on each decay tick, check TTL → remove expired entries
- Required: memories not accessed in 90 days → flag for review

**Mem0 memories** (needs forgetting):
- Add `last_recalled_at` timestamp to every memory
- Memories not recalled in 365 days → candidate for removal
- Duplicate/near-duplicate memories → merge (keep highest confidence)
- Contradicted memories → demote confidence, eventually remove

### 10.5 Wiki Knowledge Triage (Importance Tiers, Not Deletion)

**Nothing is deleted from the wiki.** Old docs may be irrelevant now but valuable later. Instead of pruning, the system assigns **importance tiers** that control visibility, injection priority, and maintenance effort.

#### Importance Tiers

| Tier | Tag | Meaning | Prompt Injection | Maintenance | Example |
|---|---|---|---|---|---|
| **T1 — Active** | `importance:active` | Actively used in current work | ✅ Always eligible | wiki-agent maintains links | Active project pages, current decisions, live skills |
| **T2 — Reference** | `importance:reference` | Useful background, not active | ⚠️ Only when retrieved | Periodic link check | Completed project pages, past decisions, learned skills |
| **T3 — Archive** | `importance:archive` | Historical record, rarely needed | ❌ Never injected | No maintenance | Old meeting notes, past season data, completed ops |
| **T4 — Raw** | `importance:raw` | Unprocessed source material | ❌ Never injected | Candidate for reduction | Joplin imports, ChatGPT exports, raw dumps |
| **T5 — Quarantine** | `importance:quarantine` | Flagged for operator review | ❌ Never injected | Operator decides | Contradicted content, suspected junk, duplicate |

#### Tier Assignment Rules

- **New knowledge pages** (`/notes/*`) start at T1 (active)
- **Session captures** (`/ops/sessions/*`) start at T2 (reference)
- **Imports** (`/joplin-import/*`, `/ai-chats/*`) start at T4 (raw)
- **Pages not touched in 90 days** → auto-demote one tier (T1→T2, T2→T3)
- **Pages recalled/referenced by agents** → promote one tier
- **Pages flagged by contradiction scan** → T5 (quarantine)
- **Operator can manually set any tier**

#### How Tiers Sharpen Cognition

- **Expression Engine** only queries T1+T2 pages for context injection
- **Mem0 seeding** only extracts from T1+T2 pages
- **wiki-agent** only maintains links on T1+T2 pages (saves cycles)
- **Graph health KPIs** only count T1+T2 in link density calculations
- **Orphan detection** only flags T1+T2 orphans as problems
- **T3+T4+T5 pages exist but don't consume cognitive resources**

This means the organism's active cognition gets sharper over time as irrelevant pages demote to T3/T4, while the active knowledge graph (T1+T2) stays dense and high-signal.

#### Implementation

- Add `importance:*` tags via `wb wiki` (tag system already exists — 83 tags in use)
- wiki-agent assigns initial tiers based on namespace + age
- Nightly: scan for auto-demotion candidates (untouched 90+ days)
- Nightly: scan for auto-promotion candidates (recently recalled)
- `wb wiki stats` should report tier distribution
- Agent Audit should show tier breakdown

#### Existing Tags That Map to Tiers

| Existing Tag | Maps To |
|---|---|
| `quarantine` (29 pages) | T5 |
| `operator-review` (29 pages) | T5 |
| `archive` (tag exists) | T3 |
| `vault-import` (825 pages) | T4 |
| `important` (tag exists) | T1 |

#### Unlinked Page Policy

**The term "orphan" is retired.** An unlinked page is not unwanted. It's just not connected yet.

The operator writes random notes, ideas, fragments, thoughts. These have value even if they don't fit neatly into the graph. A note with no inbound links might be:
- A thought that hasn't found its home yet
- A seed idea that will matter in 6 months
- A quick capture during a conversation
- A personal reference the operator values
- An import that hasn't been processed

**None of these are waste. All of them belong.**

**The system's job is to CONNECT pages, not judge them.**

**Core axiom: If the operator recorded it, it has meaning.** The act of writing something down — even a single phrase — means it was important enough to capture. The system must assume every fragment has meaning until the operator explicitly says otherwise. No agent may classify any operator-created content as meaningless, low-value, or candidates for removal. The only entity that can declare something meaningless is the operator, in their own words, in the current session.

#### Fragment Reasoning — Puzzle Assembly

The operator thinks in fragments. A short note, a phrase, a half-formed idea, a quote with no context, a name with no explanation. These are **pieces of a larger picture that the operator sees but hasn't fully articulated.**

The system must treat unlinked fragments as **puzzle pieces**, not orphans:

1. **Accumulate context over time.** A fragment that means nothing today may click into place after 3 more conversations, a new project, or a life event. The system must revisit fragments periodically with fresh context.

2. **Cross-reference against operator knowledge.** When a new fragment appears, search across:
   - Recent conversations (session captures)
   - Mem0 memories (patterns, preferences, past statements)
   - Wiki knowledge pages (projects, decisions, people)
   - SOUL.md / operator profile (values, goals, philosophy)
   - Letta core memory (recent narrative context)
   
   Ask: "Does this fragment connect to something the operator has been thinking about?"

3. **Hypothesize connections.** If the fragment says "Winter isn't the end. It's the pause that makes the next spring possible" — the system should reason:
   - Is this about a project that stalled?
   - Is this about a personal season the operator is in?
   - Does this connect to the TEP book themes?
   - Is this a philosophy that should influence SOUL.md?
   
   Generate connection hypotheses as wiki suggestions (not assertions).

4. **Revisit with deeper knowledge.** As the system learns more about the operator's internal reasoning — through conversations, decisions, memory accumulation — previously opaque fragments may suddenly make sense. The nightly reflection loop should periodically re-examine unlinked fragments against the current knowledge state.

5. **Surface insights, not cleanup tasks.** When the system finds a potential connection, surface it as:
   - "This note from March might relate to your decision about X"
   - "This fragment echoes something you said about Y in February"
   
   Not: "This page has no links, should we archive it?"

**Implementation:**
- Nightly: wiki-agent picks N unlinked fragments (start with 5/night)
- For each: LLM call with fragment + operator context (Mem0 + recent wiki decisions + SOUL themes)
- Output: connection hypotheses (0-3 per fragment)
- If confident connection found → create wiki link + add to relevant MOC
- If hypothesis only → add as wiki comment: "Possible connection: [hypothesis]"
- If nothing found → leave alone, try again in 30 days with more context
- Track: fragments successfully connected over time (puzzle completion rate)

#### Unlinked Page Handling

| Action | Who | When |
|---|---|---|
| Detect unlinked pages | wiki-agent (nightly) | Automatic |
| Attempt to find and create links | wiki-agent | Automatic — search for related MOCs, projects, concepts |
| Assign importance tier if untiered | wiki-agent | Automatic — based on namespace + content analysis |
| Surface connection suggestions | Agent Audit UI | Ongoing — "these 5 pages might relate to Project X" |
| Delete any page | **Operator only, same-session auth** | Never automatic, never carried across sessions |

**wiki-agent's job with unlinked pages:**
1. Try to link them — find related content, add to MOCs, suggest connections
2. Assign a tier if missing — T1-T4 based on namespace and content
3. Surface suggestions to operator — "this note might connect to X"
4. **Never flag as waste. Never recommend deletion. Never quarantine based on link count alone.**

**Deletion rules (unchanged):**
- No agent may delete any wiki page autonomously — ever
- Operator must explicitly authorize deletion in the same session
- Authorization does not carry across sessions
- If in doubt, keep the page and try harder to connect it

**Link rot detection:**
- Weekly: scan for broken wiki links in T1+T2 pages → wiki-agent candidates
- Track unresolved red link count in T1+T2 only (not T3+T4+T5)
- Red links in T4 (raw) are expected and not a problem

### 10.6 CLT Compaction

Per `docs/17-context-lineage-tree.md`: compaction inserts summary nodes, never deletes.

**Implementation:**
- After 1,000 nodes: compact oldest 500 into 50 summary nodes
- Summary node contains: time range, key decisions, frame transitions, outcome
- Original nodes remain in cold storage (compressed), not in active tree
- Active tree stays navigable and bounded

### 10.7 Contradiction-Driven Forgetting

When a new decision contradicts an old one:
1. Old decision's wiki page gets `superseded_by` link
2. Old Mem0 memory gets confidence reduced to 0.1
3. Old procedural rule (if exists) gets weight halved
4. Contradiction logged as event for auditability
5. The NEW decision is the only one injected into prompts

### 10.8 Daily Hygiene Loop

Add to §12.3 Nightly:
```
Nightly hygiene:
  1. Event log: compact events older than 7 days (strip payloads)
  2. Snapshots: thin to daily-only for events older than 24h  
  3. Sessions: wb session prune --budget 512KB
  4. Procedural rules: remove weight < 0.01
  5. Semantic memory: remove expired TTLs
  6. Mem0: deduplicate near-identical memories
  7. Wiki: auto-demote untouched T1→T2, T2→T3 (90-day rule); promote recently-recalled pages
  8. CLT: compact if > 1,000 active nodes
  9. SQLite VACUUM on focusa.sqlite
```

### 10.9 Cognitive Sharpening Metrics

Track weekly:
- Event log size (should stabilize, not grow linearly)
- Procedural rule count (should converge, not accumulate)
- Mem0 memory count (should grow slowly, not explosively)
- Wiki T1+T2 page count (active knowledge — should grow steadily)
- Wiki T1+T2 link density (should increase)
- Wiki T4 reduction rate (raw → T1/T2 or confirmed T3)
- Wiki T5 queue length (should stay short — operator reviews regularly)
- CLT active node count (should stay bounded)
- Snapshot storage (should stay bounded)

**The system is sharp when these metrics stabilize. It is hoarding when they grow linearly without bound.**

---

## 10A. Historical Inference Pipeline — Processing the Backlog

The organism has **terabytes of historical material that has never been inference-processed:**

| Source | Volume | Processed | Gap |
|---|---|---|---|
| Pi sessions | 111 files, 1.7GB | 0% | Every coding decision, failure, learning |
| OpenClaw sessions | 2,584 sessions | 0% | Every operator conversation, correction, preference |
| Obsidian vault | 876 .md files | 8% (71 files) | Years of notes, ideas, fragments |
| Joplin imports | 571 files | 0% | Historical personal notes |
| ChatGPT exports | 266 files | 27% (71 files) | Product ideas, research, strategy |
| Fact YAMLs | 2,498 files | 12% (286 files) | Extracted conversation facts |

**The machinery exists.** The scoreboard's Go API has:
- `POST /v1/memory/extract-vault` — LLM walks a directory, extracts structured memories from each .md
- `POST /v1/memory/extract-conversation` — LLM processes conversation turns
- Watermark system (tracks what's been processed, resumes on restart)
- Chunking (6KB chunks with overlap for long docs)
- Approval queue (memories go through review before promotion)
- Rate limiting (2s between files)

**But nothing runs it on a schedule.**

### 10A.1 Nightly Historical Processing (NEW)

Add to the nightly loop (§13.3):

```
Nightly historical inference:
  1. Vault extraction: process N unprocessed .md files (default N=20)
     POST :8100/v1/memory/extract-vault {path: "/data/wirebot/obsidian", limit: 20}
  2. Session extraction: process M oldest unprocessed OpenClaw sessions (default M=5)
     For each: POST :8100/v1/memory/extract-conversation {messages: [...]}
  3. Pi session extraction: process K oldest unprocessed Pi sessions (default K=2)
     Parse .jsonl, extract assistant turns, POST :8100/v1/memory/extract-conversation
  4. Fact backfill: push N unsynced fact YAMLs to Mem0 (default N=50)
     Continue from watermark in /data/wirebot/scoreboard/vault_watermark.json
```

**Rate:** At 20 vault files + 5 sessions + 2 Pi sessions + 50 facts per night:
- Vault: 805 remaining / 20 per night = **~40 nights to full processing**
- Sessions: 2,584 / 5 per night = **~517 nights** (increase M for faster catch-up)
- Pi sessions: 111 / 2 per night = **~56 nights**
- Facts: 2,212 / 50 per night = **~45 nights**

**Catch-up mode:** For initial backlog, run with higher limits:
- `limit=100` for vault (weekend batch)
- `limit=20` for sessions (weekend batch)
- `limit=10` for Pi sessions (weekend batch)

### 10A.2 What Gets Extracted

The LLM extraction produces structured memories:
- **Facts:** "Operator prefers telegraph mode", "Server uses AlmaLinux 8.10"
- **Decisions:** "Chose JWT with PKCE over implicit flow", "Selected bind mounts over named volumes"
- **Preferences:** "Operator values specificity and vulnerability in writing"
- **Patterns:** "Operator starts new projects when P1 is blocked"
- **Relationships:** "Startempire Wire depends on TEP book publication"
- **Failures:** "Podman stop -a killed non-target containers"
- **Ideas:** "AI coaching model: Idea → Launch → Growth with checklist scoring"

Each memory includes provenance (source file, extraction date, confidence, chunk number).

### 10A.3 Where Extracted Memories Go — The WINS Portal Approval Flow

**The promotion pipeline already exists.** It is the Scoreboard's memory queue + WINS portal approval UI + 5-sink delivery system.

**The flow:**
```
LLM extracts memory from historical item
    ↓
QueueMemoryForApproval() → memory_queue table (status: pending)
    ↓
Operator reviews in WINS portal (audit.philoveracity.com or :8100)
    │  Tinder-style swipe: approve / reject / correct-then-approve
    │  797-line Svelte UI with gamified review (power meter, streaks)
    ↓
On approve → writebackApprovedMemory()
    ↓
Auto-deliver to ALL 5 sinks:
    1. mem0        → Mem0 vector store (:8200)
    2. memory_md   → MEMORY.md in workspace
    3. fact_yaml   → fact YAML file in /home/wirebot/workspace/memory/facts/
    4. wiki        → Wiki.js knowledge page
    5. letta       → Letta core memory block (:8283)
    ↓
Delivery worker runs every 20s (retry with backoff on failure)
```

**Current state:** 15 memories pending review (nightly diary entries Mar 12-26). None approved. The UI is built. The queue works. The delivery system works. The operator hasn't been reviewing.

**CRITICAL: All historical inference output MUST route through this existing queue.** Do not create a parallel approval system. The `ExtractMemoriesFromDocument` and `ExtractMemoriesFromConversation` functions already call `QueueMemoryForApproval()` which feeds the WINS portal.

**What the historical pipeline adds is VOLUME to the queue** — not a new pipeline. More items in → operator reviews more → more memories delivered to all 5 sinks → system gets smarter.

**Implication for operator:** When the nightly historical processing starts producing 20-50 new memory candidates per night, the WINS portal queue will grow. The operator needs a review cadence (daily 5-min review session) or the queue will back up. Consider: auto-approve for high-confidence items (>0.95) from trusted sources (nightly diary, vault notes with provenance).

### 10A.6 Pipeline Integrity Audit (Skeptical Evaluation)

**Audited 2026-04-02 by tracing every code path from source → extraction → queue → approval → delivery → sinks.**

#### 6 Critical Gaps In The Existing Pipeline

| # | Gap | Severity | Detail |
|---|---|---|---|
| 1 | **Nightly diary: no LLM inference** | HIGH | `queue_and_approve_memory()` pushes raw template text and auto-approves. No LLM reasons about the content. 15 pending items are actually auto-approve failures. |
| 2 | **Conversation extraction: only last message** | HIGH | `agent_end` hook sends only final user+assistant exchange, not full conversation. Rate limited to once/2min. 97% of every conversation never inference-processed. |
| 3 | **Focusa events: no extraction exists** | HIGH | 35K events, 420MB. Contains TurnCompleted with full output, FocusStateUpdated with decisions. No code anywhere extracts memories from Focusa events. No endpoint. No script. |
| 4 | **Pi sessions: no parser exists** | HIGH | 111 files, 1.7GB .jsonl. No code reads Pi's session format. `extract-conversation` expects single exchanges, not multi-turn sessions with tool calls. |
| 5 | **Historical sessions: no batch processor** | HIGH | 2,584 OpenClaw sessions. No code iterates historical session files. No watermark for sessions (vault has one, sessions don't). |
| 6 | **Wiki sink: shallow append-only** | MEDIUM | `wikiAppendFact()` appends a bullet point to one page. Does NOT create decision pages, add wiki links, set schemas, or connect to knowledge graph. |

#### What Must Be Built

**1. Full-conversation extraction (not just last message):**
```
On agent_end: send FULL conversation (all turns), not just last exchange.
Chunk into windows of 10-20 turns.
Each chunk gets separate LLM extraction call.
Remove 2-minute rate limit for session-end extraction (it's a single event, not spam).
```

**2. Focusa event extractor (NEW):**
```
New endpoint or script that:
  - Reads Focusa SQLite event log
  - Filters for TurnCompleted + FocusStateUpdated events
  - Extracts decisions, failures, constraints from event payloads
  - Feeds through QueueMemoryForApproval()
  - Watermarks by event_id to avoid reprocessing
```

**3. Pi session extraction (TWO MODES):**

**Live mode (in focusa-pi-bridge extension, §9.9 /wbm):**
The Pi extension is already inside every session. When `/wbm` is active:
- `agent_end` hook receives `event.messages` — the full conversation from this prompt
- Extension extracts decisions/facts/failures/learnings via LLM (MiniMax M2.7)
- Routes through `QueueMemoryForApproval()` into WINS portal queue
- **This is the steady-state architecture** — no cold parsing needed for future sessions

**Historical backfill (one-time batch for 111 existing sessions):**
```
One-time script that:
  - Reads existing Pi .jsonl session files (1.7GB backlog)
  - Parses into conversation turns (skip raw tool XML/noise)
  - Windows into chunks of 15-20 turns
  - Feeds each chunk to ExtractMemoriesFromConversation()
  - Watermarks by session file + offset
  - Rate: 2 sessions/night until backlog cleared (~56 nights)
  - After backlog cleared: script retired, live mode handles all future sessions
```

**4. Historical session batch processor (NEW):**
```
New script/endpoint that:
  - Iterates OpenClaw session files (oldest first)
  - Parses each into full conversation
  - Feeds through ExtractMemoriesFromConversation() (chunked)
  - Watermarks by session_id
  - Rate: 5 sessions/night
```

**5. Wiki sink upgrade:**
```
Replace wikiAppendFact() bullet-point append with:
  - If memory is a decision: create wiki decision page with rationale + project link
  - If memory is a skill/pattern: create or update skill page
  - If memory is a fact: append to relevant project/concept page with wiki link
  - All writes schema-validated
  - All writes linked to at least one MOC
```

**6. Diary extraction upgrade:**
```
Replace raw text push with:
  - LLM reads diary content
  - Extracts structured memories (same as vault extraction)
  - Routes through QueueMemoryForApproval() (pending, not auto-approved)
  - Operator reviews in WINS portal like everything else
```

#### LLM Model for Extraction

Currently: `EXTRACTION_MODEL` defaults to `kimi-coding/k2p5` (the primary expensive model).
Should be: MiniMax M2.7 for all extraction work. Primary model is for operator responses only.
Update: `EXTRACTION_MODEL=minimax/MiniMax-M2.7` in scoreboard env.

### 10A.4 Pi Session Processing (Special)

Pi sessions are 1.7GB of .jsonl containing full coding sessions with tool calls, file edits, decisions, and reasoning. These are the richest source of technical knowledge in the system.

**Extraction approach:**
1. Parse .jsonl into conversation turns
2. Filter: keep only assistant messages + tool results with significant content (>200 chars)
3. Chunk into conversation windows (20 turns each)
4. For each chunk: `POST /v1/memory/extract-conversation`
5. Extracted memories tagged `source:pi-session, session_file:$FILENAME`

**What Pi sessions contain that nothing else does:**
- Architectural decisions made during implementation
- Why specific approaches were chosen over alternatives
- What failed and why before the working solution
- Performance characteristics discovered empirically
- Patterns in how the operator directs coding work

### 10A.5 Progress Tracking

Track in daily intelligence metrics (§13.6):
```yaml
historical_inference:
  vault_files_remaining: N
  vault_files_processed_today: N
  sessions_remaining: N
  sessions_processed_today: N  
  pi_sessions_remaining: N
  pi_sessions_processed_today: N
  facts_remaining: N
  facts_backfilled_today: N
  total_memories_extracted_today: N
  estimated_days_to_complete: N
```

**The system gets smarter not just from new conversations, but from finally understanding its own history.**

---

## 10B. Intelligence Gaps — Verified Against Scoreboard + Go API

Audited 2026-04-02 against live scoreboard API (`:8100`), Context Core (`:7400`), Focusa (`:8787`), pairing engine source, NLP extractor, trust metrics, kaizen tables, and memory extraction pipeline.

### 10A.1 Disconnected Intelligence (Systems Exist But Don't Talk)

The scoreboard's Pairing Engine is the most sophisticated operator-modeling system in the stack. It has:
- **Drift scoring** — 5-component operator-agent alignment (intent, response flow, action latency, override rate, stall gap)
- **R.A.B.I.T. detection** — distraction pattern recognition (tool hopping, social media, perfectionism, avoidance, emotional spiral)
- **Ghost Drift** — behavioral persistence tracking on silent days
- **DISC personality profiling** — NLP-derived communication style model
- **Emotional feature extraction** — frustration, urgency, vulnerability, enthusiasm detection
- **Modesty Reflex** — how guarded the operator is being (caps drift ceiling)
- **Neural Handshake** — daily sync ritual with streak tracking

**None of this feeds into Focusa.** The two most intelligent systems don't talk to each other.

**Required integration:**

| Scoreboard Signal | Focusa Integration Point | Effect |
|---|---|---|
| Drift score + signal | Focus State constraints | Low drift → "Re-establish alignment before deep work" |
| R.A.B.I.T. active | Intuition Engine signal | Surface via Focus Gate: "Operator may be spiraling" |
| Ghost Drift (silent days) | Reflection loop input | "Operator disengaged for N days — what needs attention?" |
| DISC profile | Expression Engine modulation | Match communication style to operator preference |
| Emotion features | Context Core → Focusa | Frustration detected → adjust tone, reduce ask-back |
| Modesty Reflex | Autonomy calibration | High guard → more transparency, lower autonomy ceiling |
| Override rate | Procedural rule generation | Repeated overrides → "stop doing X" rule |
| Neural Handshake streak | Session start context | Streak broken → "Check in with operator" |

### 10A.2 Idle Learning Machinery

These systems are **built and running but producing zero output:**

| System | Status | Evidence | Root Cause |
|---|---|---|---|
| Kaizen reflections | 0 reflections | `wb kaizen list` → empty | Wirebot not writing kaizen blocks, or plugin not extracting |
| Trust metrics | 0 corrections, 0 fabrications, 0 self-assessments | `wb trust status` → all zeros | Nothing writes to trust_metrics table |
| Memory extraction | Pipeline exists (`ExtractMemoriesFromConversation`) | `wb memory list` → 0 memories | Extraction not triggered or approval queue blocked |
| Season scoring | 0W-21L, 0 ships in 21 days | `wb score` → score 0 | Ships not being logged via `wb ship` |
| Drift scoring | Score 14, "disconnected" | Last handshake: Feb 13 (49 days ago) | No daily handshake happening |

**These are not code gaps. They are activation gaps.** The machinery exists. It needs to be turned on and fed data.

**Required activation:**
1. **Kaizen:** Ensure Wirebot's heartbeat plugin extracts kaizen blocks from every session and writes to kaizen table
2. **Trust:** Wire operator corrections ("no", "that's wrong", rephrasing) into trust_metrics via memory bridge plugin
3. **Memory:** Trigger `ExtractMemoriesFromConversation` after session end, route to Mem0 via approval queue
4. **Ships:** Auto-detect shippable events (git tags, deployments, wiki pages created, tasks completed) and log via scoreboard
5. **Handshake:** Integrate morning handshake into Wirebot's proactive check-in flow

### 10A.3 True Gaps (No System Addresses These)

#### Gap 1: No Self-Initiated Questions
No system asks itself:
- "What don't I know about the operator that I should?"
- "What assumptions am I making that I haven't verified?"
- "What has the operator stopped talking about?" (avoidance or resolution?)
- "What would I do differently if I knew X?"

**Required:** Nightly reflection loop (§11) should include a self-questioning pass. LLM generates 3 questions the system can't currently answer. These become wiki research candidates or operator conversation prompts.

#### Gap 2: No Multi-Agent Learning
Engineering agents (Claude Code, Pi, OpenCode) work on the same infrastructure but:
- If Claude discovers a server pattern, Pi doesn't know
- Engineering learnings don't feed into Wirebot's knowledge
- No cross-agent skill transfer

**Required:** Engineering agent session summaries should flow through the promotion pipeline (§6) into wiki knowledge pages. `wb wiki create` from agent session captures. Focusa Focus State decisions from engineering sessions should promote to shared Mem0.

#### Gap 3: No "Why" Tracking
The system tracks WHAT happened (events) and WHEN (timestamps) but not WHY the operator made a decision. The reasoning behind choices is lost.

**Required:** Decision pages in wiki must include rationale, not just the decision. The `wiki_decide` tool (§14.3) already has a `rationale` field — enforce its use. Memory extraction should specifically look for "because" / "the reason" / "I chose X over Y" patterns.

#### Gap 4: No Operator Preference Evolution Tracking
The operator's preferences change over time. The system doesn't detect "you used to prefer X but now you prefer Y" — it just overwrites.

**Required:** When a Mem0 memory is updated, preserve the old value with timestamp. Track preference drift over time. Surface to operator: "You shifted from X to Y over the last month — is that intentional?"

#### Gap 5: No Stall Detection on Objectives
`objectives.yaml` has 3 assets with milestones. No system checks:
- "TEP book hasn't advanced in 2 weeks"
- "SEW network is blocked by TEP but TEP isn't moving"
- "Wirebot is P3 but getting all the engineering attention"

**Required:** Weekly objective review in reflection loop. Compare git activity, wiki changes, Flow Mesh task completions, and scoreboard ships against stated priorities. Surface misalignment: "P1 asset stalled while P3 asset consumed 80% of effort."

#### Gap 6: No Confidence Calibration
When the system says "confidence: 0.85" — is it actually right 85% of the time? No calibration exists.

**Required:** Track predictions vs outcomes. When the system predicts something (thesis confidence, deliberation confidence, evaluation quality score) — compare against what actually happened. Over time, calibrate: "When I say 0.85, I'm actually right 72% of the time → adjust."

#### Gap 7: No Cross-Domain Synthesis
Context Core has RescueTime (6.5h social media), calendar (meeting at 9am), objectives (TEP book P1), scoreboard (0W-21L). No system connects:
- "6.5h social media + 0 ships + TEP stalled = pattern"
- "Productivity peaks at 9am but calendar blocks it with meetings"
- "3 invoices overdue while building features nobody pays for"

**Required:** Weekly cross-domain synthesis in reflection loop. Pull from: Context Core (productivity, calendar, circadian), scoreboard (ships, drift, season), objectives.yaml (priorities, milestones), wiki (decisions, project pages), Flow Mesh (task completion rate). Generate insight report. Surface the 3 most important cross-domain observations.

---

## 11. Metacognitive Reasoning Layer

The organism's intelligence comes from **thinking about thinking** — not just responding to operator input. Focusa exists so that every turn benefits from structured reasoning before, during, and after the model call. Not every LLM call is a direct response to the operator. Internal reasoning calls are how the system produces richer, more grounded answers.

### 10.1 The Thinking Architecture

The Focusa spec mandates:
- `docs/01-architecture-overview.md`: **"< 20ms additional overhead on prompt assembly"**
- `docs/G1-10-workers.md`: **"async, non-blocking, never block hot path"**
- `docs/36-reliability-focus-mode.md`: **"Validator microcells are invoked in parallel"**

Metacognitive LLM calls must **NOT** block the operator's response. Instead they run **after the response is already sent**. Each turn benefits from the **previous** turn's background thinking.

```
Operator input arrives
    │
    ▼
┌─────────────────────────────────────────┐
│ HOT PATH — <20ms, deterministic         │
│                                         │
│ 1. EXPRESSION ENGINE (no LLM call)     │  Assembles prompt from Focus State
│    Pure assembly from pre-computed      │  that was ALREADY enriched by the
│    state — Focus State, thesis,         │  previous turn's background work:
│    rules, memories, wiki context,       │  - LLM-extracted decisions/constraints
│    operator state — all ready to go     │  - Refined Thread Thesis
│                                         │  - Promoted procedural rules
│ 2. MODEL EXECUTION                     │  Primary LLM call → operator
│    (Kimi K2.5 / Qwen)                  │
│                                         │
│ 3. RESPONSE RETURNED TO OPERATOR       │  ← Operator gets answer HERE
└──────────┬──────────────────────────────┘
           │
    Response already sent. Now think.
           │
           ▼
┌─────────────────────────────────────────┐
│ BACKGROUND — async, non-blocking       │
│ (runs AFTER response is delivered)      │
│                                         │
│ 4. LLM EXTRACTION (parallel workers)   │  Extract decisions, constraints,
│    Replaces regex heuristics            │  failures, skills, memory candidates
│    ≤500 tok each, ≤2s timeout          │  Feed results into Focus State
│                                         │  → available for NEXT turn
│                                         │
│ 5. POST-TURN EVALUATION                │  "Did it answer well?"
│    (async LLM, cheap model)            │  Consistency + constraint check
│    If bad → flag for next turn          │  Quality note into Focus State
│    If terrible + R1 → regenerate *      │
│                                         │
│ 6. THESIS REFINEMENT (every Nth)       │  Update "what is this really about"
│    (async LLM, cheap model)            │  Results → Focus State + thesis
│    ≤400 tok, feeds next turn            │  → richer assembly next time
│                                         │
│ 7. FOCUS STATE UPDATE                  │  Worker results → ASCC delta
│    (deterministic reducer, no LLM)     │  Decisions/constraints/failures
│                                         │  promoted into live Focus State
│                                         │
│ 8. MEMORY PROMOTION                    │  Candidates → promotion pipeline
│    (async)                             │  Mem0 / Wiki / procedural rules
└─────────────────────────────────────────┘
           │
    * Regeneration (R1+ only) is the ONE case
      that may delay before response is sent.
      At R0 (normal), response is always immediate.
           │
    Meanwhile, on separate cadences:
           │
           ▼
┌─────────────────────────────────────────┐
│ PERIODIC METACOGNITION                  │
│                                         │
│  9. REFLECTION LOOP (hourly)           │  LLM-backed work quality review
│                                         │  Observations, risks, recommendations
│                                         │
│ 10. INTUITION ENGINE (continuous)      │  Temporal, repetition, consistency
│     (no LLM — signal aggregation)      │  pattern detection → Focus Gate
│                                         │
│ 11. CONTRADICTION SCAN (nightly)       │  Wiki vs Mem0 vs Focus State
│     (LLM-backed)                       │  consistency check
│                                         │
│ 12. GRAPH GAP DETECTION (nightly)      │  Missing knowledge, unlinked pages,
│     (LLM-backed)                       │  skill gaps, orphan reduction
└─────────────────────────────────────────┘
```

**Why this is fast:** The operator never waits for metacognition. Background thinking enriches the Focus State that the NEXT turn's Expression Engine assembles from. Each turn benefits from the previous turn's thinking. The system gets smarter turn-over-turn without adding latency.

**The one exception:** At RFM level R1+, post-turn evaluation may trigger regeneration BEFORE the response is sent. This is the deliberate tradeoff for high-risk tasks — correctness over speed. At R0 (normal), the response is always immediate.

### 10.2 Internal LLM Calls — Not Every Call Is For The Operator

The organism makes LLM calls that the operator never sees. These are **metacognitive calls** — the system thinking about its own thinking:

| Call Type | When | Purpose | Visible to Operator |
|---|---|---|---|
| **Deliberation** | Pre-turn | Assess intent, retrieve context, evaluate confidence | No |
| **Execution** | During turn | Generate response to operator | Yes |
| **Evaluation** | Post-turn | Check quality, consistency, constraint compliance | No |
| **Extraction** | Post-turn | Extract decisions, failures, skills, memory candidates | No |
| **Thesis Refinement** | Post-turn | Update Thread Thesis — "what is this really about" | No |
| **RFM Validation** | Post-turn (conditional) | Microcell validators check output correctness | No |
| **Reflection** | Scheduled (hourly) | Review work quality and focus trajectory | No |
| **Contradiction Scan** | Nightly | Check wiki vs Mem0 vs Focus State consistency | No |
| **Graph Gap Detection** | Nightly | Find missing knowledge, unlinked pages | No |

### 10.3 Worker Upgrade Directive — Regex → LLM

The 5 current workers use regex heuristics. **All must be upgraded to LLM-backed inference.**

| Worker | Current | Upgrade To |
|---|---|---|
| `ClassifyTurn` | `contains("fix")` → "correction" | LLM intent classification with confidence |
| `ExtractAsccDelta` | Keyword scanning for decisions/failures | LLM structured extraction (JSON output) |
| `DetectRepetition` | Line dedup ratio | LLM semantic similarity detection |
| `ScanForErrors` | Pattern match `"error:"`, `"panic:"` | LLM error analysis with root cause |
| `SuggestMemory` | Look for "always"/"never" | LLM fact extraction with provenance |

**Implementation approach:**
- Workers call a local/cheap model (MiniMax M2.5 or Qwen-small) — NOT the primary expensive model
- Budget: ≤500 tokens per worker call
- Timeout: ≤2 seconds per worker
- Fallback: if LLM call fails, fall back to current regex heuristic
- Workers remain async, non-blocking, advisory

### 10.4 RFM Microcell Upgrade Directive

Per `docs/36-reliability-focus-mode.md`: RFM validators are supposed to be **isolated sub-agents** making LLM calls. Currently they are heuristic-only.

**Upgrade plan:**

| Microcell | Current | Upgrade To |
|---|---|---|
| Schema Validator | Not implemented | LLM checks output structure against expected format |
| Constraint Validator | Not implemented | LLM checks output against active frame constraints |
| Consistency Validator | Not implemented | LLM checks for contradictions with prior decisions |
| Reference-Grounding Validator | Not implemented | LLM checks claims against Reference Store / CLT |

**Rules:**
- Microcells have **isolated context** — they do NOT see full session history
- Each microcell gets: the candidate output + relevant constraints/references only
- Budget: ≤300 tokens per microcell
- Only invoked at R1+ (not every turn)
- Results are structured: `pass | fail` + reason + citations

### 10.5 Thread Thesis Refinement Directive

Per `docs/38-thread-thesis-spec.md`: The Thread Thesis is a "living semantic anchor" that should be continuously refined. Currently the data structure exists but is never updated.

**Upgrade plan:**
- After every Nth turn (configurable, default N=3), make an internal LLM call:
  - Input: current thesis + last N turns + Focus State
  - Output: updated thesis (primary_intent, secondary_goals, constraints, open_questions, assumptions, confidence)
- Thesis updates are bounded: only changed fields are updated
- Thesis confidence increases with consistency, decreases with contradiction
- Budget: ≤400 tokens per refinement call
- Thesis is injected into Expression Engine prompt assembly (§11 Slot 3)

### 10.6 Reflection Loop Upgrade Directive

Per `docs/G1-14-reflection-loop.md`: The reflection loop runs (753 iterations logged) but currently uses heuristic scoring, not LLM-backed reasoning.

**Upgrade plan:**
- Replace heuristic reflection with LLM-backed review:
  - Input: recent events window + Focus State + autonomy metrics + thread thesis
  - Output: structured observations, risks, recommended_actions, confidence
- The reflection call is **not a response to the operator** — it is the system thinking about its own trajectory
- Budget: ≤800 tokens per reflection iteration
- Cadence: configurable (default: hourly when active, suppressed when idle)
- Stop conditions remain: low confidence, no evidence delta, repeated recommendations

### 10.7 Pre-Turn Context Enrichment (Replaces "Deliberation")

The previous turn's background processing already enriched Focus State with extracted decisions, refined thesis, promoted rules, and wiki/Mem0 context. The Expression Engine assembles from this pre-computed state in <20ms.

**However**, targeted retrieval for the NEW operator input can happen as part of the hot path IF it's fast enough:

**Implementation (fast path — no LLM call):**
1. Receive operator input
2. Keyword extraction from input (deterministic, <1ms)
3. Parallel async fetch (≤50ms total):
   - `wb wiki search "$KEYWORDS" --format json --limit 3`
   - `wb memory search "$KEYWORDS" --limit 5`
4. Merge results into Expression Engine assembly
5. If fetches timeout (≤50ms) → proceed without, use cached state

**Implementation (deep path — LLM call, async from PREVIOUS turn):**
1. After each turn completes, background worker generates:
   ```json
   {
     "anticipated_queries": ["query1", "query2"],
     "active_constraints": ["constraint1"],
     "confidence": 0.85,
     "risk_level": "low"
   }
   ```
2. These pre-computed queries are used for wiki/Mem0 prefetch
3. Results are cached in Focus State, ready for next turn's Expression Engine

**Result:** The operator never waits for deliberation. Fast keyword-based retrieval runs inline (≤50ms). Deep LLM-backed intent analysis runs in background from the previous turn, enriching context for the next turn.

**Budget:** 0 tokens on hot path. ≤300 tokens async (previous turn's background).

### 10.8 Post-Turn Evaluation Directive (NEW)

After the model responds and the response is **already sent to the operator**, evaluate quality asynchronously.

**R0 (normal) — response sent immediately, evaluation is background:**
1. Response sent to operator (no delay)
2. Async LLM call evaluates:
   ```json
   {
     "answers_question": true,
     "consistent_with_prior_decisions": true,
     "violates_constraints": [],
     "confidence": 0.9,
     "quality_notes": "Response is grounded"
   }
   ```
3. If constraint violations found → flag in Focus State for next turn
4. If quality low → note in Focus State: "Previous response may need correction"
5. Results feed into next turn's Expression Engine context

**R1+ (reliability mode) — evaluation MAY block before sending response:**
1. Model generates candidate response (not yet sent)
2. RFM microcell validators check in parallel (§10.4)
3. If validation passes → send response
4. If validation fails → regenerate once with additional constraints, then send
5. This is the **only metacognitive delay** the operator ever experiences

**Budget:** ≤300 tokens per evaluation (async at R0, inline at R1+)
**Model:** cheap/fast
**Triggering:** every turn at R1+; sampled at R0 (every 3rd turn)
**Fallback:** if evaluation fails, pass response through unchanged

### 10.9 Model Selection for Internal Calls

Not all LLM calls need the expensive primary model.

| Call Type | Model | Rationale |
|---|---|---|
| Operator-facing response | Primary (Kimi K2.5) | Quality matters most |
| Pre-turn deliberation | Cheap/fast (MiniMax M2.5) | Speed + cost |
| Post-turn evaluation | Cheap/fast | Speed + cost |
| Worker extraction | Cheap/fast | Volume, async |
| RFM microcells | Cheap/fast | Isolated, narrow |
| Thesis refinement | Cheap/fast | Periodic, bounded |
| Reflection loop | Cheap/fast or primary | Depth matters but infrequent |
| Contradiction scan | Cheap/fast | Batch, nightly |

**Cost control:** All internal calls have strict token budgets. Total internal overhead per turn should be ≤1500 tokens (~$0.001 at typical rates). This is negligible compared to the primary model call.

### 10.10 Thinking Budget Policy

The organism's intelligence comes at a cost. Budget policy:

| Cadence | Max Internal Tokens | Purpose |
|---|---|---|
| Per turn (R0) | 600 | Deliberation (300) + extraction (300) |
| Per turn (R1+) | 1500 | + evaluation (300) + microcells (600) |
| Per session | 400 | Thesis refinement |
| Hourly | 800 | Reflection loop |
| Nightly | 2000 | Contradiction scan + graph gap detection |

**Operator override:** `wb focusa thinking --budget high` to allow deeper internal reasoning for complex sessions.

---

## 12. Operational Flows

### 12.1 Turn Execution Flow (Updated with Metacognition)

**HOT PATH (≤20ms + model latency — operator never waits for metacognition):**
1. Operator/user message enters OpenClaw (or Pi with extension)
2. OpenClaw creates/continues session
3. User input signal emitted to Focus Gate (adapter contract requirement)
4. Fast keyword retrieval from wiki + Mem0 (≤50ms, parallel, no LLM call)
5. Focusa Expression Engine assembles prompt from **pre-enriched** Focus State + fresh retrieval + frame + rules + memories + thesis + operator state
6. **Context budget check:** assembly bounded by available headroom (Pi extension reports via `ctx.getContextUsage()`)
7. OpenClaw/Pi invokes primary model via Focusa proxy (or Pi extension injects via `context` event)
8. **Streaming chunks forwarded** via `/v1/turn/append` for real-time ASCC delta extraction
9. **Response returned to operator** (R0: immediate. R1+: after microcell validation)
10. **Error signals** (tool errors, model errors) fed to Focus Gate for pattern detection

**BACKGROUND (async, after response sent — system thinks about what just happened):**
7. **LLM-backed extraction** (parallel workers §10.3): decisions, failures, constraints, skills, memory candidates → Focus State delta
8. **Post-turn evaluation** (async LLM §10.8): quality + consistency check → notes for next turn
9. **Thread Thesis refinement** (every Nth turn §10.5): update semantic anchor
10. **Anticipatory queries** (§10.7): LLM generates predicted next-turn retrieval queries for wiki/Mem0 prefetch
11. Focusa emits: `turn_completed`, telemetry, worker results, autonomy observation
12. Promotion pipeline begins (§6)
13. **All background results land in Focus State** → ready for next turn's Expression Engine

**R1+ EXCEPTION (the only delay):**
At RFM level R1+, step 6 is preceded by parallel microcell validation (§10.4). If validators flag the response, one regeneration attempt occurs before sending. This is the spec-authorized tradeoff: correctness over speed for high-risk tasks.

### 12.2 Session Start Flow

1. Start/open Focusa session (`curl -X POST :8787/v1/session/start -d '{"adapter_id":"openclaw","workspace_id":"wirebot"}'`)
2. Resolve current Flow Mesh task or create focus frame mapping
3. Query Context Core for operator state (`GET :7400/v1/state`)
4. Query Mem0 for relevant memories (`wb memory search "$INTENT"`)
5. Query Wiki for project page, decisions, skills (`wb wiki search "tag:decision $PROJECT"`)
6. Build bounded session context package

### 12.3 Session Close Flow

1. Persist Focus State snapshot
2. Extract decisions / constraints / failures / next steps
3. Send candidate memories to promotion pipeline
4. Write session capture to wiki `ops/sessions/`
5. Reconcile task progress to Flow Mesh
6. Update scoreboards/metrics
7. Close Focusa session (`curl -X POST :8787/v1/session/close -d '{"reason":"session_ended"}'`)

---

## 13. Intelligence Growth Loops

### 12.1 Every Turn
- Focusa event capture
- Operator modulation applied
- Relevant wiki/memory retrieval
- Telemetry updates
- Candidate extraction scheduling

### 12.2 Every Session End
- Session capture generated
- Decision/failure extraction
- Memory candidate generation
- Task reconciliation
- Optional wiki draft creation

### 12.3 Nightly
- Wiki enrichment (`/data/wirebot/bin/wiki-enrich-nightly.sh`)
- wiki-agent graph maintenance (fill red links, audit staleness)
- Candidate memory dedupe
- Contradiction scan
- Kaizen extraction
- Stale page refresh queue generation
- Metric snapshot

### 12.4 Weekly
- Graph health report (`wb wiki stats`)
- Autonomy trend review
- Skill-gap report
- Orphan reduction review
- Project decision coverage review

### 12.5 Monthly
- Ontology drift audit
- Constitution review
- Promotion-rule tuning
- Archival pruning / graph compaction review

### 12.6 Daily Intelligence Metrics

```yaml
daily_metrics:
  wiki_pages_created: N
  wiki_links_created: N
  wiki_orphans_remaining: N
  wiki_knowledge_pages: N        # /notes/* only
  wiki_avg_links_per_page: N
  mem0_memories_added: N
  focusa_decisions_recorded: N
  focusa_ari_score: N
  focusa_rules_active: N
  kaizen_reflections: N
  wiki_agent_pages_fixed: N
```

---

## 14. Implementation Phases

### Phase 0: Revive Dead Systems (Day 1, ~30 min)

#### 0.1 Restart wiki-agent
```bash
systemctl start wiki-agent
systemctl enable wiki-agent
```

#### 0.2 Schedule nightly enrichment
```ini
# /etc/systemd/system/wiki-enrich.timer
[Unit]
Description=Wiki Enrichment Nightly
[Timer]
OnCalendar=*-*-* 03:00:00
Persistent=true
[Install]
WantedBy=timers.target
```
```ini
# /etc/systemd/system/wiki-enrich.service
[Unit]
Description=Wiki Enrichment Run
[Service]
Type=oneshot
ExecStart=/data/wirebot/bin/wiki-enrich-nightly.sh
TimeoutStartSec=3600
```
```bash
systemctl daemon-reload && systemctl enable --now wiki-enrich.timer
```

#### 0.3 Schedule vault→wiki sync (every 15 min)
```ini
# /etc/systemd/system/vault-wiki-sync.timer
[Unit]
Description=Vault to Wiki Sync
[Timer]
OnCalendar=*-*-* *:00,15,30,45:00
Persistent=true
[Install]
WantedBy=timers.target
```
```ini
# /etc/systemd/system/vault-wiki-sync.service
[Unit]
Description=Vault Wiki Sync Run
[Service]
Type=oneshot
ExecStart=/data/wirebot/bin/sync-vault-wiki.sh delta
TimeoutStartSec=300
```

#### 0.4 Auto-reload SOUL.md → Focusa constitution
```bash
# When SOUL.md changes, reload constitution into Focusa
wb soul reload
```
Hook into memory-syncd file watch or inotify post-commit.

**Phase 0 success:** wiki-agent active, enrichment nightly, vault sync every 15 min, SOUL.md auto-reloads.

---

### Phase 1: Wire Focusa Into the Loop (Week 1, ~5 hours)

#### 1.1 Route OpenClaw through Focusa proxy
```
OpenClaw → Focusa :8787/proxy/v1/chat/completions → Kimi/Qwen
```
In OpenClaw config: `OPENAI_BASE_URL=http://127.0.0.1:8787/proxy/v1`

**Fallback:** Health check Focusa before each call. If down → direct model passthrough.

#### 1.2 Session lifecycle calls
```bash
# Start
curl -X POST http://127.0.0.1:8787/v1/session/start \
  -d '{"adapter_id":"openclaw","workspace_id":"wirebot"}'

# Close
curl -X POST http://127.0.0.1:8787/v1/session/close \
  -d '{"reason":"session_ended"}'
```

#### 1.3 Context Core → Focusa modulation
On each turn, query Context Core and inject into Focus State constraints:
```bash
# GET http://127.0.0.1:7400/v1/state
# interruptibility: very_low → "Do not ask questions, queue them"
# circadian_phase: deep_night → "Operator may be sleeping"
```

#### 1.4 Focus Stack ↔ Flow Mesh bridge
```bash
# On focus push with beads_issue_id:
mesh_task=$(wb queue list --format json | jq ".[] | select(.title | contains(\"$BEADS_ISSUE\"))")

# On Flow Mesh task start:
curl -X POST http://127.0.0.1:8787/v1/focus/push \
  -d '{"title":"$TASK_TITLE","goal":"$TASK_GOAL","beads_issue_id":"$TASK_ID"}'
```

---

### Phase 2: Memory Integration (Week 2, ~8 hours)

#### 2.1 Session start → Mem0 seeding
```python
memories = mem0.search(query=current_focus_intent, user_id="wirebot_verious", limit=10)
for m in memories:
    focusa.semantic_memory.upsert(key=f"mem0.{m.id}", value=m.text, source="mem0")
```

#### 2.2 Session end → Mem0 writeback
```python
decisions = focusa.state.focus_state.decisions
for d in decisions:
    mem0.add(text=d, user_id="wirebot_verious",
             metadata={"source":"focusa","session_id":session_id})
```

#### 2.3 Session end → Wiki writeback
```bash
wb wiki create --title "Session $SESSION_ID Decisions" \
  --path "ops/sessions/$DATE" \
  --tags "session,focusa,decisions" <<EOF
# Session Decisions — $DATE
$(focusa state dump --json | jq -r '.focus_state.decisions[]')
## Context
- Frame: $FRAME_TITLE
- Goal: $FRAME_GOAL
EOF
```

#### 2.4 Kaizen → Wiki pipeline
```bash
# Nightly: extract high-value kaizen reflections
wb kaizen list --format json | jq '.[] | select(.similarity < 0.7)' | \
  while read reflection; do
    wb wiki update $PAGE_ID <<< "$reflection"
  done
```

#### 2.5 Procedural memory from wiki skills
```bash
# On session start, load relevant skills as procedural rules
wb wiki search "tag:skill $CURRENT_PROJECT" --format json | \
  while read skill; do
    curl -X POST http://127.0.0.1:8787/v1/memory/reinforce \
      -d "{\"rule_id\":\"wiki-skill-$SKILL_ID\",\"text\":\"$SKILL_HOW\"}"
  done
```

---

### Phase 3: Wiki as Active Knowledge Graph (Week 3, ~15 hours)

#### 3.1 OpenClaw tool: wiki_search
```json
{
    "name": "wiki_search",
    "description": "Search the knowledge wiki for prior decisions, skills, project context. Use before making decisions or starting new tasks.",
    "parameters": {
        "query": {"type": "string", "description": "Search query"}
    }
}
```
Implementation: calls `wb wiki search "$query" --format json`.

#### 3.2 OpenClaw tool: wiki_read
```json
{
    "name": "wiki_read",
    "description": "Read a specific wiki page by path for full context.",
    "parameters": {
        "path": {"type": "string", "description": "Wiki page path"}
    }
}
```

#### 3.3 OpenClaw tool: wiki_decide
```json
{
    "name": "wiki_decide",
    "description": "Record a decision in the wiki with rationale, alternatives, and project links.",
    "parameters": {
        "title": {"type": "string"},
        "rationale": {"type": "string"},
        "alternatives": {"type": "array", "items": {"type": "string"}},
        "related_project": {"type": "string"}
    }
}
```

#### 3.4 Focusa Expression Engine → Wiki context injection
When assembling prompts, Focusa should:
1. Get current frame's project tag
2. Query wiki: `wb wiki search "tag:decision $PROJECT" --format json`
3. Include top 3 prior decisions in prompt context
4. Query wiki: `wb wiki search "tag:skill $DOMAIN" --format json`
5. Include relevant skills as constraints

#### 3.5 Process orphan pages
1. Auto-categorize: joplin-import → archive (95%), ai-chats → selective reduce (50%)
2. Batch-reduce valuable ChatGPT conversations → atomic notes in `/notes/`
3. Link surviving notes into MOCs
4. Archive or delete the rest
5. Target: orphans < 100

---

### Phase 4: Ontology & Memory Convergence (Week 4, ~5 hours)

#### 4.1 Align wiki entities with ontology types
Ensure every entity in the ontology core set (§9.8) has a corresponding wiki page or explicit gap note.

#### 4.2 Use Kuzu relations to enrich retrieval
Query Mem0 graph for relational context:
- "What skills does this project require?"
- "What decisions constrain this task?"
- "What tools does this agent use?"

#### 4.3 Project graph slices into `wb ontology skills`
`wb ontology skills graph` should reflect live wiki + Mem0 state.

---

### Phase 5: Metacognitive Reasoning Activation (Week 4, ~12 hours)

#### 5.1 Upgrade workers from regex to LLM-backed
- Add HTTP client to workers for cheap model calls (MiniMax M2.5 at :8200 or local endpoint)
- Replace `classify_turn` regex with LLM intent classification
- Replace `extract_ascc_delta` regex with LLM structured extraction
- Replace `detect_repetition` with LLM semantic similarity
- Replace `scan_for_errors` with LLM error analysis
- Replace `suggest_memory` with LLM fact extraction
- Keep regex as fallback if LLM call fails
- Budget: ≤500 tokens per worker, ≤2s timeout

#### 5.2 Implement pre-turn deliberation
- Before Expression Engine assembly, call cheap model with operator input + Focus State + thesis
- Output: interpreted intent, targeted wiki/memory queries, confidence, risk level
- Feed deliberation results into Expression Engine
- Budget: ≤300 tokens

#### 5.3 Implement post-turn evaluation
- After model response, call cheap model with question + response + constraints + prior decisions
- Output: answers_question, consistent, constraint violations, should_regenerate
- At R1+: every turn. At R0: sampled (every Nth turn)
- Budget: ≤300 tokens

#### 5.4 Activate Thread Thesis refinement
- After every 3rd turn, call cheap model with current thesis + recent turns + Focus State
- Output: updated thesis fields (primary_intent, constraints, open_questions, confidence)
- Inject thesis into Expression Engine Slot 3
- Budget: ≤400 tokens

#### 5.5 Upgrade reflection loop to LLM-backed
- Replace heuristic scoring with LLM review of recent events + Focus State + thesis + autonomy
- Output: structured observations, risks, recommended_actions
- Budget: ≤800 tokens per iteration

#### 5.6 Implement RFM microcell validators
- Schema validator: LLM checks output structure
- Constraint validator: LLM checks against frame constraints
- Consistency validator: LLM checks against prior decisions
- Grounding validator: LLM checks claims against Reference Store
- Each microcell gets isolated context (NOT full session)
- Budget: ≤300 tokens per microcell
- Only invoked at R1+ risk level

---

### Phase 6: Continuous Intelligence (Week 5+, ongoing)

#### 6.1 Autonomy escalation gates
- AL0 → AL1: Focusa can auto-resume frames (30 days stable ARI > 70, operator approval)
- AL1 → AL2: Focusa can select next task from Flow Mesh (60 days ARI > 80)
- AL2 → AL3: Wirebot can create subtasks autonomously (90 days ARI > 85)

#### 6.2 SOUL.md ↔ Focusa constitution contract
- SOUL.md = human-authored master doctrine (operator-managed)
- Focusa constitution = compiled runtime projection
- `wb soul reload` = deterministic compile/projection step
- Conflicts: SOUL.md wins; Focusa constitution is derivative
- What gets projected: behavioral principles, safety rules, expression constraints
- What stays SOUL-only: pillar philosophy, operator relationship doctrine, mission

---

## 15. Implementation Priority Summary

| Priority | Task | Systems | Effort |
|----------|------|---------|--------|
| **P0** | Restart wiki-agent | Wiki.js | 5 min |
| **P0** | Schedule enrichment + vault sync timers | systemd | 15 min |
| **P0** | Auto-reload SOUL.md → Focusa | wb soul | 10 min |
| **P1** | Route OpenClaw through Focusa proxy | OpenClaw, Focusa | 2 hours |
| **P1** | Context Core → Focusa constraints | Context Core, Focusa | 1 hour |
| **P1** | Focus Stack ↔ Flow Mesh bridge | Focusa, Flow Mesh | 2 hours |
| **P2** | Mem0 → Focusa session seeding | Mem0, Focusa | 3 hours |
| **P2** | Session end → Mem0 + Wiki writeback | Focusa, Mem0, Wiki | 3 hours |
| **P2** | wiki_search / wiki_read / wiki_decide tools | OpenClaw, Wiki | 3 hours |
| **P3** | Focusa Expression Engine → Wiki injection | Focusa, Wiki | 4 hours |
| **P3** | Kaizen → Wiki pipeline | wb kaizen, Wiki | 2 hours |
| **P3** | Process 952 orphan pages | Wiki, LLM | 8 hours |
| **P3** | Daily intelligence metrics | wb CLI | 3 hours |
| **P3** | Ontology entity/relation alignment | Ontology, Wiki | 3 hours |
| **P3** | Promotion pipeline service | Focusa, Mem0, Wiki | 5 hours |
| **P2** | Upgrade workers: regex → LLM extraction | Focusa workers | 4 hours |
| **P2** | Pre-turn deliberation (internal LLM) | Focusa daemon | 3 hours |
| **P2** | Post-turn evaluation (internal LLM) | Focusa daemon | 3 hours |
| **P3** | Thread Thesis refinement (LLM-backed) | Focusa daemon | 2 hours |
| **P3** | Reflection loop upgrade (LLM-backed) | Focusa daemon | 2 hours |
| **P3** | RFM microcell validators (LLM-backed) | Focusa daemon | 4 hours |
| **P3** | Nightly contradiction scan (LLM-backed) | Focusa + Wiki + Mem0 | 3 hours |
| **P1** | Agent Audit: live Focusa panels (mobile) | Agent Audit, Focusa | 4 hours |
| **P0** | Upgrade MiniMax M2.5 → M2.7 | Mem0, Focusa workers | 1 hour |
| **P0** | Add all fallback timeouts (12 integration points) | All services | 3 hours |
| **P0** | Event log compaction (420MB → ~70MB) | Focusa SQLite | 2 hours |
| **P0** | Implement nightly hygiene loop (§10.8) | Focusa daemon | 3 hours |
| **P1** | Procedural rule removal threshold (weight < 0.01) | Focusa core | 1 hour |
| **P1** | Semantic memory TTL enforcement | Focusa core | 1 hour |
| **P2** | CLT compaction (>1000 nodes) | Focusa core | 3 hours |
| **P2** | Snapshot thinning (daily/weekly/monthly) | Focusa persistence | 2 hours |
| **P2** | Mem0 deduplication + staleness tracking | Mem0 + wb memory | 3 hours |
| **P1** | Verify OpenClaw fallback when Focusa proxy down | OpenClaw, Focusa | 1 hour |
| **P1** | Guardian alerts → Focusa Intuition Engine | Guardian, Focusa | 2 hours |
| **P0** | Activate idle kaizen/trust/memory/ships pipelines | Scoreboard, OpenClaw plugins | 4 hours |
| **P1** | Wire Pairing Engine drift/RABIT/DISC into Focusa | Scoreboard → Focusa | 4 hours |
| **P1** | Objective stall detection (weekly) | Reflection loop, objectives.yaml | 2 hours |
| **P2** | Self-initiated questions (nightly reflection) | Focusa reflection loop | 2 hours |
| **P2** | Cross-domain synthesis (weekly) | Context Core + Scoreboard + Wiki | 4 hours |
| **P1** | Build /wbm Pi extension (two-way bridge) | Pi extension, Mem0, Wiki, Scoreboard | 6 hours |
| **P1** | /wbm work cataloguing (LLM extraction) | Pi extension, MiniMax M2.7 | 4 hours |
| **P2** | Multi-agent learning pipeline | Session captures → wiki + Mem0 | 3 hours |
| **P2** | "Why" tracking in decisions | wiki_decide tool, memory extraction | 2 hours |
| **P3** | Preference evolution tracking | Mem0 versioning | 3 hours |
| **P3** | Confidence calibration | Focusa telemetry | 4 hours |

**Total estimated effort:** ~131 hours across 6 phases

---

## 16. Acceptance Criteria

The organism is working when:

1. Every Wirebot turn flows through Focusa
2. Focus State is always present and meaningful
3. Operator state affects turn behavior in measurable ways
4. Session captures are generated automatically
5. Durable decisions become wiki artifacts
6. Relevant memory is seeded from Mem0 into Focusa at session start
7. Wiki graph health improves over time (orphans decrease, links increase)
8. Autonomy score changes are evidence-backed
9. System continues operating under partial subsystem failure
10. No canonical truth concern is owned by more than one system
11. The system can explain why it knows something, where it came from, and why it acted
12. Weekly metrics show measurable intelligence growth
13. **Pre-turn deliberation runs on every turn** — the system thinks before responding
14. **Post-turn evaluation catches constraint violations** before they reach the operator
15. **Workers extract structured knowledge via LLM**, not regex keyword matching
16. **Thread Thesis is actively refined** and reflects the real meaning of the session
17. **Reflection loop produces LLM-backed observations** about work quality and trajectory
18. **RFM microcells validate high-risk output** with isolated sub-agent LLM calls
19. **Internal LLM calls are budgeted and auditable** — every metacognitive call is logged with token count
20. **Operator can see Focus Stack, ARI, gate candidates, and thesis from phone** via Agent Audit PWA
21. **All 12 integration points have explicit timeouts and fallbacks** — no integration failure blocks operator response
22. **Wirebot demonstrably gets smarter over time** — procedural rules accumulate, wiki grows, ARI trends up, recall improves
23. **Background inference uses MiniMax M2.7** — primary model untouched, internal calls use latest cheap model
24. **JARVIS 7-domain coverage verified** — every domain has a working organism subsystem mapped to it
25. **Event log size stabilizes** — not linear growth; stays bounded after compaction
26. **Procedural rule count converges** — low-weight rules removed, useful rules reinforced
27. **Wiki T1+T2 link density increases weekly** — active knowledge graph gets denser
29. **Wiki importance tiers assigned to all pages** — no untiered pages in active namespaces
28. **No data store grows without a retention policy** — every table has hot/warm/cold/delete tiers
29. **Pairing Engine drift/RABIT/emotion feeds into Focusa** — operator model shapes cognition
30. **Kaizen reflections are non-zero** — learning machinery produces actual output
31. **Trust metrics record corrections** — operator corrections become procedural rules
32. **Mem0 memories accumulate from sessions** — extraction pipeline is live
33. **Season score reflects reality** — ships auto-detected and logged
34. **System asks itself questions it can't answer** — self-initiated inquiry drives knowledge growth
35. **Cross-domain synthesis runs weekly** — RescueTime + calendar + objectives + ships connected
36. **Objective stalls are detected and surfaced** — P1 stall while P3 gets attention = alert
37. **Pi sessions with /wbm catalogue work back to Wirebot** — decisions, ships, facts, failures flow home
38. **Wirebot can recall what happened in Pi sessions** — "yesterday in Pi we decided X" works
