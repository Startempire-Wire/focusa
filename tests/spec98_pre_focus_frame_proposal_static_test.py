#!/usr/bin/env python3
"""Spec98/99 focusa-7766.5: noncanonical PRE focus-frame proposal route guard."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.3-focus-frame-beads-validation-contract.yaml"
PROPOSALS = ROOT / "crates/focusa-api/src/routes/proposals.rs"
FOCUS = ROOT / "crates/focusa-api/src/routes/focus.rs"
DAEMON = ROOT / "crates/focusa-core/src/runtime/daemon.rs"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    pre = ((data.get("implemented_slice") or {}).get("pre_focus_frame_proposal") or {})
    for field in ["route", "schema", "response_schema", "canonical", "proposal_only", "side_effects", "event_path"]:
        if pre.get(field) in (None, "", []):
            fail(f"worksheet PRE proposal slice missing {field}")
    if pre.get("canonical") is not False or pre.get("proposal_only") is not True:
        fail("PRE proposal worksheet must be noncanonical proposal_only")

    proposals = PROPOSALS.read_text()
    for term in [
        "struct FocusFrameProposalRequest",
        "focusa.pre_focus_frame_proposal.v1",
        "focusa.pre_focus_frame_proposal_response.v1",
        "submit_focus_frame_proposal",
        "ProposalKind::FocusChange",
        "proposal_only",
        "noncanonical",
        "PRE ProposalSubmitted; no FocusFramePushed emitted by this route",
        "pre_proposal_append",
        '"/v1/proposals/focus-frame"',
    ]:
        if term not in proposals:
            fail(f"proposals route missing explicit PRE focus-frame term: {term}")
    handler = proposals[proposals.find("async fn submit_focus_frame_proposal"):proposals.find("/// POST /v1/proposals — submit", proposals.find("async fn submit_focus_frame_proposal"))]
    if "FocusaEvent::FocusFramePushed" in handler or "materialize_focus_event" in handler:
        fail("PRE focus-frame proposal handler must not emit/materialize FocusFramePushed")

    for name, text in [("focus.rs", FOCUS.read_text()), ("daemon.rs", DAEMON.read_text())]:
        if "beads_issue_exists" not in text or "FocusFramePushed" not in text:
            fail(f"canonical FocusFramePushed validation guard missing in {name}")

    if "tests/spec98_pre_focus_frame_proposal_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run PRE focus-frame proposal guard")
    if "tests/spec98_pre_focus_frame_proposal_static_test.py" not in PROOF_SUITE.read_text():
        fail("proof suite static contract does not include PRE focus-frame proposal guard")
    print("✓ PASS: Spec98 PRE focus-frame proposal route is explicit noncanonical schema")


if __name__ == "__main__":
    main()
