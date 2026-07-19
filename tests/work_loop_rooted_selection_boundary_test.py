#!/usr/bin/env python3
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
SCHEDULER = (ROOT / "crates/focusa-core/src/work_item/scheduler.rs").read_text()

class RootedSelectionBoundary(unittest.TestCase):
    def test_no_global_fallback_remains(self):
        self.assertNotIn("maybe_select_global_ready_work_item", API)
        self.assertIn("maybe_select_rooted_ready_work_item", API)

    def test_missing_execution_root_fails_closed(self):
        self.assertIn("let Some(root_work_item_id) = root_work_item_id else", API)
        self.assertIn("provider_neutral_readiness(state, scope_root, Some(&root_work_item_id))", API)

    def test_scheduler_requires_descendant_ancestry(self):
        self.assertIn("is_descendant_of(item, parent, &by_key)", SCHEDULER)
        self.assertIn("blocked_ordered_leaf_does_not_sweep_later_siblings", SCHEDULER)

if __name__ == "__main__":
    unittest.main()
