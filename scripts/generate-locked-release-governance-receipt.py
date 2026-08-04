#!/usr/bin/env python3
"""Generate or verify the GH#106.5 immutable governance receipt."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
DEFAULT_RECEIPT = AUDIT / "next-locked-release-governance-receipt.json"
DEFAULT_SIGNATURE = AUDIT / "next-locked-release-governance-receipt.json.sig"
INPUT_PATHS = (
    "release-proof/audit/next-locked-release-governance-inventory.json",
    "release-proof/audit/next-locked-release-governance-reconciliation.json",
    "release-proof/audit/next-locked-release-governance-evidence-links.json",
    "release-proof/audit/next-locked-release-technical-closure-gate.json",
    "release-proof/audit/next-locked-release-candidate-ancestry.json",
    "release-proof/audit/next-locked-release-v09143-published-assets.json",
)


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_value(value: Any) -> str:
    return digest_bytes(canonical(value))


def file_digest(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def load(relative: str) -> dict[str, Any]:
    return json.loads((ROOT / relative).read_text())


def git_commit(ref: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", f"{ref}^{{commit}}"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise ValueError(result.stderr.strip() or f"unknown git ref: {ref}")
    return result.stdout.strip()


def classify_refs(evidence_links: dict[str, Any]) -> dict[str, list[str]]:
    refs = sorted(
        {ref for row in evidence_links["links"] for ref in row.get("evidence_refs", [])}
    )
    local_ref_digests = [
        {"path": ref, "digest": file_digest(ROOT / ref)}
        for ref in refs
        if (ROOT / ref).is_file()
        and (ROOT / ref) not in {DEFAULT_RECEIPT, DEFAULT_SIGNATURE}
    ]
    return {
        "artifact_refs": [ref for ref in refs if ref.startswith("release-proof/")],
        "test_refs": [ref for ref in refs if ref.startswith("tests/")],
        "referenced_file_digests": local_ref_digests,
        "documentation_and_evidence_refs": [
            ref
            for ref in refs
            if not ref.startswith("release-proof/") and not ref.startswith("tests/")
        ],
        "implementation_commit_refs": sorted(
            {
                ref
                for row in evidence_links["links"]
                for ref in row.get("implementation_commit_refs", [])
            }
        ),
    }


def build_payload(governance_source_commit: str) -> dict[str, Any]:
    source_commit = git_commit(governance_source_commit)
    inventory = load(INPUT_PATHS[0])
    reconciliation = load(INPUT_PATHS[1])
    evidence_links = load(INPUT_PATHS[2])
    gate = load(INPUT_PATHS[3])
    ancestry = load(INPUT_PATHS[4])
    published_assets = load(INPUT_PATHS[5])

    mapping_ids = sorted(row["bead_id"] for row in reconciliation["mappings"])
    if len(mapping_ids) != len(set(mapping_ids)):
        raise ValueError("governance receipt refuses duplicate admitted Bead IDs")
    gate_ids = sorted(gate["acceptance_basis"])
    if mapping_ids != gate_ids:
        raise ValueError("reconciliation and technical gate identity sets disagree")
    if gate["invalid_closed_count"] != 0:
        raise ValueError("governance receipt refuses invalid closed records")
    if ancestry["status"] != "verified" or ancestry["audit_errors"]:
        raise ValueError("candidate ancestry is not verified")

    technically_pending = sorted(gate["technically_pending_ids"])
    technically_accepted = sorted(set(mapping_ids) - set(technically_pending))
    if len(technically_accepted) != gate["technically_accepted_count"]:
        raise ValueError("technical acceptance count disagrees with closure set")
    provider_closed = sorted(
        row["bead_id"]
        for row in reconciliation["mappings"]
        if row["provider_state"] == "closed"
    )
    issue_links: dict[str, list[str]] = {}
    for row in reconciliation["mappings"]:
        for issue in row["github_issue_refs"]:
            issue_links.setdefault(str(issue), []).append(row["bead_id"])
    issue_links = {
        issue: sorted(set(bead_ids)) for issue, bead_ids in sorted(issue_links.items())
    }
    refs = classify_refs(evidence_links)

    closure_set = {
        "admitted_bead_ids": mapping_ids,
        "provider_closed_ids": provider_closed,
        "technically_accepted_ids": technically_accepted,
        "technically_pending_ids": technically_pending,
        "invalid_closed_ids": sorted(gate["invalid_closed_ids"]),
    }
    linkage = {
        "github_issue_to_admitted_beads": issue_links,
        **refs,
    }
    release_proof = {
        "immutable_tag": ancestry["immutable_release"]["tag"],
        "immutable_tag_commit": ancestry["immutable_release"]["commit"],
        "candidate_source_commit": ancestry["candidate"]["source_commit"],
        "next_stable_tag": ancestry["next_stable_tag"],
        "release_ready": ancestry["release_ready"],
        "release_blockers": ancestry["release_blockers"],
        "published_asset_count": len(published_assets["assets"]),
        "missing_required_assets": ancestry["immutable_release"][
            "missing_required_assets"
        ],
        "ancestry_digest": ancestry["ancestry_digest"],
    }
    inputs = [
        {"path": relative, "digest": file_digest(ROOT / relative)}
        for relative in INPUT_PATHS
    ]
    payload = {
        "schema": "focusa.locked_release_governance_receipt.v1",
        "status": "sealed",
        "workset_id": inventory["workset_id"],
        "governance_source_commit": source_commit,
        "authority": {
            "immutable_inventory_digest": inventory["inventory_digest"],
            "reconciliation_digest": reconciliation["reconciliation_digest"],
            "technical_gate_digest": gate["gate_digest"],
            "mapping_count": len(mapping_ids),
            "immutable_mapping_count": reconciliation["immutable_mapping_count"],
            "repair_overlay_mapping_count": reconciliation[
                "repair_overlay_mapping_count"
            ],
        },
        "closure_set": closure_set,
        "closure_set_digest": digest_value(closure_set),
        "linkage": linkage,
        "linkage_digest": digest_value(linkage),
        "release_proof": release_proof,
        "input_artifacts": inputs,
        "input_artifacts_digest": digest_value(inputs),
        "mutation_policy": {
            "canonical_replay_required": True,
            "detached_signature_required": True,
            "later_input_mutation_fails_verification": True,
            "provider_status_is_not_technical_proof": True,
            "rewrite_v0.9.143": False,
        },
    }
    payload["payload_digest"] = digest_value(payload)
    return payload


def signed_content(payload: dict[str, Any], public_key_raw: bytes) -> dict[str, Any]:
    return {
        "payload": payload,
        "seal": {
            "algorithm": "ed25519",
            "key_authority": "one_time_governance_integrity_seal",
            "authority_boundary": (
                "integrity and replay seal only; not release publication, "
                "entitlement, OTA, or production signing authority"
            ),
            "private_key_persisted": False,
            "public_key_base64": base64.b64encode(public_key_raw).decode("ascii"),
            "public_key_fingerprint": hashlib.sha256(public_key_raw).hexdigest(),
            "signed_encoding": "canonical-json-sorted-keys-compact-utf8",
        },
    }


def generate(receipt: Path, signature: Path, source_commit: str) -> None:
    private_key = Ed25519PrivateKey.generate()
    public_key = private_key.public_key()
    public_raw = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    content = signed_content(build_payload(source_commit), public_raw)
    encoded = canonical(content)
    detached = private_key.sign(encoded)
    public_key.verify(detached, encoded)
    if len(detached) != 64:
        raise ValueError("invalid Ed25519 detached signature length")
    receipt.write_text(json.dumps(content, indent=2, sort_keys=True) + "\n")
    signature.write_bytes(detached)


def verify(receipt: Path, signature: Path) -> None:
    content = json.loads(receipt.read_text())
    payload = content["payload"]
    seal = content["seal"]
    public_raw = base64.b64decode(seal["public_key_base64"], validate=True)
    if len(public_raw) != 32:
        raise ValueError("invalid Ed25519 public key length")
    if hashlib.sha256(public_raw).hexdigest() != seal["public_key_fingerprint"]:
        raise ValueError("governance seal public key fingerprint mismatch")
    public_key = Ed25519PublicKey.from_public_bytes(public_raw)
    try:
        public_key.verify(signature.read_bytes(), canonical(content))
    except InvalidSignature as error:
        raise ValueError("governance receipt detached signature invalid") from error
    replayed = build_payload(payload["governance_source_commit"])
    if payload != replayed:
        raise ValueError("governance receipt canonical replay detected mutation")
    print(
        "locked-release governance receipt: PASS "
        f"closure_set={payload['closure_set_digest']} "
        f"payload={payload['payload_digest']}"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--generate-ephemeral", action="store_true")
    mode.add_argument("--verify", action="store_true")
    parser.add_argument("--governance-source-commit")
    parser.add_argument("--receipt", type=Path, default=DEFAULT_RECEIPT)
    parser.add_argument("--signature", type=Path, default=DEFAULT_SIGNATURE)
    args = parser.parse_args()
    try:
        if args.generate_ephemeral:
            if not args.governance_source_commit:
                parser.error("--governance-source-commit is required when generating")
            generate(args.receipt, args.signature, args.governance_source_commit)
            verify(args.receipt, args.signature)
        else:
            verify(args.receipt, args.signature)
    except (KeyError, ValueError, OSError, json.JSONDecodeError) as error:
        print(f"locked-release governance receipt: FAIL: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
