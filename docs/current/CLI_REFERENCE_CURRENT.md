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

### Project bootstrap lifecycle

- `focusa project bootstrap preview --project-root <absolute-path>` — computes the local-only, idempotent bootstrap transaction without writing.
- `focusa project bootstrap apply --project-root <absolute-path>` — applies only the previewed bounded transaction; it never creates or changes a remote implicitly.
- `focusa project bootstrap status --project-root <absolute-path>` — reports marker, project identity, Genesis, Beads, and verification state without mutation.
- `focusa project bootstrap repair --project-root <absolute-path>` — repairs only transaction-owned bootstrap artifacts while preserving operator choices and reporting rollback evidence.

### Project Genesis lifecycle

- `focusa project genesis start --project-root <absolute-path> --continuity-id <id> --idempotency-key <key>` — inventories authority and stages Genesis; missing High-Level Trajectory intent enters an explicit impasse instead of inventing one.
- `focusa project genesis resume --project-root <absolute-path> --continuity-id <id> --idempotency-key <key>` — resumes the same bounded, idempotent Genesis transaction.
- `focusa project genesis status --project-root <absolute-path>` — reads the durable Genesis packet without mutation.
- `focusa project genesis commit --project-root <absolute-path> --continuity-id <id> --idempotency-key <key> --confirm` — atomically commits the confirmed Trajectory, first Workpoint, coordination state, and readiness receipt.

### Release-current lifecycle and autonomous execution

- `focusa silent --help` exposes daemon-native Silent Session list/start/reopen/tail/send/interrupt/pause/resume/restart/kill/config/receipt/capabilities operations. Mutations use exact session/run/generation plus daemon-issued approval and idempotency fields; shell/tmux aliases are not authority.
- `focusa update --help` exposes trusted OTA inventory, policy, guarded apply, activation status, and rollback surfaces across CLI/daemon/TUI/session-runner/Pi/agent-context/menubar/installer release artifacts.
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
focusa trajectory view --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --mode summary --json
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


## license — activation and entitlement operations

Generated from `focusa license --help` and subcommand help on this build. Paid fast-path: `focusa license activate-flow --license-key <KEY>` redeems an already-paid key in ONE request (no email verification, no menu, no polling) for ANY product in the authority registry; retry is idempotent.

### `license (top)`

```text
License activation and entitlement operations (Spec92 §5.2)

Usage: focusa license [OPTIONS] <COMMAND>

Commands:
  activate-flow  Interactive authority activation (Spec 152E §14.1): one shared flow renders email → verify → offer → checkout/poll → key/lease, existing key, Evaluation (Spec 172 limited-access overlay), resume, cancel, timeout, and recovery. Never accepts card data and never self-issues
  activate       Activate a Focusa license key. Saves the local license state file
  status         Show current license status (mode, status, features, offline-valid-until)
  deactivate     Deactivate the current license. The local file is removed
  doctor         Run a self-check of the local license file and remote registry reachability
  check-feature  Check whether a specific feature is enabled by the current license
  preflight      Fast preflight against the canonical entitlement decision (Spec 152F §6 chokepoint 4): renders base/premium/recovery reason and next action from the authority snapshot only, and exits nonzero when the target gate would deny. Never self-issues a grant
  devmode-full   End-to-end license provisioning harness. Generates a fresh test key, validates it against the registry (dev_mode is acceptable for operator testing but downgrades commercial_use to false), writes license.json / license_authority.json / license_receipt.json, round-trips the files through the daemon parser, and reports the result. Use this to verify the full provisioning pipeline before the first real transaction
  refresh        Re-validate the current license against the registry and update the local file. Picks up revoke / refund / expire changes that happened on the registry side since the last validation
  watch          Watch the local license file and the registry. When the registry returns a new state, the local file is updated and a notification is printed. Use this as a long-running sidecar after a purchase so refunds and revokes propagate within the poll interval

Options:
      --json                       Output in JSON format
      --config <CONFIG>            Config file path
      --verbose                    Verbose output
      --quiet                      Quiet mode — suppress non-essential output
      --lifecycle-action <ACTION>  Inspect, preview, confirm, apply, or recover a lifecycle transaction [possible values: inspect, preview, confirm, apply, resume, repair, rerun, rollback, uninstall, purge]
      --confirm                    Confirm the mutation selected by --lifecycle-action
      --confirm-purge-data         Separately confirm user-data deletion for a lifecycle purge
  -h, --help                       Print help
  -V, --version                    Print version
```

### `license activate-flow`

```text
Interactive authority activation (Spec 152E §14.1): one shared flow renders email → verify → offer → checkout/poll → key/lease, existing key, Evaluation (Spec 172 limited-access overlay), resume, cancel, timeout, and recovery. Never accepts card data and never self-issues

Usage: focusa license activate-flow [OPTIONS]

Options:
      --json                       Output in JSON format
      --registry <URL>             Override the registry URL (default: https://wpuiai.com)
      --config <CONFIG>            Config file path
      --resume <REGISTRATION_ID>   Resume a persisted activation registration (bounded poll continuation). The poll credential is re-supplied from the protected store; the snapshot never contains it
      --email <EMAIL>              Explicit email for a new activation (prompted interactively otherwise). The email only creates a pending attempt; verification is always required before any promotion
      --verbose                    Verbose output
      --poll-timeout <SECONDS>     Bounded poll wall-clock timeout in seconds (default: the registration poll budget governs; timeout settles fail-closed via cancel → recovery_only)
      --quiet                      Quiet mode — suppress non-essential output
      --agent                      Agent/JSON protocol (Spec 152E §14.2): non-interactive, never prompts, never invents an email, verification code, consent, payment confirmation, or license. Returns typed human-action envelopes with a resumable registration handle; requires --email for a new attempt or --resume for a bounded poll continuation
      --lifecycle-action <ACTION>  Inspect, preview, confirm, apply, or recover a lifecycle transaction [possible values: inspect, preview, confirm, apply, resume, repair, rerun, rollback, uninstall, purge]
      --confirm                    Confirm the mutation selected by --lifecycle-action
      --reveal-key                 Customer-controlled key reveal opt-in (agent mode): full key output is masked by default; revealing the one-time key requires BOTH this flag and --confirm-reveal
      --confirm-purge-data         Separately confirm user-data deletion for a lifecycle purge
      --license-key <KEY>          Paid fast-path (all products through the license authority): redeem an already-paid license key in ONE request — no email verification, no offer menu, no polling. The server verifies the key, promotes the account, binds this device (verbatim node identity), and returns a root-signed lease that is persisted locally. Works for every product in the authority registry (Focusa, UIAI Engine, bundles)
      --confirm-reveal             Explicit confirmation for the customer-controlled key reveal (agent mode). Without it the key stays masked
  -h, --help                       Print help
  -V, --version                    Print version
```

### `license activate`

```text
Activate a Focusa license key. Saves the local license state file

Usage: focusa license activate [OPTIONS] <KEY>

Arguments:
  <KEY>  The license key (focusa_live_xxxxx or uiai_live_xxxxx)

Options:
      --json                       Output in JSON format
      --persist-key                Persist the raw key in the local file (off-spec; default is prefix only)
      --config <CONFIG>            Config file path
      --registry <URL>             Override the registry URL (default: https://install.focusa.dev)
      --verbose                    Verbose output
      --quiet                      Quiet mode — suppress non-essential output
      --lifecycle-action <ACTION>  Inspect, preview, confirm, apply, or recover a lifecycle transaction [possible values: inspect, preview, confirm, apply, resume, repair, rerun, rollback, uninstall, purge]
      --confirm                    Confirm the mutation selected by --lifecycle-action
      --confirm-purge-data         Separately confirm user-data deletion for a lifecycle purge
  -h, --help                       Print help
  -V, --version                    Print version
```

### `license status`

```text
Show current license status (mode, status, features, offline-valid-until)

Usage: focusa license status [OPTIONS]

Options:
      --json                       Output in JSON format
      --config <CONFIG>            Config file path
      --verbose                    Verbose output
      --quiet                      Quiet mode — suppress non-essential output
      --lifecycle-action <ACTION>  Inspect, preview, confirm, apply, or recover a lifecycle transaction [possible values: inspect, preview, confirm, apply, resume, repair, rerun, rollback, uninstall, purge]
      --confirm                    Confirm the mutation selected by --lifecycle-action
      --confirm-purge-data         Separately confirm user-data deletion for a lifecycle purge
  -h, --help                       Print help
  -V, --version                    Print version
```

### `license refresh`

```text
Re-validate the current license against the registry and update the local file. Picks up revoke / refund / expire changes that happened on the registry side since the last validation

Usage: focusa license refresh [OPTIONS]

Options:
      --json                       Output in JSON format
      --registry <URL>             Override the registry URL (default: https://wpuiai.com)
      --config <CONFIG>            Config file path
      --raw-key <KEY>              Persist the raw key from --raw-key in the local file (off-spec)
      --require-real               Set FOCUSA_REQUIRE_REAL_LICENSE=1 for this run (refuse dev_mode)
      --verbose                    Verbose output
      --quiet                      Quiet mode — suppress non-essential output
      --lifecycle-action <ACTION>  Inspect, preview, confirm, apply, or recover a lifecycle transaction [possible values: inspect, preview, confirm, apply, resume, repair, rerun, rollback, uninstall, purge]
      --confirm                    Confirm the mutation selected by --lifecycle-action
      --confirm-purge-data         Separately confirm user-data deletion for a lifecycle purge
  -h, --help                       Print help
  -V, --version                    Print version
```

