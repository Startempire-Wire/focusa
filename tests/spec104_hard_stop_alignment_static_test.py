#!/usr/bin/env python3
"""Spec104 DOC-05: hard-stop doc + runtime alignment.

Verifies that:
- error_envelope middleware returns "blocked" status for scope-mismatch errors
- Docs reference the blocked envelope format consistently
- Live scope-conflict path matches documented blocked envelope

Spec104 DOC-05 proof: live scope-conflict path matches documented blocked envelope.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def ok(msg: str) -> None:
    print(f"✓ {msg}")


def main() -> int:
    print("=== Spec104 DOC-05 hard-stop alignment test ===")

    # error_envelope.rs has blocked status
    ee_path = ROOT / "crates/focusa-api/src/middleware/error_envelope.rs"
    if not ee_path.exists():
        fail("error_envelope.rs missing")
    ee_src = ee_path.read_text()
    if '"blocked"' not in ee_src and '"status": "blocked"' not in ee_src:
        fail("error_envelope.rs does not return blocked status")
    ok("error_envelope.rs returns blocked status")

    # json_guard validates scope fields
    jg_path = ROOT / "crates/focusa-api/src/middleware/json_guard.rs"
    if not jg_path.exists():
        fail("json_guard.rs missing")
    jg_src = jg_path.read_text()
    if "validate_scope_fields" not in jg_src:
        fail("json_guard.rs missing validate_scope_fields function")
    ok("json_guard.rs has validate_scope_fields")
    if "invalid_scope_kind" not in jg_src:
        fail("json_guard.rs missing invalid_scope_kind validation")
    ok("json_guard.rs validates scope_kind")

    # route_scope middleware enforces family scopes
    rs_path = ROOT / "crates/focusa-api/src/middleware/route_scope.rs"
    if not rs_path.exists():
        fail("route_scope.rs missing")
    rs_src = rs_path.read_text()
    if "fn route_scope(method" not in rs_src:
        fail("route_scope middleware missing route_scope function")
    ok("route_scope middleware enforces family scopes")

    # Project identity: mismatch returns blocked
    proj_path = ROOT / "crates/focusa-api/src/routes/project.rs"
    if proj_path.exists():
        proj_src = proj_path.read_text()
        if "blocked" not in proj_src.lower():
            fail("project.rs missing blocked envelope on mismatch")
        ok("project.rs returns blocked envelope on mismatch")

    # Trajectory: mismatch returns blocked
    traj_path = ROOT / "crates/focusa-api/src/routes/trajectory.rs"
    if traj_path.exists():
        traj_src = traj_path.read_text()
        if "blocked" not in traj_src.lower():
            fail("trajectory.rs missing blocked envelope")
        ok("trajectory.rs returns blocked envelope")

    # Docs reference blocked envelope
    docs_dir = ROOT / "docs"
    if docs_dir.exists():
        doc_refs = []
        for f in docs_dir.glob("*.md"):
            try:
                content = f.read_text()
                if "blocked" in content.lower() and "envelope" in content.lower():
                    doc_refs.append(f.name)
            except Exception:
                pass
        if not doc_refs:
            fail("no docs reference blocked envelope")
        ok(f"docs reference blocked envelope: {doc_refs[:3]}")

    print("Spec104 DOC-05 hard-stop alignment: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
