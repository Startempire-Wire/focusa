---
name: focusa-context
description: Reference current Focusa context behavior and surfaces
---

## Focusa Cognitive Context

Focusa context is **not** injected as an always-on full-state dump.

Current `focusa-pi-bridge` behavior is:
- operator-first routing
- minimal applicable slice selection
- Focusa context suppressed when the turn is not focus-relevant
- consultation traces emitted when mission, decisions, constraints, or working-set context are actually used

Use this template when you want a quick reminder of the live Focusa model:

- Check `/v1/focus/stack` for the active frame and current Focus State
- Decisions are written via `focusa_decide`
- Constraints are discovered limits and should be respected
- Failures capture specific breakage plus diagnosis
- The extension hot path lives in `apps/pi-extension/src/turns.ts`

### Usage
```
/template:focusa-context
```
