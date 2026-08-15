# Current CLI Reference

Generated from current `focusa --help` output for the present build.

```text
Focusa cognitive governance CLI

Usage: focusa [OPTIONS] <COMMAND>

Commands:
  help           Curated Focusa help and migration maps
  start          Start the Focusa daemon
  stop           Stop the Focusa daemon
  status         Show daemon status
  onboard        Deprecated alias; use `focusa setup wizard`
  first-mission  Guided project → Workpoint → proof → Mission Deck handoff
  setup          Setup wizard/init/doctor onboarding paths
  deck           Launch Focusa Mission Deck
  doctor         Run full agent-first doctor checks
  cleanup        Recoverable cleanup of generated residue
  install        Rust-orchestrated Focusa installer with Spec 132 terminal UI
  upgrade        Guarded upgrade path delegating through installer safety gates
  uninstall      Remove binaries/services while preserving user data by default
  install-service Install daemon service/LaunchAgent where supported
  continue       Resume governed continuous work and refresh state
  focus          Focus stack and Focus State operations
  stack          Show focus stack overview
  gate           Focus Gate (candidate management)
  action         Action authority / mutation preflight operations
  binary         Binary provenance and compatibility operations
  runtime        Runtime inventory and daemon hygiene operations
  memory         Memory operations
  ecs            ECS (reference store) operations
  env            Export env vars for proxy routing and environment contracts
  events         Event log inspection
  turns          Turn-level observability
  state          Dump full state (debug)
  clt            Context Lineage Tree
  lineage        Lineage API parity domain
  autonomy       Autonomy calibration
  awareness      Non-Pi agent awareness utility cards
  constitution   Agent Constitution
  telemetry      Cognitive telemetry
  rfm            Reliability Focus Mode
  release        Release proof workflow
  proposals      Proposal Resolution Engine
  predict        Prediction loop commands
  reflect        Reflection loop overlay
  metacognition  Metacognition command domain
  ontology       Ontology projections and vocab surfaces
  project        Project identity discovery and verification (Spec96)
  resource       ResourceMode / LowMem control (Spec96)
  trajectory     Per-project Trajectory Projection (Spec96)
  traverse       Bounded surgical traversal across Focusa surfaces (Spec96)
  skills         Agent skills
  thread         Thread operations (docs/38)
  export         Export training datasets (docs/20-21)
  contribute     Data contribution (docs/22)
  cache          Cache management (docs/18-19)
  working-set    Ontology working-set surface: scoped members, membership classes, freshness (Spec 49)
  workpoint      Spec88 Workpoint continuity operations
  tokens         API token management (docs/25)
  wrap           Wrap a harness CLI (Mode A proxy)
  help           Print this message or the help of the given subcommand(s)

Options:

## Context Authority commands

### `focusa action preflight`

Operational Context Gate. Produces `focusa.operational_context_gate.v1` with `verdict=allow|block|ask_operator`, conflicts, risk class, and safe alternative. Use before risky mutation.

```bash
focusa --json action preflight \
  --current-ask "initiate Phone Bridge pairing" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
```

### `focusa action classify-intent`

Intent Mode Gate. Produces `focusa.intent_mode_gate.v1`; planning/diagnosis prompts return `mutation_allowed=false`.

```bash
focusa --json action classify-intent --prompt "Maybe we can add a flag for install context"
```

### `focusa env contract show/init`

Environment Contract. Default path: `/etc/focusa/environment-contract.json`. Produces `focusa.environment_contract.v1`.

```bash
focusa --json env contract init \
  --role live_build_host \
  --project-root ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} \
  --owner wirebot \
  --machine-kind vps \
  --preferred-source local_repo_build
focusa --json env contract show
```

### `focusa runtime inventory`

Runtime/daemon hygiene. Produces `focusa.runtime_inventory.v1`.

```bash
focusa --json runtime inventory --owner wirebot
```

### `focusa binary inspect` / `focusa binary preflight-install`

Binary provenance and compatibility. Produces `focusa.binary_provenance.v1` and `focusa.binary_preflight.v1`.

```bash
focusa --json binary inspect /usr/local/bin/focusa
focusa --json binary preflight-install \
  --asset /tmp/focusa-release-asset \
  --target /usr/local/bin/focusa \
  --install-role live_build_host \
  --source github_release_asset
```

### `focusa pair --json` additions

Phone Bridge JSON includes `environment_contract`, `runtime_inventory`, and `action_preflight` in addition to transport diagnostics.

```text
      --json             Output in JSON format
      --config <CONFIG>  Config file path
      --verbose          Verbose output
      --quiet            Quiet mode — suppress non-essential output
  -h, --help             Print help
  -V, --version          Print version

```

## Current agent-first command groups

- `help all` / `help migration` — curated command inventory and old → new command map.
- `install` — Rust-owned installer orchestrator. Human interactive stderr may use the Spec 132 terminal UI; `--json` emits exactly one stdout JSON document; `--quiet` is silent except durable errors; `--no-animation` selects plain output. Supported installer UI controls: `FOCUSA_INSTALL_UI=auto|full|mono|reduced|plain`, `FOCUSA_INSTALL_SEED=<u64>`, `FOCUSA_REDUCE_MOTION=0|1`; `NO_COLOR`/`CLICOLOR=0` select monochrome animation on suitable terminals. Minimum animated size is 70×22; smaller, CI, non-TTY, and `TERM=dumb` fall back to plain. Pi extension integration is performed by Rust and reports integrated/skipped/warning truthfully.
- `project` — dashboard, discovery, selected-project convenience, status, creation, templates, and settings.
- `first-mission` — guided project → Workpoint → proof → Mission Deck handoff; dry-run is non-mutating.
- `setup wizard` — routes to First Mission; `setup init` / `setup doctor` are migration hints.
- `deck` — user-facing Mission Deck launcher.
- `onboard` — deprecated alias; use `focusa setup wizard`.
- `pairing start` — canonical pairing entry point; `pair` is a deprecated alias.
- `status --operator` — human session card with project, continuity, trajectory ladder (HLT → MLG → STG → Waypoints → Workpoint), proactive-planning doctrine, active Workpoint, next action, evidence count, drift hint, and health.
- `doctor` — full agent-first health/readiness check.
- `continue` — governed continuous-work resume and state refresh.
- `release prove` — safe release proof workflow, including optional GitHub release verification.
- `cleanup --safe` — recoverable cleanup of generated residue.
- `predict` — bounded prediction record/evaluate/capture-outcome/recent/stats loop with trajectory and ontology context.
- `tokens` and `cache` — token-budget and cache-metadata operational visibility.
- `project` — Project identity discovery/verification parity for `/v1/project/*`.
- `trajectory` — Trajectory view/define/assess/propose/checkpoint/resume parity for `/v1/trajectory/*`.
- `traverse` — bounded surgical traversal and tag verification parity for `/v1/traverse`.
- `resource` — ResourceMode/LowMem status and override parity for `/v1/resource/mode`.
- `focus update` — Focus State slot parity for `/v1/focus/update` (`--decision`, `--constraint`, `--failure`, `--intent`, `--current-focus`, `--next-step`, `--open-question`, `--recent-result`, `--note`); explicit `--project-root` values are CLI scope-checked before API calls.
- `workpoint` — scoped checkpoint/current/resume continuity operations; canonical checkpoint/resume accepts `--project-root` and `--continuity-id`; `workpoint resume --copy-prompt` prints a paste-ready continuation packet for non-Pi agents.
- `state snapshot` — create/recent/restore/diff/compare-latest snapshot parity for `/v1/focus/snapshots*`.
- `lineage extract` / `clt` / `turns` / `audit` — daemon-global advisory surfaces until Spec104 scoped APIs land; JSON includes `authority=daemon_global_advisory` and `canonical=false`.
- `memory set|reinforce` and `gate` mutations — blocked with advisory envelopes until scoped APIs exist; read/list surfaces are daemon-global advisory.
- `cleanup --safe` — resolver-scoped cleanup of generated repo-relative residue plus recoverable `/tmp` Focusa proof/log globs; unsafe roots return `CLI_SCOPE_REJECT` blocked envelopes.
- `context-cognition`, `hlt`, `call-stack`, `focus`, `workpoint`, `trajectory`, `project`, and `recover` — explicit project roots are checked with `ensure_project_root_scope_safe()` before scoped API calls.
- `lineage extract` — bounded lineage signal extraction for decision/constraint/risk compounding.
- `ecs list` / `ecs resolve` — print trajectory summaries for handles when API responses carry bounded trajectory context.
- Pi Focus Slice `EVIDENCE_HANDLES` lines include STG snippets when ontology evidence-handle context carries trajectory metadata.
- `metacognition capture` / `reflect` / `adjust` / `evaluate` — print trajectory summaries when learning packets are bound to the active trajectory.
- `metacognition recent-reflections` / `recent-adjustments` / `recent-evaluations` — read recent learning/evaluation packets.
- `awareness card --continuity-id` — non-Pi utility card injection with trajectory orientation and logical-session scoping.

### Release-current lifecycle and autonomous execution

- `focusa silent --help` exposes daemon-native Silent Session list/start/reopen/tail/send/interrupt/pause/resume/restart/kill/config/receipt/capabilities operations. Mutations use exact session/run/generation plus daemon-issued approval and idempotency fields; shell/tmux aliases are not authority.
- `focusa update --help` exposes trusted OTA inventory, policy, guarded apply, activation status, and rollback surfaces across CLI/daemon/TUI/Pi/menubar/installer release artifacts.
- `focusa uninstall --dry-run --keep-data` previews software removal with data preservation. The public bootstrapper preserves data by default; destructive removal requires explicit `--uninstall --purge-data`.
- `focusa tui --headless-self-test` provides structured non-TTY Mission Deck diagnostics.
- Mission Canvas, Work Rail, connectors, generated UI, and all Focusa Pi tools are discovered through the generated Agent Card/Spec141 registries rather than an invented parallel CLI hierarchy.

Machine-readable CLI bindings live in `docs/contracts/spec141/generated-capability-v2/cli-commands.json`; every Focusa Pi tool is separately projected in `pi-tools.json` and documented under `docs/focusa-tools/tools/`.

## Command hierarchy migration

`focusa help migration` is the canonical old → new map. Current aliases warn when executed instead of silently behaving as separate command families.

| Old command | Canonical command |
| --- | --- |
| `focusa init --quickstart --project-root <repo> --json` | Canonical existing-repository binding; creates/verifies `.focusa-project.json` and returns one next action |
| `focusa project new` | New-project creation only; not an existing-repository binding alias |
| `focusa onboard` | `focusa setup wizard` |
| `focusa preflight` | `focusa setup doctor` / future `focusa quality preflight` |
| `focusa pair` | `focusa pairing start` |
| `focusa pairing-doctor` | `focusa pairing doctor` |
| `focusa pairing-transport` | `focusa pairing transport` |
| `focusa pairing-wizard` | `focusa pairing wizard` |
| `focusa stack` | `focusa focus stack` |

Lifecycle grouping is documented as planned: `focusa lifecycle start|stop|install|uninstall|upgrade|install-service|codesign|doctor`.

## Launch-hardening notes

- `focusa stop` distinguishes `stopped`, `already_stopped`, and failed timeout states; `--json` returns a structured status envelope.
- `/v1/telemetry/snapshot` is the route-parity endpoint for TUI/menubar runtime snapshots.
- Daemon startup removes expired, incomplete pairing rooms before rehydrating active pairing state from SQLite.
- `FOCUSA_NO_DECAY_TICK=1` is supported as a runtime escape hatch to disable the memory decay tick; prefer ResourceMode/LowMem controls for normal pressure management.
- Cross-phase CLI regressions are covered by `tests/spec_cli_cross_phase_smoke_test.sh` and the cargo wrapper `cross_phase_smoke_e2e`.

## Common examples

```bash
focusa help all
focusa help migration
focusa project
focusa first-mission --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --dry-run --json
focusa setup wizard --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --dry-run --json
focusa pairing start --help
focusa status --agent
focusa status --operator
focusa onboard --agent manual  # deprecated alias; use focusa setup wizard
focusa doctor --json
focusa awareness card --adapter-id openclaw --workspace-id wirebot --agent-id wirebot --operator-id verious.smith --continuity-id cont-1
focusa continue --json
focusa release prove --tag v0.9.25-dev --fast --github --json
focusa predict record --prediction-type next_action_success --predicted-outcome completed --confidence 0.8 --recommended-action "continue" --why "bounded evidence" --ontology-context '{"object_refs":["Workpoint"],"tool_refs":["focusa_workpoint_resume"]}'
focusa predict recent --limit 20
focusa predict evaluate <prediction_id> --actual-outcome completed --score 1.0
focusa predict capture-outcome --prediction-type next_action_success --actual-outcome completed --score 1.0 --ontology-context '{"object_refs":["Workpoint"],"tool_refs":["focusa_workpoint_resume"]}'
focusa predict stats
focusa tokens doctor
focusa cache doctor
focusa focus update --decision "Use scoped Workpoints for project continuation." --json
focusa workpoint checkpoint --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --mission "Audit tools" --next-action "Run parity gates" --json
focusa workpoint current --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --json
focusa workpoint resume --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --json
focusa workpoint resume --copy-prompt
focusa cleanup --safe --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --dry-run --json
focusa memory set foo=bar --json  # blocked until scoped memory API exists
focusa gate pin candidate-1 --json # blocked until scoped Focus Gate API exists
focusa state snapshot recent --limit 5 --json
focusa state snapshot compare-latest --snapshot-reason "pre-risk check" --json
focusa lineage extract --max-candidates 12 --json
focusa metacognition recent-reflections --limit 5 --json
focusa metacognition recent-adjustments --limit 5 --json
focusa metacognition recent-evaluations --limit 5 --json
focusa project identity --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
focusa project card --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --current-ask "Choose the next evidence-backed step" --json
focusa project card-outcome --algorithm-run-id <algorithm_run_id> --actual-outcome "completed" --score 1.0 --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --evidence-ref "test:pass" --json
# API/Pi outcome payloads can include task_timing + token_usage; Pi auto-populates elapsed HH:MM:SS and token counts when omitted.
focusa project session-transfer --action save --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --current-ask "Save current work like a game save" --json
focusa project session-transfer --action continue --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
focusa project verify --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --project-id focusa --json
focusa project trajectory-guard --action verify --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
focusa project trajectory-guard --action migrate --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --confirm --idempotency-key marker-migrate-1 --json
focusa project bootstrap preview --project-root /absolute/project --project-id project --canonical-name "Project" --continuity-id project-main --idempotency-key bootstrap-project --json
focusa project bootstrap apply --project-root /absolute/project --project-id project --canonical-name "Project" --continuity-id project-main --idempotency-key bootstrap-project --hlt "Ship the verified project" --hlt-confirmed --specification-ref docs/01-project-spec.md --acceptance "First Workpoint is active" --current-state "Empty project" --desired-end-state "Disciplined project ready" --confirm --json
focusa project bootstrap status --project-root /absolute/project --json
focusa project bootstrap repair --project-root /absolute/project --project-id project --canonical-name "Project" --continuity-id project-main --idempotency-key bootstrap-project --repair-action rollback --confirm --json
focusa project genesis start --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --idempotency-key genesis-1 --hlt "Ship the verified project" --hlt-confirmed --specification-ref docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md --acceptance "First Workpoint is active" --current-state "Genesis incomplete" --desired-end-state "Project ready" --allow-task-decomposition --json
focusa project genesis status --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
focusa project genesis resume --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --idempotency-key genesis-1 --json
focusa project genesis commit --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --idempotency-key genesis-1 --hlt "Ship the verified project" --hlt-confirmed --specification-ref docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md --acceptance "First Workpoint is active" --current-state "Genesis incomplete" --desired-end-state "Project ready" --allow-task-decomposition --confirm --json
focusa temporal status --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --json
focusa temporal commit --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --idempotency-key deadline-1 --claim-id release-deadline --kind external_commitment --subject-ref release --target-at 2026-08-01T17:00:00Z --timezone America/Los_Angeles --source operator --operator-confirmed --confidence verified --evidence-ref contract:release-date --confirm --json
focusa temporal observe --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --idempotency-key build-run-1 --phase build --duration-ms 120000 --evidence-ref run:123 --json
focusa temporal forecast --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --phase build --json
focusa temporal preflight --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --json
focusa trajectory view --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --mode summary --json
focusa trajectory history --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --limit 50 --json
focusa trajectory query --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --level waypoint --as-of 2026-07-25T12:00:00Z --json
focusa trajectory define-goal --long-term-goal "Ship Spec96" --desired-end-state "All Spec96 gates pass" --mid-level-goal "Close release blockers" --short-term-goal "Run current validation gates" --waypoint "CLI/API parity proof" --waypoint "Public docs proof" --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
focusa traverse read --surface workpoints --selector current --limit 1 --json
focusa traverse verify-tags --surface workpoints --tag focusa://workpoints/current/item/example --json
focusa resource status --json
focusa resource activate-lowmem --reason "operator requested LowMem" --json
focusa export status --json
focusa export sft --output /tmp/focusa-sft.jsonl --dry-run --explain --json
focusa export preference --output /tmp/focusa-pref.parquet --format parquet --json
```

Export JSON now includes `quality_gates`, per-record `provenance`/`eligibility`, top-level `quality_summary`, `redaction_summary`, and manifest copies of the same quality metadata.

Use `--json` for machine-readable output where supported.

## `device` — Mac menubar OAuth-like device pairing (focusa-ui0y)

See [`docs/53-focusa-device-pairing-spec.md`](../53-focusa-device-pairing-spec.md) for the full architecture.

```text
focusa device pair-start     Generate an 8-char FOCUS-XXXX-XXXX code
focusa device pair qr        Shortcut: pair-start + print pair_url for QR
focusa device pair-complete  VPS-side completion; mints 30-day token
focusa device pair-status    Check pairing status (by code or device_id)
focusa device pair-list      List paired devices for a host
focusa device pair-revoke    Revoke a paired device
```

**`FOCUSA_PAIRING_URL` env var** (optional): the public URL the operator's
phone will hit (e.g. `https://focusa-conn.verious.net`). When unset, the
daemon uses `daemon_base_url` (default `http://127.0.0.1:8787`). This is
what makes pairing portable across public VPS hosts.

## `working-set` — ontology working-set parity surface (Spec 49)

Scoped to the active project workstream (same resolution as work-loop). Mirrors the REST surface at `/v1/ontology/working-set` and `/v1/ontology/actions` (refresh_working_set).

```text
focusa working-set status                 Show scoped members, membership class, freshness, score, verification handles
focusa working-set status --ask "..."     Filter by ask text
focusa working-set status --slice-type object --limit 10
focusa working-set refresh --subject <ref>  Propose refreshing membership for a target ref (idempotent per subject)
```

Requires a resolved project workstream (project root + continuity id — same
boundary as `focusa work-loop`). Output is advisory (`canonical: false`);
membership changes land as typed proposals via the ontology action route.
