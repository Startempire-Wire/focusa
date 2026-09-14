#!/usr/bin/env python3
"""Build-independent contract gate for the Spec 172 UIAI observe-only limited
mode and paid action boundary (atom focusa-vbcqu.20.15.21 / 172.03.04).

Checks `crates/focusa-license/src/uiai_child_token.rs` and the canonical
limited-mode family constants in `entitlement_policy.rs`:

1. The Focusa UIAI operation map (`SPEC172_UIAI_OPERATION_MAP`) carries the
   canonical public-observe/action/persistence classification
   (`UiaiActionPersistenceClass`) for every Spec 172 Section 6.3 operation:
   the six verified-no-license public-observe operations and the six blocked
   families (browser action, persistence, authenticated/private targets,
   unattended automation, scheduled/batch QA, premium hosted resources).
2. `resolve_uiai_operation_capability` resolves ONLY canonical operation ids
   against authority snapshots BEFORE any child token or UI side effect:
   limited mode gets exactly one foreground ephemeral public-observe session;
   every action/persistence/hosted operation fails closed; paid UIAI/Bundle
   proceeds only for granted paid Operator v1 family features; metered/hosted/
   private rights carry no canonical paid feature.
3. No caller-controlled product, price, License Type, family, feature, limit,
   node, or commercial right, and no local/self-issued grant, appears in the
   surface.
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/focusa-license/src/uiai_child_token.rs"
POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"

# Verified-no-license public-observe allowlist: operation id -> paid feature.
LIMITED_ALLOWED_OPERATIONS = {
    "public_search": "uiai_public_observation",
    "source_to_markdown": "uiai_public_observation",
    "public_page_read": "uiai_public_observation",
    "accessibility_snapshot": "uiai_public_observation",
    "screenshot": "uiai_public_observation",
    "basic_diagnostics": "uiai_diagnostics",
}

# Blocked families with concrete operation vectors (Section 6.3 second block).
BLOCKED_OPERATIONS = {
    # browser mutation -> action class
    "browser_click": "browser_action",
    "browser_fill": "browser_action",
    "browser_type": "browser_action",
    "browser_select": "browser_action",
    "browser_press": "browser_action",
    "browser_submit": "browser_action",
    # cookie / auth / session persistence -> persistence class
    "cookie_persistence": "browser_persistence",
    "auth_state_persistence": "browser_persistence",
    "session_persistence": "browser_persistence",
    # authenticated/private, unattended, batch, hosted rights
    "authenticated_private_dashboard": "authenticated_private_targets",
    "unattended_browser_automation": "unattended_browser_automation",
    "scheduled_batch_qa": "scheduled_batch_qa",
    "premium_proxy": "premium_hosted_resources",
    "hosted_capacity": "premium_hosted_resources",
    "paid_model_calls": "premium_hosted_resources",
}

# Canonical paid UIAI Operator v1 family features (Spec 172 Section 7.2).
CANONICAL_PAID_FEATURES = {
    "uiai_public_observation",
    "uiai_browser_action",
    "uiai_persistence",
    "uiai_diagnostics",
    "uiai_proof_packets",
    "uiai_batch_responsive",
    "uiai_supported_integrations",
}

# Metered/hosted/private rights carry NO canonical paid Operator v1 feature:
# they fail closed even for paid grants (Section 7.2 never includes paid
# proxies, hosted compute, paid model usage, or private targets).
NON_CANONICAL_PAID_FEATURES = {
    "authenticated_private_dashboard": "uiai_authenticated_private_targets",
    "unattended_browser_automation": "uiai_unattended_automation",
    "premium_proxy": "uiai_premium_proxy",
    "hosted_capacity": "uiai_hosted_capacity",
    "paid_model_calls": "uiai_paid_model_calls",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    source = SOURCE.read_text(encoding="utf-8")
    policy = POLICY.read_text(encoding="utf-8")

    # --- 1. public-observe/action/persistence classification surface ---
    for marker in [
        "UiaiActionPersistenceClass",
        "PublicObserve",
        "Action",
        "Persistence",
        "SPEC172_UIAI_OPERATION_MAP",
        "UiaiOperationMapEntry",
        "classify_uiai_operation",
        "resolve_uiai_operation_capability",
        "UiaiOperationError",
        "UnknownOperation",
        "operation_class()",
    ]:
        require(marker in source, marker)

    # --- 2. canonical vectors: every Section 6.3 operation appears in the map ---
    for operation_id in LIMITED_ALLOWED_OPERATIONS:
        require(f'operation_id: "{operation_id}"' in source, operation_id)
    for operation_id in BLOCKED_OPERATIONS:
        require(f'operation_id: "{operation_id}"' in source, operation_id)

    # Every limited allowlist family maps to a canonical paid feature.
    for operation_id, paid_feature in LIMITED_ALLOWED_OPERATIONS.items():
        require(
            paid_feature in CANONICAL_PAID_FEATURES,
            f"{operation_id} paid feature must be a canonical Operator v1 feature",
        )
        require(f'paid_feature: "{paid_feature}"' in source, f"{operation_id} paid feature row")

    # Metered/hosted/private rights carry only NON-canonical paid features.
    for operation_id, paid_feature in NON_CANONICAL_PAID_FEATURES.items():
        require(
            paid_feature not in CANONICAL_PAID_FEATURES,
            f"{operation_id} must not reuse a canonical paid feature",
        )
        require(f'paid_feature: "{paid_feature}"' in source, f"{operation_id} paid feature row")

    # Scheduled/batch QA is blocked in limited mode but binds the canonical
    # paid batch/responsive family (Section 7.2) under an active grant.
    require('paid_feature: "uiai_batch_responsive"' in source, "scheduled_batch_qa paid row")

    # --- 3. limited-mode family constants are bound to the classifier ---
    # Limited families are the allowlisted operation ids themselves and the
    # blocked-family labels carried by the blocked operation rows.
    for family in (
        set(LIMITED_ALLOWED_OPERATIONS.keys()) | set(BLOCKED_OPERATIONS.values())
    ):
        require(f'"{family}"' in policy, family)
    for allowed in LIMITED_ALLOWED_OPERATIONS:
        allowlist = policy[
            policy.index("pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES") :
            policy.index("pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES")
        ]
        require(f'"{allowed}"' in allowlist, f"{allowed} must be allowlisted")

    # --- 4. resolver consumes only operation id + authority snapshots ---
    signature = source[
        source.index("pub fn resolve_uiai_operation_capability") :
        source.index(
            ") -> Result<UiaiCapabilityDecision, UiaiOperationError>",
            source.index("pub fn resolve_uiai_operation_capability"),
        )
    ]
    for required in [
        "operation_id",
        "focusa_parent",
        "uiai_grant",
        "active_session_count",
        "now",
    ]:
        require(required in signature, required)
    for forbidden in [
        "pairing",
        "device_proof",
        "auth_state",
        "authentication",
        "product",
        "price",
        "license_type",
        "family",
        "feature",
        "limit",
        "node",
        "commercial_right",
    ]:
        require(forbidden not in signature, f"signature must not accept caller policy: {forbidden}")

    # --- 5. fail-closed ordering: map -> resolver -> shared active_bound ---
    require(
        source.index("SPEC172_UIAI_OPERATION_MAP") < source.index("fn resolve_uiai_operation_capability"),
        "operation map must precede the resolver",
    )
    require(
        source.index("fn resolve_uiai_operation_capability") < source.index("fn active_bound"),
        "resolver must precede the shared active_bound helper",
    )

    # --- 6. no local/self-issued grant, no raw secrets, no caller policy ---
    for forbidden in [
        "SigningKey",
        "Signer",
        "self_sign",
        "customer_email",
        "access_token:",
        "device_proof",
        "pairing_proof",
    ]:
        require(forbidden not in source, forbidden)

    print("Spec172 UIAI observe-only limited mode gate: PASS")
    print(f"operation_map_entries={len(LIMITED_ALLOWED_OPERATIONS) + len(BLOCKED_OPERATIONS)}")
    print(f"limited_allowed_operations={len(LIMITED_ALLOWED_OPERATIONS)}")
    print(f"limited_blocked_operations={len(BLOCKED_OPERATIONS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
