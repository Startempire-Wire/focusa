#!/usr/bin/env python3
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
ISSUES = [json.loads(line) for line in (ROOT / ".beads/issues.jsonl").read_text().splitlines() if line.strip()]
SPEC_REF = "spec:docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md"
ADAPTER = (ROOT / "crates/focusa-core/src/work_item/adapters/bd.rs").read_text()
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()

class Spec133WorkItemGrounding(unittest.TestCase):
    def test_every_open_spec133_slice_has_authoritative_ref_and_acceptance(self):
        targets = [item for item in ISSUES if item["id"].startswith("focusa-a6yq6.") and item.get("status") not in {"closed", "done", "cancelled"}]
        self.assertGreater(len(targets), 0)
        for item in targets:
            self.assertIn(SPEC_REF, item.get("labels", []), item["id"])
            self.assertTrue((item.get("acceptance_criteria") or "").strip(), item["id"])

    def test_adapter_promotes_explicit_spec_labels_only(self):
        self.assertIn('label.strip_prefix("spec:")', ADAPTER)
        self.assertNotIn("infer_spec", ADAPTER)

    def test_task_packet_preserves_refs_and_acceptance(self):
        self.assertIn("linked_spec_refs: item.spec_refs", DAEMON)
        self.assertIn("acceptance_criteria: item.acceptance_criteria", DAEMON)

if __name__ == "__main__":
    unittest.main()
