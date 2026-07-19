#!/usr/bin/env python3
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
OUTCOME = DAEMON.split("Action::ObserveContinuousTurnOutcome", 1)[1].split(
    "Action::CheckpointContinuousLoop", 1
)[0]


class ProductiveContinueBudgetContract(unittest.TestCase):
    def test_continue_is_not_low_productivity_solely_for_being_unverified(self):
        self.assertIn(
            "outcome_status\n                        == WorkLoopOutcomeStatus::Blocked",
            OUTCOME,
        )
        self.assertNotIn("predicted_low_productivity = !verification_satisfied", OUTCOME)
        self.assertIn("if predicted_low_productivity", OUTCOME)

    def test_completion_only_gates_do_not_block_productive_continue(self):
        self.assertGreaterEqual(
            OUTCOME.count("outcome_status == WorkLoopOutcomeStatus::Completed"), 4
        )
        self.assertIn("linked_spec_implementation_evidenced", OUTCOME)
        self.assertIn("require_verification_before_persist", OUTCOME)
        self.assertIn("&& !spec_conformant", OUTCOME)
        self.assertIn(
            "outcome_status == WorkLoopOutcomeStatus::Completed\n                    && let Some(selected_task)",
            OUTCOME,
        )
        self.assertIn("run_secondary_adversarial_closure_audit", OUTCOME)


if __name__ == "__main__":
    unittest.main()
