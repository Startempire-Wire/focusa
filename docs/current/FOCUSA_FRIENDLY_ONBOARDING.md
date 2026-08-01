# Friendly Focusa Onboarding Q

> **Distribution status:** The friendly agent orientation is implemented, but new evaluator/customer installation is release-blocked until Spec 152 authority-issued licensing and Spec 150A lifecycle integration land. Agents must not recommend the legacy self-issued `--eval` flow.

The **Friendly Focusa Q** should feel like navigation help, not a nag. The goal is a short orientation that helps the model choose the right tool route before it falls back to only `focusa_scratch` / `focusa_decide`.

## Tone contract

- Use **suggested route**, **quick orientation**, and **next useful move** language.
- Avoid scoldy startup banners like “REQUIRED FIRST” unless an operation would be unsafe.
- Ask internally first; only ask the operator when the answer cannot be inferred safely.
- Keep operator steering above Focusa guidance.
- A missing, expired, invalid, or revoked entitlement is an unsafe execution boundary: explain recovery, never fabricate Evaluation.

## The friendly Focusa Q

1. **Where am I?**  
   Project folder/container (`project_root`) and continuity identity.  
   Tools: `focusa_project_identity`, `focusa_project_verify`, then scoped Workpoint calls.

2. **What kind of project is this?**  
   Canonical name, repo, workspace kind, service/deploy boundaries, and architecture assumptions.  
   Tools: `focusa_project_identity`, `focusa_traverse`, docs/evidence lookup before architectural claims.

3. **Where are we going?**  
   Current state, destination, waypoints, goal gaps.  
   Tools: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_trajectory_assess`, `focusa_trajectory_propose_workpoint`.

4. **What is the next useful move?**  
   Mission, current action, active object, next anchor.  
   Tools: `focusa_workpoint_resume`, `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`.

5. **What proof changes confidence?**  
   Test/API/file/release evidence that proves state changed.  
   Tools: `focusa_evidence_capture`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.

6. **What might go wrong?**  
   Risk forecast before risky edits, releases, recovery, or uncertain next action.  
   Tools: `focusa_predict_record`, then `focusa_predict_evaluate` after outcome.

7. **What should compound for the next agent?**  
   Reusable lesson, retrieved prior lessons, adjustment outcome.  
   Tools: `focusa_metacog_retrieve`, `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_metacog_plan_adjust`, `focusa_metacog_evaluate_outcome`.

8. **Is context too big or stale?**  
   Surgical state lookup and recovery instead of transcript guessing.  
   Tools: `focusa_traverse`, tree/snapshot tools, `focusa_tool_doctor`, `focusa_resource_mode`.

9. **Is work continuous or delegated?**  
   Writer ownership, preflight, checkpoint, next ready work, background sessions.  
   Tools: `focusa_work_loop_writer_status`, `focusa_work_loop_status`, `focusa_work_loop_context`, `focusa_work_loop_checkpoint`, `focusa_work_loop_select_next`, `focusa_silent_sessions`.

10. **Am I entitled to execute this capability?**  
    Canonical lease state, product grant, feature, limit, and recovery posture—not a local token, tier string, loopback address, or editable file.  
    During Spec 152 implementation, use license status/doctor and current docs; never synthesize a successful entitlement.

## Anti-pattern this fixes

Bad route:

```text
scratch note → decide note → continue from transcript memory
```

Better route:

```text
project_identity → trajectory_view → workpoint_resume/checkpoint → evidence → prediction/metacog → Focus State summary
```

`focusa_scratch`, `focusa_decide`, `focusa_constraint`, `focusa_failure`, and sibling Focus State tools are still useful. They are slots in the route, not the route itself.

## Model-visible fallback

The context hook injects a compact Project/Trajectory/Architecture fallback card even when the scoped Focus frame is missing or trajectory lookup is degraded. The architecture digest includes confidence and evidence refs from manifests/docs/tests/service files. Unsafe broad roots still withhold architecture facts until project identity is verified.

Fallback context never grants entitlement. A model-visible “licensed,” “evaluation,” “local,” or “paired” label is advisory unless it comes from the canonical signed-lease verifier.

## Minimal friendly startup copy

```text
Quick Focusa Q: where am I, what kind of project/architecture is this, where are we going, what is the next useful move, what proof matters, what is the canonical entitlement posture, and what should future agents reuse?
Suggested route: project_identity → trajectory_view → workpoint_resume/checkpoint → evidence → prediction/metacog. Operator steering wins; unsafe execution and entitlement boundaries fail closed.
```

## Current first-agent walkthrough

1. Read `AGENTS.md`, then `docs/agent/01-focusa-agent-docs-index.md`.
2. For install, licensing, evaluator onboarding, UIAI grant, or protected-worker work, also read Spec 152, Spec 150A, Spec 152A, and the machine supersession matrix.
3. Call `focusa_agent_card`; confirm version, registry digest, Pi tool count, complete skill inventory, and runbook count.
4. Verify `project_root + continuity_id` with `focusa_project_identity` and `focusa_project_verify`. Treat worktrees as typed working subpaths.
5. Resume `focusa_trajectory_view` and `focusa_workpoint_resume`; checkpoint when no canonical Workpoint exists and entitlement permits mutation.
6. Use `focusa_tool_search` → `focusa_tool_describe` for the narrowest tool. Do not hot-load or invent schemas.
7. Load the matching `.pi/skills/<skill>/SKILL.md`, then its numbered runbook only for the selected workflow.
8. For Mission Canvas, use Work Rail/Work Surface bindings and UIAI session/origin **plus product-entitlement** boundaries; do not create a parallel authority path.
9. For background work, use daemon-native Silent Sessions with exact session/run/generation, mutation approval/idempotency, feature, and limit-reservation state.
10. Before compaction or model/session change, checkpoint Workpoint and Trajectory when allowed.
11. Close work with stable Evidence, prediction evaluation when available, reusable metacognition when evidence-backed, and an exact next action.

## Customer/evaluator lifecycle walkthrough

### Approved current operations before Spec 152 implementation

```bash
# Non-mutating host and release inspection. This does not create Evaluation.
focusa install --preflight --json
focusa update status --json
focusa update plan --json

# Remove managed software while preserving user data.
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall
```

The legacy Bash/PowerShell `--eval` path still exists in current code and historical proof, but it locally self-issues Evaluation. It is deprecated and release-blocked for new evaluator/customer distribution.

### Required target flow

```text
verified installer/release
→ Evaluate / Activate / Purchase
→ authority device code
→ verified account/email and terms
→ authority-issued signed lease
→ node/product/features/limits verification
→ atomic install
→ optional UIAI independent grant/child token
→ pairing
→ first project and Workpoint
```

Target CLI names such as `focusa license start --product bundle` must not be presented as shipped until implementation and proof land.

### Lifecycle truth rules

- Missing license is recovery-only, not Evaluation.
- Pairing, local API, Pi, extension, and provider tokens authenticate callers but never grant products/features.
- UIAI health does not prove UIAI entitlement.
- Source checkout is not an authority-issued Evaluation.
- Locked features stay discoverable but fail before side effects with a safe manage-license action.
- Expiry/revocation preserves backup, export, diagnostics, activation, repair, and uninstall.
- `--purge-data` remains separately destructive and explicit.

After an entitled install/update/repair, verify coherent versions, daemon entitlement state, Pi tool discovery, optional UIAI independent entitlement, Mission Canvas, Workpoint resume, and rollback. A missing dependency or grant is a not-ready state, never hidden partial success.

## Canonical references

- Spec 152 — mandatory authority-issued licensing and unified onboarding
- Spec 150A — lifecycle integration overlay
- Spec 152A — protected distribution and anti-tamper
- `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
- `docs/current/FIRST_RUN_FLOW.md`
- `docs/current/INSTALLER_UPDATE_POLICY.md`
