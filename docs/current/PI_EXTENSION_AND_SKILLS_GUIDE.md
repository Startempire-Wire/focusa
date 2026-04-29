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
- `predictive-power` — bounded prediction record/evaluate/stats workflow.

## Skill path hygiene

Canonical extension-packaged skills path:

```text
/home/wirebot/focusa/apps/pi-extension/skills
```

A stale reload path such as `~/apps/pi-extension/skills` resolves to `/root/apps/pi-extension/skills` in root-run Pi sessions. Do **not** symlink that path to the repo skill directory; that makes Pi load the same skill names twice and produces `[Skill conflicts]` collisions. Keep the stale compatibility directory present but empty, and keep canonical runtime skills in `/root/.pi/skills`.

Validate skill hygiene:

```bash
node scripts/validate-skill-hygiene.mjs
```

## Install dependencies for local validation

```bash
cd /home/wirebot/focusa/apps/pi-extension
npm install
./node_modules/.bin/tsc --noEmit
```

## Validate skills

```bash
cd /home/wirebot/focusa
node --input-type=module - <<'NODE'
import { loadSkills } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
const r = loadSkills({ cwd: process.cwd(), agentDir: '/root/.pi/agent', skillPaths: [], includeDefaults: true });
console.log(r.skills.map(s => [s.name, s.filePath]));
console.log(r.diagnostics);
NODE
```

## Tool contract validation

```bash
cd /home/wirebot/focusa
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

## Tool docs

Every current `focusa_*` tool has one individual doc under:

```text
docs/focusa-tools/tools/<tool-name>.md
```
