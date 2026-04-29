# Changelog

Focusa is under active development. Versions below are current snapshot tags, not finished-product declarations.

## Unreleased — Spec92 error and empty-state envelopes

- Expanded CLI JSON failures and API non-JSON HTTP failures to include recovery-first Spec92 envelope fields.
- Added `docs/current/ERROR_EMPTY_STATES.md`.

## Unreleased — Spec92 safe cleanup command

- Added `focusa cleanup --safe` and `--dry-run` for recoverable cleanup of known generated residue while preserving `.beads/`, `data/`, and `target/`.

## Unreleased — Spec92 release prove command

- Added `focusa release prove --tag <tag>` release-proof orchestration with Spec90/91 validation, work-loop wiring proof, daemon health, Guardian scans, optional cargo gates, optional GitHub release lookup, and standard Spec92 envelope.

## Unreleased — daemon resilience and Pi holdover

- Hardened live `focusa-daemon` systemd restart policy with `Restart=always`, `RestartSec=1`, and disabled start-limit throttling.
- Added Pi extension in-session holdover/kickstart: tools remain available, daemon start/restart is attempted automatically, health probes accelerate, and SSE/state reconciliation resumes when daemon returns.
- Added `docs/current/DAEMON_RESILIENCE.md`.

## Unreleased — Spec92 agent status command

- Added `focusa status --agent` with live Workpoint, Work-loop, token-budget, cache, and daemon status envelope.

## Unreleased — Spec92 continue command

- Added full `focusa continue` command with work-loop writer governance, optional next-work selection, optional enable, Workpoint/Work-loop refresh, and standard Spec92 envelopes.
- Added `docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md`.

## Unreleased — Spec92 full doctor command center

- Added full `focusa doctor` command-center checks plus standard Spec92 output envelope; expanded token-budget visibility with `focusa tokens doctor` and `focusa tokens compact-plan`.

## Unreleased — Spec92 cache metadata doctor

- Added `POST /v1/telemetry/cache-metadata` and `GET /v1/telemetry/cache-metadata/status` for bounded cache metadata records.
- Added `focusa cache doctor` CLI visibility.
- Pi provider hook now emits cache metadata derived from bounded provider-request summaries.

## Unreleased — Spec92 token budget telemetry

- Added `POST /v1/telemetry/token-budget` and `GET /v1/telemetry/token-budget/status` for bounded Spec92 token-budget records.
- Added `focusa telemetry token-budget` CLI visibility.
- Pi `before_provider_request` hook now records bounded token-budget metadata to the daemon when available.
- Added `docs/current/EFFICIENCY_GUIDE.md`.

## Unreleased — Spec92 hook telemetry foundation

- Added first Spec92 implementation slice: missing Pi hook registrations for resources, agent/message/provider/tool execution, and session tree lifecycle events.
- Added bounded in-memory hook/token telemetry and exposed summary details through `focusa_tool_doctor`.
- Added current hook coverage docs at `docs/current/HOOK_COVERAGE.md`.

## Unreleased — Spec92 polish/prediction spec

- Added Spec92 for agent-first polish, missing Pi hooks, token efficiency, cache UX, command-center surfaces, Mac mission-control polish, and predictive-power accumulation.
- Documented how Focusa can accumulate predictive power using current event, Workpoint, metacognition, telemetry, CLT, ontology, and evidence frameworks.

## Unreleased — command documentation pass

- Added `docs/current/PRODUCTION_RELEASE_COMMANDS.md` with copy/paste release, daemon restart, GitHub proof, Mac app, and cleanup commands.
- Updated current runtime, validation, and Pi extension docs with concrete commands for current build verification.

## Unreleased — GitHub release and Mac app refresh

- Fixed GitHub CI/release clippy and spec-gate blockers.
- Updated the Mac menubar app to version `0.9.9` across package, Tauri, lockfile, and UI version surfaces.
- Updated the Mac menubar app to current Focusa core/API surfaces: health, ontology tool contracts, Workpoint, Work-loop, recent events, state dump, and live canvas events.
- Added Bun lockfile for the menubar app because the release workflow installs with Bun.
- Updated release-note examples to use the active release tag instead of stale `v0.2.10` paths.
- Restored pending-gated compaction auto-resume retry wiring required by strict spec gates.

## Unreleased — Spec91 live tool contract proof

- Added Spec91 for live runtime proof that the daemon ontology tool-contract projection matches the canonical registry.
- Added `scripts/prove-focusa-tool-contracts-live.mjs`.
- Added read-only `--safe-fixtures` live probes for representative Workpoint, Work-loop, tree/lineage, metacognition, and Focus State surfaces.
- Added live proof docs at `docs/current/LIVE_TOOL_CONTRACT_PROOF.md`.

## Unreleased — Spec90 tool contract foundation

- Added Spec90 for ontology-backed Focusa tool contracts and parity hardening.
- Added current machine-readable registry for all 43 `focusa_*` Pi tools.
- Added JSON registry projection and `GET /v1/ontology/tool-contracts` API projection.
- Added deterministic contract validation script.
- Upgraded `focusa_tool_doctor` to report contract coverage summary.
- Added current tool contract registry documentation.

## v0.9.2-dev — Focusa tool docs split

- Added one individual doc for each current `focusa_*` Pi tool under `docs/focusa-tools/tools/`.
- Added a root README table linking all 43 individual tool docs.
- Kept family docs as navigation pages only.
- Validation: root README links 43/43 current tools with no missing or extra links.

## v0.9.1-dev — Focused skills and tool-doc navigation

- Added companion Pi skills: `focusa-workpoint`, `focusa-metacognition`, `focusa-work-loop`, `focusa-cli-api`, `focusa-troubleshooting`, `focusa-docs-maintenance`.
- Added focused tool family docs and linked them from README.
- Validated Pi skill loader diagnostics for project, extension, and installed skill directories.

## v0.9.0-dev — Public runtime docs alignment

- Updated GitHub-facing README and high-risk public docs to describe the current Rust core/API/CLI/Pi runtime snapshot.
- Removed pre-implementation and finished/frozen wording from key public docs.
- Documented current Workpoint, evidence, metacognition, state hygiene, and tool-result envelope behavior.

## Earlier current-build milestones

- Spec88 Workpoint continuity: checkpoint/current/resume/drift/evidence-link APIs and Pi tools.
- Spec89 tool hardening: common tool result envelope, Workpoint-linked evidence, tool doctor, active-object resolver, work-loop UX, metacog quality gates, dedupe/hygiene surfaces, live release proof.
