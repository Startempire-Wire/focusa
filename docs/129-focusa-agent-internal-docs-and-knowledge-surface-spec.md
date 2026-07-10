# Spec 129 — Focusa Agent Internal Docs and Knowledge Surface

## Status

Draft — added to canonical repo reorganization order as `focusa-8304a.15`.

## Parent work

- Parent epic: `focusa-8304a` — Spec 123/124 execution order.
- Ordered bead: `focusa-8304a.15` — agent internal docs and knowledge surface.
- Dependency: `focusa-8304a.14` — public wording/surface guard must land first.

## Problem

Focusa has many docs, specs, generated tool pages, CLI references, API references, and private operator notes. Agents need a dependable entry point similar to Pi's bundled docs: a concise map that tells them how Focusa works, what files to read, what commands are canonical, what must stay private, and how to recover from common states.

Without this, agents rediscover architecture from transcript history, stale docs, or folder names, which causes scope drift, wrong updates, and private/public boundary mistakes.

## Goals

1. Provide an agent-facing docs entry point for Focusa internals.
2. Make the docs available in-repo and through installed package paths.
3. Explain Focusa software parts: core, API daemon, CLI, TUI, Pi extension, menubar, docs, release/update surfaces, and private boundary.
4. Link to current generated API/CLI/tool docs instead of duplicating stale payloads.
5. Teach agents the canonical workflow: project identity → trajectory → Workpoint → evidence → prediction/metacog → release proof.
6. Include a private-boundary warning so agent docs do not reintroduce private operator data into public docs.
7. Add a static guard proving the agent docs entry point exists and links current surfaces.

## Non-goals

- No raw transcript dumps in agent docs.
- No private pricing, vendor, license registry, or SignalOS strategy in public agent docs.
- No replacement for generated API/CLI/tool docs.
- No hidden authority: docs orient agents but do not override Workpoint, Trajectory, Context Authority, or license gates.

## Proposed doc locations

Primary public-safe entry point:

- `docs/agent-internal/00-focusa-agent-map.md`

Supporting docs:

- `docs/agent-internal/01-software-parts-and-locations.md`
- `docs/agent-internal/02-canonical-agent-workflow.md`
- `docs/agent-internal/03-command-and-api-map.md`
- `docs/agent-internal/04-private-boundary-and-public-surface-rules.md`
- `docs/agent-internal/05-release-update-and-install-surfaces.md`
- `docs/agent-internal/06-troubleshooting-and-recovery.md`

Index links:

- `docs/README.md` links the agent-internal map.
- `docs/AGENTS.md` names the map as the first Focusa-specific docs route.
- Root `AGENTS.md` references the map for agents working inside this repo.
- Package/install docs later copy or expose the same map for installed agents.

## Required content

### Software parts

The docs must explain these parts and where they usually live:

- `focusa-core` — core cognition/runtime/library crate.
- `focusa-api` / `focusa-daemon` — local HTTP daemon and systemd service.
- `focusa-cli` / `focusa` — operator and automation CLI.
- `focusa-tui` — terminal cockpit / Mission Deck surface.
- `apps/pi-extension` — Pi tool layer and Focusa-aware compaction/workpoint tools.
- `apps/menubar` — Mac menubar client.
- `docs/current` — current public docs and generated references.
- `docs/focusa-tools` — per-tool agent docs.
- `release-proof/public` — public-safe release proof.
- `.focusa-private` — ignored local-only operator/internal docs.
- `/usr/local/bin/*` — installed server binaries.
- `/usr/local/lib/focusa` — runtime home/update history/rollback.

### Canonical workflow

The docs must teach agents this route:

1. Verify project identity.
2. Refresh or read trajectory.
3. Resume or create Workpoint.
4. Resolve active object if ambiguous.
5. Execute the smallest scoped task.
6. Capture evidence.
7. Evaluate predictions / capture metacog lessons when outcome changes future behavior.
8. Checkpoint before compaction or risky continuation.

### Private/public boundary

The docs must state:

- `.focusa-private/` is ignored and local-only.
- Private docs may inform local work, but must not be copied into public tracked docs without explicit operator instruction.
- Public-safe docs should link summaries, not raw proofs/transcripts.
- The public-surface guard is the enforcement layer after Spec 123.5.1.

### Release/update awareness

The docs must include:

- latest release vs installed binary distinction,
- daemon/CLI/TUI version checks,
- release proof gates,
- OTA update spec link (`docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md`),
- license-level update behavior.

## Static guard

Add:

- `tests/agent_internal_docs_static_test.sh`

The guard must verify:

1. `docs/agent-internal/00-focusa-agent-map.md` exists.
2. `docs/README.md` links the map.
3. `docs/AGENTS.md` links the map.
4. The map links current API, CLI, Workpoint, Trajectory, tool docs, install/update, release proof, and private-boundary docs.
5. No agent-internal doc contains forbidden private path leaks except bounded references to `.focusa-private/` as an ignored boundary.
6. No raw transcript/proof paths are embedded.

## Acceptance criteria

- Agent docs exist and are public-safe.
- Agent docs give one obvious first route for new agents.
- Agent docs explain software parts and installed locations.
- Agent docs link generated/current references instead of duplicating them.
- Agent docs name private boundary rules.
- Agent docs include release/update awareness.
- Static guard passes locally and in CI.
- `focusa-8304a.15` closes only after guard proof and docs index wiring.

## Implementation order

1. Complete `focusa-8304a.8` through `focusa-8304a.14` in canonical order.
2. Implement Spec 129 docs map and supporting pages.
3. Wire docs indexes and AGENTS references.
4. Add static guard.
5. Run docs guard and public-surface guard.
6. Close `focusa-8304a.15` with evidence.
