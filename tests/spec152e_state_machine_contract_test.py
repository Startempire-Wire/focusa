#!/usr/bin/env python3
"""Validate Spec 152E registration, polling, retry, and recovery semantics."""

import json
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INTERNAL = ROOT / "docs/contracts/spec152e-activation-internal.v1.json"
ERRORS = ROOT / "docs/contracts/spec152e-activation-errors.v1.json"
contract = json.loads(INTERNAL.read_text(encoding="utf-8"))
errors = json.loads(ERRORS.read_text(encoding="utf-8"))

machine = contract["registration_states"]
initial = machine["initial"]
nonterminal = set(machine["nonterminal"])
terminal = set(machine["terminal"])
transitions = {state: set(destinations) for state, destinations in machine["transitions"].items()}
all_states = nonterminal | terminal
assert initial == "attempt_created"
assert nonterminal.isdisjoint(terminal)
assert set(transitions) == all_states
assert all(destinations <= all_states for destinations in transitions.values())
assert transitions["recovery_only"] == set()
assert {"expired", "denied", "refunded", "revoked", "superseded"} <= terminal

reachable = {initial}
queue = deque([initial])
while queue:
    for destination in transitions[queue.popleft()]:
        if destination not in reachable:
            reachable.add(destination)
            queue.append(destination)
assert reachable == all_states
assert "email_verified" not in transitions["attempt_created"]
assert "account_promoted" not in transitions["email_challenge_sent"]
for privileged in ("account_promoted", "entitlement_issued", "device_registered", "lease_issued", "delivered"):
    assert privileged in reachable
assert transitions["refunded"] == {"recovery_only"}
assert transitions["revoked"] == {"recovery_only"}
assert transitions["expired"] == {"recovery_only"}
assert transitions["denied"] == {"recovery_only"}
assert transitions["superseded"] == {"recovery_only"}

presenter_states = set(contract["presenter_states"])
assert presenter_states == {
    "email_required", "email_verification_pending", "email_verified", "selection_required",
    "checkout_required", "payment_pending", "license_delivery_ready", "activated", "denied",
    "recovery_only",
}
polling = contract["polling"]
assert polling["stored_as"] == "keyed_hash_only"
assert set(polling["binding"]) == {"registration_id", "facade_id", "action", "expiry"}
assert set(polling["terminal_states"]) == {"activated", "denied", "recovery_only"}
assert 1 <= polling["default_retry_after_seconds"] <= polling["maximum_retry_after_seconds"] <= 30

rules = polling["retry_rules"]
rule_codes = [code for codes in rules.values() for code in codes]
assert len(rule_codes) == len(set(rule_codes))
registry = {row["code"]: row for row in errors["errors"]}
assert set(rule_codes) <= set(registry)
for code in rules["safe_retry"]:
    assert registry[code]["retryable"] is True
assert registry["REQUEST_IN_PROGRESS"]["safe_next_action"] == "retry_same_idempotency_key"
for code in rules["recovery_only"]:
    assert registry[code]["safe_next_action"] == "recovery_only"
for code in rules["do_not_retry_unchanged"]:
    assert registry[code]["retryable"] is False

operation_states = {state for op in contract["operations"] for state in op["success_states"]}
assert operation_states <= presenter_states
assert set(contract["canonical_output"]["required"]) == {
    "schema", "request_id", "registration_id", "state", "terminal", "retry", "next_action"
}
assert "masked_email" in contract["canonical_output"]["optional"]
assert {"email", "normalized_email", "raw_email", "full_license_key"} <= set(contract["canonical_output"]["forbidden"])
assert "unknown_operation_state_transition_or_error_fails_closed" in contract["invariants"]
assert "paid_accounts_are_never_downgraded_by_limited_access_activation" in contract["invariants"]

print(json.dumps({"schema": "focusa.spec152e.state_machine_validation.v1", "states": len(all_states), "transitions": sum(map(len, transitions.values())), "result": "passed"}, sort_keys=True))
