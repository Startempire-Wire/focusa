# docs/32-telemetry-tui.md — Telemetry TUI Integration (AUTHORITATIVE)

This document defines how CTL data is rendered in the Focusa TUI.

---

## 1. Telemetry Navigation Entry

```
▸ Telemetry
```

Subviews:
- Tokens
- Cognition
- Tools
- Autonomy
- UX Signals

---

## 2. Token View

Panels:
- tokens per model
- cache efficiency
- cost proxy
- latency histogram

Visuals:
- sparklines
- gauges

---

## 3. Cognition View

Timeline:
- focus depth
- CLT branching
- summarization

Heatmap:
- focus drift
- abandoned branches

---

## 4. Tool View

Chain graph:
- tool sequences
- failures highlighted

---

## 5. Autonomy View

- earned autonomy gauge
- success vs failure bands
- explanations

---

## 6. UX Signal View

- UXP trend
- UFI trend
- evidence citations
- override heatmap

---

## 7. Interaction Model

- arrow keys navigate
- `Enter` drills down
- `d` shows underlying events
- `e` export slice

---

## 8. Visual Semantics

- darker = higher focus
- lighter = background
- navy accent = confidence
- red only for failures

---

## 9. Canonical Rule

> **Telemetry should reveal cognition, not distract from it.**
