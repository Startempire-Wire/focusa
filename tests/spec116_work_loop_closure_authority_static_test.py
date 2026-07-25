#!/usr/bin/env python3
"""Fail-closed Work Loop → Spec 116 closure authority checks."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
TURN = (ROOT / "crates/focusa-api/src/routes/turn.rs").read_text()


class WorkLoopClosureAuthorityTest(unittest.TestCase):
    def test_work_loop_surfaces_never_close_bd_directly(self) -> None:
        direct_close = re.compile(
            r'Command::new\("bd"\)[\s\S]{0,240}?\.args\(\[(?:[^\]]*?)"close"'
        )
        self.assertIsNone(direct_close.search(DAEMON))
        self.assertIsNone(direct_close.search(API))

    def test_daemon_routes_completion_through_scoped_lifecycle(self) -> None:
        self.assertIn("complete_work_item_via_lifecycle", DAEMON)
        self.assertIn("Lifecycle::open_for_kind", DAEMON)
        self.assertIn(".run_scoped(", DAEMON)
        self.assertIn("canonical_workpoint_required", DAEMON)
        self.assertIn("workpoint_workstream_mismatch", DAEMON)
        self.assertIn("if closure_claim.is_some()", DAEMON)

    def test_prose_or_nonempty_output_is_not_verification(self) -> None:
        self.assertNotIn("verification_satisfied: true", API)
        self.assertIn("parse_work_loop_outcome_receipt", API)
        self.assertIn("parse_work_loop_outcome_receipt", TURN)
        self.assertIn("Never claim completion from prose alone", API)
        self.assertIn("WorkLoopOutcomeStatus::Continue", TURN)


if __name__ == "__main__":
    unittest.main()
