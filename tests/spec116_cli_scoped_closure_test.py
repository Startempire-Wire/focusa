#!/usr/bin/env python3
from pathlib import Path
import unittest

SOURCE = (Path(__file__).resolve().parents[1] / "crates/focusa-cli/src/commands/work_item.rs").read_text()

class ScopedClosureCli(unittest.TestCase):
    def test_close_requires_full_authority_context(self):
        for field in ["from_workpoint", "continuity_id", "agent_session_id"]:
            self.assertIn(f"pub {field}: String", SOURCE)
        self.assertIn(".run_scoped(", SOURCE)
        self.assertIn("ClosureAuthorityContext", SOURCE)

    def test_close_requires_explicit_code_and_test_evidence(self):
        self.assertIn("pub code_ref: Vec<String>", SOURCE)
        self.assertIn("pub test_ref: Vec<String>", SOURCE)
        self.assertIn("build_explicit_citations", SOURCE)
        self.assertNotIn("build_citations_from_recent_tests", SOURCE)
        self.assertNotIn('ref_: "http://127.0.0.1:8787', SOURCE)

if __name__ == "__main__":
    unittest.main()
