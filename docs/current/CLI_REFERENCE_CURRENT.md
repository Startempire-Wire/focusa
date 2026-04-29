# Current CLI Reference

Generated from current `focusa --help` output for the present build.

```text
Focusa cognitive governance CLI

Usage: focusa [OPTIONS] <COMMAND>

Commands:
  start          Start the Focusa daemon
  stop           Stop the Focusa daemon
  status         Show daemon status
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
  proposals      Proposal Resolution Engine
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

## Common examples

```bash
focusa status
focusa workpoint current
focusa workpoint resume
focusa ontology primitives
focusa ontology world
focusa metacognition --help
focusa tokens --help
```

Use `--json` for machine-readable output where supported.
