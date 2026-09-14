#!/usr/bin/env python3
"""Runtime test for signed OTA release metadata generation."""

from __future__ import annotations

import base64
import hashlib
import importlib.util
import json
import pathlib
import subprocess
import tempfile

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/release-trust-metadata.py"


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, check=check, text=True, capture_output=True)


def main() -> int:
    spec = importlib.util.spec_from_file_location("release_trust_metadata", SCRIPT)
    assert spec is not None and spec.loader is not None
    metadata = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(metadata)
    baseline = metadata.compatibility_baseline_input("v0.9.177")
    for field, invalid in (
        ("schema", "unknown"), ("tag", "v0.9.178"),
        ("source_commit", "main"), ("provider_release_id", True),
        ("checksums_sha256", "bad"), ("assets", {}),
    ):
        broken = {**baseline, field: invalid}
        try:
            metadata.validate_compatibility_baseline_input(broken, "v0.9.177")
        except ValueError:
            pass
        else:
            raise AssertionError(f"invalid baseline {field} was accepted")
    for asset_name, digest in (("../escape-v0.9.177", "a" * 64), ("extra-v0.9.177", "invalid")):
        broken = {**baseline, "assets": {**baseline["assets"], asset_name: digest}}
        try:
            metadata.validate_compatibility_baseline_input(broken, "v0.9.177")
        except ValueError:
            pass
        else:
            raise AssertionError("invalid baseline asset was accepted")
    with tempfile.TemporaryDirectory(prefix="focusa-release-trust-") as raw:
        root = pathlib.Path(raw)
        dist = root / "dist"
        dist.mkdir()
        private_key = root / "private.pem"
        trusted_keys = root / "trusted.json"

        private = Ed25519PrivateKey.generate()
        private_key.write_bytes(
            private.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            )
        )
        private_key.chmod(0o600)
        public = private.public_key()
        public_raw = public.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )
        fingerprint = hashlib.sha256(public_raw).hexdigest()
        trusted_keys.write_text(
            json.dumps(
                {
                    "schema": "focusa.trusted_release_keys.v1",
                    "keys": [
                        {
                            "key_id": "focusa-test-release-key",
                            "signing_algorithm": "ed25519",
                            "public_key_fingerprint": fingerprint,
                            "public_key_base64": base64.b64encode(public_raw).decode(
                                "ascii"
                            ),
                            "valid_from": "2026-07-15T00:00:00Z",
                            "valid_until": None,
                            "revoked_at": None,
                        }
                    ],
                }
            )
        )

        assets = [
            dist / "focusa-v0.9.95-dev-x86_64-unknown-linux-musl",
            dist / "focusa-daemon-v0.9.95-dev-aarch64-apple-darwin",
        ]
        assets[0].write_bytes(b"linux-fixture")
        assets[1].write_bytes(b"macos-fixture")

        common_args = [
            "python3",
            str(SCRIPT),
            "--dist",
            str(dist),
            "--tag",
            "v0.9.95-dev",
            "--commit",
            "a" * 40,
            "--repo",
            "Startempire-Wire/focusa",
            "--run-url",
            "https://github.com/Startempire-Wire/focusa/actions/runs/1",
            "--private-key",
            str(private_key),
            "--trusted-keys",
            str(trusted_keys),
        ]
        result = run(*common_args)
        summary = json.loads(result.stdout)
        assert summary["status"] == "completed"
        assert summary["asset_count"] == 2
        assert summary["signed_file_count"] == 6

        manifest = json.loads((dist / "release-manifest.json").read_text())
        assert manifest["schema"] == "focusa.release_manifest.v1"
        assert manifest["tag"] == "v0.9.95-dev"
        assert manifest["channel"] == "nightly"
        assert manifest["publication_status"] == "published"
        assert manifest["gates"]["release_success"] is True
        assert manifest["gates"]["release_run_url"].endswith("/runs/1")
        assert manifest["rollback_supported"] is True
        assert set(manifest["assets"]) == {asset.name for asset in assets}
        assert manifest["assets"][assets[0].name]["platform"] == "linux-x86_64-musl"
        assert manifest["assets"][assets[1].name]["platform"] == "macos-aarch64"

        metadata = [
            dist / "SHA256SUMS.txt",
            dist / "release-manifest.json",
            dist / "release-provenance.json",
            dist / "focusa-trusted-release-keys.json",
        ]
        for path in assets + metadata:
            signature = path.with_name(path.name + ".sig")
            assert signature.stat().st_size == 64
            public.verify(signature.read_bytes(), path.read_bytes())

        candidate = run(
            *common_args,
            "--candidate",
            "--compatibility-from-tag",
            "v0.9.177",
            "--workflow",
            ".github/workflows/locked-release-candidate-artifacts.yml",
        )
        assert json.loads(candidate.stdout)["status"] == "completed"
        candidate_manifest = json.loads((dist / "release-manifest.json").read_text())
        candidate_provenance = json.loads(
            (dist / "release-provenance.json").read_text()
        )
        assert candidate_manifest["publication_status"] == "candidate_only"
        assert candidate_manifest["gates"]["release_success"] is False
        assert candidate_manifest["gates"]["release_run_url"] is None
        canary = candidate_manifest["compatibility_canary"]
        assert canary == {
            "schema": "focusa.compatibility_canary_authorization.v1",
            "status": "authorized",
            "environment": "isolated_preproduction",
            "allowed_install_scope": "non_root_ephemeral_home",
            "required_previous_tag": "v0.9.177",
            "baseline_release": json.loads(
                (ROOT / "config/compatibility-canary-baselines/v0.9.177.json").read_text()
            ),
            "required_sequence": [
                "prior_release",
                "candidate_manifest_bound_apply",
                "prior_release_full_rollback",
                "candidate_manifest_bound_reapply",
            ],
            "production_apply_authorized": False,
            "system_install_authorized": False,
            "service_mutation_authorized": False,
            "automatic_apply_authorized": False,
        }
        assert all(
            value["url"].startswith(
                "https://github.com/Startempire-Wire/focusa/actions/runs/1#artifact-"
            )
            for value in candidate_manifest["assets"].values()
        )
        assert candidate_provenance["workflow"].endswith(
            "locked-release-candidate-artifacts.yml"
        )

        candidate_bytes = (dist / "release-manifest.json").read_bytes()
        candidate_signature = (dist / "release-manifest.json.sig").read_bytes()
        public.verify(candidate_signature, candidate_bytes)
        altered = json.loads(candidate_bytes)
        altered["compatibility_canary"]["baseline_release"]["checksums_sha256"] = "0" * 64
        try:
            public.verify(candidate_signature, json.dumps(altered, sort_keys=True).encode())
        except InvalidSignature:
            pass
        else:
            raise AssertionError("modified baseline binding retained candidate authority")

        unauthorized = run(
            *common_args,
            "--compatibility-from-tag",
            "v0.9.177",
            check=False,
        )
        assert unauthorized.returncode != 0
        assert "requires --candidate" in unauthorized.stderr

        assets[0].write_bytes(b"tampered")
        try:
            public.verify(
                assets[0].with_name(assets[0].name + ".sig").read_bytes(),
                assets[0].read_bytes(),
            )
        except InvalidSignature:
            pass
        else:
            raise AssertionError("tampered asset unexpectedly verified")

    print(
        "PASS: release assets, checksums, manifest, provenance, and trust metadata are detached-signed and tamper-evident"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
