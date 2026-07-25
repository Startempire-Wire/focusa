#!/usr/bin/env python3
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
TUI = (ROOT / "crates/focusa-tui/src/views/work_loop.rs").read_text()
MENUBAR = (ROOT / "apps/menubar/src/lib/components/WorkLoopPeek.svelte").read_text()
PI = (ROOT / "apps/pi-extension/src/tools.ts").read_text()

class OperatorDiagnosticsContract(unittest.TestCase):
    def test_status_exposes_full_execution_authority(self):
        for field in ["work_item_provider", "workpoint_id", "transport_session_id", "transport_work_item_id", "deferred_work_item_ids", "exact_recovery_action"]:
            self.assertIn(field, API)

    def test_human_surfaces_render_partition_and_budget_recovery(self):
        for source in [TUI, MENUBAR]:
            self.assertIn("Workpoint", source) if source is TUI else self.assertIn("workpoint", source)
            self.assertIn("budget", source.lower())
            self.assertIn("transport", source.lower())

    def test_tui_scope_formatter_is_wired_to_execution_partition(self):
        for marker in [
            "typed_scope_from_status(loop_status)",
            "lines.push(typed_scope_line(typed_scope.as_ref()))",
            'get("project_root_key")',
            'get("workstream_key")',
            'get("partition_status")',
        ]:
            self.assertIn(marker, TUI)

    def test_agent_surface_can_configure_and_renew_budgets(self):
        for field in ["renew_budget", "max_turns", "max_wall_clock_ms", "max_retries"]:
            self.assertIn(field, PI)

if __name__ == "__main__":
    unittest.main()
