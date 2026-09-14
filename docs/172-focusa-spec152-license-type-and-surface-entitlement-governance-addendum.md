# Spec 172 — Spec 152 License Type and Surface Entitlement Governance Addendum

**Status:** Normative, release-blocking addendum; implementation and reconciliation remain open  
**Extends:** Specs 150A, 152, 152A–F  
**Supersedes only:** conflicting Evaluation, initial product-model, bundle-price, and future-feature inheritance clauses identified in Section 3  
**Does not weaken:** verified-mailbox control, WPUIAI.com EDD authority for paid commerce and human keys, signed runtime assertions and leases, device binding, refund/revoke settlement, monotonic sequence, security authorization, privacy, customer-data preservation, or recovery availability  
**Primary objective:** Keep licensing simple while preserving durable distinctions among verified no-license access, products, License Types, surfaces, future capability families, and future products.

---

## 1. Operator decisions

This addendum records the following decisions as one coherent policy:

1. Product runtime access is not anonymous. A submitted email alone is insufficient; mailbox control MUST be verified.
2. A verified account with no paid license receives permanent, feature-limited access. It is not a license, free license, trial, or time-bounded Evaluation.
3. There is no countdown or automatic expiry for verified no-license access.
4. Paid rights are represented by a product-scoped License Type. `Operator` is the initial License Type; a future type may be named `Navigator` or another operator-approved name.
5. `Operator` MUST NOT be removed merely because future License Types are expected. When discontinued, it stops accepting new sales while existing rights remain valid.
6. A lifetime term applies only to the products, capability families, limits, and local-runtime rights included in that License Type.
7. A lifetime License Type does not automatically include future License Types, new products, newly introduced capability families, hosted services, or metered third-party resources.
8. Focusa Operator Lifetime is USD $697.00 before applicable taxes.
9. UIAI Engine Operator Lifetime is USD $697.00 before applicable taxes.
10. The Focusa + UIAI Operator Lifetime Bundle is exactly ten percent below the sum of the two standalone prices: USD $1,254.60 before applicable taxes.
11. The Bundle is a commerce SKU that grants the two underlying License Types. It is not a third independent feature catalog.
12. There have been no approved customer sales of these Focusa/UIAI offers. That assertion MUST be verified across every payment and issuance rail before a no-migration cutover is accepted.
13. New tools within an existing included capability family inherit that family. A materially new customer outcome requires a new family and is excluded by default until assigned.
14. New products are excluded from existing License Types and Bundles by default.
15. Cockpit, Focusa Desktop, menubar, CLI, TUI, Pi, installers, agents, APIs, and future clients are presenters or execution surfaces. They are not independent commercial authorities.

---

## 2. Vocabulary and separation of concerns

### 2.1 Product

A **Product** is a durable commercial and technical boundary such as `focusa` or `uiai_engine`. A new product requires one explicit registration decision under Section 15.

### 2.2 License Type

A **License Type** is a named model inside a product. It declares included capability families, product limits, local-runtime rights, and sale status. `Operator` is the initial License Type. `Navigator` is an example of a possible future type, not an approved current offer.

### 2.3 Term

A **Term** describes duration. `lifetime` means the entitlement does not expire through passage of time. It does not make authentication credentials, signed device leases, support obligations, hosted resources, or platform compatibility perpetual.

### 2.4 Capability family

A **Capability family** is a customer-understandable functional boundary. It is broader than a route, button, command, or implementation file. License Types include families, not hundreds of individual surfaces.

### 2.5 Operation

An **Operation** is the canonical execution identifier used by core, API, workers, and presenters. Entitlement is evaluated against the operation, its owning product, capability family, operation class, side-effect class, and initiating authority.

### 2.6 Surface

A **Surface** presents or invokes operations. A surface MUST NOT own pricing, grants, limits, or commercial policy. Installing another surface does not consume another commercial entitlement by itself.

### 2.7 Verified no-license access

`verified_no_license` is an account and runtime posture, not a License Type. It has no EDD Software Licensing key and no paid product grant. It uses an authority-signed limited-access assertion so clients cannot fabricate verified identity or allowed families.

---

## 3. Narrow supersession and contradiction resolution

This addendum supersedes only the following conflicting policy:

1. Spec 152 clauses that require every non-paid customer journey to be a time-bounded Evaluation.
2. Spec 152E clauses that require an expiring EDD-backed Evaluation license for all limited product access.
3. Spec 152F language stating that a verified Evaluation is the only non-paid route to value-producing Focusa capability.
4. Initial registry assumptions that `focusa_evaluation` is a required public product code.
5. Any public or internal claim that anonymous local `--eval` state grants product capability.
6. Any bundle price of USD $1,097.00 for the Focusa + UIAI Operator Lifetime combination.
7. Any claim that a Focusa-only purchase automatically grants UIAI Engine commercial entitlement.
8. Any claim that a lifetime purchase includes every future family product or future License Type.

All non-conflicting identity, authority, paid-checkout, key, lease, refund, revoke, node, sequence, recovery, privacy, and presenter-inheritance requirements remain in force.

---

## 4. Initial commercial model

### 4.1 Canonical posture and License Type codes

| Canonical code | Kind | Price | Duration | Grant |
|---|---|---:|---|---|
| `verified_no_license` | account/runtime posture, not a license | $0 | no automatic expiry | Explicit limited-access allowlist only |
| `focusa_operator_lifetime_v1` | Focusa License Type | $697.00 | lifetime | Included Focusa Operator v1 families and limits |
| `uiai_operator_lifetime_v1` | UIAI License Type | $697.00 | lifetime | Included UIAI Operator v1 families and limits |
| `focusa_uiai_operator_bundle_lifetime_v1` | Composite commerce SKU | $1,254.60 | lifetime | Both underlying Operator v1 License Types |

The Bundle calculation is normative:

```text
$697.00 + $697.00 = $1,394.00
$1,394.00 × 0.90 = $1,254.60
```

Prices are fixed for an issued order. If standalone prices change, the authority MUST publish a new price/version for future Bundle orders. Existing orders are never repriced.

### 4.2 Limited number of License Types

The authority MUST NOT create a License Type for every tool, surface, capability family, operating system, or client. A new License Type requires a materially different customer model that cannot be expressed safely through an existing type and its limits.

### 4.3 Operator today, future types later

`Operator` remains the approved initial License Type name. A future `Navigator` or other type:

- receives a new stable code;
- has an independently declared family set and limits;
- does not replace, mutate, or silently expand Operator;
- does not become available to Operator customers unless an explicit upgrade or cross-grade is purchased or granted.

---

## 5. Verified identity and pre-access firewall

### 5.1 Unverified state

Before mailbox verification, a person may access only:

- registration start;
- verification delivery, resend, poll, completion, expiry, and recovery;
- bounded public product documentation and security metadata;
- installer inspection and signature/checksum material;
- local uninstall and emergency local-data recovery required to avoid trapping software or customer data.

No product project, Workpoint, browser session, License Type, EDD customer, checkout, key, node, or runtime grant may be created from an unverified email.

### 5.2 Verified no-license state

After mailbox verification, the authority may create the canonical account and issue a signed `verified_no_license` access assertion. It MUST NOT create an EDD Software Licensing key or pretend that the assertion is a paid license.

### 5.3 Safety carve-out

Local uninstall, basic local export, repair, rollback, and emergency customer-data recovery are safety and ownership controls, not free product capability. They MUST remain available when identity infrastructure is unavailable, subject to local operating-system authorization and privacy controls.

---

## 6. Verified no-license limited mode

### 6.1 General rules

Verified no-license access is an explicit allowlist. New tools and new capability families MUST NOT enter it automatically. The runtime MUST show a stable upgrade explanation instead of deleting, hiding, or corrupting customer state.

### 6.2 Focusa limited mode

The initial Focusa limited mode permits:

- one mutable active project;
- explicit switching of which retained project is active;
- manual Mission, Focus State, Workpoint, Trajectory, and basic Evidence workflows inside the active project;
- read projection for all locally retained projects and evidence;
- basic customer-data export;
- account, device, license-status, diagnostics, repair, rollback, stable security update, and uninstall operations.

The initial Focusa limited mode blocks these families:

- automated Work Loop execution;
- Silent Sessions;
- scheduled or unattended execution;
- parallel agents or providers;
- multi-operator collaboration;
- commercial remote/team synchronization beyond the same verified operator's permitted node;
- governed release-proof bundles and advanced release intelligence;
- preview/nightly channels and unattended update rollout;
- any new family not explicitly added to the limited-mode allowlist.

A customer may retain unlimited project data. Projects beyond the selected active project remain readable and exportable. The system MUST NOT delete them or demand payment to retrieve them.

### 6.3 UIAI limited mode

The initial UIAI limited mode permits one foreground, ephemeral public-web observation session using bounded local or approved capacity for:

- provider-neutral public search;
- Source-to-Markdown;
- public page read;
- accessibility snapshot;
- screenshot;
- basic diagnostics.

It blocks:

- click, fill, type, select, press, and submitted browser mutations;
- cookie or authentication-state persistence;
- authenticated/private dashboard workflows;
- multiple concurrent sessions;
- unattended browser automation;
- scheduled or batch responsive QA;
- long-lived session persistence;
- premium proxies, hosted capacity, paid model calls, or other metered third-party resources.

Resource pressure, rate limits, and abuse controls remain independent of commercial entitlement. Limited mode never promises unbounded hosted compute.

---

## 7. Operator Lifetime v1 boundaries

### 7.1 Focusa Operator Lifetime v1

`focusa_operator_lifetime_v1` includes the Focusa product capability families approved at first sale, including normal Focusa core, automation, same-operator remote/device operation, release proof, and premium update workflows, subject to security, role, node, confirmation, and resource controls.

### 7.2 UIAI Operator Lifetime v1

`uiai_operator_lifetime_v1` includes the UIAI local/product capability families approved at first sale, including browser observation, browser action, local persistence, diagnostics, proof packets, batch/responsive workflows, and supported integrations.

It does not include unlimited hosted compute, paid proxies, third-party API consumption, paid model usage, managed hosting, resale, redistribution, or product embedding unless those rights are explicitly listed.

### 7.3 One operator and node baseline

The initial safe baseline is:

- one verified human operator seat per License Type;
- up to three registered operator nodes;
- CLI, TUI, Pi, menubar, Focusa Desktop, Cockpit, and other approved clients on the same node do not consume separate nodes;
- the Bundle uses the same three registered operator-node identities for both products rather than creating six unrelated activations.

A later team or enterprise License Type may change seat and node limits. Role authorization and multi-operator identity remain independent of entitlement.

### 7.4 Full features does not mean all future products

“Full features” for Operator means all capability families frozen into the applicable Operator v1 product boundary, plus implementation improvements that preserve those family semantics. It does not mean all future products, new License Types, new commercial families, or recurring hosted resources.

---

## 8. Minimal immutable License Type rule

### 8.1 Freeze point

Before the first approved sale, Operator v1 may be corrected. At the first approved sale, the authority MUST freeze:

- stable License Type code;
- product owner;
- included capability families;
- seat and node limits;
- local-runtime and hosted-resource rights;
- duration;
- refund posture;
- manifest digest.

No customer-specific feature manifest is required. Every license of the same version references the same immutable License Type record.

### 8.2 Existing-family inheritance

A new operation inherits an existing License Type when all are true:

1. it belongs to the same registered product;
2. it implements the same customer-understandable outcome as an included family;
3. its security, side-effect, privacy, and resource profile fit that family;
4. it does not introduce a separately named product or materially new hosted cost.

### 8.3 Materially new capability

A materially new customer outcome MUST receive a new capability family. New families default to:

```text
verified_no_license: denied
existing License Types: excluded pending explicit assignment
unknown/unclassified execution: denied
```

This rule prevents both accidental giveaways and a per-tool commercial bureaucracy.

---

## 9. Bundle composition

### 9.1 Grant union

The Bundle SKU MUST resolve to:

```text
focusa_operator_lifetime_v1
+
uiai_operator_lifetime_v1
```

The authority MUST derive Bundle features from those two records. It MUST NOT maintain a third hand-copied feature list.

### 9.2 One account and one human key

The Bundle uses one verified account, one EDD order, and one canonical human-facing EDD Software Licensing key. The signed lease or child-token system carries explicit Focusa and UIAI product grants.

### 9.3 Refund behavior

Initial Bundle refunds are whole-order refunds. A Bundle refund revokes both paid grants, increments authority sequence, preserves customer data and audit history, and returns the verified account to limited mode. Component-level partial refunds are not supported in v1.

### 9.4 Future products

The Bundle name and grants are limited to Focusa and UIAI Engine. A future product in the Focusa ecosystem does not enter this Bundle automatically.

---

## 10. Discontinuation, future types, and upgrades

### 10.1 Discontinuation

Discontinuing a License Type changes `sale_status` to `discontinued_no_new_sales`. It does not revoke, rename, downgrade, expire, or mutate already issued licenses.

### 10.2 Navigator and other future types

A future Navigator License Type is a separate model. Operator customers do not receive it automatically. The authority may later offer an explicit upgrade, cross-grade, or promotional grant.

### 10.3 Declining an upgrade

A customer who declines a future type retains the complete Operator v1 entitlement. Upgrade marketing or product evolution MUST NOT turn an existing lifetime license into limited mode.

### 10.4 Lifetime, support, and compatibility

Customer terms MUST distinguish:

- perpetual entitlement to the licensed product/family set;
- stable security, recovery, reinstall, and activation availability;
- support duration and response commitments;
- operating-system and hardware compatibility;
- hosted-service and third-party-resource availability.

Lifetime entitlement is not a promise of perpetual support for obsolete platforms or unlimited external services.

---

## 11. Surface inheritance

### 11.1 Universal rule

> Entitlement follows the canonical operation, not the button, route, binary, package, client, website, or operating system.

Every surface MUST project the same decision reason, required License Type, upgrade action, and recovery alternative.

### 11.2 Focusa presenters

CLI, TUI, Pi, menubar, Focusa Desktop, REST, installers, agents, generated UI, and future Focusa clients inherit Focusa operation policy. They do not maintain local commercial tables.

### 11.3 UIAI Cockpit and mixed surfaces

UIAI Cockpit is a UIAI-owned rich shell and may display or invoke both products. Each operation is evaluated independently:

- Focusa read or mutation uses Focusa policy;
- UIAI observation or action uses UIAI policy;
- a combined workflow requires both grants or the Bundle;
- rendering Focusa state in Cockpit does not grant Focusa mutation;
- pairing Cockpit or Desktop proves identity/device posture, not entitlement.

### 11.4 No direct-core bypass

Desktop, Tauri, native, and local-source clients MUST NOT bypass entitlement by calling storage or reducer code directly. Value-producing mutation requires the shared core execution guard even when HTTP middleware is absent.

### 11.5 Delayed execution

Workers, schedulers, queues, and resumable jobs inherit initiating authority and MUST revalidate at dispatch. A previously queued operation cannot continue after refund, revoke, higher sequence, or family denial.

---

## 12. Dynamic tools, plugins, and generated UI

Build-time scanning alone is insufficient for MCP tools, extensions, downloaded capsules, plugins, generated UI, and private modules.

Every production operation MUST resolve through trusted metadata containing at least:

```yaml
operation_id: stable.identifier
product_owner: focusa | uiai_engine | registered_future_product
operation_class: read | value_mutation | recovery | internal_maintenance
capability_family: registered_family
side_effect_class: none | local | remote | external
```

Dynamic operations require a trusted signed manifest. Unknown ownership, unknown mutation, unknown side effect, or unregistered family MUST fail closed before execution. A tool cannot self-label as recovery to bypass licensing.

Generated UI may render only canonical registered actions. Client-provided metadata cannot select products, prices, License Types, grants, or commercial treatment.

---

## 13. Independent security and authority gates

Commercial entitlement is one gate in this ordered path:

```text
project/scope binding
→ operation classification
→ authentication
→ authorization and role
→ confirmation and platform permission
→ recovery/read exception
→ signed access assertion or paid lease
→ License Type and capability family
→ resource reservation
→ execution
→ settlement and receipt
```

A paid license never grants operator identity, role, cognitive authority, Workpoint authority, project scope, browser origin permission, artifact trust, or consent. A verified email never grants paid capability.

---

## 14. Permanent entitlement and bounded credentials

A lifetime entitlement does not require a non-expiring device credential. Signed access assertions and device leases MAY have bounded refresh windows so that refund, chargeback, revoke, lost-device response, key rotation, and higher authority sequence propagate safely.

Offline Grace remains bounded and cannot expand products, capability families, seats, nodes, or limits. Credential expiry does not destroy the underlying lifetime entitlement; successful verified recovery can issue a replacement lease.

---

## 15. Future-product registration rule

A future product is unavailable until one operator-approved registration declares:

- stable product code and human name;
- authority and EDD mapping;
- owning repository/service;
- capability-family registry;
- available License Types;
- whether any existing Bundle includes it;
- seat, node, local, hosted, and metered-resource policy;
- recovery and customer-data behavior;
- refund/revoke/sequence behavior;
- presenter and mixed-surface behavior;
- migration, rollback, and acceptance evidence.

Default behavior is exclusion. Product namespace or marketing family resemblance MUST NOT create entitlement.

---

## 16. Paid authority and checkout

### 16.1 EDD authority

WPUIAI.com EDD remains the sole authority for paid customers, orders, human license keys, refunds, and paid License Type truth. EDD's configured Stripe gateway may process cards. Facades MUST NOT collect card details outside the approved EDD/Stripe UI or issue independent entitlement.

### 16.2 Verified no-license account

Mailbox verification may create the canonical authority account without creating a paid order or EDD Software Licensing key. The authority stores the verified account identity and signed limited-access posture separately from paid license truth.

### 16.3 Dedicated products

Focusa and UIAI offers require dedicated EDD Downloads/price records. Legacy WPUIAI products, credit packs, and Download `453` remain unrelated and quarantined unless a future explicit migration decision says otherwise.

---

## 17. Refund, revoke, downgrade, and data preservation

A paid refund, revoke, chargeback, or invalid higher authority sequence removes the paid grant. If mailbox verification remains valid, the account returns to `verified_no_license` limited mode.

When a customer has more than one mutable project:

1. all projects remain readable and exportable;
2. no project or evidence is deleted;
3. the last explicitly operator-selected project remains active when safe;
4. if no explicit selection is available, the runtime asks the operator to choose and performs no value mutation meanwhile;
5. activity heuristics MUST NOT silently select a project;
6. recovery, account control, basic export, repair, rollback, stable security update, and uninstall remain available.

---

## 18. Public contradiction removal

### 18.1 Public evidence basis

UIAI Engine browser reads captured the governing public claims on 2026-08-06 from:

- `https://focusa.dev/`, `/pricing/`, `/llms.txt`, and `/.well-known/agent-commerce.json`;
- `https://engine.focusa.dev/`, `/pricing/`, `/llms.txt`, and `/.well-known/agent-commerce.json`;
- `https://install.focusa.dev/`, `/focusa`, and `/bundle`;
- `https://wpuiai.com/wp-sitemap-posts-download-1.xml` and its public EDD Download pages.

Those reads proved the two $697 Operator offers, the conflicting $1,097 Bundle claim, anonymous/local Evaluation copy, direct Gravity Forms/Stripe positioning, broken convenience routes, broken license links, and unrelated legacy WPUIAI prices. Public content is evidence of current claims, not authority to override this addendum.

### 18.2 Required convergence

Before first sale or stable release, public and machine-readable surfaces MUST converge on this table:

| Current contradiction | Required correction |
|---|---|
| Anonymous/no-account Evaluation | Verified mailbox required for product capability |
| Local self-issued `--eval` | Verified signed limited-access assertion; no local grant creation |
| Timed Evaluation requirement | Permanent verified no-license limited mode |
| Bundle advertised at $1,097 | Bundle price $1,254.60 |
| Focusa purchase may implicitly grant UIAI | Only UIAI License Type, Bundle, or explicit authority grant provides UIAI paid capability |
| Gravity Forms/direct Stripe creates entitlement | Paid checkout and key truth converge on WPUIAI EDD with configured Stripe gateway |
| “No phone home” | “No telemetry; bounded authority communication for identity, activation, refresh, refund/revoke, and recovery” |
| `/focusa` and `/bundle` commands advertised while routes return 404 | Repair routes or remove claims before publication |
| Public `LICENSE` or `COMMERCIAL.md` links return 404 | Publish valid terms or remove broken links before publication |
| Old WPUIAI `$29/$99/$299/$149` offers appear commercially related | Keep legacy WPUIAI catalog explicitly separate from Focusa/UIAI License Types |

Public pricing and machine-readable commerce MUST use `Operator` as the current License Type. The name is not removed merely because future models may exist.

---

## 19. No-sales proof and cutover

The assertion that there have been no Focusa/UIAI offer sales MUST be proven across:

- EDD customers, orders, order items, transactions, licenses, activations, subscriptions, refunds, and chargebacks;
- Gravity Forms entries and Stripe feed records;
- Stripe Customers, Checkout Sessions, PaymentIntents, Charges, Refunds, and disputed payments;
- install-site custom license and webhook records;
- manually issued keys or access grants;
- synthetic/test records, which must be classified separately.

If any genuine sale is found, implementation MUST stop the no-migration path, preserve the record, and obtain an explicit customer-rights mapping decision. “No dedicated EDD Download exists” is not proof that no sale occurred through another rail.

Until the proof and registry are accepted, public checkout for these offers MUST remain disabled or explicitly non-purchasable. No customer may enter a commercial contract against contradictory terms.

---

## 20. Build and runtime gates

Release gates MUST prove:

1. Every production operation has trusted product, class, family, and side-effect metadata.
2. A new operation cannot become verified-no-license accessible without an explicit allowlist change.
3. A new capability family is excluded from all existing License Types until assigned.
4. A new product is excluded from existing License Types and Bundles.
5. Focusa-only entitlement cannot execute UIAI paid operations.
6. UIAI-only entitlement cannot execute Focusa paid operations.
7. Bundle resolves to the exact union of the two underlying License Types.
8. Menubar, Desktop, Cockpit, CLI, TUI, Pi, REST, workers, installers, and generated UI project equivalent decisions.
9. Direct local/core, delayed worker, stale-client, and dynamic-plugin bypasses fail closed.
10. Limited mode preserves read/export/recovery and never deletes customer data.
11. Refund/revoke transitions paid capability to limited mode and increments authority sequence.
12. Lifetime entitlement survives credential rotation while revoked credentials do not.
13. Stable security updates and uninstall remain reachable.
14. Public pricing, names, terms, and machine-readable commerce match canonical policy.
15. No-sales proof is complete before a no-migration cutover.

---

## 21. Stable errors

At minimum, implementations SHALL preserve stable meanings for:

```text
EMAIL_VERIFICATION_REQUIRED
VERIFIED_LIMITED_ACCESS
LICENSE_TYPE_REQUIRED
LICENSE_TYPE_NOT_INCLUDED
PRODUCT_NOT_INCLUDED
CAPABILITY_FAMILY_NOT_INCLUDED
ENTITLEMENT_POLICY_UNKNOWN
ENTITLEMENT_PRODUCT_MISMATCH
NODE_LIMIT_REACHED
OPERATOR_SEAT_LIMIT_REACHED
HOSTED_RESOURCE_NOT_INCLUDED
UPGRADE_AVAILABLE
RECOVERY_ONLY
```

Errors MUST include a safe recovery or upgrade action without exposing raw email, keys, tokens, customer identifiers, internal prices supplied by callers, or private policy data.

---

## 22. Required downstream reconciliation

This addendum does not silently rewrite existing files or close implementation work. The task graph MUST decompose and order at least these changes:

1. Reconcile Spec 152 Evaluation terminology and issuance state.
2. Reconcile Spec 152E identity, customer creation, product registry, checkout, key, and lease flows.
3. Reconcile Spec 152F policy grid, no-license base access, family defaults, and dormant-dimension rules.
4. Replace `focusa_evaluation` contracts and fixtures with `verified_no_license` posture and legacy input handling.
5. Define Operator v1 family sets, manifest digest, seats, nodes, local rights, and hosted exclusions.
6. Represent Bundle as two underlying grants and one order/key.
7. Repair product registry codes, prices, sale status, refund rules, and dedicated EDD mappings.
8. Implement operation-level inheritance across core/API/workers and all presenters.
9. Add Cockpit, Focusa Desktop, menubar, generated UI, dynamic tool, and mixed-product conformance.
10. Reconcile installers and public websites, including broken routes and machine-readable commerce.
11. Prove no sales across all rails or trigger a preserved customer-rights migration decision.
12. Rerun build-independent, exact-SHA installed, refund/revoke, offline, recovery, privacy, and release gates.

No task may be bulk-closed from this document alone. Existing technical evidence remains valid only where its tested semantics do not conflict with this addendum.

---

## 23. Acceptance criteria

This addendum is technically implemented only when:

- mailbox verification gates all product capability;
- verified no-license limited mode works without expiry or an EDD license key;
- Focusa Operator v1, UIAI Operator v1, and their Bundle are dedicated canonical records;
- Bundle price is exactly $1,254.60 and grants the exact two underlying License Types;
- Operator naming remains intact and future types cannot mutate it;
- first-sale freeze and future family/product defaults are enforced;
- node/seat semantics treat multiple clients on one node consistently;
- hosted-resource exclusions are explicit;
- every execution surface inherits canonical operation decisions;
- unknown and dynamic operations fail closed;
- lifetime entitlement and bounded credential lifetime remain distinct;
- refunds/revokes preserve data and return verified accounts to limited mode;
- no-sales proof is complete or a customer-preserving migration is approved;
- all public contradictions in Section 18 are removed;
- Specs 152/152E/152F, contracts, task graph, runtime, presenters, authority, websites, and evidence agree;
- REL.4–REL.7 pass at the final exact SHA.

---

## 24. Rollback and preservation

Rolling back software may restore an earlier binary or presenter, but MUST NOT roll back:

- verified identity;
- customer/order/refund truth;
- issued License Type and price version;
- immutable Operator rights after first sale;
- revocation or higher sequence;
- device and node history;
- customer data, projects, evidence, or exports;
- migration journals and acceptance evidence.

If policy reconciliation fails, new paid issuance remains disabled. Verified limited mode may continue only where its authority assertion and safety gates are proven. Existing paid entitlement, if any is later discovered, fails into customer-preserving recovery rather than deletion or silent downgrade.
