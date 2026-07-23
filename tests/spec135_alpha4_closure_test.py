#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(name):
    return json.loads((C / name).read_text())


def main():
    p = j("spec135-alpha4-work-rail-proof.json")
    assert p["status"] == "passed"
    assert [x["requirement"] for x in p["composition"]] == [
        "SPEC135-ALPHA3",
        "SPEC135-ST4",
        "SPEC135-M2",
    ]
    for item in p["composition"]:
        assert j(item["proof"])["status"] == "passed"
    assert p["authority_dimensions"] == [
        "project_root",
        "working_subpath_id",
        "continuity_id",
        "attachment_id",
        "provider_item_id",
        "workpoint_id",
    ]
    assert p["remote_validation"]["rust"] == "passed"
    assert p["closure_receipt"].startswith("receipt:spec135-alpha4:")
    reducer = (R / "crates/focusa-core/src/reducer.rs").read_text()
    rail = (R / "crates/focusa-api/src/routes/work_rail.rs").read_text()
    widget = (R / "apps/pi-extension/src/work-rail-widget.ts").read_text()
    assert (
        "Workpoint-linked proof" in reducer
        and "receipt:work-rail-closure" in rail
        and "proofCount" in widget
        and "nextAction" in widget
    )
    print("Spec 135 Alpha 4 Workpoint/Work Rail/Evidence/Receipt closure: PASS")


if __name__ == "__main__":
    main()
