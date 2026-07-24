#!/usr/bin/env python3
from pathlib import Path

R = Path(__file__).resolve().parents[1]
adapter = (R / "packages/generated/spec135/typescript/ag-ui-adapter.ts").read_text()
index = (R / "packages/generated/spec135/typescript/index.ts").read_text()
for marker in (
    "FocusaNativeStreamEvent",
    "FocusaAgUiEvent",
    "toAgUiEvent",
    "replayCursor: event.event_id",
    "Focusa's native event stream remains canonical",
    "owns no",
):
    assert marker in adapter
for scope in ("project_root", "continuity_id", "attachment_id"):
    assert scope in adapter
assert 'from "./ag-ui-adapter.js"' in index
for forbidden in ("fetch(", "localStorage", "sessionStorage", "history.push"):
    assert forbidden not in adapter
print("Spec 135 P4 stateless AG-UI compatibility adapter lint: PASS")
