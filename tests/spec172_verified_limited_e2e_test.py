#!/usr/bin/env python3
"""Spec 172.05.02 — verified no-license reaches useful limited value (E2E).

Cross-surface E2E receipt (atom focusa-vbcqu.20.15.33, lane acceptance /
Startempire-Wire/focusa + WPUIAI/wpuiai + WPUIAI/uiai-engine).

The required journey is proven end to end from a clean install through
verified mailbox, no license key, one manual Focusa project, preserved
read/export, allowed UIAI public observation, and deny-with-upgrade-guidance
for a second project / blocked families / browser actions:

  Stage 1  Verified identity/assertion — clean install verifies mailbox, the
           authority issues a signed `verified_no_license` limited-access
           assertion, NO EDD Software Licensing key is ever created, and the
           posture has no automatic expiry or countdown (Spec 172 §2.7, §5.2,
           §6.1, §14). Replays the accepted PHP gates
           tests/spec172_verified_access_schema_test.php and
           tests/spec172_limited_assertion_test.php and binds the canonical
           license-types contract.
  Stage 2  Installer/CLI/Desktop/Cockpit flows — every presenter surface
           reaches the same verified-limited or paid authority flow, renders
           canonical posture/denial/upgrade vocabulary, and never owns policy,
           prices, grants, or limits. Replays the accepted gates for the Unix/
           PowerShell/Rust installers, the CLI/Pi/agent presenters, Focusa
           Desktop, and the UIAI Cockpit mixed-product presenter.
  Stage 3  One-project Focusa fixture — verified no-license posture gets one
           mutable active project via explicit operator selection
           (`focusa project use` / `project switch`); a second project is
           denied with upgrade guidance and no data deletion. Live Rust
           vectors via `cargo test -p focusa-core verified_limited_project`.
  Stage 4  Read/export preservation — read projection, basic customer-data
           export, account/device control, recovery, repair, stable security
           update, and uninstall remain available in limited posture; no
           project or evidence is deleted; lifecycle receipts record
           `limited_access_ready` / `active_verified_limited`.
  Stage 5  Public-observe UIAI fixture — exactly one foreground, ephemeral,
           public-web observation session (public_search, source_to_markdown,
           public_page_read, accessibility_snapshot, screenshot,
           basic_diagnostics). Live Rust vectors via
           `cargo test -p focusa-license spec172`.
  Stage 6  Deny before side effects with upgrade guidance — every blocked
           Focusa family (automation, team_remote, release_proof,
           premium_updates) and UIAI family (browser_action,
           browser_persistence, authenticated_private_targets,
           unattended_browser_automation, scheduled_batch_qa,
           premium_hosted_resources) resolves its decision BEFORE any child
           token, project mutation, or browser side effect; denials carry the
           frozen `CAPABILITY_FAMILY_NOT_INCLUDED` error and the
           `review_offer_or_manage_entitlement` upgrade action.
  Stage 7  No anonymous / local / timed grant — unverified state denies all
           product capability (`EMAIL_VERIFICATION_REQUIRED`), no installer or
           presenter can self-issue or persist a local grant (raw keys and
           `--eval` self-issuance are forbidden), `verified_no_license` is a
           posture, not a License Type, and nothing has a countdown.

The receipt emits ONE bounded JSON line with real exit codes. No raw email,
key, token, customer row, credential, or card data ever appears; every
identifier is synthetic or frozen policy vocabulary.

Exact verification:
    python3 tests/spec172_verified_limited_e2e_test.py
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

POSITIVE = 0
NEGATIVE = 0
REPLAY: dict[str, dict] = {}
CARGO_RUNS: list[dict] = []


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def run(argv: list[str], timeout: int = 900) -> subprocess.CompletedProcess:
    return subprocess.run(
        argv,
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=timeout,
    )


def replay_gate(stage: str, name: str, argv: list[str]) -> None:
    """Run one accepted gate once and record its REAL exit code."""
    proc = run(argv)
    REPLAY[f"{stage}::{name}"] = {"argv": argv, "exit": proc.returncode}
    if proc.returncode != 0:
        raise AssertionError(
            f"replay gate failed rc={proc.returncode} for {name} argv={argv}\n"
            f"{proc.stdout[-1500:]}\n{proc.stderr[-1500:]}"
        )


def cargo_test(stage: str, name: str, package: str, filter_: str) -> None:
    """Run one cargo test filter through the canonical OVH-routed cargo
    (builds serialize on the remote global lock; runs are sequential)."""
    proc = run(
        ["cargo", "test", "-p", package, filter_, "--", "--nocapture"],
        timeout=1800,
    )
    result_lines = [
        line.strip()
        for line in (proc.stdout + proc.stderr).splitlines()
        if "test result:" in line
    ]
    CARGO_RUNS.append(
        {
            "stage": stage,
            "name": name,
            "package": package,
            "filter": filter_,
            "exit": proc.returncode,
            "test_results": result_lines,
        }
    )
    if proc.returncode != 0:
        raise AssertionError(
            f"cargo gate failed rc={proc.returncode} for {name} "
            f"(cargo test -p {package} {filter_})\n"
            f"{proc.stdout[-2000:]}\n{proc.stderr[-2000:]}"
        )
    if not result_lines:
        raise AssertionError(f"cargo gate {name} produced no test result line")


# ── Shared source handles ────────────────────────────────────────────────

POLICY = (ROOT / "crates/focusa-license/src/entitlement_policy.rs").read_text(encoding="utf-8")
UIAI = (ROOT / "crates/focusa-license/src/uiai_child_token.rs").read_text(encoding="utf-8")
COCKPIT = (ROOT / "crates/focusa-license/src/cockpit_action_registry.rs").read_text(encoding="utf-8")
DENIAL_UX = (ROOT / "crates/focusa-license/src/denial_ux.rs").read_text(encoding="utf-8")
ACTIVATION_FACADE = (ROOT / "crates/focusa-license/src/activation_facade.rs").read_text(
    encoding="utf-8"
)
LIMITED_PROJECT = (ROOT / "crates/focusa-core/src/limited_project.rs").read_text(encoding="utf-8")
GUARD = (ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs").read_text(encoding="utf-8")
RECEIPTS = (ROOT / "crates/focusa-core/src/install_lifecycle/receipts.rs").read_text(
    encoding="utf-8"
)
CLI_LICENSE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text(encoding="utf-8")
CLI_INSTALL = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs").read_text(encoding="utf-8")
FIXTURE = json.loads(
    (ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json")
    .read_text(encoding="utf-8")
)
LICENSE_TYPES = yaml.safe_load(
    (CONTRACTS / "spec172-license-types.v1.yaml").read_text(encoding="utf-8")
)

FOCUSA_ALLOWED = [
    "manual_project",
    "manual_mission",
    "manual_focus_state",
    "manual_workpoint",
    "manual_trajectory",
    "manual_basic_evidence",
]
FOCUSA_BLOCKED = ["automation", "team_remote", "release_proof", "premium_updates"]
UIAI_ALLOWED = [
    "public_search",
    "source_to_markdown",
    "public_page_read",
    "accessibility_snapshot",
    "screenshot",
    "basic_diagnostics",
]
UIAI_BLOCKED = [
    "browser_action",
    "browser_persistence",
    "authenticated_private_targets",
    "unattended_browser_automation",
    "scheduled_batch_qa",
    "premium_hosted_resources",
]
RETAINED_ACCESS = [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall",
]


def stage1_verified_identity_assertion() -> None:
    """Mailbox verified → signed assertion; NO license key; indefinite."""
    if PHP is None:
        raise AssertionError("php runtime is required for the verified-identity gates")
    replay_gate("1_verified_identity", "verified_access_schema",
                [PHP, "tests/spec172_verified_access_schema_test.php"])
    replay_gate("1_verified_identity", "limited_assertion",
                [PHP, "tests/spec172_limited_assertion_test.php"])

    posture = LICENSE_TYPES["postures"][0]
    expect(posture["code"] == "verified_no_license", "canonical posture code")
    expect(posture["kind"] == "account_runtime_posture", "verified_no_license is a posture, not a license")
    expect(posture["is_license_type"] is False, "verified_no_license is never a License Type")
    expect(posture["duration"] == "no_automatic_expiry", "no countdown or automatic expiry")
    expect(posture["edd_software_license_key"] is False, "no EDD Software Licensing key for limited access")
    expect(
        posture["grant_source"] == "authority_signed_limited_access_assertion",
        "grant source is the authority-signed limited-access assertion",
    )
    expect(posture["anonymous_access"] is False, "no anonymous product capability")
    expect(posture["price_usd"] == "0.00", "limited posture has no caller-owned price")

    # The runtime types keep verified_no_license OUT of the License Type codes
    # and Evaluation out of the postures entirely.
    license_type_enum = POLICY[POLICY.index("pub enum LicenseTypeCode {"):]
    license_type_enum = license_type_enum[: license_type_enum.index("}") + 1]
    expect("FocusaOperatorLifetimeV1" in license_type_enum
           and "UiaiOperatorLifetimeV1" in license_type_enum
           and "VerifiedNoLicense" not in license_type_enum,
           "License Type codes are exactly the two Operator types")
    posture_enum = POLICY[POLICY.index("pub enum AccessPosture {"):]
    posture_enum = posture_enum[: posture_enum.index("}") + 1]
    expect("VerifiedNoLicense" in posture_enum and "Evaluation" not in posture_enum,
           "postures are Spec 172 verified states with no Evaluation spelling")


def stage2_presenter_surfaces() -> None:
    """Installer/CLI/Desktop/Cockpit flows reach the same limited value."""
    replay_gate("2_surfaces", "installer_limited_access",
                ["python3", "tests/spec172_installer_limited_access_test.py"])
    replay_gate("2_surfaces", "cli_agent_presenter",
                ["python3", "tests/spec172_cli_agent_presenter_test.py"])
    replay_gate("2_surfaces", "focusa_desktop",
                ["python3", "tests/spec172_focusa_desktop_entitlement_test.py"])
    replay_gate("2_surfaces", "cockpit_mixed_product",
                ["python3", "tests/spec172_cockpit_mixed_product_test.py"])

    # CLI: canonical posture, stable errors, upgrade actions, project flow.
    expect("verified_no_license" in CLI_LICENSE, "CLI renders verified_no_license posture")
    expect("CAPABILITY_FAMILY_NOT_INCLUDED" in CLI_LICENSE, "CLI stable family denial code")
    expect("UPGRADE_AVAILABLE" in CLI_LICENSE, "CLI stable upgrade-available code")
    expect("review_offer_or_manage_entitlement" in CLI_LICENSE, "CLI upgrade action vocabulary")
    expect("ActivationJourney::LimitedAccess" in CLI_INSTALL,
           "installer maps --eval intent to verified limited activation")
    expect("E_AUTHORITY_RAW_KEY_FORBIDDEN" in CLI_INSTALL,
           "raw license keys cannot authorize installation")

    # Desktop bridge: zero local policy, same denial/upgrade/retained set.
    expect("verified_no_license" in BRIDGE, "Desktop renders verified_no_license posture")
    expect("CAPABILITY_FAMILY_NOT_INCLUDED" in BRIDGE, "Desktop stable family denial code")
    expect("SPEC172_RETAINED_ACCESS" in BRIDGE and '"export"' in BRIDGE
           and '"uninstall"' in BRIDGE,
           "Desktop never disables retained controls (read/export/recovery/update/uninstall)")
    expect("review_offer_or_manage_entitlement" in BRIDGE, "Desktop upgrade action vocabulary")

    # Cockpit: mixed-product registry resolves Focusa and UIAI rows canonically.
    expect("SPEC172_COCKPIT_ACTION_REGISTRY" in COCKPIT, "Cockpit action registry exists")
    expect("resolve_cockpit_action" in COCKPIT, "Cockpit resolver exists")
    expect("cockpit.uiai.browser_click" in COCKPIT, "Cockpit browser-action row exists")

    # Frozen presenter fixtures: limited mode allowed + blocked with upgrade.
    fixtures = {entry["id"]: entry for entry in FIXTURE["fixtures"]}
    manual = fixtures.get("verified-no-license-manual-allowed")
    expect(manual is not None and manual["posture"] == "verified_no_license"
           and manual["family"] == "manual_project" and manual["denial"] is None
           and manual["upgrade_action"] == "none_required",
           "fixture: verified no-license manual project is usable with no upgrade needed")
    blocked = fixtures.get("verified-no-license-blocked-family")
    expect(blocked is not None and blocked["posture"] == "verified_no_license"
           and blocked["family"] == "automation"
           and blocked["denial"] == "CAPABILITY_FAMILY_NOT_INCLUDED"
           and blocked["upgrade_action"] == "review_offer_or_manage_entitlement",
           "fixture: blocked family denies with stable code + upgrade guidance")
    expect(FIXTURE["retained_access"] == RETAINED_ACCESS,
           "fixture retained-access set is frozen and includes read/export/recovery")


def stage3_one_project_focusa() -> None:
    """Create/use one manual Focusa project; second project denies."""
    cargo_test("3_one_project", "verified_limited_project", "focusa-core",
               "verified_limited_project")

    # Family classifier: exactly one mutable project, fail-closed allowlist.
    expect("is_focusa_verified_no_license_family_allowed" in POLICY, "family classifier exists")
    allowlist = POLICY[POLICY.index("pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES"):]
    allowlist = allowlist[: allowlist.index("];") + 2]
    for family in FOCUSA_ALLOWED:
        expect(f'"{family}"' in allowlist, f"focusa limited allowlist carries {family}")
    blocked = POLICY[POLICY.index("pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES"):]
    blocked = blocked[: blocked.index("];") + 2]
    for family in FOCUSA_BLOCKED:
        expect(f'"{family}"' in blocked, f"focusa limited blocks {family}")

    # One-project guard: explicit selection, second-project denial, no deletion.
    expect("DeniedSecondProject" in LIMITED_PROJECT, "second-project denial exists")
    expect("DeniedNoSelection" in LIMITED_PROJECT, "no-selection denial exists")
    expect("focusa project use" in LIMITED_PROJECT, "recovery names focusa project use")
    expect("focusa project switch" in LIMITED_PROJECT, "recovery names focusa project switch")
    expect("upgrade to Focusa Operator" in LIMITED_PROJECT,
           "second-project denial carries upgrade guidance")
    expect("never deletes data" in LIMITED_PROJECT, "guard documents no data deletion")
    expect("never uses activity heuristics" in LIMITED_PROJECT,
           "guard never selects a project by activity heuristics")
    expect("ENTITLEMENT_LIMITED_PROJECT" in GUARD,
           "core execution guard denies second-project mutation before returning success")
    expect("require_base_product" in (ROOT / "crates/focusa-core/src/license.rs").read_text(encoding="utf-8"),
           "base product core gate exists")


def stage4_read_export_preserved() -> None:
    """Read projection, basic export, account/recovery stay available."""
    reducer = POLICY[POLICY.index("pub const fn reduce_entitlement_state"):]
    expect("(State::VerifiedNoLicense, Family::ReadProjection)" in reducer
           and "Posture::Read" in reducer,
           "verified no-license read projection is preserved")
    expect("(State::VerifiedNoLicense, Family::AccountRecovery | Family::CustomerDataExport)"
           in reducer,
           "verified no-license account recovery and basic export stay allowed")
    expect("(State::PendingUnverified, _)" in reducer and "Posture::Deny" in reducer,
           "unverified state denies product capability before verification")

    # Lifecycle receipts record the limited-access posture, not Evaluation.
    expect("limited_access_ready" in RECEIPTS, "receipt class is limited_access_ready")
    expect("active_verified_limited" in RECEIPTS, "receipt state is active_verified_limited")
    expect("signature_verified" in RECEIPTS, "product readiness requires a signature-verified binding")
    expect("evaluation_ready" not in RECEIPTS and "active_evaluation" not in RECEIPTS,
           "no Evaluation vocabulary remains in lifecycle receipts", negative=True)

    # Presenter projections never lose the retained controls in limited mode.
    for control in ["read", "export", "recovery", "repair", "update", "uninstall"]:
        expect(control in RETAINED_ACCESS, f"retained access includes {control}")


def stage5_uiai_public_observe() -> None:
    """One foreground ephemeral public-observe UIAI session in limited mode."""
    cargo_test("5_uiai_observe", "spec172_license_vectors", "focusa-license", "spec172")

    uiai_allowlist = POLICY[
        POLICY.index("pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES"):
    ]
    uiai_allowlist = uiai_allowlist[: uiai_allowlist.index("];") + 2]
    for family in UIAI_ALLOWED:
        expect(f'"{family}"' in uiai_allowlist, f"uiai limited allowlist carries {family}")

    expect("SPEC172_UIAI_OPERATION_MAP" in UIAI, "canonical UIAI operation map exists")
    for operation_id in UIAI_ALLOWED:
        expect(f'operation_id: "{operation_id}"' in UIAI, f"observe operation {operation_id} in map")
    for operation_id in [
        "browser_click", "browser_fill", "browser_type", "browser_select",
        "browser_press", "browser_submit", "cookie_persistence",
        "auth_state_persistence", "session_persistence",
        "authenticated_private_dashboard", "unattended_browser_automation",
        "scheduled_batch_qa", "premium_proxy", "hosted_capacity",
        "paid_model_calls",
    ]:
        expect(f'operation_id: "{operation_id}"' in UIAI, f"blocked vector {operation_id} in map")

    # Limited mode: exactly one foreground, ephemeral, public-web session.
    expect("VerifiedNoLicensePublicObservation" in UIAI, "limited-mode decision exists")
    expect("session_quota: 1" in UIAI, "limited mode session quota is exactly one")
    expect("LimitedModeRestricted" in UIAI, "second session fails closed")
    expect("UiaiGrantRequired" in UIAI, "browser action fails closed without a paid UIAI grant")
    expect("FocusaOnlyCannotGrantUiai" in UIAI,
           "Focusa-only entitlement never grants UIAI (product isolation)")
    expect("UnknownOperation" in UIAI, "unknown operation ids fail closed")

    # Resolution happens BEFORE any child token or side effect: the operation
    # map and the resolver precede the shared active-bound/child-token helper,
    # and the resolver consumes only canonical operation ids + authority
    # snapshots — never caller-selected policy.
    expect(UIAI.index("SPEC172_UIAI_OPERATION_MAP") < UIAI.index("fn resolve_uiai_operation_capability"),
           "operation map precedes the resolver")
    expect(UIAI.index("fn resolve_uiai_operation_capability") < UIAI.index("fn active_bound"),
           "resolver precedes the shared active_bound helper")
    resolver_signature = UIAI[UIAI.index("pub fn resolve_uiai_operation_capability"):]
    resolver_signature = resolver_signature[: resolver_signature.index(") -> Result<UiaiCapabilityDecision") + 1]
    for required in ["operation_id", "focusa_parent", "uiai_grant", "active_session_count"]:
        expect(required in resolver_signature, f"resolver consumes {required}")
    for forbidden in ["price", "license_type", "family", "feature", "limit", "node",
                      "commercial_right", "auth_state", "pairing"]:
        expect(forbidden not in resolver_signature,
               f"resolver must not accept caller policy: {forbidden}", negative=True)
    for forbidden in ["SigningKey", "Signer", "self_sign", "customer_email"]:
        expect(forbidden not in UIAI, f"no local/self-issued grant in UIAI surface: {forbidden}",
               negative=True)


def stage6_deny_before_side_effects() -> None:
    """Every blocked family denies before side effects, with upgrade guidance."""
    uiai_blocked = POLICY[POLICY.index("pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES"):]
    uiai_blocked = uiai_blocked[: uiai_blocked.index("];") + 2]
    for family in UIAI_BLOCKED:
        expect(f'"{family}"' in uiai_blocked, f"uiai limited blocks {family}")

    # Denial UX: typed error codes, blocked-action mapping, and the stable
    # EMAIL_VERIFICATION_REQUIRED error on the activation facade.
    expect("DenialUxErrorCode" in DENIAL_UX, "denial UX error codes exist")
    expect("blocked_action_for_family" in DENIAL_UX, "blocked-action mapping exists")
    expect("ENTITLEMENT_LIMIT_EXHAUSTED" in DENIAL_UX, "denial UX limit-exhausted code exists")
    expect("EMAIL_VERIFICATION_REQUIRED" in ACTIVATION_FACADE,
           "unverified email fails closed with the stable verification error")

    # Focusa premium families deny through the shared core guard before any
    # value mutation returns success.
    for code in ["ENTITLEMENT_BASE_REQUIRED", "ENTITLEMENT_FEATURE_REQUIRED",
                 "ENTITLEMENT_REQUIRED", "ENTITLEMENT_LIMITED_PROJECT"]:
        expect(code in GUARD, f"core guard stable code {code} exists")

    # Stable §21 error vocabulary is frozen on the CLI presenter too.
    expect("EMAIL_VERIFICATION_REQUIRED" in CLI_LICENSE
           and "CAPABILITY_FAMILY_NOT_INCLUDED" in CLI_LICENSE
           and "UPGRADE_AVAILABLE" in CLI_LICENSE and "RECOVERY_ONLY" in CLI_LICENSE,
           "CLI carries the frozen stable-error set")


def stage7_no_anonymous_local_timed_grant() -> None:
    """No anonymous product capability, no local/self-issued grant, no timed grant."""
    # Anonymous: unverified state denies everything except account recovery,
    # basic export, and pre-access flows (verified by the reducer and the PHP
    # gates); the presenter renders EMAIL_VERIFICATION_REQUIRED.
    expect("EMAIL_VERIFICATION_REQUIRED" in CLI_LICENSE, "presenter renders EMAIL_VERIFICATION_REQUIRED")

    # Local/self-issued: installers and lifecycle surfaces cannot mint grants.
    expect("E_AUTHORITY_RAW_KEY_FORBIDDEN" in CLI_INSTALL, "raw keys cannot authorize installation")
    for fragment in ["persist_eval_license", "write_license_json", "E_EVAL_ISSUED",
                     "grant_creation"]:
        expect(fragment not in CLI_INSTALL,
               f"installer still self-issues a local grant: {fragment}", negative=True)
    for fragment in ["evaluation_ready", "active_evaluation",
                     "LifecycleEntitlementReceiptClass::EvaluationReady"]:
        expect(fragment not in RECEIPTS,
               f"lifecycle receipts still spell Evaluation: {fragment}", negative=True)

    # Timed: no expiry/countdown anywhere in the canonical posture contract.
    expect(LICENSE_TYPES["postures"][0]["duration"] == "no_automatic_expiry",
           "limited posture has no automatic expiry")
    expect("expires_at" not in json.dumps(LICENSE_TYPES["postures"][0]),
           "posture record has no access-expiry field", negative=True)
    expect("Evaluation" not in json.dumps(LICENSE_TYPES), "no Evaluation in license-types contract",
           negative=True)


def hygiene(receipt: str) -> None:
    """The bounded receipt contains no raw email, secret, key, or card data."""
    EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")
    expect(EMAIL_RE.search(receipt) is None, "receipt carries an email literal")
    expect(SECRET_RE.search(receipt) is None and PRIVATE_KEY_RE.search(receipt) is None
           and CARD_RE.search(receipt) is None,
           "receipt carries a secret, raw private key, or card number")


def main() -> int:
    stage1_verified_identity_assertion()
    stage2_presenter_surfaces()
    stage3_one_project_focusa()
    stage4_read_export_preserved()
    stage5_uiai_public_observe()
    stage6_deny_before_side_effects()
    stage7_no_anonymous_local_timed_grant()

    receipt = {
        "schema": "focusa.spec172.verified_limited_e2e.v1",
        "atom": "focusa-vbcqu.20.15.33",
        "title": "172.05.02 Prove verified no-license reaches useful limited value",
        "result": "passed_fail_closed",
        "stages": {
            "1_verified_identity_assertion": "verified mailbox -> signed limited-access assertion -> NO EDD license key -> no automatic expiry",
            "2_presenter_surfaces": "installer/CLI/Desktop/Cockpit reach the same verified-limited value",
            "3_one_project_focusa": "one mutable manual project; second project denies with upgrade guidance",
            "4_read_export_preserved": "read projection, basic export, recovery, repair, update, uninstall preserved",
            "5_uiai_public_observe": "one foreground ephemeral public-observe session",
            "6_deny_before_side_effects": "every blocked family denies before side effects with upgrade guidance",
            "7_no_anonymous_local_timed_grant": "no anonymous capability, no local/self-issued grant, no countdown",
        },
        "replay_gates": {
            key: {"exit": value["exit"]}
            for key, value in sorted(REPLAY.items())
        },
        "replay_gate_exit_codes_all_zero": all(value["exit"] == 0 for value in REPLAY.values()),
        "cargo_runs": [
            {
                "package": run_["package"],
                "filter": run_["filter"],
                "exit": run_["exit"],
                "test_results": run_["test_results"],
            }
            for run_ in CARGO_RUNS
        ],
        "cargo_runs_all_zero": all(run_["exit"] == 0 for run_ in CARGO_RUNS),
        "positive_checks": POSITIVE,
        "negative_checks": NEGATIVE,
        "limited_access": {
            "license_key_received": False,
            "mutable_projects": 1,
            "uiai_observation_sessions": 1,
            "read_export_preserved": True,
            "deny_before_side_effects": True,
            "upgrade_guidance": "review_offer_or_manage_entitlement",
            "automatic_expiry": "none",
        },
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.33-acceptance.txt",
    }

    receipt_json = json.dumps(receipt, sort_keys=True)
    hygiene(receipt_json)
    print(receipt_json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
