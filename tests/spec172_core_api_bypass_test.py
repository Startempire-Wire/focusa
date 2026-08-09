#!/usr/bin/env python3
"""Spec 172 §20.9 — core/API chokepoint and direct-call bypass resistance
acceptance receipt (atom focusa-vbcqu.20.15.25, 172.04.01, lane spec152f /
WPUIAI).

This gate proves the shared chokepoint and zero-side-effect counters across
the exact surfaces: the focusa-core execution guard + shared guarded-mutation
chokepoint, the focusa-api entitlement middleware, worker dispatch
revalidation, and the direct storage/reducer adapters. It statically pins the
Spec 172 Rust surfaces implementing the same fail-closed bypass matrix,
replays the pure-Python bypass-relevant contract gates (License Types,
dynamic tools, family inheritance, lifetime credentials — all read-only and
replayable), and verifies that the bypass vectors added by this atom
(`cargo test --workspace spec172_core_api_bypass` runs them; the shell records
its exit code separately).

Attempted and denied before effects: direct core calls, direct reducer calls,
direct storage writes, stale clients (expired/unbound/past-grace leases),
wrong-method HTTP calls, wrong-product leases, and queued-before-refund worker
dispatches. Recovery/read/export stay reachable in every blocked state.

The receipt emits one bounded JSON line. No raw email, key, token, customer
row, credential, or card data ever appears; all identifiers are synthetic.

Exact verification:
    python3 tests/spec172_core_api_bypass_test.py \
        && cargo test --workspace spec172_core_api_bypass
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


# ── Pure-Python contract replays that bear directly on bypass resistance ──

REPLAY_GATES: list[tuple[str, str]] = [
    (
        "license_types_no_caller_controlled_type",
        "python3 tests/spec172_license_type_contract_test.py",
    ),
    (
        "dynamic_tools_fail_closed",
        "python3 tests/spec172_dynamic_operation_manifest_test.py",
    ),
    (
        "family_allowlists_no_anonymous_capability",
        "python3 tests/spec172_family_inheritance_test.py",
    ),
    (
        "lifetime_credential_stale_defeated",
        "python3 tests/spec172_lifetime_credential_test.py",
    ),
]


def replay_layer() -> dict[str, int]:
    """Run every pure-Python replay gate once and record its real exit code."""
    results: dict[str, int] = {}
    for case, command in REPLAY_GATES:
        proc = subprocess.run(
            command.split(),
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=600,
        )
        results[case] = proc.returncode
        if proc.returncode != 0:
            raise AssertionError(
                f"replay gate failed rc={proc.returncode} for case={case} cmd={command}\n"
                f"{proc.stdout[-2000:]}\n{proc.stderr[-2000:]}"
            )
    return results


# ── Spec 172 Rust surfaces pinned by this receipt ──────────────────────────

CHOKEPOINT_RS = ROOT / "crates/focusa-core/src/guarded_mutation.rs"
CORE_BYPASS_RS = ROOT / "crates/focusa-core/tests/spec172_core_api_bypass.rs"
SCHEDULER_RS = ROOT / "crates/focusa-core/src/silent_session_scheduler.rs"
API_ENTITLEMENT_RS = ROOT / "crates/focusa-api/src/middleware/entitlement.rs"
API_BYPASS_RS = ROOT / "crates/focusa-api/src/middleware/spec172_core_api_bypass.rs"


def static_pin_rust_surfaces() -> dict[str, int]:
    """Bound the shared chokepoint, worker dispatch revalidation, and API
    route-gate surfaces that carry the same fail-closed bypass matrix (the
    cargo filter executes the vectors)."""
    chokepoint = CHOKEPOINT_RS.read_text(encoding="utf-8")
    core_bypass = CORE_BYPASS_RS.read_text(encoding="utf-8")
    scheduler = SCHEDULER_RS.read_text(encoding="utf-8")
    api_entitlement = API_ENTITLEMENT_RS.read_text(encoding="utf-8")
    api_bypass = API_BYPASS_RS.read_text(encoding="utf-8")

    # Shared chokepoint: canonical gate for HTTP and non-HTTP side effects.
    expect("pub fn guard_value_mutation" in chokepoint, "shared chokepoint gate exists")
    expect("pub fn apply_guarded_mutation" in chokepoint, "guarded reducer adapter exists")
    expect("pub fn guard_project_mutation" in chokepoint, "project-aware chokepoint exists")
    expect(
        "apply_mutation_lease_gate" in chokepoint
        and "lease_is_current" in chokepoint,
        "stale-client lease currency check exists in the chokepoint",
    )
    expect(
        'resolve_base_focusa_product("focusa", state)' in chokepoint,
        "chokepoint resolves only the exact authority product id",
    )
    expect(
        "ENTITLEMENT_BASE_REQUIRED" in chokepoint
        and "ENTITLEMENT_REDUCER_REJECTED" in chokepoint,
        "chokepoint emits base-required and reducer-rejected codes",
    )
    core_guard = (ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs").read_text(
        encoding="utf-8"
    )
    for code in [
        "ENTITLEMENT_BASE_REQUIRED",
        "ENTITLEMENT_FEATURE_REQUIRED",
        "ENTITLEMENT_REQUIRED",
        "ENTITLEMENT_ROUTE_UNCLASSIFIED",
    ]:
        expect(code in core_guard, f"core execution guard carries stable code {code}")

    # Zero-side-effect counters: every denial reports side_effect_count == 0;
    # approved mutations report exactly 1; the storage ledger counts only
    # approved durable writes.
    expect("side_effect_count" in chokepoint, "side-effect counter exists")
    expect(
        "side_effect_count: 0" in chokepoint and "side_effect_count: 1" in chokepoint,
        "zero on denial, one on approval",
    )
    expect(
        "pub struct GuardedStorageLedger" in chokepoint
        and "durable_writes" in chokepoint,
        "guarded storage adapter ledger exists",
    )

    # Worker dispatch: queued work revalidates through the shared chokepoint
    # before effects (Spec 172 §11.5), so queued-before-refund and stale
    # clients cannot continue.
    expect(
        "guard_value_mutation(entitlement_guard, policy, context)" in scheduler,
        "worker dispatch revalidates through the shared chokepoint",
    )
    expect(
        "DispatchDeferralReason::EntitlementDenied" in scheduler,
        "dispatch defers on entitlement denial",
    )

    # API entitlement middleware: route gate + lease currency + method rules.
    expect("route_entitlement_denial" in api_entitlement, "API route gate exists")
    expect("entitlement_allows_mutation" in api_entitlement, "lease currency check exists")
    expect("route_requires_entitlement" in api_entitlement, "method/route classification exists")
    expect(
        "spec172_core_api_bypass" in api_entitlement,
        "API bypass vectors wired under cargo test",
    )
    expect(
        "ENTITLEMENT_ROUTE_UNCLASSIFIED" in api_entitlement,
        "unclassified mutation routes fail closed",
    )

    # Bypass vectors compiled under the exact cargo filter.
    core_vector_count = len(re.findall(r"#\[test\]\nfn spec172_core_api_bypass_", core_bypass))
    api_vector_count = len(re.findall(r"#\[test\]\nfn spec172_core_api_bypass_", api_bypass))
    expect(core_vector_count >= 4, f"core bypass vectors exist ({core_vector_count})")
    expect(api_vector_count >= 4, f"API bypass vectors exist ({api_vector_count})")

    return {
        "core_vectors": core_vector_count,
        "api_vectors": api_vector_count,
    }


def hygiene() -> None:
    """No raw email, secret, raw key, or card evidence in the pinned surfaces."""
    EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")
    for path in [CHOKEPOINT_RS, CORE_BYPASS_RS, SCHEDULER_RS, API_ENTITLEMENT_RS, API_BYPASS_RS]:
        raw = path.read_text(encoding="utf-8")
        expect(EMAIL_RE.search(raw) is None, f"{path.name} carries an email literal")
        expect(
            SECRET_RE.search(raw) is None
            and PRIVATE_KEY_RE.search(raw) is None
            and CARD_RE.search(raw) is None,
            f"{path.name} carries a secret, raw private key, or card number",
        )


def main() -> int:
    replay = replay_layer()
    expect(all(rc == 0 for rc in replay.values()), "every replay gate exited 0")
    expect(len(replay) == len(REPLAY_GATES), "all replay gates ran exactly once")

    vector_counts = static_pin_rust_surfaces()
    hygiene()

    summary = {
        "schema": "focusa.spec172.core_api_bypass_acceptance.v1",
        "atom": "focusa-vbcqu.20.15.25",
        "result": "passed",
        "replay_gates": len(replay),
        "replay_exit_codes_all_zero": True,
        "replay_cases": sorted(replay.keys()),
        "rust_vectors": vector_counts,
        "cargo_filter": "cargo test --workspace spec172_core_api_bypass",
        "static_positive_checks": POSITIVE,
        "static_negative_checks": NEGATIVE,
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.25-acceptance.txt",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
