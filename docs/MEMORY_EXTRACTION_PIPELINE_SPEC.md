# Memory Extraction Pipeline Spec — Go Code Fixes

**Status:** SPEC (ready for implementation)
**Component:** `/home/wirebot/wirebot-core/cmd/scoreboard/memory_extract.go`
**Also:** `/home/wirebot/wirebot-core/plugins/wirebot-memory-bridge/index.ts`
**Grounding:** UNIFIED_ORGANISM_SPEC §10A.6

---

## 1. Purpose

The memory extraction pipeline turns raw conversations and documents into structured memories that flow through the WINS portal approval queue to all 5 sinks.

---

## 2. 6 Fixes Required (From §10A.6 Pipeline Integrity Audit)

### Fix 1: Full Conversation Extraction (not just last message)

**File:** `/home/wirebot/wirebot-core/plugins/wirebot-memory-bridge/index.ts`
**Lines:** 1587-1604 (scoreboard extraction path) AND 1458-1483 (Mem0 direct path)

**Current:** Only sends `lastUser` and `lastAssistant` (`.pop()`)
**Required:** Send ALL user+assistant messages, chunked into windows of 10-20 turns

```typescript
// REPLACE lines 1587-1604:
const userMsgs = conversation.filter(m => m.role === "user" && m.content.length >= 20);
const assistantMsgs = conversation.filter(m => m.role === "assistant" && m.content.length >= 20);

// Chunk into windows of 10 turns
const chunkSize = 10;
for (let i = 0; i < Math.max(userMsgs.length, assistantMsgs.length); i += chunkSize) {
  const chunkUsers = userMsgs.slice(i, i + chunkSize).map(m => m.content).join("\n---\n");
  const chunkAssistants = assistantMsgs.slice(i, i + chunkSize).map(m => m.content).join("\n---\n");
  
  if (chunkUsers.length < 20) continue;
  
  await fetch(`${scoreboardUrl}/v1/memory/extract-conversation`, {
    method: "POST",
    headers: { "Authorization": `Bearer ${scoreboardToken}`, "Content-Type": "application/json" },
    body: JSON.stringify({ user_message: chunkUsers, assistant_message: chunkAssistants, surface: "openclaw" }),
    signal: AbortSignal.timeout(10000),
  }).catch(() => {});
  
  // 2s delay between chunks to avoid hammering
  await new Promise(r => setTimeout(r, 2000));
}
```

**Also remove 2-minute rate limit** for session-end extraction:
**File:** `memory_extract.go` lines ~848-852
**Change:** Remove or increase `convoExtractMu` / `lastConvoExtractTime` check for session-end calls. Keep rate limit for mid-conversation calls only.

### Fix 2: Change Extraction Model

**File:** `/data/wirebot/scoreboard/scoreboard.env`
**Add line:** `EXTRACTION_MODEL=minimax/MiniMax-M2.7`

**Code reads at:** `memory_extract.go` line 629:
`"model": envOr("EXTRACTION_MODEL", "kimi-coding/k2p5")`

Restart: `systemctl restart wirebot-scoreboard`

### Fix 3: Upgrade Wiki Sink

**File:** `/home/wirebot/wirebot-core/cmd/scoreboard/memory.go`
**Function:** `wikiAppendFact()`

**Current:** Appends bullet point to one page.
**Required:** Replace with `wikiDeliverMemory()`:

```go
func wikiDeliverMemory(fact, sourceType string) error {
    // Classify memory type
    lower := strings.ToLower(fact)
    
    if strings.HasPrefix(lower, "decision:") || strings.Contains(lower, "decided") {
        // Create decision page
        return wikiCreateDecisionPage(fact, sourceType)
    }
    
    if strings.Contains(lower, "skill:") || strings.Contains(lower, "pattern:") {
        // Update skill page
        return wikiUpdateSkillPage(fact, sourceType)
    }
    
    // Default: append to relevant project/concept page
    return wikiAppendToRelevantPage(fact, sourceType)
}
```

### Fix 4: Diary LLM Extraction

**File:** `/data/wirebot/bin/wiki-enrich-nightly.sh`
**Function:** `queue_and_approve_memory()` (~line 119)

**Current:** Raw text push + auto-approve
**Required:** Route through LLM extraction endpoint (see WIKI_ENRICH_NIGHTLY_SPEC.md §3.3)

### Fix 5: Focusa Event Extractor (NEW)

**Create:** `/data/wirebot/bin/focusa-event-extract.sh`
See main repo bead focusa-96k for full implementation details.

### Fix 6: Historical Session/Pi Batch Processors (NEW)

**Create:** `/data/wirebot/bin/openclaw-session-extract.sh` (bead focusa-do5)
**Create:** `/data/wirebot/bin/pi-session-extract.sh` (bead focusa-d8i)

---

## 3. The Complete Pipeline After Fixes

```
Source (any) → LLM extraction → QueueMemoryForApproval()
    → memory_queue (status: pending)
    → WINS portal review (approve / reject / correct)
    → writebackApprovedMemory()
    → 5 sinks: Mem0 + MEMORY.md + fact YAML + wiki + Letta
    → delivery worker (every 20s, retry with backoff)
```

Every source feeds the SAME queue. No parallel paths. No auto-approve.

---

## 4. Wiki Page Creation GraphQL (for upgraded wiki sink)

```graphql
mutation ($c: String!, $t: String!, $p: String!, $tags: [String]!) {
  pages { create(content: $c, description: "", editor: "markdown",
    locale: "en", isPrivate: false, isPublished: true,
    path: $p, tags: $tags, title: $t) {
    responseResult { succeeded message } page { id path }
  }
}
```

Endpoint: `http://127.0.0.1:7325/graphql`
Auth: `Authorization: Bearer $(cat /data/wirebot/secrets/wiki-api-token)`

Alternative: Use `wb wiki create` via `exec.Command("wb", "wiki", "create", ...)` from Go (handles auth internally).

## 5. Cross-References

- UNIFIED_ORGANISM_SPEC.md §10A.6 (pipeline integrity audit — 6 gaps)
- UNIFIED_ORGANISM_SPEC.md §10A.3 (WINS portal approval flow)
- WIKI_AGENT_SPEC.md (wiki-agent also creates pages — must use same GraphQL)
- WIKI_ENRICH_NIGHTLY_SPEC.md (nightly calls extraction endpoints)
- doc 44 §29 (/wbm cataloguing uses same pipeline)

---

## 6. Acceptance Criteria

1. Full conversation extracted (not just last message)
2. Extraction model is MiniMax M2.7 (not Kimi K2.5)
3. Wiki sink creates decision/skill pages (not bullet append)
4. Diary extraction uses LLM and lands as pending (not auto-approved)
5. All extractions route through QueueMemoryForApproval → WINS portal
