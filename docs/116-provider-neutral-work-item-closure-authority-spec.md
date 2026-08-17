# Spec 116 — Provider-Neutral Work Item Closure Authority

## 0. Status

**Status:** draft v1  
**Scope:** Focusa core, daemon API, CLI, Pi extension, installer, provider adapters, provider guard shims, doctor surfaces, closure audit, and release gates.  
**Authority:** This spec defines Focusa's provider-neutral work-item closure authority. It does not turn Focusa into a task manager and it does not make bd the Focusa task model.

## 1. Purpose

Focusa needs a provider-neutral closure authority that prevents agents and humans from closing work items without validated evidence.

Current task managers can be closed directly, for example:

```bash
bd close focusa-123
```

Late detection in pre-push, CI, or deploy gates is useful, but insufficient. Closure prevention must happen at close time whenever Focusa's closure guard is enabled.

Core invariant:

```text
Focusa validates closure truth. Providers store/display provider state, but provider `closed` never manufactures Focusa `verified_complete`.
```

bd is adapter #1. Linear, Asana, GitHub Issues, and other systems remain future-compatible adapters.

## 2. Normative basis

This spec extends and preserves the direction of:

| Spec | This spec uses it for |
| --- | --- |
| Spec 109 — Agent-First API / AX | typed contracts, bounded side effects, recoverable failures, agent-readable responses |
| Spec 110 — Pi Agent Tool-Layer Reminder Core Feature | Pi/shell detection for raw provider close drift |
| Spec 111 — Agent Context Bootstrap & Delivery | bootstrap visibility into active provider, policy, and close path |
| Spec 112 — Install Binary Architecture | automatic install, verification, doctor, no piecemeal setup |
| Spec 113–115 | already allocated; this feature starts at Spec 116 |
| Spec 131 — Temporal Authority and Closure | factual completion, operator disposition, temporal breach, evidence/Receipt posture, and completion-rollup eligibility remain separate |

## 3. Non-goals

- Focusa is not a replacement task manager.
- Focusa must not lock itself to bd.
- This spec must not require humans or agents to manually assemble hooks, PATH shims, config files, or provider-specific caveats.
- This spec does not promise to prevent every possible out-of-band provider API mutation; it defines layered enforcement and bypass detection.

## 4. Hard requirements

### 4.1 Provider-neutral closure model

Core language must remain provider-neutral:

- `WorkItem`
- `WorkItemRef`
- `WorkItemProvider`
- `ClosureClaim`
- `ClosurePolicy`
- `ClosureValidationResult`
- `ProviderAdapter`
- `ProviderCapabilities`

Provider-specific logic belongs only in adapters, for example:

- `BdWorkItemAdapter`
- `LinearWorkItemAdapter`
- `AsanaWorkItemAdapter`
- `GitHubIssueAdapter`

### 4.2 One obvious close path

Humans and agents close work through Focusa:

```bash
focusa work-item close <id> --from-workpoint
```

or:

```bash
focusa work-item close <id> \
  --summary "..." \
  --evidence "..."
```

Raw provider close commands are guarded when closure policy is enabled.

### 4.3 Auto-installed operational readiness

Focusa installer must install and verify closure prevention automatically.

Installer responsibilities:

1. detect available work-item providers;
2. install the provider adapter;
3. install provider guard shim where supported;
4. wire Pi reminder/guard integration where supported;
5. write closure policy in the canonical location;
6. verify command resolution;
7. run `focusa doctor closure`;
8. report exact active/degraded state.

No default path may require users or agents to search for config files.

### 4.4 Close-time block when enabled

When closure policy mode is `block`, raw provider close commands must block before task mutation when they pass through a Focusa-managed surface or installed guard shim.

Examples:

```bash
bd close focusa-123
bd update focusa-123 --status closed
bd update focusa-123 --status done
linear issue update LIN-123 --state done
asana task complete 123
```

### 4.5 Consistent blocked-reason envelope

Every block must return the same typed reason shape across CLI, API, Pi tools, provider shims, CI, and deploy gates.

No vague failure messages such as `permission denied`, `blocked`, or `invalid` are acceptable without reason details.

Required block envelope:

```json
{
  "schema": "focusa.closure_block.v1",
  "status": "blocked",
  "reason_code": "closure_evidence_missing",
  "blocked_command": "bd close focusa-123",
  "policy": "closure_guard=block",
  "why_blocked": "Raw provider close requires a validated Focusa ClosureClaim before mutation.",
  "missing": ["proof_refs", "spec_refs"],
  "required_next": "focusa work-item close focusa-123 --from-workpoint",
  "doctor": "focusa doctor closure",
  "exit_code": 73
}
```

Human-readable CLI rendering:

```text
BLOCKED: closure evidence missing

Command:
  bd close focusa-123

Reason:
  Raw provider close requires a validated Focusa ClosureClaim before mutation.

Missing:
  - proof_refs
  - spec_refs

Use:
  focusa work-item close focusa-123 --from-workpoint

Inspect:
  focusa doctor closure
```

## 5. Enforcement model

Focusa must use layered enforcement rather than pretending any single mechanism is impenetrable in all environments.

| Layer | Purpose | Guarantee |
| --- | --- | --- |
| Focusa CLI/API | primary closure path | hard block before provider mutation |
| Provider guard shim | intercept raw provider commands | hard block before provider mutation when active |
| Pi reminder/guard | catch agent drift | warn or block depending policy |
| Provider webhook/API audit | detect remote/provider-side bypasses | detect and reconcile when provider supports it |
| pre-push/CI/deploy audit | release safety net | block shipping if invalid closure/bypass is detected |
| doctor | operational proof | show active/degraded guard state and next fix |

Definition of blocked for Focusa-managed commands and installed provider shims:

```text
blocked = no provider mutation attempted
```

Definition of bypass for out-of-band provider mutations:

```text
bypass = detected, audited, and release/deploy blocked until reconciled
```

## 6. Closure policy

```ts
type ClosureGuardMode = "off" | "warn" | "block";
```

| Mode | Behavior |
| --- | --- |
| `off` | no interception |
| `warn` | raw close prints Focusa warning and passes through |
| `block` | raw close is refused unless a valid Focusa `ClosureClaim` exists |

Recommended default for Focusa-owned repositories:

```text
closure_guard = block
```

### 6.1 Config precedence

Policy source order:

1. operator explicit flag;
2. project closure policy;
3. daemon policy;
4. installed default;
5. provider default.

Project-local policy may override global policy.

Example:

```toml
[closure]
mode = "block"
provider = "bd"
require_focusa_claim = true
require_proof_refs = true
require_spec_refs = true
```

## 7. Typed model

### 7.1 Work item provider

```ts
type WorkItemProvider =
  | "bd"
  | "linear"
  | "asana"
  | "github"
  | "unknown";
```

### 7.2 Work item reference

```ts
interface WorkItemRef {
  provider: WorkItemProvider;
  provider_item_id: string;
  project_root: string;
  external_url?: string;
}
```

### 7.3 Evidence citation

```ts
interface EvidenceCitation {
  kind:
    | "code"
    | "spec"
    | "test"
    | "endpoint"
    | "artifact"
    | "workpoint"
    | "ci"
    | "deploy";
  ref: string;
  line?: number;
  result?: string;
  required: boolean;
  verified: boolean;
}
```

### 7.4 Closure claim

```ts
interface ClosureClaim {
  schema: "focusa.closure_claim.v1";
  claim_id: string;
  idempotency_key: string;

  work_item: WorkItemRef;
  project_root: string;
  continuity_id: string;
  workpoint_id?: string;

  actor_id: string;
  agent_session_id?: string;

  closure_summary: string;
  closure_kind:
    | "code"
    | "docs"
    | "deploy"
    | "investigation"
    | "no_code"
    | "admin";

  code_refs: EvidenceCitation[];
  spec_refs: EvidenceCitation[];
  proof_refs: EvidenceCitation[];
  endpoint_refs?: EvidenceCitation[];
  artifact_refs?: EvidenceCitation[];

  git_head_sha?: string;
  git_dirty_state: "clean" | "dirty" | "unknown";

  policy_version: string;
  validator_version: string;

  validation_status: "valid" | "blocked" | "expired";
  created_at: string;
  expires_at: string;

  provider_mutation_plan: ProviderMutationPlan;
}
```

### 7.5 Provider mutation plan

```ts
interface ProviderMutationPlan {
  provider: WorkItemProvider;
  action: "close" | "comment" | "status_update" | "close_with_comment";
  target_status?: string;
  comment_body?: string;
  idempotency_key: string;
}
```

### 7.6 Provider capabilities

```ts
interface ProviderCapabilities {
  provider: WorkItemProvider;
  can_intercept_raw_close: boolean;
  can_comment: boolean;
  can_set_status: boolean;
  can_reopen: boolean;
  supports_webhooks: boolean;
  supports_local_hooks: boolean;
  supports_idempotency: boolean;
}
```

## 8. Evidence profiles

Validation depends on closure kind.

| Closure kind | Required evidence |
| --- | --- |
| `code` | code refs plus proof/test refs |
| `docs` | doc refs plus review/proof refs |
| `deploy` | release, CI, or deploy refs |
| `investigation` | artifact/log refs plus conclusion |
| `no_code` | explicit no-code rationale plus proof refs |
| `admin` | command/output refs plus policy reason |

Weak schema satisfaction is invalid. A random docs path must not close a code task. A test path with no result summary must not close a release task.

## 9. Closure lifecycle

```text
prepare → validate → authorize → submit → reconcile → audit
```

### 9.1 Prepare

```bash
focusa work-item closure prepare <id> --from-workpoint
```

Collects:

- active Workpoint;
- evidence refs;
- project identity;
- trajectory gap;
- provider info;
- git state;
- closure policy.

### 9.2 Validate

Validation checks:

- work item/provider exists;
- project_root and continuity_id match;
- evidence satisfies closure profile;
- cited refs exist or are durable handles;
- git state is acceptable for policy;
- policy version is current;
- claim is not expired.

### 9.3 Authorize

Focusa authorizes mutation only when validation status is `valid` and policy permits the actor/session.

### 9.4 Submit

Provider adapter mutates the task manager only after authorization.

### 9.5 Reconcile

After provider mutation, Focusa must:

- verify provider status changed;
- write audit record;
- link closure evidence to Workpoint;
- emit result envelope;
- record bypass or partial-failure state when applicable.

## 10. Provider guard shim

Installed command resolution for bd adapter:

```text
bd → focusa provider-guard bd → real bd
```

Guarded bd commands:

```bash
bd close
bd update --status closed
bd update --status done
```

Allowed bd commands:

```bash
bd show
bd list
bd ready
bd update --status in_progress
```

Equivalent guarded operations must be defined per provider capability for Linear, Asana, GitHub, and future adapters.

## 11. API surface

```text
GET  /v1/work-items/providers
GET  /v1/work-items/closure/policy
POST /v1/work-items/closure/prepare
POST /v1/work-items/closure/validate
POST /v1/work-items/closure/submit
POST /v1/work-items/provider-guard/evaluate
GET  /v1/doctor/closure
```

All blocked/failure responses use `focusa.closure_block.v1`.

## 12. CLI surface

```bash
focusa work-item close <id> --from-workpoint
focusa work-item closure prepare <id>
focusa work-item closure validate <claim-id>
focusa work-item closure submit <claim-id>
focusa work-item provider-guard evaluate --provider bd --command "bd close <id>"
focusa doctor closure
```

Doctor output must show:

```text
Closure prevention: active
Mode: block
Provider: bd
Adapter: installed
Raw close guard: active
Command intercepted: bd
Real provider binary: /usr/local/bin/bd
Correct close path:
  focusa work-item close <id> --from-workpoint
```

## 13. Installer behavior

Installer must run the equivalent of:

```bash
focusa install closure-guard --auto
```

It must:

1. detect provider;
2. install adapter;
3. install guard shim where supported;
4. wire Pi reminder/guard integration;
5. write policy;
6. verify command resolution;
7. run doctor;
8. report exact state.

If guard cannot activate, installer must clearly report:

- why it could not activate;
- whether closure prevention is degraded;
- the exact command to inspect state;
- whether installation can proceed under current policy.

## 14. Break-glass disposition without false completion

Emergency operator disposition exists, but it cannot override factual completion or manufacture evidence.

Preferred explicit form:

```bash
focusa work-item dispose <id> --accepted-risk --reason "..."
focusa work-item dispose <id> --cancelled --reason "..."
```

A compatibility input such as:

```bash
focusa work-item close <id> --override --reason "..."
```

MUST be translated into a typed non-completion disposition or rejected; it MUST NOT set Focusa `verified_complete`.

Rules:

- disabled by default for agents;
- requires explicit operator policy, exact scope, reason, and Receipt;
- writes `closure_disposition` with `accepted_risk|waived_waivable_policy|scope_removed_by_amendment|cancelled|abandoned`;
- cannot waive required evidence, failed checks, outcome truth, reconciliation, safety, or immutable external obligations;
- provider state may be closed when necessary, but Focusa retains factual `implemented_unverified|failed|cancelled` and excludes it from verified completion/release/velocity rollups;
- CI/deploy/release gates remain blocked unless independent applicable completion requirements are verified or formally removed by specification amendment;
- block messages explain available dispositions without calling them verified completion.

## 15. Migration and rollout

Strict closure policy cannot retroactively apply to all historical closed tasks without migration.

Policy fields:

```text
legacy_cutoff = timestamp for historical tolerated closures
strict_effective_at = timestamp when strict close policy begins
```

Audits report:

- legacy tolerated closures;
- strict violations;
- bypass attempts;
- missing evidence;
- provider guard status.

## 16. Security and secrets

Provider adapters must not leak tokens or secrets in:

- block messages;
- audit records;
- Pi tool responses;
- CI logs;
- doctor output.

Adapters must use least-privilege provider tokens and support revocation reporting in `focusa doctor closure`.

## 17. Race, idempotency, and failure handling

Closure submission must be idempotent.

Required handling:

- claim validated, then files changed → revalidate or block stale claim;
- task already closed remotely → reconcile and emit status;
- provider mutation succeeds but Focusa audit write fails → emit partial failure and repair instruction;
- double submission → return prior result by idempotency key;
- network failure after provider close → reconcile before retry.

## 18. Observability

Focusa must track:

```text
blocked_closure_attempts
invalid_claims
raw_provider_close_attempts
provider_guard_active
provider_guard_degraded
last_policy_check
last_closure_audit
```

`focusa doctor closure` must expose these in compact form.

## 19. Acceptance criteria

- Raw `bd close` blocks at close time when guard mode is `block` and shim is active.
- Block message explains exactly what was blocked, why, what is missing, and next command.
- Valid `focusa work-item close --from-workpoint` closes provider item.
- Same block envelope appears in CLI, API, Pi tool, provider shim, CI, and deploy audit.
- Installer auto-installs closure guard and verifies with doctor.
- bd adapter works without bd concepts leaking into core model.
- Linear/Asana/GitHub adapter capability interfaces exist.
- Closure claims are idempotent and expire.
- Bypassed raw closures are detected by audit/CI/deploy.
- Migration policy distinguishes legacy tolerated closures from strict violations.

## 20. Final principle

Focusa is not a task manager.

Focusa is the closure authority: typed, provider-neutral, evidence-backed, auditable, and agent-readable.
