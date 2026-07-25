#!/usr/bin/env python3
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
REDUCER = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()

class BlockerDeferralContract(unittest.TestCase):
    def test_provider_commands_never_own_deferral_or_selection(self):
        self.assertNotIn('Command::new("bd")', API)
        self.assertNotIn('Command::new("bd")', DAEMON)
        self.assertIn("Action::DeferContinuousWorkItem", API)
        self.assertIn("ContinuousWorkItemDeferred", REDUCER)

    def test_deferred_work_is_excluded_from_ready_selection(self):
        self.assertIn("deferred_items", API)
        self.assertIn("work_item_is_deferred", DAEMON)
        self.assertIn("execution_work_item_id", API)

    def test_continue_does_not_falsely_complete_or_switch_task(self):
        self.assertIn("WorkLoopOutcomeStatus::Continue", REDUCER)
        self.assertIn("state.work_loop.status = WorkLoopStatus::Idle", REDUCER)

if __name__ == "__main__":
    unittest.main()
