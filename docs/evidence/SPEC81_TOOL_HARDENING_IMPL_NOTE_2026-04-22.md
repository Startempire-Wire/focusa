# SPEC81 Tool Hardening Implementation Note

**Date:** 2026-04-22  
**Spec:** `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`  
**Task:** `focusa-p5hr.2`

## What changed

The 10 required Spec81 tree/metacognition tools were tightened in the Pi extension tool layer.

### Shared hardening added
- shared validation failure envelope
- shared required/optional string validators
- shared array validator
- stricter limits and patterns for ids, turn ranges, text, and arrays

Code:
- `apps/pi-extension/src/tools.ts:978`
- `apps/pi-extension/src/tools.ts:1002`
- `apps/pi-extension/src/tools.ts:1020`
- `apps/pi-extension/src/tools.ts:1033`
- `apps/pi-extension/src/tools.ts:1047`

### Tool surfaces hardened
- tree tools: `apps/pi-extension/src/tools.ts:1068` through `apps/pi-extension/src/tools.ts:1168`
- metacognition tools: `apps/pi-extension/src/tools.ts:1170` through `apps/pi-extension/src/tools.ts:1280`

## Validation

```bash
apps/pi-extension/node_modules/.bin/tsc -p apps/pi-extension/tsconfig.json
```

Result: pass.
