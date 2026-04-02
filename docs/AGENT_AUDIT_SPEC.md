# Agent Audit Spec — Live Cognitive Dashboard Upgrade

**Status:** SPEC (ready for implementation)
**Component:** `/opt/agent-audit/main.go` (2452 lines)
**Service:** `agent-audit.service` on `:3020`
**URL:** `audit.philoveracity.com` (Cloudflare tunnel)
**PWA:** Installable on mobile (manifest.json, portrait, dark theme)
**Grounding:** UNIFIED_ORGANISM_SPEC §9.10

---

## 1. Purpose

Agent Audit is the **operator's mobile window into the organism's cognition**. Currently shows agent sessions and Focusa CI probe results. Must be upgraded to show LIVE Focusa state, memory queue status, and graph health.

---

## 2. Current State

- Agent sessions, actions, errors, tool/model breakdowns ✅
- Focusa API compliance probe (CI artifact) ✅
- PWA installable on Samsung ✅

---

## 3. Required Panels

Add after existing content in `/opt/agent-audit/main.go`:

| Panel | Focusa Endpoint | What It Shows | Timeout |
|---|---|---|---|
| Focus Stack | `GET :8787/v1/focus/stack` | Active frame title, goal, depth | 2s |
| Focus State | `GET :8787/v1/ascc/state` | Intent, decisions count, constraints count, failures count | 2s |
| Autonomy | `GET :8787/v1/autonomy` | ARI score, AL level | 2s |
| Gate | `GET :8787/v1/gate/scores` | Surfaced candidate count | 2s |
| Constitution | `GET :8787/v1/constitution/active` | Version, principles count | 2s |
| Reflection | `GET :8787/v1/reflect/status` | Last run, scheduler state | 2s |
| Telemetry | `GET :8787/v1/telemetry/tokens` | Total tokens, per-task | 2s |
| RFM | `GET :8787/v1/rfm` | Level, AIS score | 2s |
| Memory Queue | `GET :8100/v1/memory/queue` (auth'd) | Pending/approved/rejected counts | 2s |
| Graph Health | Computed from `wb wiki stats` | T1+T2 count, link density, unlinked count | 5s |

All with 2s timeout. On failure: show "unavailable" badge with staleness indicator.

---

## 4. Mobile UX

- Panels collapse to summary cards (tap to expand)
- Focus Stack: active frame title + ARI badge always visible at top
- Gate: count badge — tap for candidate list
- Memory Queue: pending count badge (red if >20)
- Dark theme matches existing UI
- PWA offline: show last-known state with "stale Xs ago" indicator

---

## 5. Implementation

**File:** `/opt/agent-audit/main.go`
**Where:** After existing Focusa API compliance panel (~line 1637)
**Pattern:** Same as existing panels — Go HTTP client + HTML template rendering

```go
// Focusa panels
focusaData := fetchFocusaPanels()  // HTTP calls with 2s timeout each
// Render in statCards template
```

**Build:** `cd /opt/agent-audit && go build -o agent-audit . && systemctl restart agent-audit`

---

## 6. Auth for Scoreboard Endpoints

Memory queue endpoint requires auth:
```go
token := os.Getenv("SCOREBOARD_TOKEN")
if token == "" {
    // Read from scoreboard env
    data, _ := os.ReadFile("/data/wirebot/scoreboard/scoreboard.env")
    // Parse GATEWAY_TOKEN line
}
req.Header.Set("Authorization", "Bearer " + token)
```

Focusa endpoints (:8787) do NOT require auth in current config.

## 7. Cross-References

- UNIFIED_ORGANISM_SPEC.md §9.10 (Agent Audit as mobile cognitive surface)
- WIKI_AGENT_SPEC.md (graph health metrics — Audit shows same KPIs)
- MEMORY_EXTRACTION_PIPELINE_SPEC.md (memory queue counts)

---

## 8. Acceptance Criteria

1. All 10 panels render on mobile
2. Panels show live data (refresh on page load)
3. Timeout failures show "unavailable" badge
4. Memory queue pending count visible
5. ARI score + frame title always visible at top
