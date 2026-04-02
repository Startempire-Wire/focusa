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

---

## 10. Operational Flows

### 10.1 Turn Execution Flow

1. Operator/user message enters OpenClaw
2. OpenClaw creates/continues session
3. OpenClaw calls Focusa proxy for prompt assembly
4. Focusa loads: active Focus State, frame/task context, procedural rules, semantic memories, artifact handles, operator modulation from Context Core
5. OpenClaw invokes model via Focusa proxy
6. Response returns through Focusa
7. Focusa emits: `turn_completed`, telemetry, worker jobs, autonomy observation, RFM validators
8. Post-turn extraction pipeline begins

### 10.2 Session Start Flow

1. Start/open Focusa session (`curl -X POST :8787/v1/session/start -d '{"adapter_id":"openclaw","workspace_id":"wirebot"}'`)
2. Resolve current Flow Mesh task or create focus frame mapping
3. Query Context Core for operator state (`GET :7400/v1/state`)
4. Query Mem0 for relevant memories (`wb memory search "$INTENT"`)
5. Query Wiki for project page, decisions, skills (`wb wiki search "tag:decision $PROJECT"`)
6. Build bounded session context package

### 10.3 Session Close Flow

1. Persist Focus State snapshot
2. Extract decisions / constraints / failures / next steps
3. Send candidate memories to promotion pipeline
4. Write session capture to wiki `ops/sessions/`
5. Reconcile task progress to Flow Mesh
6. Update scoreboards/metrics
7. Close Focusa session (`curl -X POST :8787/v1/session/close -d '{"reason":"session_ended"}'`)

---

## 11. Intelligence Growth Loops

### 11.1 Every Turn
- Focusa event capture
- Operator modulation applied
- Relevant wiki/memory retrieval
- Telemetry updates
- Candidate extraction scheduling

### 11.2 Every Session End
- Session capture generated
- Decision/failure extraction
- Memory candidate generation
- Task reconciliation
- Optional wiki draft creation

### 11.3 Nightly
- Wiki enrichment (`/data/wirebot/bin/wiki-enrich-nightly.sh`)
- wiki-agent graph maintenance (fill red links, audit staleness)
- Candidate memory dedupe
- Contradiction scan
- Kaizen extraction
- Stale page refresh queue generation
- Metric snapshot

### 11.4 Weekly
- Graph health report (`wb wiki stats`)
- Autonomy trend review
- Skill-gap report
- Orphan reduction review
- Project decision coverage review

### 11.5 Monthly
- Ontology drift audit
- Constitution review
- Promotion-rule tuning
- Archival pruning / graph compaction review

### 11.6 Daily Intelligence Metrics

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

## 12. Implementation Phases

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

### Phase 5: Continuous Intelligence (Week 4+, ongoing)

#### 5.1 Autonomy escalation gates
- AL0 → AL1: Focusa can auto-resume frames (30 days stable ARI > 70, operator approval)
- AL1 → AL2: Focusa can select next task from Flow Mesh (60 days ARI > 80)
- AL2 → AL3: Wirebot can create subtasks autonomously (90 days ARI > 85)

#### 5.2 SOUL.md ↔ Focusa constitution contract
- SOUL.md = human-authored master doctrine (operator-managed)
- Focusa constitution = compiled runtime projection
- `wb soul reload` = deterministic compile/projection step
- Conflicts: SOUL.md wins; Focusa constitution is derivative
- What gets projected: behavioral principles, safety rules, expression constraints
- What stays SOUL-only: pillar philosophy, operator relationship doctrine, mission

---

## 13. Implementation Priority Summary

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

**Total estimated effort:** ~45 hours across 5 phases

---

## 14. Acceptance Criteria

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
