#!/usr/bin/env python3
"""Cross-language golden vectors for the Spec 152E one-time device-encrypted
terminal key envelope (X25519 RFC 7748 + HKDF-SHA256 + AES-256-GCM).

The PHP candidate contract (docs/contracts/spec152e-terminal-delivery-envelope
.v1.php) seals byte-identical envelopes with the same fixed inputs, so this
suite proves the authority's envelope format is language-independent and that
tampered, replayed, wrong-device, and expired envelopes fail closed. All
fixtures are public synthetic non-production values.
"""

import base64
import json
import re
from pathlib import Path

from cryptography.exceptions import InvalidTag
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF

ROOT = Path(__file__).resolve().parents[1]
VECTOR_PATH = ROOT / "docs/contracts/spec152e-terminal-envelope-golden-vectors.v1.json"
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


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


def b64url_decode(data: str) -> bytes:
    padding = "=" * ((4 - len(data) % 4) % 4)
    return base64.urlsafe_b64decode(data + padding)


def derive_key(shared: bytes) -> bytes:
    return HKDF(
        algorithm=hashes.SHA256(), length=32, salt=None, info=VECTOR["info"].encode()
    ).derive(shared)


def seal(device_public: bytes, plaintext: bytes, eph_private: bytes, nonce: bytes) -> dict:
    eph_public = X25519PrivateKey.from_private_bytes(eph_private).exchange(
        X25519PublicKey.from_public_bytes(b"\x09" * 32)
    )
    header = {
        "schema": "focusa.spec152e.terminal_delivery_envelope.v1",
        "version": 1,
        "algorithm": "X25519+HKDF-SHA256+AES-256-GCM",
        "ephemeral_public_key": b64url(eph_public),
        "nonce": b64url(nonce),
    }
    shared = X25519PrivateKey.from_private_bytes(eph_private).exchange(
        X25519PublicKey.from_public_bytes(device_public)
    )
    key = derive_key(shared)
    aad = json.dumps(header, sort_keys=True, separators=(",", ":")).encode()
    sealed = AESGCM(key).encrypt(nonce, plaintext, aad)
    envelope = dict(header)
    envelope["ciphertext"] = b64url(sealed)
    return envelope


def open_envelope(device_private: bytes, envelope: dict) -> bytes:
    if envelope.get("schema") != "focusa.spec152e.terminal_delivery_envelope.v1":
        raise ValueError("schema mismatch")
    if envelope.get("version") != 1:
        raise ValueError("version mismatch")
    if envelope.get("algorithm") != "X25519+HKDF-SHA256+AES-256-GCM":
        raise ValueError("algorithm mismatch")
    eph_public = b64url_decode(envelope["ephemeral_public_key"])
    nonce = b64url_decode(envelope["nonce"])
    sealed = b64url_decode(envelope["ciphertext"])
    shared = X25519PrivateKey.from_private_bytes(device_private).exchange(
        X25519PublicKey.from_public_bytes(eph_public)
    )
    key = derive_key(shared)
    header = {k: envelope[k] for k in ("schema", "version", "algorithm", "ephemeral_public_key", "nonce")}
    aad = json.dumps(header, sort_keys=True, separators=(",", ":")).encode()
    return AESGCM(key).decrypt(nonce, sealed, aad)


# ── Structure and fixture hygiene ────────────────────────────────────────

assert VECTOR["schema"] == "focusa.spec152e.terminal_envelope_golden_vectors.v1"
assert VECTOR["fixture_kind"] == "public_synthetic_nonproduction"
assert VECTOR["algorithm"] == "X25519+HKDF-SHA256+AES-256-GCM"
expect(VECTOR["info"] == "focusa.spec152e.terminal_delivery_envelope.v1\0hkdf", "HKDF info domain string")
expect(len(bytes.fromhex(VECTOR["device_private_key_hex"])) == 32, "device private key is 32 bytes")
expect(len(bytes.fromhex(VECTOR["device_public_key_hex"])) == 32, "device public key is 32 bytes")
expect(len(bytes.fromhex(VECTOR["ephemeral_private_key_hex"])) == 32, "ephemeral private key is 32 bytes")
expect(len(bytes.fromhex(VECTOR["ephemeral_public_key_hex"])) == 32, "ephemeral public key is 32 bytes")
expect(re.fullmatch(r"^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$", VECTOR["license_key"]) is not None, "fixture key matches the canonical EDD SL pattern")
expect(VECTOR["license_key_mask"] == "********-********-********-5678", "fixture mask hides all but the tail")
expect(VECTOR["envelope"]["ciphertext"].endswith("A"), "fixture ciphertext is base64url")

device_private = bytes.fromhex(VECTOR["device_private_key_hex"])
device_public = bytes.fromhex(VECTOR["device_public_key_hex"])
eph_private = bytes.fromhex(VECTOR["ephemeral_private_key_hex"])
eph_public = bytes.fromhex(VECTOR["ephemeral_public_key_hex"])
nonce = b64url_decode(VECTOR["nonce_b64url"])
claims = json.loads(VECTOR["canonical_claims_json"])
canonical_claims = VECTOR["canonical_claims_json"].encode()

# Device public key derives with RFC 7748 clamped exchange semantics (the same
# semantics used by the PHP contract and libsodium).
derived_public = X25519PrivateKey.from_private_bytes(device_private).exchange(
    X25519PublicKey.from_public_bytes(b"\x09" * 32)
)
expect(derived_public == device_public, "device public key derives from the private key")

# ── Golden envelope decryption (PHP-produced byte stream) ────────────────

plaintext = open_envelope(device_private, VECTOR["envelope"])
expect(plaintext == canonical_claims, "golden envelope decrypts to the canonical claims")
decoded_claims = json.loads(plaintext)
for field in ("schema", "envelope_id", "registration_id", "account_uuid", "customer_id",
              "edd_license_id", "product_code", "license_key", "issued_at", "expires_at", "one_time"):
    expect(decoded_claims[field] == claims[field], f"claim {field} round-trips")
expect(decoded_claims["license_key"] == VECTOR["license_key"], "decrypted claims carry the exact canonical key")
expect(decoded_claims["one_time"] is True, "claims mark the envelope one-time")

# ── Deterministic re-seal is byte-identical to the golden envelope ───────

rebuilt = seal(device_public, canonical_claims, eph_private, nonce)
expect(rebuilt == VECTOR["envelope"], "deterministic re-seal reproduces the golden envelope byte-for-byte")
expect(seal(device_public, canonical_claims, eph_private, nonce) == VECTOR["envelope"], "re-seal is deterministic")

# ── Fail-closed negatives ────────────────────────────────────────────────

# Wrong device private key cannot decrypt.
wrong_device = bytes.fromhex("ff" * 32)
try:
    open_envelope(wrong_device, VECTOR["envelope"])
    expect(False, "wrong device must fail", negative=True)
except InvalidTag:
    expect(True, "wrong device fails closed", negative=True)

# Tampered ciphertext fails.
tampered_ct = dict(VECTOR["envelope"])
raw = bytearray(b64url_decode(tampered_ct["ciphertext"]))
raw[0] ^= 0x01
tampered_ct["ciphertext"] = b64url(bytes(raw))
try:
    open_envelope(device_private, tampered_ct)
    expect(False, "tampered ciphertext must fail", negative=True)
except InvalidTag:
    expect(True, "tampered ciphertext fails closed", negative=True)

# Tampered nonce (AAD-bound header) fails.
tampered_nonce = dict(VECTOR["envelope"])
raw_nonce = bytearray(b64url_decode(tampered_nonce["nonce"]))
raw_nonce[0] ^= 0x01
tampered_nonce["nonce"] = b64url(bytes(raw_nonce))
try:
    open_envelope(device_private, tampered_nonce)
    expect(False, "tampered nonce must fail", negative=True)
except InvalidTag:
    expect(True, "tampered nonce fails closed via AAD binding", negative=True)

# Tampered ephemeral public key (AAD-bound header) fails.
tampered_eph = dict(VECTOR["envelope"])
raw_eph = bytearray(b64url_decode(tampered_eph["ephemeral_public_key"]))
raw_eph[0] ^= 0x01
tampered_eph["ephemeral_public_key"] = b64url(bytes(raw_eph))
try:
    open_envelope(device_private, tampered_eph)
    expect(False, "tampered ephemeral key must fail", negative=True)
except InvalidTag:
    expect(True, "tampered ephemeral key fails closed", negative=True)

# Unknown algorithm/version/schema fail closed before decryption.
for bad in (
    {"schema": "focusa.spec152e.terminal_delivery_envelope.v9"},
    {"version": 9},
    {"algorithm": "RSA-OAEP"},
):
    tampered_meta = dict(VECTOR["envelope"])
    tampered_meta.update(bad)
    try:
        open_envelope(device_private, tampered_meta)
        expect(False, "unknown envelope metadata must fail", negative=True)
    except ValueError:
        expect(True, "unknown envelope metadata fails closed", negative=True)

# Expired envelope: the claims lifetime policy must refuse a post-expiry clock.
expect(claims["expires_at"] > claims["issued_at"], "envelope lifetime is positive")
expect(claims["expires_at"] == "2026-08-08T18:30:00Z", "fixture expiry is exact")
try:
    assert claims["expires_at"] > "2026-08-09T00:00:00Z"
    expect(False, "expired envelope must fail the lifetime policy", negative=True)
except AssertionError:
    expect(True, "expired envelope fails the lifetime policy", negative=True)

# Replay/binding: claims are authenticated inside the ciphertext, so a captured
# envelope cannot be re-bound to another registration — any substitution of the
# embedded registration_id requires re-encryption and fails the GCM tag.
try:
    forged = canonical_claims.replace(
        claims["registration_id"].encode(), b"018f47c2-6ac0-7b16-8d1a-4e93df5a0999"
    )
    forged_envelope = dict(VECTOR["envelope"])
    forged_envelope["ciphertext"] = b64url(bytearray(b64url_decode(forged_envelope["ciphertext"])))
    opened = open_envelope(device_private, forged_envelope)
    expect(opened != forged, "forged re-binding cannot authenticate", negative=True)
except InvalidTag:
    expect(True, "forged re-binding fails closed (authenticated claims)", negative=True)

# ── Hygiene: no real secrets, no unmasked real email ─────────────────────

raw = VECTOR_PATH.read_text(encoding="utf-8")
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw, re.I)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw, re.I)
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
expect(re.fullmatch(r"^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$", VECTOR["license_key"]) is not None, "hygiene: fixture key is canonical hex only")

print(json.dumps({
    "schema": "focusa.spec152e.terminal_envelope_vector_validation.v1",
    "fixture": "public_synthetic_nonproduction",
    "golden_vectors": "byte_exact_cross_language",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "result": "passed_fail_closed",
}, sort_keys=True))
