# Memory-Syncd Integration Spec — Focusa Bridge + SOUL Watch

**Status:** SPEC (ready for implementation)
**Component:** `/data/wirebot/bin/wirebot-memory-syncd` (Go binary)
**Service:** `wirebot-memory-syncd.service` on `:8201`
**Source:** `/home/wirebot/wirebot-core/cmd/memory-syncd/` (if exists) or recompile needed
**Grounding:** UNIFIED_ORGANISM_SPEC §9.4 (Letta), §14 Phase 0.4 (SOUL reload)

---

## 1. Purpose

memory-syncd is the **bidirectional sync bridge** between Wirebot's workspace files, Mem0, and Letta. It watches files via inotify and polls Mem0/Letta for changes.

---

## 2. Current Capabilities

- Watches `/home/wirebot/workspace/` for file changes (inotify + debounce)
- Syncs MEMORY.md changes → Mem0 facts
- Syncs BUSINESS_STATE.md changes → Letta core memory blocks
- Syncs Mem0 new facts → append to MEMORY.md
- Syncs Letta block changes → snapshot BUSINESS_STATE.md
- Hot cache at `:8201/cache/search` for instant recall
- Health endpoint at `:8201/health`

---

## 3. Required Additions

### 3.1 SOUL.md Watch → Focusa Constitution Reload

**Current:** memory-syncd watches workspace files but NOT SOUL.md for constitution reload.

**Add:** Watch `/home/wirebot/clawd/SOUL.md` (or `/home/wirebot/workspace/SOUL.md`). On change:
```go
// On SOUL.md file change detected:
exec.Command("wb", "soul", "reload").Run()
// Or direct HTTP: POST http://127.0.0.1:8787/v1/constitution/load
```

**Alternative:** If modifying the Go binary is too heavy, use a systemd path unit instead (see bead focusa-7p7 Option A). The path unit approach doesn't require recompiling memory-syncd.

### 3.2 Focusa Decision Writeback (Optional, Future)

When Focusa Focus State records decisions during a Wirebot session, those decisions should sync to workspace files for persistence:
- Focusa decisions → append to a `DECISIONS.md` workspace file
- memory-syncd picks up the file change → syncs to Mem0 + Letta

This creates a durable copy of Focusa decisions outside the Focusa SQLite database.

**Implementation:** Focusa daemon writes decisions to a workspace file on session close. memory-syncd's existing file watcher picks it up automatically. No memory-syncd code change needed — just Focusa writing to a watched directory.

---

## 4. What Does NOT Change

- Existing Mem0 ↔ workspace sync (working, keep)
- Existing Letta ↔ BUSINESS_STATE.md sync (working, keep)
- Hot cache functionality (working, keep)
- Health endpoint (working, keep)
- Service configuration (keep as-is)

---

## 5. Implementation Options

**Option A (recommended): systemd path unit for SOUL.md**
No memory-syncd code change. Create `/etc/systemd/system/soul-reload.path` that watches SOUL.md and triggers `wb soul reload`. See bead focusa-7p7.

**Option B: Add to memory-syncd Go binary**
Add SOUL.md path to inotify watch list. On modify, exec `wb soul reload`. Requires recompile from source.

**Recommendation:** Option A. Simpler, no recompile, same result.

---

## 6. Acceptance Criteria

1. SOUL.md changes trigger Focusa constitution reload within 10s
2. Existing sync functionality unaffected
3. No recompile required (if using Option A)
