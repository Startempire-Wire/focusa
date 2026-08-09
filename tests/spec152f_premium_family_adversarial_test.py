#!/usr/bin/env python3
"""Build-independent adversarial matrix gate for Spec 152F.04.07.

Atom focusa-vbcqu.20.14.36 (152F.04.07): prove premium limit, idempotency,
and grant isolation.

Exact verification:
    python3 tests/spec152f_premium_family_adversarial_test.py \\
        && cargo test --workspace spec152f_premium_adversarial

The gate checks the Rust reservation service, the four premium-family tests,
the feature/limit registry, and the deterministic product fixtures:

1. `LimitReservationService` re-resolves the canonical premium family decision
   before reserving, reads capacity only from authority-owned lease limits,
   binds reservations to lease identity and account/node scope, and fails
   closed on: wrong product, caller-supplied feature, caller-supplied limit
   bucket, stale/expired lease, replay/duplicate request, exhausted limit,
   concurrent reservation, Evaluation omission, Offline Grace expansion,
   cross-family features, and cross-account access.
2. The four premium-family tests each gain an adversarial isolation case
   (wrong product, Evaluation omission, Offline Grace expansion, caller
   feature) so no family can be widened by client metadata or stale state.
3. The declared server-owned limit bucket registry is frozen and authoritative;
   the premium-family feature registry in the policy contract is authoritative.
4. No raw keys/tokens/customer PII appear in the fixture or source surfaces.
"""

import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "crates/focusa-license/tests/fixtures/spec152f-premium-family-adversarial-fixtures.v1.json"
SERVICE = ROOT / "crates/focusa-license/src/limit_reservation.rs"
MATRIX = ROOT / "crates/focusa-license/tests/spec152f_premium_adversarial.rs"
POLICY = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"

FOUR_PREMIUM_TESTS = [
    ROOT / "crates/focusa-license/tests/spec152f_automation_entitlement.rs",
    ROOT / "crates/focusa-license/tests/spec152f_team_remote_entitlement.rs",
    ROOT / "crates/focusa-license/tests/spec152f_release_proof_entitlement.rs",
    ROOT / "crates/focusa-license/tests/spec152f_export_entitlement.rs",
]

REQUIRED_STEPS = [
    "wrong product",
    "caller-supplied feature",
    "stale lease",
    "duplicate request",
    "exhausted limit",
    "concurrent reservation",
    "Evaluation omission",
    "Offline Grace expansion",
    "cross-family",
    "cross-account",
]

SERVICE_MARKERS = [
    "LimitReservationService",
    "pub fn reserve",
    "pub fn revalidate",
    "pub fn settle",
    "DECLARED_SERVER_OWNED_LIMIT_BUCKETS",
    "family_limit_buckets",
    "ReservationScope",
    "from_snapshot",
    "snapshot.limits",
    "IdempotencyConflict",
    "LimitExhausted",
    "StaleLease",
    "UnknownLimitBucket",
    "UnknownReservation",
    "FamilyDenied",
    "resolve_premium_family",
    "resolve_export_packaged",
]

FORBIDDEN_FRAGMENTS = [
    "SigningKey",
    "signing_key",
    "private_key",
    "secret_key",
    "customer_email",
    "access_token",
    "pairing_proof",
    "device_proof",
]


def main() -> int:
    failures: list[str] = []

    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    if fixture["schema"] != "focusa.spec152f.premium_family_adversarial_fixtures.v1":
        failures.append("fixture schema mismatch")

    # --- 1. The reservation service exposes the full fail-closed surface ---
    service = SERVICE.read_text(encoding="utf-8")
    for marker in SERVICE_MARKERS:
        if marker not in service:
            failures.append(f"service missing marker: {marker}")

    # The reserve() decision must be re-resolved from the snapshot, capacity must
    # come from snapshot.limits, and the idempotency/stale paths must exist.
    if "let Some(capacity) = snapshot.limits.get(bucket)" not in service:
        failures.append("capacity must come from authority-owned snapshot.limits")
    if "self.records.get(idempotency_key)" not in service:
        failures.append("idempotency replay path missing")
    if "self.resolve_decision(" not in service:
        failures.append("every reservation must re-resolve the family decision")

    # --- 2. The adversarial matrix covers every required step ---
    matrix = MATRIX.read_text(encoding="utf-8")
    test_names = re.findall(r"fn (spec152f_premium_adversarial_[a-z0-9_]+)", matrix)
    if not test_names:
        failures.append("no spec152f_premium_adversarial tests found")
    joined = "\n".join(test_names).lower()
    for step in REQUIRED_STEPS:
        key = (
            step.replace(" ", "_")
            .replace("-", "_")
            .replace("(", "")
            .replace(")", "")
            .lower()
        )
        if key not in joined:
            failures.append(f"matrix missing required step: {step}")

    # --- 3. The four premium-family tests each carry adversarial isolation ---
    for path in FOUR_PREMIUM_TESTS:
        text = path.read_text(encoding="utf-8")
        if "Adversarial isolation (Spec 152F.04.07)" not in text:
            failures.append(f"{path.name} missing adversarial isolation case")
        for needle in ["wrong product", "Evaluation", "Offline Grace"]:
            if needle.lower() not in text.lower():
                failures.append(f"{path.name} missing adversarial needle: {needle}")

    # --- 4. Product fixtures are deterministic and complete ---
    product_ids = {p["id"] for p in fixture["products"]}
    expected_products = {
        "focusa-automation-operator",
        "focusa-team-remote-operator",
        "focusa-release-proof-operator",
        "focusa-premium-updates-operator",
        "focusa-export-packaged-operator",
    }
    if product_ids != expected_products:
        failures.append(f"fixture products mismatch: {sorted(product_ids)}")
    case_ids = {c["id"] for c in fixture["cases"]}
    expected_cases = {
        "wrong-product",
        "evaluation-omission",
        "offline-grace-expansion",
        "stale-lease",
        "cross-account",
        "caller-feature",
    }
    if case_ids != expected_cases:
        failures.append(f"fixture cases mismatch: {sorted(case_ids)}")
    if len(fixture["declared_limit_buckets"]) != 8:
        failures.append("declared limit bucket set must be frozen at 8 buckets")
    for product in fixture["products"]:
        for feature, granted in product["snapshot"]["features"].items():
            if not feature.startswith("focusa."):
                failures.append(f"unqualified feature in fixture: {feature}")

    # --- 5. Policy contract cross-check (feature/limit registry) ---
    policy = yaml.safe_load(POLICY.read_text(encoding="utf-8"))
    if set(policy["premium_families"]) != {
        "automation",
        "premium_updates",
        "release_proof",
        "team_remote",
    }:
        failures.append("policy premium_families must be the exact four")
    limit_dimension = next(
        (d for d in policy["future_dimensions"] if d["id"] == "limit_bucket"), None
    )
    if not limit_dimension:
        failures.append("policy limit_bucket dimension missing")
    elif limit_dimension.get("authority") != "server_owned_registry_and_lease":
        failures.append("limit_bucket authority must be server-owned registry + lease")
    invariants = " ".join(policy.get("invariants", []))
    for invariant in [
        "premium_requires_base_first",
        "offline_grace_never_expands_features_or_limits",
        "refund_revoke_or_higher_sequence_overrides_cached_grants",
    ]:
        if invariant not in invariants:
            failures.append(f"policy missing invariant: {invariant}")

    # --- 6. Forbidden material never appears ---
    for path, text in [
        (FIXTURE, json.dumps(fixture)),
        (SERVICE, service),
        (MATRIX, matrix),
    ]:
        for fragment in FORBIDDEN_FRAGMENTS:
            if fragment in text:
                failures.append(f"{path.name} contains forbidden fragment: {fragment}")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1

    print(
        "Spec152f premium family adversarial matrix gate: PASS "
        f"({len(test_names)} matrix tests, {len(fixture['products'])} product fixtures, "
        f"{len(fixture['cases'])} adversarial cases)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
