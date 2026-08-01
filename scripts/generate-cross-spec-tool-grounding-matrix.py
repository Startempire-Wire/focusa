#!/usr/bin/env python3
"""Generate the exhaustive Spec137/138/144/150 tool and family grounding baseline."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/current/focusa-tool-contracts.json"
OUTPUT = ROOT / "docs/contracts/spec137-138-144-150-tool-grounding-matrix.v1.yaml"
SPECS = ["137", "137a", "138", "138a", "144", "150"]


def row(contract: dict[str, object]) -> dict[str, object]:
    name = str(contract["name"])
    return {
        "tool_name": name,
        "family": contract.get("family"),
        "applicability_decision": "decision_required",
        "applicable_specs": SPECS,
        "status": "implementation_open",
        "machine_readable_contract_refs": [
            "docs/current/focusa-tool-contracts.json",
            str(contract.get("doc_path") or ""),
        ],
        "focus_stack_refs": [],
        "reducer_event_refs": [],
        "projection_replay_refs": [],
        "awareness_refs": [],
        "runbook_refs": [],
        "recovery_refs": [],
        "runtime_effect_refs": [],
        "adversarial_test_refs": [],
        "evidence_refs": [],
    }


def build() -> dict[str, object]:
    source = json.loads(SOURCE.read_text())
    contracts = sorted(source["contracts"], key=lambda item: item["name"])
    families = sorted({str(item["family"]) for item in contracts})
    return {
        "schema": "focusa.cross_spec_tool_grounding_matrix.v1",
        "source_contract_ref": "docs/current/focusa-tool-contracts.json",
        "source_tool_count": source["tool_count"],
        "governing_specs": SPECS,
        "status": "implementation_open",
        "tools": [row(item) for item in contracts],
        "internal_families": [
            {
                "family": family,
                "tool_count": sum(1 for item in contracts if item["family"] == family),
                "applicability_decision": "decision_required",
                "status": "implementation_open",
                "focus_stack_refs": [],
                "reducer_event_refs": [],
                "projection_replay_refs": [],
                "awareness_refs": [],
                "runbook_refs": [],
                "recovery_refs": [],
                "runtime_effect_refs": [],
                "adversarial_test_refs": [],
                "evidence_refs": [],
            }
            for family in families
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = build()
    rendered = yaml.safe_dump(expected, sort_keys=False, width=100)
    if args.check:
        if not OUTPUT.is_file():
            print("cross-spec tool grounding matrix is stale or missing")
            return 1
        actual = yaml.safe_load(OUTPUT.read_text())
        immutable_keys = ("schema", "source_contract_ref", "source_tool_count", "governing_specs")
        if any(actual.get(key) != expected.get(key) for key in immutable_keys):
            print("cross-spec tool grounding matrix is stale or missing")
            return 1
        actual_tools = actual.get("tools", [])
        expected_tools = expected["tools"]
        if [(row.get("tool_name"), row.get("family")) for row in actual_tools] != [
            (row.get("tool_name"), row.get("family")) for row in expected_tools
        ]:
            print("cross-spec tool grounding matrix is stale or missing")
            return 1
        if actual.get("status") == "verified_complete":
            required_refs = (
                "focus_stack_refs", "reducer_event_refs", "projection_replay_refs",
                "awareness_refs", "runbook_refs", "recovery_refs",
                "runtime_effect_refs", "adversarial_test_refs", "evidence_refs",
            )
            rows = [*actual_tools, *actual.get("internal_families", [])]
            if any(row.get("status") != "verified_complete" or any(not row.get(key) for key in required_refs) for row in rows):
                print("cross-spec activated tool grounding matrix is incomplete")
                return 1
        print("cross-spec tool grounding matrix: current")
        return 0
    OUTPUT.write_text(rendered)
    print(f"wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
