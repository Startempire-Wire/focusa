# Wiki Enrichment Nightly Spec — Upgrade

**Status:** SPEC (ready for implementation)
**Component:** `/data/wirebot/bin/wiki-enrich-nightly.sh` (475 lines Python embedded in bash)
**Timer:** `wiki-enrich.timer` (03:00 nightly)
**Grounding:** UNIFIED_ORGANISM_SPEC §10.8, §10A, §10A.6

---

## 1. Purpose

The nightly enrichment script is the **daily intelligence growth engine**. It runs at 3am and ensures:
- Knowledge graph grows from new vault content
- Memories are extracted (via LLM, not raw text)
- Hygiene steps keep the system sharp
- Historical backlog is chipped away
- Metrics are captured

---

## 2. Current Problems

| Problem | Impact | Fix |
|---|---|---|
| `queue_and_approve_memory()` pushes raw text and auto-approves | Bypasses WINS portal review. No LLM extraction. | Use `POST :8100/v1/memory/extract-vault` instead |
| No historical inference calls | 805 vault files, 2584 sessions unprocessed | Add vault/session extraction calls |
| No hygiene steps | Event log grows unbounded, no TTL enforcement | Add hygiene step calls |
| No graph health metrics | No tracking of improvement | Add metric collection |

---

## 3. Required Nightly Steps

The enrichment script should execute these steps in order:

### Step 1: Vault-to-Wiki Sync (existing, keep)
```bash
/data/wirebot/bin/sync-vault-wiki.sh delta
```

### Step 2: Wiki Tagging (existing, keep)
Existing tag inference logic for untagged pages.

### Step 3: Learning Diary (existing, FIX extraction)
Create nightly diary wiki page (existing behavior).

**FIX:** Replace `queue_and_approve_memory()` with:
```python
# Instead of raw text push + auto-approve:
# POST to scoreboard extraction endpoint (LLM-backed)
resp = http_json("POST", scoreboard_url + "/v1/memory/extract-vault", 
    {"path": diary_wiki_path, "limit": 1}, headers=headers, timeout=60)
```
This routes through LLM extraction → QueueMemoryForApproval → WINS portal (pending, NOT auto-approved).

### Step 4: Historical Vault Extraction (NEW)
```python
# Process 20 unprocessed vault files per night
resp = http_json("POST", scoreboard_url + "/v1/memory/extract-vault",
    {"path": "/data/wirebot/obsidian", "limit": 20}, headers=headers, timeout=600)
```

### Step 5: Historical Session Extraction (NEW)
```bash
# Process 5 oldest unprocessed OpenClaw sessions
/data/wirebot/bin/openclaw-session-extract.sh --limit 5
```

### Step 6: Historical Pi Session Extraction (NEW)
```bash
# Process 2 oldest unprocessed Pi sessions
/data/wirebot/bin/pi-session-extract.sh --limit 2
```

### Step 7: Fact Backfill (NEW)
```bash
# Push 50 unsynced fact YAMLs to Mem0
/data/wirebot/bin/backfill-facts-to-mem0.sh --limit 50
```

### Step 8: Hygiene (NEW)
```bash
# Call the Focusa nightly hygiene script
/data/wirebot/bin/focusa-nightly-hygiene.sh
```

### Step 9: Metrics Snapshot (NEW)
```python
# Collect daily intelligence metrics
metrics = {
    'date': today,
    'wiki_pages_total': len(pages),
    'wiki_knowledge_pages': count_by_tier('importance:active') + count_by_tier('importance:reference'),
    'mem0_pending': get_memory_queue_count('pending'),
    'vault_files_remaining': 876 - len(watermark),
    'sessions_remaining': count_unprocessed_sessions(),
    'focusa_ari': get_focusa_ari(),
}
append_to_jsonl('/data/wirebot/state/intelligence-metrics.jsonl', metrics)
```

---

## 4. Acceptance Criteria

1. Diary extraction goes through LLM, not raw text push
2. Diary memories land in WINS queue as pending, not auto-approved
3. 20 vault files processed per night (watermark advances)
4. Intelligence metrics written nightly
5. Hygiene steps execute without error
6. Total runtime stays under 45 minutes (MAX_SECONDS=2700)
