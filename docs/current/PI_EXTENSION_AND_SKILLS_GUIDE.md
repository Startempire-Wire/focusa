# Pi Extension and Skills Guide

## Current locations

- Pi extension source: `apps/pi-extension/`
- Project skill copies: `.pi/skills/`
- Extension-packaged skill copies: `apps/pi-extension/skills/`
- Installed runtime skill copies: `${PI_SKILLS_DIR:-$HOME/.pi/skills}/`

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
${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/pi-extension/skills
```

A stale reload path such as `~/apps/pi-extension/skills` resolves under the runtime user home and may duplicate the repo skill directory. Do **not** symlink that stale path to the repo skill directory; that makes Pi load the same skill names twice and produces `[Skill conflicts]` collisions. Keep any stale compatibility directory present but empty, and keep canonical runtime skills in `${PI_SKILLS_DIR:-$HOME/.pi/skills}`.

Validate skill hygiene:

```bash
node scripts/validate-skill-hygiene.mjs
```

## Install dependencies for local validation

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/pi-extension
npm install
./node_modules/.bin/tsc --noEmit
```

## Validate skills

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-skill-hygiene.mjs
```

## Tool contract validation

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

## Tool docs

Every current `focusa_*` tool has one individual doc under:

```text
docs/focusa-tools/tools/<tool-name>.md
```

Current public count: 62 tools. Spec97 adds `focusa_reflex_primitives` for read-only Reflex Primitive summaries; Spec98 adds `focusa_project_card_outcome` for project-card learning outcomes and `focusa_session_transfer` for save/continue transfer between Pi sessions. These pair with direct APIs, traversal/context surfaces, and bounded result envelopes.
