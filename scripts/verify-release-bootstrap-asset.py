#!/usr/bin/env python3
"""Verify one release bootstrap asset before executing it.

The candidate path pins the active release key to repository-owned metadata.
Revoked keys cannot authorize execution through historical timestamps or
rotation notes; historical exceptions require separate current-root authority.
Every path verifies detached Ed25519 signatures and binds
the executable to SHA256SUMS before execution.
"""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import pathlib
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey


def load_mapping(path: pathlib.Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} must contain a JSON object")
    return value


def validated_keys(metadata: dict[str, Any], source: str) -> list[dict[str, Any]]:
    if metadata.get("schema") != "focusa.trusted_release_keys.v1":
        raise ValueError(f"{source} trusted release metadata schema is invalid")
    keys = metadata.get("keys")
    if not isinstance(keys, list) or not keys:
        raise ValueError(f"{source} trusted release metadata requires keys")
    result: list[dict[str, Any]] = []
    key_ids: set[str] = set()
    fingerprints: set[str] = set()
    for key in keys:
        if not isinstance(key, dict) or key.get("signing_algorithm") != "ed25519":
            raise ValueError(f"{source} release key must use Ed25519")
        key_id = key.get("key_id")
        fingerprint = key.get("public_key_fingerprint")
        if not isinstance(key_id, str) or not key_id or key_id in key_ids:
            raise ValueError(f"{source} release key id is missing or repeated")
        if not isinstance(fingerprint, str) or fingerprint in fingerprints:
            raise ValueError(f"{source} release key fingerprint is missing or repeated")
        raw = base64.b64decode(key.get("public_key_base64", ""), validate=True)
        if len(raw) != 32 or hashlib.sha256(raw).hexdigest() != fingerprint:
            raise ValueError(f"{source} release key fingerprint mismatch")
        key_ids.add(key_id)
        fingerprints.add(fingerprint)
        result.append(key)
    return result


def transported_key(metadata: dict[str, Any]) -> dict[str, Any]:
    active = [
        key
        for key in validated_keys(metadata, "transported")
        if key.get("revoked_at") is None and key.get("valid_until") is None
    ]
    if len(active) != 1:
        raise ValueError("transported metadata requires exactly one active release key")
    return active[0]


def parse_instant(value: Any, field: str) -> dt.datetime:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field} is required for historical release verification")
    try:
        parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError(f"{field} is not an ISO-8601 timestamp") from error
    if parsed.tzinfo is None:
        raise ValueError(f"{field} must include a timezone")
    return parsed.astimezone(dt.timezone.utc)


def verify(public_key: Ed25519PublicKey, payload: pathlib.Path, signature: pathlib.Path) -> None:
    raw_signature = signature.read_bytes()
    if len(raw_signature) != 64:
        raise ValueError(f"{signature.name} is not a raw Ed25519 signature")
    try:
        public_key.verify(raw_signature, payload.read_bytes())
    except InvalidSignature as error:
        raise ValueError(f"detached signature is invalid for {payload.name}") from error


def verify_manifest_binding(
    pinned: dict[str, Any],
    public_key: Ed25519PublicKey,
    manifest_path: pathlib.Path | None,
    manifest_signature: pathlib.Path | None,
    expected_tag: str | None,
    asset_name: str,
    asset_digest: str,
) -> None:
    if manifest_path is None or manifest_signature is None or expected_tag is None:
        raise ValueError("bootstrap requires a signed release manifest and exact tag")
    if not manifest_path.is_file() or not manifest_signature.is_file():
        raise ValueError("historical release manifest or signature is missing")
    verify(public_key, manifest_path, manifest_signature)
    manifest = load_mapping(manifest_path)
    if (
        manifest.get("schema") != "focusa.release_manifest.v1"
        or manifest.get("tag") != expected_tag
        or manifest.get("yanked") is True
        or manifest.get("revoked") is True
    ):
        raise ValueError("historical release manifest identity or status is invalid")
    trust = manifest.get("trust")
    if not isinstance(trust, dict) or any(
        trust.get(field) != pinned.get(field)
        for field in ("key_id", "public_key_fingerprint", "signing_algorithm")
    ):
        raise ValueError("historical release manifest trust identity mismatch")

    assets = manifest.get("assets")
    manifest_asset = assets.get(asset_name) if isinstance(assets, dict) else None
    if not isinstance(manifest_asset, dict) or manifest_asset.get("sha256") != asset_digest:
        raise ValueError("historical release manifest does not bind the bootstrap asset digest")
    signature = manifest_asset.get("signature")
    if not isinstance(signature, dict) or signature.get("key_id") != pinned.get("key_id"):
        raise ValueError("historical bootstrap asset signature key is not manifest-bound")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--asset", required=True, type=pathlib.Path)
    parser.add_argument("--asset-signature", required=True, type=pathlib.Path)
    parser.add_argument("--checksums", required=True, type=pathlib.Path)
    parser.add_argument("--checksums-signature", required=True, type=pathlib.Path)
    parser.add_argument("--trusted-keys", required=True, type=pathlib.Path)
    parser.add_argument("--trusted-keys-signature", required=True, type=pathlib.Path)
    parser.add_argument("--pinned-trusted-keys", required=True, type=pathlib.Path)
    parser.add_argument("--release-manifest", type=pathlib.Path)
    parser.add_argument("--release-manifest-signature", type=pathlib.Path)
    parser.add_argument("--expected-tag")
    args = parser.parse_args()

    paths = [
        args.asset,
        args.asset_signature,
        args.checksums,
        args.checksums_signature,
        args.trusted_keys,
        args.trusted_keys_signature,
        args.pinned_trusted_keys,
    ]
    missing = [path.name for path in paths if not path.is_file()]
    if missing:
        raise ValueError(f"required bootstrap trust files are missing: {missing}")
    if args.asset.name.startswith(".") or args.asset.name != pathlib.Path(args.asset.name).name:
        raise ValueError("bootstrap asset name is unsafe")

    transported = transported_key(load_mapping(args.trusted_keys))
    pinned_keys = validated_keys(load_mapping(args.pinned_trusted_keys), "pinned")
    matches = [key for key in pinned_keys if key.get("key_id") == transported.get("key_id")]
    if len(matches) != 1:
        raise ValueError("transported release key differs from pinned key_id")
    pinned = matches[0]
    for field in ("public_key_fingerprint", "public_key_base64", "signing_algorithm"):
        if transported.get(field) != pinned.get(field):
            raise ValueError(f"transported release key differs from pinned {field}")

    raw_public_key = base64.b64decode(pinned["public_key_base64"], validate=True)
    public_key = Ed25519PublicKey.from_public_bytes(raw_public_key)
    verify(public_key, args.trusted_keys, args.trusted_keys_signature)
    verify(public_key, args.checksums, args.checksums_signature)
    verify(public_key, args.asset, args.asset_signature)

    entries: dict[str, str] = {}
    for line in args.checksums.read_text(encoding="utf-8").splitlines():
        fields = line.split()
        if len(fields) != 2:
            raise ValueError("SHA256SUMS contains a malformed line")
        digest, name = fields
        name = name.removeprefix("*")
        if name in entries:
            raise ValueError(f"SHA256SUMS repeats {name}")
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise ValueError(f"SHA256SUMS digest is invalid for {name}")
        entries[name] = digest
    expected = entries.get(args.asset.name)
    if expected is None:
        raise ValueError(f"SHA256SUMS does not bind {args.asset.name}")
    actual = hashlib.sha256(args.asset.read_bytes()).hexdigest()
    if actual != expected:
        raise ValueError(
            f"bootstrap asset digest mismatch: expected {expected}, got {actual}"
        )

    key_status = "active"
    if pinned.get("revoked_at") is not None:
        raise ValueError(
            "revoked bootstrap key: historical timestamps and rotation notes cannot "
            "authorize execution; current-root authorization of exact historical "
            "asset digests is required"
        )
    now = dt.datetime.now(dt.timezone.utc)
    if pinned.get("valid_from") is not None and now < parse_instant(pinned["valid_from"], "valid_from"):
        raise ValueError("bootstrap key is not yet valid")
    if pinned.get("valid_until") is not None and now >= parse_instant(pinned["valid_until"], "valid_until"):
        raise ValueError("bootstrap key is expired")

    if any((args.release_manifest, args.release_manifest_signature, args.expected_tag)):
        verify_manifest_binding(
            pinned, public_key, args.release_manifest,
            args.release_manifest_signature, args.expected_tag, args.asset.name, actual,
        )

    print(
        json.dumps(
            {
                "schema": "focusa.release_bootstrap_asset_verification.v1",
                "status": "verified",
                "asset": args.asset.name,
                "sha256": actual,
                "key_id": pinned["key_id"],
                "key_fingerprint": pinned["public_key_fingerprint"],
                "key_status": key_status,
                "release_tag": args.expected_tag,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"release bootstrap verification failed: {error}", file=__import__("sys").stderr)
        raise SystemExit(1)
