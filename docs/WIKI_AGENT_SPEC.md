# Wiki-Agent Spec — Autonomous Knowledge Graph Maintenance

**Status:** SPEC (ready for implementation)
**Component:** `/opt/wikijs/agent/wiki-agent.py` (582 lines, needs major upgrade)
**Service:** `wiki-agent.service` (systemd, Nice=15, CPUQuota=25%, MemoryMax=512M)
**Port:** None (daemon, accesses Wiki.js GraphQL at :7325)
**Grounding:** UNIFIED_ORGANISM_SPEC §10.5, §10A, §10B

---

## 1. Purpose

wiki-agent is the **autonomous knowledge graph maintainer**. It ensures the wiki grows denser, better-connected, and more useful over time — without operator intervention.

It is NOT a content creator. It is a **librarian and connector** that:
- Finds pages that should be linked and links them
- Assigns importance tiers so cognition focuses on high-signal content
- Reasons about fragments to discover hidden connections
- Detects staleness and demotes irrelevant pages
- Maintains MOC coverage and graph quality metrics
- Never deletes anything without operator approval

---

## 2. Current State (What Exists)

The current implementation does ONE thing:
1. Scan rendered HTML for CSS class `is-invalid-page` (red links)
2. Find source content from local directories
3. Generate pages to fill red links (up to 2 AI-generated per cycle)
4. Sleep 30 minutes between cycles

**It does NOT:**
- Detect unlinked pages
- Assign importance tiers
- Reason about fragments
- Detect staleness
- Maintain MOC connections
- Track graph health metrics
- Use LLM for connection discovery
- Feed the memory extraction pipeline

---

## 3. Required Capabilities (From Organism Spec)

### 3.1 Importance Tier Management (§10.5)

Assign and maintain `importance:*` tags on all wiki pages:

| Tier | Tag | Initial Assignment |
|---|---|---|
| T1 Active | `importance:active` | `/notes/*` pages with recent edits |
| T2 Reference | `importance:reference` | `/notes/*` pages untouched 90+ days |
| T3 Archive | `importance:archive` | Completed ops, old data |
| T4 Raw | `importance:raw` | `/joplin-import/*`, `/ai-chats/*`, `vault-import` tagged |
| T5 Quarantine | `importance:quarantine` | `quarantine` or `operator-review` tagged |

**Auto-demotion:** Nightly scan. Pages not touched in 90 days → drop one tier.
**Auto-promotion:** Pages recalled by agents (referenced in Focusa Focus State or Mem0 query results) → promote one tier.

**Implementation:**
```python
def assign_tiers(pages):
    for page in pages:
        current_tier = get_importance_tag(page)
        if current_tier:
            continue  # Already tiered
        
        # Initial assignment by namespace
        if page['path'].startswith('notes/'):
            set_tag(page, 'importance:active')
        elif page['path'].startswith('ops/sessions/') or page['path'].startswith('ops/'):
            set_tag(page, 'importance:reference')
        elif page['path'].startswith('joplin-import/') or page['path'].startswith('ai-chats/'):
            set_tag(page, 'importance:raw')
        elif has_tag(page, 'quarantine') or has_tag(page, 'operator-review'):
            set_tag(page, 'importance:quarantine')
        elif has_tag(page, 'vault-import'):
            set_tag(page, 'importance:raw')
        else:
            set_tag(page, 'importance:reference')  # Default

def auto_demote(pages):
    cutoff = datetime.now(timezone.utc) - timedelta(days=90)
    for page in pages:
        if page['updatedAt'] < cutoff:
            tier = get_importance_tag(page)
            if tier == 'importance:active':
                replace_tag(page, 'importance:active', 'importance:reference')
            elif tier == 'importance:reference':
                replace_tag(page, 'importance:reference', 'importance:archive')
```

### 3.2 Fragment Reasoning — Puzzle Assembly (§10.5)

Each nightly cycle, pick N unlinked pages (default 5) and reason about connections.

**Implementation:**
```python
def reason_about_fragments(unlinked_pages, limit=5):
    for page in unlinked_pages[:limit]:
        content = get_page_content(page['id'])
        
        # Gather operator context for reasoning
        context = {
            'mem0_search': search_mem0(content[:200]),  # Top memories
            'recent_decisions': search_wiki('tag:decision', limit=5),
            'soul_themes': read_soul_themes(),
            'active_projects': search_wiki('tag:project importance:active', limit=5),
        }
        
        # LLM call (MiniMax M2.7, ≤500 tok, 5s timeout)
        prompt = f"""This fragment was written by the operator. What might it connect to?

FRAGMENT: {content[:1000]}

OPERATOR CONTEXT:
Recent decisions: {context['recent_decisions']}
Active projects: {context['active_projects']}
Mem0 associations: {context['mem0_search']}

Generate 0-3 connection hypotheses. For each, specify:
- target_page: which existing wiki page this connects to
- relationship: how they connect
- confidence: 0.0-1.0

Return JSON array."""
        
        hypotheses = call_llm(prompt)
        
        for h in hypotheses:
            if h['confidence'] > 0.7:
                # Create wiki link
                create_link(page, h['target_page'])
                add_to_moc(page, h['target_page'])
            elif h['confidence'] > 0.4:
                # Add as comment hypothesis
                add_comment(page, f"Possible connection: {h['relationship']} → {h['target_page']}")
            # else: try again in 30 days
```

### 3.3 Connection Finding (No Red Links ≠ Connected)

Red links = 0 means no broken links. But 952 pages have zero INBOUND links.

**Implementation:**
```python
def find_and_create_connections(pages):
    # Build inbound link map
    inbound = count_inbound_links(pages)
    
    # For T1+T2 pages with 0 inbound links
    for page in pages:
        tier = get_importance_tag(page)
        if tier not in ('importance:active', 'importance:reference'):
            continue
        if inbound.get(page['path'], 0) > 0:
            continue
        
        # Try to find a MOC or project page to link FROM
        content = get_page_content(page['id'])
        tags = page.get('tags', [])
        
        # Rule-based: match tags to MOCs
        for tag in tags:
            moc = find_moc_for_tag(tag)
            if moc:
                add_link_to_moc(moc, page)
                break
        else:
            # LLM-assisted: ask which MOC this belongs to
            # (Only for T1 pages — T2 can wait)
            if tier == 'importance:active':
                suggest_moc_placement(page, content)
```

### 3.4 Staleness Detection

```python
def detect_staleness(pages):
    now = datetime.now(timezone.utc)
    for page in pages:
        tier = get_importance_tag(page)
        age_days = (now - page['updatedAt']).days
        
        if tier == 'importance:active' and age_days > 90:
            # Active page hasn't been updated in 90 days
            log(f"Stale active page: {page['path']} ({age_days} days)")
            # Don't auto-demote in this pass — tier management handles that
            
        if tier == 'importance:active' and age_days > 30:
            # Flag for potential refresh
            add_tag(page, 'needs-refresh')
```

### 3.5 Graph Health Metrics

Each cycle, compute and log:
```python
def compute_graph_health():
    pages = get_all_pages()
    t1 = [p for p in pages if has_tag(p, 'importance:active')]
    t2 = [p for p in pages if has_tag(p, 'importance:reference')]
    
    inbound = count_inbound_links(pages)
    outbound = count_outbound_links(pages)
    
    metrics = {
        'total_pages': len(pages),
        't1_count': len(t1),
        't2_count': len(t2),
        'unlinked_t1_t2': len([p for p in t1+t2 if inbound.get(p['path'], 0) == 0]),
        'avg_inbound_t1_t2': mean([inbound.get(p['path'], 0) for p in t1+t2]),
        'avg_outbound_t1_t2': mean([outbound.get(p['path'], 0) for p in t1+t2]),
        'moc_count': len([p for p in pages if p['path'].startswith('notes/_mocs/')]),
        'stale_t1': len([p for p in t1 if (now - p['updatedAt']).days > 30]),
        'red_links': count_red_links(pages),
        'fragments_connected_today': 0,  # Updated by fragment reasoning
    }
    
    # Write to state file
    save_metrics(metrics)
    # Also write to wiki page for dashboard
    update_wiki_metrics_page(metrics)
```

---

## 4. Cycle Structure

Each daemon cycle (every 30 minutes):

```
1. Fetch all pages (GraphQL)
2. Tier management:
   a. Assign tiers to untiered pages
   b. Auto-demote untouched 90d pages
   c. Auto-promote recently-recalled pages
3. Red link scan (existing behavior, keep)
4. Connection finding:
   a. Count inbound links for T1+T2
   b. For unlinked T1 pages: try rule-based MOC placement
   c. For unlinked T1 pages (no rule match): LLM-assisted placement (max 2/cycle)
5. Fragment reasoning:
   a. Pick 5 unlinked T4/untiered pages
   b. LLM reasoning for connection hypotheses (max 2/cycle)
   c. Create links for high-confidence, add comments for medium
6. Staleness detection:
   a. Flag T1 pages >30 days as needs-refresh
7. Compute and log graph health metrics
8. Save state
9. Sleep 30 minutes
```

---

## 5. LLM Usage

**Model:** MiniMax M2.7 (via OpenClaw gateway or direct API)
**Budget per cycle:** ≤2000 tokens (max 2 LLM calls × ~500 tok each + 2 fragment calls × ~500 tok)
**Timeout:** 5s per call
**Fallback:** On LLM failure, skip LLM-dependent steps. Heuristic steps still run.

---

## 6. Configuration

```python
# In wiki-agent.py config section
BATCH_SIZE = 5              # Pages to process per step
SLEEP_BETWEEN = 1800        # 30 min between cycles
MAX_AI_PER_CYCLE = 2        # Max LLM calls for connections
MAX_FRAGMENT_PER_CYCLE = 2  # Max LLM calls for fragment reasoning
DEMOTION_DAYS = 90          # Days before auto-demotion
STALE_DAYS = 30             # Days before stale flag
LLM_MODEL = "minimax/MiniMax-M2.7"
LLM_TIMEOUT = 5
LLM_MAX_TOKENS = 500
```

---

## 7. Core Axioms (From Organism Spec)

1. **If the operator recorded it, it has meaning.** Never classify content as meaningless.
2. **Connect, don't judge.** The agent's job is to find relationships, not evaluate worth.
3. **Fragments are puzzle pieces.** Unlinked notes may click into place with more context.
4. **Nothing is deleted.** Pages demote in tier but are never removed by the agent.
5. **Deletion requires same-session operator approval.** Never auto-delete. Never carry blanket authorization.

---

## 8. Files

| File | Purpose |
|---|---|
| `/opt/wikijs/agent/wiki-agent.py` | Main daemon (rewrite) |
| `/opt/wikijs/agent/state.json` | Persistent state (cycle count, metrics, watermarks) |
| `/opt/wikijs/agent/wiki-agent.log` | Log file |
| `/opt/wikijs/agent/queue.jsonl` | Work queue |
| `/opt/wikijs/agent/notifications.jsonl` | Operator notifications |
| `/etc/systemd/system/wiki-agent.service` | Systemd service |

---

## 9. Actual API Details

### Wiki.js GraphQL (tag mutation)
```graphql
# Update page tags
mutation ($c: String!, $t: String!, $p: String!, $tags: [String]!) {
  pages { update(id: PAGE_ID, content: $c, description: "",
    editor: "markdown", locale: "en", isPrivate: false, isPublished: true,
    path: $p, tags: $tags, title: $t) {
    responseResult { succeeded message }
  }
}
```
Endpoint: `http://127.0.0.1:7325/graphql`
Auth: `Authorization: Bearer $(cat /data/wirebot/secrets/wiki-api-token)`

### Mem0 Search (for fragment reasoning context)
```
POST http://127.0.0.1:8200/v1/search
Body: {"query": "fragment text", "limit": 5, "namespace": "wirebot_verious"}
Returns: [{"memory": "...", "score": 0.85, "id": "..."}]
```

### LLM Call (MiniMax M2.7 via OpenClaw gateway)
```
POST http://127.0.0.1:18789/v1/chat/completions
Headers: Authorization: Bearer $GATEWAY_TOKEN
Body: {"model": "minimax/MiniMax-M2.7", "messages": [...], "max_tokens": 500}
GATEWAY_TOKEN: 88f4cdab-357a-464f-b68d-ebec3ddd2531
```

### Scoreboard Memory Queue (for fragment insights)
```
POST http://127.0.0.1:8100/v1/memory/queue
Headers: Authorization: Bearer $SCOREBOARD_TOKEN
SCOREBOARD_TOKEN: from /data/wirebot/scoreboard/scoreboard.env (GATEWAY_TOKEN line)
```

### Cross-References
- UNIFIED_ORGANISM_SPEC.md §10.5 (importance tiers, fragment reasoning, axioms)
- UNIFIED_ORGANISM_SPEC.md §10A (historical inference pipeline)
- MEMORY_EXTRACTION_PIPELINE_SPEC.md (memory queue integration)
- WIKI_ENRICH_NIGHTLY_SPEC.md (nightly enrichment calls wiki-agent metrics)

---

## 10. Dependencies

- Wiki.js running (:7325) with GraphQL API
- Wiki API token at `/data/wirebot/secrets/wiki-api-token`
- MiniMax M2.7 accessible via OpenClaw gateway (:18789)
- Gateway token: `88f4cdab-357a-464f-b68d-ebec3ddd2531`
- Mem0 running (:8200) for fragment reasoning context
- Scoreboard running (:8100) for memory queue

---

## 10. Acceptance Criteria

1. All pages in active namespaces have importance tiers assigned
2. T1+T2 unlinked page count decreases weekly
3. Fragment reasoning produces connection hypotheses nightly
4. Graph health metrics logged every cycle
5. Auto-demotion runs for 90-day untouched pages
6. LLM calls stay within budget (≤2000 tok/cycle)
7. No pages deleted without operator approval
8. Red link filling continues to work (existing behavior preserved)
