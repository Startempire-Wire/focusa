#!/usr/bin/env python3
"""Classify preserved agent/archive refs against the locked-release integration commit."""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JSON_OUT = ROOT / "docs/evidence/release/141-archived-worktree-ref-semantic-diff.json"
MD_OUT = ROOT / "docs/evidence/release/141-archived-worktree-ref-semantic-diff.md"
SECURITY_PREFIXES = (
    ".github/",
    "release",
    "scripts/release",
    "crates/focusa-license",
    "crates/focusa-installer",
)
SUPERSESSION = {
    "agents/canary-a42": {
        "disposition": "evidence_superseded",
        "rationale": "The locked release implements the rollover materialization and recovery contract under the newer Spec130A lifecycle.",
        "evidence_refs": [
            "apps/pi-extension/tests/spec130-rollover-command-lifecycle.test.mjs",
            "tests/spec130a_proactive_compaction_runtime_test.sh",
            "tests/spec130a_release_stress_runtime_test.mts",
        ],
    },
    "agents/focusa-bootstrap-w2": {
        "disposition": "evidence_superseded",
        "rationale": "Project Bootstrap discipline, broad-root rejection, provider isolation, and post-bind verification are covered by the locked Spec143 implementation.",
        "evidence_refs": [
            "tests/spec143_project_bootstrap_release_gate_test.py",
            "tests/spec96_broad_root_scope_isolation_static_test.sh",
        ],
    },
    "agents/focusa-genesis-w2": {
        "disposition": "evidence_superseded",
        "rationale": "Atomic Genesis activation, first-Workpoint creation, ambient bootstrap, and warm resume are covered by the locked Spec143 implementation.",
        "evidence_refs": [
            "tests/spec143_project_genesis_release_gate_test.py",
            "tests/spec143_project_bootstrap_release_gate_test.py",
        ],
    },
    "agents/spark-h2-runtime-matrix": {
        "disposition": "evidence_superseded",
        "rationale": "The strict native updater contract matrix is covered by the locked Spec132 runtime and portable-binary gates.",
        "evidence_refs": [
            "tests/spec132_portable_binary_selection_test.sh",
            "tests/spec132_pty_lifecycle_runtime_test.sh",
            "tests/spec132_public_uninstall_preservation_test.sh",
        ],
    },
    "agents/worker-env-inventory": {
        "disposition": "evidence_superseded",
        "rationale": "The locked installer publishes and tests the complete OS, architecture, shell, package-manager, privilege, PATH, install, license, and policy inventory.",
        "evidence_refs": [
            "tests/spec128_installer_preflight_static_test.sh",
            "crates/focusa-cli/src/commands/install.rs",
        ],
    },
    "archive/focusa-api-scope": {
        "disposition": "evidence_superseded",
        "rationale": "The locked release closes API singleton scope across typed project/workstream identity with hostile-scope and restart coverage.",
        "evidence_refs": [
            "tests/spec104_api_scope_singleton_closure_static_test.py",
            "tests/security_api_route_scope_dynamic_test.sh",
            "tests/spec96_broad_root_scope_isolation_static_test.sh",
        ],
    },
    "archive/focusa-pi-scope": {
        "disposition": "evidence_superseded",
        "rationale": "The locked release replaces Pi singleton shadows with typed attachment and project/workstream scope stores plus lifecycle isolation tests.",
        "evidence_refs": [
            "apps/pi-extension/tests/spec104-attachment-runtime-isolation.test.mjs",
            "tests/spec104_pi_runtime_scope_integrity_test.sh",
            "tests/spec98_pi_scope_cache_switch_handling_runtime_test.mts",
        ],
    },
    "archive/focusa-spec132": {
        "disposition": "evidence_superseded",
        "rationale": "The archived Spec132 draft and fixture proofs are superseded by the final locked runtime, ownership, portability, and uninstall-preservation gates.",
        "evidence_refs": [
            "tests/spec132_pi_extension_ownership_test.sh",
            "tests/spec132_portable_binary_selection_test.sh",
            "tests/spec132_pty_lifecycle_runtime_test.sh",
            "tests/spec132_public_uninstall_preservation_test.sh",
        ],
    },
    "archive/focusa-spec133": {
        "disposition": "evidence_superseded",
        "rationale": "The archived phase-zero and Pi scope work is superseded by the complete locked Spec133 supervision, isolation, evidence, and operator gates.",
        "evidence_refs": [
            "tests/spec133_phase4_runtime_gate.sh",
            "tests/spec133_phase5_isolation_gate.sh",
            "tests/spec133_phase6_evidence_gate.sh",
            "tests/spec133_phase7_operator_gate.sh",
        ],
    },
}


def git(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args], cwd=ROOT, text=True, capture_output=True, check=check
    )


def sha256(value: object) -> str:
    body = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(body).hexdigest()


def preserved_refs() -> list[str]:
    result = git(
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/heads/agents",
        "refs/heads/archive",
    )
    return sorted(line for line in result.stdout.splitlines() if line)


def is_ancestor(ref: str, head: str) -> bool:
    return git("merge-base", "--is-ancestor", ref, head, check=False).returncode == 0


def classify(ref: str, head: str) -> dict[str, object]:
    ref_sha = git("rev-parse", ref).stdout.strip()
    merge_base = git("merge-base", head, ref).stdout.strip()
    cherry = [line for line in git("cherry", head, ref).stdout.splitlines() if line]
    unique = [line[2:] for line in cherry if line.startswith("+")]
    equivalent = [line[2:] for line in cherry if line.startswith("-")]
    paths = sorted(
        set(git("diff", "--name-only", f"{head}...{ref}").stdout.splitlines())
    )
    conflict_paths: list[str] = []
    if unique:
        merge = git("merge-tree", "--write-tree", head, ref, check=False)
        if merge.returncode != 0:
            conflict_paths = sorted(
                {
                    line.split("CONFLICT", 1)[-1].strip()
                    for line in (merge.stdout + merge.stderr).splitlines()
                    if "CONFLICT" in line
                }
            )
    ancestor = is_ancestor(ref, head)
    if ancestor:
        classification = "obsolete_integrated_ancestor"
    elif not unique and equivalent:
        classification = "patch_equivalent"
    elif conflict_paths:
        classification = "conflicting_candidate"
    else:
        classification = "unique_candidate"
    security_sensitive = any(path.startswith(SECURITY_PREFIXES) for path in paths)
    settlement = SUPERSESSION.get(ref)
    return {
        "ref": ref,
        "ref_sha": ref_sha,
        "merge_base": merge_base,
        "classification": classification,
        "unique_commit_count": len(unique),
        "patch_equivalent_commit_count": len(equivalent),
        "unique_commits": unique,
        "changed_paths": paths,
        "conflict_markers": conflict_paths,
        "security_sensitive": security_sensitive,
        "settlement": settlement,
    }


def build(head: str | None = None) -> dict[str, object]:
    head = head or git("rev-parse", "HEAD").stdout.strip()
    rows = [classify(ref, head) for ref in preserved_refs()]
    counts: dict[str, int] = {}
    for row in rows:
        key = str(row["classification"])
        counts[key] = counts.get(key, 0) + 1
    unsettled = [
        row
        for row in rows
        if row["classification"] in {"unique_candidate", "conflicting_candidate"}
        and row["settlement"] is None
    ]
    payload: dict[str, object] = {
        "schema": "focusa.archived_worktree_ref_semantic_diff.v1",
        "status": "review_required" if unsettled else "verified",
        "unsettled_ref_count": len(unsettled),
        "locked_release_head": head,
        "ref_namespaces": ["refs/heads/agents/*", "refs/heads/archive/*"],
        "preserved_bundle": {"status": "not_present", "searched_roots": ["/root", "/tmp"]},
        "classification_counts": counts,
        "refs": rows,
    }
    payload["evidence_digest"] = sha256(payload)
    return payload


def markdown(payload: dict[str, object]) -> str:
    rows = payload["refs"]
    lines = [
        "# 141 — Focusa Archived Worktree and Ref Semantic-Diff Evidence",
        "",
        f"- Locked release head: `{payload['locked_release_head']}`",
        f"- Status: `{payload['status']}`",
        f"- Evidence digest: `{payload['evidence_digest']}`",
        "- Preserved bundle: none present under the operator-declared search roots; refs remain preserved in Git.",
        "",
        f"- Unsettled refs: `{payload['unsettled_ref_count']}`",
        "",
        "| Ref | Classification | Unique | Equivalent | Conflicts | Security-sensitive | Settlement |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    assert isinstance(rows, list)
    for row in rows:
        assert isinstance(row, dict)
        lines.append(
            f"| `{row['ref']}` | `{row['classification']}` | {row['unique_commit_count']} | "
            f"{row['patch_equivalent_commit_count']} | {len(row['conflict_markers'])} | "
            f"{'yes' if row['security_sensitive'] else 'no'} | "
            f"{row['settlement']['disposition'] if row['settlement'] else 'not_required'} |"
        )
    lines.extend(
        [
            "",
            "## Classification policy",
            "",
            "- `obsolete_integrated_ancestor`: ref tip is an ancestor of the locked release head.",
            "- `patch_equivalent`: all non-ancestor ref commits have patch-equivalent commits in the locked release.",
            "- `unique_candidate`: unique patch identity remains and requires explicit integration or supersession evidence.",
            "- `conflicting_candidate`: trial merge reports conflicts and requires explicit integration or supersession evidence.",
            "- Security-sensitive is a review tag, never an automatic integration authorization.",
            "- A conflicting or unique ref is settled only by an explicit integration or evidence-supersession record with stable proof refs.",
            "",
            "## Supersession evidence",
            "",
        ]
    )
    for row in rows:
        assert isinstance(row, dict)
        if row["settlement"]:
            settlement = row["settlement"]
            lines.append(f"### `{row['ref']}`")
            lines.append("")
            lines.append(str(settlement["rationale"]))
            lines.append("")
            lines.extend(f"- `{ref}`" for ref in settlement["evidence_refs"])
            lines.append("")
    lines.extend(
        [
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    pinned_head = None
    if args.check and JSON_OUT.exists():
        pinned_head = json.loads(JSON_OUT.read_text()).get("locked_release_head")
    payload = build(pinned_head)
    json_body = json.dumps(payload, indent=2) + "\n"
    md_body = markdown(payload)
    if args.check:
        if not JSON_OUT.exists() or JSON_OUT.read_text() != json_body:
            raise SystemExit(f"stale semantic-diff evidence: {JSON_OUT}")
        if not MD_OUT.exists() or MD_OUT.read_text() != md_body:
            raise SystemExit(f"stale semantic-diff evidence: {MD_OUT}")
    else:
        JSON_OUT.parent.mkdir(parents=True, exist_ok=True)
        JSON_OUT.write_text(json_body)
        MD_OUT.write_text(md_body)
    print(json.dumps({"status": payload["status"], "counts": payload["classification_counts"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
