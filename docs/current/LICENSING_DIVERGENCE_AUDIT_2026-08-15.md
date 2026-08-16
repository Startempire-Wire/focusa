# Licensing Divergence Audit — 2026-08-15 (#119)

**Gate:** Spec 152 mandatory licensing + unified onboarding (slice 3: collapse
all license decisions into one entitlement service).
**State:** IR1 divergence audit; collapse implementation planned.

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
4. Remove self-issued/no-file Evaluation and local tier overrides;
   developer origin becomes the only no-license path (#307).
5. Signed authority-lease verification + recovery-only startup
   (Spec 152 slices 2) lands on top of the unified service.

## Evidence

- Consumer map + engine inventory captured above (both crates verified in
  the current tree).
- `cargo check --workspace` clean; no behavioral change until IR2 merges.
