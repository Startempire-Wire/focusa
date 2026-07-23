#!/usr/bin/env python3
"""Runtime proof for signed production deploy-success metadata generation."""

from __future__ import annotations

import base64
import json
import pathlib
import subprocess
import tempfile

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/release-deploy-proof.py"


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="focusa-deploy-proof-") as raw:
        work = pathlib.Path(raw)
        private_key = Ed25519PrivateKey.generate()
        key_path = work / "release-key.pem"
        key_path.write_bytes(
            private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            )
        )
        tag = "v0.9.99-dev"
        commit = "1" * 40
        asset = work / f"focusa-daemon-{tag}-x86_64-unknown-linux-musl"
        asset.write_bytes(b"verified-daemon")
        import hashlib

        digest = hashlib.sha256(asset.read_bytes()).hexdigest()
        public_raw = private_key.public_key().public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )
        manifest_path = work / "release-manifest.json"
        manifest_path.write_text(
            json.dumps(
                {
                    "schema": "focusa.release_manifest.v1",
                    "tag": tag,
                    "commit": commit,
                    "trust": {
                        "key_id": "test-key",
                        "signing_algorithm": "ed25519",
                        "public_key_fingerprint": hashlib.sha256(
                            public_raw
                        ).hexdigest(),
                        "public_key_base64": base64.b64encode(public_raw).decode(),
                        "revoked_at": None,
                    },
                    "assets": {asset.name: {"sha256": digest}},
                }
            )
        )
        manifest_signature = work / "release-manifest.json.sig"
        manifest_signature.write_bytes(private_key.sign(manifest_path.read_bytes()))
        output = work / "deploy-success.json"
        generated = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--output",
                str(output),
                "--tag",
                tag,
                "--commit",
                commit,
                "--run-url",
                "https://github.com/Startempire-Wire/focusa/actions/runs/1",
                "--asset",
                str(asset),
                "--manifest",
                str(manifest_path),
                "--manifest-signature",
                str(manifest_signature),
                "--version",
                "0.9.99-dev",
                "--private-key",
                str(key_path),
            ],
            capture_output=True,
            text=True,
        )
        assert generated.returncode == 0, generated.stderr
        signature = output.with_name(output.name + ".sig").read_bytes()
        private_key.public_key().verify(signature, output.read_bytes())
        proof = json.loads(output.read_text())
        assert proof["schema"] == "focusa.deploy_success.v1"
        assert proof["success"] is True and proof["smoke_success"] is True
        assert proof["asset_sha256"] == digest
        assert len(signature) == 64

        tampered = output.read_bytes().replace(b'"success": true', b'"success": false')
        try:
            private_key.public_key().verify(signature, tampered)
        except Exception:
            pass
        else:
            raise AssertionError("tampered deploy-success proof unexpectedly verified")

        bad_manifest = json.loads(manifest_path.read_text())
        bad_manifest["assets"][asset.name]["sha256"] = "0" * 64
        manifest_path.write_text(json.dumps(bad_manifest))
        manifest_signature.write_bytes(private_key.sign(manifest_path.read_bytes()))
        failed = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--output",
                str(work / "bad.json"),
                "--tag",
                tag,
                "--commit",
                commit,
                "--run-url",
                "https://github.com/Startempire-Wire/focusa/actions/runs/2",
                "--asset",
                str(asset),
                "--manifest",
                str(manifest_path),
                "--manifest-signature",
                str(manifest_signature),
                "--version",
                "0.9.99-dev",
                "--private-key",
                str(key_path),
            ],
            capture_output=True,
            text=True,
        )
        assert failed.returncode != 0
        assert "checksum does not match" in failed.stderr

    print(
        "PASS: deploy-success metadata is release-bound, detached-signed, and tamper-evident"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
