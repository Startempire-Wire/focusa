# Pi Extension and Skills Guide

## v0.9.142 runtime surface

The release surface is a 135-tool, scope-bound runtime. New/expanded routes include Context Cognition curation/proof/optimization, Project Card and Genesis/bootstrap, Temporal Authority, session transfer/rollover, UIAI/WebMCP capability intake, preload packets, Silent Sessions, device pairing, prediction authority, and progressive Tool Discovery. Every route preserves `project_root + continuity_id`, Workpoint authority, evidence refs, and operator steering precedence.

The generated skill copies below are the same 29 manifests in `.pi/skills/`, `apps/pi-extension/skills/`, and `${PI_SKILLS_DIR:-$HOME/.pi/skills}/`. Regenerate and validate before release:

```bash
python3 scripts/generate-agent-skills.py --check
node scripts/validate-skill-hygiene.mjs
python3 scripts/audit-agent-first-tool-surfaces.py --json /tmp/focusa-agent-first.json
```

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
- `focusa-agent-bootstrap` — bounded startup/resume orientation.
- `focusa-tool-discovery` — progressive search/describe/graph/bundle loading.
- `focusa-project-scope` — project-root and continuity authority.
- `focusa-session-recovery` — compaction, rollover, transfer, and lineage recovery.
- `focusa-browser-uiai` — UIAI/WebMCP session, diagnostics, evidence, and cleanup.
- `focusa-install-lifecycle` — install, repair, OTA, rollback, and uninstall proof.
- `focusa-security-auth-licensing` — permissions, pairing, revocation, licensing, and secrets.
- `focusa-resource-performance` — LowMem, Bloatgaurd, bounded traversal, and token budgets.
- `focusa-mission-canvas` — Mission Canvas, CRIST, Work Rail, and generated UI.
- `focusa-release-proof` — acceptance evidence, issues, changelog, and authorized release gates.
- `focusa-temporal-authority` — deadlines, freshness, history, and grounded forecasts.
- `focusa-spec-implementation` — call-stack/spec/task implementation discipline.
- `focusa-evidence-outcomes` — evidence, receipts, settlement, prediction outcomes, and learning.

Generated coverage and root/package parity: `docs/evidence/141-focusa-skill-runbook-coverage.json`.

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
python3 scripts/generate-agent-skills.py --check
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

The current count is generated from `docs/current/focusa-tool-contracts.json`; hand-maintained totals are non-authoritative. Descriptor, Pi, MCP, OpenAI, CLI, REST, Agent Card, and per-tool docs projections must pass their Spec141 drift checks before release.
