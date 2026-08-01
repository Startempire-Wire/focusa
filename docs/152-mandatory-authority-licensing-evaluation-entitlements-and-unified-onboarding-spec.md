# Spec 152 — Mandatory Authority-Issued Licensing, Evaluation Entitlements, and Unified Product Onboarding

**Status:** Proposed — release-blocking before the next customer/evaluator distribution cut  
**Created:** 2026-08-01  
**Scope:** Focusa, official Focusa installers and binaries, Focusa menubar onboarding, and the shared entitlement boundary with UIAI Engine  
**Authority boundary:** Public-safe product contract. Registry internals, commercial calculations, anti-abuse rules, customer records, and signing operations remain private.

## 1. Decision

Every official Focusa runtime must require an authority-issued entitlement, including evaluation builds.

Evaluation is a real, revocable, time-bounded license tier. It is not the absence of a license, a locally self-issued record, a command-line bypass, or an implied permission caused by loopback execution.

The canonical rule is:

```text
no verified authority lease
→ no mutable Focusa work or agent execution
→ license recovery, health, safe data export, and uninstall only
```

This applies to official release binaries, installer-managed builds, desktop packages, Pi integration packages, and unmodified source builds. Source visibility under BSL 1.1 does not itself grant commercial rights, eliminate product entitlement requirements, or make Focusa open source.

A determined party can modify visible client code. Runtime licensing therefore provides practical product control and a clear contractual boundary; it is not represented as impossible-to-bypass DRM. High-value authority, signing, anti-abuse, and private commercial components must remain server-side or in private repositories.

## 2. Why the current loop is open

The current repository contains useful licensing pieces, but they do not form one fail-closed authority chain.

| Current surface | Current behavior | Required correction |
| --- | --- | --- |
| `crates/focusa-license/src/lib.rs` | Missing state falls back to a self-issued seven-day eval; local JSON/TOML is trusted; an environment key can become `Licensed` without a registry response | Require a signed authority lease; remove self-issued eval, local tier override, and unverified environment promotion |
| `crates/focusa-core/src/license.rs` | A second license model treats a missing file as unlimited Evaluation and trusts local `commercial_use`, tier, features, and status fields | Collapse to one entitlement implementation and verify every persisted claim cryptographically |
| `crates/focusa-cli/src/commands/license.rs` | Online activation exists, but response and default-registry behavior diverge from other paths; runtime features do not consistently use it | Make activation/refresh the only source of runtime entitlement and publish one versioned authority contract |
| `scripts/install-focusa.sh` | `--eval` writes a local self-signed eval record without verified email or authority issuance | Replace with an online authority-backed evaluation/device-code flow before product activation |
| `/v1/license/status` | Reports one guard model but does not own activation, refresh, or universal route enforcement | Return the canonical entitlement state and make all mutation/execution surfaces depend on it |
| `docs/current/FIRST_RUN_FLOW.md` | Begins with device pairing and does not establish account/license authority | Add entitlement onboarding before daemon pairing and normal Mission Canvas use |
| Feature gates | Only selected release/export/device paths call the existing feature helper | Require a central gate for every mutation, execution, packaged artifact, remote, team, and premium surface |

Two divergent license implementations are especially dangerous. A release must not be able to report one license posture from the daemon while another crate grants features from a different local record.

## 3. Canonical entitlement states

All Focusa surfaces use the same state machine:

| State | Meaning | Runtime posture |
| --- | --- | --- |
| `unactivated` | No authority lease has ever been installed | Recovery-only |
| `active_evaluation` | Valid authority-issued evaluation lease | Explicit evaluation allowlist and limits |
| `active_paid` | Valid commercial, team, enterprise, cohort, or developer lease | Explicit signed entitlements |
| `offline_grace` | A previously verified lease is temporarily beyond refresh time but remains inside its signed offline window | Signed features continue; warning and refresh required |
| `expired` | Absolute license/evaluation expiry reached | Recovery-only; data preserved |
| `revoked` | Authority has revoked/refunded/disabled the entitlement and the revocation is known locally | Recovery-only; data preserved |
| `invalid` | Signature, product, schema, node, sequence, or time validation failed | Recovery-only and security diagnostic |

No state is inferred from an empty feature list, a tier string, a local address, source-build status, or the existence of a plaintext file.

## 4. Authority-issued lease contract

The persisted client artifact is a signed entitlement lease, not a writable statement of permission.

The public lease contract includes at least:

```json
{
  "schema": "focusa.entitlement_lease.v1",
  "lease_id": "opaque",
  "license_id": "opaque",
  "account_id": "opaque",
  "license_class": "evaluation|operator|team|enterprise|developer",
  "status": "active",
  "allowed_products": ["focusa", "uiai-engine"],
  "features": ["focusa.core.mission", "focusa.core.workpoint"],
  "limits": {},
  "node_id": "opaque",
  "issued_at": "RFC3339 timestamp",
  "not_before": "RFC3339 timestamp",
  "refresh_after": "RFC3339 timestamp",
  "offline_valid_until": "RFC3339 timestamp",
  "expires_at": "RFC3339 timestamp or null",
  "sequence": 1,
  "authority_key_id": "public-key identifier",
  "signature": "detached or enveloped signature"
}
```

Rules:

1. The authority signs the canonical payload. The private signing key never ships with Focusa.
2. Focusa ships a pinned trust root and supports signed key rotation and revocation metadata.
3. The client verifies schema, signature, product grant, status, time bounds, node binding, sequence, feature, and limit before granting a capability.
4. Local files may cache a lease and non-authoritative usage receipts, but editing local JSON cannot create an entitlement.
5. Raw license keys, bearer tokens, one-time activation codes, and unmasked evaluator email addresses never appear in logs, command output, Evidence, Workpoints, or animated installer events.
6. A license key is an activation credential, not the runtime source of truth. Runtime uses the signed lease.
7. Developer mode is authority-issued and release-channel constrained. It is never selected by a local environment variable alone.

## 5. Mandatory evaluation acquisition

The old unauthenticated `--eval` path is removed from production installers.

The replacement is an authority-backed evaluation flow:

```text
verified bootstrap
→ choose Evaluate / Activate / Purchase
→ receive device code and browser URL
→ verify email/account
→ accept current license and privacy terms
→ choose promotional-email consent separately
→ authority creates or resolves evaluation license
→ register this node
→ issue signed product lease
→ installer verifies lease
→ install and start entitled products
```

### 5.1 Email and consent

- A verified email/account is required for evaluation issuance, recovery, security notices, and evaluator lifecycle communication.
- Transactional/account communication and promotional communication are separate purposes.
- Promotional consent must be explicit, versioned, auditable, and revocable; evaluation access must not silently imply marketing consent.
- The authority retains the canonical email. Local product state should display a masked identity or account handle unless the operator deliberately requests more detail.

### 5.2 Headless and agent-first flow

The default headless path uses a device-code exchange rather than placing a raw license key in shell history:

```bash
focusa license start --product bundle --json
focusa license poll --device-code <redacted-handle> --json
focusa license status --json
```

An existing-key path remains available for recovery and automation, but it exchanges the key for a signed lease and does not persist the raw key by default.

A non-mutating installer dry run may remain available without activation. It may inspect platform, dependencies, target paths, release trust, and the expected entitlement step, but it must not install or activate runnable product components.

## 6. Evaluation product posture

Evaluation must demonstrate the complete Focusa value loop without becoming an indefinite free edition.

The public contract is:

- authority-issued and time-bounded;
- one named evaluator/account unless the lease says otherwise;
- node and concurrency bounded;
- explicit feature allowlist rather than “everything except a few premium commands”;
- versioned usage/limit policy supplied by the authority;
- clear remaining-time/remaining-capability status in CLI, menubar, TUI, and agent responses;
- locked features remain discoverable with a concise explanation and purchase/activation action;
- expiry preserves all user data and permits safe export, license activation, diagnostics, backup, and uninstall;
- no hosted, redistribution, product embedding, team, unattended fleet, or production-commercial entitlement unless explicitly signed.

The private authority policy owns exact commercial caps and anti-abuse rules. Public clients consume `features` and `limits`; they do not hard-code secret business policy.

The minimum evaluation journey should prove:

1. create or open a bounded Focusa project;
2. establish Trajectory and a Workpoint;
3. execute a small governed task;
4. use an entitled UIAI Engine browser action when the bundle grant includes it;
5. link Evidence;
6. resume from the canonical Workpoint;
7. see what the paid entitlement unlocks next.

## 7. One entitlement service inside Focusa

`focusa-license` becomes the only entitlement library. The separate permissive model in `focusa-core::license` is removed or reduced to a compatibility adapter that delegates every decision to the canonical library.

Required API:

```rust
entitlements.status()
entitlements.require_product("focusa")
entitlements.require_feature("focusa.agent.silent_sessions")
entitlements.consume("focusa.agent.silent_session_start", 1)
entitlements.refresh()
entitlements.recovery_posture()
```

Every check returns a structured decision containing:

- allowed/denied;
- canonical state;
- feature and product;
- current limit/remaining value when applicable;
- lease expiry/offline window;
- stable error code;
- safe recovery action;
- redacted purchase/manage URL.

### 7.1 Fail-closed corrections

Implementation must remove these behaviors:

- missing file becomes Evaluation;
- self-issued eval lease;
- plaintext tier or feature overrides in `~/.focusa/license.toml`;
- trusting local `commercial_use`, `status`, `features`, or `expires_at` without a valid signature;
- unknown tier defaults to Operator;
- any non-empty environment key becomes Licensed;
- non-cryptographic key fingerprinting labeled as SHA-256;
- licensed tier implicitly permits telemetry or every future capability;
- feature checks that ignore revoked/refunded/expired/offline-window/product state.

## 8. Runtime recovery mode

The daemon may start without a valid lease, but only in recovery mode. This avoids trapping customer data behind a network or billing failure while still closing the execution loop.

Recovery mode permits:

- health/version;
- license status, start, activate, refresh, and doctor;
- read-only inventory needed to locate user data;
- safe backup/export of operator-owned data;
- support bundle generation with redaction;
- uninstall and data-preserving repair.

Recovery mode denies:

- project mutation;
- agent turns and Silent Sessions;
- Workpoint mutation and Evidence ingestion;
- packaged exports, release proof, remote streams, and team operations;
- UIAI Engine execution tokens;
- updater apply when the current lease does not permit it.

No denial deletes or encrypts existing user data.

## 9. Unified Focusa + UIAI Engine onboarding

Focusa owns the shared product onboarding experience because it is mission control and already owns installation, lifecycle, and the menubar first-run path. UIAI Engine remains an independently enforceable service, not an implicit free dependency.

### 9.1 Shared steps

1. Verify bootstrap/release trust.
2. Establish authority identity and signed product lease.
3. Install the entitled Focusa components.
4. Offer UIAI Engine when the lease includes it or when the evaluator selected the bundle journey.
5. Register UIAI Engine as a product instance under the same account/node authority.
6. Focusa mints or requests a short-lived, scoped child token for UIAI Engine; it never forwards the raw commercial license key on every browser call.
7. Verify Focusa daemon health and UIAI Engine license/health posture.
8. Complete the current Mac/phone device-pairing flow.
9. Run the bounded first mission and proof walkthrough.
10. Show evaluation status, locked capabilities, manage-license path, and data-preservation policy.

### 9.2 Separation of authority

- The license authority decides product and feature entitlement.
- Focusa locally brokers scoped execution credentials and records receipts.
- UIAI Engine verifies its own product grant and token scope on every execution path.
- Pairing tokens authenticate devices; they do not create product entitlement.
- Local API tokens authenticate callers; they do not imply a paid tier.
- Loopback origin affects network authentication risk only; it never grants an evaluation or commercial license.

## 10. Installer changes

The canonical installer becomes entitlement-first:

1. Parse uninstall and non-mutating dry-run modes.
2. Verify platform and release trust metadata.
3. Resolve an existing valid signed lease or start activation/evaluation.
4. Verify product and installer entitlements before downloading runnable product assets.
5. Install atomically.
6. Start services in entitled or recovery posture.
7. Verify the canonical license status from the running daemon.
8. Continue into unified onboarding.

Production behavior must not include:

- `--eval` as an offline/no-account bypass;
- a locally generated evaluation license;
- a default runtime entitlement created before authority contact;
- a test fixture status silently converted into a customer evaluation;
- raw keys stored for convenience;
- installer success when the daemon reports a different entitlement model.

Test-only fixtures must be compile-time or test-harness constrained and impossible to enable in a release artifact.

## 11. Feature namespace and coverage

Entitlements use stable product-qualified keys. Representative groups:

- `focusa.core.mission`
- `focusa.core.workpoint`
- `focusa.core.evidence`
- `focusa.agent.silent_sessions`
- `focusa.agent.parallelism`
- `focusa.export.packaged`
- `focusa.release.proof`
- `focusa.remote.stream`
- `focusa.team.multi_operator`
- `focusa.update.apply`
- `focusa.update.unattended`
- `uiai.session.execute`
- `uiai.screenshot.capture`
- `uiai.analysis.critique`
- `uiai.analysis.reverse`
- `uiai.media.produce`
- `uiai.remote.api`

A generated feature ledger must map each key to every CLI command, REST route, Pi tool, TUI action, menubar action, background worker, and service-to-service token scope. Release proof fails when a mutable or execution-capable surface is unmapped.

## 12. Public/private source boundary

Git ignore is not a security boundary and must not become the only home of commercially critical code.

Keep public:

- BSL license text and public product terms;
- entitlement schemas and stable error contracts;
- public verification keys/trust roots;
- client-side signature verification;
- recovery-mode behavior;
- integration interfaces and public-safe tests.

Keep private or server-side:

- signing keys and issuance logic;
- customer/license records;
- anti-abuse scoring and evaluator eligibility policy;
- private commercial caps and experiments;
- payment/refund synchronization;
- CRM/email automation details;
- premium proprietary assets or algorithms selected for private distribution;
- operational admin routes and raw audit evidence.

When proprietary product code must be removed from a public tree, move it into a versioned private repository/package or a server-side service with a stable public interface. Do not merely place it in an untracked folder that lacks durable review, backup, access control, and release provenance.

## 13. Migration

1. Freeze new public releases that create self-issued eval state.
2. Define and stage the signed lease contract at the authority.
3. Add canonical lease verification and recovery mode.
4. Exchange existing legitimate paid keys for signed leases.
5. Give existing unlicensed evaluation installs a short, clearly displayed migration window to verify an email and obtain an authority-issued evaluation; do not grandfather indefinite free runtime.
6. Remove the second license decision engine and all local tier overrides.
7. Convert installer `--eval` guidance to authority-backed evaluation guidance.
8. Integrate entitlement into menubar first run before device pairing.
9. Enforce UIAI Engine independently.
10. Run release proof against missing, forged, expired, offline, revoked, wrong-product, and bundled-license cases.

## 14. Required tests and release gates

At minimum:

- no lease starts recovery-only;
- evaluation cannot be issued without authority identity verification;
- edited local JSON cannot unlock a feature;
- a valid signature for the wrong product is denied;
- expired and revoked leases deny mutation while preserving export/uninstall;
- offline grace obeys signed bounds and cannot be extended by editing the clock-state file;
- stale lease sequence cannot replace a newer revocation/lease;
- unknown tiers and unknown features fail closed;
- commercial and evaluation keys are redacted everywhere;
- one bundle activation provisions Focusa and UIAI Engine without a second account flow;
- pairing/local tokens do not create entitlement;
- every mutable CLI/API/Pi/TUI/menubar route appears in the generated entitlement coverage ledger;
- release artifacts cannot enable test/dev bypasses;
- installer and daemon report the same lease id, product grants, status, and feature digest;
- uninstall and data export remain available after expiry.

## 15. Documentation supersession

When implementation lands, this spec supersedes the permissive evaluation portions of:

- `docs/118-focusa-license-guard-spec.md`;
- `docs/118-focusa-license-tiers-spec.md` where Evaluation is described as missing/default state;
- `docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md` where evaluation/community runtime lacks authority issuance;
- `docs/current/FIRST_RUN_FLOW.md` where pairing precedes entitlement;
- `docs/current/INSTALLER_UPDATE_POLICY.md` examples using unauthenticated `--eval`;
- `docs/current/COMMERCIAL_PACKAGING.md` descriptions that imply an ungated runnable community edition.

Those documents must be updated in the implementation PR rather than left as contradictory active guidance.

## 16. Definition of closed loop

The license loop is closed only when all of the following are true:

```text
lead/evaluator identity captured
+ terms and consent recorded
+ authority creates license
+ signed lease issued
+ product/node grant verified
+ every execution surface gated
+ limits and expiry enforced
+ refresh/revoke propagate
+ locked-feature upsell is visible
+ user data remains recoverable
+ authority and product receipts reconcile
```

A license page, BSL notice, local JSON file, or a few gated commands alone does not satisfy this definition.
