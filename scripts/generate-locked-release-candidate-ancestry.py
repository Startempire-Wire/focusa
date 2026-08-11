#!/usr/bin/env python3
"""Generate the GH#106.4 immutable tag and candidate ancestry receipt."""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import importlib.util
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
INVENTORY = AUDIT / "next-locked-release-governance-inventory.json"
RECONCILIATION = AUDIT / "next-locked-release-governance-reconciliation.json"
TECHNICAL_CLOSURE = AUDIT / "next-locked-release-technical-closure-gate.json"
EVIDENCE_LINKS = AUDIT / "next-locked-release-governance-evidence-links.json"
ASSET_SNAPSHOT = AUDIT / "next-locked-release-v09143-published-assets.json"
OUTPUT = AUDIT / "next-locked-release-candidate-ancestry.json"

VERSION_PATHS = (
    ("workspace", "Cargo.toml", re.compile(r'^version = "([^"]+)"$', re.MULTILINE)),
    ("menubar", "apps/menubar/package.json", None),
    ("pi_extension", "apps/pi-extension/package.json", None),
)
TAG_CHAINS = {
    "stable": (
        "v0.9.136",
        "v0.9.137",
        "v0.9.138",
        "v0.9.139",
        "v0.9.140",
        "v0.9.141",
        "v0.9.142",
        "v0.9.143",
    ),
    "dev": (
        "v0.9.135-dev",
        "v0.9.136-dev",
        "v0.9.137-dev",
    ),
}
ALLOWED_RECEIPT_PATHS = {
    ".github/workflows/locked-release-candidate-artifacts.yml",
    ".github/workflows/windows-ota-e2e.yml",
    "crates/focusa-cli/src/commands/install.rs",
    "crates/focusa-cli/src/commands/update.rs",
    "release-proof/audit/next-locked-release-candidate-ancestry.json",
    "release-proof/audit/next-locked-release-github106-closure-proof.json",
    "release-proof/audit/next-locked-release-governance-evidence-links.json",
    "release-proof/audit/next-locked-release-governance-receipt.json",
    "release-proof/audit/next-locked-release-governance-receipt.json.sig",
    "release-proof/audit/next-locked-release-governance-reconciliation.json",
    "release-proof/audit/next-locked-release-technical-closure-gate.json",
    "release-proof/audit/next-locked-release-v09143-published-assets.json",
    "scripts/generate-locked-release-candidate-ancestry.py",
    "scripts/generate-locked-release-governance-receipt.py",
    "tests/166_focusa_locked_release_candidate_ancestry_test.py",
    "tests/167_focusa_locked_release_governance_receipt_test.py",
    "tests/168_focusa_windows_native_ota_workflow_test.py",
    "tests/169_focusa_rel4_candidate_artifact_workflow_test.py",
    "tests/final_release_gap_gate.sh",
}


def git(*args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, text=True, capture_output=True, check=False
    )
    if check and result.returncode:
        raise SystemExit(result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout.strip()


def is_ancestor(older: str, newer: str) -> bool:
    return (
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", older, newer],
            cwd=ROOT,
            check=False,
        ).returncode
        == 0
    )


def digest_value(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def file_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_versions(ref: str) -> dict[str, str]:
    versions = {}
    for surface, path, pattern in VERSION_PATHS:
        content = git("show", f"{ref}:{path}")
        if pattern:
            match = pattern.search(content)
            if not match:
                raise SystemExit(f"version missing from {ref}:{path}")
            versions[surface] = match.group(1)
        else:
            versions[surface] = json.loads(content)["version"]
    return versions


def tags_before_candidate_publication(
    all_tags: list[str], candidate_tag: str, candidate_tag_commit: str, audit_head: str
) -> list[str]:
    """Keep a pre-tag receipt stable after its exact audited tag is published."""
    if candidate_tag_commit == audit_head:
        return [tag for tag in all_tags if tag != candidate_tag]
    return all_tags


def build(candidate_ref: str, audit_ref: str) -> dict:
    candidate = git("rev-parse", f"{candidate_ref}^{{commit}}")
    head = git("rev-parse", f"{audit_ref}^{{commit}}")
    inventory = json.loads(INVENTORY.read_text())
    reconciliation = json.loads(RECONCILIATION.read_text())
    technical_closure = json.loads(TECHNICAL_CLOSURE.read_text())
    evidence_links = json.loads(EVIDENCE_LINKS.read_text())
    if (
        technical_closure["reconciliation_digest"]
        != reconciliation["reconciliation_digest"]
    ):
        raise SystemExit("technical closure gate does not bind current reconciliation")
    assets = json.loads(ASSET_SNAPSHOT.read_text())

    tag_records = {}
    ancestry_errors = []
    for channel, tags in TAG_CHAINS.items():
        channel_records = []
        previous_commit = None
        for tag in tags:
            object_id = git("rev-parse", tag)
            commit = git("rev-parse", f"{tag}^{{commit}}")
            object_type = git("cat-file", "-t", tag)
            follows_previous = previous_commit is None or is_ancestor(
                previous_commit, commit
            )
            if not follows_previous:
                ancestry_errors.append(f"non_linear_{channel}_tag:{tag}")
            channel_records.append(
                {
                    "tag": tag,
                    "object_type": object_type,
                    "object_id": object_id,
                    "commit": commit,
                    "follows_previous": follows_previous,
                }
            )
            previous_commit = commit
        tag_records[channel] = channel_records

    immutable_tag_commit = git("rev-parse", "v0.9.143^{commit}")
    if not is_ancestor(immutable_tag_commit, candidate):
        ancestry_errors.append("v0.9.143_not_ancestor_of_candidate")

    accepted_commits = sorted(
        {
            ref.removeprefix("git:")
            for row in evidence_links["links"]
            for ref in row.get("implementation_commit_refs", [])
        }
    )
    missing_accepted_commits = [
        commit for commit in accepted_commits if not is_ancestor(commit, candidate)
    ]

    changed_after_candidate = [
        path
        for path in git("diff", "--name-only", f"{candidate}..{head}").splitlines()
        if path
    ]
    unexpected_after_candidate = sorted(
        set(changed_after_candidate) - ALLOWED_RECEIPT_PATHS
    )

    versions = source_versions(candidate)
    candidate_versions = set(versions.values())
    version_agreement = len(candidate_versions) == 1
    published_version = "0.9.143"

    version_module = load_module(
        ROOT / "scripts" / "select-release-version.py", "focusa_version_selection"
    )
    candidate_version = next(iter(candidate_versions)) if version_agreement else ""
    candidate_tag = f"v{candidate_version}" if candidate_version else ""
    candidate_tag_commit = (
        git("rev-parse", f"{candidate_tag}^{{commit}}", check=False)
        if candidate_tag
        else ""
    )
    all_tags = tags_before_candidate_publication(
        git("tag", "--list").splitlines(), candidate_tag, candidate_tag_commit, head
    )
    next_selection = version_module.select_version("0.9", None, all_tags)
    next_stable_tag = f"v0.9.{next_selection['selected_patch']}"
    next_stable_version = next_stable_tag.removeprefix("v")
    if not version_agreement:
        ancestry_errors.append("candidate_source_versions_disagree")
    elif candidate_versions not in ({published_version}, {next_stable_version}):
        ancestry_errors.append("candidate_source_version_not_immutable_or_next_stable")

    asset_module = load_module(
        ROOT / "scripts" / "verify-canonical-release-assets.py",
        "focusa_canonical_assets",
    )
    asset_names = {row["name"] for row in assets["assets"]}
    missing_assets = [
        name
        for name in asset_module.required_exact("v0.9.143")
        if name not in asset_names
    ]
    missing_assets.extend(
        f"pattern:{pattern}"
        for pattern in asset_module.required_patterns()
        if not any(fnmatch.fnmatchcase(name, pattern) for name in asset_names)
    )
    manifest_assets = {
        name: next((row for row in assets["assets"] if row["name"] == name), None)
        for name in (
            "release-manifest.json",
            "release-manifest.json.sig",
            "release-provenance.json",
            "release-provenance.json.sig",
            "SHA256SUMS.txt",
        )
    }

    immutable_ids = {
        row["bead_id"]
        for row in reconciliation["mappings"]
        if row["authority"] == "immutable_workset_r7"
    }
    excluded_ids = {row["bead_id"] for row in inventory["excluded_reconstructed_epics"]}
    excluded_workset_collisions = sorted(immutable_ids & excluded_ids)

    commit_lines = git(
        "log", "--format=%H%x09%s", f"v0.9.142..{candidate}"
    ).splitlines()
    pr_records = []
    for line in commit_lines:
        commit, _, subject = line.partition("\t")
        for number in sorted({int(value) for value in re.findall(r"#(\d+)", subject)}):
            pr_records.append({"pr": number, "commit": commit, "subject": subject})

    release_blockers = []
    source_is_immutable_version = candidate_versions == {published_version}
    if missing_assets and source_is_immutable_version:
        release_blockers.append("immutable_v0.9.143_missing_required_assets")
    if source_is_immutable_version:
        release_blockers.append(f"source_version_must_advance_to_{next_stable_tag}")
    if technical_closure["technically_pending_count"]:
        release_blockers.append("technical_acceptance_pending")
    if technical_closure["invalid_closed_count"]:
        release_blockers.append("invalid_technical_closure")

    audit_errors = (
        ancestry_errors
        + missing_accepted_commits
        + unexpected_after_candidate
        + excluded_workset_collisions
    )
    result = {
        "schema": "focusa.locked_release_candidate_ancestry.v1",
        "status": "verified" if not audit_errors else "blocked",
        "release_ready": not audit_errors and not release_blockers,
        "candidate": {
            "source_commit": candidate,
            "audit_head": head,
            "source_is_ancestor_of_audit_head": is_ancestor(candidate, head),
            "changed_after_source_commit": changed_after_candidate,
            "unexpected_changes_after_source_commit": unexpected_after_candidate,
        },
        "immutable_release": {
            "tag": "v0.9.143",
            "commit": immutable_tag_commit,
            "disposition": "immutable_incomplete_never_rewrite",
            "published_asset_snapshot": str(ASSET_SNAPSHOT.relative_to(ROOT)),
            "published_asset_snapshot_digest": file_digest(ASSET_SNAPSHOT),
            "published_asset_count": len(assets["assets"]),
            "missing_required_assets": missing_assets,
            "manifest_assets": manifest_assets,
        },
        "tag_chain": tag_records,
        "source_versions": versions,
        "source_version_agreement": version_agreement,
        "next_version_selection": next_selection,
        "next_stable_tag": next_stable_tag,
        "accepted_implementation_commits": accepted_commits,
        "missing_accepted_implementation_commits": missing_accepted_commits,
        "pull_request_ancestry": pr_records,
        "issue_ledger": {
            "workset_id": inventory["workset_id"],
            "inventory_digest": inventory["inventory_digest"],
            "reconciliation_digest": reconciliation["reconciliation_digest"],
            "technical_closure_gate_digest": technical_closure["gate_digest"],
            "mapping_count": reconciliation["admitted_mapping_count"],
            "technically_accepted_count": technical_closure[
                "technically_accepted_count"
            ],
            "pending_technical_acceptance_count": technical_closure[
                "technically_pending_count"
            ],
            "invalid_closed_count": technical_closure["invalid_closed_count"],
        },
        "excluded_post_release_worksets": sorted(excluded_ids),
        "excluded_workset_collisions": excluded_workset_collisions,
        "release_blockers": release_blockers,
        "audit_errors": audit_errors,
        "mutation_policy": {
            "rewrite_v0.9.143": False,
            "create_tag": False,
            "publish_release": False,
        },
    }
    result["ancestry_digest"] = digest_value(result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-ref", required=True)
    parser.add_argument("--audit-ref", required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = (
        json.dumps(build(args.candidate_ref, args.audit_ref), indent=2, sort_keys=True)
        + "\n"
    )
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered:
            print(f"candidate ancestry drift: regenerate {OUTPUT.relative_to(ROOT)}")
            return 1
        candidate = git("rev-parse", f"{args.candidate_ref}^{{commit}}")
        current_changes = {
            path
            for path in git("diff", "--name-only", f"{candidate}..HEAD").splitlines()
            if path
        }
        unexpected = sorted(current_changes - ALLOWED_RECEIPT_PATHS)
        if unexpected:
            print(f"candidate ancestry has post-audit source changes: {unexpected}")
            return 1
        print("locked-release candidate ancestry: PASS")
        return 0
    OUTPUT.write_text(rendered)
    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
