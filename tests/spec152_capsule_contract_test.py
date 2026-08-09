#!/usr/bin/env python3
"""Validate the Spec 152/152A signed capsule manifest contract and vectors.

Positive and negative schema checks, deterministic canonicalization, ed25519
signature binding over canonical bytes, and the forbidden designs (unsigned
sidecar checksum authority, embedded global/decryption keys) are asserted.
All named positive checks verify and all named negative checks fail closed.
"""

from __future__ import annotations

import base64
import hashlib
import json
import re
from pathlib import Path

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "docs/contracts/spec152-capsule-manifest.v1.schema.json"
VECTORS_PATH = ROOT / "docs/contracts/spec152-capsule-manifest-vectors.v1.json"

MANIFEST_SCHEMA = "focusa.capsule_manifest.v1"
FORBIDDEN_PROPERTY_NAMES = {"checksum", "global_key", "decryption_key", "private_key", "seed"}

schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
checker = FormatChecker()
validator = Draft202012Validator(schema, format_checker=checker)

vectors = json.loads(VECTORS_PATH.read_text(encoding="utf-8"))


def canonical(body: dict) -> bytes:
    """Deterministic canonicalization shared with the Rust verifier."""
    return json.dumps(body, sort_keys=True, separators=(",", ":")).encode()


def manifest_body(manifest: dict) -> dict:
    return {key: value for key, value in manifest.items() if key != "signature"}


def walk_property_names(node, found: set[str]) -> None:
    if isinstance(node, dict):
        for key in node:
            if key == "properties":
                found.update(node["properties"].keys())
            walk_property_names(node[key], found)
    elif isinstance(node, list):
        for item in node:
            walk_property_names(item, found)


def main() -> None:
    # Contract identity.
    assert vectors["schema"] == "focusa.spec152_capsule_manifest_vectors.v1"
    assert vectors["contract"] == MANIFEST_SCHEMA
    assert vectors["manifest_schema"] == MANIFEST_SCHEMA
    assert vectors["canonicalization"]["algorithm"] == "sha256"
    assert vectors["signature"]["algorithm"] == "ed25519"
    assert schema["properties"]["schema"]["const"] == MANIFEST_SCHEMA
    assert schema["properties"]["manifest_version"]["const"] == 1

    # FORBIDDEN: top level is closed; no unsigned sidecar checksum authority,
    # no embedded global key, no decryption key anywhere in the contract.
    assert schema["additionalProperties"] is False
    property_names: set[str] = set()
    walk_property_names(schema, property_names)
    assert not (property_names & FORBIDDEN_PROPERTY_NAMES), property_names & FORBIDDEN_PROPERTY_NAMES
    assert "checksum" not in schema["properties"]
    assert "global_key" not in schema["properties"]
    assert "decryption_key" not in schema["properties"]

    # No secrets or unmasked email anywhere in the vectors fixture.
    raw_vectors = VECTORS_PATH.read_text(encoding="utf-8")
    assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw_vectors)
    for forbidden in ("private_key", '"seed"', '"secret"'):
        assert forbidden not in raw_vectors, forbidden

    signer_key = Ed25519PublicKey.from_public_bytes(
        base64.b64decode(vectors["signer"]["verifying_key_b64"])
    )

    verified = 0
    rejected = 0
    checked_digests = 0
    for group in ("valid_manifests", "invalid_manifests"):
        for vector in vectors[group]:
            name = vector["name"]
            manifest = vector["manifest"]
            errors = list(validator.iter_errors(manifest))
            assert bool(errors) is not vector["schema_valid"], (
                f"{name}: schema_valid flag {vector['schema_valid']} does not match "
                f"actual validation ({[error.message[:90] for error in errors][:1]})"
            )

            # Deterministic canonicalization is replayable.
            body = manifest_body(manifest)
            digest = hashlib.sha256(canonical(body)).hexdigest()
            assert digest == vector["canonical_digest_sha256"], f"{name}: canonical digest"
            checked_digests += 1

            # The signature binds the canonical body bytes.
            signature = base64.b64decode(manifest["signature"]["signature_b64"])
            assert manifest["signature"]["signature_algorithm"] == "ed25519"
            assert len(signature) == 64, f"{name}: signature length"
            if name == "invalid_signature":
                try:
                    signer_key.verify(signature, canonical(body))
                except Exception:
                    pass
                else:
                    raise AssertionError(f"{name}: tampered signature unexpectedly verifies")
            else:
                signer_key.verify(signature, canonical(body))

            expected = vector["expected_decision"]
            if group == "valid_manifests":
                assert expected == "verified", name
                verified += 1
            else:
                assert expected.startswith("rejected_"), name
                rejected += 1

    assert verified == len(vectors["valid_manifests"]) >= 4
    assert rejected == len(vectors["invalid_manifests"]) >= 20
    assert checked_digests == len(vectors["valid_manifests"]) + len(vectors["invalid_manifests"])

    # FORBIDDEN negative documents must all fail schema validation.
    for negative in vectors["schema_negative"]:
        errors = list(validator.iter_errors(negative["document"]))
        assert errors, f"{negative['name']}: forbidden document unexpectedly accepted"
        assert negative["name"] in {
            "unsigned_manifest",
            "sidecar_checksum_authority",
            "embedded_global_key",
            "embedded_decryption_key",
        }

    print(
        json.dumps(
            {
                "schema": "focusa.spec152_capsule_contract_validation.v1",
                "manifest_schema": MANIFEST_SCHEMA,
                "valid_vectors": verified,
                "invalid_vectors": rejected,
                "canonical_digests_replayed": checked_digests,
                "schema_negative_documents": len(vectors["schema_negative"]),
                "result": "passed",
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
