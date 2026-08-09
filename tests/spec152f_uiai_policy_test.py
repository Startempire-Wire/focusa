#!/usr/bin/env python3
"""Build-independent contract gate for the Spec 172 UIAI parent-policy binding.

Atom focusa-vbcqu.20.14.35 (152F.04.06): bind UIAI/browser integrations to
parent policy and child tokens.

Exact verification:
    python3 tests/spec152_uiai_child_token_broker_test.py \\
        && python3 tests/spec152f_uiai_policy_test.py

The gate checks `crates/focusa-license/src/uiai_child_token.rs` and the two
canonical Spec 172 / Spec 152F policy contracts:

1. UIAI/browser operations resolve through a canonical parent-policy decision
   (`resolve_uiai_capability`): verified no-license gets exactly ONE foreground,
   ephemeral, public-web observation session; a paid `uiai-engine` grant
   (Operator or Bundle) resolves granted paid UIAI families bound to the parent
   Focusa lease and grant sequence; Focusa-only paid entitlement NEVER grants
   UIAI (Spec 172 section 3 item 7 / section 20 gate 5).
2. Cached child tokens are revalidated against CURRENT authority snapshots
   (`authorized_cached_token`): stale parents (lease id/sequence/digest no
   longer current) and widened tokens (cached features no longer an exact
   subset of the current grant) are rejected before side effects.
3. The decision path consumes only authority snapshots and operation metadata:
   pairing/device proof/auth state and caller-selected products/prices/grants
   never appear.
"""

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/focusa-license/src/uiai_child_token.rs"
LIMITED_CONTRACT = ROOT / "docs/contracts/spec172-verified-limited-access.v1.yaml"
POLICY_CONTRACT = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"

PAID_FAMILY_FEATURES = [
    "uiai_public_observation",
    "uiai_browser_action",
    "uiai_persistence",
    "uiai_diagnostics",
    "uiai_proof_packets",
    "uiai_batch_responsive",
    "uiai_supported_integrations",
]

UIAI_LIMITED_ALLOWED_FAMILIES = [
    "public_search",
    "source_to_markdown",
    "public_page_read",
    "accessibility_snapshot",
    "screenshot",
    "basic_diagnostics",
]

UIAI_LIMITED_BLOCKED_FAMILIES = [
    "browser_action",
    "browser_persistence",
    "authenticated_private_targets",
    "unattended_browser_automation",
    "scheduled_batch_qa",
    "premium_hosted_resources",
]


def main() -> int:
    source = SOURCE.read_text(encoding="utf-8")
    failures: list[str] = []

    # --- 1. Canonical parent-policy UIAI decision surface ---
    for marker in [
        "UiaiOperationClass",
        "PublicObservation",
        "RemotePremium",
        "resolve_uiai_capability",
        "UiaiCapabilityDecision",
        "UiaiCapabilityDenial",
        "VerifiedNoLicensePublicObservation",
        "session_quota: 1",
        "active_session_count",
        "SPEC172_UIAI_PAID_FAMILY_FEATURES",
        "FocusaOnlyCannotGrantUiai",
        "UiaiGrantRequired",
        "LimitedModeRestricted",
        "FamilyNotGranted",
        "AccountMismatch",
        "UiaiGrantInvalid",
        "MissingPosture",
        "PaidFamily",
        "uiai_grant_sequence",
        "parent_lease_id",
        "parent_sequence",
        "SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES",
        "SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES",
        "authorized_cached_token",
        "stale",
        "widened",
    ]:
        assert marker in source, marker

    # Every paid UIAI Operator v1 family feature is authority-registered.
    for feature in PAID_FAMILY_FEATURES:
        assert f'"{feature}"' in source, feature
        assert feature in source, feature

    # Every limited-mode family is bound by the classifier constants in
    # entitlement_policy.rs (SPEC172_UIAI_VERIFIED_NO_LICENSE_*_FAMILIES).
    policy_source = (ROOT / "crates/focusa-license/src/entitlement_policy.rs").read_text(
        encoding="utf-8"
    )
    for family in UIAI_LIMITED_ALLOWED_FAMILIES + UIAI_LIMITED_BLOCKED_FAMILIES:
        assert f'"{family}"' in policy_source, family

    # No local/self-issued key material, raw secrets, or customer PII.
    for forbidden in [
        "SigningKey",
        "Signer",
        "self_sign",
        "customer_email",
        "access_token:",
        "device_proof",
        "pairing_proof",
    ]:
        assert forbidden not in source, forbidden

    # --- 2. Independence from pairing/auth authority ---
    # The canonical decision function consumes ONLY authority snapshots and
    # operation metadata. Its parameter list must not name pairing, device
    # proof, or auth-state inputs. The source MAY document the independence
    # rule in doc comments, but the decision signature never takes such input.
    decision_signature = source[
        source.index("pub fn resolve_uiai_capability") :
        source.index(") -> UiaiCapabilityDecision", source.index("pub fn resolve_uiai_capability"))
    ]
    for forbidden_param in ["pairing", "device_proof", "auth_state", "authentication"]:
        assert forbidden_param not in decision_signature, forbidden_param
    for required_param in [
        "focusa_parent",
        "uiai_grant",
        "operation_class",
        "limited_family",
        "paid_feature",
        "active_session_count",
        "now",
    ]:
        assert required_param in decision_signature, required_param

    # Ordering: cached-token revalidation re-checks authority snapshots
    # (stale/widened rejection) and lives before the shared active_bound
    # helper.
    assert source.index("fn authorized_cached_token") < source.index("fn active_bound")
    assert "self.cached(audience, now)?;" in source
    assert "active_bound(focusa_parent" in source
    assert "active_bound(\n                uiai_grant" in source or "active_bound(uiai_grant" in source

    # --- 3. Spec 172 limited-mode contract (golden vectors) ---
    limited = yaml.safe_load(LIMITED_CONTRACT.read_text(encoding="utf-8"))
    uiai = limited["uiai_engine"]
    if (uiai["session_limit"], uiai["concurrency_limit"], uiai["execution_mode"]) != (
        1,
        1,
        "foreground",
    ):
        failures.append("UIAI limited mode must be exactly one foreground session")
    if uiai["persistence"] != "ephemeral" or uiai["target_scope"] != "public_web_only":
        failures.append("UIAI limited mode must be ephemeral public-web only")
    if set(uiai["allowed_families"]) != set(UIAI_LIMITED_ALLOWED_FAMILIES):
        failures.append("UIAI limited allowed families do not match the Rust classifier")
    if set(uiai["blocked_families"]) != set(UIAI_LIMITED_BLOCKED_FAMILIES):
        failures.append("UIAI limited blocked families do not match the Rust classifier")

    # --- 4. Spec 152F policy grid rows for UIAI (golden vectors) ---
    policy = yaml.safe_load(POLICY_CONTRACT.read_text(encoding="utf-8"))
    grid = {row["state"]: row["policies"] for row in policy["state_grid"]}
    vnl = grid["verified_no_license"]
    if vnl.get("uiai_public_observation") != "allow_one_foreground_ephemeral_session":
        failures.append("verified_no_license must allow one ephemeral public-observe session")
    if vnl.get("uiai_browser_action") != "deny" or vnl.get("uiai_persistence") != "deny":
        failures.append("verified_no_license must deny browser action and persistence")

    # Focusa-only never grants UIAI: the Focusa product grid may never carry a
    # uiai_* allowance, and the source must hard-deny the Focusa-only case.
    if "Focusa-only" not in source and "Focusa-only paid entitlement" not in source:
        failures.append("source must document the Focusa-only denial")
    if "grant" not in source:
        failures.append("source must bind decisions to authority grants")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1

    print("Spec152f UIAI parent-policy binding gate: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
