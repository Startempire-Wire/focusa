# Focusa Install Lifecycle Runbook

## Preconditions

- Read Spec 152, Spec 150A, Spec 152A, and the Spec 152 supersession matrix before install/license/evaluator/UIAI/protected-component work.
- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work when entitlement permits mutation.
- Confirm current operator steering and mutation approval boundaries.
- Treat cwd, missing markers, local tokens, editable license files, loopback origin, source access, and UI health as weak/non-authoritative evidence.
- Start wall-clock measurement and a bounded prediction for meaningful work; evaluate it at completion.
- Use targeted local gates during development; production release requires explicit authorization and the combined lifecycle/entitlement gates.

## Mandatory authority laws

- Every official runtime, including Evaluation, requires an authority-issued signed lease.
- Missing/invalid/expired/revoked entitlement means recovery-only, not Evaluation.
- The legacy Bash/PowerShell `--eval` path is release-blocked and must not be recommended.
- Pairing, local API, Pi, extension, provider, and UIAI tokens authenticate callers; they do not create product grants.
- A source checkout is not an authority-issued Evaluation.
- Uninstall, safe export/backup, license recovery, and data-preserving repair remain available without active execution entitlement.
- Purge remains separately destructive and explicit.

## Dependency graph

```text
release_trust
  -> entitlement_status/device_code
  -> node_and_product_grant
  -> lifecycle_preflight
  -> optional_ui_ai_grant
  -> pairing
  -> project_identity
  -> first_workpoint
  -> evidence
```

Project/tool subgraph:

```text
focusa_project_identity -> focusa_workpoint_checkpoint
focusa_workpoint_checkpoint -> focusa_evidence_capture
focusa_evidence_capture -> focusa_tool_doctor
```

## Current approved path

Until Spec 152 implementation is shipped:

1. Inspect without mutation using `focusa install --preflight --json` and update-status/plan commands.
2. Do not install a new evaluator through legacy `--eval`.
3. Use recovery/doctor/repair only to restore health, licensing surfaces, backup/export, or uninstall.
4. Public uninstall preserves user data by default; destructive removal requires explicit purge confirmation.
5. Report `implementation blocked by mandatory authority-issued licensing` rather than substituting a local license.

## Target entitled install path

1. Verify exact repository/host/release identity and signed release metadata.
2. Resolve an existing canonical signed lease or start the authority device-code flow.
3. User verifies account/email and terms at the authority origin; promotional consent is separate.
4. Authority issues Evaluation/paid/developer product grants and registers the node.
5. Client verifies lease signature, key id, schema, product, status, node, sequence, time bounds, features, and limits.
6. Build a Spec 150 lifecycle request containing the immutable entitlement binding.
7. Stage and verify public artifacts and entitled protected workers/capsules.
8. Activate atomically; start daemon in entitled or recovery posture.
9. If UIAI is selected, verify its product grant and scoped child token independently.
10. Pair devices only after entitlement resolution.
11. Select/verify project explicitly; run Bootstrap/Genesis/first Workpoint only when features permit.
12. Produce one lifecycle acceptance receipt whose lease id/sequence/digests reconcile across authority, Focusa, and optional UIAI.

## Branches

- No lease: recovery-only; start device-code/activation or safe uninstall/export.
- Authority unavailable: continue only inside a previously signed offline window; never create/extend Evaluation locally.
- Expired/revoked/refunded: deny execution; preserve data and activation/recovery paths.
- Wrong product: deny product execution; offer correct manage-license action.
- Node limit: keep current nodes safe; require authority-managed deactivation/replacement.
- Unknown tool/schema: `focusa_tool_search` → `focusa_tool_describe`.
- Scope conflict: `focusa_project_verify` → blocked receipt; do not infer.
- Daemon degraded: `focusa_tool_doctor`; retry only with safe recovery posture.
- Resource timeout: `focusa_resource_mode` → bounded `focusa_traverse`.
- Browser failure: UIAI diagnostics → `focusa_browser_diagnostics_intake` → Evidence, while retaining independent entitlement checks.
- Mutation ambiguity: inspect side effects/receipts before retry; require declared confirmation.
- Protected worker/capsule missing: do not pretend gateway success; report product/feature/compatibility blocker.

## Required evidence

- current head/version and governing spec digests;
- release manifest/signature/provenance result;
- redacted entitlement state, lease id/sequence/digests, node/product/features result;
- lifecycle transaction/idempotency/confirmation receipt;
- coherent public/protected component versions;
- optional UIAI independent entitlement/child-token audience result;
- health and first-Workpoint result when selected;
- rollback/data-preservation result;
- no-secret scan.

Never capture raw keys, activation secrets, bearer/refresh/child tokens, emails, customer rows, capsule keys/envelopes, or production admin URLs.

## Closure

Done means the requested lifecycle state passes end-to-end proof with entitlement, rollback, and preservation evidence. Spec 150 lifecycle mechanics alone are not customer/evaluator completion; Spec 150A and Spec 152 gates must also pass.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, REST, menubar, TUI, and UIAI bindings through Agent Capability Descriptor V2. Semantics, entitlement feature keys, limit posture, and authority remain identical across surfaces.
