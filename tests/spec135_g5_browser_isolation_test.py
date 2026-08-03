#!/usr/bin/env python3
"""Spec 135G-5 browser context isolation proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-browser-context-isolation.v1.json").read_text())
T=(ROOT/"crates/focusa-core/src/types.rs").read_text()
R=(ROOT/"crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
assert C["ownership"]["separate_tabs_imply_isolation"] is False
assert C["ownership"]["shared_context"] == "explicit action + visible badge only"
assert len(C["isolation_classes"]) == 5
for rust_name in ("SharedAuthenticated","IsolatedAuthenticated","EphemeralIsolated","ReadOnlyObserver","CaptureWorker"):
    assert rust_name in T, rust_name
for field in ("attachment_id","work_surface_id","browser_isolation_class","authentication_sharing","retention_policy"):
    assert f"pub {field}" in T, field
for policy in ("persistent","dispose_on_close","manual"):
    assert policy in R, policy
assert C["cleanup"]["ephemeral_dispose_on_close"] is True
assert "Target provenance retains original session/context/target refs" in C["safeguards"]
for ref in C["proof_refs"]:
    assert (ROOT/ref).exists(), ref
print("Spec 135 G5 browser context isolation: PASS")
