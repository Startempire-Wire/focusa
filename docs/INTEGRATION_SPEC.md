# Wirebot Unified Intelligence System — Integration Spec

**Status:** SPEC (ready for implementation)
**Goal:** All systems function as a single organism that grows smarter daily
**Author:** Claude Code + Operator
**Date:** 2026-04-02

---

## Current State Audit

### All Systems

| System | Port | Status | Connected To |
|--------|------|--------|--------------|
| **OpenClaw Gateway** | :18789 | ✅ Running | Wirebot agent runtime |
| **Context Core** | :7400 | ✅ Running | Operator state, session tracking |
| **Scoreboard** | :8100 | ✅ Running | OpenClaw, Context Core |
| **Focusa** | :8787 | ✅ Running | **NOTHING** (isolated) |
| **Mem0** | :8200 | ✅ Running | memory-syncd only |
| **Letta** | :8283 | ✅ Running | memory-syncd only |
| **memory-syncd** | :8201 | ✅ Running | Mem0 ↔ Letta ↔ workspace |
| **Wiki.js** | :7325 | ✅ Running | Obsidian vault (one-way sync) |
| **wiki-agent** | — | ❌ STOPPED | Was: Wiki.js maintenance |
| **Flow Mesh** | — | ✅ Running | Task queue (100 backlog items) |
| **UIAI Engine** | :7456 | ✅ Running | Vision/screenshots |
| **ntfy** | :2586 | ✅ Running | Push notifications |
| **Guardian** | — | ✅ Running | Server health monitoring |
| **wb CLI** | — | ✅ 45+ commands | Orchestration layer |

### Connection Map (Current Reality)

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

### What's BROKEN (Disconnections)

| # | Gap | Impact |
|---|-----|--------|
| 1 | **Focusa is isolated** — no agent flows through it | Meta-cognition exists but isn't used |
| 2 | **Wiki not queried during reasoning** — Wirebot doesn't `wb wiki search` during turns | Prior decisions/skills invisible |
| 3 | **Mem0 not connected to Focusa** — no memory seeding on session start | Focusa can't remember across sessions |
| 4 | **wiki-agent is STOPPED** — no autonomous graph maintenance | Wiki decays, red links accumulate |
| 5 | **Nightly enrichment not scheduled** — no timer/cron | Knowledge doesn't grow automatically |
| 6 | **952 orphan wiki pages** — 90% of wiki is dead weight | Graph traversal impossible |
| 7 | **Engineering agents don't use wiki** — Claude/Pi don't read decisions/skills | Repeated mistakes |
| 8 | **Session learnings don't flow to wiki** — decisions made in sessions vanish | Knowledge doesn't accumulate |
| 9 | **Focusa doesn't read constitution from SOUL.md** — `wb soul reload` exists but isn't automated | Constitution drift |
| 10 | **Context Core not connected to Focusa** — operator state invisible to cognitive runtime | No circadian/interruptibility awareness |
| 11 | **Flow Mesh not connected to Focusa Focus Stack** — tasks and focus are separate systems | Work tracking fragmented |
| 12 | **Kaizen reflections not feeding wiki** — learnings stay in JSONL, don't become knowledge | Lessons don't persist |

---

## Target State

```
                    ┌──────────────┐
                    │   OPERATOR   │
                    │  (Obsidian)  │
                    └──────┬───────┘
                           │ git sync
                    ┌──────▼───────┐     sync-vault-wiki
                    │   Obsidian   ├──────────────────────┐
                    │    Vault     │                       │
                    └──────────────┘                       │
                                                   ┌──────▼───────┐
    ┌──────────┐    ┌──────────────┐    wb wiki    │   Wiki.js    │
    │ Context  ├────┤   OpenClaw   ├──────search───►│  Knowledge   │
    │   Core   │    │   Gateway    │◄──────read────┤    Graph     │
    └────┬─────┘    └──────┬───────┘               └──────┬───────┘
         │                 │ LLM calls                     │
         │          ┌──────▼───────┐     wiki read         │
         │          │   Wirebot    ├◄──────────────────────┘
         │          │   (Agent)    │
         │          └──────┬───────┘
         │                 │
         │          ┌──────▼───────┐    ┌──────────────┐
         │          │  memory-     ├────┤   Focusa     │
         └──────────┤   syncd      │    │  Cognitive   │
                    └───┬──────┬───┘    │   Runtime    │
                        │      │        └───┬──────┬───┘
                 ┌──────▼┐  ┌──▼──────┐     │      │
                 │ Mem0  ├──┤  Letta  │     │      │
                 │(8200) │  │ (8283)  │◄────┘      │
                 └───────┘  └─────────┘            │
                                            ┌──────▼───────┐
                                            │  wiki-agent  │
                                            │ (autonomous  │
                                            │  maintenance)│
                                            └──────────────┘
```

---

## Integration Phases

### Phase 0: Revive Dead Systems (Day 1)

**Goal:** Get stopped systems running, schedule what should be scheduled.

#### 0.1 Restart wiki-agent
```bash
systemctl start wiki-agent
systemctl enable wiki-agent
```
- Fills red links from source data
- Audits staleness
- Maintains graph integrity
- **Verify:** `systemctl is-active wiki-agent` → `active`

#### 0.2 Schedule nightly enrichment
Create `/etc/systemd/system/wiki-enrich.timer`:
```ini
[Unit]
Description=Wiki Enrichment Nightly

[Timer]
OnCalendar=*-*-* 03:00:00
Persistent=true

[Install]
WantedBy=timers.target
```
Create `/etc/systemd/system/wiki-enrich.service`:
```ini
[Unit]
Description=Wiki Enrichment Run

[Service]
Type=oneshot
ExecStart=/data/wirebot/bin/wiki-enrich-nightly.sh
TimeoutStartSec=3600
```
```bash
systemctl daemon-reload
systemctl enable --now wiki-enrich.timer
```

#### 0.3 Schedule vault→wiki sync
Create `/etc/systemd/system/vault-wiki-sync.timer`:
```ini
[Unit]
Description=Vault to Wiki Sync

[Timer]
OnCalendar=*-*-* *:00,15,30,45:00
Persistent=true

[Install]
WantedBy=timers.target
```
Create `/etc/systemd/system/vault-wiki-sync.service`:
```ini
[Unit]
Description=Vault Wiki Sync Run

[Service]
Type=oneshot
ExecStart=/data/wirebot/bin/sync-vault-wiki.sh delta
TimeoutStartSec=300
```

#### 0.4 Auto-reload SOUL.md → Focusa constitution
Add to memory-syncd or as post-commit hook:
```bash
# When SOUL.md changes, reload constitution into Focusa
wb soul reload
```

**Phase 0 success criteria:** wiki-agent active, enrichment nightly, vault sync every 15 min, SOUL.md auto-reloads.

---

### Phase 1: Wire Focusa Into the Loop (Week 1)

**Goal:** Focusa observes every Wirebot turn. Focus State persists.

#### 1.1 Focusa proxy mode for OpenClaw

OpenClaw's LLM calls should route through Focusa's proxy:
```
OpenClaw → Focusa :8787/proxy/v1/chat/completions → Kimi/Qwen
```

**Implementation:**
- In OpenClaw's config, set `OPENAI_BASE_URL=http://127.0.0.1:8787/proxy/v1`
- Focusa intercepts every request, injects Focus State, records turn
- Focusa passes through to the actual model endpoint
- On response: Focusa records TurnCompleted, runs workers, updates ASCC

**Fallback:** If Focusa is down, OpenClaw should fall back to direct model calls. Add health check + fallback in gateway config.

#### 1.2 Session start → Focusa session start

When OpenClaw starts a conversation:
```bash
curl -X POST http://127.0.0.1:8787/v1/session/start \
  -d '{"adapter_id": "openclaw", "workspace_id": "wirebot"}'
```

When conversation ends:
```bash
curl -X POST http://127.0.0.1:8787/v1/session/close \
  -d '{"reason": "session_ended"}'
```

#### 1.3 Context Core → Focusa awareness

Focusa should know operator state (interruptibility, circadian phase):
- On each turn, query Context Core: `GET http://127.0.0.1:7400/v1/state`
- Inject into Focus State as constraints:
  - `interruptibility: very_low` → constraint: "Do not ask questions, queue them"
  - `circadian_phase: deep_night` → constraint: "Operator may be sleeping"

#### 1.4 Focus Stack ↔ Flow Mesh bridge

When Focusa pushes a focus frame, check if a Flow Mesh task exists:
```bash
# On focus push with beads_issue_id:
mesh_task=$(wb queue list --format json | jq ".[] | select(.title | contains(\"$BEADS_ISSUE\"))")
```

When a Flow Mesh task is started:
```bash
curl -X POST http://127.0.0.1:8787/v1/focus/push \
  -d '{"title": "$TASK_TITLE", "goal": "$TASK_GOAL", "beads_issue_id": "$TASK_ID"}'
```

**Phase 1 success criteria:** Every Wirebot turn flows through Focusa. Focus State updates. Operator state visible. Tasks linked to focus frames.

---

### Phase 2: Memory Integration (Week 2)

**Goal:** Mem0, Focusa, Wiki, and Letta share a unified memory layer.

#### 2.1 Session start → Mem0 seeding

When Focusa starts a session, query Mem0 for relevant context:
```python
# Pseudo-code for Focusa adapter
memories = mem0.search(
    query=current_focus_intent,
    user_id="wirebot_verious",
    limit=10
)
for m in memories:
    focusa.semantic_memory.upsert(
        key=f"mem0.{m.id}",
        value=m.text,
        source="mem0"
    )
```

**Implementation:** Add a Mem0 client to Focusa's daemon startup (HTTP calls to :8200).

#### 2.2 Session end → Mem0 writeback

When Focusa closes a session, extract decisions and learnings:
```python
# On session close
decisions = focusa.state.focus_state.decisions
for d in decisions:
    mem0.add(
        text=d,
        user_id="wirebot_verious",
        metadata={"source": "focusa", "session_id": session_id}
    )
```

#### 2.3 Session end → Wiki writeback

Decisions and learnings should flow to the wiki:
```bash
# On session close, if significant decisions were made:
wb wiki create --title "Session $SESSION_ID Decisions" \
  --path "ops/sessions/$DATE" \
  --tags "session,focusa,decisions" <<EOF
# Session Decisions — $DATE

$(focusa state dump --json | jq -r '.focus_state.decisions[]')

## Context
- Frame: $FRAME_TITLE
- Goal: $FRAME_GOAL
- Duration: $DURATION
EOF
```

#### 2.4 Kaizen → Wiki pipeline

Kaizen reflections should become wiki knowledge:
```bash
# Nightly: extract high-value kaizen reflections
wb kaizen list --format json | jq '.[] | select(.similarity < 0.7)' | while read reflection; do
    # Create or update relevant wiki page
    wb wiki update $PAGE_ID <<< "$reflection"
done
```

#### 2.5 Procedural memory from wiki skills

Focusa's procedural memory should be seeded from wiki skill pages:
```bash
# On session start, load relevant skills as rules
wb wiki search "tag:skill $CURRENT_PROJECT" --format json | while read skill; do
    curl -X POST http://127.0.0.1:8787/v1/memory/reinforce \
      -d "{\"rule_id\": \"wiki-skill-$SKILL_ID\", \"text\": \"$SKILL_HOW\"}"
done
```

**Phase 2 success criteria:** Mem0 seeds Focusa on start. Decisions flow to Mem0 + wiki on end. Skills become procedural rules. Kaizen becomes wiki knowledge.

---

### Phase 3: Wiki as Active Knowledge Graph (Week 3)

**Goal:** Wiki becomes queryable intelligence, not passive storage.

#### 3.1 OpenClaw tool: wiki_search

Add a tool to Wirebot's OpenClaw toolset:
```json
{
    "name": "wiki_search",
    "description": "Search the knowledge wiki for prior decisions, skills, project context, and operational knowledge. Use before making decisions or starting new tasks.",
    "parameters": {
        "query": {"type": "string", "description": "Search query"}
    }
}
```

**Implementation:** Tool calls `wb wiki search "$query" --format json` and returns results.

#### 3.2 OpenClaw tool: wiki_read

```json
{
    "name": "wiki_read",
    "description": "Read a specific wiki page by path. Use to get full context on a topic.",
    "parameters": {
        "path": {"type": "string", "description": "Wiki page path (e.g., notes/decisions/decision---wirebot-dual-role)"}
    }
}
```

#### 3.3 OpenClaw tool: wiki_decide

```json
{
    "name": "wiki_decide",
    "description": "Record a decision in the wiki. Creates a decision page with rationale, alternatives considered, and links to relevant context.",
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
3. Include top 3 prior decisions in the prompt context
4. Query wiki: `wb wiki search "tag:skill $DOMAIN" --format json`
5. Include relevant skills as constraints

#### 3.5 Process orphan pages

Run the 6R pipeline on the 796 orphan pages (571 Joplin + 225 ChatGPT):
1. **Auto-categorize** by namespace: joplin-import → archive (95%), ai-chats → reduce (50%), extract (50%)
2. **Batch reduce** valuable ChatGPT conversations → atomic notes in `/notes/`
3. **Link** surviving notes into MOCs
4. **Archive** or delete the rest

Target: reduce orphans from 952 to <100.

**Phase 3 success criteria:** Wirebot queries wiki during turns. Decisions auto-recorded. Focusa injects wiki context. Orphans reduced to <100.

---

### Phase 4: Continuous Intelligence Growth (Week 4+)

**Goal:** The system gets measurably smarter every day.

#### 4.1 Daily intelligence metrics

Track and report daily:
```yaml
daily_metrics:
  wiki_pages_created: N       # New knowledge added
  wiki_links_created: N       # Graph connectivity increase
  wiki_orphans_remaining: N   # Should decrease
  mem0_memories_added: N      # Cross-session memory growth
  focusa_decisions_recorded: N # Decisions captured
  focusa_ari_score: N         # Autonomy reliability
  focusa_rules_active: N      # Procedural memory depth
  kaizen_reflections: N       # Self-improvement signals
  wiki_agent_pages_fixed: N   # Autonomous maintenance
```

#### 4.2 Weekly graph health report

```bash
wb wiki stats  # Should show improving numbers:
# - Orphans decreasing
# - Links increasing
# - Stale pages decreasing
# - Active knowledge growing
```

#### 4.3 Monthly ontology review

- Are skills pages accurate?
- Are decision pages current?
- Are MOCs comprehensive?
- Are Focusa's procedural rules converging on useful patterns?
- Is ARI score trending up?

#### 4.4 Autonomy escalation gates

As the system proves reliable:
- AL0 → AL1: Focusa can auto-resume frames (after 30 days stable ARI > 70)
- AL1 → AL2: Focusa can select next task from Flow Mesh (after 60 days ARI > 80)
- AL2 → AL3: Wirebot can create subtasks autonomously (after 90 days ARI > 85)

Each gate requires operator approval + evidence.

---

## Implementation Priority

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
| **P2** | wiki_search / wiki_read tools for OpenClaw | OpenClaw, Wiki | 2 hours |
| **P2** | wiki_decide tool for OpenClaw | OpenClaw, Wiki | 1 hour |
| **P3** | Focusa Expression Engine → Wiki injection | Focusa, Wiki | 4 hours |
| **P3** | Kaizen → Wiki pipeline | wb kaizen, Wiki | 2 hours |
| **P3** | Process 796 orphan pages | Wiki, LLM | 8 hours |
| **P3** | Daily intelligence metrics | wb CLI | 3 hours |

**Total estimated effort:** ~35 hours across 4 phases

---

## Success Criteria

The system is "working as a single organism" when:

1. **Every Wirebot turn flows through Focusa** — meta-cognition is active, not theoretical
2. **Wirebot queries wiki before making decisions** — prior knowledge is used
3. **Decisions are automatically recorded** — in wiki, Mem0, and Focusa
4. **Skills become procedural rules** — wiki skills → Focusa rules → prompt constraints
5. **The graph grows daily** — wiki-agent creates pages, enrichment adds links, sessions add decisions
6. **Orphan pages < 100** — the wiki is connected, not a dump
7. **ARI score trends up** — autonomy is being earned through evidence
8. **Session learnings persist** — nothing vanishes between conversations
9. **Operator state affects agent behavior** — circadian phase, interruptibility respected
10. **Weekly metrics show improvement** — measurable intelligence growth

---

## Architecture Principles

1. **No silent data loss.** Every decision, every learning, every failure is captured somewhere permanent (wiki, Mem0, or Focusa event log).
2. **Graph over flat.** Wiki links, Mem0 graph (Kuzu), Focusa CLT — everything is connected, traversable.
3. **Fail-safe degradation.** If Focusa is down → passthrough. If wiki is down → use Mem0. If Mem0 is down → use local memory. Never break the agent.
4. **Earned autonomy.** No system escalates privileges without evidence + operator approval.
5. **Single source of truth per concern.** Wiki = knowledge. Mem0 = memory. Focusa = cognition. Flow Mesh = tasks. No duplication.
6. **Continuous growth.** Nightly enrichment, autonomous maintenance, session writeback — the system is always adding knowledge, never just consuming it.
