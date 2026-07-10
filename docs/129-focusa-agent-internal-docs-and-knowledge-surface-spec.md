# Spec 129 — Focusa Agent-Internal Docs and Knowledge Surface

**Status:** Implemented
**Scope:** Public-safe agent documentation surface, AGENTS entry point, docs index wiring, and static guard.
**Boundary:** Agent-facing docs may explain architecture, commands, APIs, update policy, Workpoints, Trajectory, and public/private rules, but must not contain private paths, backend/admin URLs, secrets, full chat logs, or runtime-only operator material.

## 1. Problem

AI agents working in Focusa need a bounded documentation surface similar to Pi's docs: a stable place to understand what Focusa is, how the repo is organized, which commands are canonical, how Workpoints and Trajectory relate, and what public/private boundaries apply.

Without this, agents rediscover architecture from scattered specs or over-trust conversation tail memory.

## 2. Product boundary

The agent docs are public-safe. They are not a dump of local operator memory.

Allowed:

- Focusa architecture overview
- public command shape
- local daemon/API concepts
- Workpoint, Evidence, Trajectory, Context Authority explanations
- update/release policy at a high level
- public/private boundary rules
- software layout and proof commands

Forbidden:

- private host paths
- private admin URLs
- full chat logs
- secrets, keys, customer data, license records
- local-only runtime objects
- internal launch strategy or commercial calculations

## 3. Delivered docs surface

Create:

```text
docs/agent/01-focusa-agent-docs-index.md
```

This one bounded document is the hot-path entry point for agents. Additional agent docs may be added later as `02-...`, `03-...`, etc. using the repo naming rule.

## 4. Entry points

Wire the surface from:

- `AGENTS.md`
- `docs/INDEX.md`
- README docs section, if needed by future product pages

Agents should start with `AGENTS.md`, then read `docs/agent/01-focusa-agent-docs-index.md` before broad code changes.

## 5. Static guard

Create a guard test that proves:

- Spec 129 exists.
- Agent docs index exists.
- `AGENTS.md` links to the agent docs index.
- `docs/INDEX.md` links to the agent docs index.
- Agent docs include architecture, commands, API, update policy, Workpoints, Trajectory, and private-boundary sections.
- Agent docs avoid private paths, backend/admin URLs, and full chat-log leakage.

## 6. Acceptance

```bash
bash tests/spec129_agent_docs_surface_static_test.sh
cargo test -p focusa-cli --test public_surface_guard_e2e
scripts/guard-public-surface.sh
```

All must pass before closing the work item.
