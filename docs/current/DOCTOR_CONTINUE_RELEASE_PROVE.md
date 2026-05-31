# Doctor, Continue, and Release Proof Commands

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

This page documents the current Spec92 command-center surfaces.

## Doctor

```bash
focusa doctor
focusa --json doctor
focusa doctor security
focusa --json doctor security
```

`focusa doctor` checks daemon health, daemon executable path, API/capability inventory, Spec90 tool contracts, Spec91 proof harness presence, Pi skill paths, Workpoint canonicality, Work-loop state, token telemetry, cache metadata, Mac app package presence, release docs, and Guardian scanner presence. `/v1/doctor` now exposes an `api_cli_parity` block naming the shared CLI/API check set, recovery commands, and expected status fields so agents can compare API and CLI doctor behavior without guessing.

`focusa doctor security` reports API resource/security posture: non-loopback `FOCUSA_AUTH_TOKEN` boundary, request body cap, mutation route rate limit, JSON shape guard, and reverse-proxy rate-limit guidance. It is a deployment/API security report, not a license-plan or entitlement command.

## Agent status

```bash
focusa status --agent
focusa --json status --agent
focusa status --operator
focusa --json status --operator
```

`focusa status --agent` returns a standard Spec92 envelope combining live `/v1/status`, Workpoint, Work-loop, token-budget, and cache metadata surfaces. `focusa status --operator` returns the current Operator Preview session card: project, continuity, trajectory, active Workpoint, next action, evidence count, drift hint, and health.

## Continue

```bash
focusa continue
focusa continue --parent-work-item-id focusa-bzwt
focusa continue --enable --parent-work-item-id focusa-bzwt
focusa --json continue --reason "resume after compaction"
```

`focusa continue` is a governed write command. It uses the work-loop writer id header, refreshes Workpoint and Work-loop state, optionally selects the next ready Beads subtask, resumes continuous work, and returns a standard Spec92 envelope.

## Standard output envelope

Human output includes:

```text
Status: <completed|watch|degraded|blocked>
Summary: <one sentence>
Next action: <exact next action>
Why: <short explanation>
Command: <copyable command>
Recovery: <copyable fallback>
Evidence: <refs/handles>
Docs: <paths>
```

JSON output includes:

```json
{
  "status": "completed",
  "summary": "...",
  "next_action": "...",
  "why": "...",
  "commands": [],
  "recovery": [],
  "evidence_refs": [],
  "docs": [],
  "warnings": [],
  "details": {}
}
```

## Release proof

Release proof command:

```bash
focusa release prove --tag <tag>
focusa release prove --tag <tag> --fast
focusa release prove --tag <tag> --github
```

`focusa release prove` orchestrates the standard validation gates: git state, Spec90 contract validation, Spec91 safe live proof, work-loop auto-continue wiring test, daemon health, Guardian scans, and optionally cargo workspace gates plus GitHub release asset lookup.


## Cleanup

```bash
focusa cleanup --safe --dry-run
focusa cleanup --safe
```

`focusa cleanup --safe` moves known generated residue to `${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-*` and preserves `.beads/`, `data/`, and `target/`.
