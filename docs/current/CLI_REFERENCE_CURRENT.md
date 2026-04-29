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
  focus          Focus stack operations
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
  constitution   Agent Constitution
  telemetry      Cognitive telemetry
  rfm            Reliability Focus Mode
  release        Release proof orchestration
  proposals      Proposal Resolution Engine
  predict        Prediction loop commands
  reflect        Reflection loop overlay
  metacognition  Metacognition command domain
  ontology       Ontology projections and vocab surfaces
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
- `release prove` — safe release proof orchestration, including optional GitHub release verification.
- `cleanup --safe` — recoverable cleanup of generated residue.
- `predict` — bounded prediction record/evaluate/recent/stats loop.
- `tokens` and `cache` — token-budget and cache-metadata operational visibility.
- `workpoint` — checkpoint/current/resume continuity operations.

## Common examples

```bash
focusa status --agent
focusa doctor --json
focusa continue --json
focusa release prove --tag v0.9.11-dev --fast --github --json
focusa predict record --prediction-type next_action_success --predicted-outcome completed --confidence 0.8 --recommended-action "continue" --why "bounded evidence"
focusa predict recent --limit 20
focusa predict evaluate <prediction_id> --actual-outcome completed --score 1.0
focusa predict stats
focusa tokens doctor
focusa cache doctor
focusa workpoint current --json
focusa workpoint resume --json
```

Use `--json` for machine-readable output where supported.
