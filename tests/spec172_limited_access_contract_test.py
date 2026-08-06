#!/usr/bin/env python3
"""Contract and vector gate for Spec 172 verified limited access."""

from __future__ import annotations

import json
import pathlib
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/contracts/spec172-verified-limited-access.v1.yaml"
POLICY = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"
CASES = ROOT / "tests/fixtures/spec172-limited-access-cases.v1.json"


def load_yaml(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def expected_decision(registry: dict, case: dict) -> str:
    posture = case["posture"]
    family = case["family"]
    if posture == "unverified":
        return "allow" if family in registry["postures"]["unverified"]["allowed_operations"] else "deny"
    if posture != "verified_no_license":
        return "deny"
    if family in registry["permanent_allowances"]["families"]:
        return "allow"
    product = case["product"]
    if product == "focusa":
        if family in registry["focusa"]["blocked_families"]:
            return "deny"
        if family in registry["focusa"]["allowed_families"]:
            return "allow" if case.get("mutable_project_count", 1) <= 1 else "deny"
    if product == "uiai_engine":
        if family in registry["uiai_engine"]["blocked_families"]:
            return "deny"
        if family in registry["uiai_engine"]["allowed_families"]:
            valid_session = case.get("session_count", 1) <= 1
            valid_mode = case.get("session_mode", "foreground_ephemeral") == "foreground_ephemeral"
            return "allow" if valid_session and valid_mode else "deny"
    return "deny"


def main() -> int:
    failures: list[str] = []
    registry = load_yaml(REGISTRY)
    policy = load_yaml(POLICY)
    with CASES.open(encoding="utf-8") as handle:
        vectors = json.load(handle)

    unverified = registry["postures"]["unverified"]
    limited = registry["postures"]["verified_no_license"]
    if unverified.get("product_access") != "registration_only" or unverified.get("default") != "deny":
        failures.append("unverified posture must be registration-only and default deny")
    for key, expected in {
        "is_license_type": False,
        "creates_edd_key": False,
        "expiry": "none",
        "automatic_expiry": False,
        "default": "deny",
    }.items():
        if limited.get(key) != expected:
            failures.append(f"verified_no_license.{key}: expected {expected!r}")

    focusa = registry["focusa"]
    uiai = registry["uiai_engine"]
    if focusa.get("mutable_project_limit") != 1:
        failures.append("Focusa limited mode must allow exactly one mutable project")
    if (uiai.get("session_limit"), uiai.get("concurrency_limit"), uiai.get("execution_mode"), uiai.get("persistence")) != (1, 1, "foreground", "ephemeral"):
        failures.append("UIAI limited mode must be one foreground ephemeral session")

    required_focusa_blocked = {"automation", "team_remote", "release_proof", "premium_updates"}
    required_uiai_blocked = {"browser_action", "browser_persistence"}
    if not required_focusa_blocked <= set(focusa["blocked_families"]):
        failures.append("Focusa blocked-family boundary is incomplete")
    if not required_uiai_blocked <= set(uiai["blocked_families"]):
        failures.append("UIAI action/persistence boundary is incomplete")

    permanent = set(registry["permanent_allowances"]["families"])
    if not {"read_projection", "basic_customer_data_export", "repair", "rollback", "stable_security_update", "uninstall"} <= permanent:
        failures.append("permanent read/export/recovery allowances are incomplete")
    if registry["permanent_allowances"].get("expiry") != "none":
        failures.append("permanent allowances must not expire")

    ids: set[str] = set()
    covered: dict[tuple[str, str], set[str]] = {}
    for case in vectors.get("cases", []):
        if case["id"] in ids:
            failures.append(f"duplicate case id: {case['id']}")
        ids.add(case["id"])
        actual = expected_decision(registry, case)
        if actual != case["decision"]:
            failures.append(f"{case['id']}: expected vector {case['decision']}, resolver returned {actual}")
        covered.setdefault((case["product"], case["decision"]), set()).add(case["family"])

    for product, section in (("focusa", focusa), ("uiai_engine", uiai)):
        if not set(section["allowed_families"]) <= covered.get((product, "allow"), set()):
            failures.append(f"{product}: positive vectors do not cover every allowed family")
        if not set(section["blocked_families"]) <= covered.get((product, "deny"), set()):
            failures.append(f"{product}: negative vectors do not cover every blocked family")

    grid = {item["state"]: item for item in policy.get("state_grid", [])}
    if "evaluation" in grid or "verified_no_grant" in grid:
        failures.append("Spec 152F active state grid retains superseded Evaluation/no-grant state")
    projected = grid.get("verified_no_license", {})
    if projected.get("expiry") != "none" or projected.get("edd_software_license_key") is not False:
        failures.append("Spec 152F verified_no_license projection differs from Spec 172")
    if policy.get("spec172_limited_access_policy") != "docs/contracts/spec172-verified-limited-access.v1.yaml":
        failures.append("Spec 152F does not reference the canonical Spec 172 limited-access registry")

    if failures:
        print("Spec 172 limited-access contract test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Spec 172 limited-access contract test passed")
    print(f"cases={len(vectors['cases'])}")
    print("postures=unverified,verified_no_license")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
