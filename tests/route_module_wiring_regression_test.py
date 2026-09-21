#!/usr/bin/env python3
"""Prevent recurrence of the interleave-era API route orphan regression (#339)."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
ROUTES = ROOT / "crates/focusa-api/src/routes"
MOD_RS = (ROUTES / "mod.rs").read_text(encoding="utf-8")
SERVER_RS = (ROOT / "crates/focusa-api/src/server.rs").read_text(encoding="utf-8")

WIRED_MODULES = (
    "adapters",
    "background_jobs",
    "callgraph",
    "completion_claims",
    "direction",
    "events_retention",
    "remote_workspaces",
    "research_packet",
    "runtime_constitution",
    "session_fanout",
    "silent_sessions_wait",
    "worksets",
)


class RouteModuleWiringRegressionTest(unittest.TestCase):
    def test_original_orphan_modules_are_declared_and_merged(self) -> None:
        for module in WIRED_MODULES:
            with self.subTest(module=module):
                self.assertTrue((ROUTES / f"{module}.rs").is_file())
                self.assertIn(f"pub mod {module};", MOD_RS)
                self.assertIn(f".merge(routes::{module}::router())", SERVER_RS)

    def test_legacy_compaction_controller_is_explicitly_superseded(self) -> None:
        self.assertFalse((ROUTES / "compaction_controller.rs").exists())
        self.assertNotIn("mod compaction_controller;", MOD_RS)
        for module in (
            "compaction",
            "compaction_policy",
            "compaction_policy_resolution",
        ):
            with self.subTest(module=module):
                self.assertIn(f"pub mod {module};", MOD_RS)
                self.assertIn(f".merge(routes::{module}::router())", SERVER_RS)


if __name__ == "__main__":
    unittest.main()
