# docs/27-tui-spec.md — Focusa TUI Specification (ratatui) (AUTHORITATIVE)

This document specifies the **terminal UI (TUI)** for Focusa, implemented using
[`ratatui`](https://github.com/ratatui).

The TUI is:
- real-time
- introspective
- non-invasive
- calm
- composable

It is a **first-class capability client**, not a debug toy.

---

## 0. Design Principles

1. **Observe, don’t interrupt**
2. **Hierarchy over clutter**
3. **Live cognition > static logs**
4. **Everything navigable**
5. **No hidden state**

---

## 1. Global Layout (Default View)

```
┌────────────────────────────────────────────┐
│ Focusa — Cognitive Runtime (session: xyz) │
├───────────────┬───────────────────────────┤
│ Left Sidebar  │ Main Panel                │
│ (Navigation)  │ (Contextual View)         │
├───────────────┴───────────────────────────┤
│ Status Bar (Focus, Autonomy, UXP/UFI)     │
└────────────────────────────────────────────┘
```

---

## 2. Navigation Sidebar (Domains)

Mapped directly to Capability Domains:

```
▸ Focus State
▸ Focus Stack
▸ Lineage (CLT)
▸ References
▸ Gate
▸ Intuition
▸ Constitution
▸ Autonomy
▸ Metrics
▸ Cache
▸ Contribution
▸ Export
▸ Agents
▸ Events
```

Navigation keys:
- `↑↓` move
- `Enter` select
- `/` search
- `q` quit

---

## 3. Focus State View

Displays **current canonical cognition**.

### Panels:
- Intent
- Constraints
- Active Focus Frame
- Confidence
- Salient References
- Lineage Head

Supports:
- history browsing
- diffing (`d`)
- copy/export (`y`)

---

## 4. Focus Stack View

Visualizes nested focus frames:

```
[ Root Intent ]
  └─ [ Coding Task ]
      └─ [ File Refactor ] ← ACTIVE
```

Keys:
- `↑↓` navigate
- `Enter` inspect
- `Esc` back

---

## 5. Lineage (CLT) View

Tree visualization (scrollable):

```
● clt_001
│
├─● clt_004
│  └─○ summary
│
└─● clt_007 ← HEAD
```

Legend:
- ● active path
- ○ summary node
- faded = abandoned

Keys:
- `←→` expand/collapse
- `Enter` inspect node
- `b` mark branch
- `s` view summary

---

## 6. References View

Table view:

| Ref ID | Type | Size | Linked |
|------|------|------|--------|

Supports:
- preview
- range fetch
- provenance view

---

## 7. Gate View

Displays Focus Gate internals:

- candidate list
- scores
- decay curves
- explanation per candidate

Read-only, visual only.

---

## 8. Intuition View

Displays:
- signal timeline
- pattern clusters
- confidence bands

Signals are visually distinct from facts.

---

## 9. Constitution View

- current constitution text
- version history
- diffs
- CS drafts

Drafts can be:
- reviewed
- edited
- proposed (command)

---

## 10. Autonomy View

- current autonomy level
- earned score
- success/failure timeline
- explanations

Clear visual boundary between:
- allowed
- denied
- pending autonomy

---

## 11. Metrics View

Charts:
- UXP trend
- UFI trend
- cache hit/miss
- latency

ratatui sparklines & gauges.

---

## 12. Cache View

- cache classes
- live hit/miss feed
- recent bust reasons

---

## 13. Contribution View

- contribution status
- queue items
- review UI
- policy editor

No uploads without confirmation.

---

## 14. Agents View

- registered agents
- capabilities per agent
- active sessions

Agents are inspectable, not opaque.

---

## 15. Events View (Live)

Scrollable live event stream:

```
[12:41:02] focus_state.updated
[12:41:05] cache.bust (reason: fresh evidence)
```

Filterable by type.

---

## 16. Status Bar (Always Visible)

Displays:
- active focus frame
- autonomy level
- UXP / UFI
- session time
- connection health

---

## 17. Key Bindings Summary

- `?` help
- `/` search
- `d` diff
- `y` copy
- `Esc` back
- `q` quit

---

## 18. Implementation Notes (ratatui)

- Each domain = module
- Each panel = component
- Shared app state from Capabilities API
- Event-driven updates (SSE)
- No blocking calls

---

## 19. Canonical Rule

> **If the TUI cannot show it, the system does not truly understand it.**
