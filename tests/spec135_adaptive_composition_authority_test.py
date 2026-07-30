#!/usr/bin/env python3
"""P01 authority gate for adaptive Mission Canvas composition."""
from __future__ import annotations

import hashlib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
HANDOFF = ROOT / "docs/contracts/spec135/authoritative-handoff/spec135_agent_handoff_apple_principles.md"
ACTIVITY_IMAGE = ROOT / "docs/contracts/spec135/authoritative-handoff/focusa_activity_mode_recomposition.png"
VERTICAL_IMAGE = ROOT / "docs/contracts/spec135/authoritative-handoff/focusa_dynamic_vertical_recomposition.png"
HOST_PATH = ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
MANIFEST = (ROOT / "docs/135-series-current-manifest.md").read_text()
DIRECTIVE = (ROOT / "docs/agent/spec135-implementation-acceleration-directive.md").read_text()
AGENTS = (ROOT / "AGENTS.md").read_text()
HOST_TEXT = HOST_PATH.read_text()
HOST = yaml.safe_load(HOST_TEXT)
ADAPTIVE = yaml.safe_load((ROOT / "docs/contracts/spec135-adaptive-composition.v1.yaml").read_text())
QUALITY = yaml.safe_load((ROOT / "docs/contracts/spec135-ux-quality-bar.v1.yaml").read_text())
RESPONSIVE = yaml.safe_load((ROOT / "docs/contracts/spec135-responsive-matrix.v1.yaml").read_text())
ADAPTIVE_PROOF = yaml.safe_load((ROOT / "docs/contracts/spec135-adaptive-composition-proof-matrix.v1.yaml").read_text())
PROOF_MATRIX_TEXT = (ROOT / "docs/contracts/spec135-proof-matrix.v1.yaml").read_text()

assert hashlib.sha256(HANDOFF.read_bytes()).hexdigest() == "88f41bd2aadf248a7ddfe2a1ec13b886559a1cc6886429ae2e971d7ed07c4614"
assert hashlib.sha256(ACTIVITY_IMAGE.read_bytes()).hexdigest() == "e7a116b47a77eb8b8bae6ebd3cf048146fa5217e72f27d8f126e89e7f7faba93"
assert hashlib.sha256(VERTICAL_IMAGE.read_bytes()).hexdigest() == "a53ba95b3f411a76c75d2a46ecaa206f58700e8646c6dc85846dca39d0d18763"

authority = HOST["authority"]["adaptive_composition_authority"]
assert authority["replacement_text"].endswith("spec135_agent_handoff_apple_principles.md")
assert authority["precedence"] == "replacement_text_over_images_over_older_contracts_for_occupancy"
assert authority["images_are_fixed_inventory"] is False
assert HOST["authority"]["superseded_history_must_not_close_gates"] is True

rich_host = HOST["required_enhanced_pi_host"]
assert rich_host["implementation_owner"] == "Pi_extension"
assert rich_host["required_platforms"] == ["macOS", "Windows", "Linux"]
assert rich_host["release_path"] == "canonical_Git_and_GitHub_release_pipeline"

shell = HOST["canonical_shell_invariant"]
assert "ordered_regions" not in shell
assert shell["semantic_contribution_capabilities"] == [
    "work_surface_strip",
    "focused_work_surface_with_focusa_right_inspector",
    "work_rail",
    "steering_queue",
    "follow_up_queue",
    "prompt_editor",
]
for key in (
    "capability_order_is_not_permanent_geometry",
    "omission_before_geometry",
    "omitted_contribution_leaves_no_heading_border_control_or_reserved_space",
    "remaining_contributions_reflow_deterministically",
    "semantic_substitution_to_fill_space_forbidden",
    "populated_reference_images_are_examples_not_inventory",
):
    assert shell[key] is True, key

for text in (MANIFEST, DIRECTIVE, AGENTS):
    assert "spec135_agent_handoff_apple_principles.md" in text
    assert "focusa_activity_mode_recomposition.png" in text
    assert "focusa_dynamic_vertical_recomposition.png" in text
    assert "populated examples" in text.lower()

assert "required **semantic contribution capabilities**" in MANIFEST
assert "omitted before geometry" in MANIFEST
assert "macOS, Windows, and Linux" in MANIFEST
assert "The six canonical concepts are contribution capabilities, not permanent panel slots." in DIRECTIVE
assert "The replacement text outranks images and older contracts for occupancy." in AGENTS

assert ADAPTIVE["core_law"]["no_dead_chrome"] is True
assert ADAPTIVE["resolved_workspace_projection"]["type_name"] == "ResolvedWorkspaceProjection"
assert ADAPTIVE["resolved_workspace_projection"]["canonical_owner"] == "Focusa_Core"
assert len(ADAPTIVE["resolved_workspace_projection"]["inputs_in_required_order"]) == 10
assert ADAPTIVE["eligibility"]["evaluation_order"] == [
    "semantic_relevance",
    "applicable_activity_mode",
    "meaningful_content",
    "operator_authority",
    "runtime_capability",
    "active_work_surface_relationship",
    "viewport_suitability",
]
assert len(ADAPTIVE["omission_diagnostics"]["canonical_reasons"]) == 8
assert ADAPTIVE["queue_composition"]["one_populated"] == [
    "single_queue_spans_queue_region",
    "no_blank_sibling_lane",
]
assert ADAPTIVE["semantic_anti_counterfeiting"]["missing_contribution_may_be_replaced_by_different_semantic_panel"] is False
assert len(ADAPTIVE["proof_requirements"]) == 13
assert len(ADAPTIVE["forbidden"]) == 13

assert QUALITY["authority"]["operator_quality_boundary"] == "absolutely_no_compromised_slop"
assert QUALITY["cross_platform"]["required"] == ["macOS", "Windows", "Linux"]
assert QUALITY["friction_gate"]["severe_unresolved_friction_blocks_release"] is True
assert QUALITY["accessibility"]["keyboard_complete"] is True
assert QUALITY["motion"]["every_transition"] == [
    "interruptible",
    "bounded",
    "deterministic_final_state",
    "no_placeholder_flash",
    "no_layout_hole",
]

assert RESPONSIVE["host_scope"]["platforms"] == ["macOS", "Windows", "Linux"]
assert RESPONSIVE["host_scope"]["minimum_window"] == {
    "css_width": 1024,
    "css_height": 720,
    "enforcement": "host_minimum_size",
}
assert [entry["id"] for entry in RESPONSIVE["viewport_classes"]] == [
    "minimum",
    "compact",
    "standard",
    "productive",
    "wide",
    "reference_capture",
]
assert len(RESPONSIVE["UIAI_Engine_Eval"]["scenarios"]) == 13

proofs = ADAPTIVE_PROOF["proofs"]
assert [proof["proof_id"] for proof in proofs] == [f"AC-{index:02d}" for index in range(1, 14)]
assert all(proof["status"] == "pending" for proof in proofs)
assert ADAPTIVE_PROOF["closure"]["all_13_required"] is True
assert ADAPTIVE_PROOF["closure"]["merge_blocked_until_all_pass_with_evidence_and_receipts"] is True
assert "spec135-adaptive-composition-proof-matrix.v1.yaml" in PROOF_MATRIX_TEXT
assert "all 13 adaptive-composition no-dead-chrome runtime proofs" in PROOF_MATRIX_TEXT

print("Spec 135 adaptive-composition authority: PASS")
