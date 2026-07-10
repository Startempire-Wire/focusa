# Focusa Agent Docs Index

This is the bounded, public-safe starting point for AI agents working in the Focusa repo. Use it before broad code changes or after context loss.

## 1. What Focusa is

Focusa is the local-first proof and continuity layer for AI coding agents. It keeps long-running work attached to a typed Workpoint, linked Evidence, and a next safe action so agents do not rely on chat tail memory.

## 2. Architecture map

| Layer | Purpose | Key locations |
|---|---|---|
| CLI | Operator and agent command surface | `crates/focusa-cli/src/commands/` |
| API daemon | Local typed HTTP API | `crates/focusa-api/src/routes/` |
| Core | reducers, Workpoints, Evidence, runtime state, persistence | `crates/focusa-core/src/` |
| TUI / Mission Deck | terminal cockpit | `crates/focusa-tui/` |
| Pi extension | Pi tool bridge | `apps/pi-extension/` |
| Menubar preview | macOS/Tauri cockpit preview | `apps/menubar/` |
| Public docs | current reference and specs | `docs/`, `docs/current/` |

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

## 5. Workpoints, Evidence, and Trajectory

- **Workpoint** is the immediate continuation contract: mission, scope, current action, next action, blockers, and proof handles.
- **Evidence** is proof linked to the active Workpoint: tests, files, route checks, screenshots, command output, or release checks.
- **Trajectory** is advisory north-star context: long-term direction and current gap. It orients work but does not override a canonical Workpoint.
- **Context Authority** decides whether a proposed action matches the task, project, environment, and install role.

Never treat transcript tail as canonical authority when a Workpoint or scope gate is available.

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
|---|---|
| host-specific paths | `~/projects/focusa-demo` or `$PWD` |
| backend/admin URLs | `https://focusa.dev/support` or `https://install.focusa.dev/license` |
| full conversation dumps | bounded proof summaries or Evidence refs |
| license/customer records | public license terms and support path |

## 8. Software layout checklist for agents

Before code changes:

1. `git fetch origin`
2. `git status --short --branch`
3. Read this doc and the linked spec/current reference for the touched surface.
4. Identify the active bead/work item.
5. Make the smallest scoped change.
6. Run focused proof.
7. Update bead notes, commit, and push for normal public code repos.

## 9. Helpful references

- README product overview: `README.md`
- Current CLI reference: `docs/current/CLI_REFERENCE_CURRENT.md`
- Public-surface guard: `scripts/guard-public-surface.sh`
- Cross-phase smoke: `tests/spec_cli_cross_phase_smoke_test.sh`
- Workpoint CLI implementation: `crates/focusa-cli/src/commands/workpoint.rs`
- Project command implementation: `crates/focusa-cli/src/commands/project.rs`
- API route implementations: `crates/focusa-api/src/routes/`
