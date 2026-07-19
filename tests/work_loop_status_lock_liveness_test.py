#!/usr/bin/env python3
from pathlib import Path
import re
import unittest

SOURCE = (Path(__file__).resolve().parents[1] / "crates/focusa-api/src/routes/work_loop.rs").read_text()

def function_body(name, next_name):
    match = re.search(rf"async fn {name}\([\s\S]*?(?=\nasync fn {next_name}\()", SOURCE)
    if not match:
        raise AssertionError(f"missing function {name}")
    return match.group(0)

class WorkLoopStatusLockLiveness(unittest.TestCase):
    def test_health_clones_projection_before_writer_claim_lock(self):
        body = function_body("health", "status")
        clone = body.index("state.focusa.read().await.clone()")
        claims = body.index("state.writer_claims.read().await")
        self.assertLess(clone, claims)

    def test_status_drops_projection_guard_before_all_secondary_awaits(self):
        body = function_body("status", "status_deep")
        self.assertIn("state.focusa.read().await.clone()", body)
        self.assertNotIn("let s = state.focusa.read().await;", body)
        for awaited in ["state.writer_claims.read().await", "state.pi_rpc_session.lock().await", "worktree_status_snapshot(&scope_root).await"]:
            self.assertIn(awaited, body)

if __name__ == "__main__":
    unittest.main()
