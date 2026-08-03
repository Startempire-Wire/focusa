#!/usr/bin/env python3
"""Generate Spec 135G-5 browser context isolation contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-browser-context-isolation.v1.json"
C={
 "schema":"focusa.spec135.browser_context_isolation.v1",
 "acceptance_criteria":"Browser proof scenarios prevent accidental shared state and preserve target provenance.",
 "isolation_classes":["shared_authenticated","isolated_authenticated","ephemeral_isolated","read_only_observer","capture_worker"],
 "binding_identity":["project_root","continuity_id","attachment_id","work_surface_id","browser_context_ref","browser_target_ref"],
 "ownership":{"context_owner":"exact attachment","target_owner":"exact attachment binding","shared_context":"explicit action + visible badge only","separate_tabs_imply_isolation":False},
 "safeguards":["Cross-attachment target move requires preview and confirmation","Context cookie/storage/permission state never copied implicitly","Target provenance retains original session/context/target refs","Separate targets inside one context do not constitute container isolation","Shared context badge remains visible on every bound Work Surface"],
 "cleanup":{"retention_policies":["persistent","dispose_on_close","manual"],"ephemeral_dispose_on_close":True,"close_view_does_not_close_persistent_context":True},
 "implementation_refs":["crates/focusa-core/src/types.rs::MissionCanvasBrowserIsolationClass","crates/focusa-core/src/types.rs::MissionCanvasSurfaceBindingRecord","crates/focusa-api/src/routes/mission_canvas_surfaces.rs"],
 "proof_refs":["tests/spec135_m5_browser_context_isolation_test.py","tests/spec135_m5_browser_context_isolation_e2e_test.py"],
}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135G-5 browser context isolation generated")
