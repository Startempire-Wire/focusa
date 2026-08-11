#!/usr/bin/env python3
"""Acceptance gate for GH#106.4 tag and candidate ancestry reconciliation."""

from __future__ import annotations

import importlib.util
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
ANCESTRY_PATH = AUDIT / "next-locked-release-candidate-ancestry.json"
ASSETS_PATH = AUDIT / "next-locked-release-v09143-published-assets.json"
SCRIPT = ROOT / "scripts" / "generate-locked-release-candidate-ancestry.py"
spec = importlib.util.spec_from_file_location("candidate_ancestry", SCRIPT)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

ancestry = json.loads(ANCESTRY_PATH.read_text())
assets = json.loads(ASSETS_PATH.read_text())
candidate = ancestry["candidate"]["source_commit"]
audit_head = ancestry["candidate"]["audit_head"]
subprocess.run(
    [
        "python3",
        str(SCRIPT),
        "--candidate-ref",
        candidate,
        "--audit-ref",
        audit_head,
        "--check",
    ],
    cwd=ROOT,
    check=True,
)

assert ancestry["schema"] == "focusa.locked_release_candidate_ancestry.v1"
assert ancestry["status"] == "verified"
assert ancestry["release_ready"] is False
assert ancestry["audit_errors"] == []
assert ancestry["candidate"]["source_is_ancestor_of_audit_head"] is True
assert ancestry["candidate"]["unexpected_changes_after_source_commit"] == []

immutable = ancestry["immutable_release"]
assert immutable["tag"] == "v0.9.143"
assert immutable["commit"] == "ac40a3a769b679e684f6592d075cabd24ab64fd5"
assert immutable["disposition"] == "immutable_incomplete_never_rewrite"
assert immutable["published_asset_count"] == len(assets["assets"]) == 66
assert all(immutable["manifest_assets"].values())
assert all(
    row["digest"].startswith("sha256:") for row in immutable["manifest_assets"].values()
)

missing = set(immutable["missing_required_assets"])
for target in ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"):
    for surface in ("focusa", "focusa-daemon", "focusa-tui"):
        assert f"{surface}-v0.9.143-{target}.exe" in missing
assert "focusa-generated-clients-v0.9.143.tar.gz" in missing
assert "focusa-installer-v0.9.143.ps1" in missing
assert all("windows" not in row["name"].lower() for row in assets["assets"])

source_versions = set(ancestry["source_versions"].values())
assert len(source_versions) == 1
source_version = next(iter(source_versions))
assert ancestry["source_version_agreement"] is True
assert ancestry["next_version_selection"]["selected_tag"] == f"v{source_version}-dev"
assert ancestry["next_stable_tag"] == f"v{source_version}"
assert ancestry["release_blockers"] == ["technical_acceptance_pending"]
assert "immutable_v0.9.143_missing_required_assets" not in ancestry["release_blockers"]
assert not any(
    blocker.startswith("source_version_must_advance_to_")
    for blocker in ancestry["release_blockers"]
)
assert module.tags_before_candidate_publication(
    ["v0.9.144", f"v{source_version}"],
    f"v{source_version}",
    audit_head,
    audit_head,
) == ["v0.9.144"]

for channel in ("stable", "dev"):
    assert ancestry["tag_chain"][channel]
    assert all(row["follows_previous"] for row in ancestry["tag_chain"][channel])
    assert all(row["object_type"] == "commit" for row in ancestry["tag_chain"][channel])
assert ancestry["tag_chain"]["stable"][-1]["tag"] == "v0.9.143"

assert ancestry["missing_accepted_implementation_commits"] == []
for commit in ancestry["accepted_implementation_commits"]:
    subprocess.run(
        ["git", "merge-base", "--is-ancestor", commit, candidate],
        cwd=ROOT,
        check=True,
    )
assert any(row["pr"] == 115 for row in ancestry["pull_request_ancestry"])
assert len(ancestry["excluded_post_release_worksets"]) == 7
assert ancestry["excluded_workset_collisions"] == []
assert ancestry["issue_ledger"]["mapping_count"] == 465
assert (
    ancestry["issue_ledger"]["technically_accepted_count"]
    + ancestry["issue_ledger"]["pending_technical_acceptance_count"]
    == ancestry["issue_ledger"]["mapping_count"]
)
assert ancestry["issue_ledger"]["pending_technical_acceptance_count"] > 0
assert ancestry["issue_ledger"]["invalid_closed_count"] == 0
assert ancestry["issue_ledger"]["technical_closure_gate_digest"].startswith("sha256:")

policy = ancestry["mutation_policy"]
assert policy == {
    "create_tag": False,
    "publish_release": False,
    "rewrite_v0.9.143": False,
}
print("GH#106.4 locked-release candidate ancestry: PASS (release remains blocked)")
