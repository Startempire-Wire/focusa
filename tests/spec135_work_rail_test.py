#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(name):
    return json.loads((C / name).read_text())


def main():
    t = (R / "crates/focusa-core/src/types.rs").read_text()
    r = (R / "crates/focusa-core/src/reducer.rs").read_text()
    a = (R / "crates/focusa-api/src/routes/work_rail.rs").read_text()
    u = (R / "packages/a2ui-renderer/proof/work-rail.ts").read_text()
    for x in [
        "WorkRailRecord",
        "WorkRailStatus",
        "VerifiedComplete",
        "ProviderClosedFocusaUnverified",
    ]:
        assert x in t
    for x in [
        "project, working sub-path, continuity, and Bead",
        "Workpoint-linked proof",
        "closure claim, and Receipt",
        "WorkRailRevised",
    ]:
        assert x in r
    for x in [
        "RailAction",
        "VerifyClose",
        "close_bead",
        "working_subpath_id",
        "evidence_refs",
        "receipt:work-rail-closure",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    op = ops["focusa.work_rail.mutate"]
    assert (
        op["scope"]["required_keys"]
        == ["project_root", "working_subpath_id", "continuity_id", "attachment_id"]
        and op["control"]["idempotency_required"]
        and op["control"]["receipt_required"]
    )
    for s in [
        "focusa.work_rail_list.request.v1",
        "focusa.work_rail_list.v1",
        "focusa.work_rail_mutation.request.v1",
        "focusa.work_rail_mutation_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Activate Workpoint" in u
        and "playwright" not in u.lower()
    )
    proof = j("spec135-st4-work-rail-proof.json")
    assert proof["contracts"]["operation_count"] == 79
    print("Spec 135 ST4 Work Rail static proof: PASS")


if __name__ == "__main__":
    main()
