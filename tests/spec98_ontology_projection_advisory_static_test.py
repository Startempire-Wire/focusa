#!/usr/bin/env python3
"""Spec98 focusa-877z.12: ontology projections are advisory until promoted."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT / "docs/worksheets/focusa-877z.12-ontology-projection-advisory-contract.yaml"
)
ONTOLOGY = ROOT / "crates/focusa-api/src/routes/ontology.rs"
TYPES = ROOT / "crates/focusa-core/src/types.rs"

SURFACES = [
    "ontology_world",
    "ontology_read_index_cache_metadata",
    "ontology_adjacency_read_index",
    "ontology_working_set_projection",
    "ontology_prompt_safe_context",
    "ontology_affordance_execution_projection",
    "ontology_retrieval_governor",
]


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def fn_body(source: str, name: str) -> str:
    for marker in [f"fn {name}", f"pub fn {name}", f"async fn {name}"]:
        start = source.find(marker)
        if start != -1:
            break
    else:
        fail(f"missing function {name}")
    brace = source.find("{", start)
    depth = 0
    for i in range(brace, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    fail(f"unterminated function {name}")


def assert_advisory_payload(
    body: str, name: str, canonical_key: str = '"canonical"'
) -> None:
    for needle in [
        '"advisory_only": true',
        '"promotion_path"',
        '"authority": ontology_projection_authority_metadata()',
        '"canonical_truth_mutation": false',
    ]:
        if needle not in body:
            fail(f"{name} missing advisory projection marker {needle}")
    if canonical_key == '"canonical"' and '"canonical": false' not in body:
        fail(f"{name} must expose canonical=false for projection authority")
    if (
        canonical_key == '"projection_canonical"'
        and '"projection_canonical": false' not in body
    ):
        fail(f"{name} must expose projection_canonical=false")


def main() -> None:
    contract = yaml.safe_load(CONTRACT.read_text())
    if (
        contract.get("schema_version")
        != "focusa.ontology_projection_advisory_contract.v1"
    ):
        fail("unexpected .12 contract schema")
    rule = contract.get("normative_rule", "")
    for phrase in [
        "advisory by default",
        "not canonical task meaning",
        "Workpoint",
        "PRE proposal",
        "reducer/governance",
    ]:
        if phrase not in rule:
            fail(f"contract normative rule missing {phrase}")
    for surface in SURFACES:
        if surface not in contract.get("projection_surfaces", {}):
            fail(f"contract missing surface {surface}")

    types = TYPES.read_text()
    if '("ontology", AuthorityPlane::AdvisoryProjection)' not in types:
        fail("FocusaState plane contract must keep ontology as AdvisoryProjection")

    ontology = ONTOLOGY.read_text()
    helper = fn_body(ontology, "ontology_projection_authority_metadata")
    for needle in [
        '"advisory_only": true',
        '"canonical": false',
        '"canonical_truth_mutation": false',
        '"promotion_path"',
        '"canonicalization_tools"',
        '"focusa_workpoint_checkpoint"',
        '"focusa_active_object_resolve"',
        '"focusa_evidence_capture"',
        '"canonical_task_meaning"',
        '"resume_authority"',
        '"focus_state_mutation"',
    ]:
        if needle not in helper:
            fail(f"authority helper missing {needle}")

    assert_advisory_payload(
        fn_body(ontology, "adjacency_index_payload"), "adjacency_index_payload"
    )
    assert_advisory_payload(
        fn_body(ontology, "working_set_payload"), "working_set_payload"
    )
    assert_advisory_payload(
        fn_body(ontology, "ontology_context_payload"), "ontology_context_payload"
    )
    assert_advisory_payload(
        fn_body(ontology, "affordances_payload"), "affordances_payload"
    )
    assert_advisory_payload(
        fn_body(ontology, "retrieval_governor_payload"), "retrieval_governor_payload"
    )

    cache = fn_body(ontology, "ontology_read_index_cache_metadata")
    assert_advisory_payload(
        cache,
        "ontology_read_index_cache_metadata",
        canonical_key='"projection_canonical"',
    )
    if (
        '"canonical_meaning": "cache_entry_freshness_only_not_task_authority"'
        not in cache
    ):
        fail("read-index cache canonical field must be scoped to cache freshness only")

    world = fn_body(ontology, "world")
    assert_advisory_payload(world, "world")
    if '"canonical_and_projection_are_distinct"' not in world:
        fail("world projection profile must preserve canonical/projection distinction")

    for name in ["ontology_context_payload", "retrieval_governor_payload"]:
        body = fn_body(ontology, name)
        if "state.focusa.write().await" in body or "dispatch_event" in body:
            fail(f"{name} must remain read/projection only")

    print(
        "✓ PASS: ontology projections expose advisory/canonical flags and promotion path"
    )


if __name__ == "__main__":
    main()
