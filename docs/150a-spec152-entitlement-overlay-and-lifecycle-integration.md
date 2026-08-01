# Spec 150A — Spec 152 Entitlement Overlay and Lifecycle Integration

**Status:** Normative amendment; release-blocking for evaluator/customer distribution  
**Created:** 2026-08-01  
**Amends:** Spec 150, Specs 112/118/128/132 where they describe licensing or evaluation, and current install/onboarding guidance  
**Depends on:** Spec 152 and Spec 152A

## 1. Purpose

Spec 150's verified lifecycle implementation proves important mechanics: typed host/project/maintenance transactions, explicit confirmation, preservation declarations, hash-chained lifecycle journals, coherent version sets, adapter outcomes, rollback, and first-Workpoint acceptance.

That proof does **not** establish licensed customer/evaluator distribution. The current lifecycle request and adapter contracts do not bind a signed entitlement lease, product grant, lease sequence, feature digest, evaluation limits, node activation, or recovery-only posture.

This amendment makes Spec 152 a mandatory authority overlay on every Spec 150 lifecycle operation.

```text
Spec 150 lifecycle mechanics verified
+ Spec 152 entitlement authority verified
+ Spec 152A protected-distribution boundary verified where required
= distribution acceptance eligible
```

No ledger or receipt may claim customer/evaluator lifecycle completion from Spec 150 alone.

## 2. Precedence

For licensing, evaluation, entitlement, node activation, protected modules, and recovery posture:

1. Spec 152 is the canonical product entitlement authority.
2. Spec 152A is the canonical protected-distribution and anti-tamper authority.
3. This amendment integrates those authorities into Spec 150.
4. Spec 150 continues to own lifecycle transaction mechanics and preservation.
5. Specs 112, 118, 128, and 132 remain historical/current implementation references only where they do not conflict.

Conflicts fail closed. A lower-numbered document, current implementation, generated ledger, guide, installer example, or test fixture cannot restore self-issued Evaluation or bypass authority issuance.

## 3. Current verified boundary

The following are considered implemented lifecycle foundations, not final license closure:

- `crates/focusa-core/src/install_lifecycle.rs` and submodules;
- typed `HostInstallTransaction`, `ProjectOnboardingTransaction`, and `LifecycleMaintenanceTransaction`;
- lifecycle journal hashing and idempotent resume;
- artifact-signature and preservation checks;
- complete version-set acceptance;
- Pi/UIAI/provider/menubar adapter capability receipts;
- CLI guided lifecycle preview/confirmation;
- Spec 150 implementation ledgers and current proof receipts.

Until this amendment is implemented, all public claims must say:

```text
lifecycle mechanics verified; mandatory authority-issued licensing integration open
```

## 4. Required lifecycle authority binding

Every mutating lifecycle request must carry an immutable entitlement binding resolved by the canonical entitlement service.

Recommended public contract:

```rust
pub struct LifecycleEntitlementBinding {
    pub state: EntitlementState,
    pub lease_id: String,
    pub lease_sequence: u64,
    pub lease_payload_digest: String,
    pub product_grants_digest: String,
    pub feature_grants_digest: String,
    pub node_id: String,
    pub license_class: String,
    pub refresh_after: DateTime<Utc>,
    pub offline_valid_until: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authority_key_id: String,
    pub signature_verified: bool,
}
```

`LifecycleOperationRequest` must add either this binding or a stable reference to an immutable verified entitlement snapshot. A boolean such as `licensed`, `eval`, `commercial`, or `license_ok` is insufficient.

### 4.1 Required operation features

| Operation | Required entitlement behavior |
| --- | --- |
| inspect | Allowed without active lease only for recovery-safe inventory; must not expose protected/customer content |
| install | Requires authority-issued lease granting the selected product and installer channel, except non-mutating dry run |
| repair/rerun | Recovery-safe repair may restore licensing components; product execution repair requires the applicable grant |
| update | Requires `focusa.update.apply`; unattended apply additionally requires `focusa.update.unattended` |
| rollback | May restore the last trusted licensed/recovery version; cannot restore stale entitlement authority |
| uninstall | Always available and preserves user data by default |
| purge | Always requires separate destructive confirmation; licensing never authorizes deletion by itself |
| project onboarding | Requires a valid Focusa product grant and each invoked feature grant before mutation |
| UIAI integration | Requires a `uiai-engine` product grant and scoped child-token issuance; health discovery alone grants nothing |

## 5. State-machine integration

Spec 150 lifecycle states remain, with entitlement preconditions and recovery states added.

Required entitlement states:

```text
entitlement_unactivated
entitlement_pending_identity
entitlement_pending_device_code
entitlement_active_evaluation
entitlement_active_paid
entitlement_offline_grace
entitlement_expired
entitlement_revoked
entitlement_invalid
```

Required lifecycle ordering:

```text
uninspected
→ preflighted
→ release_trust_verified
→ entitlement_resolved
→ product_grant_verified
→ artifact_selected
→ artifact_verified
→ host_installed
→ daemon_recovery_or_entitled_ready
→ integrations_selected
→ integration_product_grants_verified
→ project_selected
→ project_verified
→ project_feature_grants_verified
→ bootstrap/genesis/first Workpoint
→ accepted
```

`accepted` is forbidden when:

- lease signature is absent or invalid;
- the product grant is absent;
- lease state is unactivated, expired, revoked, invalid, or outside signed offline grace;
- lease/node/sequence/digest does not match the receipt;
- required feature or limit reservation is absent;
- UIAI is marked active without an independently verified UIAI grant;
- a test fixture or development trust root is present in a customer artifact.

Recovery-only startup is a truthful successful service state but is not product acceptance.

## 6. Adapter integration

`LifecycleAdapterReceipt` currently reports capability readiness. It must separately report entitlement readiness.

Recommended extension:

```rust
pub struct AdapterEntitlementPosture {
    pub product: String,
    pub lease_id: String,
    pub lease_sequence: u64,
    pub product_granted: bool,
    pub required_features_granted: bool,
    pub child_token_audience: Option<String>,
    pub child_token_expires_at: Option<DateTime<Utc>>,
    pub entitlement_digest: String,
}
```

Rules:

- `PresentCompatible` or `Healthy` never implies licensed.
- `UIAI_LOCAL_API_TOKEN`, pairing token, extension token, provider token, or loopback origin never supplies a product grant.
- An optional UIAI adapter can be absent or opted out without failing Focusa core installation.
- If UIAI is selected, `Active` requires health **and** independently verified entitlement posture.
- Provider-auth handoff remains credential-neutral and separate from Focusa licensing.
- Menubar pairing authenticates a device; it cannot create or widen entitlement.

## 7. Acceptance receipt changes

`LifecycleAcceptanceReceipt` must include:

```text
entitlement_state
lease_id
lease_sequence
lease_payload_digest
product_grants_digest
feature_grants_digest
node_id
license_class
signature_verified
offline_valid_until
entitlement_evidence_refs
protected_component_set_digest (when applicable)
```

The receipt must distinguish:

- `recovery_ready`;
- `evaluation_ready`;
- `paid_ready`;
- `development_ready`;
- `blocked_entitlement`.

A health/version/first-Workpoint result cannot override `blocked_entitlement`.

## 8. Installer integration

The current Bash and PowerShell `--eval` behavior is a legacy pre-Spec-152 implementation and is release-blocked for new evaluator distribution.

Approved target flow:

```text
non-mutating preflight
→ release trust verification
→ resolve existing signed lease or start device-code flow
→ verified account/email and terms
→ authority-issued Evaluation or paid grant
→ node registration
→ signed lease verification
→ runnable asset/protected capsule acquisition
→ atomic install
→ recovery/entitled daemon status proof
→ optional UIAI grant and child token
→ pairing
→ first project/Workpoint
```

Until implemented:

- dry run may inspect without activation;
- uninstall and data-preserving recovery remain available;
- no guide may recommend legacy `--eval` as the approved evaluator install path;
- customer/evaluator distribution remains blocked rather than silently using the old bypass.

## 9. Source-build boundary

A source checkout may remain readable and buildable under the repository's BSL terms, but:

- source visibility is not an authority-issued Evaluation;
- an unmodified official runtime must still require a signed lease for mutable/execution capability;
- a public-source development shell may expose recovery, contracts, tests, and non-mutating development surfaces;
- protected private workers/capsules are absent unless a valid entitlement authorizes their delivery;
- a locally modified fork is unsupported and cannot receive authority-trusted protected components merely by reporting success.

## 10. Exact implementation map

### Focusa core and daemon

- Replace divergent decisions in `crates/focusa-license/src/lib.rs` and `crates/focusa-core/src/license.rs` with one canonical entitlement verifier.
- Add entitlement snapshot/binding to `crates/focusa-core/src/install_lifecycle/` models, transactions, orchestrator, adapters, and tests.
- Add recovery-mode route classification and central gate middleware in `crates/focusa-api`.
- Make `/v1/license/status` report the canonical lease state/digests and no locally inferred tier.
- Bind UIAI child-token issuance to a verified bundle/UIAI grant.

### CLI and installers

- Add device-code start/poll/activation/refresh commands to `crates/focusa-cli/src/commands/license.rs`.
- Make `crates/focusa-cli/src/commands/install.rs` consume the canonical entitlement binding.
- Replace production self-issued evaluation in `scripts/install-focusa.sh` and `scripts/install-focusa.ps1`.
- Preserve non-mutating dry run, recovery repair, uninstall, and explicit purge.
- Redact all key, code, token, lease-account, and email values from presenters and receipts.

### Menubar, Pi, and TUI

- Insert entitlement onboarding before pairing in `FirstRunWizard.svelte` or its successor state machine.
- Project the canonical status/remaining evaluation posture consistently in menubar, TUI, CLI JSON, REST, and Pi tools.
- Add `license_feature`, product, and optional limit metadata to generated capability descriptors.
- Deny invocation before side effects while keeping locked capabilities discoverable.

### UIAI Engine

- Follow the UIAI mandatory-entitlement specification and protected-worker addendum.
- Separate auth identity from entitlement.
- Remove loopback/local-token/tier-derived product permission.
- Verify a UIAI product grant and scoped child token independently.

## 11. Migration map

| Legacy source | Migration |
| --- | --- |
| `~/.config/focusa/license.json` plaintext tier/features | Preserve for audit, exchange real key/account authority for signed lease, then archive as non-authoritative migration input |
| `~/.focusa/license.toml` tier override | Stop reading as authority; preserve only as diagnostic evidence if present |
| self-issued Evaluation | Require verified account/device-code migration within a bounded displayed window |
| existing paid key | Exchange without repurchase for a signed lease reflecting the actual purchased product/terms |
| `dev_mode` test fixture | Rename/restrict as test-only; real developers receive authority-issued developer licenses |
| UIAI local/extension tokens | Reissue as authentication-only or scoped child tokens under a verified parent grant |
| existing paired devices | Preserve pairing identity but require entitlement binding before execution |

Migration never deletes Workpoints, Evidence, project state, browser artifacts, or operator-authored data.

## 12. Cross-spec release gate

A release is ineligible for evaluator/customer distribution unless all are true:

1. Spec 150 lifecycle runtime gate passes.
2. Spec 152 entitlement coverage gate passes.
3. Spec 152 documentation consistency gate passes.
4. Authority staging E2E passes for Evaluation, paid, bundle, wrong-product, expired, revoked, refunded, offline, and node-limit cases.
5. UIAI endpoint-feature coverage passes.
6. Protected-component/capsule proof passes for features selected under Spec 152A.
7. Installer, daemon, CLI, menubar, Pi, TUI, UIAI, and authority report the same lease id/sequence and compatible digests.
8. Release artifacts contain no production-enabled fixture/dev bypass.
9. Uninstall/export/recovery remain available after denial or expiry.

Existing Spec 150 `implementation_verified` status remains valid only for its lifecycle-mechanics scope. It does not satisfy this combined release gate.

## 13. Documentation rule

Active guides must not present any of the following as approved current evaluator onboarding:

- `--eval` without authority issuance;
- missing license as Evaluation;
- loopback as Evaluation entitlement;
- Community/source checkout as an automatically entitled runnable edition;
- device pairing before entitlement resolution;
- healthy UIAI adapter as proof of UIAI license.

Historical specs/evidence may retain old behavior only with a supersession marker or through the machine-readable Spec 152 matrix.

## 14. Definition of completion

Spec 150A is complete when a new evaluator can perform the full authority-issued flow, complete the bounded first mission, hit signed limits, expire into recovery-only mode, activate a paid grant without reinstalling, and produce one reconciled lifecycle receipt whose entitlement fields match the authority, Focusa, and optional UIAI Engine.
