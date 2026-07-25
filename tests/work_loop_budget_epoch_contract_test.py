#!/usr/bin/env python3
"""Spec79 renewable, agent-readable Work Loop budget contract."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
REDUCER = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
SERVER = (ROOT / "crates/focusa-api/src/server.rs").read_text()
PI = (ROOT / "apps/pi-extension/src/tools.ts").read_text()


class WorkLoopBudgetEpochContractTest(unittest.TestCase):
    def test_exhaustion_is_typed_and_epoch_scoped(self) -> None:
        self.assertIn("pub enum WorkLoopBudgetDimension", TYPES)
        self.assertIn("pub budget_epoch_id: Option<Uuid>", TYPES)
        self.assertIn("pub budget_exhaustion: Option<WorkLoopBudgetExhaustion>", TYPES)
        self.assertIn("WorkLoopBudgetDimension::WallClock", DAEMON)
        self.assertIn('"state": if wl.budget_exhaustion.is_some()', API)

    def test_exhausted_resume_requires_explicit_renewal(self) -> None:
        self.assertIn("resume requires explicit renew_budget=true", DAEMON)
        self.assertIn("budget_renewed: renew_budget", DAEMON)
        self.assertIn("state.work_loop.budget_exhaustion = None", REDUCER)
        self.assertIn("renew_budget", PI)

    def test_supervisor_never_silently_inflates_exhausted_budget(self) -> None:
        self.assertNotIn('reason.contains("max_turns budget exhausted")', SERVER)
        self.assertIn("never silently", SERVER)
        self.assertIn('"exhausted"', API)


if __name__ == "__main__":
    unittest.main()
