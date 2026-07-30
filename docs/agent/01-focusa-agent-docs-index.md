# Focusa Agent Docs Index

This is the bounded, public-safe starting point for AI agents working in the Focusa repo. Use it before broad code changes or after context loss.

## 0. Spec 135 and Mission Canvas authority

Before changing Mission Canvas, Pi UI, Work Surfaces, workspace verticals, generated C.R.I.S.T. UI, renderer code, proof artifacts, or Spec 135 closure state, read:

1. `docs/135-series-current-manifest.md`
2. `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`
3. `docs/agent/spec135-implementation-acceleration-directive.md`
4. the affected existing 135A–135K documents
5. current machine-readable ledger and runtime proof

Do not create another lettered companion for clarification. The series is frozen at 135K. The manifest and machine-readable contract resolve host, renderer, toggle, proof, and closure conflicts.

The fixed operator intent is:

```text
Pi terminal interaction
        ⇅ light switch controlled directly from Pi
Focusa-owned rich Mission Canvas professional GUI over the same live Pi session
```

Record `interaction_mode` and `host_renderer` separately. A terminal TUI projection is not a rich graphical GUI.

## 1. What Focusa is

Focusa is the local-first proof and continuity layer for AI coding agents. It keeps long-running work attached to a typed Workpoint, linked Evidence, and a next safe action so agents do not rely on chat-tail memory.

## 2. Architecture map

| Layer | Purpose | Key locations |
| --- | --- | --- |
| CLI | Operator and agent command surface | `crates/focusa-cli/src/commands/` |
| API daemon | Local typed HTTP API | `crates/focusa-api/src/routes/` |
| Core | reducers, Workpoints, Evidence, runtime state, persistence | `crates/focusa-core/src/` |
| Work loop + Silent Sessions | governed execution, durable runs, steering, receipts | `crates/focusa-core/src/silent_sessions/`, `docs/133-silent-sessions-final-release-proof.md` |
| Mission Canvas + Work Rail | scoped professional workspace projection, Work Surfaces, queues, generated UI | `docs/135-series-current-manifest.md`, `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml` |
| Rich Focusa Pi host | Pi-controlled Focusa Mission Canvas webview/window over the same live session | Spec 135 implementation target; must not be replaced by a terminal shell claim |
| Connectors + domains | provider-neutral context, auth lifecycle, software/domain projections | `crates/focusa-core/src/connectors.rs`, `docs/contracts/spec135/` |
| Native TUI / terminal projections | truthful terminal client and compatibility fallbacks | `crates/focusa-tui/`, `apps/pi-extension/` |
| Pi extension | Focusa Pi tools, authority hooks, compaction/OTA/runtime bridge, rich-host lifecycle control | `apps/pi-extension/` |
| Agent machine contracts | Pi/MCP/OpenAI/CLI/REST schemas and Agent Card | `docs/contracts/spec141/generated-capability-v2/` |
| Skills + runbooks | progressive agent onboarding and recovery playbooks | `.pi/skills/`, `apps/pi-extension/skills/` |
| Menubar preview | bounded status, urgent peeks, lifecycle controls, and rich Mission Canvas launch/focus | `apps/menubar/` |
| UIAI Engine Cockpit | distinct UIAI-owned browser execution, FPV, Test Lab, diagnostics, and browser proof product | UIAI Engine repository/contracts |
| Public docs | current reference, onboarding, lifecycle, and specs | `README.md`, `docs/`, `docs/current/` |

### 2.1 Current authority and recovery model

- Exact authority is `project_root + continuity_id`; parent repositories and worktrees are ranked binding candidates, then verified before mutation.
- Workpoint is immediate action authority; Trajectory supplies destination, current state, gap, and waypoints.
- Focus State is the bounded decision/constraint/failure journal, not a transcript replacement.
- Silent Sessions are daemon-native. Exact `session_id`, `run_id`, `generation`, approval, and idempotency values govern mutations.
- Proactive compaction preserves canonical Workpoint/Trajectory packets and queues governed automatic rollover after bounded transport exhaustion.
- Cache-safe context keeps stable prefixes and current user-tail authority while classifying degraded fallbacks explicitly.
- Mission Canvas binds Work Surfaces to canonical operations and project scope; browser/UIAI capabilities remain session-and-origin bound.
- Mission Canvas visual focus is presentation state, not daemon-global project, session, or Workpoint authority.
- Customer lifecycle requires verified install/repair, trusted update or OTA rollback, and uninstall that preserves user data unless purge is explicit.

### 2.2 Mission Canvas renderer classifications

```text
focusa_pi_rich_window
  Required primary rich Canvas host for Focusa-enhanced Pi.

pi_terminal_projection
  Compatibility/terminal fallback. Never label it the rich GUI.

uiai_engine_cockpit
  Distinct rich UIAI host that may embed Focusa projections.

mission_deck_web
  Focusa guided PWA/web host.

native_tui
  Separate terminal client projection.

menubar_peek
  Bounded status/launcher projection, not the full Canvas.
```

### 2.3 All-Pi-tool and skill discovery

1. `focusa_agent_card` reports the runtime tool count, complete installed skill/runbook inventory, interfaces, auth, and registry digest.
2. `focusa_tool_search` finds the narrowest capability without hot-loading every schema.
3. `focusa_tool_describe` cold-loads one strict contract; `focusa_tool_graph` or `focusa_tool_bundle` expands only the selected workflow.
4. `docs/contracts/spec141/generated-capability-v2/pi-tools.json` is the machine projection for every Focusa Pi tool.
5. `docs/focusa-tools/tools/focusa_<name>.md` is the human reference for each tool.
6. Load the matched `.pi/skills/<skill>/SKILL.md`, then its numbered runbook under `references/`.

A release gate must prove runtime tool count = contracts = Pi descriptors = per-tool docs, and installed skills/runbooks = packaged skill/runbook copies.

## 3. Canonical command surface

Start with:

```bash
focusa help all
focusa help migration
focusa project
focusa setup wizard --dry-run
focusa first-mission --project-root "$PWD" --dry-run --json
focusa status operator --json
```

Core continuity commands:

```bash
focusa workpoint checkpoint --project-root "$PWD" --continuity-id demo --mission "Mission" --next-action "Next slice" --json
focusa workpoint evidence-link --target-ref tests --result "smoke passed" --evidence-ref "test:smoke" --json
focusa workpoint resume --project-root "$PWD" --continuity-id demo --copy-prompt
```

Background execution and lifecycle discovery:

```bash
focusa silent --help
focusa tui --headless-self-test
focusa update --help
bash scripts/install-focusa.sh --dry-run --eval
bash scripts/install-focusa.sh --uninstall        # preserves user data
focusa uninstall --dry-run --keep-data
```

Safety and proof commands:

```bash
focusa action preflight --current-ask "change binary" --kind binary_replace --target /usr/local/bin/focusa --source github_release_asset --install-role live_build_host --project-root "$PWD" --json
focusa cleanup --safe --project-root "$PWD" --dry-run --json
scripts/guard-public-surface.sh
bash tests/spec_cli_cross_phase_smoke_test.sh
```

## 4. API and daemon rules

- Default daemon URL: `http://127.0.0.1:8787`.
- Health route: `GET /v1/health`.
- Workpoint resume route: `POST /v1/workpoint/resume` with a JSON body.
- Telemetry snapshot route: `GET /v1/telemetry/snapshot`.
- Project-scoped mutations must use a verified safe project root.
- Daemon-global advisory surfaces must say they are advisory and non-canonical.
- Rich-host lifecycle and Canvas layout operations must use typed exact scope and generated contracts.

## 5. Workpoints, Evidence, and Trajectory

- **Workpoint** is the immediate continuation contract: mission, scope, current action, next action, blockers, and proof handles.
- **Evidence** is proof linked to the active Workpoint: tests, files, route checks, screenshots, command output, or release checks.
- **Trajectory** is advisory north-star context: long-term direction and current gap. It orients work but does not override a canonical Workpoint.
- **Context Authority** decides whether a proposed action matches the task, project, environment, and install role.

Never treat transcript tail or Canvas visual focus as canonical authority when a Workpoint or scope gate is available.

## 6. Update and release policy

- Use the GitHub release pipeline for public install/release artifacts.
- Keep CLI/daemon versions paired.
- Run focused tests for changed crates, then broader smoke tests when command surfaces change.
- Public release gates include the public-surface guard and cross-phase CLI smoke script.
- Do not publish local-only runtime data or internal proof bundles as public release proof.

## 7. Public/private boundary rules

Agent-facing docs must stay public-safe.

Do not add:

- private host paths
- private admin URLs
- secrets, tokens, keys, or customer data
- full chat logs
- local runtime databases, ledgers, or pairing state
- internal launch strategy or commercial calculations

Use public-safe replacements:

| Unsafe category | Public-safe wording |
| --- | --- |
| host-specific paths | `~/projects/focusa-demo` or `$PWD` |
| backend/admin URLs | `https://focusa.dev/support` or `https://install.focusa.dev/license` |
| full conversation dumps | bounded proof summaries or Evidence refs |
| license/customer records | public license terms and support path |

## 8. Software layout checklist for agents

Before code changes:

1. `git fetch origin`
2. `git status --short --branch`
3. Read this doc and the linked authority for the touched surface.
4. Identify the active bead/work item and exact Workpoint.
5. Record interaction mode and host renderer for UI work.
6. Make the smallest scoped change.
7. Run the required proof class.
8. Update bead notes, commit, and push for normal public code repos.

## 9. Helpful references

- Spec 135 delivery authority: `docs/135-series-current-manifest.md`
- Mission Canvas host/renderer machine contract: `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`
- Spec 135 agent directive: `docs/agent/spec135-implementation-acceleration-directive.md`
- README product overview: `README.md`
- Current CLI reference: `docs/current/CLI_REFERENCE_CURRENT.md`
- Public-surface guard: `scripts/guard-public-surface.sh`
- Cross-phase smoke: `tests/spec_cli_cross_phase_smoke_test.sh`
- Workpoint CLI implementation: `crates/focusa-cli/src/commands/workpoint.rs`
- Project command implementation: `crates/focusa-cli/src/commands/project.rs`
- API route implementations: `crates/focusa-api/src/routes/`
