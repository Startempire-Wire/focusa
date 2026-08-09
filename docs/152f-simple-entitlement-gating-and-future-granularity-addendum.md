# Spec 152F — Simple Entitlement Gating and Future Granularity Addendum

**Status:** Normative licensing policy for the `v0.9.144` correction train; implementation accepted with receipts (see docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt); stable release and publication remain forbidden until Spec 152E final closure (focusa-vbcqu.20.13.63) and REL.4–REL.7 close truthfully
**Extends:** Spec 152, Spec 152A–E, and Spec 150A
**Does not weaken:** EDD authority, verified-mailbox requirements, signed runtime leases, refunds, revocation, node binding, sequence monotonicity, or recovery availability
**Primary objective:** Start collecting revenue with a small, understandable paywall while preserving controlled future expansion.

---

## 1. Decision

Focusa SHALL NOT create a separate commercial paywall for every route, command, button, worker, or source file.

The current unmatched-surface inventory is a coverage audit, not a request for hundreds of product decisions. Its entries SHALL resolve through inheritance from a small canonical policy:

1. one base entitlement gate for value-producing Focusa operations;
2. four optional premium capability families;
3. a permanent recovery and customer-control allowance;
4. one authoritative decision in the core/authority layer, inherited by every presenter.

Granular controls MAY be activated later, but their policy dimensions SHALL remain dormant until they satisfy the activation requirements in this addendum.

---

## 2. Core principles

### P1. Revenue-first simplicity

The initial commercial boundary MUST be understandable in one sentence:

> A verified Evaluation or paid Focusa entitlement enables value-producing Focusa work; without one, Focusa remains available for registration, reading, export, recovery, repair, account control, and uninstall.

### P2. Few commercial decisions, broad technical coverage

Hundreds of surfaces MAY be inventoried, but they MUST inherit from a small number of commercial decisions. Inventory size SHALL NOT determine pricing complexity.

### P3. Base entitlement before feature granularity

A valid authority-issued lease for product `focusa` grants the base product. Base capability does not require a separately purchased feature for each core route.

The existing `focusa.core.mission`, `focusa.core.workpoint`, and `focusa.core.evidence` identifiers MAY remain in leases, telemetry, and compatibility projections, but they SHALL initially resolve as parts of the base Focusa product rather than separate purchases.

### P4. Add-ons only at meaningful value boundaries

A premium feature gate is justified only when customers can understand the additional value, the boundary can be enforced at a stable chokepoint, and a product or operational reason exists for selling or limiting it separately.

### P5. Presenters never invent entitlement

REST, CLI, desktop, TUI, Pi, agents, installers, UIAI, workers, and schedulers SHALL project the same canonical decision. A button, command, or facade MUST NOT independently grant, deny, price, or reinterpret entitlement.

### P6. Recovery survives commercial denial

Expired, refunded, revoked, missing, or corrupt entitlement MUST NOT block registration, checkout, activation, license status, account management, customer-data export, repair, stable security updates, rollback, diagnostics, refund handling, or uninstall.

### P7. Authentication is not entitlement

An always-available operation MAY still require verified identity, local authorization, device proof, role permission, confirmation, or rate limiting. “No commercial entitlement required” does not mean anonymous or unrestricted.

### P8. Unknown side effects fail closed

An operation with unknown mutation or side-effect classification MUST NOT silently execute as free or read-only. It MUST be classified or denied before side effects.

### P9. Server-owned grants

Products, prices, tiers, features, limits, commercial use, Evaluation duration, and node allowances remain authority-owned. Clients and branded facades cannot request or expand their own grants.

### P10. Granularity without migration traps

Future granular controls MUST be introduced through versioned policy and lease claims. Existing customers, Evaluations, offline-grace behavior, recovery access, and rollback safety MUST receive explicit migration treatment.

---

## 3. Canonical capability families

| Family | Initial commercial treatment | Canonical examples | Existing feature identifiers |
|---|---|---|---|
| `account_recovery` | Always reachable; identity and security controls still apply | Register, verify email, checkout, activate, status, refresh, refund/account actions, diagnostics, repair, rollback, uninstall | Recovery allowance; not a purchased feature |
| `read_projection` | Available for existing local/customer data when safe | Read status, inspect projects, view evidence, view history, explain denial | No purchased feature required |
| `base_focusa` | Requires valid Evaluation, Active paid lease, or valid Offline Grace | Create/change projects, missions, Focus State, Workpoints, Trajectories, Work Loops, evidence, cognition, and normal value-producing mutations | Product `focusa`; core identifiers are compatibility/projection claims |
| `automation` | Optional premium family | Silent Sessions, unattended work, scheduled execution, subagent concurrency, parallel providers | `focusa.agent.silent_sessions`, `focusa.agent.parallelism` |
| `team_remote` | Optional premium family after activation/bootstrap | Additional devices, peers, team synchronization, remote streaming, multi-operator collaboration | `focusa.team.multi_operator`, `focusa.remote.stream` |
| `release_proof` | Optional premium family | Release orchestration, governed proof bundles, advanced release intelligence | `focusa.release.proof` |
| `premium_updates` | Optional premium family; stable security paths remain available | Unattended updates, preview/nightly channels, managed rollout | `focusa.update.unattended`, `focusa.install.channel.preview`, `focusa.install.channel.nightly` |
| `customer_data_export` | Always available for customer-owned data | Export projects, evidence, history, account data, and recovery packages | `focusa.export.packaged` MAY govern enhanced packaging, never basic customer-data access |
| `internal_maintenance` | Inherits the initiating operation; not independently sold | Schedulers, workers, telemetry, cache maintenance, migration helpers | No independent grant unless promoted under Section 9 |

### 3.1 Stable updates and repair

Manual stable-channel security updates, trust-metadata refresh, repair, and rollback SHALL remain available when commercial entitlement is blocked. Premium update features MAY govern unattended execution, managed channels, and preview/nightly access.

### 3.2 Pairing and devices

Pairing required to activate or recover the licensed node SHALL remain reachable. Adding devices, peers, operators, or remote collaboration beyond the base node allowance MAY require `team_remote` grants and node-limit reservation.

### 3.3 Export

Customers retain access to their data. A paid packaged-export feature MAY add convenience, hosted delivery, transformation, or premium report formats, but MUST NOT prevent basic export of customer-owned records.

---

## 4. Entitlement-state grid

Legend:

- **ALLOW** — no commercial entitlement required; normal auth/security may still apply.
- **READ** — safe read/projection only; no value-producing mutation.
- **BASE** — valid Focusa Evaluation, Active paid lease, or valid Offline Grace required.
- **FEATURE** — BASE plus the named optional feature grant and applicable limit required.
- **CACHED FEATURE** — feature may continue only when already granted by a still-valid cached Offline Grace lease; no grant expansion.
- **DENY** — operation cannot proceed.

| Entitlement state | Account/recovery | Existing-data read | Base Focusa mutations | Automation | Team/remote | Release proof | Stable security update/repair | Premium updates | Customer-data export |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Pending/unverified registration | ALLOW | Local READ where applicable | DENY | DENY | DENY | DENY | ALLOW | DENY | ALLOW for existing local data |
| Verified, no Evaluation or paid grant | ALLOW | READ | DENY | DENY | DENY | DENY | ALLOW | DENY | ALLOW |
| Evaluation | ALLOW | READ | BASE | FEATURE only when Evaluation grant includes it | FEATURE only when Evaluation grant includes it | FEATURE only when Evaluation grant includes it | ALLOW | FEATURE only when Evaluation grant includes it | ALLOW |
| Active paid | ALLOW | READ | BASE | FEATURE | FEATURE | FEATURE | ALLOW | FEATURE | ALLOW |
| Offline Grace | ALLOW where network-independent | READ | BASE | CACHED FEATURE | CACHED FEATURE | CACHED FEATURE | ALLOW | CACHED FEATURE where technically safe | ALLOW |
| Expired | ALLOW | READ | DENY | DENY | DENY | DENY | ALLOW | DENY | ALLOW |
| Refunded or revoked | ALLOW | READ | DENY | DENY | DENY | DENY | ALLOW | DENY | ALLOW |
| Missing, corrupt, or unusable authority state | ALLOW | Local READ | DENY | DENY | DENY | DENY | ALLOW | DENY | ALLOW |

A refund, revocation, or higher authority sequence always overrides older cached grants. Offline Grace cannot create customers, licenses, nodes, purchases, feature expansion, or limit expansion.

---

## 5. Canonical decision order

Every protected execution path SHALL resolve in this order:

1. **Classify the operation:** read, value-producing mutation, recovery, or unknown.
2. **Apply identity and permission controls:** authentication, role, device, confirmation, and scope.
3. **Allow recovery/customer-control operations:** subject to their security requirements.
4. **Allow safe read projection:** without enabling hidden mutation.
5. **Require usable authority state for value-producing mutation:** Evaluation, Active, or valid Offline Grace.
6. **Resolve base versus optional family:** base product or one of the four premium families.
7. **Check limits and reserve atomically:** before side effects when a limit applies.
8. **Execute or deny:** unknown, mismatched, expired, revoked, exhausted, or unclassified mutations fail before side effects.

Licensing grants capability only. It does not grant operator authority, cognitive authority, Workstream authority, Focus State authority, Trajectory authority, Workpoint authority, role permission, or mutation confirmation.

---

## 6. Enforcement chokepoints

Initial enforcement SHOULD remain intentionally small:

1. **Authority and core guard:** validates the signed lease, product, state, sequence, audience, node, expiry, Offline Grace, features, and limits.
2. **API mutation middleware:** applies the canonical decision before REST side effects.
3. **Core execution guard:** protects non-HTTP CLI, agent, worker, scheduler, and embedded-client execution.
4. **Limit reservation service:** atomically reserves limited operations before execution and settles them afterward.

No presenter-specific commercial policy is permitted outside these chokepoints. Presenters MAY improve explanation, recovery guidance, and purchase conversion, but MUST display the canonical decision and error.

---

## 7. Surface inheritance grid

| Surface | Entitlement source | Required behavior | Forbidden behavior |
|---|---|---|---|
| REST route | Linked canonical operation plus API middleware | Declare method, mutation class, recovery status, and operation family | Route-local pricing or caller-selected grants |
| CLI command | Underlying core/API operation | Preflight for fast feedback, then honor canonical execution result | Independent CLI license truth |
| Desktop/menubar action | Underlying API/core operation | Navigation and display remain usable; protected actions explain denial and next step | Per-button product policy |
| TUI action | Underlying API/core operation | Same result and recovery route as CLI/desktop | TUI-only bypass |
| Pi/agent tool | Canonical operation descriptor and core guard | Preflight before side effects; preserve operator/permission checks | Treating tool availability as entitlement |
| Installer | Authority onboarding and activation state machine | Allow install, verify, activate, repair, update, and uninstall flows | Local Evaluation or self-issued lease |
| Worker/scheduler | Initiating operation, persisted reservation, and dispatch-time revalidation | Revalidate before delayed side effects | Continuing after refund/revoke because work was previously queued |
| UIAI/browser adapter | Parent entitlement and bounded child token | Enforce audience, scope, expiry, feature, and parent sequence | Independent UIAI entitlement authority |
| Branded facade | WPUIAI.com EDD authority | Present/proxy the canonical registration, checkout, and recovery flow | Creating customers, grants, keys, or leases independently |
| Test/fixture | Test-only trust roots and deterministic fixtures | Remain visibly non-production | Counting test files as commercial runtime surfaces |

---

## 8. Resolution of the current unmatched inventory

The unmatched inventory SHALL be reconciled as follows; it SHALL NOT become 395 independent paywalls.

| Inventory group | Current count | Resolution |
|---|---:|---|
| REST routes | 199 | Known mutations inherit base or optional-family policy through middleware. The 31 unknown-method routes require metadata repair before execution classification. |
| CLI commands | 86 | Commands inherit their canonical API/core operations. Top-level command names are not independent commercial features. |
| Desktop/menubar click handlers | 83 | Actions inherit the operation they invoke. Navigation, display, recovery, and account controls remain usable. |
| Release/update/export/scheduler file matches | 27 | Exclude seven test-only files. Classify the remaining 20 runtime entrypoints by initiating operation or capability family rather than filename. |
| **Total** | **395** | Reduce to base entitlement, four optional families, recovery/read allowances, and metadata fixes. |

The REST inventory currently groups into these technical families:

| REST technical family | Count | Initial policy direction |
|---|---:|---|
| Automation and agents | 75 | Base for ordinary Work Loop/session operations; premium only for silent, scheduled, parallel, or unattended execution |
| Cognition and evidence | 64 | Base Focusa capability unless later activated as a justified premium family |
| Team, identity, and synchronization | 35 | Activation/recovery pairing allowed; additional devices, peers, sync, and remote collaboration use `team_remote` |
| Project and constitution | 11 | Base Focusa capability |
| Integration, proxy, browser, and MCP | 10 | Base when local; parent-scoped feature/child-token policy where remote or UIAI capability requires it |
| Lifecycle and data | 4 | Stable repair/update/export allowances preserved; premium convenience features may be separately granted |

These counts are audit guidance, not SKU definitions.

---

## 9. Dormant future-granularity model

The authority contract MAY carry dormant dimensions without enforcing or selling them independently:

| Dimension | Examples | Initial state |
|---|---|---|
| Capability family | base, automation, team, release, updates | Active only as defined above |
| Sub-capability | silent sessions, parallel agents, remote stream | Active where already represented by a premium feature |
| Operation | exact API/tool operation identifier | Audit and observability only |
| Limit bucket | nodes, concurrent agents, scheduled runs | Active only for declared server-owned limits |
| Product/tier | Focusa product and commercial plan | Authority-owned and active |
| Role/permission | owner, operator, viewer | Security authorization; never inferred from licensing |
| Node/device | bound node and node allowance | Authority-owned and active |
| Channel | stable, preview, nightly | Stable open for security/repair; premium channels feature-governed |
| Time window | Evaluation expiry, lease expiry, Offline Grace | Authority-owned and active |
| Origin/facade | WPUIAI.com and branded facades | Routing/security policy; facades cannot grant entitlement |

Dormant dimensions MUST NOT deny customer capability merely because a claim is absent. Activation requires the process below.

---

## 10. Future-granularity activation requirements

A new granular gate MUST NOT become release-blocking or customer-visible until all requirements are satisfied:

1. **Business justification:** a named customer value, cost, abuse, capacity, or product reason exists.
2. **Stable family boundary:** enforcement occurs at a canonical operation or core-service boundary, not scattered buttons or commands.
3. **Authority ownership:** EDD/product mapping and authority grants are server-owned; callers cannot request the feature.
4. **Backward compatibility:** existing paid customers, Evaluations, and Offline Grace receive explicit migration behavior.
5. **Recovery analysis:** export, repair, account control, security update, rollback, and uninstall remain available.
6. **Presenter inheritance:** CLI, desktop, TUI, Pi, agents, installers, and facades project the same decision.
7. **Limit semantics:** reservation, idempotency, settlement, and exhaustion behavior are defined when capacity is limited.
8. **Denial UX:** the customer receives a stable error, explanation, recovery/purchase action, and no destructive partial side effect.
9. **Acceptance evidence:** positive, negative, refund/revoke, offline, migration, and bypass tests pass.
10. **Operator approval:** the gate and customer impact are explicitly approved before activation.

A registry entry MAY exist before activation, but it MUST be marked `dormant` and MUST NOT alter runtime authorization.

---

## 11. Customer conversion principles

1. Installation, registration, email verification, and Evaluation activation SHOULD be fast and card-free unless the operator changes commercial policy.
2. Evaluation SHOULD demonstrate the complete base Focusa value loop, not a crippled collection of disconnected screens.
3. Purchase SHOULD continue the same customer, project, node, and data state without reinstallation.
4. Expiry SHOULD transition to read/export/recovery mode with clear renewal guidance.
5. A denial SHOULD identify the blocked capability family and next action without exposing internal secrets or raw authority material.
6. Paid activation SHOULD not require users to understand route names, feature registries, leases, or the 395-surface audit.

---

## 12. Acceptance criteria

This addendum is implemented only when:

- one base entitlement decision protects value-producing core mutations;
- the four premium families are enforced only at canonical chokepoints;
- account, recovery, read, export, stable security update, repair, rollback, and uninstall allowances are tested;
- all 31 unknown-method REST entries are classified or denied before side effects;
- CLI and desktop actions inherit decisions rather than owning commercial policy;
- test-only files are absent from the runtime entitlement frontier;
- workers and schedulers revalidate authority at dispatch;
- Evaluation, Active, Offline Grace, Expired, Refunded, Revoked, and corrupt-state grid cases pass;
- an end-to-end verified Evaluation reaches the first useful Focusa outcome without a card;
- a paid EDD purchase continues into the same canonical customer and licensed runtime;
- refund and revocation remove value-producing capability without removing customer data or recovery access;
- future granular entries remain dormant unless Section 10 is satisfied.

Static inventory success alone is not acceptance. Installed, authority-backed, cross-presenter evidence is required.

---

## 13. Non-goals

This addendum does not:

- turn 395 surfaces into 395 prices, SKUs, or feature flags;
- permit local Evaluation issuance;
- permit facade-owned customers, keys, grants, nodes, or leases;
- permit raw EDD keys to become runtime bearer authority;
- grant operator or cognitive authority through licensing;
- block customer data, repair, account control, refund handling, security update, rollback, or uninstall;
- count the unrestricted demo prerelease as stable licensing acceptance;
- implement or admit Spec 158 into this locked-release scope;
- authorize publication of stable `v0.9.144` before Specs 152/152E/152F and REL.4–REL.7 close truthfully.

---

## 14. Final policy summary

Focusa begins with one understandable commercial boundary:

- **No entitlement:** register, buy, activate, read, export, recover, repair, update securely, and uninstall.
- **Evaluation or paid entitlement:** use the complete base Focusa value loop.
- **Optional grants:** automation, team/remote, release proof, and premium updates.
- **Future granularity:** represented safely, dormant by default, and activated only with business justification, migration, recovery protection, unified enforcement, acceptance evidence, and operator approval.

This structure preserves revenue speed now and technical flexibility later without making customers—or the implementation—carry hundreds of independent paywalls.
