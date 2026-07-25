#!/usr/bin/env python3
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
ISSUES = [json.loads(line) for line in (ROOT / ".beads/issues.jsonl").read_text().splitlines() if line.strip()]
BY_ID = {item["id"]: item for item in ISSUES}

def blocking_dependencies(item):
    return {
        edge.get("depends_on_id") or edge.get("id")
        for edge in item.get("dependencies", [])
        if edge.get("type") != "parent-child"
    }

class Spec133ExecutionDependencyGraph(unittest.TestCase):
    def test_each_open_phase_is_a_sequential_leaf_chain(self):
        for phase in range(3, 11):
            prefix = f"focusa-a6yq6.{phase}."
            children = sorted(
                [item for item in ISSUES if item["id"].startswith(prefix) and item["id"][len(prefix):].isdigit()],
                key=lambda item: int(item["id"].split(".")[-1]),
            )
            self.assertGreaterEqual(len(children), 2, phase)
            leaves, gate = children[:-1], children[-1]
            for previous, current in zip(leaves, leaves[1:]):
                self.assertIn(previous["id"], blocking_dependencies(current), current["id"])
            for leaf in leaves:
                self.assertIn(leaf["id"], blocking_dependencies(gate), gate["id"])

    def test_each_phase_starts_after_previous_phase_gate(self):
        for phase in range(3, 11):
            first = BY_ID[f"focusa-a6yq6.{phase}.1"]
            previous_children = [
                item for item in ISSUES
                if item["id"].startswith(f"focusa-a6yq6.{phase - 1}.")
                and item["id"].split(".")[-1].isdigit()
            ]
            previous_gate = max(previous_children, key=lambda item: int(item["id"].split(".")[-1]))
            self.assertIn(previous_gate["id"], blocking_dependencies(first), first["id"])

if __name__ == "__main__":
    unittest.main()
