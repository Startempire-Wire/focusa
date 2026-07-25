#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
source = (R / "crates/focusa-core/src/google_drive_connector.rs").read_text()
lib = (R / "crates/focusa-core/src/lib.rs").read_text()
contract = json.loads(
    (R / "docs/contracts/spec135-c4-google-drive-connector.v1.yaml").read_text()
)
for marker in (
    "GoogleDriveConnector",
    "GoogleDriveDiscovery",
    "GoogleDriveImportCandidate",
    "DRIVE_FILES_URL",
    "bearer_auth(token)",
    "page_size.clamp(1, 100)",
    'schema: "focusa.workspace_artifact_intake.request.v1"',
    'source_system: "connector"',
    "self.auth.access_token()",
    "self.auth.revoke()",
):
    assert marker in source
for scope in ("project_root", "continuity_id", "attachment_id", "connector_id"):
    assert scope in source
for forbidden in ("println!", "dbg!", "access_token: String", "client_secret"):
    assert forbidden not in source
assert "pub mod google_drive_connector;" in lib
assert contract["context_pipeline_handoff"]["same_pipeline"]
assert contract["security"]["tokens_in_records_or_evidence"] is False
print("Spec 135 C4 Google Drive typed connector strict lint: PASS")
