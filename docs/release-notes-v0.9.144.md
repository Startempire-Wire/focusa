# Focusa v0.9.144 Release Notes

**Release lane:** v0.9.144 (locked release candidate REL19)
**Canonical repo:** github.com/Startempire-Wire/focusa
**Headline:** Spec 152 authority licensing with unified Focusa/UIAI entitlement — simple base product, premium families, recovery preservation.

## What's new

### Licensing — one base product, no paywall sprawl (Spec 152/152F)
- **Base Focusa product resolution:** one usable signed product entitlement (Active paid or valid Offline Grace) for product `focusa` gates value-producing core mutations. The legacy `focusa.core.mission` / `focusa.core.workpoint` / `focusa.core.evidence` identifiers resolve as base-product compatibility claims — never separately purchased features.
- **Canonical presenter-neutral projection:** `EntitlementDecisionProjection` + `base_product_projection()` give REST, CLI, TUI, Pi, menubar, and lifecycle presenters one canonical decision; `GET /v1/license/status` and `focusa license status` emit the same decision.
- **Entitlement state grid:** seven binding states × nine capability families resolve to typed `allow / read / base / feature / deny` postures with deterministic reasons; recovery, read, export, and security-maintenance allowances are always reachable under denial.
- **Premium families:** automation, team/remote, release-proof, and premium-updates resolve base-first with authority-owned feature mappings, sequence binding, and cached-only Offline Grace.
- **Dormant granularity:** future feature dimensions stay dormant; no 395 independent paywalls, no caller-controlled product/price/grants.

### EDD/UIAI authority licensing (Spec 152E)
- **Verified-identity EDD customer resolution:** exact email → verified WP user → Stripe ref; evidence-backed merges; never creates unverified stub customers.
- **Verification challenge flow:** facade-branded magic-link/OTP with rate limits, single-use hashed verifiers, enumeration-resistant status.
- **Registration reducer + promotion:** pending → verified → promoted in one idempotent transaction; consent ledger separates transactional from promotional consent; conflicts enter review without partial writes.
- **Authority outbox:** durable order/refund/subscription events with retry + dead-lettering; reconciliation command classifies discrepancies.
- **Checkout intent + terminal delivery:** server-side priced intents bound to registration/customer/product/node; mailbox-verified single-use envelopes for legacy key delivery.
- **Product isolation:** only protected Focusa/UIAI products grant licenses; credit packs excluded; UIAI child-token broker binds to the same verified account; bundle grants both products.
- **Migration safety:** key quarantine (synthetic/duplicate/ambiguous), facade cutover gate, migration canary, recovery-only denial binding.

### Spec 172 overlay
- Verified-no-license family allowlist enforcement; dynamic operation-policy registry binding; license-type lifecycle contracts; EDD operator product projections.

## Reliability & governance
- Final release gap gate repaired to 30/30 checks (installer preservation, tool-contract registry, dry-run mutation fence, agent-first tool audit).
- 136 tool contracts with choreography projections; capability descriptors regenerated; cross-surface parity gates green.
- Acceptance evidence recorded per atom under `docs/evidence/spec152{f,e}/`.

## Notes
- No telemetry by default; no raw keys/tokens/PII in evidence; no push/deploy/release performed by atom agents (governed release workflow).
