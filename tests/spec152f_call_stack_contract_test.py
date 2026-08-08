#!/usr/bin/env python3
"""Validate the Spec 152F resolver call stack and stable error ownership."""

import hashlib
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
STACK_PATH = ROOT / "docs/contracts/spec152f-entitlement-call-stack.v1.yaml"
ERROR_PATH = ROOT / "docs/contracts/spec152f-entitlement-errors.v1.json"
POLICY_PATH = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"

stack = yaml.safe_load(STACK_PATH.read_text(encoding="utf-8"))
errors = json.loads(ERROR_PATH.read_text(encoding="utf-8"))
policy = yaml.safe_load(POLICY_PATH.read_text(encoding="utf-8"))

assert len(STACK_PATH.read_text().splitlines()) < 500
assert len(ERROR_PATH.read_text().splitlines()) < 500
assert stack["schema"] == "focusa.spec152f.entitlement_call_stack.v1"
assert stack["contract_version"] == 1
assert errors["schema"] == "focusa.spec152f.entitlement_errors.v1"
assert errors["contract_version"] == 1
assert stack["authority"]["policy"] == "docs/contracts/spec152f-entitlement-policy.v1.yaml"
assert stack["authority"]["signed_lease"] == "crates/focusa-license/src/authority.rs"
assert stack["authority"]["runtime_guard"] == "crates/focusa-license/src/lib.rs"
assert stack["authority"]["customer_commerce_human_key_entitlement"] == "WPUIAI.com EDD"
assert stack["authority"]["stable_errors"].startswith("docs/contracts/")

required_rules = {
    "classify_before_entitlement_decision",
    "security_authorization_before_commercial_allow",
    "recovery_and_safe_read_before_base_or_premium_gate",
    "signed_lease_before_base_or_premium_allow",
    "base_before_premium_feature",
    "idempotency_and_limit_reservation_before_side_effect",
    "execution_before_reservation_settlement",
    "one_canonical_decision_projected_to_all_presenters",
    "unknown_side_effect_policy_family_or_activation_fails_closed",
    "licensing_never_grants_operator_role_or_cognitive_authority",
}
assert required_rules.issubset(set(stack["rules"]))

stage_order = [
    "entrypoint_binding",
    "operation_classification",
    "identity_permission_confirmation",
    "recovery_read_resolution",
    "signed_lease_verification",
    "state_base_resolution",
    "premium_feature_resolution",
    "idempotency_limit_reservation",
    "protected_execution",
    "reservation_settlement",
    "decision_projection",
]
stages = stack["stages"]
assert len(stages) == len(stage_order) == 11
assert [stage["order"] for stage in stages] == list(range(1, 12))
assert [stage["id"] for stage in stages] == stage_order
assert len({stage["id"] for stage in stages}) == 11
for stage in stages:
    assert isinstance(stage["owner"], str) and stage["owner"].strip()
    assert stage["responsibility"].strip()
    assert stage["input"].strip() and stage["output"].strip()
    assert isinstance(stage["failure_codes"], list)
    for surface in stage["current_surfaces"]:
        if surface.startswith("planned:"):
            continue
        assert (ROOT / surface).exists(), f"{stage['id']}: missing current surface {surface}"

assert stages[2]["owner"] == "security_authorization"
assert stages[3]["owner"] == stages[5]["owner"] == stages[6]["owner"] == "entitlement_policy_resolver"
assert stages[7]["owner"] == stages[9]["owner"] == "entitlement_limit_reservation"
assert stages[8]["owner"] == "operation_handler"
assert stages[10]["owner"] == "entitlement_decision_projection"

paths = stack["execution_paths"]
assert set(paths) == {"http", "non_http", "delayed_dispatch"}
for name, path in paths.items():
    assert path["stage_order"] == stage_order, name
    assert path["entrypoints"], name
    chokepoint = path["mandatory_chokepoint"]
    if chokepoint.startswith("planned:"):
        assert chokepoint == "planned:crates/focusa-core/src/entitlement_execution_guard.rs"
    else:
        assert (ROOT / chokepoint).is_file()
assert "queue time never extends Offline Grace" in paths["delayed_dispatch"]["additional_rule"]

presenters = stack["presenter_boundaries"]
assert {row["presenter"] for row in presenters} == {
    "REST",
    "CLI",
    "menubar_TUI",
    "Pi_agent",
    "installer_lifecycle",
    "branded_facade",
    "UIAI_child",
}
for presenter in presenters:
    assert presenter["commercial_decision"] == "forbidden"
    assert presenter["owns"]

output = stack["canonical_output"]
assert output["type"] == "EntitlementDecision"
assert set(output["forbidden_fields"]) == {
    "raw_key",
    "refresh_token",
    "node_secret",
    "customer_email",
    "payment_data",
}
assert {
    "status",
    "entitlement_state",
    "operation_id",
    "operation_class",
    "capability_family",
    "commercial_treatment",
    "required_feature",
    "limit_bucket",
    "reason_code",
    "recovery_action",
    "policy_digest",
    "lease_sequence",
} == set(output["fields"])

error_rows = errors["errors"]
error_codes = [row["code"] for row in error_rows]
assert len(error_codes) == len(set(error_codes)) == 10
expected_existing = {
    "ENTITLEMENT_FEATURE_REQUIRED",
    "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
    "ENTITLEMENT_LIMIT_EXHAUSTED",
    "ENTITLEMENT_REQUIRED",
    "ENTITLEMENT_RESERVATION_FAILED",
    "ENTITLEMENT_ROUTE_UNCLASSIFIED",
    "ENTITLEMENT_SNAPSHOT_MISSING",
}
expected_new = {
    "ENTITLEMENT_FAMILY_UNKNOWN",
    "ENTITLEMENT_POLICY_ACTIVATION_FORBIDDEN",
    "ENTITLEMENT_POLICY_UNKNOWN",
}
assert set(error_codes) == expected_existing | expected_new
assert {row["code"] for row in error_rows if row["source_status"] == "existing"} == expected_existing
assert {row["code"] for row in error_rows if row["source_status"] == "spec152f_new"} == expected_new

source_text = "\n".join(
    (ROOT / path).read_text(encoding="utf-8")
    for path in (
        "crates/focusa-api/src/middleware/entitlement.rs",
        "crates/focusa-license/src/lib.rs",
    )
)
source_codes = set(re.findall(r"ENTITLEMENT_[A-Z0-9_]+", source_text))
assert expected_existing <= source_codes
assert not (expected_new & source_codes), "new errors are contract-frozen, not falsely claimed implemented"

allowed_context = {
    "capability_family",
    "entitlement_state",
    "operation_id",
    "required_feature",
    "limit_bucket",
    "policy_digest",
}
for row in error_rows:
    assert row["category"]
    assert row["http_status"] in {403, 428, 429, 503}
    assert isinstance(row["retryable"], bool)
    assert row["public_message"] and row["recovery_action"]
    assert set(row["required_context"]) <= allowed_context
    lower = row["public_message"].lower()
    assert "customer_email" not in lower and "raw key" not in lower and "token=" not in lower
for stage in stages:
    assert set(stage["failure_codes"]) <= set(error_codes), stage["id"]

assert errors["rules"] == {
    "before_side_effect": True,
    "public_messages_are_redacted": True,
    "presenters_must_not_rewrite_codes": True,
    "unknown_codes_fail_closed": True,
    "licensing_denial_does_not_bypass_security": True,
}
recovery = set(errors["recovery_paths"]["always_reachable_subject_to_security"])
assert {
    "account_management",
    "activation",
    "basic_customer_data_export",
    "diagnostics",
    "license_status",
    "node_deactivation",
    "purchase",
    "registration",
    "repair",
    "rollback",
    "stable_security_update",
    "uninstall",
    "verification",
} == recovery
assert set(errors["recovery_paths"]["never_implied_by_error"]) == {
    "anonymous_access",
    "commercial_grant",
    "cognitive_authority",
    "operator_permission",
    "role_permission",
}

assert stack["legacy_boundaries"]["local_license_json_or_toml"].startswith("migration input only")
assert stack["legacy_boundaries"]["installer_eval_flag"] == "forbidden as entitlement authority"
assert stack["implementation_sequence"] == [f"focusa-vbcqu.20.14.{n}" for n in range(5, 22)]
assert policy["authority"]["customer_commerce_human_key_entitlement"] == "WPUIAI.com EDD"
assert policy["commercial_model"]["base_gate_count"] == 1
assert policy["commercial_model"]["premium_family_count"] == 4

stack_digest = hashlib.sha256(
    json.dumps(stack, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
error_digest = hashlib.sha256(
    json.dumps(errors, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
print(json.dumps({
    "schema": "focusa.spec152f.call_stack_validation.v1",
    "stages": len(stages),
    "execution_paths": len(paths),
    "presenters": len(presenters),
    "errors": len(error_rows),
    "existing_errors": len(expected_existing),
    "new_errors": len(expected_new),
    "stack_sha256": stack_digest,
    "errors_sha256": error_digest,
    "result": "passed",
}, sort_keys=True))
