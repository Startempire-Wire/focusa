# Context Authority Architecture Translation Workorder Spec

**Status:** architecture translation / implementation workorder source
**Incident source:** `docs/current/INCIDENT_PHONE_BRIDGE_CONTEXT_AUTHORITY_2026-06-11.md`
**Project:** Focusa
**Project root:** `${FOCUSA_PROJECT_ROOT:-<focusa-repo>}`
**Purpose:** translate every architecture need exposed by the Phone Bridge context-authority incident into Focusa-specific software work, bead decomposition, enforcement points, and acceptance tests.

**Implementation status:** implemented through `dc9a335` and documented in `docs/current/CONTEXT_AUTHORITY_CURRENT.md`. Child beads `focusa-9o3v.1`–`.10` and parent `focusa-9o3v` were closed after validation.

---

## 1. Thesis

The incident exposed a multi-layer architecture failure: Focusa had context, but did not enforce it at the moment of action.

The correct fix is not one flag. It is a set of cooperating architecture components:

1. verified environment facts,
2. verbatim HLT history,
3. Workpoint/current ask continuity,
4. mutation-time action preflight,
5. task-substitution detection,
6. daemon/binary provenance,
7. planning-vs-implementation discipline,
8. incident replay tests.

The primary invariant:

> A Focusa agent must not perform a risky mutation until current ask, Workpoint, environment facts, and HLT/history authority have been reconciled into an allow/block/ask preflight verdict.

---

## 2. Architecture translation matrix

| Incident symptom | Exact architecture need | Primary enforcement point |
|---|---|---|
| Agent installed GitHub release asset on live build host | Environment Contract + Binary Provenance + Mutation Preflight | `focusa action preflight`, CLI install/restart paths |
| Existing daemon/repo setup ignored | Runtime Inventory service | `focusa status/doctor/pair`, preflight packet |
| Version mismatch interpreted as install problem | Build-host repair policy | daemon repair + binary policy resolver |
| Context preserved but not used | Operational Context Gate | every risky mutation |
| TL/HLT generic and misleading | HLT validity model + degraded bootstrap state | trajectory view + `focusa hlt verify` |
| Prior verbatim HLT not used | HLT History Fallback Resolver | trajectory view + `focusa hlt ls` |
| MLG/STG polluted by Workpoint text | Ladder derivation engine with source constraints | trajectory projection builder |
| Current ask replaced by install task | Task Substitution Detector | preflight action classifier |
| Planning prompt triggered implementation | Intent Mode Gate | agent prompt/CLI planning wrapper |
| Release asset glibc mismatch | Host Compatibility Checker | binary install/download path |
| Daemon root/stale lock confusion | Daemon Runtime Hygiene | `focusa start/stop/status/doctor` |
| Need reproducible prevention | Incident Replay Golden Tests | CI/spec gates |

---

## 3. Architectural need A — Operational Context Gate

### Problem exposed

The agent performed a high-risk mutation (binary overwrite) without reconciling current ask, Workpoint, repo, daemon, binary provenance, environment role, and HLT/history.

### Responsibility

Create a single preflight authority packet for risky actions. The gate classifies the proposed action, gathers facts, detects contradictions, and returns a verdict.

### Inputs

- current operator ask
- current Workpoint/resume packet
- project identity/root
- git head/tag/status
- runtime inventory
- binary provenance
- environment contract
- `focusa hlt ls`
- `focusa hlt history`
- proposed action
- risk class

### Output model

```json
{
  "schema": "focusa.operational_context_gate.v1",
  "verdict": "allow|block|ask_operator",
  "risk_class": "read|low|medium|high|destructive",
  "current_ask": "initiate Phone Bridge pairing",
  "workpoint_id": "...",
  "project_root": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
  "environment_role": "live_build_host",
  "proposed_action": {
    "kind": "binary_replace",
    "target": "/usr/local/bin/focusa",
    "source": "github_release_asset"
  },
  "conflicts": [
    {
      "class": "consumer_install_path_conflicts_with_live_build_host",
      "why": "This host is the live Focusa build host; release assets are not the repair source."
    }
  ],
  "safe_alternative": "Build from ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} and restart daemon as wirebot.",
  "evidence_refs": []
}
```

### Required surfaces

- `focusa action preflight --kind <kind> --target <path> --source <source> --json`
- API route: `POST /v1/action/preflight`
- reusable Rust module: `operational_context_gate`

### Enforcement points

Mandatory before:

- overwriting Focusa binaries,
- downloading/installing release assets,
- daemon kill/restart,
- daemon lock mutation,
- pairing state mutation,
- HLT supersession,
- filesystem cleanup outside generated artifacts.

### Acceptance tests

- Live build host + GitHub release asset install => blocked.
- Live build host + local rebuild/restart => allowed/medium risk.
- Consumer install host + compatible release asset + explicit operator request => allowed/ask depending policy.
- Missing environment contract => ask operator or degraded verify-first.

---

## 4. Architectural need B — Focusa Environment Contract

### Problem exposed

The agent did not know, or did not operationalize, that this VPS was a live Focusa build host.

### Responsibility

Persist machine/install role as a durable, machine-readable local contract.

### Data model

```json
{
  "schema": "focusa.environment_contract.v1",
  "install_role": "live_build_host|consumer_install|dev_worktree|unknown",
  "machine_kind": "vps|mac|local|container|unknown",
  "project_root": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
  "owner": "wirebot",
  "binary_policy": {
    "preferred_source": "local_repo_build|release_asset|package_manager",
    "release_asset_install_allowed": false,
    "local_build_required": true
  },
  "pairing_state": "never_paired|paired|unknown",
  "host": {
    "os": "AlmaLinux",
    "arch": "x86_64",
    "glibc": "detected"
  },
  "created_at": "...",
  "updated_at": "..."
}
```

### Storage

Preferred:

- `/etc/focusa/environment-contract.json` for machine-wide install facts.

Project mirror/read model:

- `.focusa/environment-contract.json` or runtime generated view, if safe.

### Required surfaces

- `focusa env contract show --json`
- `focusa env contract init --role live_build_host --project-root ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} --owner wirebot`
- `focusa doctor` includes contract.
- `focusa pair --json` includes contract summary.

### Enforcement

- If `release_asset_install_allowed=false`, preflight blocks release asset installation.
- If `preferred_source=local_repo_build`, daemon repair recommends local build/restart.

### Acceptance tests

- Contract says live build host => release asset install blocked.
- Contract absent => high-risk mutation asks operator.
- Contract project root mismatch => verify-first.

---

## 5. Architectural need C — Binary Provenance and Host Compatibility

### Problem exposed

A GitHub binary incompatible with VPS glibc was installed.

### Responsibility

Every Focusa binary should expose provenance and compatibility metadata; installation should preflight compatibility before overwrite.

### Data model

```json
{
  "schema": "focusa.binary_provenance.v1",
  "binary": "focusa",
  "version": "0.9.25-dev",
  "git_sha": "...",
  "build_profile": "release",
  "source_type": "local_repo_build|release_asset|package_manager|unknown",
  "source_ref": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
  "target_triple": "x86_64-unknown-linux-gnu",
  "host_glibc_required": "...",
  "built_on": "..."
}
```

### Required surfaces

- `focusa --version --json`
- `focusa-daemon --version --json`
- `focusa binary inspect /usr/local/bin/focusa --json`
- `focusa binary preflight-install --asset <path|url> --target /usr/local/bin/focusa --json`

### Enforcement

- Release asset install must check OS/arch/glibc.
- Existing binary should be backed up before replacement.
- Provenance mismatch should be reported, not silently overwritten.

### Acceptance tests

- AlmaLinux older glibc + GitHub asset requiring newer glibc => blocked.
- Local repo build => allowed.
- CLI/daemon version mismatch => reports provenance and safe repair path.

---

## 6. Architectural need D — Runtime Inventory and Daemon Hygiene

### Problem exposed

The agent had to manually inspect daemon PID/user/version/bind/lock, and a root daemon plus stale lock appeared during recovery.

### Responsibility

Provide one reliable runtime inventory and repair surface.

### Runtime inventory fields

```json
{
  "schema": "focusa.runtime_inventory.v1",
  "daemon": {
    "running": true,
    "pid": 3616130,
    "user": "wirebot",
    "bind": "127.0.0.1:8787",
    "version": "0.9.25-dev",
    "lock_pid": 3616130,
    "lock_matches_process": true,
    "one_listener_per_bind": true
  },
  "cli": {
    "path": "/usr/local/bin/focusa",
    "version": "0.9.25-dev"
  }
}
```

### Required surfaces

- `focusa runtime inventory --json`
- `focusa doctor` includes runtime inventory.
- `focusa start` refuses or repairs stale lock with explicit diagnostics.
- `focusa stop` reports exact PID/user/bind stopped.

### Enforcement

- Daemon should run as configured owner unless operator explicitly overrides.
- Lock PID must match live process.
- Multiple daemons on same bind must be detected.
- Root daemon on project owner host is warning/block depending policy.

### Acceptance tests

- stale lock => safe repair with diagnostic.
- daemon version mismatch => safe restart using allowed binary source.
- root daemon while owner is wirebot => doctor warns and repair recommends owner restart.

---

## 7. Architectural need E — Verbatim HLT History Fallback Resolver

### Problem exposed

Active HLT became a generic bootstrap phrase instead of using prior verbatim HLT history when unsure.

### Responsibility

When current HLT is missing, generic, stale, or low-confidence, resolve the active HLT from the latest valid verbatim HLT ledger record.

### Existing surfaces

The CLI already exists:

```bash
focusa hlt ls
focusa hlt history
focusa hlt verify
```

This is not a new surface requirement. It is a correctness requirement.

### Resolver inputs

- active trajectory record
- HLT ledger history
- goal provenance
- operator confirmation/evidence refs
- continuity/project root
- staleness status

### Valid HLT criteria

A valid active HLT must be:

- verbatim from operator or durable supersession, or
- explicitly recorded in HLT ledger with provenance, and
- scoped to project root / continuity where applicable, and
- not a generic bootstrap placeholder, and
- not contradicted by current operator ask.

### Degraded HLT criteria

Mark degraded if:

- text starts with generic bootstrap pattern like `Maintain and improve ... within verified project scope`,
- no ledger/provenance exists,
- MLG/STG are derived from unrelated Workpoint text,
- history exists but was not applied.

### Required behavior

- `focusa hlt ls` shows active verbatim HLT.
- If current active HLT is unsure, `focusa hlt ls` uses latest valid verbatim historical HLT.
- Generic bootstrap is displayed as degraded placeholder, not real HLT.
- `focusa hlt verify` fails when HLT is generic and history has a valid HLT.

### Acceptance tests

- History has verbatim HLT, active projection missing => active HLT resolves to history.
- Active generic HLT, history has verbatim HLT => active HLT resolves to history and warns.
- No history and no operator HLT => HLT undefined/degraded, no `proceed` clarity.

---

## 8. Architectural need F — Trajectory Ladder Derivation Discipline

### Problem exposed

MLG/STG became polluted by Workpoint/current-focus/aborted implementation text.

### Responsibility

Enforce ladder derivation order:

```text
HLT → MLG → STG → Waypoints → Workpoint
```

### Rules

- HLT is durable north-star.
- MLG derives from valid HLT.
- STG derives from valid HLT + MLG.
- Waypoints derive from MLG/STG.
- Workpoint remains canonical immediate continuation.
- Workpoint text can inform STG only if compatible with valid HLT.
- Workpoint text can never become HLT.
- Workpoint text can never silently become MLG/STG when HLT is invalid.

### Required data annotations

Every ladder field must include:

```json
{
  "value": "...",
  "source": "hlt_history|trajectory_record|operator|workpoint|focus_state|bootstrap",
  "source_ref": "...",
  "confidence": "high|medium|low",
  "derived_from": ["..."],
  "degraded": false
}
```

### Acceptance tests

- invalid/generic HLT + active Workpoint => MLG/STG remain degraded/undefined, not polluted.
- valid HLT + compatible Workpoint => STG may align to Workpoint with source annotations.
- `focusa hlt verify` catches MLG/STG source violations.

---

## 9. Architectural need G — Task Substitution Detector

### Problem exposed

The current task was pairing initiation, but the agent substituted release installation.

### Responsibility

Detect when proposed action belongs to a different workflow than current ask/Workpoint.

### Examples

```text
Current ask: initiate Phone Bridge pairing
Proposed action: install GitHub release asset
Verdict: block
Reason: consumer install path conflicts with live build-host pairing test
```

### Required surfaces

- integrated into Operational Context Gate.
- optional direct command: `focusa action classify --current-ask ... --proposed-action ... --json`.

### Acceptance tests

- pairing ask + binary install => blocked/ask.
- pairing ask + local daemon restart => allowed.
- implementation ask + code edit => allowed if repo clean/preflight passes.

---

## 10. Architectural need H — Intent Mode Gate

### Problem exposed

The agent almost implemented code after exploratory/planning language from the operator.

### Responsibility

Classify operator turns before mutation.

### Modes

- `planning_discussion`
- `diagnosis`
- `implementation_authorized`
- `runtime_operation_authorized`
- `destructive_or_high_risk_requires_confirmation`

### Rules

- “Maybe we can...” => planning_discussion.
- “What if...” => planning_discussion.
- “Read the spec” => diagnosis/read-only.
- “Implement X” => implementation_authorized, subject to preflight.
- “Restart daemon” => runtime_operation_authorized, subject to runtime preflight.

### Acceptance tests

- exploratory flag proposal creates no code changes.
- “read spec” creates no code changes.
- explicit “rebuild and restart daemon” permits daemon operation but not release asset install.

---

## 11. Architectural need I — Incident Replay Golden Tests

### Problem exposed

This failure must become a permanent regression test.

### Golden scenario

```json
{
  "scenario": "live_build_host_pairing_test_stale_runtime",
  "facts": {
    "project_root": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
    "install_role": "live_build_host",
    "repo_version": "0.9.25-dev",
    "daemon_version": "0.9.23-dev",
    "cli_version": "0.9.22-dev",
    "current_ask": "initiate Phone Bridge pairing"
  },
  "proposed_action": {
    "kind": "binary_replace",
    "source": "github_release_asset",
    "target": "/usr/local/bin/focusa"
  },
  "expected": {
    "verdict": "block",
    "reason": "consumer_install_path_conflicts_with_live_build_host",
    "safe_alternative": "local_repo_build_and_daemon_restart"
  }
}
```

### Required CI gate

- static scenario JSON checked into tests/golden.
- Rust or shell test invokes preflight engine.
- CI fails if release asset install is allowed for this scenario.

---

## 12. Architectural need J — Phone Bridge Pairing Preflight Integration

### Problem exposed

`focusa pair` is a high-context operation and should summarize environment authority before proceeding.

### Responsibility

`focusa pair --json` should include:

- environment contract summary,
- runtime inventory,
- HLT/Workpoint sanity summary,
- selected transport diagnostics,
- safe next action.

### Required behavior

- If daemon stale but local build host, repair path is local rebuild/restart.
- If release asset install would be needed, ask/block depending environment contract.
- Pairing continues only after runtime inventory passes.

### Acceptance tests

- live build host with stale daemon => pair recommends/executes local daemon repair.
- pair output includes `environment_contract`, `runtime_inventory`, and existing `diagnostics`.

---

## 13. Bead decomposition plan

Parent bead:

- `focusa-9o3v` — Incident workorder: Focusa context authority failure during Phone Bridge pairing

Child beads to create:

1. Operational Context Gate architecture + implementation
2. Environment Contract schema/CLI/doctor integration
3. Binary Provenance + Host Compatibility preflight
4. Runtime Inventory + Daemon Hygiene
5. HLT History Fallback Resolver
6. TL Derivation Discipline + verification
7. Task Substitution Detector
8. Intent Mode Gate
9. Incident Replay Golden Tests
10. Phone Bridge Pairing Preflight Integration

Each child bead must include:

- architecture need,
- impacted files/modules,
- acceptance criteria,
- tests required,
- dependency relation.

---

## 14. Implementation priority

1. Incident Replay Golden Test skeleton
2. Operational Context Gate
3. Environment Contract + Runtime Inventory
4. Binary Provenance/Compatibility
5. HLT History Fallback Resolver
6. TL Derivation Discipline
7. `hlt verify` hardening
8. Intent Mode Gate
9. Phone Bridge preflight integration
10. Documentation/doctor integration

Reasoning:

- Golden tests preserve the incident.
- Context gate prevents recurrence even while TL/HLT are being fixed.
- HLT/TL correctness restores Focusa trajectory promise.
- Phone Bridge integration ensures the original workflow is protected.

---

## 15. Definition of done

This architecture work is complete when:

1. A fresh agent can discover live build-host context before mutation.
2. Release asset install is blocked on this host unless explicitly allowed.
3. Local repo build/restart is recommended for stale runtime on live build host.
4. `focusa hlt ls` resolves active HLT from prior verbatim HLT history when current HLT is unsure.
5. Generic HLT bootstrap is degraded and cannot produce “proceed”.
6. MLG/STG cannot be polluted by unrelated Workpoint text.
7. `focusa hlt verify` fails on generic/polluted ladder state.
8. Planning prompts do not trigger implementation.
9. Phone Bridge pairing preflight reports environment/runtime/trajectory status.
10. Incident replay golden scenario is permanently in CI.
