# GPT5.4 Unified Intelligence Integration Spec

**Author:** GPT5.4  
**Status:** Proposed architecture spec  
**Scope:** Wirebot as a single adaptive organism across OpenClaw, wb CLI, Context Core, Focusa, Wiki.js, Obsidian vault, Mem0, Letta, Flow Mesh, ontology, memory-syncd, Scoreboard, Guardian, UIAI Engine  
**Constraint:** Separate spec. Does **not** modify or supersede Opus document; intended as an independently reasoned design.

---

## 1. Purpose

Build a system where:

1. **Every turn is observed**
2. **Important meaning survives**
3. **Knowledge compounds daily**
4. **Agent behavior improves from evidence, not vibes**
5. **Each subsystem has one clear job**
6. **Failure in one subsystem degrades gracefully, not catastrophically**

The target is not “many connected tools.”  
The target is **one operating organism** with:
- sensory surfaces
- working memory
- long-term memory
- knowledge graph
- active cognition
- task discipline
- self-observation
- feedback loops
- constrained self-improvement

---

## 2. Canonical Design Principle

> **Live cognition, durable knowledge, retrievable memory, and task execution must be separated—but continuously reconciled.**

This prevents the current failure mode where everything partially remembers the same thing and nothing cleanly owns truth.

---

## 3. System Roles — Hard Boundaries

## 3.1 Canonical authority matrix

| Concern | Canonical system | Notes |
|---|---|---|
| Live intent / current meaning | **Focusa** | Focus State + Focus Stack |
| Prompt shaping / context injection | **Focusa** | deterministic expression layer |
| Durable reviewed knowledge | **Wiki.js / Obsidian vault** | human-readable, linked, curated |
| Retrievable long-term memory | **Mem0** | vector + graph recall |
| Agent-local continuity / persona blocks | **Letta** | local stateful continuity |
| Task/work state | **Flow Mesh** | queue, status, backlog, dependencies |
| Operator state | **Context Core** | interruptibility, circadian, workload |
| Agent doctrine / behavioral constitution | **SOUL.md** | human-authored master doctrine |
| Runtime projected constitution | **Focusa Constitution** | compiled subset of SOUL |
| Orchestration surface | **wb CLI** | agent-facing facade |
| Infrastructure health | **Guardian + wb health** | service truth |
| UI/browser perception | **UIAI Engine** | screenshots, browser state |
| Audit trail | **Focusa event log + OpenClaw audit + Flow Mesh history** | append-first provenance |

## 3.2 Non-authority rules

- **Mem0 is not canonical policy**
- **Letta is not canonical project truth**
- **Wiki is not live cognition**
- **Focusa is not durable knowledge graph**
- **Flow Mesh is not semantic memory**
- **wb is not a storage system**

---

## 4. Core Architecture

```text
Human / Operator
   ↓
Context Core ───────────────┐
   ↓                        │
OpenClaw / Wirebot runtime  │
   ↓                        │
Focusa  ← live cognition spine
  ↓  ↓  ↓
Wiki  Mem0  Letta
  ↓    ↓      ↓
Knowledge  Recall  Local continuity
  \    |     /
     wb CLI facade
         ↓
   Flow Mesh / Scoreboard / Guardian / UIAI
```

### Interpretation
- **OpenClaw** executes the live agent turn
- **Focusa** observes and shapes cognition in-flight
- **Wiki** stores durable, linked, reviewable truth
- **Mem0** stores retrievable memories and associations
- **Letta** stores agent-local continuity and narrative state
- **wb** is the default control plane
- **Flow Mesh** holds the work graph
- **Context Core** modulates behavior according to real operator state

---

## 5. Data Classes

All information in the organism must fall into one of these classes.

## 5.1 Event
Atomic observed fact.
Examples:
- turn_started
- turn_completed
- decision_recorded
- task_started
- focus_changed
- memory_candidate_extracted

**Storage:** Focusa event log / audit systems  
**Properties:** append-only, timestamped, provenance-preserving

## 5.2 State
Current condition of a subsystem.
Examples:
- active frame
- operator interruptibility
- current ARI
- Flow Mesh task status

**Storage:** subsystem-owned, mutable, authoritative only within owner boundary

## 5.3 Memory
Retrievable learned fact/pattern.
Examples:
- “operator prefers telegraph mode”
- “posthog bind mounts beat named volumes”
- “Startempire Wire project uses X constraint”

**Storage:** Mem0 primary; Focusa bounded semantic/procedural subsets as runtime projections

## 5.4 Knowledge
Durable, linked, reviewed, human-readable understanding.
Examples:
- decision pages
- skill pages
- project pages
- ontology-aligned concept notes

**Storage:** Wiki.js / Obsidian vault

## 5.5 Doctrine
Behavioral principles governing the agent.
Examples:
- SOUL.md
- Focusa constitution projection

**Storage:** SOUL.md canonical; Focusa constitution runtime derivative

---

## 6. Event Spine

The organism requires a unified event contract.

## 6.1 Required event families

### Session lifecycle
- `session_started`
- `session_closed`
- `thread_attached`
- `thread_detached`

### Turn lifecycle
- `turn_started`
- `prompt_assembled`
- `turn_completed`
- `turn_failed`

### Cognition
- `focus_frame_pushed`
- `focus_frame_resumed`
- `focus_frame_completed`
- `focus_state_updated`
- `candidate_surfaced`
- `candidate_suppressed`
- `candidate_pinned`

### Memory
- `semantic_memory_upserted`
- `procedural_rule_reinforced`
- `memory_candidate_extracted`
- `memory_promoted`
- `memory_decay_tick`

### Knowledge graph
- `wiki_page_created`
- `wiki_page_updated`
- `decision_recorded`
- `skill_added`
- `link_created`
- `graph_gap_detected`

### Task/work
- `task_selected`
- `task_started`
- `task_blocked`
- `task_completed`

### Operator modulation
- `operator_state_changed`
- `interruptibility_changed`
- `circadian_phase_changed`

### Governance
- `constitution_reloaded`
- `autonomy_score_changed`
- `autonomy_level_changed`
- `rfm_regeneration_triggered`

## 6.2 Event transport rule

- Services may emit locally in their own storage
- **Integration-grade events** must be bridgeable to a shared stream via `wb` or direct API adapters
- No subsystem may silently mutate another subsystem’s canonical state without an auditable event

---

## 7. Operational Flows

## 7.1 Turn execution flow

1. Operator/user message enters OpenClaw
2. OpenClaw creates/continues session
3. OpenClaw calls Focusa proxy for prompt assembly
4. Focusa loads:
   - active Focus State
   - active frame/task context
   - relevant procedural rules
   - selected semantic memories
   - bounded artifact handles
   - operator modulation from Context Core
5. OpenClaw invokes model
6. Response returns through Focusa
7. Focusa emits:
   - turn_completed
   - telemetry updates
   - worker jobs
   - autonomy observation
   - RFM validators
8. Post-turn extraction pipeline begins

## 7.2 Session start flow

On session start:
1. Start/open Focusa session
2. Resolve current Flow Mesh task or create focus frame mapping
3. Query Context Core for operator state
4. Query Mem0 for relevant memory candidates
5. Query Wiki for:
   - project page
   - active decision pages
   - relevant skill pages
6. Build bounded session context package

## 7.3 Session close flow

On session close:
1. Persist Focus State snapshot
2. Extract decisions / constraints / failures / next steps
3. Send candidate memories to promotion pipeline
4. Write session capture to wiki `ops/sessions/`
5. Reconcile task progress to Flow Mesh
6. Update scoreboards/metrics

---

## 8. Promotion Pipeline

No raw model output should go directly into durable knowledge.

## 8.1 Pipeline stages

### Stage 1 — Observe
Input:
- events
- turns
- session captures
- command traces
- kaizen reflections
- workspace changes

### Stage 2 — Extract
Generate candidates:
- memory candidates
- decision candidates
- skill candidates
- ontology relation candidates
- project updates

### Stage 3 — Validate
Each candidate checked for:
- provenance
- novelty
- contradiction
- confidence
- schema compliance
- duplication
- scope appropriateness

### Stage 4 — Promote
Promotion target depends on class:
- Mem0 → recall memory
- Wiki → durable knowledge
- Focusa procedural memory → bounded active rule set
- Letta → agent-local continuity block

### Stage 5 — Audit
Every promotion must record:
- source event/session
- reason
- confidence
- reviewer mode (automatic / threshold / human)

## 8.2 Write-trust levels

| Class | Allowed write mode |
|---|---|
| telemetry / traces | automatic |
| session captures | automatic |
| Mem0 candidate memory | automatic after validation |
| Focusa semantic memory seeding | automatic bounded projection |
| procedural rule promotion | thresholded or operator-approved |
| wiki durable page creation | thresholded + schema validated |
| constitution changes | human-approved only |
| ontology schema changes | human-approved only |

This follows Focusa memory trust constraints: workers may suggest, not silently redefine agent behavior.

---

## 9. Focusa Integration Rules

## 9.1 Focusa must remain backend-agnostic
Per Focusa architecture docs:
- no dependence on Letta internals
- wrap via generic stdin/stdout or HTTP
- do not turn Focusa into app-specific business logic

## 9.2 Focusa owns live cognition only
Focusa must own:
- active frame
- current intent
- current constraints
- current decisions-in-session
- active artifact references
- autonomy status

Focusa must **not** become:
- primary wiki
- primary vector DB
- canonical task DB

## 9.3 Focusa memory policy

### Semantic memory inside Focusa
Only bounded, whitelisted, active-use memories:
- operator response style
- project constraints
- current active project anchors
- high-confidence active preferences

### Procedural memory inside Focusa
Only stable behavioral rules with:
- explicit provenance
- reinforcement history
- decay
- scope

**No single event may become a global procedural rule.**

## 9.4 Focusa skill surface rule
Agents interact with Focusa via:
- inspection
- explanation
- proposals

Never direct mutation outside sanctioned commands/reducers.

---

## 10. Wiki / Vault Integration Rules

## 10.1 Wiki is the reviewed knowledge graph
The Wiki/Vault layer must contain:
- project pages
- decision pages
- skill pages
- concept pages
- MOCs
- session captures
- handoffs
- inbox items pending processing

## 10.2 Mandatory graph quality rules
Every durable knowledge page must satisfy:
- at least one inbound or planned MOC reference
- at least two outbound semantic links where possible
- schema-valid metadata
- provenance note if machine-generated

## 10.3 Graph health KPIs
Track daily/weekly:
- orphan ratio
- stale critical page ratio
- average inbound links / knowledge page
- average outbound links / knowledge page
- MOC coverage by active project
- skill coverage by active project
- decision coverage by active project
- unresolved red link count

## 10.4 Page classes

### Durable classes
- `/notes/projects/*`
- `/notes/skills/*`
- `/notes/concepts/*`
- `/notes/decisions/*`
- `/notes/_mocs/*`

### Operational classes
- `/ops/sessions/*`
- `/ops/handoffs/*`
- `/ops/inbox/*`
- `/ops/journal/*`

### Archive classes
- imports, raw chats, joplin dump, raw source material

## 10.5 Raw imports are not graph health
Imported archives do not count as graph maturity.
Only linked, active knowledge pages count.

---

## 11. Mem0 Integration Rules

## 11.1 Mem0 is recall memory, not policy authority
Store:
- facts
- preferences
- patterns
- extracted learnings
- relational memory candidates

Do not store as canonical truth:
- active task state
- final policy
- reviewed architecture decisions

## 11.2 Session-start memory seeding
At session/frame start:
1. derive retrieval query from Focus State intent + task + project
2. search Mem0
3. validate top-N memories
4. project only bounded, relevant items into Focusa semantic memory

## 11.3 Session-end writeback
At session close:
- push candidate facts and learnings to Mem0 only after extraction/validation
- attach metadata:
  - source session
  - source frame
  - originating project
  - confidence
  - promotion level

## 11.4 Kuzu graph role
Use graph store for:
- relation recall
- entity linkage
- project-skill-decision mappings
- operator/project/tool relationships

Do not use Kuzu as a substitute wiki.

---

## 12. Letta Integration Rules

## 12.1 Letta stores local continuity
Letta is appropriate for:
- agent-local narrative continuity
- persona blocks
- short/medium horizon statefulness
- contextual continuity between conversations

## 12.2 Letta must not become the hidden durable truth source
If Letta learns something durable:
- promote to Mem0 and/or Wiki
- do not leave critical project truth trapped in Letta memory blocks

## 12.3 Focusa x Letta relationship
If Letta is wrapped by Focusa:
- Focusa injects runtime cognition into prompt layer
- Letta remains harness/runtime, not cognition owner

---

## 13. Context Core Integration Rules

## 13.1 Operator state must modulate cognition every turn
Context Core data should influence:
- verbosity
- ask-vs-act threshold
- interruption behavior
- autonomy aggressiveness
- timing of risky operations

## 13.2 Required operator fields
Minimum fields:
- interruptibility
- confidence / cognitive load signal
- circadian phase / local time posture
- active operator focus mode
- recent fatigue / overload signals if available

## 13.3 Mapping examples
- `interruptibility=very_low` → queue questions, avoid interruptions
- `late-night / low-bandwidth` → concise, low-churn output
- `operator overloaded` → reduce branchiness, emphasize execution

Context Core should not directly overwrite Focus State; it should modulate it.

---

## 14. Flow Mesh Integration Rules

## 14.1 Flow Mesh is canonical work graph
It owns:
- task status
- backlog
- dependencies
- queue order
- completion state

## 14.2 Focusa <-> Flow Mesh bridge
Each active focus frame should ideally map to:
- Flow Mesh task ID or
- explicit no-task reason

### Required mapping fields
- `task_id`
- `project_id`
- `frame_id`
- `session_id`

## 14.3 Rules
- task selection may influence frame creation
- frame changes may update task state proposals
- task completion does not silently close focus frame; reducer/event path required

---

## 15. wb CLI Integration Rule

## 15.1 wb is the agent-facing facade
Agents should prefer `wb` for cross-system interactions because it already consolidates:
- wiki
- focusa
- memory
- ontology
- queue
- soul
- health
- session
- kaizen

## 15.2 Direct API usage rule
Direct service-to-service APIs are allowed when:
- latency matters
- no `wb` wrapper exists
- internal daemon integration is cleaner

But agent-facing workflows should default to `wb`.

---

## 16. Ontology Integration Rules

## 16.1 Ontology purpose
Ontology exists to define:
- entity types
- relation types
- action contracts
- policy checks
- skill graph projections

## 16.2 Ontology is not another wiki
Do not use ontology as freeform note storage.

## 16.3 Core entity set
- Agent
- Role
- Operator
- Project
- Task
- Skill
- Tool
- Decision
- Constraint
- Memory
- Session
- Thread
- Artifact
- Objective
- Policy

## 16.4 Core relation set
- `agent_has_role`
- `agent_has_skill`
- `project_requires_skill`
- `task_advances_project`
- `task_advances_objective`
- `decision_applies_to_project`
- `decision_constrains_task`
- `memory_supports_decision`
- `artifact_evidences_decision`
- `session_updates_project`
- `operator_state_modulates_agent`

---

## 17. Degraded Mode Matrix

| Failure | Behavior |
|---|---|
| Focusa down | OpenClaw direct passthrough; log cognition deficit |
| Wiki down | use Mem0 + workspace + Letta; defer durable writes |
| Mem0 down | use Wiki + Focusa bounded memory + Letta |
| Letta down | continue with Wiki + Mem0 + Focusa |
| Context Core down | use last-known operator state with TTL |
| Flow Mesh down | maintain temporary local work shadow; reconcile later |
| Wiki-agent down | continue manual/wiki sync paths; flag graph health degradation |
| sync timers down | system continues, but mark knowledge growth degraded |

No subsystem failure should fully halt agent execution unless safety requires it.

---

## 18. Daily / Weekly / Monthly Loops

## 18.1 Every turn
- Focusa capture
- operator modulation
- relevant wiki/memory retrieval
- telemetry updates
- candidate extraction scheduling

## 18.2 Every session end
- session capture
- decision extraction
- failure extraction
- memory candidate generation
- task reconciliation
- optional wiki draft creation

## 18.3 Nightly
- wiki enrichment
- wiki-agent graph maintenance
- candidate memory dedupe
- contradiction scan
- kaizen extraction
- stale page refresh queue generation
- metric snapshot

## 18.4 Weekly
- graph health report
- autonomy trend review
- skill-gap report
- orphan reduction review
- project decision coverage review

## 18.5 Monthly
- ontology drift audit
- constitution review
- promotion-rule tuning
- archival pruning / graph compaction review

---

## 19. Contradiction Resolution Policy

When systems disagree, precedence is:

1. direct operator instruction
2. active safety/constitution doctrine
3. current Focus State constraints
4. reviewed wiki decision pages
5. current Context Core operator state
6. validated Mem0 recall
7. Letta local continuity memory
8. raw extraction candidates

Contradictions must be logged, not silently resolved.

---

## 20. Implementation Plan

## Phase A — Reactivate dormant intelligence (1 day)
1. enable `wiki-agent`
2. add timers for enrichment + vault/wiki sync
3. automate `wb soul reload` on doctrine changes
4. verify all daily loops emit health signals

## Phase B — Make Focusa the active live spine (2–4 days)
1. route OpenClaw through Focusa proxy
2. enforce session/turn lifecycle calls
3. integrate Context Core modulation
4. map focus frames to Flow Mesh tasks

## Phase C — Build the promotion pipeline (3–5 days)
1. candidate extraction service
2. validation stage
3. Mem0 writeback
4. wiki session capture + decision draft creation
5. kaizen promotion path

## Phase D — Turn wiki into real graph (1–2 weeks)
1. reduce orphan count
2. enforce schemas
3. expand MOCs
4. link active project/skill/decision ecosystem
5. convert imported archive into curated graph or archive it hard

## Phase E — Ontology and memory convergence (1 week)
1. align wiki entities with ontology types
2. use Kuzu relations to enrich retrieval
3. project skill/decision/task graph slices into `wb ontology skills`

---

## 21. Acceptance Criteria

The organism is working when:

1. every Wirebot turn passes through Focusa
2. Focus State is always present and meaningful
3. operator state affects turn behavior in measurable ways
4. session captures are generated automatically
5. durable decisions become wiki artifacts
6. relevant memory is seeded from Mem0 into Focusa at start
7. wiki graph health improves over time
8. orphan ratio drops steadily
9. autonomy score changes are evidence-backed
10. system continues operating under partial subsystem failure
11. no canonical truth concern is owned by more than one system
12. the system can explain why it knows something, where it came from, and why it acted

---

## 22. Final Design Summary

This organism should behave like this:

- **OpenClaw acts**
- **Focusa thinks about the acting**
- **Wiki remembers what should stay true**
- **Mem0 recalls what might matter again**
- **Letta carries local continuity**
- **Context Core reflects the human reality**
- **Flow Mesh disciplines the work**
- **wb ties the surfaces together**
- **Ontology prevents conceptual drift**

That is the intended single-organism architecture.
