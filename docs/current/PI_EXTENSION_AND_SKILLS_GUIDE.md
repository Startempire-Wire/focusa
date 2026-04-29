# Pi Extension and Skills Guide

## Current locations

- Pi extension source: `apps/pi-extension/`
- Project skill copies: `.pi/skills/`
- Extension-packaged skill copies: `apps/pi-extension/skills/`
- Installed runtime skill copies: `/root/.pi/skills/`

## Main skill and companion skills

- `focusa` — router/mental model.
- `focusa-workpoint` — Workpoint continuity.
- `focusa-metacognition` — learning loop.
- `focusa-work-loop` — continuous work-loop control.
- `focusa-cli-api` — direct daemon/CLI/API operations.
- `focusa-troubleshooting` — degraded/offline/pending/blocked recovery.
- `focusa-docs-maintenance` — public docs, tool docs, evidence, snapshot wording.

## Validate skills

```bash
node --input-type=module - <<'NODE'
import { loadSkills } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
const r = loadSkills({ cwd: process.cwd(), agentDir: '/root/.pi/agent', skillPaths: [], includeDefaults: true });
console.log(r.skills.map(s => [s.name, s.filePath]));
console.log(r.diagnostics);
NODE
```

## Tool docs

Every current `focusa_*` tool has one individual doc under:

```text
docs/focusa-tools/tools/<tool-name>.md
```
