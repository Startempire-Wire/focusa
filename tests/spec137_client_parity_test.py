#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
cli = (R / "crates/focusa-cli/src/commands/temporal_clients.rs").read_text()
main = (R / "crates/focusa-cli/src/main.rs").read_text()
api = (R / "crates/focusa-api/src/routes/temporal_clients.rs").read_text()
pi = (R / "apps/pi-extension/src/tools.ts").read_text()
tui = (R / "crates/focusa-tui/src/mission_control.rs").read_text()
menubar = (R / "apps/menubar/src/lib/components/TemporalAuthorityPeek.svelte").read_text()
ts = (R / "packages/generated/spec135/typescript/temporal.ts").read_text()
rust = (R / "packages/generated/spec135/rust/src/temporal.rs").read_text()
registry = json.loads((R / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text())
openapi = json.loads((R / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text())

families = ["Time", "Deadline", "Estimate", "Progress", "NoProgress", "LostTime", "Opportunity", "Cancellation"]
for family in families:
    assert f"{family}(commands::temporal_clients::" in main, family

operations = {
    "focusa.time.now": ("GET", "/v1/time/now"),
    "focusa.time.status": ("GET", "/v1/time/status"),
    "focusa.deadline.set": ("POST", "/v1/deadline/set"),
    "focusa.deadline.revise": ("POST", "/v1/deadline/revise"),
    "focusa.deadline.clear": ("POST", "/v1/deadline/clear"),
    "focusa.deadline.inspect": ("GET", "/v1/deadline/{id}"),
    "focusa.estimate.request": ("POST", "/v1/estimate/request"),
    "focusa.estimate.validate": ("POST", "/v1/estimate/validate"),
    "focusa.estimate.evaluate": ("POST", "/v1/estimate/evaluate"),
    "focusa.progress.record": ("POST", "/v1/progress/record"),
    "focusa.progress.status": ("GET", "/v1/progress/status"),
    "focusa.no_progress.inspect": ("GET", "/v1/no-progress/incidents"),
    "focusa.lost_time.list": ("GET", "/v1/lost-time/incidents"),
    "focusa.opportunity.inspect": ("GET", "/v1/opportunities/{subject}"),
    "focusa.cancellation.inspect": ("GET", "/v1/cancellation/{id}"),
}
by_id = {row["operation_id"]: row for row in registry["operations"]}
for operation_id, (method, path) in operations.items():
    row = by_id[operation_id]
    assert (row["method"], row["path"], row["canonical"]) == (method, path, True)
    assert openapi["paths"][path][method.lower()]["operationId"] == operation_id
    assert path.replace("{id}", "{id}").replace("{subject}", "{subject}") in api

for marker in ["confirmation_required", "deadline_evidence_required", "expected_revision_required", "progress_evidence_required", "estimate_evidence_required"]:
    assert marker in api
for marker in ["createTemporalClient", "invalid_temporal_response_envelope", "DeadlineSetRequest", "ProgressRecordRequest"]:
    assert marker in ts
for marker in ["TemporalClient", "set_deadline", "record_progress", '"/v1/deadline/set"']:
    assert marker in rust
for action in ["time-now", "deadline-inspect", "progress-status", "lost-time-inspect", "opportunity-inspect", "cancellation-inspect"]:
    assert f'Type.Literal("{action}")' in pi
for label in ["Last progress", "No-progress age", "Lost-time incidents", "Opportunity", "Cancellation"]:
    assert label in menubar
for marker in ["no_progress_ms=", "lost_time=", "opportunity=", "cancellation="]:
    assert marker in tui
assert len((R / "crates/focusa-api/src/routes/temporal_clients.rs").read_text().splitlines()) < 500
assert len((R / "crates/focusa-cli/src/commands/temporal_clients.rs").read_text().splitlines()) < 500
print("Spec137 client parity: PASS")
