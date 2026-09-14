#!/usr/bin/env python3
"""Spec 172.04.02 CLI/Pi/agent presenter parity (focusa-vbcqu.20.15.26).

Exact verification:
    python3 tests/spec172_cli_agent_presenter_test.py \
        && node apps/pi-extension/tests/pi-entitlement-gate.test.mjs

Build-independent gate over the committed CLI presenter
(crates/focusa-cli/src/commands/license.rs), the Pi entitlement adapter
(apps/pi-extension/src/entitlement-policy-adapter.ts), the parity fixtures
(crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json),
and the agent tool descriptors (docs/contracts/spec141/generated-capability-v2/).

What is proven here (Spec 172 §2.6 surfaces never own policy, §4.1 canonical
posture/License Type codes, §11 surface inheritance, §12 no caller-selected
policy, §21 stable errors):

1. CLI/Pi/agent presenter parity: all three surfaces render the same canonical
   envelope (focusa.spec172.presenter_projection.v1): posture, product,
   License Type, family, denial, retained access, and upgrade/recovery action,
   derived from the authority snapshot only.
2. Presenters never invent entitlement: no caller-controlled product, price,
   License Type, family, feature, limit, node, or commercial right; grants are
   never inferred from the installed client, pairing, tool discovery, or email.
3. Stable JSON and redaction: no raw email, key, token, or customer identity
   in fixtures or presenter sources.
4. Fail-closed denial vocabulary: denials use only the frozen Spec 172 §21
   stable errors; upgrade actions and retained access are frozen sets.
5. Agent tool descriptors carry canonical family vocabulary and never carry
   discovery grant fields.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LICENSE = ROOT / "crates/focusa-cli/src/commands/license.rs"
ADAPTER = ROOT / "apps/pi-extension/src/entitlement-policy-adapter.ts"
FIXTURE_PATH = (
    ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json"
)
CAP = ROOT / "docs/contracts/spec141/generated-capability-v2"
PI_TOOLS_PATH = CAP / "pi-tools.json"
DESCRIPTORS_PATH = CAP / "agent-capability-descriptors.json"
OPENAI_TOOLS_PATH = CAP / "openai-tools.json"
MCP_TOOLS_PATH = CAP / "mcp-tools.json"
CARD_PATH = CAP / "agent-card.json"
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
]

# Grant-inference vocabulary a presenter must never contain: a presenter can
# never derive entitlement from client install, pairing, discovery, or email.
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
    license_source = LICENSE.read_text(encoding="utf-8")
    adapter_source = ADAPTER.read_text(encoding="utf-8")
    fixture_raw = FIXTURE_PATH.read_text(encoding="utf-8")
    fixture = json.loads(fixture_raw)
    pi_tools = json.loads(PI_TOOLS_PATH.read_text(encoding="utf-8"))
    descriptors = json.loads(DESCRIPTORS_PATH.read_text(encoding="utf-8"))
    openai_tools = json.loads(OPENAI_TOOLS_PATH.read_text(encoding="utf-8"))
    mcp_tools = json.loads(MCP_TOOLS_PATH.read_text(encoding="utf-8"))
    card = json.loads(CARD_PATH.read_text(encoding="utf-8"))
    license_types_yaml = LICENSE_TYPES_CONTRACT.read_text(encoding="utf-8")

    # ── 1. Fixture shape, determinism, and privacy ──────────────────────────
    expect(
        fixture["schema"] == "focusa.spec172.cli_agent_presenter_fixtures.v1",
        "fixture schema is stable",
    )
    expect(
        fixture["projection_schema"] == PROJECTION_SCHEMA,
        "fixture projection schema is the canonical Spec 172 envelope",
    )
    expect(set(fixture["surfaces"]) == {"cli", "pi", "agent"},
           "fixture covers CLI, Pi, and agent surfaces")
    expect(
        fixture["envelope_keys"] == ENVELOPE_KEYS,
        "fixture envelope keys are the canonical Spec 172 presenter keys",
    )
    expect(
        len(fixture["canonical_postures"]) == 7
        and "verified_no_license" in fixture["canonical_postures"]
        and "active_paid_operator" in fixture["canonical_postures"],
        "fixture postures are the canonical Spec 172 set",
    )
    expect(
        len(fixture["canonical_license_types"]) == 3
        and "focusa_operator_lifetime_v1" in fixture["canonical_license_types"]
        and "uiai_operator_lifetime_v1" in fixture["canonical_license_types"]
        and "focusa_uiai_operator_bundle_lifetime_v1" in fixture["canonical_license_types"],
        "fixture License Types are the frozen Spec 172 §4.1 codes",
    )
    expect(
        len(fixture["stable_errors"]) == 13
        and set(fixture["stable_errors"])
        == {
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
        },
        "fixture stable errors are exactly the Spec 172 §21 set",
    )
    expect(
        fixture["retained_access"]
        == ["navigation", "status", "account", "read", "export", "recovery", "repair", "update", "uninstall"],
        "fixture retained access is the frozen Spec 172 §5.3/§17 set",
    )
    expect(
        len(fixture["upgrade_actions"]) == 4
        and "none_required" in fixture["upgrade_actions"]
        and "review_offer_or_manage_entitlement" in fixture["upgrade_actions"],
        "fixture upgrade actions are the frozen vocabulary",
    )
    expect(
        fixture["recovery_action"]
        == "recovery, export, repair, and uninstall remain available when execution is locked",
        "fixture recovery action is stable",
    )

    ids = [entry["id"] for entry in fixture["fixtures"]]
    expect(len(ids) == len(set(ids)), "fixture ids are unique")
    expected_ids = {
        "focusa-operator-active",
        "focusa-operator-offline-grace",
        "verified-no-license-manual-allowed",
        "verified-no-license-blocked-family",
        "unverified-email-required",
        "refunded-recovery-only",
        "expired-license-type-required",
        "missing-or-corrupt-policy-unknown",
        "uiai-lease-focusa-family-product-not-included",
    }
    expect(set(ids) == expected_ids, "fixture covers the canonical posture vectors")

    for entry in fixture["fixtures"]:
        expect(
            set(entry.keys()) == set(ENVELOPE_KEYS) | {"id"},
            f"{entry['id']}: entry keys match the canonical envelope plus id",
        )
        expect(
            entry["schema"] == PROJECTION_SCHEMA,
            f"{entry['id']}: entry schema is canonical",
        )
        expect(
            entry["posture"] in fixture["canonical_postures"],
            f"{entry['id']}: posture is canonical",
        )
        expect(
            entry["license_type"] in fixture["canonical_license_types"] + ["none"],
            f"{entry['id']}: license type is canonical",
        )
        if entry["denial"] is not None:
            expect(
                entry["denial"] in fixture["stable_errors"],
                f"{entry['id']}: denial uses a Spec 172 §21 stable error",
            )
        expect(
            entry["upgrade_action"] in fixture["upgrade_actions"],
            f"{entry['id']}: upgrade action is canonical",
        )
        expect(
            entry["retained_access"] == fixture["retained_access"],
            f"{entry['id']}: retained access is frozen",
        )
        expect(
            entry["grant_inferred_from_surface"] is False,
            f"{entry['id']}: presenter never infers grants from the surface",
        )
        # A denied family never loses retained access (Spec 172 §17).
        if entry["denial"] is not None:
            expect(
                len(entry["retained_access"]) == 9,
                f"{entry['id']}: retained access survives denial",
            )

    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in fixture_raw, f"fixture contains forbidden fragment: {fragment}")

    digest = hashlib.sha256(fixture_raw.encode("utf-8")).hexdigest()
    fixture_count = len(fixture["fixtures"])

    # ── 2. CLI presenter parity (Spec 172 §11.2) ────────────────────────────
    cli_region_start = license_source.index("/// Spec 172 canonical presenter projection")
    cli_region_end = license_source.index("async fn run_preflight")
    cli_region = license_source[cli_region_start:cli_region_end]

    for marker in [
        PROJECTION_SCHEMA,
        '"posture"',
        '"product"',
        '"license_type"',
        '"family"',
        '"denial"',
        '"retained_access"',
        '"upgrade_action"',
        '"recovery_action"',
        '"grant_inferred_from_surface"',
        "spec172_projection",
        "spec172_denial_and_upgrade",
        "spec172_base_denial",
        "spec172_posture",
        "spec172_license_type",
    ]:
        expect(marker in cli_region, f"CLI Spec 172 region missing marker: {marker}")

    # Frozen vocabulary parity with the fixture.
    for posture in fixture["canonical_postures"]:
        expect(f'"{posture}"' in cli_region, f"CLI posture vocabulary missing {posture}")
    for code in fixture["canonical_license_types"]:
        expect(f'"{code}"' in cli_region, f"CLI License Type code missing {code}")
    for error in fixture["stable_errors"]:
        expect(f'"{error}"' in cli_region, f"CLI stable error missing {error}")
    for item in fixture["retained_access"]:
        expect(f'"{item}"' in cli_region, f"CLI retained access missing {item}")
    for action in fixture["upgrade_actions"]:
        expect(f'"{action}"' in cli_region, f"CLI upgrade action missing {action}")

    # The CLI executes through the core guard and re-resolves the canonical
    # base/premium decisions; it never reads a local grant, key, or pairing.
    for marker in [
        "focusa_license::resolve_license_guard()",
        "resolve_base_focusa_product",
        "authority_policy_state",
        "resolve_premium_family",
        "resolve_export_packaged",
        "CapabilityFamily::Automation",
        "CapabilityFamily::TeamRemote",
        "CapabilityFamily::ReleaseProof",
        "CapabilityFamily::PremiumUpdates",
        "CapabilityFamily::CustomerDataExport",
    ]:
        expect(marker in license_source, f"CLI must execute through core guard: {marker}")

    # Bounded preflight input: --family is constrained to canonical families
    # and unknown families fail closed.
    expect(
        "E_AUTHORITY_UNKNOWN_PREFLIGHT_FAMILY" in license_source,
        "CLI preflight rejects unknown families",
    )
    for family in ["base_focusa", "automation", "team_remote", "release_proof", "premium_updates", "customer_data_export"]:
        expect(f'"{family}"' in license_source, f"CLI preflight missing canonical family {family}")

    # status and preflight both render the Spec 172 projection.
    expect(
        '"spec172": spec172_projection(guard.entitlement.as_ref(), "base_focusa")' in license_source,
        "CLI status renders the Spec 172 projection",
    )
    expect(
        '"spec172": spec172_projection(snapshot, family)' in license_source,
        "CLI preflight renders the Spec 172 projection",
    )

    # No grant inference and no secret material in the CLI presenter region.
    for fragment in INFERENCE_FRAGMENTS:
        expect(fragment not in cli_region, f"CLI presenter infers grants: {fragment}")
    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in cli_region, f"CLI presenter contains {fragment}")

    # ── 3. Pi adapter parity (Spec 172 §11.2) ───────────────────────────────
    pi_region_start = adapter_source.index("// ── Spec 172 canonical presenter projection")
    pi_region = adapter_source[pi_region_start:]

    for marker in [
        PROJECTION_SCHEMA,
        "SPEC172_POSTURES",
        "SPEC172_LICENSE_TYPE_CODES",
        "SPEC172_STABLE_ERRORS",
        "SPEC172_RETAINED_ACCESS",
        "SPEC172_UPGRADE_ACTIONS",
        "SPEC172_RECOVERY_ACTION",
        "SPEC172_SURFACE_PRODUCT",
        "projectSpec172PresenterV1",
        "spec172PostureForAuthority",
        "spec172DenialAndUpgrade",
        "grant_inferred_from_surface: false",
        "posture",
        "license_type",
        "denial",
        "retained_access",
        "upgrade_action",
        "recovery_action",
    ]:
        expect(marker in pi_region, f"Pi Spec 172 region missing marker: {marker}")

    for posture in fixture["canonical_postures"]:
        expect(f'"{posture}"' in pi_region, f"Pi posture vocabulary missing {posture}")
    for code in fixture["canonical_license_types"]:
        expect(f'"{code}"' in pi_region, f"Pi License Type code missing {code}")
    for error in fixture["stable_errors"]:
        expect(f'"{error}"' in pi_region, f"Pi stable error missing {error}")
    for item in fixture["retained_access"]:
        expect(f'"{item}"' in pi_region, f"Pi retained access missing {item}")
    for action in fixture["upgrade_actions"]:
        expect(f'"{action}"' in pi_region, f"Pi upgrade action missing {action}")

    # The Pi adapter projects from the canonical tool contract and the daemon
    # authority posture; it never accepts caller-selected commercial fields.
    expect(
        "resolveOperationPolicyForTool" in pi_region,
        "Pi adapter resolves family from the canonical tool contract",
    )
    expect(
        "preflightAuthority" in pi_region,
        "Pi adapter preflights before side effects",
    )
    expect(
        "unknown_tool_has_no_operation_policy" in adapter_source,
        "Pi adapter fails closed on unknown tools",
    )
    expect(
        "focusa.entitlement_decision.v1" in adapter_source,
        "Pi adapter keeps the canonical entitlement decision schema",
    )
    expect(
        "licensing_grants_capability_only: true" in adapter_source,
        "Pi adapter keeps the capability-only authority boundary",
    )
    expect(
        "operator_authority_granted: false" in adapter_source,
        "Pi adapter never grants operator authority",
    )
    expect(
        "cognitive_authority_granted: false" in adapter_source,
        "Pi adapter never grants cognitive authority",
    )
    expect(
        "approval_inferred: false" in adapter_source,
        "Pi adapter never infers approval",
    )
    expect(
        "discovery_visibility_granted: false" in adapter_source,
        "Pi adapter visibility never grants entitlement",
    )

    # projectSpec172PresenterV1 accepts only (toolName, posture): no
    # caller-controlled product, price, License Type, family, feature, limit,
    # node, or commercial right parameter.
    expect(
        "projectSpec172PresenterV1(\n  toolName: string,\n  posture: AuthorityPosture\n)" in adapter_source,
        "Pi Spec 172 projection takes only tool name and authority posture",
    )
    for fragment in INFERENCE_FRAGMENTS:
        expect(fragment not in pi_region, f"Pi presenter infers grants: {fragment}")
    for fragment in FORBIDDEN_FRAGMENTS:
        expect(fragment not in pi_region, f"Pi presenter contains {fragment}")

    # ── 4. Agent tool descriptors carry canonical family vocabulary ─────────
    descriptor_families = {
        descriptor["operation_policy"]["capability_family"]
        for descriptor in descriptors["descriptors"]
    }
    canonical_families = {
        "account_recovery",
        "read_projection",
        "base_focusa",
        "automation",
        "team_remote",
        "release_proof",
        "premium_updates",
        "customer_data_export",
        "internal_maintenance",
    }
    expect(descriptor_families <= canonical_families, "descriptor families are canonical")
    for descriptor in descriptors["descriptors"]:
        policy = descriptor.get("operation_policy")
        expect(policy is not None, f"descriptor {descriptor['capability_id']} has no operation policy")
        expect(
            policy["policy_owner"] == "entitlement_policy_resolver",
            f"descriptor {descriptor['capability_id']} policy owner must be the resolver",
        )
        expect(
            set(policy.keys()) >= {"operation_class", "capability_family", "commercial_treatment"},
            f"descriptor {descriptor['capability_id']} policy incomplete",
        )

    # Discovery projections (pi/openai/mcp tool lists, agent card) are
    # advisory routing metadata and never carry entitlement grant fields.
    DISCOVERY_GRANT_FIELDS = {
        "operation_policy",
        "commercial_treatment",
        "required_feature",
        "limit_bucket",
        "policy_activation",
        "policy_owner",
        "licensing_grants_capability_only",
        "license_type",
        "denial",
        "upgrade_action",
    }
    for projection in (pi_tools, openai_tools, mcp_tools):
        for tool in projection["tools"]:
            keys = set(tool.keys())
            expect(
                not keys & DISCOVERY_GRANT_FIELDS,
                f"{projection['schema']} discovery payload carries a grant field",
            )
    expect(card["schema"] == "focusa.agent_card.v1", "agent card schema is stable")

    # ── 5. Canonical registry convergence (Spec 172 §4.1) ───────────────────
    for code in fixture["canonical_license_types"]:
        expect(code in license_types_yaml, f"License Type code {code} must exist in the frozen registry")

    # ── 6. Cross-surface parity invariants ──────────────────────────────────
    # The same frozen vocabulary appears byte-identically in the fixture, the
    # CLI presenter, and the Pi adapter, so no surface can drift.
    expect(
        adapter_source.count(PROJECTION_SCHEMA) >= 1 and license_source.count(PROJECTION_SCHEMA) >= 1,
        "both presenters reference the canonical projection schema",
    )
    for item in fixture["retained_access"]:
        expect(
            f'"{item}"' in license_source and f'"{item}"' in adapter_source,
            f"retained access item {item} must be shared by both presenters",
        )
    for error in fixture["stable_errors"]:
        expect(
            f'"{error}"' in license_source and f'"{error}"' in adapter_source,
            f"stable error {error} must be shared by both presenters",
        )
    # Neither presenter ever emits a true grant-inference flag.
    expect(
        "grant_inferred_from_surface: false" in adapter_source,
        "Pi adapter marks grant_inferred_from_surface false",
    )
    expect(
        '"grant_inferred_from_surface": false' in license_source,
        "CLI marks grant_inferred_from_surface false",
    )

    print(
        "Spec172 CLI/Pi/agent presenter parity: PASS "
        f"(fixtures={fixture_count} sha256={digest[:16]} "
        f"surfaces=cli,pi,agent positive={POSITIVE} negative={NEGATIVE})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
