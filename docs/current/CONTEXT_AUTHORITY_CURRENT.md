# Context Authority Current Guide

**Status:** implemented current development slice  
**Source incident:** `docs/current/INCIDENT_PHONE_BRIDGE_CONTEXT_AUTHORITY_2026-06-11.md`  
**Architecture translation:** `docs/current/CONTEXT_AUTHORITY_ARCHITECTURE_WORKORDER_SPEC_2026-06-12.md`  
**Implementation commits:** `b9d653f`, `a5e1171`, `266b9a4`, `8060cd1`, `5362c19`, `32a453c`, `dc9a335`  
**Validation bundle:** context-authority tests, 2026-06-12

## Purpose

Context Authority makes preserved context operational at the mutation boundary. Focusa must not treat Workpoint, HLT/TL, runtime facts, binary provenance, or current ask as passive memory. For risky work, those facts are reconciled into a verdict before mutation.

The incident rule is:

> A Focusa agent must not perform risky mutation until current ask, Workpoint, environment role, runtime inventory, binary compatibility, HLT/TL validity, and intent mode have been reconciled.

## Implemented surfaces

### `focusa action preflight`

Mutation-time Operational Context Gate.

```bash
focusa --json action preflight \
  --current-ask "initiate Phone Bridge pairing" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} \
  --repo-version 0.9.25-dev \
  --cli-version 0.9.22-dev \
  --daemon-version 0.9.23-dev
```

Envelope:

```json
{
  "schema": "focusa.operational_context_gate.v1",
  "verdict": "block",
  "risk_class": "high",
  "environment_role": "live_build_host",
  "conflicts": [
    {"class": "consumer_install_path_conflicts_with_live_build_host"},
    {"class": "task_substitution_detected"}
  ],
  "safe_alternative": "Build from the verified local Focusa repo and restart the daemon as the project owner."
}
```

Rules:

- `live_build_host` + `github_release_asset`/`release_asset` binary replacement of `focusa` or `focusa-daemon` blocks.
- Pairing ask + binary replacement blocks as task substitution.
- Unknown install role + release binary replacement asks operator / verify-first.

### `focusa action classify-intent`

Intent Mode Gate for planning vs mutation.

```bash
focusa --json action classify-intent --prompt "Maybe we can add a flag for install context"
```

Envelope:

```json
{
  "schema": "focusa.intent_mode_gate.v1",
  "mode": "planning_discussion",
  "mutation_allowed": false,
  "requires_preflight": false,
  "recommended_action": "produce plan/spec only; do not mutate files or runtime"
}
```

Modes:

- `planning_discussion` — no mutation.
- `diagnosis` — read/inspect only.
- `implementation_authorized` — mutation allowed only after repo/status and context preflight.
- `runtime_operation_authorized` — runtime mutation allowed only after operational preflight.
- `destructive_or_high_risk_requires_confirmation` — explicit confirmation plus preflight required.

### `focusa env contract show/init`

Machine-readable install/environment contract.

```bash
focusa --json env contract init \
  --role live_build_host \
  --project-root ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} \
  --owner wirebot \
  --machine-kind vps \
  --preferred-source local_repo_build

focusa --json env contract show
```

Default path:

```text
/etc/focusa/environment-contract.json
```

Schema:

```json
{
  "schema": "focusa.environment_contract.v1",
  "install_role": "live_build_host",
  "machine_kind": "vps",
  "project_root": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
  "owner": "wirebot",
  "binary_policy": {
    "preferred_source": "local_repo_build",
    "release_asset_install_allowed": false,
    "local_build_required": true
  },
  "pairing_state": "unknown",
  "host": {"os": "linux", "arch": "x86_64", "glibc": "2.28"}
}
```

Rules:

- `live_build_host` implies local build repair policy.
- `release_asset_install_allowed=false` blocks release asset replacement at preflight.
- Missing contract is a verify-first state, not silent permission.

### `focusa runtime inventory`

CLI/daemon runtime and hygiene facts.

```bash
focusa --json runtime inventory --owner wirebot
```

Envelope:

```json
{
  "schema": "focusa.runtime_inventory.v1",
  "daemon": {
    "running": true,
    "pid": 3616130,
    "user": "wirebot",
    "bind": "127.0.0.1:8787",
    "version": "0.9.25-dev",
    "lock_pid": null,
    "lock_matches_process": null,
    "one_listener_per_bind": true
  },
  "cli": {"path": "/usr/local/bin/focusa", "version": "0.9.25-dev"},
  "hygiene": {"status": "ok", "warnings": []}
}
```

Rules:

- CLI/daemon version mismatch is degraded and recommends doctor/local repair.
- Lock PID mismatch is degraded.
- Expected owner mismatch is degraded.

### `focusa binary inspect/preflight-install`

Binary provenance and compatibility preflight.

```bash
focusa --json binary inspect /usr/local/bin/focusa

focusa --json binary preflight-install \
  --asset /tmp/focusa-release-asset \
  --target /usr/local/bin/focusa \
  --install-role live_build_host \
  --source github_release_asset
```

Envelopes:

- `focusa.binary_provenance.v1`
- `focusa.binary_preflight.v1`

Rules:

- Release asset install on `live_build_host` blocks.
- GLIBC requirement above host GLIBC blocks.
- Unknown install role asks operator before binary replacement.

### `focusa pair --json` context preflight

Phone Bridge Flow now includes context authority fields in JSON output:

```json
{
  "environment_contract": {"schema": "focusa.environment_contract.v1"},
  "runtime_inventory": {"schema": "focusa.runtime_inventory.v1"},
  "action_preflight": {"schema": "focusa.operational_context_gate.v1"},
  "diagnostics": {"surface": "phone_bridge_flow"}
}
```

Rules:

- Pairing is `pairing_start`, not install.
- Pairing JSON exposes whether the daemon/runtime/environment were verified.
- Agents must not substitute release installation for pairing initiation.

## HLT/TL behavior

Generic bootstrap HLT text such as:

```text
Maintain and improve Focusa within verified project scope
```

is now treated as a degraded placeholder, not an authoritative HLT.

Rules:

- Generic bootstrap HLT sets `hlt_degraded_placeholder=true`.
- Generic bootstrap HLT does not satisfy the `long_term_goal` missing-fact gate.
- Generic bootstrap HLT produces `needs_definition=true` and `degraded=true`.
- Workpoint/current_focus cannot populate MLG/STG when HLT is invalid or generic.
- `trajectory_ladder.source_metadata` reports HLT/MLG/STG source and degraded status.
- `latest_valid_historical_trajectory` provides the code path for verbatim history fallback when a valid prior HLT record exists.

## Validation commands

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_preflight_golden_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_environment_contract_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_runtime_inventory_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_binary_preflight_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_intent_mode_test.sh
as-user wirebot 'cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>} && tests/spec_context_authority_hlt_ladder_static_test.sh && tests/spec_context_authority_pair_preflight_static_test.sh'
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority /root/.cargo/bin/cargo check -q -p focusa-api -p focusa-cli --locked
```

## Agent rule

Before risky mutation, run the smallest applicable context-authority preflight:

1. classify prompt mode if intent is ambiguous,
2. inspect environment contract,
3. inspect runtime inventory,
4. inspect binary compatibility for binary replacement,
5. run action preflight,
6. only mutate if verdict allows it.

