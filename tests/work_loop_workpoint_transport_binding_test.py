#!/usr/bin/env python3
"""Spec79 canonical Workpoint/WorkItem/Workstream/transport binding checks."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
SERVER = (ROOT / "crates/focusa-api/src/server.rs").read_text()


class WorkLoopPartitionBindingTest(unittest.TestCase):
    def test_workpoint_is_reducer_owned_execution_partition_member(self) -> None:
        self.assertIn("pub execution_workpoint_id: Option<WorkpointId>", TYPES)
        self.assertIn("workpoint_id: WorkpointId", TYPES)
        self.assertIn("canonical Workpoint", API)

    def test_enable_requires_exact_scope_root_item_and_workpoint(self) -> None:
        self.assertIn("canonical_workpoint_id_for_scope_and_item", API)
        self.assertIn("does not match execution scope and root WorkItem", DAEMON)
        self.assertIn("execution_workpoint_id", SERVER)

    def test_transport_and_events_are_partition_bound(self) -> None:
        self.assertIn("pub transport_scope: Option<WorkstreamKey>", TYPES)
        self.assertIn("pub transport_workpoint_id: Option<WorkpointId>", TYPES)
        self.assertIn("transport session partition does not match", DAEMON)
        self.assertIn("transport event rejected: session or execution partition mismatch", DAEMON)


if __name__ == "__main__":
    unittest.main()
