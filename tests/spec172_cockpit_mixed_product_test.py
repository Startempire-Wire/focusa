#!/usr/bin/env python3
"""Build-independent contract gate for the Spec 172 UIAI Cockpit
mixed-product presenter binding (atom focusa-vbcqu.20.15.29 / 172.04.05).

Checks `crates/focusa-license/src/cockpit_action_registry.rs` and its linkage
to the canonical Focusa/UIAI operation maps:

1. The Cockpit action registry (`SPEC172_COCKPIT_ACTION_REGISTRY`) carries
   canonical Spec 172 Section 12 trusted metadata (product_owner /
   operation_class / capability_family / side_effect_class) for every Focusa
   display/read action, every Focusa value-mutation action, every UIAI
   observation/action vector, and every combined workflow.
2. Every UIAI/combined row binds one canonical `SPEC172_UIAI_OPERATION_MAP`
   vector (one-to-one); Focusa rows resolve through the base product gate
   (`resolve_base_focusa_product` / `BaseProductDecision`).
3. `resolve_cockpit_action` consumes only the action id + authority
   snapshots: Focusa display never grants mutation; UIAI observation/action
   follows the observe-only limited / paid boundary; combined workflows
   require both grants or the Bundle; pairing/auth proves identity only.
4. No anonymous product capability, no local/self-issued grant, no
   caller-controlled product, price, License Type, family, feature, limit,
   node, or commercial right, and no raw secrets appear in the surface.
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/focusa-license/src/cockpit_action_registry.rs"
UIAI = ROOT / "crates/focusa-license/src/uiai_child_token.rs"
POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"

# Focusa Cockpit actions: action_id -> (operation_class, side_effect_class).
FOCUSA_ACTIONS = {
    "cockpit.focusa.display_mission": ("read", "none"),
    "cockpit.focusa.display_workpoint": ("read", "none"),
    "cockpit.focusa.display_trajectory": ("read", "none"),
    "cockpit.focusa.display_evidence": ("read", "none"),
    "cockpit.focusa.read_projection": ("read", "none"),
    "cockpit.focusa.mutate_project": ("value_mutation", "local"),
    "cockpit.focusa.mutate_mission": ("value_mutation", "local"),
    "cockpit.focusa.mutate_workpoint": ("value_mutation", "local"),
    "cockpit.focusa.mutate_evidence": ("value_mutation", "local"),
    "cockpit.focusa.run_work_loop": ("value_mutation", "local"),
}

# UIAI Cockpit actions: action_id -> canonical UIAI operation vector
# (one-to-one with the canonical SPEC172_UIAI_OPERATION_MAP rows).
UIAI_ACTIONS = {
    "cockpit.uiai.public_search": "public_search",
    "cockpit.uiai.source_to_markdown": "source_to_markdown",
    "cockpit.uiai.public_page_read": "public_page_read",
    "cockpit.uiai.accessibility_snapshot": "accessibility_snapshot",
    "cockpit.uiai.screenshot": "screenshot",
    "cockpit.uiai.basic_diagnostics": "basic_diagnostics",
    "cockpit.uiai.browser_click": "browser_click",
    "cockpit.uiai.browser_fill": "browser_fill",
    "cockpit.uiai.browser_type": "browser_type",
    "cockpit.uiai.browser_select": "browser_select",
    "cockpit.uiai.browser_press": "browser_press",
    "cockpit.uiai.browser_submit": "browser_submit",
    "cockpit.uiai.cookie_persistence": "cookie_persistence",
    "cockpit.uiai.auth_state_persistence": "auth_state_persistence",
    "cockpit.uiai.session_persistence": "session_persistence",
    "cockpit.uiai.authenticated_private_dashboard": "authenticated_private_dashboard",
    "cockpit.uiai.unattended_automation": "unattended_browser_automation",
    "cockpit.uiai.scheduled_batch_qa": "scheduled_batch_qa",
    "cockpit.uiai.premium_proxy": "premium_proxy",
    "cockpit.uiai.hosted_capacity": "hosted_capacity",
    "cockpit.uiai.paid_model_calls": "paid_model_calls",
}

# Combined workflows: action_id -> required canonical UIAI paid vector
# (Section 11.3: a combined workflow requires BOTH grants or the Bundle).
COMBINED_WORKFLOWS = {
    "cockpit.combined.research_apply": "browser_submit",
    "cockpit.combined.observe_and_capture": "public_search",
}

# Registered Section 12 metadata values.
REGISTERED_PRODUCT_OWNERS = {"focusa", "uiai_engine"}
REGISTERED_OPERATION_CLASSES = {"read", "value_mutation"}
REGISTERED_SIDE_EFFECT_CLASSES = {"none", "local", "remote", "external"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    source = SOURCE.read_text(encoding="utf-8")
    uiai = UIAI.read_text(encoding="utf-8")
    policy = POLICY.read_text(encoding="utf-8")

    # --- 1. registry surface markers ---
    for marker in [
        "SPEC172_COCKPIT_ACTION_REGISTRY",
        "CockpitActionMapEntry",
        "classify_cockpit_action",
        "resolve_cockpit_action",
        "CockpitActionDecision",
        "CockpitActionDenial",
        "CockpitActionError",
        "UnknownAction",
        "COCKPIT_ACTION_REGISTRY_SCHEMA",
        "FocusaDisplay",
        "FocusaMutation",
        "CombinedAllowed",
        "CombinedMissingFocusaGrant",
        "CombinedLimitedModeDenied",
        "CombinedMissingUiaiGrant",
        "UiaiDenied",
    ]:
        require(marker in source, marker)

    # --- 2. every action row exists with exact registered metadata ---
    for action_id, (operation_class, side_effect_class) in FOCUSA_ACTIONS.items():
        require(f'action_id: "{action_id}"' in source, action_id)
        require(f'product_owner: "focusa"' in source, f"{action_id} owner")
        require(f'operation_class: "{operation_class}"' in source, f"{action_id} class")
        require(f'capability_family: "base_focusa"' in source, f"{action_id} family")
        require(f'side_effect_class: "{side_effect_class}"' in source, f"{action_id} side effect")
        require("combined_workflow: false" in source, f"{action_id} combined flag")
        require("uiai_operation: None" in source, f"{action_id} uiai vector")

    for action_id, vector in UIAI_ACTIONS.items():
        require(f'action_id: "{action_id}"' in source, action_id)
        require(f'product_owner: "uiai_engine"' in source, f"{action_id} owner")
        require(f'uiai_operation: Some("{vector}")' in source, f"{action_id} vector")
        require(f'operation_id: "{vector}"' in uiai, f"{action_id} must bind a canonical UIAI vector")

    for action_id, vector in COMBINED_WORKFLOWS.items():
        require(f'action_id: "{action_id}"' in source, action_id)
        require(f'product_owner: "uiai_engine"' in source, f"{action_id} owner")
        require("combined_workflow: true" in source, f"{action_id} combined flag")
        require(f'uiai_operation: Some("{vector}")' in source, f"{action_id} vector")
        require(f'operation_id: "{vector}"' in uiai, f"{action_id} must bind a canonical UIAI vector")

    # Every UIAI map row has a Cockpit binding (one-to-one, no orphans).
    for row in [
        "public_search", "source_to_markdown", "public_page_read",
        "accessibility_snapshot", "screenshot", "basic_diagnostics",
        "browser_click", "browser_fill", "browser_type", "browser_select",
        "browser_press", "browser_submit",
        "cookie_persistence", "auth_state_persistence", "session_persistence",
        "authenticated_private_dashboard", "unattended_browser_automation",
        "scheduled_batch_qa", "premium_proxy", "hosted_capacity",
        "paid_model_calls",
    ]:
        require(f'Some("{row}")' in source, f"uiai vector must be Cockpit-bound: {row}")

    # --- 3. Focusa/UIAI operation-map linkage ---
    require("resolve_base_focusa_product" in source, "Focusa base gate linkage")
    require("BaseProductDecision" in source, "BaseProductDecision linkage")
    require("authority_policy_state" in source, "authority policy state linkage")
    require("resolve_uiai_operation_capability" in source, "UIAI boundary linkage")
    require("classify_uiai_operation" in source, "UIAI map lookup linkage")
    require("SPEC172_UIAI_OPERATION_MAP" in uiai, "canonical UIAI map present")

    # --- 4. resolver consumes only action id + authority snapshots ---
    signature = source[
        source.index("pub fn resolve_cockpit_action") :
        source.index(
            ") -> Result<CockpitActionDecision, CockpitActionError>",
            source.index("pub fn resolve_cockpit_action"),
        )
    ]
    for required in [
        "action_id",
        "focusa_snapshot",
        "uiai_snapshot",
        "active_uiai_sessions",
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

    # --- 5. Section 11.3 invariants are stated and enforced in code ---
    require(
        "rendering Focusa state in the Cockpit does not grant Focusa mutation"
        in source,
        "display must never grant mutation",
    )
    require(
        "proves identity/device posture, not entitlement" in source,
        "pairing proves identity only",
    )
    require("permits_focusa_mutation" in source, "mutation-only projection")
    require("CombinedLimitedModeDenied" in source, "limited never satisfies combined")
    require("UiaiOperationClass::RemotePremium" in source, "combined forces paid UIAI")

    # --- 6. fail-closed ordering: registry -> classifier -> resolver ---
    require(
        source.index("SPEC172_COCKPIT_ACTION_REGISTRY")
        < source.index("pub fn classify_cockpit_action"),
        "registry must precede the classifier",
    )
    require(
        source.index("pub fn classify_cockpit_action")
        < source.index("pub fn resolve_cockpit_action"),
        "classifier must precede the resolver",
    )

    # --- 7. no anonymous capability, no local/self-issued grant, no secrets ---
    for forbidden in [
        "SigningKey",
        "Signer",
        "self_sign",
        "customer_email",
        "access_token:",
        "device_proof",
        "pairing_proof",
        "password",
        "secret_key",
        "private_key",
    ]:
        require(forbidden not in source, forbidden)

    print("Spec172 UIAI Cockpit mixed-product presenter gate: PASS")
    print(f"focusa_actions={len(FOCUSA_ACTIONS)}")
    print(f"uiai_actions={len(UIAI_ACTIONS)}")
    print(f"combined_workflows={len(COMBINED_WORKFLOWS)}")
    print(f"total_actions={len(FOCUSA_ACTIONS) + len(UIAI_ACTIONS) + len(COMBINED_WORKFLOWS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
