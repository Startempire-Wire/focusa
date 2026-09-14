#!/usr/bin/env python3
"""Regression tests for the pre-execution release bootstrap verifier."""

from __future__ import annotations

import base64
import hashlib
import json
import pathlib
import subprocess
import tempfile

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/verify-release-bootstrap-asset.py"


def key_record(
    private: Ed25519PrivateKey,
    key_id: str = "test-release-key",
    *,
    valid_from: str = "2026-08-01T00:00:00Z",
    valid_until: str | None = None,
    revoked_at: str | None = None,
) -> dict[str, object]:
    raw = private.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    return {
        "key_id": key_id,
        "signing_algorithm": "ed25519",
        "public_key_fingerprint": hashlib.sha256(raw).hexdigest(),
        "public_key_base64": base64.b64encode(raw).decode("ascii"),
        "valid_from": valid_from,
        "valid_until": valid_until,
        "revoked_at": revoked_at,
    }


def metadata(*keys: dict[str, object]) -> dict[str, object]:
    return {"schema": "focusa.trusted_release_keys.v1", "keys": list(keys)}


def run(
    paths: dict[str, pathlib.Path],
    *,
    expected_tag: str | None = None,
) -> subprocess.CompletedProcess[str]:
    command = [
        "python3",
        str(SCRIPT),
        "--asset",
        str(paths["asset"]),
        "--asset-signature",
        str(paths["asset_signature"]),
        "--checksums",
        str(paths["checksums"]),
        "--checksums-signature",
        str(paths["checksums_signature"]),
        "--trusted-keys",
        str(paths["trusted_keys"]),
        "--trusted-keys-signature",
        str(paths["trusted_keys_signature"]),
        "--pinned-trusted-keys",
        str(paths["pinned_trusted_keys"]),
    ]
    if expected_tag is not None:
        command += [
            "--release-manifest",
            str(paths["release_manifest"]),
            "--release-manifest-signature",
            str(paths["release_manifest_signature"]),
            "--expected-tag",
            expected_tag,
        ]
    return subprocess.run(command, text=True, capture_output=True)


def sign(private: Ed25519PrivateKey, *paths: pathlib.Path) -> None:
    for payload in paths:
        payload.with_name(payload.name + ".sig").write_bytes(
            private.sign(payload.read_bytes())
        )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="focusa-bootstrap-verify-") as raw:
        root = pathlib.Path(raw)
        private = Ed25519PrivateKey.generate()
        active = key_record(private)
        asset = root / "focusa-v0.9.188-x86_64-unknown-linux-musl"
        asset.write_bytes(b"candidate-cli")
        digest = hashlib.sha256(asset.read_bytes()).hexdigest()
        checksums = root / "SHA256SUMS.txt"
        checksums.write_text(f"{digest}  {asset.name}\n", encoding="utf-8")
        trusted_keys = root / "focusa-trusted-release-keys.json"
        trusted_keys.write_text(
            json.dumps(metadata(active), sort_keys=True) + "\n", encoding="utf-8"
        )
        pinned = root / "pinned-trusted-release-keys.json"
        pinned.write_bytes(trusted_keys.read_bytes())

        paths = {
            "asset": asset,
            "asset_signature": root / f"{asset.name}.sig",
            "checksums": checksums,
            "checksums_signature": root / "SHA256SUMS.txt.sig",
            "trusted_keys": trusted_keys,
            "trusted_keys_signature": root / "focusa-trusted-release-keys.json.sig",
            "pinned_trusted_keys": pinned,
            "release_manifest": root / "release-manifest.json",
            "release_manifest_signature": root / "release-manifest.json.sig",
        }
        sign(private, asset, checksums, trusted_keys)

        accepted = run(paths)
        assert accepted.returncode == 0, accepted.stderr
        receipt = json.loads(accepted.stdout)
        assert receipt["status"] == "verified"
        assert receipt["asset"] == asset.name
        assert receipt["sha256"] == digest
        assert receipt["key_status"] == "active"

        asset.write_bytes(b"tampered")
        rejected = run(paths)
        assert rejected.returncode != 0
        assert "signature is invalid" in rejected.stderr
        asset.write_bytes(b"candidate-cli")

        other = Ed25519PrivateKey.generate()
        pinned.write_text(json.dumps(metadata(key_record(other))), encoding="utf-8")
        wrong_key = run(paths)
        assert wrong_key.returncode != 0
        assert "differs from pinned" in wrong_key.stderr

        # Neither a backdated signed manifest nor rotation prose authorizes
        # execution under a currently revoked key.
        rotated = dict(active)
        rotated["valid_until"] = "2026-08-24T00:00:00Z"
        rotated["revoked_at"] = "2026-08-24T00:00:00Z"
        successor = key_record(Ed25519PrivateKey.generate(), "successor")
        successor["supersedes"] = active["key_id"]
        successor["note"] = "rotation: prior key unrecoverable"
        pinned.write_text(
            json.dumps(metadata(rotated, successor), sort_keys=True) + "\n",
            encoding="utf-8",
        )
        manifest = {
            "schema": "focusa.release_manifest.v1",
            "tag": "v0.9.177",
            "published_at": "2026-08-19T16:55:23Z",
            "yanked": False,
            "revoked": False,
            "trust": {
                "key_id": active["key_id"],
                "public_key_fingerprint": active["public_key_fingerprint"],
                "signing_algorithm": "ed25519",
            },
            "assets": {
                asset.name: {
                    "sha256": digest,
                    "signature": {"key_id": active["key_id"]},
                }
            },
        }
        paths["release_manifest"].write_text(
            json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8"
        )
        paths["release_manifest_signature"].write_bytes(
            private.sign(paths["release_manifest"].read_bytes())
        )
        # Even active-key bootstrap evidence must bind the caller's exact tag.
        pinned.write_text(json.dumps(metadata(active)), encoding="utf-8")
        bound = run(paths, expected_tag="v0.9.177")
        assert bound.returncode == 0, bound.stderr
        wrong_tag = run(paths, expected_tag="v0.9.178")
        assert wrong_tag.returncode != 0
        assert "identity or status is invalid" in wrong_tag.stderr
        pinned.write_text(
            json.dumps(metadata(rotated, successor), sort_keys=True) + "\n",
            encoding="utf-8",
        )
        historical = run(paths, expected_tag="v0.9.177")
        assert historical.returncode != 0
        assert "revoked bootstrap key" in historical.stderr

        missing_history = run(paths)
        assert missing_history.returncode != 0
        assert "revoked bootstrap key" in missing_history.stderr

        successor["note"] = "revoked after suspected compromise"
        pinned.write_text(
            json.dumps(metadata(rotated, successor), sort_keys=True) + "\n",
            encoding="utf-8",
        )
        compromised = run(paths, expected_tag="v0.9.177")
        assert compromised.returncode != 0
        assert "revoked bootstrap key" in compromised.stderr
        successor["note"] = "rotation: prior key unrecoverable"
        pinned.write_text(
            json.dumps(metadata(rotated, successor), sort_keys=True) + "\n",
            encoding="utf-8",
        )

        after_rotation = dict(manifest)
        after_rotation["published_at"] = "2026-08-25T00:00:00Z"
        paths["release_manifest"].write_text(
            json.dumps(after_rotation, sort_keys=True) + "\n", encoding="utf-8"
        )
        paths["release_manifest_signature"].write_bytes(
            private.sign(paths["release_manifest"].read_bytes())
        )
        late = run(paths, expected_tag="v0.9.177")
        assert late.returncode != 0
        assert "revoked bootstrap key" in late.stderr

    print("release bootstrap asset verifier: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
