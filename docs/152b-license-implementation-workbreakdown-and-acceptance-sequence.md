# Spec 152B — Client Implementation Work Breakdown and Acceptance Sequence

**Status:** Implementation plan; P0 release work  
**Created:** 2026-08-01  
**Tracks:** `Startempire-Wire/focusa#119`  
**Authority dependency:** `WPUIAI/wpuiai#1`  
**UIAI dependency:** `WPUIAI/uiai-engine#5`

## Purpose

Convert Specs 150A, 152, and 152A into an ordered Focusa implementation sequence. Each work package must preserve recovery/export/uninstall, produce focused proof, and avoid creating a second entitlement authority.

## WP0 — Baseline and fixture isolation

- Claim a Beads work item and record current head, active Workpoint, affected surfaces, and test mode.
- Inventory all license reads/writes, feature gates, installer flags, status projections, generated descriptors, and pairing/first-run dependencies.
- Add production-build assertions rejecting test trust roots, `test_fixture`, and local bypass flags.
- Preserve current paid/evaluator state only as non-authoritative migration fixtures.

Acceptance:

- complete code-surface inventory;
- no unclassified license decision path;
- test fixture boundary proven.

## WP1 — Canonical lease schema and verifier

Implement in `crates/focusa-license`:

- versioned signed envelope and payload types;
- RFC3339 time parsing;
- product-qualified feature and limit claims;
- node id, lease sequence/digests, authority key id;
- Ed25519 verification over the canonical domain-separated payload supplied by the authority contract;
- signed key-set rotation/revocation verification;
- stable entitlement states and errors;
- secure persisted lease/refesh metadata with raw credentials excluded;
- golden vectors shared with authority and UIAI.

Remove or disable:

- self-issued Eval;
- local TOML tier override;
- non-empty environment key => Licensed;
- non-cryptographic fingerprint labeled SHA-256;
- unknown tier => Operator;
- blanket Licensed/Open capability grants.

Acceptance:

- forged/edited/wrong-product/unknown-key/stale-sequence/expired/revoked tests;
- Rust verifies authority golden vectors byte-for-byte;
- missing state returns `unactivated/recovery_only`.

## WP2 — Collapse duplicate license authority

Refactor `crates/focusa-core/src/license.rs`:

- stop making decisions from plaintext local fields;
- preserve legacy parsing only as migration input;
- delegate status/features to the canonical verifier;
- make `require_feature` include status/product/node/time/sequence/offline/limit decisions;
- remove mode inference from empty features or `eval` booleans.

Acceptance:

- daemon/CLI/core report the same immutable entitlement snapshot;
- no second feature table or tier fallback remains.

## WP3 — Recovery-mode daemon gate

Add central API classification:

- public health/version;
- recovery license start/poll/activate/refresh/status/doctor;
- safe inventory/export/backup/repair/uninstall;
- entitled read/mutate/execute route groups;
- product-qualified middleware before side effects;
- redacted structured denial envelopes.

Acceptance:

- missing/invalid/expired/revoked state starts daemon but denies mutation/execution;
- Workpoint/Evidence/turn/Silent Session/gated export/update tests fail before side effects;
- recovery operations preserve user data.

## WP4 — Device-code license client

Extend `focusa license`:

```text
start
poll
activate/exchange-key
refresh
status
doctor
nodes
deactivate-node
```

Requirements:

- no raw key in process list/query/logs/receipts by default;
- bounded polling and stable errors;
- idempotency/request ids;
- OS-protected refresh/node credentials;
- authority origin pinning/configuration policy;
- offline grace only from previously signed claims.

Acceptance:

- synthetic authority E2E for Evaluation, paid, bundle, pending, slow_down, expiry, wrong product, revoke, refund, node limit, outage.

## WP5 — Installer replacement

Update Rust installer, Bash, and PowerShell:

- retain non-mutating preflight without license;
- remove production self-issued `--eval` semantics;
- resolve existing lease or initiate device code before runnable asset activation;
- bind Spec 150 transaction to `LifecycleEntitlementBinding`;
- retrieve protected components only after grant;
- start daemon in entitled/recovery posture;
- verify canonical daemon status before success;
- preserve uninstall and explicit purge.

Acceptance:

- Bash/PowerShell/Rust parity;
- interrupted authorization/install/rollback recovery;
- no partial product-ready receipt;
- all presenters redact secrets/email/capsule envelopes.

## WP6 — Spec 150 lifecycle binding

Extend lifecycle models/orchestrator/adapters/receipts with Spec 150A fields.

- immutable entitlement snapshot reference;
- operation-required features;
- UIAI product posture separately from health;
- paid/evaluation/development/recovery/blocked receipt classes;
- update feature checks;
- protected component set digest.

Acceptance:

- Spec 150 mechanics remain passing;
- acceptance impossible with invalid/missing/wrong-product lease;
- lifecycle receipt reconciles with authority and daemon.

## WP7 — Menubar entitlement-first onboarding

Refactor first-run state machine:

```text
release/recovery health
→ Evaluate/Activate/Manage
→ device code/account verification
→ lease verification
→ optional UIAI grant
→ pairing
→ first project/Workpoint
```

Requirements:

- pairing token never widens entitlement;
- Keychain/Secure Enclave/platform storage for node/token material;
- evaluation remaining/locked-feature/manage-license UI;
- expiry/revoke recovery UI;
- no connection-saved-only completion.

Acceptance:

- restart/resume at every state;
- callback/deep-link/manual fallback;
- unified completion receipt.

## WP8 — Pi/TUI/CLI/REST capability coverage

- Add product-qualified `license_feature` and optional limit bucket to every execution/mutation descriptor.
- Generate coverage ledger across runtime registration, CLI, REST, Pi, TUI, menubar, workers, docs.
- Keep locked tools discoverable; deny before side effects.
- Project identical status/error semantics across interfaces.

Acceptance:

- zero unmapped mutable/execution surfaces;
- generated parity gates pass.

## WP9 — UIAI bundle broker

- Consume explicit bundle/UIAI product grant.
- Issue/request short-lived audience-bound child token.
- Bind parent lease id/sequence/digest, node/client, features, limits, expiry, token id/nonce.
- Never forward raw commercial key.
- Revoke/expire/refund propagation and refresh.

Acceptance:

- UIAI independently accepts valid narrow token;
- rejects Focusa pairing/local token, wrong audience, broader scope, stale parent, replay, expired token.

## WP10 — Protected workers/capsules

After core licensing is proven:

- choose first crown-jewel feature family;
- publish stable public IPC;
- private signed worker/capsule build;
- node-bound key envelope;
- short-lived operation capability;
- atomic update/rollback and compatibility;
- adversarial patch/copy/extract/substitute/downgrade tests.

Acceptance:

- patched public gate cannot execute missing protected capability;
- no global content key in client/repo;
- unsupported hardware has explicit reduced-assurance policy.

## WP11 — Migration

- paid keys exchange without repurchase and preserve contractual product rights;
- current evaluators receive bounded verification window;
- legacy local records archived as non-authoritative evidence;
- paired devices preserved but execution requires entitlement;
- real developers migrate to authority-issued developer licenses;
- no local Eval grandfathering;
- no Workpoint/Evidence/project deletion.

Acceptance:

- migration fixtures and counts reconcile;
- paid activation works without reinstall;
- rollback does not roll back revocation/sequence/absolute Evaluation expiry.

## WP12 — Release closure

Required green evidence:

- documentation consistency workflow;
- authority/client/UIAI golden vectors;
- all interface coverage ledger;
- installer platform matrix;
- recovery/data preservation;
- staging device-code/lease/refund/revoke/node E2E;
- UIAI standalone/bundle E2E;
- test-root/bypass exclusion;
- protected-component proof when included;
- cohort entry checklist.

Release claim remains:

```text
blocked for new evaluator/customer distribution
```

until all applicable packages pass and the authority, Focusa, and optional UIAI receipts reconcile.
