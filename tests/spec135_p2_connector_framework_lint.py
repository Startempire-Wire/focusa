#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
source = (R / "crates/focusa-core/src/connectors.rs").read_text()
lib = (R / "crates/focusa-core/src/lib.rs").read_text()
contract = json.loads(
    (R / "docs/contracts/spec135-p2-connector-framework.v1.yaml").read_text()
)

for marker in contract["canonical_types"]:
    assert f"{marker} " in source or f"{marker} {{" in source
for marker in (
    "trait ConnectorAdapter",
    "struct HttpJsonConnector",
    "apply_rate_policy",
    "retry_statuses",
    "origin_denied",
    "capability_missing",
    "focusa.connector_result.v1",
    "focusa.connector_error.v1",
    "request.send().await",
):
    assert marker in source
for scope in ("project_root", "continuity_id", "attachment_id"):
    assert f"pub {scope}: String" in source
assert "pub mod connectors;" in lib
assert "mock" not in source.lower()
assert contract["adapter_contract"]["bounded_retry"]
assert contract["adapter_contract"]["bounded_rate"]
print("Spec 135 P2 typed connector framework strict lint: PASS")
