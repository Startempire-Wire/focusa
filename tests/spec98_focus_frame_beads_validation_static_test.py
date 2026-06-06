#!/usr/bin/env python3
"""Spec98 focusa-877z.3: canonical FocusFramePushed requires real Beads issue validation."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.3-focus-frame-beads-validation-contract.yaml"
FOCUS = ROOT / "crates/focusa-api/src/routes/focus.rs"
DAEMON = ROOT / "crates/focusa-core/src/runtime/daemon.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def before(text: str, first: str, second: str) -> bool:
    a = text.find(first)
    b = text.find(second)
    return a != -1 and b != -1 and a < b


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.focus_frame_beads_validation_contract.v1":
        fail("unexpected .3 contract schema")
    if data.get("status") != "canonical_focus_frame_push_requires_real_project_beads_issue":
        fail("unexpected .3 contract status")

    focus = FOCUS.read_text()
    daemon = DAEMON.read_text()

    if "fn beads_issue_exists(project_root: &str, beads_issue_id: &str) -> bool" not in focus:
        fail("focus.rs must define project Beads existence validation")
    for source in ['Path::new(project_root).join(".beads/issues.jsonl")', 'Path::new(project_root).join(".git/beads-worktrees/beads-sync/.beads/issues.jsonl")']:
        if source not in focus:
            fail(f"focus.rs must validate against Beads JSONL source {source}")
    if '"canonical": false' not in focus or '"reason": "beads_issue_not_found"' not in focus:
        fail("focus.rs missing canonical=false beads_issue_not_found rejection payload")
    push_start = focus.find("async fn push_frame(")
    push_end = focus.find("#[derive(Deserialize)]\nstruct PopFrameBody", push_start)
    if push_start == -1 or push_end == -1:
        fail("could not isolate push_frame route body")
    push_body = focus[push_start:push_end]
    if not before(push_body, "if !beads_issue_exists(&project_root, &beads_issue_id)", "materialize_focus_event("):
        fail("focus.rs must validate Beads existence before materializing FocusFramePushed")
    if not before(push_body, "if !beads_issue_exists(&project_root, &beads_issue_id)", "FocusaEvent::FocusFramePushed"):
        fail("focus.rs must validate Beads existence before constructing FocusFramePushed")

    if "fn beads_issue_exists(project_root: &str, beads_issue_id: &str) -> bool" not in daemon:
        fail("daemon.rs must define Beads existence validation")
    if "canonical FocusFramePushed rejected" not in daemon:
        fail("daemon Action::PushFrame must reject missing Beads issue before event emission")
    if not before(daemon, "if !beads_issue_exists(root, &beads_issue_id)", "FocusaEvent::FocusFramePushed"):
        fail("daemon Action::PushFrame must validate Beads existence before FocusFramePushed event")

    issue_sources = [
        ROOT / ".beads/issues.jsonl",
        ROOT / ".git/beads-worktrees/beads-sync/.beads/issues.jsonl",
    ]
    if not any(path.exists() and '"id":"focusa-877z.3"' in path.read_text() for path in issue_sources):
        fail("current project Beads JSONL sources must contain focusa-877z.3 as a real issue")

    print("✓ PASS: canonical FocusFramePushed creation validates real project Beads issue IDs")


if __name__ == "__main__":
    main()
