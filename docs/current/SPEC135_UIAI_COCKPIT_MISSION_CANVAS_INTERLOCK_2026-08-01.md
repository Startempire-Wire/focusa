# Spec 135 Current Correction — UIAI Cockpit and Focusa Mission Canvas Interlock

**Status:** current normative cross-spec correction; implementation not implied  
**Date:** 2026-08-01  
**Owner:** Focusa / Verious Smith  
**Series posture:** Spec 135 remains frozen at 135K; this is not Spec 135L  
**Amends:** Spec 135A, 135C, 135E, 135G, 135I, and 135J  
**Machine authority:** `docs/contracts/spec135-uiai-cockpit-mission-canvas-interlock.v1.yaml`  
**Host authority:** `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`

---

## 0. Constitutional correction

Focusa Mission Canvas and UIAI Engine Cockpit are two distinct graphical products over one governed system. They MUST be deeply integrated, but they MUST NOT become competing mission stores, task systems, Work Rails, approval systems, schedules, credential authorities, or completion ledgers.

```text
Focusa
  canonical Mission Kernel and project/work authority

Focusa Mission Canvas
  canonical mission, professional-workspace, Work Surface,
  Work Rail, steering, proof, and continuity projection

UIAI Engine
  execution, browser, research, notebook, document, media,
  diagnostic, artifact, resource, and proof-production plane

UIAI Engine Cockpit
  rich execution, verification, inspection, and operator-control
  environment that may host bounded Focusa Mission Canvas projections
```

The prior shorthand `Cockpit = Mission Experience layer` is superseded wherever it implies that Cockpit owns mission state, Workpoints, Work Rail, steering, next-safe-action cognition, Evidence meaning, completion, or settlement. Cockpit is the rich execution and operator shell. Mission Canvas is Focusa's canonical mission and professional-workspace experience.

The governing rule is:

```text
Mission Canvas answers:
  What work exists?
  Why does it exist?
  Which project, Workpoint, session, and Attachment own it?
  What is current and next?
  What authority permits or blocks it?
  What proof exists?
  Is the outcome verified or settled?

Cockpit answers:
  How is the work being executed?
  Which browser, notebook, document, test, node, or resource is active?
  What can the operator observe or control now?
  Which artifacts, diagnostics, and technical receipts were produced?
```

No visual host, focused tab, browser target, notebook, job, or local Cockpit store becomes canonical project, session, Workpoint, task, authority, or completion state.

---

## 1. Existing ownership remains authoritative

### 1.1 Focusa owns

- `ProjectRootKey`, `WorkstreamKey`, `AttachmentKey`, and exact scope;
- ProjectIdentity and Project Operating Profile;
- C.R.I.S.T. Context, Role, Interview, Spec, and Tasks state;
- Mission, Trajectory, Workpoints, work items, and Work Rail;
- Instances, Sessions, Attachments, Work Surfaces, and Mission Canvas restoration;
- steering and follow-up queues;
- capabilities, permissions, approvals, proposals, and authority;
- semantic Context, candidate/canonical distinction, and domain packs;
- Evidence meaning, verification posture, Receipts, completion, and settlement;
- next-safe-action cognition;
- recurring mission intent and temporal scheduling authority;
- durable canonical events, operation registry, and generated action bindings.

### 1.2 UIAI Engine owns

- browser sessions, browser contexts, browser targets, and browser profiles;
- browser actuation, FPV, operator mouse/keyboard control, and target observation;
- DOM/accessibility snapshots, screenshots, visual comparison, console, and network diagnostics;
- research search/read/capture, source-to-Markdown, and browser-derived artifacts;
- notebook providers, kernels, cells, variables, outputs, computation, and simulation execution;
- document rendering, conversion, annotation execution, media jobs, and Test Lab runners;
- execution resource accounting and local operational telemetry;
- secret storage, opaque secret handles, and approved credential injection;
- technical Action Receipts, artifacts, and execution evidence candidates;
- UIAI-local safety freeze and actuator reconciliation.

### 1.3 Cockpit owns only client state

Cockpit may own ephemeral drafts, selected panels, open local inspectors, viewport state, focus, local cache, and unsent forms. It MUST NOT own a canonical mission, Workpoint, task, Work Rail item, approval, CredentialUseGrant, recurrence policy, completion fact, or Focusa event history.

### 1.4 Mission Canvas owns only projection state

Mission Canvas presentation state may include open/pinned/grouped/split Work Surfaces, focused surface, density, theme, and accessibility preferences. Presentation state MUST NOT grant operational authority or replace runtime identity.

---

## 2. Cross-GUI product model

### 2.1 Pi-native Mission Canvas remains primary

The Focusa-enhanced Pi distribution remains the primary authoritative interactive Mission Canvas projection. Cockpit integration does not replace the Pi-native Canvas, require Cockpit for ordinary Focusa use, or transfer renderer authority to UIAI Engine.

### 2.2 Cockpit may host conformant Focusa projections

Cockpit MAY host four bounded levels of Focusa projection:

1. **Mission Context Strip** — compact project, Workpoint, Work Surface, authority, proof, and freshness context on every Focusa-bound UIAI work object.
2. **Mission Inspector** — bounded Focusa read model and typed Focusa actions for current work, next-safe action, authority, proof, Attachments, contention, and Receipts.
3. **Hosted Mission Canvas Dock** — an actual Focusa-owned generated Mission Canvas projection rendered through the approved Focusa client package and action bindings.
4. **Full Mission Canvas Work Object** — a first-class Cockpit tab/window containing the complete Focusa Mission Canvas projection for project overview, multiplexed Work Surfaces, Work Rail, queues, and governed controls.

Levels 3 and 4 MUST consume Focusa-generated contracts and trusted renderers. Cockpit MUST NOT hand-author an imitation Work Rail, steering queue, next-safe-action engine, or Mission Canvas reducer.

### 2.3 Mission Canvas may host bounded UIAI projections

Mission Canvas may present UIAI Work Surfaces containing:

- UIAI session, context, and target identity;
- execution lifecycle and health;
- browser profile and isolation posture;
- node/runner/resource posture;
- current target URL/title and last observation;
- credential grant posture without secret material;
- operator-control state;
- artifact and diagnostic counts;
- proof readiness, freshness, and retention;
- `Open in Cockpit`, `Take control`, `Pause`, `Inspect artifacts`, and other typed actions.

Mission Canvas MUST NOT reproduce full browser chrome, raw DOM editing, complete network waterfalls, raw secret values, notebook editors, complete Test Lab, or visual-comparison tooling. Those remain Cockpit-native execution surfaces.

---

## 3. Canonical Work Surface to UIAI Work Object binding

Every mission-bound UIAI work object MUST bind through a versioned Focusa record.

```yaml
schema: focusa.uiai_work_surface_binding.v1
binding_id:
binding_revision:

scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workpoint_id:
  work_item_ref:

focusa:
  instance_id:
  session_id:
  attachment_id:
  work_surface_id:
  mission_ref:

uiai:
  work_object_id:
  work_object_kind: browser | research | document | notebook | test_run | visual_compare | automation_run | report | media_job
  uiai_session_id:
  browser_context_id:
  browser_target_ids: []
  notebook_session_id:
  document_session_id:
  test_run_id:
  job_id:

control:
  access_mode: observe | interact | execute | verify
  authority_posture:
  operator_control_state:
  browser_isolation_class:
  authentication_sharing:
  credential_grant_refs: []
  resource_policy_ref:
  retention_policy:

freshness:
  source_state_revision:
  event_cursor:
  observed_at:
  stale_after:

created_at:
updated_at:
closed_at:
```

### 3.1 Binding laws

1. Focusa mints the canonical binding ID and exact project/workstream/Attachment scope.
2. UIAI supplies stable work-object and runtime references but does not mint Focusa authority.
3. A Cockpit object may remain local and unbound; it MUST be labeled `Cockpit local` and cannot appear as canonical project work.
4. A mission-bound object cannot silently change project, workstream, Workpoint, Attachment, browser context, or credential grant.
5. Rebinding requires preview, expected revisions, idempotency, and a canonical Focusa event.
6. Closing a Cockpit view does not close the Focusa Work Surface or UIAI runtime unless an explicit governed action says so.
7. Closing a Mission Canvas Work Surface does not terminate the UIAI runtime by implication.
8. Aggregate Cockpit views are read-only until an explicit mutation target is selected.
9. Cross-project binding is forbidden by default.
10. Missing or stale Focusa binding produces a visible degraded state, never implicit local ownership.

---

## 4. Tab, page, document, and notebook to Workpoint handoff

Cockpit SHALL provide one reusable **Hand Off to Focusa** interaction rather than separate task-creation systems per workspace.

### 4.1 UIAI capture responsibility

UIAI captures bounded context references, including applicable:

- current page and selected region;
- UIAI session, browser context, and target;
- screenshot, DOM/accessibility snapshot, console, and network refs;
- document, notebook, cell, dataset, test, comparison, or artifact refs;
- authenticated-session posture without cookies or secret values;
- requested outcome and expected technical evidence;
- untrusted content classifications;
- retention and cleanup posture.

### 4.2 Focusa commitment responsibility

Focusa determines through typed preview/commit operations whether the request:

- attaches to an existing Workpoint;
- proposes a new Workpoint;
- becomes steering or follow-up for an existing Attachment;
- remains a Context/Evidence candidate;
- is rejected, duplicated, blocked, or requires scope clarification;
- creates a Work Surface and binding.

Cockpit MUST NOT create a local canonical task and synchronize it later.

### 4.3 Handoff sequence

```text
UIAI work object
→ capture bounded refs and operator instruction
→ classify trusted instruction versus untrusted context
→ Focusa intake/handoff preview
→ operator confirms scope and consequence
→ Focusa commits Workpoint/Attachment/Work Surface/binding as applicable
→ Focusa emits canonical events
→ Cockpit and Mission Canvas refresh from the same state
```

---

## 5. Typed cross-surface intake

Browser, email, mobile, menubar, Slack, Agent Inbox, document, notebook, and other intake surfaces MUST converge on one general Focusa primitive.

```yaml
schema: focusa.task_intake_envelope.v1
intake_id:
origin_surface:
origin_instance_ref:
verified_principal_ref:
trusted_instruction:
untrusted_context_refs: []
attachment_refs: []

suggested_scope:
  project_root:
  continuity_id:
  workpoint_id:

requested_outcome:
expected_evidence: []
suggested_priority:
requested_delivery_route:

trust:
  sender_verified:
  quoted_text_is_instruction: false
  page_content_is_instruction: false
  document_text_is_instruction: false
  attachment_text_is_instruction: false
  tool_output_is_instruction: false

status: proposed | scope_review | accepted | attached | rejected | duplicate | blocked
created_at:
```

Only a verified principal's explicit instruction may carry operator authority. Page content, forwarded messages, quoted text, attachments, model output, tool output, and third-party content remain untrusted context until separately authorized.

Cockpit may display the Focusa intake lifecycle but MUST NOT own accepted/assigned/completed state.

---

## 6. Reversible operator takeover and reconciliation

Takeover spans two coordinated but non-duplicated state machines.

### 6.1 Focusa session-level intervention

```text
running
→ pause_requested
→ paused_for_operator
→ operator_intervention
→ resume_proposed
→ resumed | redirected | stopped | blocked
```

Focusa owns the durable decision, affected Attachment/run, operator steering, and resulting mission state.

### 6.2 UIAI actuator-control lease

```text
agent_controlled
→ local_freeze
→ operator_controlled
→ operator_delta_capture
→ reobservation_required
→ agent_controlled | terminated
```

UIAI owns the browser/computer control lease, immediate local safety freeze, viewport/target control, and observable state delta.

### 6.3 Immediate safety freeze

Cockpit MAY freeze local actuation immediately before Focusa confirms a pause when continuing could create risk. Until canonical acknowledgment it MUST display:

```text
local safety freeze
Focusa reconciliation pending
```

It MUST NOT claim the Focusa session is canonically paused.

### 6.4 Operator delta receipt

Before an agent resumes, UIAI SHALL create an observable delta such as:

```yaml
schema: uiai.operator_delta_receipt.v1
control_lease_ref:
work_surface_binding_ref:
started_at:
ended_at:
changed:
  - navigation
  - selected account
  - form values
  - file attachments
  - browser target set
not_performed:
  - final submit
current_state_refs:
  - screenshot
  - dom_snapshot
  - url
pending_side_effect:
reobservation_required: true
```

The agent MUST re-observe and reconcile instead of continuing from stale assumptions.

### 6.5 Control placement

Mission Canvas may show `Take control`, `Pause`, `Stop`, and `Open in Cockpit`. `Take control` focuses or opens the bound Cockpit execution surface. Mission Canvas does not implement the browser viewport.

Cockpit may show `Pause`, `Stop`, and `Return control`, but session-level actions route through generated Focusa operations. Cockpit-local actuator controls do not become Focusa lifecycle authority.

---

## 7. Scoped Credential Broker split

Credential handling is divided between mission authority and secret custody.

### 7.1 Focusa CredentialUseGrant

```yaml
schema: focusa.credential_use_grant.v1
grant_id:
grant_revision:
scope:
  project_root:
  continuity_id:
  attachment_id:
  work_surface_id:
requesting_actor_ref:
capability_ref:
purpose:
allowed_origins: []
allowed_operation_classes: []
side_effect_ceiling:
spend_ceiling:
use_limit:
issued_at:
expires_at:
approval_ref:
revoked_at:
receipt_required: true
```

Focusa owns whether use is authorized. The grant contains no secret.

### 7.2 UIAI SecretBinding

```yaml
schema: uiai.secret_binding.v1
secret_binding_id:
credential_ref:
vault_provider:
target_origins: []
allowed_routes: []
allowed_methods: []
injection_mode: header_proxy | browser_fill | connector | environment_proxy
active_lease_ref:
use_count:
last_used_at:
rotation_state:
revocation_state:
```

UIAI owns secret storage and injection. Agents, Focusa events, prompts, artifacts, and Mission Canvas never receive raw secret values.

### 7.3 Use law

```text
Focusa authorizes use but never possesses the secret.
UIAI possesses/injects the secret but cannot authorize itself.
```

Every consequential use produces a secret-safe receipt linking the Focusa grant, UIAI binding, origin, operation class, time, result, and revocation posture.

Mission Canvas displays only availability, approval, expiry, revocation, origin mismatch, usage boundary, and affected Work Surface. Secret configuration and rotation remain Cockpit-native.

---

## 8. Workflow context and recurrence split

Mission-bound recurring work MUST NOT exist as independent schedules in both Focusa and UIAI.

### 8.1 Focusa owns `WorkflowContextPolicy`

```yaml
schema: focusa.workflow_context_policy.v1
policy_id:
scope:
mission_ref:
workpoint_policy:
schedule_or_trigger_ref:
completion_contract_ref:
authority_ref:

frozen_refs: []
refresh_each_run: []
carry_forward: []
prohibited_reuse: []

freshness_requirements: []
evidence_requirements: []
settlement_policy_ref:
```

Focusa owns purpose, recurrence, temporal authority, Workpoint continuity, context freshness, completion, and settlement.

### 8.2 UIAI owns `ExecutionContextManifest`

```yaml
schema: uiai.execution_context_manifest.v1
run_id:
workflow_context_policy_ref:
work_surface_binding_ref:
runner_and_node_refs: []
browser_context_refs: []
notebook_environment_refs: []
document_and_dataset_refs: []
credential_grant_and_binding_refs: []
tool_and_provider_versions: []
resource_budget_ref:
actual_inputs: []
actual_outputs: []
diagnostics_refs: []
receipt_refs: []
```

UIAI records what was actually used. It does not decide mission freshness or completion.

### 8.3 Local utility exception

A Cockpit-local utility automation may exist without Focusa while it is explicitly unscoped and non-mission-bearing. Once it is project-, mission-, Workpoint-, Evidence-, or outcome-bound, Focusa becomes the canonical recurrence and continuity owner.

---

## 9. Projection contracts

### 9.1 Mission Context Strip

Every Focusa-bound Cockpit object SHALL show, when available:

- project/workstream;
- Workpoint and work item;
- Focusa Work Surface;
- mission/session status;
- one next-safe action;
- authority/approval posture;
- proof posture;
- source state revision, cursor, and freshness;
- `Open Mission Canvas`.

The strip MUST NOT reproduce the Work Rail.

### 9.2 Mission Inspector

The inspector may show bounded Focusa projections for:

- Mission;
- Current work;
- Next-safe action;
- Authority;
- Proof;
- Attachments;
- Contention;
- Receipts.

Every mutation control uses a generated `focusa.ui_action_binding.v1`. Cockpit cannot derive routes, permissions, confirmation, or completion from labels.

### 9.3 UIAI Work Surface projection in Mission Canvas

A UIAI Work Surface SHALL show bounded:

- execution lifecycle and health;
- session/context/target or notebook/document/test identities;
- isolation and authentication-sharing posture;
- current observed object;
- control state;
- active CredentialUseGrant posture;
- resource and node posture;
- artifact/diagnostic/proof status;
- retention and cleanup;
- deep links and typed controls.

### 9.4 Deep-link law

Deep links MUST contain stable opaque refs, never secrets or inline canonical payloads. Opening a link performs a fresh scope, capability, permission, existence, revision, and freshness check. A stale link opens a recovery surface rather than silently binding a replacement object.

---

## 10. Operation and event integration

### 10.1 Focusa operation families

The generated Operation Registry SHALL own operation descriptors for:

- Mission Canvas projection reads;
- Work Surface list/create/arrange/suspend/resume/close-view;
- UIAI Work Surface binding list/preview/commit/unbind;
- task intake preview/commit/reject;
- Workpoint attach/create proposal;
- operator intervention preview/commit/reconcile;
- CredentialUseGrant preview/commit/revoke;
- WorkflowContextPolicy preview/commit/revise;
- Evidence/artifact capture and linkage;
- deep-link resolution and recovery.

Operation IDs, routes, schemas, capabilities, permissions, confirmations, idempotency, optimistic concurrency, reversibility, and receipt requirements derive from Rust/OpenAPI and generated contracts. UIAI MUST NOT maintain a handwritten duplicate operation catalog.

### 10.2 Focusa to UIAI events

Required event classes include:

- Mission Canvas projection invalidated;
- Work Surface created/revised/suspended/resumed/closed;
- UIAI binding created/revised/unbound/stale;
- Workpoint or work-item binding revised;
- intake state revised;
- operator intervention revised;
- CredentialUseGrant issued/expired/revoked;
- WorkflowContextPolicy revised;
- Evidence/Receipt linked;
- capability/permission/approval changed;
- source state revision changed.

Events carry stable refs, exact scope, revision, cursor, causation, and invalidation keys. Large payloads remain behind handles.

### 10.3 UIAI to Focusa law

UIAI does not inject canonical Focusa events directly. It invokes typed Focusa operations with stable artifact, diagnostic, execution-manifest, control-delta, and receipt refs. Focusa validates and emits canonical events.

### 10.4 Reconciliation

Cockpit may show local operational truth before Focusa commitment only with explicit labels such as:

- `binding pending`;
- `evidence capture pending`;
- `local safety freeze`;
- `Focusa reconciliation pending`;
- `offline projection`;
- `stale Focusa state`.

It may not show canonical success, completion, or settlement until the Focusa state revision acknowledges it.

---

## 11. No-duplication matrix

| Concern | Canonical Focusa/Mission Canvas role | UIAI/Cockpit role |
|---|---|---|
| Project, mission, Trajectory, Workpoint | owner | projection |
| Work Rail | owner | never duplicate |
| steering/follow-up | owner | controls route to Focusa |
| session/Attachment lifecycle | owner | execution adapter and telemetry |
| browser context/target | governing binding projection | runtime owner |
| notebook/document/test execution | Workpoint, claim, proof posture | runtime/editor/runner owner |
| Activity | mission/project canonical history | execution jobs/logs/diagnostics |
| Evidence | semantic meaning and settlement | artifact production/inspection |
| Automation | mission intent, recurrence, continuity | recipe execution and telemetry |
| Credential | CredentialUseGrant authority | secret custody/injection |
| Intake | canonical proposal lifecycle | intake producer/origin display |
| Pause/stop | canonical lifecycle decision | actuator action and safety freeze |
| Completion | verification and settlement owner | technical result and receipt producer |

Any implementation introducing a second Work Rail, mission task store, steering queue, schedule authority, permission system, approval ledger, credential-use authority, or completion reducer in Cockpit is nonconformant.

---

## 12. Required amendments by existing Spec 135 document

### 12.1 Spec 135A

Mission Canvas composition MUST include UIAI Work Surface projections, stable `Open in Cockpit` navigation, Mission Context Strip requirements for hosted clients, and the prohibition against Cockpit-local Work Rail duplication.

### 12.2 Spec 135C

The UIAI bridge MUST carry not only rich artifacts but binding, intake, takeover, credential-posture, workflow-context, execution-manifest, resource, and reconciliation refs. UIAI artifacts remain evidence candidates until Focusa captures/links them.

### 12.3 Spec 135E

Cross-repository compatibility MUST lock the Focusa/UIAI generated-contract handshake, version negotiation, unavailable/degraded states, migration, rollback, and zero-unknown-impact closure for the interlock.

### 12.4 Spec 135G

Work Surface binding MUST preserve UIAI work-object identity, control lease, browser/notebook/document/test refs, exact deep links, close-versus-terminate semantics, and restoration across both products.

### 12.5 Spec 135I

Cockpit-hosted Focusa surfaces MUST use the trusted Focusa A2UI/Lit/Svelte Custom Element stack and generated action bindings. Cockpit MUST NOT implement a second Focusa surface model or derive authority from component copy.

### 12.6 Spec 135J

The Operation Registry and durable Focusa event stream MUST cover cross-GUI bindings, intake, intervention, credential grants, workflow context, and reconciliation. UIAI adapters are generated from published OpenAPI; no handwritten duplicate DTO/operation registry is allowed.

This correction governs those subjects until the affected documents are edited directly in a future consolidation pass.

---

## 13. Implementation order

```text
I0 — freeze ownership and terminology in machine-readable contracts
I1 — generate Focusa interlock schemas, operations, and TypeScript client
I2 — implement UIAI Focusa Projection Adapter and compatibility handshake
I3 — implement Work Surface ↔ UIAI Work Object binding and deep links
I4 — implement Mission Context Strip and Mission Inspector
I5 — implement Tab/Page/Document/Notebook → Workpoint handoff
I6 — implement operator control lease, takeover, delta, and reconciliation
I7 — implement CredentialUseGrant + UIAI SecretBinding broker
I8 — implement typed intake across browser/email/mobile/Agent Inbox surfaces
I9 — implement WorkflowContextPolicy + ExecutionContextManifest
I10 — host conformant Focusa Mission Canvas projections in Cockpit
I11 — complete cross-GUI reconnect, restoration, security, accessibility, and release proof
```

No later phase may bypass exact scope, generated operations, durable replay, or truthful degraded states to demonstrate UI early.

---

## 14. Functional and visual proof

Integration is not complete until the same build/revision proves:

1. Mission Context Strip reflects a real Focusa projection and updates after a canonical event.
2. `Open Mission Canvas` and `Open in Cockpit` resolve exact stable bindings.
3. A Cockpit page handoff creates or attaches a real Focusa Workpoint/Work Surface through preview/commit.
4. Mission Canvas reflects the resulting UIAI session/context/target without duplicating Cockpit internals.
5. Takeover produces local freeze, canonical intervention, operator control, delta receipt, reobservation, and resume.
6. Credential use succeeds only with matching Focusa grant and UIAI secret binding; raw secrets never appear in events, screenshots, logs, prompts, or receipts.
7. Workflow recurrence has one Focusa authority record and one UIAI execution manifest, not two schedules.
8. Disconnect/reconnect replays Focusa events, preserves UIAI runtime identity, and restores both GUIs without adopting a replacement scope.
9. Closing a view versus terminating a runtime remains distinct in both GUIs.
10. Aggregate views cannot mutate without an explicit target.
11. Browser, notebook, document, research, and Test Lab objects share one binding model.
12. Accessibility, narrow viewport, reduced motion, keyboard operation, stale/offline, blocked, conflict, revoked, expired, and origin-mismatch states are visibly proven.
13. UIAI Engine Eval supplies actual browser and cross-GUI screen-capture evidence, DOM/accessibility assertions, diagnostics, hashes, build/commit refs, and Evidence/Receipt refs.
14. No static card, handwritten success JSON, fixture-specific reducer, manually supplied screenshot, or local task store can satisfy closure.

---

## 15. Closure rule

The interlock remains open until:

```text
one Focusa canonical mission authority
+ one Mission Canvas work projection
+ one UIAI execution/proof plane
+ generated cross-repository contracts
+ exact Work Surface bindings
+ typed handoff and intake
+ reversible control reconciliation
+ split credential authority/custody
+ one recurrence authority
+ bidirectional stable refs and durable events
+ functional assertions and inspected screen-capture proof
```

are verified together.

Two polished GUIs that disagree about project, Workpoint, session, authority, credentials, recurrence, proof, or completion are a release-blocking architectural failure.