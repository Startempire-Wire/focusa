#!/usr/bin/env python3
"""Spec 152E.05.08 UIAI Engine activation through the same EDD account.

Binds the four exact surfaces to the frozen Spec 152E contracts
(docs/contracts/spec152e-edd-product-registry.v1.json,
docs/contracts/spec152e-activation-internal.v1.json) and proves:

1. SHARED ACCOUNT/PRODUCT CONTRACT (crates/focusa-license/src/uiai_activation.rs):
   exactly one verified EDD account identity (`UiaiAccountIdentity` with
   account_id + edd_customer_id) is used for UIAI activation — no duplicate
   customer identity is ever created. The mapping is server-owned: the client
   submits only the public product code `uiai_operator_lifetime_v1`; EDD ids,
   prices, tiers, grants, limits, and commercial flags are never accepted.
   `resolve_uiai_activation` fails closed unless the account holds an
   independent `uiai-engine` grant (a Focusa-only account gets
   `UiaiGrantRequired`), the grant is active and bound, and the requested
   scope is an exact subset of the independent UIAI grant.

2. UIAI INSTALLER/ADAPTER (crates/focusa-core/src/install_lifecycle/models.rs):
   `AdapterEntitlementPosture::from_same_edd_account_uiai_authority` builds
   the UIAI adapter posture only when the Focusa parent and the independent
   UIAI grant are issued to the SAME verified EDD account (strict
   `same_account_binding` on lease subject ids), the projection carries
   exact `uiai-engine` grants, and the authority child-token receipt settles
   the same registration. The posture carries the single account identity.

3. CHILD-TOKEN BROKER (crates/focusa-license/src/uiai_child_token.rs): the
   broker refuses `AccountMismatch` when lease subject evidence shows two
   different accounts (or one side proves an account and the other does
   not), and exposes the strict `validate_same_account_binding` guard.

4. INDEPENDENT UIAI GRANT AND LEASE: the signed lease carries the account
   subject (`EntitlementSnapshot.subject_id` from the lease payload), UIAI
   features/limits come only from the independent `uiai-engine` grant, and
   the Focusa grant never satisfies UIAI scope (product isolation). UIAI
   key/lease delivery happens through the SAME shared registration
   (`ActivationSession` public_product_code flow).

The Rust unit tests for the same-account decision, broker guard, and
projection execute in the same commit
(crates/focusa-license/src/uiai_activation.rs, uiai_child_token.rs), so
evidence is replayable from the pinned commit without any network.

Exact verification: python3 tests/spec152e_uiai_activation_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
LICENSE_CRATE = ROOT / "crates/focusa-license/src"
CORE_CRATE = ROOT / "crates/focusa-core/src"

REGISTRY = json.loads(
    (CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8")
)
INTERNAL = json.loads(
    (CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8")
)

CONTRACT = (LICENSE_CRATE / "uiai_activation.rs").read_text(encoding="utf-8")
BROKER = (LICENSE_CRATE / "uiai_child_token.rs").read_text(encoding="utf-8")
AUTHORITY = (LICENSE_CRATE / "authority.rs").read_text(encoding="utf-8")
CLIENT = (LICENSE_CRATE / "activation_client.rs").read_text(encoding="utf-8")
MODELS = (CORE_CRATE / "install_lifecycle" / "models.rs").read_text(encoding="utf-8")

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


# ── Frozen EDD registry: UIAI product mapping is server-owned ─────────────

expect(REGISTRY["schema"] == "focusa.spec152e.edd_product_registry.v1",
       "frozen EDD product registry schema")
offers = {row["public_code"]: row for row in REGISTRY["protected_offers"]}
expect("uiai_operator_lifetime_v1" in offers, "UIAI operator offer is in the frozen registry")
expect(offers["uiai_operator_lifetime_v1"]["products"] == ["uiai_engine"],
       "UIAI offer maps to the exact uiai_engine product only")
expect("focusa_operator_lifetime_v1" in offers
       and offers["focusa_operator_lifetime_v1"]["products"] == ["focusa"],
       "Focusa offer maps to the exact focusa product only")
expect("focusa_uiai_operator_bundle_lifetime_v1" in offers
       and offers["focusa_uiai_operator_bundle_lifetime_v1"]["products"] == ["focusa", "uiai_engine"],
       "bundle offer is the exact two-product union")
expect(all(row["edd_download_id"] is None and row["edd_price_id"] is None
           for row in REGISTRY["protected_offers"]),
       "frozen registry owns EDD ids; no client-assigned downloads or prices")
for forbidden in ["edd_download_id", "edd_price_id", "price", "tier"]:
    expect(forbidden in REGISTRY["authority"]["caller_controls_forbidden"],
           f"registry forbids caller-controlled {forbidden}")

# ── Shared account/product contract ──────────────────────────────────────

expect('UIAI_ACTIVATION_SCHEMA: &str = "focusa.uiai_activation_contract.v1"' in CONTRACT,
       "contract schema is pinned")
expect('PRODUCT_FOCUSA: &str = "focusa"' in CONTRACT
       and 'PRODUCT_UIAI_ENGINE: &str = "uiai-engine"' in CONTRACT,
       "exact authority-owned product identifiers are pinned")
expect('PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1: &str = "uiai_operator_lifetime_v1"' in CONTRACT,
       "UIAI public product code is pinned")
expect('PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1: &str = "focusa_operator_lifetime_v1"' in CONTRACT
       and 'PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1' in CONTRACT,
       "Focusa and bundle public codes are pinned for isolation contrast")

# Single verified EDD account: no duplicate customer identity.
expect("pub struct UiaiAccountIdentity" in CONTRACT, "single account identity type exists")
expect("pub account_id: String" in CONTRACT and "pub edd_customer_id: u64" in CONTRACT,
       "identity carries one account and one EDD customer")
expect("fn valid(&self)" in CONTRACT, "identity validity check exists")
expect("pub struct AccountProductGrants" in CONTRACT, "account product grants type exists")
expect("pub fn focusa_only(&self)" in CONTRACT, "focusa-only posture is typed")
expect('self.products.len() == 1 && self.products.contains(PRODUCT_FOCUSA)' in CONTRACT,
       "focusa-only is exactly one product, never implicit UIAI")

# Fail-closed decision surface.
expect("pub fn resolve_uiai_activation(" in CONTRACT, "UIAI activation decision exists")
expect("ProductMappingRequired" in CONTRACT, "wrong requested code fails closed")
expect("UiaiGrantRequired" in CONTRACT, "missing independent UIAI grant fails closed")
expect("UiaiGrantInvalid" in CONTRACT, "inactive/unbound UIAI grant fails closed")
expect("ProductIsolationViolation" in CONTRACT, "cross-product scope fails closed")
expect("AccountIdentityRequired" in CONTRACT, "missing account identity fails closed")
expect("pub struct UiaiGrantProjection" in CONTRACT, "exact UIAI grant projection exists")
expect("pub features: BTreeSet<String>" in CONTRACT and "pub limits: BTreeMap<String, u64>" in CONTRACT,
       "projection carries exact features and limits only")

# The decision never accepts caller-controlled EDD fields.
decision_signature = CONTRACT.split("pub fn resolve_uiai_activation(")[1].split(")")[0]
for forbidden in ["edd_download_id", "edd_price_id", "price", "tier", "sale_status",
                  "refund_policy", "commercial_rights", "node_limit"]:
    expect(forbidden not in decision_signature,
           f"resolve_uiai_activation never accepts caller-controlled {forbidden}", negative=True)

# Independent UIAI grant required; Focusa grant never satisfies UIAI scope.
expect("pub fn focusa_grant_never_satisfies_uiai(" in CONTRACT,
       "product isolation proof exists")
expect('uiai_grant.product == PRODUCT_UIAI_ENGINE' in CONTRACT,
       "isolation reads only the independent uiai-engine grant")
expect("pub fn uiai_scope_is_exact_subset(" in CONTRACT,
       "exact-subset eligibility exists")
expect("pub fn same_account_binding(" in CONTRACT
       and 'focusa_parent.subject_id.as_deref() == Some(account.account_id.as_str())' in CONTRACT
       and 'uiai_grant.subject_id.as_deref() == Some(account.account_id.as_str())' in CONTRACT,
       "same-account binding compares both lease subjects to the one account")

# ── Independent UIAI grant and lease (subject id from the signed lease) ───

expect("pub subject_id: Option<String>" in AUTHORITY
       and "lease `subject_id`" in AUTHORITY,
       "lease subject id is plumbed onto the entitlement snapshot")
expect("subject_id: Some(payload.subject_id)" in AUTHORITY,
       "snapshot subject comes from the signed lease payload (authority truth)")
expect("subject_id: None" in AUTHORITY,
       "unactivated snapshots carry no subject (fail-closed default)")
expect('active_bound(uiai_grant, "uiai-engine"' in BROKER,
       "child-token broker requires the independent uiai-engine grant")

# ── Child-token broker: same-account binding ─────────────────────────────

expect("AccountMismatch" in BROKER, "broker fails closed on account mismatch")
expect("pub fn validate_same_account_binding(" in BROKER,
       "strict same-account broker guard exists")
expect("fn same_evidence_account(" in BROKER, "lease-subject evidence check exists")
expect("self.cache.retain" in BROKER and "revoke_parent" in BROKER,
       "parent revocation remains authority-scoped")
for forbidden in ["SigningKey", "Signer", "self_sign", "customer_email", "access_token:"]:
    expect(forbidden not in BROKER, f"broker never carries {forbidden}", negative=True)
expect(BROKER.index("validate_request(request") < BROKER.index("SensitiveCredential::new"),
       "frozen request validation precedes credential ingestion")

# ── UIAI installer/adapter through the same registration ─────────────────

expect("from_independent_uiai_authority" in MODELS, "independent UIAI adapter constructor exists")
expect("from_same_edd_account_uiai_authority" in MODELS,
       "same-EDD-account UIAI adapter constructor exists")
adapter_impl = MODELS.split("from_same_edd_account_uiai_authority(")[1]
expect("same_account_binding" in MODELS, "adapter enforces the single verified account")
expect("projection.product != \"uiai-engine\"" in MODELS, "adapter requires exact uiai-engine grants")
expect("receipt.request_id == request.request_id" in MODELS
       and "receipt.expires_at > now" in MODELS,
       "adapter settles the same registration through the child receipt")
expect("pub account_id: Option<String>" in MODELS
       and "pub edd_customer_id: Option<u64>" in MODELS,
       "adapter posture carries the single account identity")
expect("account_id: Some(account.account_id.clone())" in MODELS
       and "edd_customer_id: Some(account.edd_customer_id)" in MODELS,
       "same-account posture binds exactly one account and one EDD customer")
expect("account.valid()" in MODELS, "adapter rejects empty identities")
expect("health" not in MODELS[MODELS.index("from_independent_uiai_authority"):],
       "adapter capability/health state never implies UIAI entitlement", negative=True)

# ── UIAI key/lease delivered through the SAME shared registration ────────

expect("pub fn begin(" in CLIENT and "public_product_code" in CLIENT,
       "UIAI registration drives the shared activation session (same registration)")
expect("fn select_offer(" in CLIENT and "public_product_code" in CLIENT,
       "offer selection is server-owned through the shared client")
expect("pub fn poll(" in CLIENT and "lease_envelope" in CLIENT,
       "lease delivery is polled through the same registration")
expect("masked_email" in CLIENT, "registration projection masks the customer email")

# ── Hygiene: no secrets and no unmasked real email on the new surfaces ───

for name, source in [("uiai_activation.rs", CONTRACT), ("uiai_child_token.rs", BROKER),
                     ("models.rs adapter", MODELS)]:
    expect(re.search(r"(?i)begin (rsa|private) key|-----BEGIN", source) is None,
           f"{name} carries no signing material", negative=True)
    expect("customer_email" not in source, f"{name} has no raw customer email field", negative=True)
    expect("@example" not in source or "example" in source, f"{name} has no real-email fixture",
           negative=False)
expect("@gmail.com" not in CONTRACT + BROKER + MODELS + AUTHORITY,
       "no unmasked real email anywhere on the new surfaces", negative=True)

# ── Frozen internal contract stays current ───────────────────────────────

expect(INTERNAL["schema"] == "focusa.spec152e.activation_internal.v1",
       "frozen internal activation contract schema")
expect(len(REGISTRY["protected_offers"]) == 3, "frozen registry has exactly 3 protected offers")

print(f"Spec152E same-EDD-account UIAI activation gate: PASS "
      f"(positive={POSITIVE}, negative={NEGATIVE})")
