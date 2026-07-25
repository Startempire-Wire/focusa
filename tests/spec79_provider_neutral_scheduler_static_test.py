#!/usr/bin/env python3
"""Static authority boundary checks for Spec 79 provider-neutral scheduling."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCHEDULER_SURFACES = [
    ROOT / "crates/focusa-api/src/routes/work_loop.rs",
    ROOT / "crates/focusa-core/src/runtime/daemon.rs",
]


class ProviderNeutralSchedulerBoundaryTest(unittest.TestCase):
    def test_scheduler_surfaces_do_not_invoke_bd_traversal_commands(self) -> None:
        forbidden = re.compile(
            r'Command::new\("bd"\)[\s\S]{0,240}?\.args\(\[(?:[^\]]*?)"(?:ready|show|list)"'
        )
        for path in SCHEDULER_SURFACES:
            source = path.read_text()
            self.assertIsNone(forbidden.search(source), path)

    def test_api_selection_and_alternate_status_share_core_readiness(self) -> None:
        source = SCHEDULER_SURFACES[0].read_text()
        self.assertIn("async fn provider_neutral_readiness(", source)
        self.assertGreaterEqual(source.count("provider_neutral_readiness("), 3)
        self.assertIn("evaluate_readiness(&items, &query)", source)

    def test_daemon_selection_and_tranche_checks_use_adapter_snapshots(self) -> None:
        source = SCHEDULER_SURFACES[1].read_text()
        self.assertIn("adapter.list(&query).await?", source)
        self.assertIn("evaluate_readiness(&items, &query)", source)
        self.assertIn("async fn tranche_has_remaining_ready_work(", source)


if __name__ == "__main__":
    unittest.main()
