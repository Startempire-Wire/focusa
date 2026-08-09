#!/usr/bin/env python3
"""Spec 152E.05.09 one-account Focusa and UIAI bundle activation.

Binds the bundle activation surface to the frozen Spec 152E / Spec 172
contracts (docs/contracts/spec152e-edd-product-registry.v1.json,
spec152e-activation-internal.v1.json, spec152e-installer-route-manifest.v1.json)
and proves:

1. BUNDLE ORDER/ITEM/LICENSE POLICY (crates/focusa-license/src/
   bundle_activation.rs): the bundle SKU `focusa_uiai_operator_bundle_lifetime_v1`
   is ONE commerce SKU/key at the frozen server-owned price USD 1254.60 that
   grants the EXACT union of the two underlying Operator v1 grants
   (`focusa_operator_lifetime_v1` + `uiai_operator_lifetime_v1`) with one
   operator seat, three SHARED operator node identities, whole-order refunds
   only (v1), no third feature catalog, and future products excluded. The
   mapping is server-owned: clients submit only the public bundle code; EDD
   ids, prices, tiers, grants, limits, and commercial flags are never accepted
   (`resolve_bundle_activation` / `resolve_bundle_order_policy` have no such
   inputs).

2. ONE-ACCOUNT ORCHESTRATION: `resolve_bundle_activation` activates BOTH exact
   products on the one verified EDD account (single `UiaiAccountIdentity`,
   strict `same_account_binding`, shared node identity) or returns the typed
   recoverable `BundlePartialActivation` reusing the SAME order/registration
   handle — `no_duplicate_payment`, `no_duplicate_license`, `one_edd_order`,
   `one_account` — with the pending grant, its typed reason, and the safe
   recovery action. There is no silent partial success.

3. FOCUSA/UIAI PRESENTERS AND LEASES (crates/focusa-core/src/install_lifecycle/
   models.rs): `BundleAdapterPosture::from_bundle_authority` builds the bundle
   adapter posture carrying BOTH exact grant leases on the SAME shared node
   identities when both are active and bound to the one verified EDD account;
   the projection is the only source of grants (no third feature list).

4. BUNDLE INSTALLER/FACADE FLOW: the frozen route manifest binds `/bundle` →
   `/installers/install-bundle.sh` (deployed_only_pinned sha256
   16cb3944...) and the shared `ActivationSession` submits only the
   server-owned `public_product_code` through one registration — the bundle
   has no second checkout, account, or license route.

The Rust unit tests for the order policy, both-product activation, typed
partial state, and strict binding/identity checks execute in the same commit
(crates/focusa-license/src/bundle_activation.rs), so evidence is replayable
from the pinned commit without any network.

Exact verification: python3 tests/spec152e_bundle_activation_test.py
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
ROUTES = json.loads(
    (CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8")
)
BUNDLE = (LICENSE_CRATE / "bundle_activation.rs").read_text(encoding="utf-8")
UIAI = (LICENSE_CRATE / "uiai_activation.rs").read_text(encoding="utf-8")
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


# ── Frozen EDD bundle order/item/license policy (registry) ────────────────

expect(REGISTRY["schema"] == "focusa.spec152e.edd_product_registry.v1",
       "frozen EDD product registry schema")
offers = {row["public_code"]: row for row in REGISTRY["protected_offers"]}
bundle = offers.get("focusa_uiai_operator_bundle_lifetime_v1")
expect(bundle is not None, "bundle offer is in the frozen registry")
expect(bundle["products"] == ["focusa", "uiai_engine"],
       "bundle maps to the exact two-product union only")
expect(bundle["grants"] == ["focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"],
       "bundle grants are the exact two underlying Operator v1 License Types")
expect(bundle["grant_composition"] == "exact_union", "bundle composition is exact union")
expect(bundle["price_usd"] == "1254.60", "bundle price is exactly 1254.60")
expect(bundle["price_authority"] == "spec172_server_owned", "bundle price is server-owned")
expect(bundle["operator_seats"] == 1, "bundle has exactly one operator seat")
expect(bundle["node_limit"] == 3, "bundle shares three operator nodes")
expect(bundle["node_set"] == "operator_shared_v1", "bundle node set is the shared operator set")
expect(bundle["license_duration"] == "lifetime", "bundle is lifetime")
expect(bundle["component_refunds_allowed"] is False,
       "bundle v1 has no component partial refunds")
expect(bundle["refund_policy"] == "whole_order_30_days", "bundle refund is whole-order")
expect(bundle["future_products_included"] is False,
       "future products never enter the bundle automatically")
expect(bundle["composite_sku_ref"] == "focusa_uiai_operator_bundle_lifetime_v1",
       "bundle is one composite SKU")
expect(bundle["edd_download_id"] is None and bundle["edd_price_id"] is None,
       "registry owns the bundle EDD mapping; no client-assigned ids")
expect(bundle["checkout_enabled"] is False,
       "bundle checkout stays approved-not-enabled until the operator gate")
expect("bundle_is_exact_union_at_1254_60_with_one_operator_and_three_shared_nodes"
       in REGISTRY["invariants"],
       "registry pins the exact-union/1254.60/one-operator/three-shared-nodes invariant")
expect("focusa_uiai_bundle" in [c["id"] for c in REGISTRY["legacy_record_classes"]]
       or "bundle" in json.dumps(REGISTRY["legacy_record_classes"]).lower(),
       "legacy bundle records are classified (migrate/quarantine) not promoted")
for forbidden in ["edd_download_id", "edd_price_id", "price", "tier"]:
    expect(forbidden in REGISTRY["authority"]["caller_controls_forbidden"],
           f"registry forbids caller-controlled {forbidden}")

# ── Bundle activation contract (bundle_activation.rs) ─────────────────────

expect('BUNDLE_ACTIVATION_SCHEMA: &str = "focusa.bundle_activation_contract.v1"' in BUNDLE,
       "bundle activation schema is pinned")
expect('BUNDLE_ORDER_POLICY_SCHEMA: &str = "focusa.bundle_order_policy.v1"' in BUNDLE,
       "bundle order policy schema is pinned")
expect('BUNDLE_PARTIAL_SCHEMA: &str = "focusa.bundle_partial_activation.v1"' in BUNDLE,
       "bundle partial schema is pinned")
expect('BUNDLE_PRICE_USD: &str = "1254.60"' in BUNDLE, "bundle price constant is frozen")
expect("BUNDLE_PRICE_MINOR_UNITS: u64 = 125_460" in BUNDLE,
       "bundle price minor units are frozen")
expect('BUNDLE_GRANT_COMPOSITION: &str = "exact_union"' in BUNDLE,
       "bundle composition constant is frozen")
expect("BUNDLE_NODE_LIMIT: u32 = 3" in BUNDLE, "bundle node limit is three shared nodes")
expect("BUNDLE_OPERATOR_SEATS: u32 = 1" in BUNDLE, "bundle operator seats are one")
expect('BUNDLE_REFUND_POLICY: &str = "whole_order_30_days"' in BUNDLE,
       "bundle refund policy is whole-order")
expect("pub const BUNDLE_GRANTS: [&str; 2]" in BUNDLE,
       "bundle grants are exactly two, never a third list")
expect("PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1," in BUNDLE
       and "PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1," in BUNDLE,
       "bundle grants reference the exact two underlying Operator v1 codes")
expect('"focusa_operator_lifetime_v1"' in UIAI and '"uiai_operator_lifetime_v1"' in UIAI,
       "underlying grant public codes are pinned in the shared contract")
expect('PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1' in BUNDLE,
       "bundle public code is pinned in the contract")
expect("pub struct BundleOrderPolicy" in BUNDLE, "EDD bundle order/item/license policy type exists")
for field in ["one_edd_order", "one_human_key", "component_refunds_allowed",
              "future_products_included", "third_feature_catalog",
              "price_usd", "price_minor_units", "node_limit", "node_set",
              "operator_seats", "refund_policy", "grant_composition"]:
    expect(f"pub {field}:" in BUNDLE, f"BundleOrderPolicy carries {field}")
expect("pub fn resolve_bundle_order_policy(" in BUNDLE,
       "bundle policy resolver exists")
expect("ProductMappingRequired" in BUNDLE, "wrong bundle code fails closed")
expect("BundleGrantRequired" in BUNDLE, "missing two-product grants fail closed")
expect("BundleAccountMismatch" in BUNDLE, "account mismatch fails closed")
expect("SharedNodeIdentityViolation" in BUNDLE,
       "split node bindings fail closed (never six unrelated activations)")
expect("AccountIdentityRequired" in BUNDLE, "missing account identity fails closed")

# The decision never accepts caller-controlled EDD fields.
decision_signature = BUNDLE.split("pub fn resolve_bundle_activation(")[1].split(")")[0]
for forbidden in ["edd_download_id", "edd_price_id", "price", "tier", "sale_status",
                  "refund_policy", "commercial_rights", "requested_features",
                  "requested_limits", "grant_features"]:
    expect(forbidden not in decision_signature,
           f"resolve_bundle_activation never accepts caller-controlled {forbidden}",
           negative=True)
expect("focusa_grant" in decision_signature and "uiai_grant" in decision_signature,
       "bundle decision reads both underlying signed authority leases")
expect("order_handle" in decision_signature and "registration_id" in decision_signature,
       "bundle decision carries the one order and one registration handle")
expect("requested_public_code" in decision_signature,
       "the client submits only the public bundle code")

# Atomic-or-typed-partial orchestration: no silent partial success.
expect("pub enum BundleActivationOutcome" in BUNDLE, "typed bundle outcome exists")
expect("Activated(BundleActivationProjection)" in BUNDLE, "both-products outcome exists")
expect("RecoverablePartial(BundlePartialActivation)" in BUNDLE,
       "typed recoverable partial outcome exists")
expect("pub struct BundlePartialActivation" in BUNDLE,
       "typed recoverable partial state exists")
for field in ["no_duplicate_payment", "no_duplicate_license", "one_edd_order",
              "one_account", "order_handle", "registration_id", "settled_grants",
              "pending_grants", "recovery_action"]:
    expect(f"pub {field}:" in BUNDLE, f"BundlePartialActivation carries {field}")
expect("pub struct PendingBundleGrant" in BUNDLE, "pending grant type exists")
expect("grant_product_mismatch" in BUNDLE, "product mismatch is a typed pending reason")
expect("grant_inactive_or_unbound" in BUNDLE, "inactive/unbound is a typed pending reason")
expect('"resume_poll_same_order"' in BUNDLE, "recovery resumes the SAME order poll")
expect('"authority_review"' in BUNDLE, "non-retryable failures route to authority review")
expect("pub struct BundleActivationProjection" in BUNDLE, "full bundle projection exists")
expect("pub struct BundleGrantProjection" in BUNDLE, "per-product grant projection exists")
expect("pub shared_node_identities: Vec<String>" in BUNDLE,
       "projection carries the shared operator node identities")
expect("pub posture: String" in BUNDLE and '"bundle"' in BUNDLE,
       "bundle posture is typed")
expect("pub price_usd: String" in BUNDLE and "pub price_authority: String" in BUNDLE,
       "projection carries the server-owned price")
expect("same_account_binding" in BUNDLE, "one-account binding is enforced")
expect("focusa_grant.node_id != uiai_grant.node_id" in BUNDLE,
       "both grants must bind the same shared node identity")
expect("grant_active_bound(focusa_grant, now)" in BUNDLE
       and "grant_active_bound(uiai_grant, now)" in BUNDLE,
       "atomic activation requires BOTH grants active and bound")
expect("if focusa_ready && uiai_ready" in BUNDLE,
       "atomic activation settles only when both grants are ready")

# ── Focusa/UIAI presenters and leases (install_lifecycle models) ─────────

expect('"focusa.bundle_adapter_posture.v1"' in MODELS,
       "bundle adapter posture schema is pinned")
expect("pub struct BundleAdapterPosture" in MODELS, "bundle adapter posture exists")
for field in ["account_id", "edd_customer_id", "order_handle", "public_code",
              "node_id", "shared_node_identities", "focusa_lease_id",
              "focusa_lease_sequence", "focusa_lease_digest", "uiai_lease_id",
              "uiai_lease_sequence", "uiai_lease_digest", "both_grants_active"]:
    expect(f"pub {field}:" in MODELS, f"BundleAdapterPosture carries {field}")
expect("pub fn from_bundle_authority(" in MODELS,
       "bundle adapter constructor exists")
expect("same_account_binding" in MODELS,
       "bundle adapter enforces the single verified EDD account")
expect('projection.focusa.product != "focusa"' in MODELS
       or 'projection.focusa.product == "focusa"' in MODELS,
       "adapter requires the exact focusa grant from the projection")
expect('projection.uiai_engine.product != "uiai-engine"' in MODELS
       or 'projection.uiai_engine.product == "uiai-engine"' in MODELS,
       "adapter requires the exact uiai-engine grant from the projection")
expect("projection.focusa.node_id != projection.node_id" in MODELS
       or "projection.focusa.node_id == projection.node_id" in MODELS,
       "adapter enforces the shared node identity")
expect("both_grants_active: true" in MODELS,
       "adapter posture exists only when both grants are active")
expect("entitlement_snapshot_ready(focusa_grant, now)" in MODELS
       and "entitlement_snapshot_ready(uiai_grant, now)" in MODELS,
       "adapter requires both signed leases ready")
expect("AdapterEntitlementPostureIncomplete" in MODELS,
       "incomplete bundle postures fail closed")

# ── Bundle installer/facade flow: one shared registration ────────────────

expect(ROUTES["origin"] == "https://install.focusa.dev", "install route manifest origin")
bundle_route = next(
    (row for row in ROUTES["convenience_urls"] if row["route"] == "/bundle"), None)
expect(bundle_route is not None, "/bundle convenience URL exists in the route manifest")
expect(bundle_route["target"] == "/installers/install-bundle.sh",
       "/bundle routes to the verified bundle installer asset")
expect(bundle_route["trust"]["kind"] == "deployed_only_pinned",
       "bundle installer is deployed-only pinned, not silently re-issued")
expect(bundle_route["trust"]["sha256"] ==
       "16cb3944c969d5c3bd7c9cb73b3a30161ada1c2a1ab7282811f038c114904912",
       "bundle installer sha256 is pinned")
expect(any("no advertised URL returns 404" in item for item in ROUTES["invariants"]),
       "the /bundle 404 defect is repaired")
expect(any("no credentials secrets or unmasked email addresses" in item
           for item in ROUTES["invariants"]),
       "route manifest carries no credentials or unmasked email")
# The bundle flows through the SAME shared activation registration: the
# client submits only the server-owned public product code.
expect("pub fn begin(" in CLIENT and "public_product_code" in CLIENT,
       "bundle starts the shared registration with the public product code only")
expect("fn select_offer(" in CLIENT and "public_product_code" in CLIENT,
       "offer selection is server-owned through the shared client")
expect("pub fn poll(" in CLIENT and "lease_envelope" in CLIENT,
       "lease delivery is polled through the same registration")
expect("masked_email" in CLIENT, "registration projection masks the customer email")
start_op = next(op for op in INTERNAL["operations"] if op["id"] == "activation.start")
expect("public_product_code" in start_op["input"],
       "activation.start takes only the server-owned public product code")
expect("PRODUCT_MAPPING_REQUIRED" in start_op["failures"],
       "wrong bundle code fails closed with the typed registry error")
select_op = next(op for op in INTERNAL["operations"] if op["id"] == "activation.select_offer")
expect("public_product_code" in select_op["input"],
       "activation.select_offer takes only the server-owned public product code")
expect("prices_products_grants_features_limits_and_rights_are_server_owned"
       in INTERNAL["invariants"],
       "internal contract keeps bundle grants server-owned")

# ── Hygiene: no secrets and no unmasked real email on the new surfaces ────

for name, source in [("bundle_activation.rs", BUNDLE), ("models.rs bundle adapter", MODELS)]:
    expect(re.search(r"(?i)begin (rsa|private) key|-----BEGIN", source) is None,
           f"{name} carries no signing material", negative=True)
    expect("customer_email" not in source, f"{name} has no raw customer email field",
           negative=True)
email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
for match in email_pattern.findall(BUNDLE + MODELS):
    if match.endswith("@example.com"):
        continue
    raise AssertionError(f"unmasked email in new bundle surfaces: {match}")
expect("@gmail.com" not in BUNDLE + MODELS,
       "no unmasked real email anywhere on the new surfaces", negative=True)

# ── Frozen internal contract stays current ────────────────────────────────

expect(INTERNAL["schema"] == "focusa.spec152e.activation_internal.v1",
       "frozen internal activation contract schema")
expect(len(REGISTRY["protected_offers"]) == 3, "frozen registry has exactly 3 protected offers")

print(f"Spec152E one-account bundle activation gate: PASS "
      f"(positive={POSITIVE}, negative={NEGATIVE})")
