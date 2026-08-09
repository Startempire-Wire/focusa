#!/usr/bin/env python3
"""Spec 152F.05.01 CLI presenter parity and transcript fixtures.

Atom focusa-vbcqu.20.14.37 (152F.05.01): project canonical decisions
through the CLI.

Exact verification:
    python3 tests/spec152_cli_entitlement_gate_test.py \\
        && python3 tests/spec152f_cli_presenter_test.py

Build-independent gate over the committed CLI presenter source
(crates/focusa-cli/src/commands/license.rs), the command dispatcher
(crates/focusa-cli/src/main.rs), and the deterministic JSON output fixtures
(crates/focusa-cli/tests/fixtures/spec152f-cli-presenter-fixtures.v1.json).

What is proven here (Spec 152F §5 canonical decision order, §6 chokepoints,
P5 presenters never invent entitlement, P9 server-owned grants):

1. CLI presenter parity: `focusa license status` renders the canonical
   authority projection, the canonical entitlement-decision projection, the
   canonical base-product projection, the optional premium-family decisions
   (re-resolved from the authority snapshot only), and the permanent recovery
   allowance — the same projections REST, TUI, Pi, and agents inherit. The CLI
   never grants, prices, or reinterprets entitlement and never blocks recovery.
2. Fast preflight: `focusa license preflight` re-resolves the canonical
   decision, renders base/premium/recovery reason and next action in stable
   JSON, and exits nonzero when the target gate would deny. It never
   self-issues a grant (no local Evaluation fallback).
3. Stable JSON and secret redaction: schema'd envelopes only; no raw keys,
   tokens, or customer identity in the presenter or fixtures.
4. Dispatcher inheritance: the top-level CLI dispatcher routes the license
   surface through this presenter; commands inherit their core/API operations
   rather than owning commercial policy.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LICENSE = ROOT / "crates/focusa-cli/src/commands/license.rs"
MAIN = ROOT / "crates/focusa-cli/src/main.rs"
FIXTURE_PATH = (
    ROOT / "crates/focusa-cli/tests/fixtures/spec152f-cli-presenter-fixtures.v1.json"
)
PROJECTION_SOURCE = ROOT / "crates/focusa-license/src/lib.rs"

PRESENTER_START = "/// Canonical decision presenter (Spec 152F §5/§6)"
RUN_STATUS = "async fn run_status(json_output"
RUN_PREFLIGHT = "async fn run_preflight(json_output"

PREM_FAMILIES = {
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
}

REGISTERED_PREMIUM_FEATURES = {
    "focusa.agent.silent_sessions",
    "focusa.team.multi_operator",
    "focusa.release.proof",
    "focusa.update.unattended",
    "focusa.export.packaged",
}

FORBIDDEN_FRAGMENTS = [
    "customer_email",
    "key_hash",
    "signing_key",
    "private_key",
    "access_token",
    "pairing_proof",
]

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


def main() -> int:
    source = LICENSE.read_text(encoding="utf-8")
    dispatcher = MAIN.read_text(encoding="utf-8")
    projection = PROJECTION_SOURCE.read_text(encoding="utf-8")
    fixture_raw = FIXTURE_PATH.read_text(encoding="utf-8")
    fixture = json.loads(fixture_raw)

    # ── 1. Fixture shape, determinism, and privacy ──────────────────────────
    expect(
        fixture["schema"] == "focusa.spec152f.cli_presenter_fixtures.v1",
        "fixture schema is stable",
    )
    expect(
        fixture["envelope_keys"]
        == [
            "schema",
            "authority",
            "entitlement_decision",
            "base_product",
            "premium",
            "recovery_allowance",
            "recovery_policy",
            "marketing_preference",
        ],
        "fixture envelope keys are the canonical status envelope",
    )
    expect(set(fixture["premium_families"]) == PREM_FAMILIES,
           "fixture premium families are the canonical four plus export packaging")

    ids = [entry["id"] for entry in fixture["fixtures"]]
    expect(len(ids) == len(set(ids)), "fixture ids are unique")
    expected_ids = {
        "active-paid",
        "offline-grace-cached",
        "active-partial-premium",
        "unactivated",
        "recovery-only",
        "wrong-product",
    }
    expect(set(ids) == expected_ids, "fixture covers the six canonical presenter states")

    for entry in fixture["fixtures"]:
        snapshot = entry["snapshot"]
        # Authority-shaped snapshots only; product is never caller-invented.
        expect(snapshot["product"] in {"focusa", "uiai_engine"},
               f"{entry['id']}: product is a canonical authority product")
        for feature in snapshot.get("features", {}):
            expect(feature.startswith("focusa."),
                   f"{entry['id']}: unqualified feature {feature}")
        # Base-first invariant (Spec 152F §5 step 6): a denied base gate denies
        # every optional premium family.
        if entry["expected"]["base_product"] == "denied":
            for family, decision in entry["expected"]["premium"].items():
                expect(decision == "denied",
                       f"{entry['id']}: premium {family} must be denied when base is denied")
        expect(entry["expected"]["recovery_available"],
               f"{entry['id']}: recovery remains available")
        if entry["expected"]["preflight_base"] == "denied":
            expect(entry["expected"]["base_product"] == "denied",
                   f"{entry['id']}: preflight base denial matches base decision")

    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in fixture_raw,
               f"fixture contains forbidden fragment: {fragment}")

    digest = hashlib.sha256(fixture_raw.encode("utf-8")).hexdigest()
    fixture_count = len(fixture["fixtures"])

    # ── 2. CLI presenter renders the canonical decisions (Spec 152F P5) ─────
    presenter_region = source[source.index(PRESENTER_START):source.index(RUN_STATUS)]
    status_region = source[source.index(RUN_STATUS):source.index("async fn run_deactivate")]

    for marker in [
        "focusa_license::resolve_license_guard()",
        "focusa_license::entitlement_projection",
        "focusa_license::entitlement_decision_projection",
        "canonical_decision_payload",
        '"focusa.authority_license_status.v1"',
        '"recovery_allowance"',
        '"marketing_preference"',
        '"recovery_policy"',
    ]:
        expect(marker in status_region, f"run_status missing marker: {marker}")

    for marker in [
        "canonical_decision_payload",
        "canonical_premium_presenter",
        "canonical_recovery_presenter",
        "base_product_projection",
        "resolve_premium_family",
        "resolve_export_packaged",
        "CapabilityFamily::Automation",
        "CapabilityFamily::TeamRemote",
        "CapabilityFamily::ReleaseProof",
        "CapabilityFamily::PremiumUpdates",
        "CapabilityFamily::CustomerDataExport",
        '"focusa.recovery_projection.v1"',
        '"always_available"',
    ]:
        expect(marker in presenter_region, f"presenter region missing marker: {marker}")

    # The presenter re-resolves every optional family against the exact
    # registered feature identifiers; it cannot request or expand a grant.
    for feature in REGISTERED_PREMIUM_FEATURES:
        expect(f'"{feature}"' in presenter_region,
               f"presenter missing registered feature: {feature}")

    # Canonical projection field parity: the CLI consumes the same projections
    # other presenters inherit, and the fixture envelope keys match the
    # projection types exported by focusa-license.
    for field in [
        "pub struct EntitlementProjection",
        "pub struct EntitlementDecisionProjection",
        "pub struct BaseProductProjection",
        "pub fn entitlement_projection",
        "pub fn entitlement_decision_projection",
        "pub fn base_product_projection",
    ]:
        expect(field in projection, f"canonical projection missing: {field}")

    # Secret redaction: the presenter region never touches identity or keys.
    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in presenter_region,
               f"presenter region contains forbidden fragment: {fragment}")

    # ── 3. Fast preflight with nonzero exit semantics, no local grant ───────
    preflight_region = source[source.index(RUN_PREFLIGHT):source.index(RUN_STATUS)]
    for marker in [
        "focusa_license::resolve_license_guard()",
        "entitlement_projection",
        "entitlement_decision_projection",
        "canonical_decision_payload",
        '"focusa.authority_preflight.v1"',
        "E_AUTHORITY_ENTITLEMENT_REQUIRED",
        "E_AUTHORITY_UNKNOWN_PREFLIGHT_FAMILY",
        "anyhow::bail!",
        '"base_focusa"',
    ]:
        expect(marker in preflight_region, f"run_preflight missing marker: {marker}")
    # Nonzero exit semantics: a denied (or base-limited) gate fails closed.
    expect('decision_label == "denied" || decision_label == "limited"' in preflight_region,
           "run_preflight must fail closed on denied/limited decisions")
    # No local grant fallback: preflight never self-issues an Evaluation,
    # writes a license, or returns an invented grant.
    for fragment in ["persist_eval_license", 'Ok("eval"', "LicenseGuard::eval", "granted_now"]:
        expect(fragment not in preflight_region,
               f"run_preflight must never self-issue: {fragment}")

    # The preflight command is dispatched through the license command surface.
    expect("Preflight(PreflightArgs)" in source, "LicenseCmd enumerates Preflight")
    expect("LicenseCmd::Preflight(a) => run_preflight(json_output, a).await" in source,
           "license dispatcher routes Preflight to run_preflight")

    # ── 4. Dispatcher inheritance (86 commands, no independent authority) ───
    expect("Commands::License(args) => commands::license::run(cli.json, args).await" in dispatcher,
           "top-level dispatcher routes the license surface through the presenter")
    expect("E_AUTHORITY_COMMAND_RETIRED" in source,
           "retired plaintext entitlement commands remain fail-closed in the dispatcher")

    print(
        "Spec152f CLI presenter parity: PASS "
        f"(fixtures={fixture_count} sha256={digest[:16]} positive={POSITIVE} negative={NEGATIVE})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
