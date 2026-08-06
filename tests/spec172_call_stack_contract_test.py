#!/usr/bin/env python3
"""Contract gate for the Spec 172 entitlement call stack and stable errors."""
from __future__ import annotations

import json
import pathlib
import sys
import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
STACK = ROOT / "docs/contracts/spec172-entitlement-call-stack.v1.yaml"
ERRORS = ROOT / "docs/contracts/spec172-entitlement-errors.v1.json"
LEGACY = ROOT / "docs/contracts/spec152f-entitlement-call-stack.v1.yaml"


def yaml_map(path: pathlib.Path) -> dict:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(value, dict), f"{path}: expected mapping"
    return value


def main() -> int:
    failures: list[str] = []
    stack = yaml_map(STACK)
    legacy = yaml_map(LEGACY)
    errors = json.loads(ERRORS.read_text(encoding="utf-8"))

    expected_chain = ["verified_identity", "limited_assertion_or_edd_paid_key", "node", "signed_lease", "product", "license_type", "family", "resource", "execution"]
    if stack.get("commercial_chain") != expected_chain:
        failures.append("commercial chain is not the frozen identity-to-execution order")
    expected_independent = {"auth", "role", "scope", "confirmation", "trust", "privacy", "entitlement"}
    if set(stack.get("independent_controls", [])) != expected_independent:
        failures.append("independent controls are incomplete or coupled")

    stages = stack.get("stages", [])
    if [s.get("order") for s in stages] != list(range(1, len(stages) + 1)):
        failures.append("stage order must be contiguous")
    if len({s.get("id") for s in stages}) != len(stages):
        failures.append("stage ids must be unique")
    for stage in stages:
        owner = stage.get("owner")
        if not isinstance(owner, str) or not owner.strip():
            failures.append(f"{stage.get('id')}: must have exactly one scalar owner")
        chokepoint = stage.get("chokepoint")
        if not isinstance(chokepoint, str) or not chokepoint.strip():
            failures.append(f"{stage.get('id')}: must locate its chokepoint")

    locations = stack.get("implementation_locations", {})
    for kind in ("handlers", "services", "stores", "chokepoints"):
        entries_for_kind = locations.get(kind)
        if not isinstance(entries_for_kind, dict) or not entries_for_kind:
            failures.append(f"implementation_locations.{kind} must locate every {kind[:-1]}")
        elif any(not isinstance(path, str) or not path.strip() for path in entries_for_kind.values()):
            failures.append(f"implementation_locations.{kind} contains an empty location")

    unknown_metadata = stack.get("unknown_metadata", {})
    expected_unknown_codes = {
        "product": "PRODUCT_NOT_INCLUDED",
        "license_type": "LICENSE_TYPE_NOT_INCLUDED",
        "capability_family": "CAPABILITY_FAMILY_NOT_INCLUDED",
        "side_effect": "SIDE_EFFECT_UNCLASSIFIED",
    }
    for kind, code in expected_unknown_codes.items():
        policy = unknown_metadata.get(kind, {})
        if policy != {"disposition": "deny_before_execution", "error_code": code}:
            failures.append(f"unknown {kind} must fail with {code} before execution")

    presenters = stack.get("presenters", {})
    if not presenters or any(p.get("authority") != "none" for p in presenters.values()):
        failures.append("every presenter must have zero authority")

    entries = errors.get("errors", [])
    codes = [entry.get("code") for entry in entries]
    if len(codes) != len(set(codes)) or None in codes:
        failures.append("error codes must be present and unique")
    recoveries: dict[str, str] = {}
    for entry in entries:
        recovery = entry.get("recovery_action")
        if not isinstance(recovery, str) or not recovery.strip():
            failures.append(f"{entry.get('code')}: missing stable recovery action")
        else:
            recoveries[entry["code"]] = recovery
    referenced = {code for stage in stages for code in stage.get("failure_codes", [])}
    missing = referenced - set(codes)
    if missing:
        failures.append(f"stage errors absent from stable registry: {sorted(missing)}")
    unknown = errors.get("unknown_error", {})
    if recoveries.get(unknown.get("code")) != unknown.get("recovery_action"):
        failures.append("unknown error fallback must reuse one registered stable recovery")

    supersession = legacy.get("supersession", {})
    if supersession.get("canonical_contract") != "docs/contracts/spec172-entitlement-call-stack.v1.yaml":
        failures.append("Spec 152F does not defer to the Spec 172 call stack")
    if legacy.get("authority", {}).get("stable_errors") != "docs/contracts/spec172-entitlement-errors.v1.json":
        failures.append("Spec 152F does not use Spec 172 stable errors")

    if failures:
        print("Spec 172 call-stack contract test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Spec 172 call-stack contract test passed")
    print(f"stages={len(stages)} errors={len(entries)} presenters={len(presenters)}")
    print("commercial_chain=" + "->".join(expected_chain))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
