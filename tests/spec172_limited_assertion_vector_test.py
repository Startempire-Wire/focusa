#!/usr/bin/env python3
"""Cross-language vectors for the Spec 172 limited-access assertion service
(atom focusa-vbcqu.20.15.11, 172.02.02).

The PHP candidate (docs/contracts/spec172-limited-access-assertion-service
.v1.php) signs each fixture payload with real RFC 8032 Ed25519 using a fixed
synthetic seed; this suite independently verifies the same bytes with Python
cryptography's Ed25519 and then applies the same fail-closed client/store
policy evaluation: valid assertions round-trip, and unverified, tampered,
stale-sequence, wrong-node, unknown-family, and paid-family claims fail
closed. The client store persists only verified assertions and never
self-issues. All fixture values are public synthetic non-production data.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

ROOT = Path(__file__).resolve().parents[1]
VECTOR_PATH = ROOT / "docs/contracts/spec172-limited-access-assertion-vectors.v1.json"
FIXTURE = json.loads(VECTOR_PATH.read_text(encoding="utf-8"))

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


def canonical_payload(presented: dict) -> bytes:
    payload = {
        "schema": FIXTURE["payload_schema"],
        "algorithm": FIXTURE["algorithm"],
        "posture_uuid": presented["posture_uuid"],
        "account_uuid": presented["account_uuid"],
        "identity_uuid": presented["identity_uuid"],
        "product_scope": presented["product_scope"],
        "node_uuid": presented["node_uuid"],
        "family_allowlist": sorted(set(presented["family_allowlist"])),
        "sequence": int(presented["sequence"]),
        "issued_at": presented["issued_at"],
        "refresh_at": presented["refresh_at"],
        "signer": presented["signer"],
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()


def registered_families(product_scope: str) -> set[str]:
    registry = FIXTURE["registries"][product_scope]
    return set(registry["limited"]) | set(registry["permanent"])


class LimitedAssertionClientStore:
    """Focusa-side client/store: verify + persist only valid assertions."""

    def __init__(self, public_key: Ed25519PublicKey) -> None:
        self.public_key = public_key
        self._records: dict[tuple[str, int], dict] = {}

    def evaluate(self, presented: dict, posture: dict | None, at: str | None = None) -> str:
        try:
            data = canonical_payload(presented)
        except (KeyError, TypeError, ValueError):
            return "ASSERTION_UNKNOWN"
        try:
            self.public_key.verify(bytes.fromhex(presented["signature"]), data)
        except (InvalidSignature, ValueError):
            return "SIGNATURE_INVALID"
        if posture is None:
            return "EMAIL_VERIFICATION_REQUIRED"
        if posture["status"] not in ("issued", "refreshed"):
            return "VERIFIED_LIMITED_ACCESS"
        if presented["product_scope"] != posture["product_scope"]:
            return "ENTITLEMENT_PRODUCT_MISMATCH"
        if presented["node_uuid"] != posture["node_uuid"]:
            return "NODE_LIMIT_REACHED"
        if presented["account_uuid"] != posture["account_uuid"]:
            return "ASSERTION_TAMPERED"
        allowed = set(posture["family_allowlist"])
        try:
            families = set(presented["family_allowlist"])
        except (KeyError, TypeError):
            return "ASSERTION_UNKNOWN"
        if not families:
            return "ASSERTION_UNKNOWN"
        for family in families:
            if family not in allowed or family not in registered_families(presented["product_scope"]):
                return "CAPABILITY_FAMILY_NOT_INCLUDED"
        if int(presented["sequence"]) != int(posture["sequence"]):
            return "ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED"
        if presented["issued_at"] > presented["refresh_at"]:
            return "ASSERTION_TAMPERED"
        if at is not None and at > presented["refresh_at"]:
            return "CREDENTIAL_REFRESH_REQUIRED"
        return "valid"

    def verify_and_store(self, presented: dict, posture: dict | None, at: str | None = None) -> str:
        verdict = self.evaluate(presented, posture, at)
        if verdict == "valid":
            self._records[(presented["posture_uuid"], int(presented["sequence"]))] = presented
        return verdict

    def stored_count(self) -> int:
        return len(self._records)

    def stored_postures(self) -> list[str]:
        return sorted({posture_uuid for posture_uuid, _sequence in self._records})

    def allowlist_for(self, posture_uuid: str) -> list[str]:
        rows = [record for (key, _seq), record in self._records.items() if key == posture_uuid]
        if not rows:
            return []
        latest = max(rows, key=lambda record: int(record["sequence"]))
        return sorted(set(latest["family_allowlist"]))


def main() -> int:
    # ── Structural invariants ────────────────────────────────────────────
    expect(FIXTURE["schema"] == "focusa.spec172.limited_access_assertion_vectors.v1", "fixture schema")
    expect(FIXTURE["fixture_kind"] == "public_synthetic_nonproduction", "fixture kind is public synthetic")
    expect(FIXTURE["algorithm"] == "ed25519.spec172.v1", "fixture algorithm label matches the server-owned constant")
    expect(FIXTURE["payload_schema"] == "focusa.spec172.limited_access_assertion.v1", "fixture payload schema")
    expect(re.fullmatch(r"^[0-9a-f]{64}$", FIXTURE["seed_hex"]) is not None, "fixture seed is bounded hex")
    expect(re.fullmatch(r"^[0-9a-f]{64}$", FIXTURE["public_key_hex"]) is not None, "fixture public key is bounded hex")

    # ── Key derivation is cross-language: the PHP keypair == Python keypair ──
    derived = Ed25519PrivateKey.from_private_bytes(bytes.fromhex(FIXTURE["seed_hex"]))
    derived_public = derived.public_key().public_bytes_raw().hex()
    expect(derived_public == FIXTURE["public_key_hex"], "PHP and Python derive the same Ed25519 public key from the seed")

    public_key = Ed25519PublicKey.from_public_bytes(bytes.fromhex(FIXTURE["public_key_hex"]))
    client = LimitedAssertionClientStore(public_key)

    # ── Registry coherence ───────────────────────────────────────────────
    focusa = set(FIXTURE["registries"]["focusa"]["limited"])
    uiai = set(FIXTURE["registries"]["uiai_engine"]["limited"])
    permanent = set(FIXTURE["registries"]["focusa"]["permanent"])
    expect(focusa == {"manual_project", "manual_mission", "manual_focus_state", "manual_workpoint", "manual_trajectory", "manual_basic_evidence"}, "focusa limited registry matches Spec 172")
    expect(uiai == {"public_search", "source_to_markdown", "public_page_read", "accessibility_snapshot", "screenshot", "basic_diagnostics"}, "uiai limited registry matches Spec 172")
    expect(focusa.isdisjoint(uiai), "limited families do not leak across products")
    expect("release_proof" not in focusa | permanent, "paid family is never registered to the limited posture")

    postures = {posture["posture_uuid"]: posture for posture in FIXTURE["postures"]}
    expect(len(postures) == 4, "fixture carries four authoritative posture records")
    for posture in FIXTURE["postures"]:
        expect(posture["expiry"] == "none", f"posture {posture['posture_uuid']} has no access expiry")

    # ── Vectors: signature + fail-closed policy evaluation ───────────────
    valid_count = 0
    denied_count = 0
    signature_valid_vectors = 0
    for vector in FIXTURE["vectors"]:
        presented = vector["presented"]
        posture = postures.get(presented["posture_uuid"])
        # Independent Ed25519 verification (Python cryptography over PHP bytes).
        try:
            public_key.verify(bytes.fromhex(presented["signature"]), canonical_payload(presented))
            signature_ok = True
        except (InvalidSignature, ValueError):
            signature_ok = False
        if signature_ok:
            signature_valid_vectors += 1
        verdict = client.verify_and_store(presented, posture, vector.get("at"))
        if vector["expected"] == "valid":
            valid_count += 1
            expect(verdict == "valid", f"{vector['id']}: expected valid, got {verdict}")
            expect(signature_ok, f"{vector['id']}: a valid vector must carry a valid Ed25519 signature")
        else:
            denied_count += 1
            expect(verdict == vector["expected"], f"{vector['id']}: expected {vector['expected']}, got {verdict}")
            if vector["expected"] in ("SIGNATURE_INVALID",):
                expect(not signature_ok, f"{vector['id']}: tampered vector must fail signature verification")
            else:
                expect(signature_ok, f"{vector['id']}: policy-denied vector must still fail closed despite a valid signature")

    # ── Fail-closed coverage required by the acceptance criteria ─────────
    covered = {vector["id"]: vector["expected"] for vector in FIXTURE["vectors"]}
    for required in (
        "issue_valid_roundtrip",
        "tampered_signature",
        "tampered_payload_widened_family",
        "stale_sequence_rejected",
        "wrong_node_rejected",
        "unknown_family_rejected",
        "paid_family_rejected",
        "unverified_account_rejected",
        "revoked_assertion_rejected",
    ):
        expect(covered.get(required) is not None, f"required fail-closed vector missing: {required}")

    # ── Client/store persistence: only verified assertions are kept ─────
    # Roundtrip and refresh share (posture, sequence) → one deduplicated record.
    expect(client.stored_count() == 3, "client store persists only valid assertions (deduplicated by posture+sequence)")
    expect(client.stored_postures() == sorted([FIXTURE["postures"][0]["posture_uuid"], FIXTURE["postures"][3]["posture_uuid"], FIXTURE["postures"][1]["posture_uuid"]]), "client store mirrors only verified postures")
    alpha = next(posture for posture in FIXTURE["postures"] if posture["posture_uuid"] == FIXTURE["postures"][0]["posture_uuid"])
    expect(client.allowlist_for(alpha["posture_uuid"]) == sorted(alpha["family_allowlist"]), "client store keeps the canonical allowlist (no widening)")

    # A self-issued (unsigned) claim is rejected and never persisted.
    self_issued = dict(FIXTURE["vectors"][0]["presented"])
    self_issued["signature"] = "00" * 64
    self_issued["family_allowlist"] = ["release_proof"]
    expect(client.evaluate(self_issued, alpha) == "SIGNATURE_INVALID", "self-issued unsigned claim fails closed")
    expect(client.stored_count() == 3, "self-issued claim is never persisted")

    # ── Hygiene: no real secrets, no unmasked real email ─────────────────
    raw = VECTOR_PATH.read_text(encoding="utf-8")
    assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw, re.I)
    assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
    assert "secret_key_hex" not in raw, "fixture must not carry the Ed25519 secret key"

    print(
        json.dumps(
            {
                "schema": "focusa.spec172.limited_assertion_vector_validation.v1",
                "fixture": "byte_exact_cross_language",
                "vectors": len(FIXTURE["vectors"]),
                "signature_valid": signature_valid_vectors,
                "valid": valid_count,
                "denied": denied_count,
                "client_stored": client.stored_count(),
                "key_derivation": "cross_language_identical",
                "positive_checks": POSITIVE,
                "negative_checks": NEGATIVE,
                "result": "passed_fail_closed",
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
