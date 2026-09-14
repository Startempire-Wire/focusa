#!/usr/bin/env python3
"""Spec 172 public facade and commerce convergence gate (build-independent).

Proves DONE ONLY WHEN: all registered facades and machine endpoints project
identical current policy; no payment/purchase link uses contradictory authority
or price. The shared public copy/commerce contract
(docs/contracts/spec172-public-facade-convergence.v1.json) is the single
authority for public copy; this gate cross-checks it read-only against the
accepted facade registry, dedicated EDD Operator v1 Downloads, installer route
manifest, install facade routes, product registry, license types, and the
Section 18 public-commerce baseline. Everything is deterministic and offline:
no live network, no authenticated capture, no publication, no cargo build.
"""

import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"

CONVERGENCE = json.loads((CONTRACTS / "spec172-public-facade-convergence.v1.json").read_text(encoding="utf-8"))
FACADE_REGISTRY = json.loads((CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))
EDD_DOWNLOADS = json.loads((CONTRACTS / "spec172-edd-operator-v1-downloads.v1.json").read_text(encoding="utf-8"))
PRODUCT_REGISTRY = json.loads((CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8"))
ROUTE_MANIFEST = json.loads((CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8"))
BASELINE = json.loads((CONTRACTS / "spec172-public-commerce-baseline.v1.json").read_text(encoding="utf-8"))
LICENSE_TYPES = yaml.safe_load((CONTRACTS / "spec172-license-types.v1.yaml").read_text(encoding="utf-8"))
CALL_STACK = yaml.safe_load((CONTRACTS / "spec152e-activation-call-stack.v1.yaml").read_text(encoding="utf-8"))

CONVERGENCE_RAW = (CONTRACTS / "spec172-public-facade-convergence.v1.json").read_text(encoding="utf-8")

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+")
LICENSE_KEY_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")

positive = 0
negative = 0


def check(condition: bool, message: str, kind: str = "positive") -> None:
    global positive, negative
    if not condition:
        raise AssertionError(f"FAIL ({kind}): {message}")
    if kind == "positive":
        positive += 1
    else:
        negative += 1


# ---------------------------------------------------------------------------
# 1. Shared contract is present, current, and authority-frozen.
# ---------------------------------------------------------------------------
check(CONVERGENCE["schema"] == "focusa.spec172.public_facade_convergence.v1", "convergence schema")
check(CONVERGENCE["version"] == 1, "convergence version")
check(CONVERGENCE["owner"] == "WPUIAI/wpuiai + website/install facade owners", "convergence owner")
check(CONVERGENCE["authority"]["canonical_checkout_authority"] == "WPUIAI.com EDD", "checkout authority is WPUIAI EDD")
check(CONVERGENCE["authority"]["stripe_gateway"] == "edd_configured_stripe_gateway_only", "Stripe is EDD-configured only")
check(CONVERGENCE["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facades are presenter-only")
check(CONVERGENCE["authority"]["forbidden_implicit_download"] == 453, "Download 453 never implicitly grants")
check(CONVERGENCE["authority"]["no_anonymous_product_capability"] is True, "no anonymous product capability")
check(CONVERGENCE["authority"]["no_local_or_self_issued_grant"] is True, "no local/self-issued grant")
check(CONVERGENCE["authority"]["no_presenter_owned_policy"] is True, "no presenter-owned policy")
for field in ("edd_download_id", "edd_price_id", "amount_minor", "price", "price_usd",
              "currency", "tier", "product", "product_code", "products", "license_type",
              "license_type_ref", "capability_family", "families", "features", "limits",
              "node_limit", "node_set", "operator_seats", "sale_status", "refund_policy",
              "refund_days", "license_duration", "license_length", "license_unit",
              "activation_limit", "evaluation_duration", "checkout_enabled",
              "future_products_included", "future_license_types_included",
              "redirect_url", "success_url", "cancel_url", "callback_url",
              "sender_email", "authority", "credential", "secret"):
    check(field in CONVERGENCE["authority"]["caller_controls_forbidden"], f"caller cannot control {field}")

# ---------------------------------------------------------------------------
# 2. Canonical policy: Operator names, $697 standalones, $1,254.60 Bundle.
# ---------------------------------------------------------------------------
types = {row["public_code"]: row for row in CONVERGENCE["canonical_policy"]["license_types"]}
check(set(types) == {
    "focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
}, "exactly the three dedicated Operator v1 offers")
check(types["focusa_operator_lifetime_v1"]["human_name"] == "Focusa Operator Lifetime v1", "Operator name published")
check(types["uiai_operator_lifetime_v1"]["human_name"] == "UIAI Engine Operator Lifetime v1", "UIAI Operator name published")
check(types["focusa_uiai_operator_bundle_lifetime_v1"]["human_name"] == "Focusa and UIAI Engine Operator Lifetime Bundle v1", "Bundle name published")
check(types["focusa_operator_lifetime_v1"]["price_usd"] == "697.00" and types["focusa_operator_lifetime_v1"]["amount_minor"] == 69700, "Focusa standalone $697.00")
check(types["uiai_operator_lifetime_v1"]["price_usd"] == "697.00" and types["uiai_operator_lifetime_v1"]["amount_minor"] == 69700, "UIAI standalone $697.00")
bundle = types["focusa_uiai_operator_bundle_lifetime_v1"]
check(bundle["price_usd"] == "1254.60" and bundle["amount_minor"] == 125460, "Bundle exactly $1,254.60")
check(bundle["grant_composition"] == "exact_union", "Bundle is exact grant union")
check(set(bundle["grants"]) == {"focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"}, "Bundle grants the two underlying License Types")
check(bundle["component_refunds_allowed"] is False, "Bundle refunds are whole-order only")
for code, row in types.items():
    check(row["checkout_enabled"] is False, f"{code} checkout stays disabled")
    check(row["sale_status"] == "approved_not_yet_enabled", f"{code} sale status approved-not-enabled")
    check(row["term"] == "lifetime", f"{code} term is lifetime")
    check(row["node_limit"] == 3 and row["node_set"] == "operator_shared_v1", f"{code} three shared nodes")
    check(row["operator_seats"] == 1, f"{code} one operator seat")
    check(row["refund_policy"] == "whole_order_30_days", f"{code} whole-order 30-day refund")

limited = CONVERGENCE["canonical_policy"]["verified_limited_mode"]
check(limited["kind"] == "account_runtime_posture", "verified no-license is a posture, not a License Type")
check(limited["is_license_type"] is False, "verified no-license is not a License Type")
check(limited["creates_edd_key"] is False, "verified no-license never creates an EDD key")
check(limited["expiry"] == "none" and limited["automatic_expiry"] is False, "verified no-license is permanent")
check(limited["grant_source"] == "authority_signed_limited_access_assertion", "limited mode uses signed assertion only")

check(CONVERGENCE["canonical_policy"]["privacy_copy"]["no_telemetry"].startswith(
    "No telemetry; bounded authority communication"), "no-telemetry copy is bounded-authority copy")
check(CONVERGENCE["canonical_policy"]["checkout"]["authority"] == "WPUIAI.com EDD", "checkout truth is EDD")
check(CONVERGENCE["canonical_policy"]["checkout"]["facade_collects_card_data"] is False, "facades never collect card data")
check(CONVERGENCE["canonical_policy"]["checkout"]["direct_gravity_or_stripe_entitlement"] is False, "no direct Gravity/Stripe entitlement")
check(CONVERGENCE["canonical_policy"]["product_isolation"]["focusa_purchase_grants_uiai"] is False, "no implicit UIAI from Focusa purchase")
check("uiai_operator_lifetime_v1" in CONVERGENCE["canonical_policy"]["product_isolation"]["uiai_requires"], "UIAI requires its own License Type or Bundle")
check(CONVERGENCE["canonical_policy"]["future_boundaries"]["new_family_default"] == "deny_pending_explicit_assignment", "new families default to denial")
check(CONVERGENCE["canonical_policy"]["future_boundaries"]["new_product_default"] == "excluded", "new products excluded by default")
check(CONVERGENCE["canonical_policy"]["future_boundaries"]["operator_name_preserved"] is True, "Operator naming preserved")
for op in ("basic_customer_data_export", "account_control", "device_control", "license_status",
           "diagnostics", "repair", "rollback", "stable_security_update", "uninstall",
           "emergency_customer_data_recovery"):
    check(op in CONVERGENCE["canonical_policy"]["preserved_always_reachable"], f"{op} stays reachable")

# ---------------------------------------------------------------------------
# 3. Every registered facade projects identical current policy.
# ---------------------------------------------------------------------------
check(CONVERGENCE["facade_projection"]["identical_current_policy"] is True, "facades project identical policy")
projection_keys = set(CONVERGENCE["facade_projection"]["projection_keys"])
check(projection_keys == {
    "license_types", "verified_limited_mode", "privacy_copy", "checkout",
    "product_isolation", "future_boundaries", "preserved_always_reachable",
}, "projection keys are exactly the canonical policy sections")

registered = {row["facade_id"]: row for row in FACADE_REGISTRY["facades"]}
converged = {row["facade_id"]: row for row in CONVERGENCE["facade_projection"]["facades"]}
check(set(registered) == set(converged), "convergence covers every registered facade")
for facade_id, reg in registered.items():
    conv = converged[facade_id]
    check(conv["origin"] == reg["exact_origins"][0], f"{facade_id} origin matches registry")
    check(reg["exact_origins"] == [conv["origin"]], f"{facade_id} has one exact origin")
    check(conv["products"] == reg["products"], f"{facade_id} product allowlist matches registry")
    for key in ("checkout", "verification", "manage", "recovery", "success", "cancel"):
        check(conv["paths"][key] == reg["paths"].get(key, conv["paths"][key]), f"{facade_id} path {key} matches registry")
    check(conv["paths"]["checkout"] == "/activate/checkout", f"{facade_id} checkout path is the canonical path")
    check(reg["status"] == "registered_presenter", f"{facade_id} is a registered presenter")

# All facades share identical checkout/verify/manage/recovery paths.
check({row["paths"]["checkout"] for row in converged.values()} == {"/activate/checkout"}, "all checkout paths identical")
check({row["paths"]["verification"] for row in converged.values()} == {"/activate/verify"}, "all verification paths identical")
check({row["paths"]["manage"] for row in converged.values()} == {"/account"}, "all manage paths identical")
check({row["paths"]["recovery"] for row in converged.values()} == {"/activate/recovery"}, "all recovery paths identical")

# Product isolation: Focusa-only facades never project the standalone UIAI type.
focusa_only = {"focusa_marketing_v1", "focusa_forge_v1", "focusa_arena_v1"}
for facade_id in focusa_only:
    check("uiai_operator_lifetime_v1" not in converged[facade_id]["products"], f"{facade_id} has no standalone UIAI offer")
    check("focusa_uiai_operator_bundle_lifetime_v1" in converged[facade_id]["products"], f"{facade_id} may project the Bundle")
check("focusa_operator_lifetime_v1" not in converged["uiai_engine_v1"]["products"], "engine facade has no Focusa standalone offer")

# ---------------------------------------------------------------------------
# 4. EDD checkout links: dedicated Downloads, no contradictory price anywhere.
# ---------------------------------------------------------------------------
edd = EDD_DOWNLOADS
check(edd["schema"] == "focusa.spec172.edd_operator_v1_downloads.v1", "EDD downloads contract schema")
check(edd["counts"]["assigned_edd_downloads"] == 3, "exactly three dedicated EDD Downloads")
check(edd["counts"]["checkout_enabled"] == 0, "no checkout enabled yet")
check(edd["counts"]["sum_amount_minor"] == 264860, "dedicated minor-unit sum is 69700+69700+125460")
check(set(CONVERGENCE["edd_checkout_links"]["dedicated_edd_download_ids"]) == {458, 459, 460}, "dedicated EDD download ids are 458/459/460")
check(CONVERGENCE["edd_checkout_links"]["proxy_route"] == "/v1/activation/checkout", "checkout proxy route is canonical")
check(CONVERGENCE["edd_checkout_links"]["checkout_enabled"] is False, "checkout stays disabled awaiting validation")
check(CONVERGENCE["edd_checkout_links"]["legacy_downloads_never_grant"] is True, "legacy downloads never grant Operator")
check(CONVERGENCE["edd_checkout_links"]["no_contradictory_price_in_any_purchase_link"] is True, "no contradictory price in purchase links")

# Every dedicated record matches the canonical policy price and never reuses legacy ids.
by_public_code = {row["public_code"]: row for row in edd["records"]}
check(set(by_public_code) == set(types), "dedicated records match the canonical offer set")
for code, record in by_public_code.items():
    check(record["amount_minor"] == types[code]["amount_minor"], f"{code} minor units match canonical policy")
    check(record["price_usd"] == types[code]["price_usd"], f"{code} USD string matches canonical policy")
    check(record["checkout_enabled"] is False and record["sale_status"] == "approved_not_yet_enabled", f"{code} not purchasable")
    check(record["edd_download_id"] not in edd["authority"]["legacy_download_ids"], f"{code} never reuses a legacy download")
    check(record["edd_download_id"] != 453, f"{code} never uses Download 453")

# Registry offers agree too.
protected = {row["public_code"]: row for row in PRODUCT_REGISTRY["protected_offers"]}
check(set(protected) == set(types), "product registry offers match canonical set")
for code, row in protected.items():
    check(row["price_usd"] == types[code]["price_usd"], f"{code} registry price matches")
    check(row["checkout_enabled"] is False, f"{code} registry checkout disabled")
    check(row["price_authority"] == "spec172_server_owned", f"{code} price is server-owned")

# License Types YAML agrees.
lt = {row["code"]: row for row in LICENSE_TYPES["license_types"]}
lt_sku = {row["code"]: row for row in LICENSE_TYPES["composite_skus"]}
check(lt["focusa_operator_lifetime_v1"]["price_usd"] == "697.00", "license-types Focusa $697.00")
check(lt["uiai_operator_lifetime_v1"]["price_usd"] == "697.00", "license-types UIAI $697.00")
check(lt_sku["focusa_uiai_operator_bundle_lifetime_v1"]["price_usd"] == "1254.60", "license-types Bundle $1,254.60")
check(lt_sku["focusa_uiai_operator_bundle_lifetime_v1"]["standalone_sum_usd"] == "1394.00", "Bundle 10% below $1,394.00 sum")
check(lt_sku["focusa_uiai_operator_bundle_lifetime_v1"]["discount_basis_points"] == 1000, "Bundle discount is 1000 basis points")
check(lt_sku["focusa_uiai_operator_bundle_lifetime_v1"]["independent_feature_catalog"] is False, "Bundle has no third feature catalog")

# Checkout operation is a facade proxy to the EDD authority.
checkout_op = next(op for op in CALL_STACK["operations"] if op["id"] == "activation.checkout")
check(checkout_op["path"] == "/v1/activation/checkout", "call-stack checkout path is the proxy route")
check("EddCheckoutAdapter" in checkout_op["services"], "checkout routes through the EDD adapter")
check(CONVERGENCE["edd_checkout_links"]["proxy_route"] == checkout_op["path"], "convergence proxy route matches call stack")

# ---------------------------------------------------------------------------
# 5. Route/link fixtures: convenience URLs and transactional links resolve.
# ---------------------------------------------------------------------------
check(ROUTE_MANIFEST["origin"] == "https://install.focusa.dev", "route manifest origin is install")
check({row["route"] for row in ROUTE_MANIFEST["convenience_urls"]} == {
    "/focusa", "/bundle", "/engine", "/powershell",
}, "convenience URLs are the repaired /focusa /bundle /engine /powershell")
for row in ROUTE_MANIFEST["convenience_urls"]:
    check(row["status"] == 200, f"{row['route']} resolves 200 (no 404)")
    check(row["target"].startswith("/installers/"), f"{row['route']} targets a verified installer asset")
check(CONVERGENCE["route_link_fixtures"]["install_convenience_routes"]["routes"] == [
    "/focusa", "/bundle", "/engine", "/powershell",
], "convergence convenience routes match the repaired manifest")
check(CONVERGENCE["route_link_fixtures"]["install_convenience_routes"]["all_resolve_200"] is True, "all convenience routes resolve")
check(set(CONVERGENCE["route_link_fixtures"]["transactional_links"]) == set(ROUTE_MANIFEST["transactional_links"]), "transactional links match manifest")
for name, link in ROUTE_MANIFEST["transactional_links"].items():
    check(link["status"] == 200, f"transactional link {name} resolves 200")
check(CONVERGENCE["route_link_fixtures"]["no_broken_license_or_commerce_links"] is True, "no broken license/commerce links")
check(CONVERGENCE["route_link_fixtures"]["license_terms_link_status"] == "valid_or_removed_before_publication", "license terms links valid or removed")
install_routes_php = (CONTRACTS / "spec152e-install-facade-routes.v1.php").read_text(encoding="utf-8")
check("spec152e.install_facade_routes.v1" in install_routes_php, "install facade routes contract present")
check("https://install.focusa.dev" in install_routes_php, "install facade routes bound to exact origin")
check("WPUIAI.com EDD" in install_routes_php, "install facade routes defer to EDD authority")

# Machine commerce endpoints for focusa and engine facades are declared.
machine_surfaces = {row["facade_id"]: row for row in CONVERGENCE["facade_projection"]["facades"]}
for facade_id in ("focusa_marketing_v1", "uiai_engine_v1"):
    mc = machine_surfaces[facade_id]["machine_commerce"]
    check("/pricing/" in mc and "/llms.txt" in mc and "/.well-known/agent-commerce.json" in mc, f"{facade_id} machine commerce endpoints declared")

# ---------------------------------------------------------------------------
# 6. Section 18.2 contradictions removed; replacements frozen.
# ---------------------------------------------------------------------------
replaced = {row["code"]: row["replacement"] for row in CONVERGENCE["removed_contradictions"]}
check(len(replaced) == 10, "all ten Section 18.2 contradictions addressed")
expected_replacements = {
    row["current"]: row["required"] for row in BASELINE["required_replacements"]
}
check(replaced["anonymous_local_evaluation"] == expected_replacements["Anonymous/no-account Evaluation"], "anonymous eval removed")
check(replaced["local_self_issued_eval"] == expected_replacements["Local self-issued --eval"], "local self-issued eval removed")
check(replaced["timed_evaluation"] == expected_replacements["Timed Evaluation requirement"], "timed Evaluation removed")
check(replaced["conflicting_bundle_1097"] == expected_replacements["Bundle advertised at $1,097"], "$1,097 replaced by $1,254.60")
check(replaced["implicit_uiai_grant"] == expected_replacements["Focusa purchase may implicitly grant UIAI"], "implicit UIAI removed")
check(replaced["gravity_or_direct_stripe_entitlement"] == expected_replacements["Gravity Forms/direct Stripe creates entitlement"], "Gravity/direct Stripe removed")
check(replaced["no_phone_home"] == expected_replacements["No phone home"], "no-phone-home replaced")
check(replaced["broken_convenience_routes"] == expected_replacements["/focusa and /bundle commands advertised while routes return 404"], "broken convenience routes repaired")
check(replaced["broken_license_links"] == expected_replacements["Public LICENSE or COMMERCIAL.md links return 404"], "broken license links repaired")
check(replaced["legacy_wpuiai_prices"] == expected_replacements["Old WPUIAI $29/$99/$299/$149 offers appear commercially related"], "legacy WPUIAI catalog separated")

# Forbidden claims must be absent from the shared public copy contract.
for forbidden in ("$1,097", "1,097", "anonymous evaluation", "no phone home", "Gravity Forms/direct Stripe creates entitlement"):
    check(forbidden.lower() not in CONVERGENCE_RAW.lower(), f"forbidden public claim absent: {forbidden}")

# ---------------------------------------------------------------------------
# 7. UIAI browser proof plan is bounded and fail-closed.
# ---------------------------------------------------------------------------
plan = CONVERGENCE["uiai_browser_proof_plan"]
check(plan["owner"] == "focusa-vbcqu.20.15.39 (Capture UIAI public pricing, route, and contradiction-removal proof)", "proof plan owned by the downstream capture atom")
check(plan["method"] == "uiai_engine_browser_read", "proof method is UIAI browser read")
check(plan["authenticated_capture"] is False, "no authenticated capture in proof plan")
check(plan["anonymous_eval_or_local_grant_proof"] == "none", "no anonymous/local eval proof step")
check(plan["evidence_posture"]["raw_capture_embedded"] is False, "no raw capture embedded")
check(plan["evidence_posture"]["raw_email_or_key_embedded"] is False, "no raw email/key embedded")
check(plan["evidence_posture"]["authenticated_or_customer_data_capture"] is False, "no customer-data capture")
for claim in ("Operator names published", "standalone price 697.00", "bundle price 1254.60 and no 1097",
              "verified limited mode and no anonymous eval", "no telemetry bounded authority copy",
              "EDD-backed checkout and no direct Gravity/Stripe entitlement",
              "product isolation no implicit UIAI", "valid license and install routes no 404"):
    check(claim in plan["claims_to_verify"], f"proof plan verifies: {claim}")

# ---------------------------------------------------------------------------
# 8. Hygiene: no raw email, key, token, customer row, credential, or card data.
# ---------------------------------------------------------------------------
for forbidden in ("customer_email", "license_key", "access_token", "cookie", "card_number",
                  "cvv", "stripe_payment_intent", "customer_id"):
    check(forbidden not in CONVERGENCE_RAW, f"no {forbidden} in shared public copy")
check(EMAIL_RE.search(CONVERGENCE_RAW) is None, "no email addresses in shared public copy", "negative")
check(SECRET_RE.search(CONVERGENCE_RAW) is None, "no secret-shaped values in shared public copy", "negative")
check(LICENSE_KEY_RE.search(CONVERGENCE_RAW) is None, "no license-shaped evidence in shared public copy", "negative")
check("*" not in CONVERGENCE_RAW, "no wildcard authority in shared public copy", "negative")

# ---------------------------------------------------------------------------
# 9. Fail-closed validator: a facade cannot project a contradictory policy.
# ---------------------------------------------------------------------------
def validate_convergence(candidate: dict) -> None:
    """Accept only a fully converged contract (used for negative mutation checks)."""
    assert candidate["facade_projection"]["identical_current_policy"] is True
    for row in candidate["facade_projection"]["facades"]:
        assert row["paths"]["checkout"] == "/activate/checkout"
        assert row["origin"].startswith("https://")
    prices = {r["public_code"]: (r["price_usd"], r["amount_minor"]) for r in candidate["canonical_policy"]["license_types"]}
    assert prices["focusa_operator_lifetime_v1"] == ("697.00", 69700)
    assert prices["uiai_operator_lifetime_v1"] == ("697.00", 69700)
    assert prices["focusa_uiai_operator_bundle_lifetime_v1"] == ("1254.60", 125460)
    for row in candidate["canonical_policy"]["license_types"]:
        assert row["checkout_enabled"] is False
        assert row["sale_status"] == "approved_not_yet_enabled"
    bundle = next(r for r in candidate["canonical_policy"]["license_types"]
                  if r["public_code"] == "focusa_uiai_operator_bundle_lifetime_v1")
    assert bundle["price_usd"] == "1254.60" and bundle["amount_minor"] == 125460
    assert candidate["canonical_policy"]["product_isolation"]["focusa_purchase_grants_uiai"] is False
    assert candidate["canonical_policy"]["checkout"]["direct_gravity_or_stripe_entitlement"] is False
    assert candidate["canonical_policy"]["verified_limited_mode"]["creates_edd_key"] is False
    assert candidate["canonical_policy"]["verified_limited_mode"]["automatic_expiry"] is False


def denied(mutator, message):
    candidate = json.loads(CONVERGENCE_RAW)
    mutator(candidate)
    try:
        validate_convergence(candidate)
    except (AssertionError, KeyError, TypeError):
        return
    raise AssertionError(message)


denied(lambda c: c["canonical_policy"]["license_types"][0].update({"price_usd": "1097.00", "amount_minor": 109700}),
       "contradictory $1,097 price accepted")
denied(lambda c: c["canonical_policy"]["license_types"][0].update({"checkout_enabled": True}),
       "checkout enabled before validation accepted")
denied(lambda c: c["facade_projection"]["facades"][0]["paths"].update({"checkout": "/stripe/checkout"}),
       "direct Stripe checkout path accepted")
denied(lambda c: c["canonical_policy"]["product_isolation"].update({"focusa_purchase_grants_uiai": True}),
       "implicit UIAI grant accepted")
denied(lambda c: c["canonical_policy"]["checkout"].update({"direct_gravity_or_stripe_entitlement": True}),
       "direct Gravity/Stripe entitlement accepted")
denied(lambda c: c["facade_projection"].update({"identical_current_policy": False}),
       "non-identical facade projection accepted")
denied(lambda c: c["canonical_policy"]["verified_limited_mode"].update({"creates_edd_key": True}),
       "limited mode creating EDD key accepted")
denied(lambda c: c["canonical_policy"]["verified_limited_mode"].update({"automatic_expiry": True}),
       "timed limited mode accepted")

result = {
    "schema": "focusa.spec172.public_facade_convergence_validation.v1",
    "registered_facades": len(converged),
    "dedicated_edd_downloads": edd["counts"]["assigned_edd_downloads"],
    "canonical_offers": len(types),
    "removed_contradictions": len(replaced),
    "positive_checks": positive,
    "negative_checks": negative,
    "result": "passed_fail_closed",
}
print(json.dumps(result, sort_keys=True))
print("Spec 172 public facade and commerce convergence: PASS")
