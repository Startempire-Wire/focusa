#!/usr/bin/env python3
"""Validate, sign, or verify predeployment compatibility-canary evidence."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import pathlib

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

SCHEMA = "focusa.compatibility_canary_success.v1"
SEQUENCE = [
    "prior_release_healthy",
    "candidate_manifest_bound_apply_healthy",
    "prior_release_full_rollback_healthy",
    "candidate_manifest_bound_reapply_healthy",
]


def validate_receipt(path: pathlib.Path, tag: str, commit: str, previous_tag: str) -> bytes:
    encoded = path.read_bytes()
    receipt = json.loads(encoded)
    if not isinstance(receipt, dict):
        raise ValueError("compatibility canary receipt must be a JSON object")
    if (
        receipt.get("schema") != SCHEMA
        or receipt.get("status") != "passed"
        or receipt.get("candidate") != {"tag": tag, "commit": commit}
        or receipt.get("environment") != "isolated_preproduction"
        or receipt.get("sequence") != SEQUENCE
        or receipt.get("database_quick_check") != "ok"
        or receipt.get("signed_lease_preserved") is not True
        or receipt.get("user_sentinel_preserved") is not True
        or receipt.get("production_runtime_preserved") is not True
        or receipt.get("system_install_performed") is not False
        or receipt.get("service_mutation_performed") is not False
        or receipt.get("automatic_apply_performed") is not False
        or receipt.get("interrupted_install_recovered") is not True
    ):
        raise ValueError("compatibility canary receipt is incomplete or unsafe")
    previous = receipt.get("previous_release_tag")
    if (
        not isinstance(previous, str)
        or previous != previous_tag
        or not previous.startswith("v")
        or len(previous.split(".")) != 3
        or previous == tag
    ):
        raise ValueError("compatibility canary prior release identity is invalid")
    evidence = receipt.get("database_evidence")
    if not isinstance(evidence, dict) or set(evidence) != {
        "prior_initial",
        "prior_interrupted_recovery",
        "candidate_first",
        "prior_rollback",
        "candidate_reapply",
    }:
        raise ValueError("compatibility canary database evidence is incomplete")
    for phase in evidence.values():
        if not isinstance(phase, dict) or any(
            not isinstance(phase.get(field), str)
            or len(phase[field]) != 64
            or any(char not in "0123456789abcdef" for char in phase[field])
            for field in ("schema_sha256", "row_counts_sha256")
        ):
            raise ValueError("compatibility canary database evidence digest is invalid")
    parity = receipt.get("distribution_parity")
    if not isinstance(parity, dict) or parity.get("status") != "passed" or any(
        not isinstance(parity.get(field), str)
        or len(parity[field]) != 64
        or any(char not in "0123456789abcdef" for char in parity[field])
        for field in ("candidate_first_sha256", "candidate_reapply_sha256")
    ):
        raise ValueError("compatibility canary distribution parity evidence is invalid")
    run_url = receipt.get("run_url")
    if not isinstance(run_url, str) or "/actions/runs/" not in run_url:
        raise ValueError("compatibility canary receipt run URL is not canonical")
    return encoded


def active_key(path: pathlib.Path) -> dict[str, object]:
    trusted = json.loads(path.read_text(encoding="utf-8"))
    keys = trusted.get("keys") if isinstance(trusted, dict) else None
    active = [
        key
        for key in (keys or [])
        if isinstance(key, dict) and key.get("revoked_at") is None
    ]
    if len(active) != 1 or active[0].get("signing_algorithm") != "ed25519":
        raise ValueError("exactly one active Ed25519 release key is required")
    expected_raw = base64.b64decode(
        str(active[0].get("public_key_base64", "")), validate=True
    )
    if len(expected_raw) != 32 or hashlib.sha256(expected_raw).hexdigest() != active[
        0
    ].get("public_key_fingerprint"):
        raise ValueError("trusted release key fingerprint mismatch")
    active[0]["raw"] = expected_raw
    return active[0]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    for command in ("sign", "verify"):
        sub = subparsers.add_parser(command)
        sub.add_argument("--receipt", required=True, type=pathlib.Path)
        sub.add_argument("--trusted-keys", required=True, type=pathlib.Path)
        sub.add_argument("--tag", required=True)
        sub.add_argument("--commit", required=True)
        sub.add_argument("--previous-tag", required=True)
        if command == "sign":
            sub.add_argument("--private-key", required=True, type=pathlib.Path)
        else:
            sub.add_argument("--signature", required=True, type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    encoded = validate_receipt(args.receipt, args.tag, args.commit, args.previous_tag)
    key = active_key(args.trusted_keys)
    public = Ed25519PublicKey.from_public_bytes(key["raw"])
    signature_path: pathlib.Path

    if args.command == "sign":
        private = serialization.load_pem_private_key(
            args.private_key.read_bytes(), password=None
        )
        if not isinstance(private, Ed25519PrivateKey):
            raise ValueError("compatibility canary signing key must be Ed25519")
        actual_raw = private.public_key().public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )
        if actual_raw != key["raw"]:
            raise ValueError("compatibility canary signing key is not the active release key")
        signature_path = args.receipt.with_name(args.receipt.name + ".sig")
        signature_path.write_bytes(private.sign(encoded))
        status = "signed"
    else:
        signature_path = args.signature
        status = "verified"

    signature = signature_path.read_bytes()
    if len(signature) != 64:
        raise ValueError("compatibility canary signature is not raw Ed25519")
    try:
        public.verify(signature, encoded)
    except InvalidSignature as error:
        raise ValueError("compatibility canary signature is invalid") from error
    print(
        json.dumps(
            {
                "schema": "focusa.compatibility_canary_signature.v1",
                "status": status,
                "receipt": str(args.receipt),
                "signature": str(signature_path),
                "key_id": key["key_id"],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(
            f"compatibility canary receipt verification failed: {error}",
            file=__import__("sys").stderr,
        )
        raise SystemExit(1)
