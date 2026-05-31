# SPEC97 Scope Guard Live Integration Proof — 2026-05-31

## Scope

Bead: `focusa-yv8d.12` — live proof for the anti-forgetting / project-scope override guard.

Spec: `docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md`

## Runtime under test

- Current source built with `cargo build -p focusa-api` using `CARGO_TARGET_DIR=/tmp/focusa-cargo-target-yv8d12`.
- Temporary daemon: `FOCUSA_BIND=127.0.0.1:18787`, temp `FOCUSA_DATA_DIR=/tmp/focusa-yv8d12-data.*`.
- Reason: installed/live daemon on `127.0.0.1:8787` was stale for newly added Workpoint action-authority fields; current-source daemon proves implemented behavior without mutating production Focusa state.

## Proof commands

```bash
# Current-source API proof
bash /tmp/focusa_yv8d12_live_proof.sh
python3 - <<'JSONPY' >/tmp/focusa-yv8d12-ptm-verify-req.json
import json
print(json.dumps({
  "project_root": "/home/planmarr/plan-the-marriage",
  "project_id": "plan-the-marriage",
  "canonical_name": "plan-the-marriage",
  "remote_host": "ptm-remote",
  "remote_user": "planmarr",
  "remote_repo_remote": "https://github.com/example/plan-the-marriage.git",
  "remote_workspace_kind": "rust-monorepo",
  "remote_deploy_root": "/home/planmarr/plan-the-marriage"
}))
JSONPY
curl -fsS -X POST http://127.0.0.1:18787/v1/project/verify   -H "Content-Type: application/json"   -d @/tmp/focusa-yv8d12-ptm-verify-req.json
# PTM anchor proof
bash /tmp/focusa_yv8d12_ptm_anchor.sh
# Ledger/regression proof
bun tests/pi_session_project_switch_ledger_runtime_test.mts
bun tests/current_ask_project_override_runtime_test.mts
bun tests/scope_arbitration_runtime_test.mts
tests/scope_routing_regression_eval.sh
```

## Observed results

### 1. Saved Focusa Workpoint remains canonical for saved scope but loses current-action authority for PTM correction

`/v1/workpoint/resume` response for saved Focusa scope plus current ask `wrong place — this is the PTM remote project at /home/planmarr/plan-the-marriage`:

```json
{
  "status": "completed",
  "canonical": true,
  "canonical_for_saved_scope": true,
  "matches_current_ask_scope": false,
  "action_authority_for_current_ask": false,
  "scope_conflict_reason": "operator named different project path /home/planmarr/plan-the-marriage",
  "next_tools": [
    "focusa_project_verify",
    "focusa_project_identity",
    "focusa_workpoint_checkpoint",
    "focusa_workpoint_resume"
  ],
  "tool_result": {
    "ok": false,
    "degraded": true,
    "failure_class": "scope_conflict",
    "retry": { "posture": "do_not_retry_unchanged", "safe": false }
  }
}
```

### 2. PTM project identity verifies from explicit remote evidence boundary

`/v1/project/verify` with PTM root and remote evidence:

```json
{
  "verified": true,
  "status": "verified",
  "project_root": "/home/planmarr/plan-the-marriage",
  "confidence": "high",
  "authority_boundary": "remote_host_plus_project_root_plus_fingerprint",
  "remote_context": {
    "remote_user": "planmarr",
    "remote_workspace_kind": "rust-monorepo",
    "remote_deploy_root": "/home/planmarr/plan-the-marriage",
    "verification_note": "remote evidence is caller-supplied after SSH/repo inspection; Focusa daemon does not open SSH sessions"
  },
  "tool_result": { "ok": true, "canonical": true, "degraded": false }
}
```

### 3. PTM action anchor establishes authority after rebind

PTM-scoped `/v1/workpoint/checkpoint` accepted; PTM-scoped `/v1/workpoint/resume` with current ask `continue PTM at /home/planmarr/plan-the-marriage` returned:

```json
{
  "status": "completed",
  "canonical": true,
  "canonical_for_saved_scope": true,
  "matches_current_ask_scope": true,
  "action_authority_for_current_ask": true,
  "scope_conflict_reason": "none",
  "tool_result": { "ok": true, "degraded": false, "failure_class": null }
}
```

### 4. Same-session project-switch ledger and scope-conflict telemetry prove hot-path behavior

Regression commands passed:

- `bun tests/pi_session_project_switch_ledger_runtime_test.mts` → `SPEC project-switch ledger runtime proof passed`
- `bun tests/current_ask_project_override_runtime_test.mts` → `SPEC current-ask project override runtime proof passed`
- `bun tests/scope_arbitration_runtime_test.mts` → `SPEC scope arbitration runtime proof passed`
- `tests/scope_routing_regression_eval.sh` → `Tests passed: 6`, `Tests failed: 0`, and telemetry distinguishes `scope_conflict_detected` from generic `scope_mismatch`.

## Operator-visible response contract

When saved scope and current ask conflict, response should be concise:

> Scope conflict: saved Workpoint is `/home/wirebot/focusa`; current ask indicates PTM `/home/planmarr/plan-the-marriage`. Verifying and rebinding before action.

## Acceptance mapping

- Saved Focusa Workpoint remains valid for saved scope: `canonical_for_saved_scope=true`.
- PTM correction suppresses Focusa action before file/API action: `action_authority_for_current_ask=false`, `failure_class=scope_conflict`, `safe=false`.
- Project-switch ledger surfaces PTM evidence: runtime ledger test passed.
- Project verify/rebind occurs: `/v1/project/verify` succeeded and PTM Workpoint checkpoint/resume established an action anchor.
- Operator-visible response explains rebind path: contract above.
