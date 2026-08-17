<p align="center">
  <img src="docs/assets/readme/focusa-hero.svg" alt="Focusa Mission Deck illustrative visual proof" width="900">
</p>

<h1 align="center">Focusa</h1>

<p align="center">
  <strong>Keep AI coding agents on mission.</strong><br>
  <sub>Local-first proof, Workpoints, Evidence, and continuation for long-running coding agents.</sub>
</p>

<p align="center">
  <a href="https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml"><img alt="Release" src="https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml/badge.svg"></a>
  <img alt="Version" src="https://img.shields.io/badge/version-0.9.121--dev-blue">
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.91%2B-dea584?logo=rust">
  <img alt="License" src="https://img.shields.io/badge/license-BSL--1.1-orange">
  <img alt="Local first" src="https://img.shields.io/badge/local--first-proof%20layer-2b82ff">
</p>

Focusa is the local-first proof and continuity layer for AI coding agents. Current source version: `v0.9.157-dev`.

When a coding session gets long, context compacts, the mission drifts, proof gets buried, or another agent takes over, Focusa preserves the work as a proof-backed **Workpoint** with linked **Evidence** and a **next safe action**. The next agent should not have to guess from transcript memory.

### Current release architecture

- **Exact scope and worktrees:** authority is `project_root + continuity_id`; worktrees are verified working subpaths, not accidental new projects.
- **Daemon-native Silent Sessions:** durable background runs support observation, steering, pause/resume/restart, approvals, idempotency, and receipts.
- **Governed work loop and recovery:** one writer, canonical checkpoints, proactive compaction, cache-safe context, and automatic rollover after bounded transport exhaustion.
- **Mission Canvas and Work Rail:** scoped Work Surfaces, CRIST interviews, workspace artifacts, UIAI browser context, live refresh, connectors, software/domain projections, and adaptive generated UI.
- **Customer lifecycle:** idempotent install/repair, trusted Linux/macOS/Windows release assets, OTA/update rollback, and uninstall with user data preserved unless purge is explicit.
- **Agent-ready by construction:** every Focusa Pi tool has a runtime contract, generated machine schema, per-tool document, skill routing, and a progressively disclosed runbook.

Trajectory ladder: **HLT** (High-Level Trajectory) → **MLG** (Mid-Level Goal) → **STG** (Short-Term Goal) → **Waypoints** (concrete progress markers). Workpoint remains immediate action authority. The operator has authority; agents actively offer HLT-aligned Waypoints, STGs, and MLGs without silently changing the root goal.

## Install

[Release Install Postcard](docs/RELEASE_INSTALL_POSTCARD.md) — install, verify health, quickstart, and open Mission Deck.

```bash
curl -fsS https://install.focusa.dev/focusa | bash

Evaluation and entitlement issuance are authority-issued under Spec 152 (spec152): the bootstrapper resolves a signed, node-bound authority lease and never creates local evaluation state. Recovery posture: the same authority reissues device grants; no local credential persistence.
focusa start
focusa setup wizard --dry-run
focusa doctor
```

Interactive install offers the complete agent workflow and asks before installing missing Node.js/npm, Pi, the bundled Focusa Pi extension, and UIAI Engine. For unattended installation, pass `--install-dependencies --assume-yes`. Linux/amd64 can install the pinned checksummed UIAI binary; other platforms must set `UIAI_ENGINE_URL` to a healthy private endpoint. Focusa does not report full workflow readiness until Pi/plugin and `${UIAI_ENGINE_URL:-http://127.0.0.1:7456}/v1/health` checks pass.

Safe installer/update checks before changing anything:

```bash
focusa install --preflight --json
focusa update status --json
focusa update plan --json
focusa update apply --yes --allow-apply --dry-run false --json
focusa update scheduler --install --json  # Linux/root: verified two-minute timer
```

These commands report stale CLI/daemon/TUI surfaces, dependency prompts, update policy, rollback/safety gates, and notification routes. `update apply` promotes only checksum + Sigstore signature verified assets using an atomic staged rename and rollback journal; `scheduler --install` enables persistent two-minute verified refresh checks. Neither overwrites `.env`, license, or project data; daemon restart remains separately gated.

Prefer a local build while evaluating from source?

```bash
git clone https://github.com/Startempire-Wire/focusa.git
cd focusa
cargo build -p focusa-cli -p focusa-api
./target/debug/focusa help all
```

## Five-minute proof

Run a non-destructive continuity proof in any project folder:

```bash
focusa project discover --max-depth 2 --json
focusa first-mission --project-root "$PWD" --dry-run --json
focusa workpoint checkpoint \
  --project-root "$PWD" \
  --continuity-id demo-continuity \
  --mission "First Focusa proof" \
  --next-action "Resume from the Workpoint packet" \
  --json
focusa workpoint evidence-link \
  --target-ref tests \
  --result "cargo test -p focusa-cli passed" \
  --evidence-ref "test:cargo test -p focusa-cli" \
  --json
focusa workpoint resume \
  --project-root "$PWD" \
  --continuity-id demo-continuity \
  --copy-prompt
```

For the complete non-destructive Operator Preview path, run `scripts/demo-workpoint-happy-path.sh`.

Expected result: a typed resume packet with mission, scope, proof handle, and next action. If scope is unsafe or unclear, Focusa returns a blocked envelope instead of pretending the work is canonical.

## The Focusa you can use today

### 01 · Resume after compaction

![Focusa resume packet](docs/assets/readme/01-resume-after-compaction.svg)

A Workpoint gives the next agent a typed continuation contract instead of a transcript guess. It carries mission, scope, next action, and proof links.

```bash
focusa workpoint resume --project-root "$PWD" --continuity-id demo-continuity --copy-prompt
```

### 02 · Evidence that survives handoff

![Focusa evidence refs](docs/assets/readme/02-evidence-refs.svg)

Focusa stores proof as Evidence refs tied to the active Workpoint: tests, files, routes, screenshots, command output, and release checks.

```bash
focusa workpoint evidence-link \
  --target-ref crates/focusa-cli \
  --result "CLI smoke passed" \
  --evidence-ref "test:cross_phase_smoke_e2e" \
  --json
```

### 03 · Context Authority stops off-mission mutation

![Focusa Context Authority](docs/assets/readme/03-context-authority.svg)

Before risky changes, Focusa checks whether the current task, project, environment, and install role match the operator’s actual intent.

```bash
focusa action preflight \
  --current-ask "install Focusa locally" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root "$PWD" \
  --json
```

### 04 · Mission Deck in the terminal

![Focusa TUI Mission Deck](docs/assets/readme/04-mission-deck.svg)

The Mission Deck gives operators a compact cockpit: current focus, Workpoint, proof state, trajectory, health, and next safe action.

```bash
focusa deck --headless-self-test --json
# or
focusa tui --headless-self-test --json
```

### 05 · Pi extension integration

![Focusa Pi extension](docs/assets/readme/05-pi-extension.svg)

The Pi extension lets agents call Focusa directly, linking Workpoints, Evidence, trajectory, recovery tools, and bounded state without inventing a parallel memory system.

```text
focusa_workpoint_resume → focusa_trajectory_view → focusa_evidence_capture
```

### 06 · Local daemon, typed API

![Focusa local daemon](docs/assets/readme/06-local-api.svg)

Focusa runs beside the agent as a local Rust daemon. State stays on your machine or VPS. The HTTP API is typed, inspectable, and scriptable.

```bash
curl -fsS http://127.0.0.1:8787/v1/health
focusa status operator --json
focusa help migration
```

### 07 · Menubar cockpit preview

![Focusa menubar preview](docs/assets/readme/07-menubar-preview.svg)

The macOS/Tauri menubar is a preview surface. The primary Operator Preview today is daemon, CLI, TUI, Workpoints, Evidence, and Pi integration.

```bash
focusa pairing start --help
focusa device pair-start --device-name operator-macbook --platform macos
```

### 08 · Public proof, redacted by default

![Focusa public proof cards](docs/assets/readme/08-public-proof.svg)

Focusa can produce public-safe proof summaries without exposing local paths, full chat logs, license data, secrets, or personal operator state.

```bash
focusa release prove --tag v0.9.80-dev --fast --json
bash tests/spec_cli_cross_phase_smoke_test.sh
```

## Agent-first capability discovery

Focusa publishes one generated Agent Capability Descriptor V2 across Pi, MCP, OpenAI-compatible functions, CLI JSON help, REST, skills, and browser workflows. All 146 Focusa Pi tools are projected one-to-one into strict machine contracts and per-tool docs. Agents start with metadata—not 146 hot schemas—and progressively load only what the next action needs:

1. `focusa_agent_card` — interfaces, auth, families, capability count, and discovery entry points.
2. `focusa_tool_search` — ranked metadata by action, object, failure, or workflow.
3. `focusa_tool_describe` — one strict input/output/error contract.
4. `focusa_tool_graph` — bounded dependency and likely-next edges.
5. `focusa_tool_bundle` — one family, schemas deferred by default.

MCP exposes the generated callable catalog with pagination, `listChanged`, strict schemas, structured output, safety annotations, and scoped REST authority. UIAI/WebMCP capabilities are bound to one browser session and origin; page safety claims remain untrusted, mutations require governance, and results become evidence.

Machine contracts: [`docs/contracts/spec141/generated-capability-v2/`](docs/contracts/spec141/generated-capability-v2/) · Every Pi tool: [`docs/focusa-tools/tools/`](docs/focusa-tools/tools/) · Skills/runbooks: [`.pi/skills/`](.pi/skills/) · Agent fast start: [`docs/agent/01-focusa-agent-docs-index.md`](docs/agent/01-focusa-agent-docs-index.md) · Release gate: [`docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md`](docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md)

## Core commands

```bash
focusa help all
focusa project
focusa setup wizard --dry-run
focusa first-mission --project-root "$PWD" --dry-run --json
focusa status operator --json
focusa workpoint checkpoint --project-root "$PWD" --continuity-id demo --mission "Ship proof" --next-action "Run smoke" --json
focusa workpoint resume --project-root "$PWD" --continuity-id demo --copy-prompt
focusa silent --help
focusa update --help
focusa tui --headless-self-test
focusa cleanup --safe --project-root "$PWD" --dry-run --json
```

Migration help is built in:

```bash
focusa help migration
```

Deprecated aliases warn and point to canonical commands; for example, `focusa pair` routes users toward `focusa pairing start`.

## Architecture at a glance

- **`focusa-api`** — local Rust daemon and typed HTTP API.
- **`focusa-cli`** — operator and agent command surface.
- **`focusa-core`** — Workpoints, Evidence, reducers, runtime state, and persistence.
- **`focusa-tui`** — terminal Mission Deck.
- **`apps/pi-extension`** — Pi coding-agent integration.
- **`apps/menubar`** — preview macOS/Tauri cockpit.

## Proof and CI

Focusa’s public proof posture is command-first:

```bash
cargo test -p focusa-cli
cargo test -p focusa-api
cargo test -p focusa-core persistence_sqlite
bash tests/spec_cli_cross_phase_smoke_test.sh
```

The cross-phase smoke suite checks project dashboard commands, first-mission dry run, status aliases, Mission Deck self-test, scope rejection, uninstall keep flags, route parity, and daemon-global mutation blocking.

## Documentation

- Current CLI reference: [`docs/current/CLI_REFERENCE_CURRENT.md`](docs/current/CLI_REFERENCE_CURRENT.md)
- Production/release commands: [`docs/current/PRODUCTION_RELEASE_COMMANDS.md`](docs/current/PRODUCTION_RELEASE_COMMANDS.md)
- Troubleshooting: [`docs/current/TROUBLESHOOTING_CURRENT.md`](docs/current/TROUBLESHOOTING_CURRENT.md)
- Spec-first lifecycle and claim discipline: [`docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`](docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md)
- Complete Focusa tool documentation index: [`docs/focusa-tools/README.md`](docs/focusa-tools/README.md)
- Agent architecture, every-Pi-tool discovery, and skill/runbook onboarding: [`docs/agent/01-focusa-agent-docs-index.md`](docs/agent/01-focusa-agent-docs-index.md)
- Friendly onboarding walkthrough: [`docs/current/FOCUSA_FRIENDLY_ONBOARDING.md`](docs/current/FOCUSA_FRIENDLY_ONBOARDING.md)
- Install/update/rollback/uninstall policy: [`docs/current/INSTALLER_UPDATE_POLICY.md`](docs/current/INSTALLER_UPDATE_POLICY.md)
- Mission Canvas and generated-UI manifest: [`docs/135-series-current-manifest.md`](docs/135-series-current-manifest.md)
- Public shipped/planned status map: [`docs/PUBLIC_DOCS_SYNC.md`](docs/PUBLIC_DOCS_SYNC.md)
- Agent Awareness Quickstart: [`docs/current/AGENT_AWARENESS_QUICKSTART.md`](docs/current/AGENT_AWARENESS_QUICKSTART.md)
- Friendly Focusa Onboarding Q: [`docs/current/FOCUSA_FRIENDLY_ONBOARDING.md`](docs/current/FOCUSA_FRIENDLY_ONBOARDING.md)
- Tool Implementation-to-Spec Audit: [`docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md`](docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md)
- Model-Visible Awareness Surfaces: [`docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md`](docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md)
- Non-Pi Agent Focusa Usage: [`docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`](docs/current/NON_PI_AGENT_FOCUSA_USAGE.md)

## License

Focusa is source-available under the Business Source License 1.1. See [`LICENSE.md`](LICENSE.md).
# trigger Mon Aug 17 03:12:29 PDT 2026
# trigger Mon Aug 17 03:45:29 PDT 2026
