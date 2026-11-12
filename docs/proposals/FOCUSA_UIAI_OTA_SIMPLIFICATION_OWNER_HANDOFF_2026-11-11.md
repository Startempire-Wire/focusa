# Focusa + UIAI Engine — OTA Simplification and Release/Install Authority Alignment

**Date:** 2026-11-11 (operator instruction, Pacific time)  
**Status:** OWNER-DIRECTED PROPOSAL / IMPLEMENTATION HANDOFF; **no runtime change, approved release, installed upgrade, or production proof in this PR**  
**Primary code owner:** `Startempire-Wire/focusa` (canonical Focusa installer, update policy, trust and release authority)  
**Consuming code owner:** `WPUIAI/uiai-engine` (separate Go worker deploy and Cockpit native Tauri updater; requires its own focused implementation PR)  
**Purpose:** Make Focusa OTA substantially easier to understand and operate while sharing the **minimal release/update contract** with UIAI Engine, without duplicating installers or weakening cryptographic/release guarantees.

## 1. Preserve earlier owner simplification decisions

This is a continuation, **not** a replacement of:

1. [2026-08-20 Elon 5-step OTA simplification](../current/2026-08-20-elon-5-step-ota-simplification.md) — delete unnecessary dev-mode double-confirmation; preserve signatures/checksums and rollback.
2. [2026-08-17 canonical release optimizations](../current/RELEASE_CANONICAL_OPTIMIZATIONS_2026-08-17.md) — one stamping authority; dev fast lane; real source/CI proof rather than redundant retries.
3. [Spec 145 provider-neutral canonical release control plane](../145-focusa-canonical-core-release-cycle-fast-release-architecture.md) and [canonical live release pipeline](../canonical-live-release-pipeline.md) — one release decision, immutable candidate, bounded evidence DAG, actual installed/running receiver proof.
4. [Spec 128 Focusa OTA](../128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md), [current installer policy](../current/INSTALLER_UPDATE_POLICY.md), [production consistency policy](../current/PRODUCTION_CONSISTENCY_POLICY.md) — full supported surface coverage, signed entitlement, safe scheduling, prior-version rollback, user data preservation.

Apply the original owner's sequence: **question each requirement → delete redundant work → simplify the remaining paths → optimize → automate**, preserving real failure-closed boundaries.

## 2. Source-grounded architecture (what exists, not an imagined universal updater)

### Focusa — multiple components and authority adapters

- [`crates/focusa-core/src/update.rs`](../../crates/focusa-core/src/update.rs): release/channel/policy/manifest/trust eligibility and cryptographic primitives (~1,621 source lines at audit).
- [`crates/focusa-cli/src/commands/update.rs`](../../crates/focusa-cli/src/commands/update.rs): commands `status/check/plan/apply/rollback/scheduler/history/admin`, GitHub release resolution, trust and compatibility, staging, locks, backups and recovery (~5,089 lines).
- [`crates/focusa-cli/src/commands/update_trust.rs`](../../crates/focusa-cli/src/commands/update_trust.rs): pinned signing key, signed release key set, manifest, provenance, detached SHA256SUMS, asset signatures, deployed-daemon proof and compatibility canary (~685 lines).
- [`crates/focusa-api/src/routes/update.rs`](../../crates/focusa-api/src/routes/update.rs): read/projection, policy, scheduler and guarded mutation routes; [`update_authority.rs`](../../crates/focusa-api/src/routes/update_authority.rs) intentionally calls **the canonical CLI** for plan/apply/rollback instead of maintaining a second binary installer.
- **Important existing convergence:** manifest-bound OTA already invokes the canonical Rust install transaction via `crate::commands::install::run` (in `execute_manifest_bound_apply`), while legacy per-part replacement remains in the older path. **Reconcile, then retire** equivalent legacy paths only when compatible and independently proven.
- Platform scheduling is `systemd` or `launchd` calling the same CLI `update apply` (nominal 120-second interval); the Focusa Menubar additionally uses the **native Tauri updater** and checks `/v1/update/policy` before automatic apply.
- Latest publicly published GitHub Focusa release observed during prior review: `v0.9.207` with a full multi-platform asset matrix. Published artifact presence is not proof of a live customer's installation.

### UIAI Engine — two distinct delivery lanes

- [Go engine parent release](https://github.com/WPUIAI/uiai-engine/blob/main/docs/ENGINE_RELEASE_DEPLOY_RUNBOOK.md): immutable `engine-v*` / `uiai-engine-v*` release with a linux/amd64 binary/checksum and authorized OVH [deploy script](https://github.com/WPUIAI/uiai-engine/blob/main/scripts/deploy-engine-ovh.sh). The script checks idle sessions, backs up the previous binary, replaces/restarts, verifies service health, browser smoke and **running process hash**. This is **not** the Focusa multi-component OTA client.
- [Cockpit Tauri updater](https://github.com/WPUIAI/uiai-engine/blob/main/apps/cockpit/src/lib/updater.ts): native signed `check → downloadAndInstall → relaunch → verify running version`, separate `cockpit-v*` immutable release; mutable `cockpit-latest` feed contains `latest.json`, native updater archives/signatures.
- [Cockpit Tauri config](https://github.com/WPUIAI/uiai-engine/blob/main/apps/cockpit/src-tauri/tauri.conf.json) points to `cockpit-latest/latest.json`; the current auto-check calls `setInterval(..., 5_000)`, which is extremely frequent for an ordinary release channel. Focusa Menubar checks at startup, not every five seconds.
- UIAI currently [documents Focusa release architecture as a canonical mirror](https://github.com/WPUIAI/uiai-engine/blob/main/docs/release-canonical-mirror-adapter-map.md). **This states shared policy and process, not one actual updater implementation.**
- The public GitHub parent Latest and Cockpit desktop `cockpit-latest` are **different selectors**. Preserve them, and prevent Cockpit release metadata from accidentally displacing the parent engine's Latest.
- Neither product's current source and release listings alone constitute live installed OTA/rollback proof.

### Observed architectural complexity

Focusa currently mixes **producer-side release qualification** (CI/release/deploy proof), **client-side signed assertion validation**, **policy/entitlement**, **scheduling**, **legacy per-part installation**, and **native Tauri desktop updates** around one user-intent `update`. Real safety is mandatory; repeated reconstruction of the whole release pipeline on every client is not.

Related defects requiring explicit regression gates:
- [Focusa #670](https://github.com/Startempire-Wire/focusa/issues/670): signing and native Tauri Minisign sidecar collision / update feed integrity.
- [Focusa #680](https://github.com/Startempire-Wire/focusa/issues/680): quota exhausted and health false-green; updater must preflight actual writable storage, not merely liveness.
- [Focusa PR #710](https://github.com/Startempire-Wire/focusa/pull/710): independent release bottleneck/CI repair. **Do not mix with or supersede that PR.**
- [UIAI #306](https://github.com/WPUIAI/uiai-engine/issues/306): independent Go standard-library patched-toolchain security gate; this simplification does not waive it.

## 3. Target: one minimal **contract**, three existing execution owners

```text
EXISTING PRODUCT RELEASE PUBLISHER (Focusa or UIAI source owner)
  compile → test → signed complete artifacts → exact candidate receipts
  └── published, immutable, signed release descriptor/index
                  |
          UPDATE DECISION / POLICY
  resolve channel → verify trusted signed descriptor + artifact hashes
  → check anti-rollback, revocation, compatibility, owner-granted entitlement
  → return one typed plan/status (NO live re-execution of release CI)
                  |
      ┌───────────┴─────────────────┐
      │                             │
FOCUSA canonical Rust          UIAI Go service deploy
installer transaction          (existing OVH adapter)
CLI/daemon/TUI/Pi/etc.         immutable binary and rollback
      │                             │
      └──────────────┬──────────────┘
                     │
       Native Tauri updater per desktop app
       (Focusa Menubar, UIAI Cockpit)
       Tauri's own signature format/key remains authoritative
                     │
       ACTIVATION: installed version + actual running
       process/receiver + smoke + rollback receipt
```

**Authority rules:**
- **Release publisher is sole authority for full release qualification.** It signs a compact, tamper-evident assertion of the released source SHA, version/channel, supported platforms, expected artifact hashes/URLs, signer/key, eligibility flags, and proof references. It never delegates release publication authority to an updater client or new external CI provider.
- **Client must still verify cryptography, not blindly trust JSON.** Existing pinned trust root, signed descriptor, signatures/hashes of every used artifact, signed revocation/yank status, timestamp/replay/anti-rollback and compatibility must fail closed; the client does **not** need to query and reconstruct every CI job/deploy workflow in ordinary signed OTA consumption.
- **One Focusa binary/package mutation authority:** existing Rust install lifecycle, invoked by OTA and installer. Keep state/DB/config, entitlements, secret storage, ownership, platform service units, safe stop/restart, atomic backup/journal, versioned migration and exact-previous-release rollback.
- **UIAI Go worker stays its own service deployment adapter.** Cockpit and Focusa desktop apps continue to use native Tauri signed updater/activation, not the CLI binary replacer; this is a different artifact type, not a second source of cryptographic truth.
- **Do not conflate Tauri Minisign `.sig` and release Ed25519 signatures.** Respect #670 and preserve detached sidecars separately; no signature overwrite, key substitution, or weakened trust under the shared semantic contract.
- **No new required cloud updater service, database, scheduler, memory, agent, key authority or top-level OS.** Reuse GitHub Releases or approved existing provider transport as distribution; Sovereign installs may use an approved offline signed mirror/bundle with the same descriptor.
- **Do not collapse exact publisher proof into an unsigned boolean.** The signed descriptor must be issuer-bound, versioned, expire/revoke safely and refer to immutable independently inspectable receipts. Publication itself remains fail-closed for incomplete supported release matrices.

### Minimal semantic descriptor (contract proposal, not yet serialized production schema)

```text
schema/version | product | channel | release tag | source commit
signing key id + manifest signature | issued/expires | revoked/yanked
compatibility: OS/arch/ABI + minimum installed version + schema-migration bounds
artifact map: component, target, digest, immutable URL, signature reference
publisher verdict + immutable CI/release/deploy proof reference(s)
```

This is **not** a request to invent a second release manifest when Focusa already has `focusa.release_manifest.v1` and `distribution-manifest.json`. First audit the existing fields; reuse or define a backwards-compatible *projection* and a UIAI adapter, rather than replacing working signed contracts. Native Tauri feed remains the Tauri schema.

## 4. Owner experience and channel policy

The user-visible goal is **one command and one truthful outcome** (up-to-date / updated-and-activated / blocked-with-reason / failed-and-restored / activation-unconfirmed). Preserve existing CLI subcommands and API schemas during migration; add a concise front door or alias **only after** proving backwards compatibility.

- **Developer or opted-in managed development:** fast bounded automatic checks/updates, no redundant second consent once a valid standing grant is already in force; no bypass of signatures, installed readiness, or release identity.
- **Stable customer:** notify/opt-in auto-update respecting maintenance windows, downtime tolerance, prior release and rollback.
- **Sovereign owner:** explicit local version pin, maintenance policy, trusted offline bundle/mirror and zero implicit cloud remote-control permission.
- **Desktop cadence:** UIAI Cockpit's current 5-second interval should become a bounded, channel-aware setting; e.g., a few minutes for dev and hours for stable, with jitter/backoff and explicit user-triggered check. Verify actual code/tests and customer behavior before selecting exact default values.
- **Interrupted installations:** recover from durable local transaction journal; no retry-induced duplicate restarts/effects; fail closed if new data migration prevents lossless rollback, requiring prior compatible backup and an explicitly tested forward recovery path.

## 5. Three focused implementation PRs — source ownership and acceptance

### PR-A — Focusa contract projection + decision boundary (Focusa source owner)

1. Inventory `release-manifest.json`, distribution manifest, signed `SHA256SUMS`, key metadata, immutable receipts, current loader and policy serialization, and UIAI delivery equivalents.
2. Specify minimal existing-field signed manifest view + a single reusable **read-only** `check/plan` decision that verifies signature, hash, provenance issuer, channel, revocation, compatibility, entitlement and anti-rollback without repeatedly querying GitHub Actions.
3. Keep all historical pinned trust, old release handling and versioned API compatibility. Reject stale/unsigned/foreign identity, wrong platform, wrong bundle and injected URL.
4. Proof: old→new manifest compatibility, malformed and revoked fixtures, offline signed bundle, unchanged CI publisher trust, same effective policy decisions.

**Do not promote this docs-only PR as PR-A implementation.**

### PR-B — Focusa canonical installer convergence + cleanup (Focusa source owner)

1. Keep manifest-bound `execute_manifest_bound_apply → commands::install::run` authoritative. Inventory **all** legacy per-part update paths and component-specific copies; delete only after actual replacement fixtures and real supported old-version upgrade prove parity.
2. Make scheduler and daemon API thin callers of the one policy + transaction; test systemd/launchd, Windows supported installation/update, Pi extension reload, CLI/TUI/session-runner version convergence, Menubar Tauri independent update and no two concurrent mutations.
3. Preserve local data, permissions, service config, exact previous version backup; assert canceled/failed/mid-crash/missing-disk/stale-signature/migration failure → correct restored or actionable blocked state.
4. Proof: source+consumer+cross-version+real installed acceptance on supported OS, exact running executable SHA/manifest, rollback verified by independent receiver; no false green.

### PR-C — UIAI Engine mirror contract, Go worker + Cockpit cadence (UIAI code owner)

1. Adapt same *semantic* signed descriptor/policy vocabulary to Go Engine's existing `engine-release.yml`/OVH deploy adapter and Tauri Cockpit's native updater feed—**no Focusa CLI dependency for Go worker**.
2. Keep immutable parent Engine Latest separate from Cockpit mutable `cockpit-latest`; native Tauri signature/activation and 2-architecture Mac artifact proof remain mandatory.
3. Replace Cockpit 5-second automatic polling with owned channel-based intervals, jitter, exponential backoff, no-overlap and an immediate manual check.
4. Proof: corrupted signature refuses install, existing authorized user preferences preserved, release feed unavailable behavior truthful, installed Cockpit version receipt, worker idle gate/backup/health/running hash/rollback tested. Address any independent Go security gate, including [UIAI #306](https://github.com/WPUIAI/uiai-engine/issues/306), without hiding it in updater refactor.

### Required safe ordering

`PR-A → PR-B` in Focusa; `PR-C` can prepare in parallel once PR-A's signed semantic contract is stable. **No risky all-at-once rewrite.** Each PR has its own exact SHA, regression receipt, rollback, owning repo and independent installed consumer verification. Do not auto-merge other agents' divergent release branches.

## 6. Acceptance: what makes the new architecture objectively better

**Safety/equivalence gates (mandatory):**
- Signed complete stable and dev releases remain fail-closed across the current supported Linux/macOS/Windows matrix; no change to producer acceptance quality or trust roots.
- No release client accepts unsigned, revoked, yanked, wrong-source, wrong-channel, wrong-tenant, downgrade or injected URL artifact.
- Focusa components converge to one approved compatible release; Desktop Tauri retains native trust and independent activation proof. User/customer data survives error, update and rollback.
- One update call yields a typed result and exact installed + running receipt; a failed activation is **not** reported as success.
- Offline/Sovereign, permission denied, scheduler outage, low disk, interrupted download, process crash, second concurrent updater, broken migration and rollback cases are covered.

**Simplicity metrics (measured against baseline, not guessed):**
- Reduce number of **independently mutating** Focusa updater paths to **one canonical install transaction**; keep only necessary platform adapters.
- Reduce duplicate policy/signature decision implementations and redundant network/proof requests per successful update; report source/behavior diff.
- Record `check` latency, bytes fetched, update elapsed, restart downtime, wasted repeat CI/provider requests, retry count and mean operator actions for dev/stable/Sovereign.
- No additional mandatory service, signing provider, database, or GUI dependency.
- Provide one human-readable `focusa update status` or equivalent summary showing `installed/current/target/channel/eligibility/blocker/next action` without requiring operators to learn the internal manifest terminology.

## 7. Handoff and boundaries

- This PR is **an architecture-and-delivery amendment only**. It does not implement code or authorize unattended updates on new customer devices. It should remain draft until the owning team reviews proof requirements.
- Existing Focusa signed release / install policy is canonical until a proven backwards-compatible migration; avoid replacing accepted published manifests with an invented cross-repo format.
- Workstreams/Beads/issue IDs must be real IDs returned by the owning runtime; do not invent Focusa operation admission or claim deployment from GitHub docs.
- Keep [Focusa PR #710](https://github.com/Startempire-Wire/focusa/pull/710) release-bottleneck and provider-credit pilot **independent**. Its CI fix is not OTA refactor acceptance.
- UIAI owning team must receive a separate PR-C handoff/implementation task. A Focusa PR cannot merge UIAI code or alter its installed Cockpit.

**Current disposition:** design/handoff prepared for agents. Runtime simplification, provider CI, signed OTA integrity, and actual installed proof remain separate work.
