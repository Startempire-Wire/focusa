# Licensing Divergence Audit — 2026-08-15 (#119)

**Gate:** Spec 152 mandatory licensing + unified onboarding (slice 3: collapse
all license decisions into one entitlement service).
**State:** IR1 divergence audit; collapse implementation planned.

**2026-09-07 verification (#307 — existing-server developer access):** The
inventory below is historical. Current source centralizes origin discovery and
software feature policy in `focusa-license`; core retains compatibility exports.
Runtime discovery reports `developer_origin_eligible` separately from signed
entitlement. It requires the pinned operator tailnet, stable Self identity and a
bounded cache; failed commands, anonymous health responses and deserialized
caller-supplied guards cannot establish origin. Private network identifiers are
not serialized.

A current authority-issued `focusa_developer` lease selects `DeveloperFull`.
Registered software features share one resolver with Operator inclusion; explicit
signed false claims, lease validity, revocation and independent resource/role
constraints remain binding. Effective status does not rewrite signed claims.
Spec 152 §4 supersedes the historical no-license developer proposal below.

The issuer source adds an internal, non-purchasable profile requiring provider-owned
`first_party_developer_v1` assurance on the existing verified account/node/device
binding. Billing provenance is null, not fabricated. Node-scoped predecessors and
account-wide lease allocation preserve profile-transition continuity without
changing entitlement revisions. Issue #589 tracks EDD SDK limits, decimal/schema
parity and replay revocation checks. Producer fixtures retain byte-identical paid,
Evaluation and bundle vectors; refresh and installed consumer acceptance remain
separate gates.

The PHP-issued Developer vector now verifies in the Rust consumer. Existing-node
renewal passes without historical billing; first-party lease events retain null
billing references without weakening paid-event requirements. The five production
ledger tables were converted to InnoDB with a private backup and all 194 rows
verified unchanged. New issuer tables explicitly require transactional storage.

**Not activated:** no production Developer grant, first-party schema migration,
signing-key change or runtime installation is established by these checks. The installed trust
root failure and protected issuer signing configuration must be resolved through
their canonical paths; origin eligibility is never an unsigned recovery bypass.

## Two engines today

### A. `focusa-license` (445 LOC) — tier/capability engine

- `Tier` enum + commercial/hosted/eval permission predicates.
- `Capability` enum + `CapabilityCheck::{permitted,denied}`.
- Consumers: `focusa-api` (main, routes/license, routes/training),
  `focusa-cli` (commands/license).

### B. `focusa-core::license` (640 LOC) — feature/status engine

- `LicenseMode` enum (Evaluation/Operator/FoundersForge/Team/Enterprise).
- `feature_enabled` / `require_feature` (feature-string gates).
- Local license.json load/activate/validate + registry activation flow.
- `license_developer_origin` (#307): agent-kb/tailnet developer_full resolver.
- Consumers: awareness.rs, binary.rs, device_pairing.rs, release.rs,
  export.rs (+ the license CLI).

## Divergences

| Axis | focusa-license | focusa-core::license |
| --- | --- | --- |
| Identity | Tier | LicenseMode |
| Gate unit | Capability | feature string |
| State | registry lease posture | local license.json + hashes |
| Developer origin | absent | present (#307) |
| Decision points | 4 call sites | 5 call sites |

## Collapse plan (IR2+)

1. Unify on one canonical `EntitlementService` (in `focusa-license`):
   tier + capabilities + feature aliases + developer-origin resolution.
2. `focusa-core::license` becomes a thin facade (deprecated but preserved
   for API compatibility during the transition release).
3. All nine decision points call the service; feature strings map to
   capabilities in one table.
4. Remove self-issued/no-file Evaluation and local tier overrides.
   The original no-license developer proposal was superseded by Spec 152:
   developer entitlement is authority-issued; origin is eligibility evidence.
5. Signed authority-lease verification + recovery-only startup
   (Spec 152 slices 2) lands on top of the unified service.

## Evidence

- Consumer map + engine inventory captured above (both crates verified in
  the current tree).
- `cargo check --workspace` clean; no behavioral change until IR2 merges.
