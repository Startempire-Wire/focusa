# Focusa Documentation Maintenance Runbook

## Coverage map

Every behavior change must update the applicable surfaces:

1. public `README.md` or `docs/current/` operator documentation;
2. `AGENTS.md` and `docs/agent/01-focusa-agent-docs-index.md` for agent routing;
3. `.pi/skills/` plus packaged `apps/pi-extension/skills/` skill/runbook parity;
4. Spec141 generated capability projections and all per-tool docs;
5. onboarding, lifecycle, recovery, and release proof where behavior affects users.

## Workflow

1. Map changed code since the previous release to feature/architecture terms.
2. Run `scripts/generate-agent-capability-descriptors.ts` and `scripts/generate-agent-tool-docs.ts`.
3. Run `scripts/generate-agent-skills.py` and verify root/package parity.
4. Run Spec141 conformance, Markdown links/lint, version-surface, and generated-drift checks.
5. Record gaps and proof on the canonical Workpoint.

## Boundaries

- New docs use numbered descriptive names except tool-mandated `README.md`, `AGENTS.md`, and `SKILL.md`.
- Keep private operational/commercial material outside public docs.
- Never claim completion from documentation presence alone; validate links, generated counts, and runtime parity.
