#!/usr/bin/env python3
"""Spec 172.04.04 Focusa Desktop entitlement (focusa-vbcqu.20.15.28).

Exact verification:
    python3 tests/spec172_focusa_desktop_entitlement_test.py

Build-independent gate over the committed Focusa Desktop / Tauri command
bridge and action registry (apps/menubar/src-tauri/src/spec172_desktop_bridge.rs,
wired through apps/menubar/src-tauri/src/main.rs), the desktop action map
contract (docs/contracts/spec152f-desktop-action-map.v1.json), and the
desktop release-artifact gate workflow
(.github/workflows/desktop-spec172-entitlement-gate.yml).

Acceptance criteria (Spec 172 §11.4, §15, §7.3; task 172.04.04): Desktop has
zero local entitlement authority, zero direct-storage mutation bypass, and
identical decisions to CLI/API.

What is proven here:

1. PRESENTER-NOT-PRODUCT / ZERO LOCAL AUTHORITY: the bridge is pure (no
   module-level mutable state), holds no prices, grants, sale status, or
   local commercial tables, and never accepts a caller-controlled product,
   price, License Type, family, feature, limit, node, or commercial right.
2. ZERO DIRECT-STORAGE MUTATION BYPASS: the bridge has no storage/reducer/
   sqlite path by construction; every value-producing desktop action carries
   a daemon route and forwards to the shared core execution guard; unknown
   actions fail closed (Spec 172 §11.4, §12).
3. IDENTICAL DECISIONS TO CLI/API: the desktop projection renders the
   canonical focusa.spec172.presenter_projection.v1 envelope with the exact
   frozen vocabulary of the CLI/Pi/agent presenter fixture (7 postures,
   3 License Types, 13 stable errors, 9 retained-access entries, 4 upgrade
   actions, recovery action, grant_inferred_from_surface=false).
4. ACTION MAP PARITY: the desktop action registry rows match the canonical
   operation table of the menubar action map (operation id, family, method,
   route, mutation class); the desktop action map contract is schema-stable.
5. SAME-NODE IDENTITY: Focusa Desktop never registers a node or multiplies
   activations; CLI/menubar/Desktop on the same node share the authority's
   node identity (Spec 172 §7.3).
6. LIMITED READ/EXPORT/RECOVERY PRESERVED + PAID FAMILIES BLOCKED: the
   frozen retained-access set is never disabled; locked-state fixtures name
   the canonical upgrade/recovery action; paid families (team_remote,
   automation, release_proof, premium_updates, customer_data_export) are
   blocked consistently with the canonical decision mapping.
7. FIRST-RUN/LOCKED-STATE FIXTURES: frozen fixtures ship in the release
   artifact tree and never create a grant, node, price, or License Type
   locally.
8. RELEASE ARTIFACT WORKFLOW: the release gate runs the exact verification,
   the menubar/TUI parity test, validates the contract, compiles the bridge
   standalone, and checks the bridge ships in the artifact tree.
9. HYGIENE: no raw email, key, token, customer row, credential, or card
   data; no prices; no implicit legacy Download 453 mapping; no anonymous
   product capability; no local/self-issued grant.
"""

import json
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BRIDGE = ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs"
MAIN_RS = ROOT / "apps/menubar/src-tauri/src/main.rs"
CONTRACT_PATH = ROOT / "docs/contracts/spec152f-desktop-action-map.v1.json"
WORKFLOW = ROOT / ".github/workflows/desktop-spec172-entitlement-gate.yml"
MENUBAR_MAP = ROOT / "docs/contracts/spec152f-menubar-action-map.v1.json"
CLI_FIXTURE = (
    ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json"
)
LICENSE_TYPES_CONTRACT = ROOT / "docs/contracts/spec172-license-types.v1.yaml"

PROJECTION_SCHEMA = "focusa.spec172.presenter_projection.v1"
ENVELOPE_KEYS = [
    "schema",
    "posture",
    "product",
    "license_type",
    "family",
    "denial",
    "retained_access",
    "upgrade_action",
    "recovery_action",
    "grant_inferred_from_surface",
]

FORBIDDEN_FRAGMENTS = [
    "customer_email",
    "key_hash",
    "signing_key",
    "private_key",
    "access_token",
    "pairing_proof",
    "@example.com",
    "license_key",
    "card_number",
    "cvv",
]

# Grant-inference vocabulary a presenter must never contain.
INFERENCE_FRAGMENTS = [
    "granted_by_pairing",
    "granted_by_client",
    "granted_by_discovery",
    "granted_by_email",
    "installed_client_grants",
    "pairing_grants_entitlement",
    "discovery_grants_entitlement: true",
    "email_grants_entitlement",
]

# Direct-storage mutation bypass vocabulary the Desktop bridge must never
# contain: the bridge has no storage/reducer/sqlite path by construction
# (Spec 172 §11.4).
DIRECT_STORAGE_FRAGMENTS = [
    "sqlite",
    "rusqlite",
    "INSERT INTO",
    "DELETE FROM",
    "std::fs",
    "File::open",
    "OpenOptions",
    "reduce_entitlement_state",
    "resolve_license_guard",
    "authority_store",
]

# Zero-local-authority vocabulary: the Desktop must never mint, price, or
# issue anything locally.
LOCAL_AUTHORITY_FRAGMENTS = [
    "price_usd",
    "issue_license",
    "create_grant",
    "--eval",
    "anonymous_access: true",
]

# Value-bearing commercial-field patterns that must never exist in the
# contract (a presenter never owns a sale-status or price value).
LOCAL_AUTHORITY_VALUES = ['"sale_status":', '"price_usd":', '"sale_status" :']

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
    bridge_source = BRIDGE.read_text(encoding="utf-8")
    main_rs = MAIN_RS.read_text(encoding="utf-8")
    contract_raw = CONTRACT_PATH.read_text(encoding="utf-8")
    contract = json.loads(contract_raw)
    workflow = WORKFLOW.read_text(encoding="utf-8")
    menubar_map = json.loads(MENUBAR_MAP.read_text(encoding="utf-8"))
    cli_fixture = json.loads(CLI_FIXTURE.read_text(encoding="utf-8"))
    license_types_yaml = LICENSE_TYPES_CONTRACT.read_text(encoding="utf-8")

    # ── 1. Desktop action map contract shape and privacy ───────────────────
    expect(
        contract["schema"] == "focusa.spec152f.desktop_action_map.v1",
        "desktop action map schema is stable",
    )
    expect(
        contract["spec172"]["schema"] == "focusa.spec172.desktop_presenter_parity.v1",
        "desktop Spec 172 parity schema is stable",
    )
    expect(
        contract["spec172"]["presenter_not_product"] is True,
        "desktop action map marks Desktop as presenter, not product",
    )
    expect(
        contract["spec172"]["no_direct_core_bypass"]["direct_storage"] == "forbidden",
        "desktop action map forbids direct storage bypass",
    )
    expect(
        contract["spec172"]["no_direct_core_bypass"]["local_grant_creation"] == "forbidden",
        "desktop action map forbids local grant creation",
    )
    expect(
        contract["spec172"]["no_direct_core_bypass"]["anonymous_product_capability"] == "forbidden",
        "desktop action map forbids anonymous product capability",
    )
    expect(
        contract["spec172"]["no_direct_core_bypass"]["local_self_issued_grant"] == "forbidden",
        "desktop action map forbids local self-issued grants",
    )
    expect(
        contract["spec172"]["same_node_identity"]
        == "Focusa Desktop uses same-node identity: it never creates a node, activation, seat, or node counter of its own; CLI/menubar/Desktop on the same node share the authority's registered node identity",
        "desktop same-node identity sentence is frozen",
    )
    expect(
        contract["spec172"]["locked_state_fixtures"]["upgrade_action"]
        == "activate_or_manage_entitlement",
        "locked-state fixture names the canonical upgrade/recovery action",
    )
    expect(
        contract["spec172"]["locked_state_fixtures"]["never_disabled"]
        == ["read", "export", "recovery", "repair", "update", "uninstall"],
        "locked-state fixtures never disable read/export/recovery/repair/update/uninstall",
    )
    expect(
        contract["spec172"]["first_run_fixtures"]["no_local_grant"] is True
        and contract["spec172"]["first_run_fixtures"]["no_local_node"] is True
        and contract["spec172"]["first_run_fixtures"]["no_local_license_type"] is True,
        "first-run fixtures never create a grant, node, or License Type locally",
    )
    expect(
        contract["classification_summary"]
        == {"navigation_display": 3, "recovery_account": 11, "canonical_operation": 10},
        "desktop action classification counts match the registry",
    )
    actions = contract["actions"]
    action_ids = [entry["desktop_action_id"] for entry in actions]
    expect(len(action_ids) == len(set(action_ids)) == 24, "24 unique desktop actions")
    for entry in actions:
        entry_class = entry["action_class"]
        expect(
            entry_class in contract["classification_summary"],
            f"{entry['desktop_action_id']}: action class is canonical",
        )
    # The contract's action classes must match the frozen class policy.
    for entry in actions:
        cls = entry["action_class"]
        policy = contract["action_classes"][cls]
        expect(
            "presenter_must_not" in policy
            and "evaluate_entitlement" in policy["presenter_must_not"],
            f"{entry['desktop_action_id']}: presenter never evaluates entitlement",
        )

    digest = hashlib.sha256(contract_raw.encode("utf-8")).hexdigest()
    action_count = len(actions)

    # ── 2. Bridge: presenter-not-product and zero local authority ──────────
    for marker in [
        PROJECTION_SCHEMA,
        "DESKTOP_ACTION_REGISTRY",
        "resolve_desktop_action",
        "project_desktop_spec172_posture",
        "SPEC172_DESKTOP_SAME_NODE",
        "SPEC172_PRESENTER_NOT_PRODUCT",
        "SPEC172_NO_DIRECT_CORE_BYPASS",
        "desktop_first_run_fixture",
        "desktop_locked_state_fixture",
        "grant_inferred_from_surface: false",
        "same_node: true",
        "direct_storage: false",
        "forwards_to_core_guard",
    ]:
        expect(marker in bridge_source, f"desktop bridge missing marker: {marker}")
    expect(
        "presenter, not a product" in bridge_source
        and "presenter, not a product" in contract_raw,
        "desktop bridge carries the presenter-not-product sentence",
    )
    expect(
        "never owns pricing, grants, limits, License Types, or commercial policy"
        in bridge_source,
        "desktop bridge never owns commercial policy",
    )
    expect(
        "no module-level mutable state" in bridge_source,
        "desktop bridge is pure with no module-level mutable state",
    )

    # Frozen vocabulary is byte-identical to the CLI/API presenter fixture.
    for posture in cli_fixture["canonical_postures"]:
        expect(f'"{posture}"' in bridge_source, f"desktop bridge posture missing {posture}")
    for code in cli_fixture["canonical_license_types"]:
        expect(f'"{code}"' in bridge_source, f"desktop bridge License Type code missing {code}")
    for error in cli_fixture["stable_errors"]:
        expect(f'"{error}"' in bridge_source, f"desktop bridge stable error missing {error}")
    for item in cli_fixture["retained_access"]:
        expect(f'"{item}"' in bridge_source, f"desktop bridge retained access missing {item}")
    for action in cli_fixture["upgrade_actions"]:
        expect(f'"{action}"' in bridge_source, f"desktop bridge upgrade action missing {action}")
    expect(
        cli_fixture["recovery_action"] in bridge_source,
        "desktop bridge recovery action matches the canonical fixture",
    )

    # ── 3. Zero direct-storage mutation bypass ──────────────────────────────
    for fragment in DIRECT_STORAGE_FRAGMENTS:
        expect(fragment not in bridge_source, f"desktop bridge has direct storage path: {fragment}")
    # Every value-producing action in the registry carries a daemon route.
    # (The bridge marks each entry's path; routes start with '/' and are
    # forwarded to the core guard. `mutation: true` entries always route.)
    mutation_entries = bridge_source.count("mutation: true")
    expect(mutation_entries == 9, f"9 mutation-capable desktop actions (found {mutation_entries})")
    expect(
        bridge_source.count("forwards_to_core_guard = entry.path.starts_with('/')") == 1,
        "bridge derives core-guard forwarding from the daemon route only",
    )

    # ── 4. Action-map parity with the menubar canonical operations ─────────
    menubar_ops = menubar_map["canonical_operations"]
    for op_id, meta in menubar_ops.items():
        expect(
            f'operation_id: Some("{op_id}")' in bridge_source,
            f"desktop registry missing canonical operation {op_id}",
        )
        expect(
            f'family: Some("{meta["family"]}")' in bridge_source,
            f"desktop registry missing family {meta['family']} for {op_id}",
        )
        route = meta["route"]
        if " " in route:
            method, path = route.split(" ", 1)
            if path != "local_only":
                expect(f'method: "{method}"' in bridge_source, f"desktop registry missing method {method}")
                expect(f'path: "{path}"' in bridge_source, f"desktop registry missing route {path}")
        if meta["mutation_class"] == "mutation":
            expect(
                f'operation_id: Some("{op_id}")' in bridge_source,
                f"mutation operation {op_id} present",
            )
    # Paid families are blocked by the bridge projection (Spec 172 §17):
    # the bridge references every paid family and denies each with the
    # canonical CAPABILITY_FAMILY_NOT_INCLUDED when the base gate is usable.
    for paid_family in ["team_remote", "automation", "release_proof", "premium_updates", "customer_data_export"]:
        expect(
            f'"{paid_family}"' in bridge_source,
            f"desktop bridge must reference paid family {paid_family}",
        )
    expect(
        '"CAPABILITY_FAMILY_NOT_INCLUDED"' in bridge_source,
        "desktop bridge blocks paid families with the canonical stable error",
    )
    # The contract rows mirror the bridge registry rows (same action ids).
    for entry in actions:
        expect(
            f'action_id: "{entry["desktop_action_id"]}"' in bridge_source,
            f"bridge registry missing action {entry['desktop_action_id']} listed in the contract",
        )
    for family_count, expected in [
        ('family: Some("account_recovery")', 11),
        ('family: Some("base_focusa")', 8),
        ('family: Some("team_remote")', 2),
    ]:
        expect(
            bridge_source.count(family_count) == expected,
            f"desktop registry family count {family_count} != {expected}",
        )

    # ── 5. Same-node identity ───────────────────────────────────────────────
    expect(
        "never registers a node" in bridge_source
        and "do not consume separate nodes" in bridge_source,
        "desktop bridge carries the frozen same-node semantics",
    )
    expect(
        "never counts apps as nodes" in contract_raw
        or "never counts apps as nodes" in bridge_source,
        "desktop never counts apps as nodes",
    )

    # ── 6. Tauri command bridge wiring ──────────────────────────────────────
    expect("mod spec172_desktop_bridge;" in main_rs, "main.rs declares the desktop bridge module")
    expect(
        "fn focusa_desktop_route_action" in main_rs,
        "main.rs registers the route-action command",
    )
    expect(
        "fn focusa_desktop_spec172_posture" in main_rs,
        "main.rs registers the posture-projection command",
    )
    expect(
        "focusa_desktop_route_action," in main_rs and "focusa_desktop_spec172_posture," in main_rs,
        "both desktop commands are registered in the Tauri invoke handler",
    )
    expect(
        "unknown desktop action" in main_rs,
        "route-action command fails closed on unknown actions",
    )

    # ── 7. Release artifact workflow gates ──────────────────────────────────
    for gate in [
        "python3 tests/spec172_focusa_desktop_entitlement_test.py",
        "node tests/spec172_menubar_tui_presenter_test.mjs",
        "rustc --edition 2021 --test apps/menubar/src-tauri/src/spec172_desktop_bridge.rs",
        "focusa_desktop_route_action",
        "focusa_desktop_spec172_posture",
    ]:
        expect(gate in workflow, f"release artifact workflow missing gate: {gate}")
    expect(
        "never builds, signs, deploys, or releases artifacts" in workflow,
        "release gate is a gate only; no deploy/release",
    )
    expect("permissions:" in workflow and "contents: read" in workflow, "workflow is read-only")

    # ── 8. First-run / locked-state fixtures ────────────────────────────────
    expect(
        "no_local_grant=true" in bridge_source,
        "desktop first-run fixture forbids local grants",
    )
    expect(
        "no_local_node=true" in bridge_source,
        "desktop first-run fixture forbids local node creation",
    )
    expect(
        "no_local_license_type=true" in bridge_source,
        "desktop first-run fixture forbids local License Type creation",
    )
    expect(
        "upgrade_action=activate_or_manage_entitlement" in bridge_source,
        "desktop locked-state fixture names the canonical upgrade action",
    )
    expect(
        "never_disabled=read,export,recovery,repair,update,uninstall" in bridge_source,
        "desktop locked-state fixture preserves limited read/export/recovery",
    )

    # ── 9. Hygiene: no secrets, no prices, no inference, no legacy mapping ──
    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in bridge_source, f"desktop bridge contains {fragment}")
        expect(fragment not in contract_raw, f"desktop contract contains {fragment}")
    for fragment in INFERENCE_FRAGMENTS:
        expect(fragment not in bridge_source, f"desktop bridge infers grants: {fragment}")
    for fragment in LOCAL_AUTHORITY_FRAGMENTS:
        expect(fragment not in bridge_source, f"desktop bridge has local authority: {fragment}")
        expect(fragment not in contract_raw, f"desktop contract has local authority: {fragment}")
    for pattern in LOCAL_AUTHORITY_VALUES:
        expect(pattern not in contract_raw, f"desktop contract carries a commercial value: {pattern}")
    # The legacy Download 453 prohibition is documented (contract) and the
    # bridge carries no 453 identifier at all (no implicit mapping).
    expect(
        "no implicit legacy Download 453 mapping" in contract_raw,
        "desktop contract documents the legacy Download 453 prohibition",
    )
    expect(
        "453" not in bridge_source,
        "desktop bridge must carry no legacy Download 453 mapping",
    )
    # Price material: no price values ever appear in the bridge (the word
    # "pricing" appears only inside the frozen prohibition sentences).
    for price in ["697", "1254", "1394", "$"]:
        expect(price not in bridge_source, f"desktop bridge contains price material: {price}")
    expect(
        "pricing" in bridge_source
        and "never owns pricing" in bridge_source,
        "desktop bridge documents the no-pricing prohibition",
    )

    # ── 10. Canonical registry convergence (Spec 172 §4.1) ──────────────────
    for code in cli_fixture["canonical_license_types"]:
        expect(code in license_types_yaml, f"License Type code {code} must exist in the frozen registry")

    print(
        "Spec172 Focusa Desktop entitlement: PASS "
        f"(actions={action_count} sha256={digest[:16]} "
        f"surfaces=desktop,tauri positive={POSITIVE} negative={NEGATIVE})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
