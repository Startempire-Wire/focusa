---
name: focusa-context
description: Include Focusa cognitive context in your prompt
---

## Focusa Cognitive Context

Current Focus State is injected automatically by the focusa-pi-bridge extension.

Use this template when you need to explicitly reference Focusa state:

- Check `/v1/focus/stack` for current frame and Focus State
- Decisions are recorded via `focusa_decide` tool
- Constraints limit what actions are allowed
- Failures track what went wrong for learning

### Usage
```
/template:focusa-context
```
