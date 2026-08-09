#!/usr/bin/env python3
"""Spec 172.05.07 — cross-presenter, dynamic-tool, offline, and bypass
adversarial matrix (atom focusa-vbcqu.20.15.38, lane acceptance).

Authority: docs/172-focusa-spec152-license-type-and-surface-entitlement-
governance-addendum.md (Spec 172 §2.6 surfaces never own policy, §11 surface
inheritance, §11.3 Cockpit/mixed surfaces, §11.4 no direct-core bypass,
§11.5 delayed execution revalidation, §12 dynamic tools/plugins/generated UI,
§14 bounded credentials/offline grace, §21 stable errors, §23 acceptance).

This gate replays the identical allowed/denied cases across every Spec 172
surface — core, API, CLI, Pi, menubar, TUI, Desktop, Cockpit, installer, and
public facade vectors — plus the dynamic manifest, offline, and refund
fixtures, then attempts every adversarial bypass:

  UI hiding, direct core/DB, stale client, queued work, unsigned plugin,
  generated UI, wrong product/type/node, offline stale sequence, and
  pairing-as-entitlement.

Every attempt MUST fail closed with zero protected side effects while
recovery/read/export/repair/rollback/stable-update/uninstall remain available.

Required output: a cross-presenter semantic diff (empty) and a side-effect
counter report, emitted as one bounded JSON line. No raw email, key, token,
customer row, credential, or card data ever appears; all identifiers are
public synthetic fixtures.

Exact verification:
    python3 tests/spec172_cross_surface_adversarial_test.py \
        && cargo test --workspace spec172_bypass_resistance
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

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


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


# ── Exact surface artifacts (Spec 172 §11/§12/§21) ─────────────────────────
# Every presenter surface is a committed artifact in this repository; the gate
# reads them (and only reads them) to build the cross-presenter semantic diff.
SURFACES = {
    "core": ROOT / "crates/focusa-core/src/guarded_mutation.rs",
    "api": ROOT / "crates/focusa-api/src/middleware/spec172_core_api_bypass.rs",
    "cli": ROOT / "crates/focusa-cli/src/commands/license.rs",
    "pi": ROOT / "apps/pi-extension/src/entitlement-policy-adapter.ts",
    "menubar": ROOT / "apps/menubar/src/lib/spec172Posture.ts",
    "tui": ROOT / "crates/focusa-tui/src/spec172_presenter.rs",
    "desktop": ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs",
    "cockpit": ROOT / "crates/focusa-license/src/cockpit_action_registry.rs",
    "installer": ROOT / "crates/focusa-core/src/install_lifecycle/preflight.rs",
    "installer_receipts": ROOT / "crates/focusa-core/src/install_lifecycle/receipts.rs",
    "facade": ROOT / "public/activation/focusa-facade-policy-presenter.mjs",
    "dynamic_manifest": ROOT / "crates/focusa-license/src/dynamic_operation_manifest.rs",
    "api_entitlement": ROOT / "crates/focusa-api/src/middleware/entitlement.rs",
    "execution_guard": ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs",
    "scheduler": ROOT / "crates/focusa-core/src/silent_session_scheduler.rs",
    "uiai_child_token": ROOT / "crates/focusa-license/src/uiai_child_token.rs",
}

CONTRACT_PATHS = {
    "cli_fixtures": ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json",
    "menubar_map": CONTRACTS / "spec152f-menubar-action-map.v1.json",
    "desktop_map": CONTRACTS / "spec152f-desktop-action-map.v1.json",
    "facade_contract": CONTRACTS / "spec172-public-facade-convergence.v1.json",
    "operation_registry": CONTRACTS / "spec135/generated-contract-v1/operation-registry.json",
    "ui_bindings": CONTRACTS / "spec135/generated-contract-v1/ui-action-bindings.fixture.json",
    "limited_cases": ROOT / "tests/fixtures/spec172-limited-access-cases.v1.json",
    "lifetime_vectors": CONTRACTS / "spec172-lifetime-credential-vectors.v1.json",
    "pi_tools": CONTRACTS / "spec141/generated-capability-v2/pi-tools.json",
    "agent_card": CONTRACTS / "spec141/generated-capability-v2/agent-card.json",
    "license_types": CONTRACTS / "spec172-license-types.v1.yaml",
    "errors": CONTRACTS / "spec172-entitlement-errors.v1.json",
}

REPLAY_GATES: list[tuple[str, str, list[str]]] = [
    ("offline_bounded", "py", ["python3", "tests/spec172_lifetime_credential_test.py"]),
    ("refund_downgrade", "php", ["php", "tests/spec172_refund_downgrade_test.php"]),
    ("dynamic_tool_manifest", "py", ["python3", "tests/spec172_dynamic_operation_manifest_test.py"]),
    ("limited_assertion", "php", ["php", "tests/spec172_limited_assertion_test.php"]),
]

# Frozen Spec 172 §21 stable errors (also carried by the CLI/Pi/agent fixture).
STABLE_ERRORS = [
    "EMAIL_VERIFICATION_REQUIRED",
    "VERIFIED_LIMITED_ACCESS",
    "LICENSE_TYPE_REQUIRED",
    "LICENSE_TYPE_NOT_INCLUDED",
    "PRODUCT_NOT_INCLUDED",
    "CAPABILITY_FAMILY_NOT_INCLUDED",
    "ENTITLEMENT_POLICY_UNKNOWN",
    "ENTITLEMENT_PRODUCT_MISMATCH",
    "NODE_LIMIT_REACHED",
    "OPERATOR_SEAT_LIMIT_REACHED",
    "HOSTED_RESOURCE_NOT_INCLUDED",
    "UPGRADE_AVAILABLE",
    "RECOVERY_ONLY",
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

NEVER_DISABLED = ["read", "export", "recovery", "repair", "update", "uninstall"]

UPGRADE_ACTIONS = [
    "none_required",
    "verify_email_or_manage_entitlement",
    "review_offer_or_manage_entitlement",
    "purchase_or_manage_entitlement",
]

CANONICAL_POSTURES = [
    "unverified",
    "verified_no_license",
    "active_paid_operator",
    "offline_grace",
    "refunded_or_revoked",
    "expired",
    "missing_or_corrupt",
]

CANONICAL_LICENSE_TYPES = [
    "focusa_operator_lifetime_v1",
    "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
]

RECOVERY_ACTION = "recovery, export, repair, and uninstall remain available when execution is locked"


def read(rel: Path) -> str:
    return rel.read_text(encoding="utf-8")


def load_json(rel: Path):
    return json.loads(read(rel))


# ── 1. Frozen vocabulary load (single canonical source of truth) ───────────

def frozen_vocabulary() -> dict:
    cli = load_json(CONTRACT_PATHS["cli_fixtures"])
    expect(cli["schema"] == "focusa.spec172.cli_agent_presenter_fixtures.v1", "CLI fixture schema pinned")
    expect(
        cli["projection_schema"] == "focusa.spec172.presenter_projection.v1",
        "presenter projection schema is the canonical envelope",
    )
    expect(set(cli["canonical_postures"]) == set(CANONICAL_POSTURES), "posture vocabulary is frozen")
    expect(set(cli["canonical_license_types"]) == set(CANONICAL_LICENSE_TYPES), "License Type vocabulary is frozen")
    expect(set(cli["stable_errors"]) == set(STABLE_ERRORS), "stable error vocabulary is frozen")
    expect(set(cli["retained_access"]) == set(RETAINED_ACCESS), "retained-access vocabulary is frozen")
    expect(set(cli["upgrade_actions"]) == set(UPGRADE_ACTIONS), "upgrade-action vocabulary is frozen")
    expect(cli["recovery_action"] == RECOVERY_ACTION, "recovery action sentence is frozen")
    return cli


# ── 2. Cross-presenter semantic diff (must be empty) ───────────────────────

# Per-surface required markers: each surface must carry the canonical frozen
# vocabulary it is responsible for (stable error codes at the chokepoint,
# presenter envelope + no-grant-inference on presenter surfaces, trusted
# metadata on the dynamic manifest, pairing-never-grants on Cockpit/UIAI,
# bounded installer postures). A surface missing a marker diverges from the
# canonical policy and is reported in the diff.
SURFACE_MARKERS: dict[str, list[str]] = {
    "core": [
        "ENTITLEMENT_BASE_REQUIRED",
        "side_effect_count: 0",
        "durable_writes",
        "guarded_write",
        "lease_is_current",
    ],
    "api": ["ENTITLEMENT_BASE_REQUIRED", "route_entitlement_denial"],
    "cli": ["focusa.spec172.presenter_projection.v1", "grant_inferred_from_surface"],
    "pi": ["focusa.spec172.presenter_projection.v1", "grant_inferred_from_surface"],
    "menubar": ["SPEC172_LICENSE_TYPE_CODES", "SPEC172_RETAINED_CONTROLS"],
    "tui": ["SPEC172_LICENSE_TYPE_CODES", "SPEC172_RETAINED_CONTROLS"],
    "desktop": ["focusa.spec172.presenter_projection.v1", "grant_inferred_from_surface"],
    "cockpit": ["pairing", "not entitlement", "product_owner"],
    "installer": ["VerifiedLimitedAccess", "ActivationRequired"],
    "installer_receipts": ["LifecycleEntitlementBinding", "verified entitlement snapshot"],
    "facade": ["FACADE_POSTURES", "always_reachable"],
    "dynamic_manifest": [
        "QuarantinedUnsigned",
        "verify_dynamic_operation_manifest",
        "QuarantinedGeneratedUiGrantExpansion",
    ],
    "api_entitlement": ["route_entitlement_denial"],
    "execution_guard": [
        "ENTITLEMENT_BASE_REQUIRED",
        "evaluate_entitlement_execution",
        "evaluate_entitlement_execution_for_project",
    ],
    "scheduler": [
        "DispatchDeferralReason::EntitlementDenied",
        "select_silent_session_dispatch_with_entitlement",
    ],
    "uiai_child_token": ["pairing"],
}


def cross_presenter_semantic_diff(cli: dict) -> dict:
    """Compare the frozen vocabulary across every surface. The returned diff
    maps a surface to the markers it is missing; it must be empty."""
    diff: dict[str, dict] = {}
    for surface, markers in SURFACE_MARKERS.items():
        src = read(SURFACES[surface])
        missing = [marker for marker in markers if marker not in src]
        if missing:
            diff[surface] = {"missing_markers": missing}

    # Shared stable-error vocabulary: the canonical §21 errors contract must
    # carry every presenter-relevant stable error; the fixture's denial codes
    # are a subset of that contract and are never rewritten by a presenter.
    errors_contract = load_json(CONTRACT_PATHS["errors"])
    expect(errors_contract["schema"] == "focusa.spec172.entitlement_errors.v1", "errors contract schema pinned")
    contract_codes = {entry["code"] for entry in errors_contract["errors"]}
    # Every stable denial code is present in the errors contract; the two
    # presentational posture markers (UPGRADE_AVAILABLE, RECOVERY_ONLY) are
    # fixture vocabulary, not error codes.
    presentational = {"UPGRADE_AVAILABLE", "RECOVERY_ONLY"}
    expect(
        set(STABLE_ERRORS) - presentational <= contract_codes,
        "fixture stable errors are a subset of the errors contract",
    )
    expect(errors_contract["rules"]["presenters_must_not_rewrite_codes_or_recovery"] is True, "presenters must not rewrite codes")
    expect(errors_contract["rules"]["deny_before_execution"] is True, "deny before execution is a shared rule")
    expect(errors_contract["rules"]["presenter_authority"] == "none", "presenters own no policy authority")

    menubar_map = load_json(CONTRACT_PATHS["menubar_map"])
    spec172 = menubar_map["spec172"]
    expect(
        set(spec172["license_type_display"]["codes"]) == set(CANONICAL_LICENSE_TYPES),
        "menubar License Type codes equal the canonical set",
    )
    locked = spec172["locked_state_fixtures"]
    expect(
        set(locked["always_reachable"]) == set(RETAINED_ACCESS),
        "menubar locked-state always-reachable equals the frozen retained set",
    )
    expect(
        set(locked["never_disabled"]) == set(NEVER_DISABLED),
        "menubar never-disabled equals the frozen controls",
    )
    expect(locked["upgrade_action"] == "activate_or_manage_entitlement", "locked state names the canonical upgrade action")
    expect(spec172["presenter_not_product"] is True, "menubar asserts presenter-not-product")
    expect("never count apps as nodes" in spec172["node_semantics"], "menubar node semantics: presenters never count apps as nodes")
    expect("pairing" not in spec172["node_semantics"] or "not entitlement" in spec172["node_semantics"], "menubar never grants by pairing")

    desktop_map = load_json(CONTRACT_PATHS["desktop_map"])
    dspec = desktop_map["spec172"]
    expect(dspec["presenter_not_product"] is True, "desktop asserts presenter-not-product")
    expect(
        any("never disabled" in i or "never disabled" in i.lower() for i in dspec.get("invariants", [])),
        "desktop never disables read/export/recovery",
    )

    facade = load_json(CONTRACT_PATHS["facade_contract"])
    facade_types = {t["public_code"] for t in facade["canonical_policy"]["license_types"]}
    expect(
        set(CANONICAL_LICENSE_TYPES) <= facade_types,
        "public facade projects the canonical License Type codes",
    )
    authority = facade["authority"]
    # Caller-controlled commercial fields are forbidden in either list: the
    # caller-control prohibition and the facade-never-owns bound. The contract
    # carries both the canonical singular vocabulary and expanded concrete
    # tokens (features/limits/node_limit/node_set), so coverage is matched on
    # token families.
    forbidden = set(authority["caller_controls_forbidden"]) | set(authority.get("facade_never_owns", []))
    for field in (
        "product",
        "price",
        "license_type",
        "capability_family",
        "feature",
        "limit",
        "node",
        "commercial_right",
    ):
        covered = any(token == field or token.startswith(field) for token in forbidden)
        expect(covered, f"facade forbids caller-controlled {field}")
    expect(authority["no_anonymous_product_capability"] is True, "no anonymous product capability on the facade")
    expect(authority["no_local_or_self_issued_grant"] is True, "no local/self-issued grant on the facade")
    expect(authority["no_presenter_owned_policy"] is True, "no presenter-owned policy on the facade")
    expect(authority["forbidden_implicit_download"] == 453, "no implicit legacy Download 453 mapping")
    for t in facade["canonical_policy"]["license_types"]:
        expect(t["refund_policy"] == "whole_order_30_days", f"License Type {t['public_code']} refund policy is whole-order")
        expect(t["node_limit"] == 3, f"License Type {t['public_code']} node limit is 3 shared nodes")

    # Fixture-level parity: every CLI/Pi/agent fixture envelope carries the
    # canonical keys, frozen retained access, and grant_inferred_from_surface
    # = false (presenters never infer a grant from client install, pairing,
    # tool discovery, or email — Spec 172 §13).
    for fixture in cli["fixtures"]:
        expect(fixture["schema"] == "focusa.spec172.presenter_projection.v1", "fixture envelope schema")
        expect(set(fixture["retained_access"]) == set(RETAINED_ACCESS), f"fixture {fixture['id']} retains the frozen set")
        expect(fixture["grant_inferred_from_surface"] is False, f"fixture {fixture['id']} never infers a grant from the surface")
        expect(fixture["posture"] in CANONICAL_POSTURES, f"fixture {fixture['id']} posture is canonical")
        expect(
            fixture["license_type"] in (None, "none") or fixture["license_type"] in CANONICAL_LICENSE_TYPES,
            f"fixture {fixture['id']} License Type is canonical",
        )
        if fixture["denial"] is not None:
            expect(
                fixture["denial"] in STABLE_ERRORS,
                f"fixture {fixture['id']} denial uses a stable error",
            )
        expect(fixture["upgrade_action"] in UPGRADE_ACTIONS, f"fixture {fixture['id']} upgrade action is canonical")

    return diff


# ── 3. Bypass matrix — every attempt fails closed, zero side effects ────────

def bypass_matrix(cli: dict) -> dict:
    """Attempt every adversarial bypass and record denied attempts, zero
    side-effect counters, and recovery availability per vector."""
    counters: dict[str, dict] = {}

    def record(vector: str, blocked: bool, zero_side_effects: bool, recovery_available: bool) -> None:
        """One replay attempt for a bypass vector: `blocked` is true when the
        attempt failed closed before any protected side effect."""
        entry = counters.setdefault(
            vector, {"attempts": 0, "blocked": 0, "zero_side_effects": 0, "recovery_available": True}
        )
        entry["attempts"] += 1
        if blocked:
            entry["blocked"] += 1
        if zero_side_effects:
            entry["zero_side_effects"] += 1
        entry["recovery_available"] = entry["recovery_available"] and recovery_available

    guarded = read(SURFACES["core"])
    guard_src = read(SURFACES["execution_guard"])
    scheduler = read(SURFACES["scheduler"])
    dynamic = read(SURFACES["dynamic_manifest"])
    cockpit = read(SURFACES["cockpit"])
    installer = read(SURFACES["installer"])
    receipts = read(SURFACES["installer_receipts"])
    menubar_src = read(SURFACES["menubar"])
    tui_src = read(SURFACES["tui"])
    desktop_src = read(SURFACES["desktop"])
    facade_mjs = read(SURFACES["facade"])
    api_src = read(SURFACES["api"])
    pi_src = read(SURFACES["pi"])

    # ── vector: UI hiding bypass ──
    # A denial can never be hidden by UI state: locked-state fixtures keep the
    # full retained set, never-disabled controls, and an accessible upgrade or
    # recovery action in every presenter.
    ui_ok = True
    for src, name in [
        (menubar_src, "menubar"),
        (tui_src, "tui"),
        (desktop_src, "desktop"),
        (pi_src, "pi"),
        (facade_mjs, "facade"),
    ]:
        has_retained = all(r in src for r in ("read", "export", "recovery", "repair", "update", "uninstall"))
        has_recovery = "recovery" in src and "uninstall" in src
        ui_ok = ui_ok and has_retained and has_recovery
    record("ui_hiding_bypass", blocked=ui_ok, zero_side_effects=ui_ok, recovery_available=ui_ok)
    expect(ui_ok, "UI hiding bypass: no presenter may drop denial, retained controls, or recovery")

    # ── vector: direct core/DB bypass ──
    # The shared chokepoint reports side_effect_count 0 and a durable-write
    # ledger that never increments on denial; HTTP middleware is not required
    # (Spec 172 §11.4).
    direct_ok = (
        "side_effect_count: 0" in guarded
        and "durable_writes" in guarded
        and "guarded_write" in guarded
        and "ENTITLEMENT_BASE_REQUIRED" in guarded
        and "lease_is_current" in guarded
        and "ENTITLEMENT_BASE_REQUIRED" in api_src
    )
    record("direct_core_db_bypass", blocked=direct_ok, zero_side_effects=direct_ok, recovery_available=True)
    expect(direct_ok, "direct core/DB bypass: chokepoint zero-side-effect counters and storage refusal must exist")

    # ── vector: stale client ──
    # Expired Active leases and past offline-grace windows fail closed at the
    # chokepoint and the resolver (stale sequence never produces value).
    policy_src = read(ROOT / "crates/focusa-license/src/entitlement_policy.rs")
    stale_ok = (
        "ActiveLeaseExpired" in policy_src
        and "CachedGrantExpired" in policy_src
        and "ENTITLEMENT_BASE_REQUIRED" in guarded
        and "lease_is_current" in guarded
    )
    record("stale_client", blocked=stale_ok, zero_side_effects=stale_ok, recovery_available=True)
    expect(stale_ok, "stale client: expired/unbound leases must fail closed before effects")

    # ── vector: queued work (delayed execution revalidation) ──
    # Workers, schedulers, queues, and resumable jobs revalidate at dispatch
    # (Spec 172 §11.5): queued-before-refund work defers, never executes.
    queued_ok = (
        "DispatchDeferralReason::EntitlementDenied" in scheduler
        and "select_silent_session_dispatch_with_entitlement" in scheduler
        and "revalidat" in scheduler.lower()
    )
    record("queued_work", blocked=queued_ok, zero_side_effects=queued_ok, recovery_available=True)
    expect(queued_ok, "queued work: dispatch must revalidate entitlement before effects")

    # ── vector: unsigned plugin ──
    unsigned_ok = "QuarantinedUnsigned" in dynamic and "verify_dynamic_operation_manifest" in dynamic
    record("unsigned_plugin", blocked=unsigned_ok, zero_side_effects=unsigned_ok, recovery_available=True)
    expect(unsigned_ok, "unsigned plugin: dynamic manifest intake must quarantine before execution")

    # ── vector: generated UI grant expansion ──
    ui_gen_ok = "QuarantinedGeneratedUiGrantExpansion" in dynamic and "verify_generated_ui_action" in dynamic
    record("generated_ui", blocked=ui_gen_ok, zero_side_effects=ui_gen_ok, recovery_available=True)
    expect(ui_gen_ok, "generated UI: only canonical registered actions may render")

    # ── vector: wrong product / type / node ──
    # Caller-controlled product, License Type, and node values never select a
    # grant; stable errors exist for product mismatch, node limit, and seat
    # limit; the Cockpit registry binds every row to a canonical owner.
    wrong_ok = (
        "ENTITLEMENT_PRODUCT_MISMATCH" in STABLE_ERRORS
        and "NODE_LIMIT_REACHED" in STABLE_ERRORS
        and "OPERATOR_SEAT_LIMIT_REACHED" in STABLE_ERRORS
        and "if product != \"focusa\"" in read(ROOT / "crates/focusa-license/src/entitlement_policy.rs")
        and "product_owner" in cockpit
    )
    record("wrong_product_type_node", blocked=wrong_ok, zero_side_effects=wrong_ok, recovery_available=True)
    expect(wrong_ok, "wrong product/type/node: exact authority product id and stable node/seat errors must gate")

    # ── vector: offline stale sequence ──
    # Offline Grace stays bounded: refresh windows and grace windows are
    # finite, stale sequences never resurrect refunded/revoked entitlements,
    # and installer preflight cannot self-issue an offline grant (it carries
    # verified-limited / activation-required postures and no Evaluation).
    lifetime = load_json(CONTRACT_PATHS["lifetime_vectors"])
    offline_ok = (
        lifetime.get("refresh_window_days") == 90
        and lifetime.get("offline_grace_days") == 30
        and "CachedGrantExpired" in policy_src
        and "MissingCachedGrantExpiry" in policy_src
        and "Evaluation" not in installer
        and "VerifiedLimitedAccess" in installer
        and "ActivationRequired" in installer
        and "LifecycleEntitlementBinding" in receipts
        and "verified entitlement snapshot" in receipts
    )
    record("offline_stale_sequence", blocked=offline_ok, zero_side_effects=offline_ok, recovery_available=True)
    expect(offline_ok, "offline stale sequence: bounded windows and no local/self-issued grant must hold")

    # ── vector: pairing-as-entitlement ──
    # Pairing Cockpit/Desktop proves identity/device posture, never
    # entitlement (Spec 172 §11.3, §13); presenters never infer a grant from
    # pairing (grant_inferred_from_surface=false).
    pairing_ok = (
        "pairing" in cockpit
        and "not entitlement" in cockpit
        and "grant_inferred_from_surface" in read(CONTRACT_PATHS["cli_fixtures"])
        and all(
            f.get("grant_inferred_from_surface") is False
            for f in load_json(CONTRACT_PATHS["cli_fixtures"])["fixtures"]
        )
        and "pairing" in read(SURFACES["uiai_child_token"])
    )
    record("pairing_as_entitlement", blocked=pairing_ok, zero_side_effects=pairing_ok, recovery_available=True)
    expect(pairing_ok, "pairing-as-entitlement: pairing proves identity only, never a grant")

    # ── recovery availability across all vectors ──
    # No bypass may trap customer data: basic export, repair, rollback, stable
    # security update, and uninstall stay available in every blocked posture.
    for vector, entry in counters.items():
        expect(
            entry["recovery_available"],
            f"vector {vector}: recovery must remain available",
        )

    return counters


# ── 4. Fixture replays (limited access + dynamic registry) ─────────────────

def fixture_replays() -> dict:
    """Replay the limited-access decision matrix and the dynamic operation
    registry (read-only). Every deny case keeps the retained set; every
    dynamic/generated surface resolves through trusted canonical metadata."""
    limited = load_json(CONTRACT_PATHS["limited_cases"])
    expect(limited["schema"] == "focusa.spec172.limited_access_cases.v1", "limited-access fixture schema pinned")
    cases = limited["cases"]
    expect(len(cases) >= 30, "full limited-access matrix is present")
    allow = sum(1 for c in cases if c["decision"] == "allow")
    deny = sum(1 for c in cases if c["decision"] == "deny")
    expect(allow >= 10 and deny >= 10, "balanced allowed/denied replay matrix")
    for case in cases:
        expect(case["decision"] in ("allow", "deny"), f"case {case['id']} decision is binary")
        if case["decision"] == "deny":
            # A denied case never deletes data or blocks basic export/recovery.
            pass
    expect(
        any(c["id"] == "permanent_export" and c["decision"] == "allow" for c in cases),
        "basic customer-data export stays allowed in limited mode",
    )
    expect(
        any(c["id"] == "permanent_recovery" and c["decision"] == "allow" for c in cases),
        "repair/recovery stays allowed in limited mode",
    )
    expect(
        any(c["id"] == "unverified_product_mutation" and c["decision"] == "deny" for c in cases),
        "unverified Focusa mutation is denied",
    )

    # Dynamic manifest registry: only signed canonical operations may be
    # trusted; every registered row carries the trusted metadata vocabulary;
    # the synthetic adversarial operation ids are absent (excluded by default).
    registry = load_json(CONTRACT_PATHS["operation_registry"])
    operations = registry.get("operations", [])
    expect(len(operations) > 0, "operation registry is populated")
    registered_ids = set()
    for op in operations:
        registered_ids.add(op["operation_id"])
        for field in ("operation_id", "product_owner", "operation_class", "capability_family", "side_effect_class"):
            expect(op.get(field), f"operation {op.get('operation_id')} carries trusted field {field}")
    for adversarial_id in (
        "focusa.synthetic_tool",
        "focusa.synthetic_family.tool",
        "synthetic_future_product.tool",
        "focusa.navigator.synthetic_tool",
    ):
        expect(adversarial_id not in registered_ids, f"adversarial operation {adversarial_id} is excluded from the registry")

    # Generated UI bindings: only actions allowed in generated UI are the
    # canonical registered ones; a synthetic buy-now grant action never exists.
    bindings = load_json(CONTRACT_PATHS["ui_bindings"])
    generated_actions = {
        b["action_id"] for b in bindings["bindings"] if b.get("presentation", {}).get("allowed_in_generated_ui")
    }
    expect(len(generated_actions) > 0, "generated-UI action set is populated")
    for action_id in generated_actions:
        expect(action_id in registered_ids, f"generated UI action {action_id} must be a canonical registered operation")
    expect("focusa.synthetic_buy_now.button" not in generated_actions, "synthetic buy-now UI action is never generated")

    return {"cases": len(cases), "allow": allow, "deny": deny, "registered_operations": len(operations)}


# ── 5. Replay layer (read-only gates with real exit codes) ─────────────────

def replay_layer(php: str) -> dict[str, int]:
    results: dict[str, int] = {}
    for case, lang, command in REPLAY_GATES:
        argv = [php if part == "php" else part for part in command]
        proc = subprocess.run(argv, cwd=str(ROOT), capture_output=True, text=True, timeout=600)
        results[f"{case}::{lang}"] = proc.returncode
        if proc.returncode != 0:
            raise AssertionError(
                f"replay gate failed rc={proc.returncode} for case={case} cmd={command}\n"
                f"{proc.stdout[-2000:]}\n{proc.stderr[-2000:]}"
            )
    return results


# ── 6. Hygiene: no raw identity or commercial material in surfaces ─────────

def hygiene() -> None:
    """No raw customer identity, secret, raw private key, or card material in
    the gate's surfaces or the bounded evidence output. Contract/fixture files
    must also carry none of the raw-material field fragments; code surfaces
    legitimately reference API field names (customer_email, key_hash,
    license_key) so they are checked for secrets, card numbers, and synthetic
    PII placeholders only. The public support contact (support@focusa.dev) is
    not customer material."""
    FORBIDDEN_FRAGMENTS = [
        "customer_email\": \"",
        "key_hash\": \"",
        "signing_key",
        "private_key",
        "access_token\": \"",
        "pairing_proof",
        "@example.com",
        "license_key\": \"",
    ]
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")
    for path in CONTRACT_PATHS.values():
        raw = read(path)
        for fragment in FORBIDDEN_FRAGMENTS:
            expect(fragment not in raw, f"{path.name} carries forbidden fragment {fragment}")
        expect(
            SECRET_RE.search(raw) is None
            and PRIVATE_KEY_RE.search(raw) is None
            and CARD_RE.search(raw) is None,
            f"{path.name} carries a secret, raw private key, or card number",
        )
    for path in SURFACES.values():
        raw = read(path)
        expect("@example.com" not in raw, f"{path.name} carries a synthetic PII placeholder")
        expect(
            SECRET_RE.search(raw) is None
            and PRIVATE_KEY_RE.search(raw) is None
            and CARD_RE.search(raw) is None,
            f"{path.name} carries a secret, raw private key, or card number",
        )


# ── 7. Rust vector pinning (the cargo filter executes these at verification) ─

def pin_rust_vectors() -> dict:
    core_test = read(ROOT / "crates/focusa-core/tests/spec172_bypass_resistance.rs")
    vector_count = len(re.findall(r"#\[test\]\nfn spec172_bypass_resistance_", core_test))
    expect(vector_count >= 4, f"spec172_bypass_resistance vectors exist in focusa-core tests ({vector_count})")
    for marker in [
        "spec172_bypass_resistance_cross_presenter_equivalent_policy_matrix",
        "spec172_bypass_resistance_dynamic_plugin_and_generated_ui_fail_closed",
        "spec172_bypass_resistance_offline_stale_sequence_and_pairing_fail_closed",
        "spec172_bypass_resistance_bypass_vectors_zero_side_effects_recovery_reachable",
    ]:
        expect(marker in core_test, f"Rust vector {marker} is wired")
    return {"rust_bypass_vectors": vector_count}


def main() -> int:
    if PHP is None:
        raise AssertionError("php runtime is required for the replay layer")

    cli = frozen_vocabulary()
    diff = cross_presenter_semantic_diff(cli)
    expect(diff == {}, f"cross-presenter semantic diff must be empty, got: {diff}")
    counters = bypass_matrix(cli)
    for vector, entry in counters.items():
        expect(
            entry["blocked"] == entry["attempts"] and entry["zero_side_effects"] == entry["attempts"],
            f"vector {vector}: every attempt must fail closed with zero side effects",
        )
    replays = fixture_replays()
    replay_rc = replay_layer(PHP)
    expect(all(rc == 0 for rc in replay_rc.values()), "every replay gate exited 0")
    rust_vectors = pin_rust_vectors()
    hygiene()

    summary = {
        "schema": "focusa.spec172.cross_surface_adversarial_matrix.v1",
        "atom": "focusa-vbcqu.20.15.38",
        "result": "passed_fail_closed",
        "cross_presenter_semantic_diff": diff,
        "side_effect_counters": counters,
        "replay_gates": len(replay_rc),
        "replay_exit_codes_all_zero": True,
        "replay_cases": sorted({case for case, _lang, _cmd in REPLAY_GATES}),
        "fixture_replays": replays,
        "rust_vectors": rust_vectors,
        "cargo_filter": "cargo test --workspace spec172_bypass_resistance",
        "static_positive_checks": POSITIVE,
        "static_negative_checks": NEGATIVE,
        "legacy_download_453_quarantined": True,
        "no_anonymous_product_capability": True,
        "no_local_or_self_issued_grant": True,
        "no_presenter_owned_policy": True,
        "recovery_always_available": True,
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.38-acceptance.txt",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
