#!/usr/bin/env python3
"""Generate the admission-qualified locked-release governance inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
OUTPUT = AUDIT / "next-locked-release-governance-inventory.json"

AUTHORITY_FILES = {
    "scope": AUDIT / "next-locked-release-scope.json",
    "definition": AUDIT / "next-locked-release-workset-definition.json",
    "members": AUDIT / "next-locked-release-workset-members.jsonl",
    "edges": AUDIT / "next-locked-release-workset-edges.jsonl",
    "events": AUDIT / "next-locked-release-workset-events.jsonl",
    "completion_contract": AUDIT / "next-locked-release-workset-completion-contract.json",
    "repair_graph": ROOT / "docs" / "153-focusa-locked-release-repair-task-graph.md",
    "callgraph_reconstruction": ROOT / "docs" / "156-focusa-mangled-release-delta-callgraph-reconstruction.md",
    "spec152e_correction": ROOT / "docs" / "152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md",
    "spec152_document_set": ROOT / "docs" / "contracts" / "spec152-document-set.v1.yaml",
}

EXCLUDED_EPICS = [
    ("focusa-vbcqu.11", 45, "mission_canvas_transition"),
    ("focusa-vbcqu.12", 89, "reconstructed_broad_multiplexing_epic"),
    ("focusa-vbcqu.13", 101, "future_fleet_convergence"),
    ("focusa-vbcqu.15", 107, "optional_letta_integration"),
    ("focusa-vbcqu.16", 112, "reconstructed_broad_adaptive_compaction_epic"),
    ("focusa-vbcqu.17", 114, "unsupported_external_uiai_solver_capability"),
    ("focusa-vbcqu.18", 52, "future_onboarding_and_dead_road_consolidation"),
]

REPAIR_OVERLAY = [
    {
        "bead_id": "focusa-vbcqu.14",
        "github_issue": 106,
        "purpose": "release_governance_reconciliation",
        "authority": "operator_message:remaining_tranche_only_mangled_previous_release",
    },
    {
        "bead_id": "focusa-vbcqu.19",
        "github_issue": None,
        "purpose": "all_surface_artifact_install_ota_and_publication_repair",
        "authority": "operator_message:remaining_tranche_only_mangled_previous_release",
    },
    {
        "bead_id": "focusa-vbcqu.20",
        "github_issue": 119,
        "purpose": "mandatory_spec152_and_152e_edd_centered_licensing_correction",
        "authority": "operator_trajectory:complete_truthful_locked_release_with_specs150a_152_152e",
    },
    *[
        {
            "bead_id": f"focusa-vbcqu.10.{phase}",
            "github_issue": 119,
            "purpose": "mandatory_spec152_granular_implementation_and_acceptance_decomposition",
            "authority": "operator_trajectory:complete_truthful_locked_release_with_specs150a_152_152e",
        }
        for phase in range(7, 13)
    ],
]

RETAINED_INVARIANTS = [
    "v0.9.143_is_immutable",
    "new_release_version_must_be_monotonic_and_truthful",
    "signed_leases_are_the_only_production_entitlement_authority",
    "plaintext_tiers_and_self_issued_evaluation_never_grant_capability",
    "ambiguous_or_foreign_scope_fails_closed_without_foreign_payload",
    "pi_owns_native_compaction_execution",
    "windows_x64_and_arm64_assets_require_real_installed_ota_proof",
    "publication_requires_exact_sha_artifact_and_installed_acceptance",
    "wpuiai_edd_is_the_sole_customer_commerce_human_key_and_entitlement_authority",
    "unverified_email_never_creates_canonical_customer_or_entitlement_truth",
]


def sha256(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def stable_digest(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line]


def build() -> dict:
    scope = json.loads(AUTHORITY_FILES["scope"].read_text())
    definition = json.loads(AUTHORITY_FILES["definition"].read_text())
    members = jsonl(AUTHORITY_FILES["members"])
    edges = jsonl(AUTHORITY_FILES["edges"])
    events = jsonl(AUTHORITY_FILES["events"])
    member_ids = [row["member_id"] for row in members]
    member_set = set(member_ids)

    if len(member_ids) != 275 or len(member_set) != 275:
        raise SystemExit("immutable r7 membership must contain exactly 275 unique identities")
    if definition["workset_id"] != "workset:focusa-next-locked-release:r7":
        raise SystemExit("unexpected workset identity")
    if definition["membership_digest"] != "sha256:03e384bbf5728df135f36838451098413701f4ee09430fc31797fe2d5e1379f0":
        raise SystemExit("immutable membership digest changed")
    if definition["graph_digest"] != "sha256:d1d095bf0e9f8b8d6f5e41fef14711369bdffd523b6cdc01005a932807214b70":
        raise SystemExit("immutable graph digest changed")

    excluded = []
    for bead_id, github_issue, reason in EXCLUDED_EPICS:
        collisions = sorted(
            member_id
            for member_id in member_set
            if member_id == bead_id or member_id.startswith(bead_id + ".")
        )
        if collisions:
            raise SystemExit(f"excluded epic overlaps immutable r7: {bead_id}: {collisions}")
        excluded.append(
            {
                "bead_id": bead_id,
                "github_issue": github_issue,
                "disposition": "future_or_repository_work_excluded_from_locked_release",
                "reason": reason,
                "code_and_history": "retained",
                "release_parent": None,
                "publication_blocking": False,
            }
        )

    explicit_additions = [
        {
            "issue_id": row["issue_id"],
            "authorization_source": row["authorization_source"],
            "summary": row["summary"],
        }
        for row in scope["operator_authorized_post_lock_additions"]
    ]

    inventory = {
        "schema": "focusa.locked_release_governance_inventory.v1",
        "status": "frozen",
        "workset_id": definition["workset_id"],
        "workset_revision": definition["revision"],
        "immutable_member_count": len(members),
        "immutable_edge_count": len(edges),
        "immutable_event_count": len(events),
        "membership_digest": definition["membership_digest"],
        "graph_digest": definition["graph_digest"],
        "scope_additions_closed": scope["scope_additions_closed"],
        "further_additions_allowed": scope["final_scope_admission"]["further_additions_allowed"],
        "authority_file_digests": {
            name: {"path": str(path.relative_to(ROOT)), "sha256": sha256(path)}
            for name, path in AUTHORITY_FILES.items()
        },
        "operator_authorized_post_lock_additions": explicit_additions,
        "authorized_release_repair_overlay": REPAIR_OVERLAY,
        "excluded_reconstructed_epics": excluded,
        "retained_release_invariants": RETAINED_INVARIANTS,
        "admission_rules": [
            "bare_bead_id_is_not_admission",
            "github_issue_presence_is_not_admission",
            "labels_parentage_and_code_presence_are_not_admission",
            "future_valid_work_may_remain_in_repository_without_release_gating",
            "only_immutable_r7_exact_purpose_explicit_operator_addition_or_scope_preserving_release_repair_is_admitted",
        ],
        "terminal_release_path": [
            "workset:focusa-next-locked-release:r7",
            "bead:focusa-vbcqu.14",
            "bead:focusa-vbcqu.20",
            "bead:focusa-vbcqu.19",
            "new_monotonic_stable_release",
        ],
    }
    inventory["inventory_digest"] = stable_digest(inventory)
    return inventory


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = json.dumps(build(), indent=2, sort_keys=True) + "\n"
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered:
            print(f"governance inventory drift: regenerate {OUTPUT.relative_to(ROOT)}")
            return 1
        print("locked-release governance inventory: PASS")
        return 0
    OUTPUT.write_text(rendered)
    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
