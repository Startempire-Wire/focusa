#!/usr/bin/env python3
"""Reconcile admitted locked-release work with live Beads state and proof refs."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
MEMBERS = AUDIT / "next-locked-release-workset-members.jsonl"
INVENTORY = AUDIT / "next-locked-release-governance-inventory.json"
EVIDENCE_LINKS = AUDIT / "next-locked-release-governance-evidence-links.json"
OUTPUT = AUDIT / "next-locked-release-governance-reconciliation.json"

COMMIT_RE = re.compile(
    r"(?i)(?:git:|commit(?:ted)?(?:\s+at)?\s+|\bat\s+)([0-9a-f]{8,40})\b"
)
PATH_RE = re.compile(
    r"\b(?:apps|crates|docs|release-proof|scripts|tests)/[A-Za-z0-9_.@+/-]+"
)
URL_RE = re.compile(r"https?://[^\s)\]}>;,]+")
GITHUB_ISSUE_RE = re.compile(r"(?i)(?:github\s*#|github:|/issues/)(\d+)\b")
EXACT_DUPLICATE_RE = re.compile(
    r"(?i)\bduplicate\s+of\s+([a-z0-9-]+(?:\.[0-9]+)+)(?![-.0-9])"
)
TASK_CODE_RE = re.compile(r"(?i)^P\d+:\s*([A-Z]+(?:-[A-Z0-9]+)?)\s+[—-]")


def load_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def digest_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def digest_value(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def natural_key(value: str) -> list[object]:
    return [int(part) if part.isdigit() else part for part in re.split(r"(\d+)", value)]


def evidence_from(
    member: dict, provider: dict
) -> tuple[list[str], list[str], list[str]]:
    text = "\n".join(
        str(provider.get(field) or "")
        for field in ("title", "description", "acceptance_criteria", "close_reason")
    )
    commits = {
        ref.removeprefix("git:")
        for ref in member.get("evidence_refs", [])
        if ref.startswith("git:")
    }
    commits.update(COMMIT_RE.findall(text))
    stable_refs = {
        ref
        for ref in member.get("evidence_refs", []) + member.get("receipt_refs", [])
        if not ref.startswith("git:")
    }
    stable_refs.update(PATH_RE.findall(text))
    stable_refs.update(URL_RE.findall(text))
    issues = {
        int(ref.split(":", 1)[1])
        for ref in member.get("spec_refs", [])
        if ref.startswith("github:") and ref.split(":", 1)[1].isdigit()
    }
    issues.update(int(value) for value in GITHUB_ISSUE_RE.findall(text))
    return sorted(commits), sorted(stable_refs), sorted(issues)


def provider_parent(provider: dict) -> str | None:
    parents = sorted(
        {
            dep["depends_on_id"]
            for dep in provider.get("dependencies", [])
            if dep.get("type") == "parent-child" and dep.get("depends_on_id")
        }
    )
    if len(parents) > 1:
        return "MULTIPLE:" + ",".join(parents)
    return parents[0] if parents else None


def build(provider_path: Path) -> dict:
    inventory = json.loads(INVENTORY.read_text())
    evidence_links_document = json.loads(EVIDENCE_LINKS.read_text())
    if (
        evidence_links_document.get("schema")
        != "focusa.locked_release_governance_evidence_links.v1"
    ):
        raise SystemExit("unexpected governance evidence-link schema")
    evidence_links = {
        row["bead_id"]: row for row in evidence_links_document.get("links", [])
    }
    if len(evidence_links) != len(evidence_links_document.get("links", [])):
        raise SystemExit("duplicate Bead authority in governance evidence links")
    for link in evidence_links.values():
        for ref in link.get("evidence_refs", []):
            if not (ROOT / ref).is_file():
                raise SystemExit(f"governance evidence ref is missing: {ref}")
        for ref in link.get("implementation_commit_refs", []):
            if not re.fullmatch(r"git:[0-9a-f]{40}", ref):
                raise SystemExit(f"invalid governance implementation commit ref: {ref}")
        if not link.get("evidence_refs") and not link.get("implementation_commit_refs"):
            raise SystemExit(f"empty governance evidence link: {link['bead_id']}")
    immutable = load_jsonl(MEMBERS)
    provider_rows = load_jsonl(provider_path)
    provider_by_id: dict[str, dict] = {}
    duplicate_provider_ids: list[str] = []
    for row in provider_rows:
        issue_id = row.get("id")
        if not issue_id:
            continue
        if issue_id in provider_by_id:
            duplicate_provider_ids.append(issue_id)
        provider_by_id[issue_id] = row

    overlay_roots = {
        row["bead_id"] for row in inventory["authorized_release_repair_overlay"]
    }
    overlay_ids = {
        issue_id
        for issue_id in provider_by_id
        if any(
            issue_id == root or issue_id.startswith(root + ".")
            for root in overlay_roots
        )
    }
    immutable_ids = {row["member_id"] for row in immutable}
    admitted_ids = immutable_ids | overlay_ids
    locked_label_ids = {
        row["id"] for row in provider_rows if "locked-release" in row.get("labels", [])
    }

    immutable_by_id = {row["member_id"]: row for row in immutable}
    unknown_evidence_link_ids = set(evidence_links) - admitted_ids
    if unknown_evidence_link_ids:
        raise SystemExit(
            "governance evidence links reference unadmitted Beads: "
            f"{sorted(unknown_evidence_link_ids)}"
        )
    direct_proof: dict[str, bool] = {}
    for issue_id in admitted_ids:
        provider = provider_by_id.get(issue_id, {})
        commits, stable_refs, _ = evidence_from(
            immutable_by_id.get(issue_id, {}), provider
        )
        link = evidence_links.get(issue_id, {})
        stable_refs.extend(link.get("evidence_refs", []))
        commits.extend(
            ref.removeprefix("git:")
            for ref in link.get("implementation_commit_refs", [])
        )
        direct_proof[issue_id] = bool(commits or stable_refs)

    def infer_duplicate_target(issue_id: str, provider: dict) -> str | None:
        close_reason = provider.get("close_reason") or ""
        explicit = EXACT_DUPLICATE_RE.search(close_reason)
        if explicit:
            target = explicit.group(1)
            return target if target.startswith("focusa-") else "focusa-" + target
        if not re.search(r"(?i)\bduplicate\b", close_reason):
            return None
        code_match = TASK_CODE_RE.search(provider.get("title") or "")
        if not code_match:
            return None
        root = issue_id.rsplit(".", 1)[0]
        code = code_match.group(1).upper()
        candidates = []
        for candidate_id in admitted_ids:
            if candidate_id == issue_id or candidate_id.rsplit(".", 1)[0] != root:
                continue
            candidate = provider_by_id.get(candidate_id, {})
            candidate_code = TASK_CODE_RE.search(candidate.get("title") or "")
            if (
                candidate_code
                and candidate_code.group(1).upper() == code
                and candidate.get("status") == "closed"
                and (
                    not re.search(
                        r"(?i)\bduplicate\b", candidate.get("close_reason") or ""
                    )
                    or direct_proof.get(candidate_id, False)
                )
            ):
                candidates.append(candidate_id)
        return candidates[0] if len(candidates) == 1 else None

    mappings: list[dict] = []
    for issue_id in sorted(admitted_ids, key=natural_key):
        member = immutable_by_id.get(issue_id, {})
        provider = provider_by_id.get(issue_id)
        authority = (
            "immutable_workset_r7"
            if issue_id in immutable_ids
            else "authorized_release_repair_overlay"
        )
        if provider is None:
            mappings.append(
                {
                    "bead_id": issue_id,
                    "authority": authority,
                    "provider_state": "missing",
                    "evidence_state": "orphan",
                }
            )
            continue

        commits, stable_refs, github_issues = evidence_from(member, provider)
        link = evidence_links.get(issue_id, {})
        stable_refs = sorted(set(stable_refs + link.get("evidence_refs", [])))
        commits = sorted(
            set(
                commits
                + [
                    ref.removeprefix("git:")
                    for ref in link.get("implementation_commit_refs", [])
                ]
            )
        )
        close_reason = provider.get("close_reason") or None
        exact_duplicate_of = infer_duplicate_target(issue_id, provider)
        duplicate_claim = bool(re.search(r"(?i)\bduplicate\b", close_reason or ""))
        exact_duplicate_valid = bool(
            exact_duplicate_of
            and exact_duplicate_of in admitted_ids
            and provider_by_id.get(exact_duplicate_of, {}).get("status") == "closed"
        )
        duplicate_target_proven = bool(
            exact_duplicate_valid and direct_proof.get(exact_duplicate_of, False)
        )
        has_proof = bool(commits or stable_refs)
        status = provider.get("status", "unknown")
        if status != "closed":
            evidence_state = "pending_technical_acceptance"
        elif duplicate_claim and not exact_duplicate_valid and has_proof:
            evidence_state = "evidence_linked"
        elif duplicate_claim and not exact_duplicate_valid:
            evidence_state = "ambiguous_duplicate_closure"
        elif exact_duplicate_valid and duplicate_target_proven:
            evidence_state = "exact_duplicate_receipt"
        elif exact_duplicate_valid:
            evidence_state = "duplicate_target_without_proof"
        elif has_proof:
            evidence_state = "evidence_linked"
        else:
            evidence_state = "closed_without_proof"

        blockers = sorted(
            {
                dep["depends_on_id"]
                for dep in provider.get("dependencies", [])
                if dep.get("type") == "blocks"
                and dep.get("depends_on_id") in admitted_ids
                and provider_by_id.get(dep.get("depends_on_id"), {}).get("status")
                != "closed"
            },
            key=natural_key,
        )
        mappings.append(
            {
                "bead_id": issue_id,
                "authority": authority,
                "task_plan_ref": member.get("task_plan_ref"),
                "ordered_task_key": [
                    int(part) if part.isdigit() else part
                    for part in re.split(r"[.-]", issue_id)
                ],
                "parent_bead_id": provider_parent(provider),
                "github_issue_refs": github_issues,
                "title": provider.get("title"),
                "provider_state": status,
                "provider_updated_at": provider.get("updated_at"),
                "provider_record_digest": digest_value(provider),
                "frozen_projection": member.get("current_status_projection"),
                "projection_drift": bool(
                    member and member.get("current_status_projection") != status
                ),
                "implementation_commit_refs": [f"git:{value}" for value in commits],
                "runtime_or_acceptance_evidence_refs": stable_refs,
                "active_blocker_refs": blockers,
                "closure_receipt": {
                    "closed_at": provider.get("closed_at"),
                    "close_reason": close_reason,
                    "exact_duplicate_of": exact_duplicate_of,
                }
                if status == "closed"
                else None,
                "evidence_state": evidence_state,
            }
        )

    # A closed parent may inherit proof only when every admitted descendant is
    # independently evidenced. Process deepest parents first so aggregation is
    # deterministic and cannot hide a pending or ambiguous child.
    resolved_states = {
        "evidence_linked",
        "exact_duplicate_receipt",
        "aggregate_child_evidence",
    }
    for row in sorted(
        mappings, key=lambda value: value["bead_id"].count("."), reverse=True
    ):
        if row["evidence_state"] != "closed_without_proof":
            continue
        prefix = row["bead_id"] + "."
        descendants = [
            candidate
            for candidate in mappings
            if candidate["bead_id"].startswith(prefix)
        ]
        if descendants and all(
            candidate["evidence_state"] in resolved_states for candidate in descendants
        ):
            row["evidence_state"] = "aggregate_child_evidence"
            row["aggregate_evidence_member_refs"] = [
                f"bead:{candidate['bead_id']}" for candidate in descendants
            ]

    state_counts = Counter(row["provider_state"] for row in mappings)
    evidence_counts = Counter(row["evidence_state"] for row in mappings)
    gaps = {
        "orphan_bead_ids": [
            row["bead_id"] for row in mappings if row["evidence_state"] == "orphan"
        ],
        "duplicate_provider_ids": sorted(set(duplicate_provider_ids), key=natural_key),
        "ambiguous_duplicate_closure_ids": [
            row["bead_id"]
            for row in mappings
            if row["evidence_state"] == "ambiguous_duplicate_closure"
        ],
        "duplicate_target_without_proof_ids": [
            row["bead_id"]
            for row in mappings
            if row["evidence_state"] == "duplicate_target_without_proof"
        ],
        "closed_without_proof_ids": [
            row["bead_id"]
            for row in mappings
            if row["evidence_state"] == "closed_without_proof"
        ],
        "pending_technical_acceptance_ids": [
            row["bead_id"]
            for row in mappings
            if row["evidence_state"] == "pending_technical_acceptance"
        ],
        "untracked_locked_release_ids": sorted(
            locked_label_ids - admitted_ids, key=natural_key
        ),
        "projection_drift_ids": [
            row["bead_id"] for row in mappings if row.get("projection_drift")
        ],
    }
    unresolved = sum(
        len(value) for key, value in gaps.items() if key != "projection_drift_ids"
    )
    result = {
        "schema": "focusa.locked_release_governance_reconciliation.v1",
        "status": "reconciled" if unresolved == 0 else "blocked",
        "workset_id": inventory["workset_id"],
        "inventory_digest": inventory["inventory_digest"],
        "evidence_links_digest": digest_bytes(EVIDENCE_LINKS),
        "provider_snapshot": {
            "source": "canonical:.beads/issues.jsonl",
            "sha256": digest_bytes(provider_path),
            "record_count": len(provider_rows),
        },
        "admitted_mapping_count": len(mappings),
        "immutable_mapping_count": len(immutable_ids),
        "repair_overlay_mapping_count": len(overlay_ids),
        "provider_state_counts": dict(sorted(state_counts.items())),
        "evidence_state_counts": dict(sorted(evidence_counts.items())),
        "unresolved_gap_count": unresolved,
        "gaps": gaps,
        "mappings": mappings,
    }
    result["reconciliation_digest"] = digest_value(result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--provider-jsonl", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = (
        json.dumps(build(args.provider_jsonl.resolve()), indent=2, sort_keys=True)
        + "\n"
    )
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered:
            print(
                f"governance reconciliation drift: regenerate {OUTPUT.relative_to(ROOT)}"
            )
            return 1
        print("locked-release governance reconciliation snapshot: PASS")
        return 0
    OUTPUT.write_text(rendered)
    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
