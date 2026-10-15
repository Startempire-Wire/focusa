#!/usr/bin/env python3
"""Project source-only Spec187 tasks into the existing Spec155 structural type.

This compiler never contacts a daemon or admits, starts or settles any work.
The checked-in fixture uses synthetic scope and cannot be runtime authority.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "docs/contracts/spec187/release-workset.v1.json"
FIXTURE = ROOT / "docs/contracts/spec187/callgraph.structural-fixture.v1.json"


def compile_graph(profile: dict, project_root: str, continuity_id: str,
                  created_at: str, graph_id: str, fixture: bool = False) -> dict:
    if not Path(project_root).is_absolute() or not continuity_id.strip():
        raise ValueError("explicit absolute project root and non-empty continuity are required")
    frames, edges = [], []
    incoming = {row["step"]: len(row["deps"]) for row in profile["work_items"]}
    for row in profile["work_items"]:
        frames.append({
            "frame_id": row["step"], "name": row["title"],
            "purpose": row["done"], "kind": "agent",
            "input_schema": {"type": "object", "required": ["work_item_ref", "scope_ref", "authority_ref"],
                "properties": {"work_item_ref": {"const": row["work_item_ref"]},
                    "scope_ref": {"type": "string", "minLength": 1},
                    "authority_ref": {"type": "string", "minLength": 1}}},
            "return_schema": {"type": "object", "required": ["evidence_refs", "remaining_requirements"],
                "properties": {"evidence_refs": {"type": "array", "items": {"type": "string"}, "minItems": 1},
                    "remaining_requirements": {"type": "array", "items": {"type": "string"}}}},
            "preconditions": ["Current exact scope, operation, writer, approval and budget must be verified by their existing owners",
                "Required execution bindings are resolved before dispatch; this source definition grants none"],
            "postconditions": [row["done"], "Source readiness does not settle installed requirements"],
            "side_effect_class": "external" if row["step"] in {"FLOW-15", "FLOW-16", "FLOW-17"} else "local",
            "capability_refs": [],
            "authority_requirement": {"authority_kind": "existing_scoped_operation_authority",
                "reference": profile["operator_request_ref"]},
            "retry_policy": {"max_attempts": 1, "backoff_ms": 0, "jitter": False},
            "failure_boundary": {"kind": "scoped", "unwind_policy": "preserve evidence; reconcile effects; use supported repair or truthful bounded pause"},
            "acceptance": {"acceptance_atoms": row["requirement_ids"],
                "verifier": profile["spec_ref"] + "#6-reference-mission-and-measurable-acceptance"}
        })
        for dependency in row["deps"]:
            edges.append({"edge_id": dependency + "--" + row["step"],
                "from_frame_id": dependency, "to_frame_id": row["step"],
                "kind": "await", "join_policy": "all" if incoming[row["step"]] > 1 else None})
    return {
        "schema": "focusa.callgraph.v1", "graph_id": graph_id, "revision": 1,
        "scope": {"project_root": project_root, "continuity_id": continuity_id},
        "mission_ref": profile["spec_ref"], "trajectory_ref": None, "workpoint_refs": [],
        "title": "Autonomous Agentic Flow State — source workset dependency projection",
        "description": "SYNTHETIC STRUCTURAL FIXTURE ONLY; NEVER ADMIT OR DISPATCH" if fixture else
            "Source definition only: canonical validation, current scope, membership, bindings and admission are still required",
        "entry_frame_ids": [row["step"] for row in profile["work_items"] if not row["deps"]],
        "frames": frames, "edges": edges,
        "policies": {"max_depth": 32, "default_cycle_policy": "deny"},
        "required_evidence": [{"evidence_type": kind, "min_count": 1} for kind in
            ["versioned_contract", "producer_test", "consumer_test", "cross_version_interop", "installed_e2e"]],
        "created_at": created_at,
        "created_by": {"authority_kind": "synthetic_fixture" if fixture else "operator_request_reference",
            "reference": "fixture-only-never-runtime-authority" if fixture else profile["operator_request_ref"]}
    }


def fixture_graph(profile: dict) -> dict:
    return compile_graph(profile, "/tmp/focusa-spec187-structural-fixture",
        "synthetic-spec187-fixture-only", "2026-10-15T00:00:00Z",
        "synthetic-spec187-structural-fixture-only", True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", action="store_true")
    parser.add_argument("--check-fixture", action="store_true")
    parser.add_argument("--project-root")
    parser.add_argument("--continuity-id")
    parser.add_argument("--graph-id")
    args = parser.parse_args()
    profile = json.loads(PROFILE.read_text(encoding="utf-8"))
    if args.check_fixture:
        if json.loads(FIXTURE.read_text(encoding="utf-8")) != fixture_graph(profile):
            raise SystemExit("structural fixture is stale; regenerate from the authoring profile")
        print("spec187 structural fixture matches source authoring; no runtime mutation")
        return
    if args.fixture:
        graph = fixture_graph(profile)
    else:
        if not all([args.project_root, args.continuity_id, args.graph_id]):
            parser.error("real projection requires explicit verified --project-root, --continuity-id and --graph-id")
        graph = compile_graph(profile, args.project_root, args.continuity_id,
            datetime.now(timezone.utc).isoformat(), args.graph_id)
    print(json.dumps(graph, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
