#!/usr/bin/env python3
"""Verify the public synthetic Spec 152E signed golden vector."""

import base64
import hashlib
import hmac
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VECTOR_PATH = ROOT / "docs/contracts/spec152e-facade-golden-vectors.v1.json"
VECTOR = json.loads(VECTOR_PATH.read_text(encoding="utf-8"))
REQUEST = VECTOR["request"]
CREDENTIAL = VECTOR["credential"]

SIGNED_FIELDS = [
    "schema", "credential_id", "timestamp", "nonce", "request_id",
    "idempotency_key", "registration_id", "facade_id", "origin",
    "product_code", "action", "redirect_handle", "continuation_token",
    "body_sha256",
]

assert VECTOR["schema"] == "focusa.spec152e.facade_golden_vectors.v1"
assert CREDENTIAL["fixture_kind"] == "public_synthetic_nonproduction"
assert CREDENTIAL["credential_id"] == REQUEST["credential_id"]
assert CREDENTIAL["facade_id"] == REQUEST["facade_id"]
assert REQUEST["schema"] == "focusa.spec152e.facade_protocol.v1"
assert REQUEST["timestamp"] == VECTOR["now"]
assert REQUEST["body_sha256"] == hashlib.sha256(b"{}").hexdigest()

canonical = "\n".join(str(REQUEST[field]) for field in SIGNED_FIELDS)
assert canonical == VECTOR["canonical_request"]
key = CREDENTIAL["key_utf8"].encode()
expected_signature = hmac.new(key, canonical.encode(), hashlib.sha256).hexdigest()
assert hmac.compare_digest(expected_signature, REQUEST["signature"])

payload, signature = REQUEST["continuation_token"].split(".")
expected_token_signature = hmac.new(
    key, f"continuation-v1\n{payload}".encode(), hashlib.sha256
).hexdigest()
assert hmac.compare_digest(expected_token_signature, signature)
padding = "=" * ((4 - len(payload) % 4) % 4)
claims = base64.urlsafe_b64decode(payload + padding).decode().split("\n")
assert claims == [
    REQUEST["registration_id"],
    REQUEST["facade_id"],
    REQUEST["action"],
    REQUEST["nonce"],
    str(VECTOR["now"] + 300),
]

assert VECTOR["expected"] == {
    "authority_route": "/v1/activation/start",
    "safe_redirect": "https://install.focusa.dev/activate/callback/success",
}
raw = VECTOR_PATH.read_text(encoding="utf-8")
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw, re.I)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw, re.I)
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)

print(json.dumps({
    "schema": "focusa.spec152e.facade_golden_vector_validation.v1",
    "signed_fields": len(SIGNED_FIELDS),
    "continuation_claims": len(claims),
    "result": "passed",
}, sort_keys=True))
