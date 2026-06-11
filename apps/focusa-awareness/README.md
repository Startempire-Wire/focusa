# Focusa Awareness Plugin

Standalone OpenClaw-compatible plugin that injects a Focusa Utility Card before agent start.

## Build

```bash
cd apps/focusa-awareness
bun run check
bun run build
```

The plugin intentionally declares a minimal local host API interface instead of importing `openclaw/plugin-sdk`; this keeps the package buildable in this repository while preserving the expected runtime hook shape.

## Runtime config

See `openclaw.plugin.json` for supported config fields: `focusaUrl`, `adapterId`, `workspaceId`, `agentId`, `operatorId`, `projectRoot`, `timeoutMs`, and `enabled`.
