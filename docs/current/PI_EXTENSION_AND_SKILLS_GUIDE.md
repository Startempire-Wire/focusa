# Pi Extension and Skills Guide

## Source inventory and installed activation

Use [the shared project journey](../agent/02-focusa-cohesive-project-flow.md). Discover tool and skill counts from the current registry and actual loaded harness; neither this guide nor generated parity proves installed support. New/expanded routes include Context Cognition curation/proof/optimization, Project Card and Genesis/bootstrap, Temporal Authority, session transfer/rollover, UIAI/WebMCP capability intake, preload packets, Silent Sessions, device pairing, prediction authority, and progressive Tool Discovery. Required canonical binding is exact Scope/Workstream/continuity/attachment, with current instruction/operation/frontier admission; root/continuity input fields alone do not establish it.

The canonical registry/generator produces project and packaged skill/runbook mirrors. Installed roots are configured by the actual owning harness; source mirrors are not evidence that installed copies or a running process refreshed. Regenerate, install through the approved mechanism and verify native reload separately. Counts come from the registry, not this prose:

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
- `focusa-project-scope` — verified Project/Workstream/continuity/attachment and operation scope.
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

## Approved dependency setup and validation

Use the declared locked dependency setup only when its installation/execution is authorized, under the owning user and approved host. Existing compatible cached dependencies may be reused; do not substitute an incompatible compiler or auto-install new dependencies. Builds/tests use the required job executor. The commands below are examples, not permission or proof they ran.

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/pi-extension
npm ci
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
