#!/usr/bin/env python3
"""Acceptance and mutation tests for the GH#106.5 governance receipt."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
SCRIPT = ROOT / "scripts" / "generate-locked-release-governance-receipt.py"
RECEIPT = AUDIT / "next-locked-release-governance-receipt.json"
SIGNATURE = AUDIT / "next-locked-release-governance-receipt.json.sig"
GATE = AUDIT / "next-locked-release-technical-closure-gate.json"
GITHUB106_PROOF = AUDIT / "next-locked-release-github106-closure-proof.json"

subprocess.run(["python3", str(SCRIPT), "--verify"], cwd=ROOT, check=True)
receipt = json.loads(RECEIPT.read_text())
gate = json.loads(GATE.read_text())
github106 = json.loads(GITHUB106_PROOF.read_text())
payload = receipt["payload"]
seal = receipt["seal"]
closure = payload["closure_set"]

assert payload["schema"] == "focusa.locked_release_governance_receipt.v1"
assert payload["status"] == "sealed"
assert payload["workset_id"] == "workset:focusa-next-locked-release:r7"
assert payload["authority"]["mapping_count"] == 465
assert payload["authority"]["immutable_mapping_count"] == 275
assert payload["authority"]["repair_overlay_mapping_count"] == 14
assert len(closure["admitted_bead_ids"]) == 465
assert len(closure["technically_accepted_ids"]) == gate["technically_accepted_count"]
assert len(closure["technically_pending_ids"]) == gate["technically_pending_count"]
assert set(closure["technically_accepted_ids"]).isdisjoint(
    closure["technically_pending_ids"]
)
assert (
    sorted(closure["technically_accepted_ids"] + closure["technically_pending_ids"])
    == closure["admitted_bead_ids"]
)
assert closure["invalid_closed_ids"] == []

linkage = payload["linkage"]
assert "106" in linkage["github_issue_to_admitted_beads"]
assert "focusa-vbcqu.14" in linkage["github_issue_to_admitted_beads"]["106"]
assert linkage["artifact_refs"]
assert linkage["test_refs"]
assert linkage["implementation_commit_refs"]
assert any(
    row["path"]
    == "release-proof/audit/next-locked-release-github106-closure-proof.json"
    and row["digest"].startswith("sha256:")
    for row in linkage["referenced_file_digests"]
)
assert github106["issue"]["number"] == 106
assert github106["issue"]["state"] == "CLOSED"
assert github106["issue"]["state_reason"] == "COMPLETED"
assert github106["mutation_policy"]["rewrite_v0.9.143"] is False
assert payload["release_proof"]["immutable_tag"] == "v0.9.143"
assert payload["release_proof"]["release_ready"] is False
assert payload["mutation_policy"]["rewrite_v0.9.143"] is False

assert seal["algorithm"] == "ed25519"
assert seal["private_key_persisted"] is False
assert "not release publication" in seal["authority_boundary"]
assert SIGNATURE.stat().st_size == 64
public_key = Ed25519PublicKey.from_public_bytes(
    __import__("base64").b64decode(seal["public_key_base64"], validate=True)
)
public_key.verify(
    SIGNATURE.read_bytes(),
    json.dumps(receipt, sort_keys=True, separators=(",", ":")).encode(),
)

with tempfile.TemporaryDirectory() as directory:
    temp = Path(directory)
    tampered_receipt = temp / "receipt.json"
    tampered = json.loads(RECEIPT.read_text())
    tampered["payload"]["status"] = "tampered"
    tampered_receipt.write_text(json.dumps(tampered, indent=2, sort_keys=True) + "\n")
    failed = subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--verify",
            "--receipt",
            str(tampered_receipt),
            "--signature",
            str(SIGNATURE),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert failed.returncode != 0
    assert "signature invalid" in failed.stdout

    tampered_signature = temp / "receipt.sig"
    signature_bytes = bytearray(SIGNATURE.read_bytes())
    signature_bytes[0] ^= 1
    tampered_signature.write_bytes(signature_bytes)
    failed = subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--verify",
            "--receipt",
            str(RECEIPT),
            "--signature",
            str(tampered_signature),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert failed.returncode != 0
    assert "signature invalid" in failed.stdout

print("GH#106.5 immutable signed governance receipt: PASS")
