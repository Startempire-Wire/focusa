# Current CLI Reference

Generated from current `focusa --help` output for the present build.

```text
Focusa cognitive governance CLI

Usage: focusa [OPTIONS] <COMMAND>

Commands:
  start          Start the Focusa daemon
  stop           Stop the Focusa daemon
  status         Show daemon status
  doctor         Run full agent-first doctor checks
  cleanup        Recoverable cleanup of generated residue
  continue       Resume governed continuous work and refresh state
  focus          Focus stack and Focus State operations
  stack          Show focus stack overview
  gate           Focus Gate (candidate management)
  memory         Memory operations
  ecs            ECS (reference store) operations
  env            Export env vars for proxy routing
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
  workpoint      Spec88 Workpoint continuity operations
  tokens         API token management (docs/25)
  wrap           Wrap a harness CLI (Mode A proxy)
  help           Print this message or the help of the given subcommand(s)

Options:
      --json             Output in JSON format
      --config <CONFIG>  Config file path
      --verbose          Verbose output
      --quiet            Quiet mode — suppress non-essential output
  -h, --help             Print help
  -V, --version          Print version
```

## Current agent-first command groups

- `doctor` — full agent-first health/readiness check.
- `continue` — governed continuous-work resume and state refresh.
- `release prove` — safe release proof workflow, including optional GitHub release verification.
- `cleanup --safe` — recoverable cleanup of generated residue.
- `predict` — bounded prediction record/evaluate/recent/stats loop.
- `tokens` and `cache` — token-budget and cache-metadata operational visibility.
- `project` — Project identity discovery/verification parity for `/v1/project/*`.
- `trajectory` — Trajectory view/define/assess/propose/checkpoint/resume parity for `/v1/trajectory/*`.
- `traverse` — bounded surgical traversal and tag verification parity for `/v1/traverse`.
- `resource` — ResourceMode/LowMem status and override parity for `/v1/resource/mode`.
- `focus update` — Focus State slot parity for `/v1/focus/update` (`--decision`, `--constraint`, `--failure`, `--intent`, `--current-focus`, `--next-step`, `--open-question`, `--recent-result`, `--note`).
- `workpoint` — scoped checkpoint/current/resume continuity operations; canonical checkpoint/resume accepts `--project-root` and `--continuity-id`.
- `state snapshot` — create/recent/restore/diff/compare-latest snapshot parity for `/v1/focus/snapshots*`.
- `lineage extract` — bounded lineage signal extraction for decision/constraint/risk compounding.
- `metacognition recent-reflections` / `recent-adjustments` — read recent learning packets.
- `awareness card --continuity-id` — non-Pi utility card injection with trajectory orientation and logical-session scoping.

## Common examples

```bash
focusa status --agent
focusa doctor --json
focusa awareness card --adapter-id openclaw --workspace-id wirebot --agent-id wirebot --operator-id verious.smith --continuity-id cont-1
focusa continue --json
focusa release prove --tag v0.9.12-dev --fast --github --json
focusa predict record --prediction-type next_action_success --predicted-outcome completed --confidence 0.8 --recommended-action "continue" --why "bounded evidence"
focusa predict recent --limit 20
focusa predict evaluate <prediction_id> --actual-outcome completed --score 1.0
focusa predict stats
focusa tokens doctor
focusa cache doctor
focusa focus update --decision "Use scoped Workpoints for project continuation." --json
focusa workpoint checkpoint --project-root /home/wirebot/focusa --continuity-id cont-1 --mission "Audit tools" --next-action "Run parity gates" --json
focusa workpoint current --project-root /home/wirebot/focusa --continuity-id cont-1 --json
focusa workpoint resume --project-root /home/wirebot/focusa --continuity-id cont-1 --json
focusa state snapshot recent --limit 5 --json
focusa state snapshot compare-latest --snapshot-reason "pre-risk check" --json
focusa lineage extract --max-candidates 12 --json
focusa metacognition recent-reflections --limit 5 --json
focusa metacognition recent-adjustments --limit 5 --json
focusa project identity --project-root /home/wirebot/focusa --json
focusa project verify --project-root /home/wirebot/focusa --project-id focusa --json
focusa trajectory view --project-root /home/wirebot/focusa --mode summary --json
focusa trajectory define-goal --long-term-goal "Ship Spec96" --desired-end-state "All Spec96 gates pass" --project-root /home/wirebot/focusa --json
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
