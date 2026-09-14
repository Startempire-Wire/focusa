#!/usr/bin/env python3
"""Independent cross-language verification of the Spec 152E EDD-bound signed
lease golden vectors (paid / Evaluation / bundle) plus the fail-closed negative
matrix. The vectors are produced by the PHP candidate contract
(docs/contracts/spec152e-edd-bound-lease-issuer.v1.php) with fixed public
synthetic seeds; this suite re-verifies every signature with the `cryptography`
Ed25519 implementation (RFC 8032, same algorithm family as ed25519-dalek and
libsodium) and applies the existing authority-lease verifier rules: key-set
trust, key validity, domain-separated signature, key-id match, product/node
match, minimum sequence, previous digest, status, not-before, and
expiry/offline-grace. All fixtures are public synthetic non-production values;
no email, license key, or secret material is present.
"""

import base64
import hashlib
import json
import re
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ROOT = Path(__file__).resolve().parents[1]
VECTOR_PATH = ROOT / "docs/contracts/spec152e-lease-golden-vectors.v1.json"
VECTOR = json.loads(VECTOR_PATH.read_text(encoding="utf-8"))

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


def b64decode(data: str) -> bytes:
    return base64.b64decode(data)


def verify_signature(public_key_b64: str, signature_b64: str, domain: bytes, payload: bytes) -> None:
    key = Ed25519PublicKey.from_public_bytes(b64decode(public_key_b64))
    key.verify(b64decode(signature_b64), domain + payload)


def verify_envelope(envelope: dict, key: dict, domain: bytes, context: dict) -> dict:
    """Apply the existing authority-lease verifier rules; raise on rejection."""
    if envelope.get("schema") != "focusa.signed_envelope.v1":
        raise ValueError("unsupported_envelope_schema")
    signer_key_id = envelope.get("signer_key_id", "")
    if key.get("key_id") != signer_key_id:
        raise ValueError("unknown_key")
    if key.get("status") == "revoked":
        raise ValueError("revoked_key")
    now = context["now"]
    if now < key["not_before"] or now > key["not_after"]:
        raise ValueError("key_outside_validity")
    payload = b64decode(envelope["payload_b64"])
    try:
        verify_signature(key["public_key_b64"], envelope["signature_b64"], domain, payload)
    except InvalidSignature:
        raise ValueError("invalid_signature")
    claims = json.loads(payload)
    if claims.get("schema") != "focusa.authority_lease.v1":
        raise ValueError("unsupported_payload_schema")
    if claims.get("authority_key_id") != signer_key_id:
        raise ValueError("authority_key_mismatch")
    if claims.get("product") != context.get("expected_product"):
        raise ValueError("wrong_product")
    if claims.get("node_id") != context.get("expected_node_id"):
        raise ValueError("wrong_node")
    sequence = claims.get("sequence", 0)
    if context.get("minimum_sequence") is not None and sequence < context["minimum_sequence"]:
        raise ValueError("stale_sequence")
    expected_previous = context.get("expected_previous_digest")
    if expected_previous is not None and claims.get("previous_lease_digest") != expected_previous:
        raise ValueError("previous_digest_mismatch")
    if claims.get("status") == "revoked":
        raise ValueError("revoked_lease")
    if now < claims["not_before"]:
        raise ValueError("not_yet_valid")
    if now > claims["expires_at"]:
        grace = claims.get("offline_grace_until")
        if grace is not None and now <= grace:
            return {"state": "offline_grace", "claims": claims, "lease_digest": "sha256:" + hashlib.sha256(payload).hexdigest()}
        raise ValueError("expired")
    return {"state": "active", "claims": claims, "lease_digest": "sha256:" + hashlib.sha256(payload).hexdigest()}


# ── Structure and fixture hygiene ────────────────────────────────────────

assert VECTOR["schema"] == "focusa.spec152e.lease_golden_vectors.v1"
assert VECTOR["fixture_kind"] == "public_synthetic_nonproduction"
LEASE_DOMAIN = VECTOR["domains"]["lease"].encode()
KEY_SET_DOMAIN = VECTOR["domains"]["key_set"].encode()
assert LEASE_DOMAIN == b"FOCUSA-AUTHORITY-LEASE-V1\x00"
assert KEY_SET_DOMAIN == b"FOCUSA-AUTHORITY-KEY-SET-V1\x00"
expect(len(b64decode(VECTOR["root_public_key_b64"])) == 32, "root public key is 32 bytes")
expect(len(b64decode(VECTOR["lease_public_key_b64"])) == 32, "lease public key is 32 bytes")

# ── Key set envelope: trusted by the existing verifier ───────────────────

key_set_envelope = VECTOR["key_set_envelope"]
assert key_set_envelope["schema"] == "focusa.signed_envelope.v1"
assert key_set_envelope["signer_key_id"] == VECTOR["root_key_id"]
verify_signature(
    VECTOR["root_public_key_b64"],
    key_set_envelope["signature_b64"],
    KEY_SET_DOMAIN,
    b64decode(key_set_envelope["payload_b64"]),
)
expect(True, "root-signed key set envelope verifies (KEY_SET_DOMAIN)")
key_set = json.loads(b64decode(key_set_envelope["payload_b64"]))
expect(key_set["schema"] == "focusa.authority_key_set.v1", "key set schema")
expect(key_set["sequence"] == 7, "key set sequence")
expect(key_set["expires_at"] > key_set["issued_at"], "key set lifetime positive")
expect(key_set["expires_at"] > VECTOR["now"], "key set not expired at the vector now")
keys = key_set["keys"]
expect(len(keys) == 1, "key set carries one lease key")
lease_key = keys[0]
expect(lease_key["key_id"] == VECTOR["lease_key_id"], "lease key id matches")
expect(lease_key["public_key_b64"] == VECTOR["lease_public_key_b64"], "lease public key matches the vector")
expect(lease_key["status"] == "active", "lease key is active")
expect(lease_key["not_before"] <= VECTOR["now"] <= lease_key["not_after"], "lease key valid at the vector now")

# The published key set is byte-identical to the trusted Spec 152 golden
# vector already consumed by the existing Rust verifier.
rust_fixture = json.loads((ROOT / "crates/focusa-license/tests/fixtures/spec152-authority-golden-vector.json").read_text(encoding="utf-8"))
expect(VECTOR["root_public_key_b64"] == rust_fixture["root_public_key_b64"], "root key equals the trusted Rust fixture")
expect(key_set_envelope == rust_fixture["key_set_envelope"], "key set envelope equals the trusted Rust fixture byte-for-byte")

# ── Positive vectors: paid / Evaluation / bundle ─────────────────────────

expected_postures = {"paid": "paid", "evaluation": "evaluation", "bundle": "bundle"}
expected_products = {"paid": "focusa", "evaluation": "focusa", "bundle": "focusa"}
for name, vec in VECTOR["vectors"].items():
    envelope = vec["envelope"]
    context = {
        "expected_product": vec["product"],
        "expected_node_id": vec["node_id"],
        "now": VECTOR["now"],
        "minimum_sequence": vec["sequence"],
    }
    snapshot = verify_envelope(envelope, lease_key, LEASE_DOMAIN, context)
    expect(snapshot["state"] == "active", f"{name} lease verifies as Active at the vector now")
    claims = snapshot["claims"]
    expect(claims["schema"] == "focusa.authority_lease.v1", f"{name} payload schema")
    expect(claims["lease_id"] == vec["lease_id"], f"{name} lease id")
    expect(claims["product"] == expected_products[name], f"{name} product claim")
    expect(claims["posture"] == expected_postures[name], f"{name} posture claim")
    expect(claims["subject_id"] == claims["account_id"] == vec["account_uuid"], f"{name} account claim")
    expect(claims["customer_id"] == vec["customer_id"], f"{name} customer claim")
    expect(claims["order_id"] == vec["order_id"], f"{name} order claim")
    expect(claims["order_item_id"] == vec["order_item_id"], f"{name} order item claim")
    expect(claims["edd_license_id"] == vec["edd_license_id"], f"{name} license claim")
    expect(claims["node_id"] == vec["node_id"], f"{name} node claim")
    expect(claims["sequence"] == vec["sequence"], f"{name} sequence claim")
    expect(claims["authority_key_id"] == VECTOR["lease_key_id"], f"{name} kid claim")
    expect(claims["issued_at"] == claims["not_before"] == VECTOR["now"], f"{name} time claims")
    expect(claims["expires_at"] > claims["issued_at"], f"{name} expiry after issue")
    expect(claims["offline_grace_until"] is None or claims["offline_grace_until"] > claims["expires_at"], f"{name} offline grace after expiry")
    expect(claims["status"] == "active", f"{name} status claim")
    expect(len(claims["features"]) >= 1, f"{name} feature claims present")
    expect(claims["limits"].get("operator_seats") == 1, f"{name} operator seat limit")
    expect(claims["limits"].get("node_limit", 0) >= 1, f"{name} node limit")
    expect(claims["commercial"]["term"] in ("lifetime", "evaluation_30_days"), f"{name} commercial term")
    expect(claims["commercial"]["price_usd"] in ("697.00", "1254.60", "0.00"), f"{name} commercial price")
    expect(snapshot["lease_digest"].startswith("sha256:"), f"{name} derived lease digest")

# Posture-specific claim invariants.
paid_claims = VECTOR["vectors"]["paid"]["claims"]
eval_claims = VECTOR["vectors"]["evaluation"]["claims"]
bundle_claims = VECTOR["vectors"]["bundle"]["claims"]
expect(paid_claims["features"]["premium_updates"] is True, "paid grants all operator families")
expect(paid_claims["commercial"]["price_usd"] == "697.00", "paid commercial price")
expect(paid_claims["offline_grace_until"] > paid_claims["expires_at"], "paid has offline grace")
expect(eval_claims["features"]["automation"] is False, "evaluation grants the bounded subset only")
expect(eval_claims["limits"]["node_limit"] == 1, "evaluation node limit is 1")
expect(eval_claims["offline_grace_until"] is None, "evaluation carries no offline grace")
expect(eval_claims["expires_at"] == "2026-09-07T18:30:00Z", "evaluation expiry is exactly now + 30 days")
expect(bundle_claims["posture"] == "bundle", "bundle posture")
expect(bundle_claims["features"]["base_focusa"] is True and bundle_claims["features"]["base_uiai"] is True, "bundle exact-union features")
expect(bundle_claims["commercial"]["price_usd"] == "1254.60", "bundle commercial price")
expect(bundle_claims["commercial"]["refund_policy"] == "whole_order_30_days", "bundle refund policy")

# Paid lease offline grace boundary: before grace end maps to OfflineGrace.
grace_snapshot = verify_envelope(VECTOR["vectors"]["paid"]["envelope"], lease_key, LEASE_DOMAIN, {
    "expected_product": "focusa",
    "expected_node_id": "node-paid-golden-001",
    "now": "2026-12-01T00:00:00Z",
    "minimum_sequence": 42,
})
expect(grace_snapshot["state"] == "offline_grace", "post-expiry pre-grace-end maps to OfflineGrace")

# ── Negative matrix: the verifier rejects each named case ────────────────

for negative in VECTOR["negatives"]:
    case = negative["case"]
    context = {
        "expected_product": negative.get("expected_product", "focusa"),
        "expected_node_id": negative.get("expected_node_id", "node-paid-golden-001"),
        "now": negative.get("now", VECTOR["now"]),
        "minimum_sequence": negative.get("minimum_sequence"),
    }
    try:
        verify_envelope(negative["envelope"], lease_key, LEASE_DOMAIN, context)
        expect(False, f"negative case {case} must be rejected", negative=True)
    except ValueError as error:
        reason = str(error)
        expect(reason == negative["reason"], f"negative case {case} rejects with {reason} (expected {negative['reason']})", negative=True)

# Refunded case: the license refund advanced the account sequence to 45; the
# presented lease (sequence 42) is stale and refresh is denied.
refunded = next(n for n in VECTOR["negatives"] if n["case"] == "refunded")
expect(refunded["refund_sequence"] == 45, "refund fixture carries the post-refund sequence")
expect(refunded["reason"] == "stale_sequence", "refund rejection reason is stale_sequence")

# Unknown-key envelope: signer key is not in the key set.
unknown_key = next(n for n in VECTOR["negatives"] if n["case"] == "unknown_key")
expect(unknown_key["envelope"]["signer_key_id"] == "authority-lease-2026-99", "unknown key id is not the authority key")

# Invalid-signature envelope: signature bytes are all-zero, so the same rules
# must fail on the signature check (and the tampered payload variant too).
invalid_sig = next(n for n in VECTOR["negatives"] if n["case"] == "invalid_signature")
expect(b64decode(invalid_sig["envelope"]["signature_b64"]) == b"\x00" * 64, "invalid signature fixture is all-zero")
try:
    verify_envelope(invalid_sig["envelope"], lease_key, LEASE_DOMAIN, {
        "expected_product": "focusa", "expected_node_id": "node-paid-golden-001",
        "now": VECTOR["now"], "minimum_sequence": 42,
    })
    expect(False, "invalid signature must be rejected", negative=True)
except ValueError as error:
    expect(str(error) == "invalid_signature", "invalid signature fails closed", negative=True)

# Tampered payload bytes (flip one payload bit) must fail the signature check.
tampered = dict(VECTOR["vectors"]["paid"]["envelope"])
raw = bytearray(b64decode(tampered["payload_b64"]))
raw[0] ^= 0x01
tampered["payload_b64"] = base64.b64encode(bytes(raw)).decode()
try:
    verify_envelope(tampered, lease_key, LEASE_DOMAIN, {
        "expected_product": "focusa", "expected_node_id": "node-paid-golden-001",
        "now": VECTOR["now"], "minimum_sequence": 42,
    })
    expect(False, "tampered payload must be rejected", negative=True)
except ValueError as error:
    expect(str(error) == "invalid_signature", "tampered payload fails closed", negative=True)

# ── Hygiene: no real secrets, no unmasked real email, no key material ─────

raw = VECTOR_PATH.read_text(encoding="utf-8")
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw, re.I)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw, re.I)
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}", raw, re.I)
expect(True, "hygiene: no secrets, emails, or license keys in the vector file")

print(json.dumps({
    "schema": "focusa.spec152e.lease_golden_vector_validation.v1",
    "fixture": "public_synthetic_nonproduction",
    "golden_vectors": sorted(VECTOR["vectors"].keys()),
    "negative_cases": len(VECTOR["negatives"]),
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "result": "passed_fail_closed",
}, sort_keys=True))
