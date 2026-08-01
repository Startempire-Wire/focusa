# Spec 152A — Protected Distribution, Private Feature Capsules, and Anti-Tamper Cost Escalation

**Status:** Proposed — security addendum to Spec 152  
**Created:** 2026-08-01  
**Scope:** Focusa, UIAI Engine bundle integration, official installers, protected modules, local workers, update delivery, and development-agent access boundaries  
**Public boundary:** This document defines architecture and guarantees. Exact check placement, obfuscation configuration, private module layout, key-delivery policy, anti-abuse signals, and operational signing details remain private.

## 1. Security objective

Focusa cannot make software running with customer administrator privileges mathematically impossible to patch. Client code must eventually execute in plaintext machine form, and a sufficiently capable operator can modify binaries, intercept calls, or replace processes.

The correct objective is therefore:

```text
patching one visible license condition
must not create a usable paid capability
```

A bypass should require an attacker to reconstruct missing proprietary functionality, defeat independent cryptographic checks, replace multiple processes, lose official updates and authority services, and continue maintaining a private fork.

This is cost escalation and capability withholding, not a claim of perfect DRM.

## 2. Core rule: absence is stronger than encryption

The strongest protection is not encrypting a license Boolean. It is ensuring that commercially valuable implementation is not present in the public source tree or base recovery binary.

The official architecture separates:

| Component | Visibility | Responsibility |
| --- | --- | --- |
| Public shell/core contracts | Public BSL repository | CLI/API schemas, recovery mode, signed-lease verification, public-safe orchestration, local data ownership, extension interfaces |
| Entitlement broker | Official signed Focusa distribution | Verify signed lease, maintain node identity, request scoped child tokens, enforce recovery posture |
| Protected Focusa worker/modules | Private repository and signed binary distribution | Premium execution paths, proprietary policies/algorithms selected for protection |
| Protected UIAI Engine worker/modules | Private repository and signed binary distribution | Entitled browser/analysis/media execution selected for protection |
| License authority | Server-side private implementation | Account, issuance, signing, product grants, limits, refresh, revocation, encrypted module-key envelopes |
| Optional hosted crown-jewel services | Server-side | Capabilities that should never be reproducible from a local binary alone |

A public source build may compile a recovery/development shell, but it must not contain the protected implementation or a production bypass that synthesizes it.

## 3. Protected feature capsule

A protected feature capsule is a versioned distribution unit containing one or more private workers, modules, models, prompts, policies, or assets.

Each capsule has:

- immutable capsule id and version;
- product and feature namespace;
- platform/architecture compatibility;
- plaintext digest recorded at build time;
- ciphertext digest for delivery verification;
- signed manifest and provenance;
- minimum compatible public-shell contract;
- required lease features and limit policy version;
- release/channel status and revocation state;
- optional customer/node watermark identifier;
- encrypted payload or separately encrypted sensitive assets.

The capsule-signing key and payload-encryption keys never ship in source control.

## 4. Device-bound key flow

1. The official installer or entitlement broker creates a device key pair.
2. Where supported, the private key is hardware-backed and non-exportable. A secure OS credential store is the fallback.
3. Only the public key and attestation/registration evidence are sent to the authority.
4. The authority verifies the license, product grant, node allowance, release, and capsule entitlement.
5. The authority returns:
   - the signed entitlement lease;
   - a signed capsule manifest;
   - a content-key envelope encrypted for that node/device key;
   - a bounded refresh/offline policy.
6. The broker verifies all signatures and unwraps the content key through the platform security provider.
7. The capsule is decrypted only for the minimum time and scope required to load or execute it.
8. The protected worker independently verifies lease/product/feature/time/node/token state before performing work.

There is no global symmetric key embedded in Focusa, UIAI Engine, the installer, or the repository.

## 5. Independent enforcement layers

No single `licensed: bool` or `require_feature()` call owns all permission.

The same operation may be checked at these independent boundaries:

1. authority issuance eligibility;
2. capsule download authorization;
3. content-key envelope issuance;
4. installer/product activation;
5. capsule manifest and signature verification;
6. device-key unwrap;
7. protected worker startup;
8. parent lease verification;
9. short-lived operation token verification;
10. feature and limit reservation;
11. route/command/Pi/TUI/menubar gate;
12. usage receipt commit/reconciliation;
13. update and capsule refresh eligibility;
14. hosted-service authorization where applicable.

Checks must use shared canonical claims, but should be implemented in independently signed components so replacing the public shell alone is insufficient.

## 6. Focusa process split

Recommended runtime topology:

```text
focusa CLI / menubar / Pi
        |
        v
public Focusa daemon + entitlement broker
        |
        | signed short-lived operation capability
        v
protected Focusa worker
        |
        +----> protected UIAI Engine worker
        |
        +----> optional authority/hosted capabilities
```

Rules:

- The public daemon can expose health, license recovery, data export, and public contracts without the protected worker.
- Premium commands fail because the implementation endpoint is absent, not merely because a public Boolean is false.
- The protected worker accepts only narrow operation tokens with audience, feature, lease id, node id, request id, expiry, and idempotency binding.
- Worker IPC is local, authenticated, versioned, and not a general shell or arbitrary command channel.
- The protected worker re-verifies the parent lease or a signed parent digest on a bounded cadence.
- A patched public daemon cannot mint a valid worker capability without the broker/device key and valid lease.

## 7. UIAI Engine split

UIAI Engine should separate its public gateway/recovery surface from protected execution workers.

The public gateway may retain:

- health/version;
- license start/status/refresh/doctor;
- public-safe tool metadata;
- local configuration and diagnostics;
- tokenized public viewers that cannot escalate;
- IPC and protocol contracts.

Protected workers should own selected high-value paths, including browser execution, premium analysis, media generation, proprietary prompt/model-routing policy, or other modules chosen for protection.

Patching gateway middleware to return success must not create a worker, decrypt a capsule, generate a worker token, or reproduce missing proprietary behavior.

## 8. Integrity and signed distribution

All official executables, workers, capsules, manifests, and update metadata are signed.

Required posture:

- verify code/capsule identity before load;
- reject modified or unknown modules;
- pin trusted roots with signed rotation/revocation;
- protect against rollback, freeze, and mixed-version capsule sets;
- bind capsule compatibility to exact contract digests;
- maintain atomic update and rollback;
- mark unofficial/re-signed builds as unsupported and ineligible for protected key delivery;
- never treat a checksum embedded beside an unsigned payload as sufficient authority.

Platform hardening should include native code-signing/runtime protections, restrictive service permissions, sandboxing, and authenticated IPC where supported.

## 9. Obfuscation and anti-analysis

Obfuscation is a secondary cost raiser, never the entitlement root.

Private release builds may apply a reviewed combination of:

- symbol/debug/path removal;
- whole-program optimization and inlining;
- control-flow and constant/data obfuscation;
- string and protocol-name minimization;
- split verification logic across broker and worker;
- per-release check diversification;
- private module packing;
- customer/node watermarking;
- tamper evidence and integrity receipts.

Requirements:

- do not store reusable secrets in obfuscated constants;
- do not depend on undocumented anti-debugging that destabilizes normal customers;
- preserve supportable crash diagnostics through private symbol servers/maps;
- test accessibility, performance, startup, rollback, and false-positive behavior;
- assume a determined analyst can eventually recover local machine code.

## 10. Hardware-backed and fallback assurance classes

The lease/capsule system should expose an assurance class rather than pretending every platform is identical.

| Class | Example posture | Use |
| --- | --- | --- |
| `hardware_bound_attested` | Non-exportable device key with supported attestation | Highest confidence; eligible for sensitive capsule delivery and longer offline windows |
| `hardware_bound` | Non-exportable device key without full app attestation | Strong local key protection |
| `os_protected` | OS credential store bound to user/machine | Standard desktop fallback |
| `software_fallback` | Encrypted local key with reduced trust | Short offline window and reduced protected entitlement |
| `development` | Explicit authority-issued developer lease and private build identity | Internal only; never inferred from source checkout |

The authority may choose different offline windows, capsule availability, or refresh requirements by assurance class.

## 11. Hosted and hybrid protection

Capabilities that are exceptionally valuable, rapidly changing, data-light, or hard to protect locally should remain server-side.

The local product can send a bounded, consented request and receive only the result. Candidate hosted boundaries include:

- proprietary model routing and policy;
- premium evaluation/verification services;
- high-value templates or transformation algorithms;
- fleet coordination and commercial registries;
- abuse-sensitive operations;
- frequently updated intelligence that should not be embedded permanently.

Local-first guarantees remain explicit: ordinary project data, Workpoints, Evidence, and browser state must not be uploaded merely to strengthen licensing.

## 12. Development-agent isolation

Coding and server agents must not automatically receive access to every protection layer.

- Public product agents work from public contracts and synthetic fixtures.
- Private worker repositories use separate GitHub App installations/credentials.
- Authority database and signing operations are exposed through narrow audited tools, not raw filesystem or database access.
- Signing keys live in a dedicated signing service, HSM, KMS, or equivalent isolated mechanism; they are not repository secrets available to general agents.
- Production issuance, key rotation, policy changes, and bulk customer actions require explicit scoped authorization.
- Agents never receive raw production customer keys or reusable signing material.
- Build jobs use short-lived identities and publish signed provenance.

A compromised development agent should not be able to produce an authority-trusted lease or protected module release.

## 13. Leakage deterrence

Protected capsules may be uniquely watermarked at the manifest, metadata, or behavior-safe binary-data layer.

Watermarks must:

- avoid customer source/project data;
- use opaque identifiers;
- not weaken reproducibility or runtime integrity;
- support revocation and leak investigation;
- be disclosed appropriately in commercial terms;
- never become covert telemetry.

Watermarking deters redistribution but does not substitute for technical enforcement.

## 14. Forbidden designs

- hard-coded global decryption key;
- encrypted JavaScript/WASM/native module with its key beside it;
- one central license Boolean;
- public source containing every premium implementation with only UI hiding;
- locally writable tier/feature files treated as authority;
- a gitignored private folder as the sole durable source-control boundary;
- raw license key forwarded to every worker call;
- local token or loopback address converted into paid entitlement;
- unsigned dynamic plug-ins;
- permanent plaintext extraction of protected capsules without access controls;
- anti-debugging that deletes data, disables recovery, or punishes legitimate customers;
- claims that local encryption makes patching impossible.

## 15. Adversarial proof

Release proof must include an authorized bypass exercise in a clean synthetic environment.

A red-team agent is given the public repositories and official evaluation binaries and asked to:

- patch all obvious license branches;
- edit cached leases and limits;
- replace local tokens;
- invoke protected IPC directly;
- replay expired operation tokens;
- copy capsules to another node;
- downgrade broker/worker/capsule versions;
- inject or substitute a worker;
- run without the authority after the offline deadline;
- extract or redistribute protected assets.

Success criteria:

- patched public code cannot create missing protected functionality;
- forged files/tokens/capsules are rejected;
- copied capsules cannot unwrap on another node under hardware/OS-bound modes;
- official update/key delivery stops for modified or unsupported builds;
- recovery/export/uninstall remain safe;
- no user data is destroyed;
- proof records the work factor and residual risks honestly.

## 16. Implementation order

1. Complete Spec 152 signed leases and canonical recovery mode.
2. Identify crown-jewel features to remove from public source.
3. Create private Focusa/UIAI worker repositories and stable IPC contracts.
4. Add protected capsule build, signing, provenance, and delivery.
5. Add device-key registration and content-key envelopes.
6. Add independent worker capability verification.
7. Add platform code signing, service sandboxing, and authenticated IPC.
8. Add update freshness/rollback protections for workers and capsules.
9. Add private-build obfuscation and watermarking only after functional boundaries work.
10. Run adversarial bypass proof before customer distribution.

## 17. Definition of success

The architecture succeeds when changing a public license check to `true` yields only a lying UI or gateway response, while the attacker still lacks:

- a valid signed lease;
- a node-bound capsule key;
- the protected implementation;
- a signed compatible worker;
- a valid operation token;
- limit reservations and authority reconciliation;
- official protected updates and hosted services.

That is the practical standard for making license removal tedious on many independent levels.
