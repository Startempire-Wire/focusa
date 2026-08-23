#!/usr/bin/env python3
"""Runtime test for signed OTA release metadata generation."""

from __future__ import annotations

import base64
import hashlib
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
        assert all(
            value["url"].startswith(
                "https://github.com/Startempire-Wire/focusa/actions/runs/1#artifact-"
            )
            for value in candidate_manifest["assets"].values()
        )
        assert candidate_provenance["workflow"].endswith(
            "locked-release-candidate-artifacts.yml"
        )

        ledger = root / "appveyor-ledger.json"
        ledger.write_text(
            json.dumps(
                {
                    "schema": "focusa.release_gate_ledger.v1",
                    "provider": "appveyor",
                    "repository": "Startempire-Wire/focusa",
                    "tag": "v0.9.95-dev",
                    "commit": "a" * 40,
                    "build_id": "54588263",
                    "build_url": "https://ci.appveyor.com/project/verioussmith/focusa/build/11",
                    "configuration_sha256": "b" * 64,
                    "all_green": True,
                    "gates": [{"gate": index, "status": "passed"} for index in range(1, 15)],
                }
            )
        )
        appveyor = run(
            *common_args,
            "--builder",
            "appveyor",
            "--workflow",
            ".appveyor.yml",
            "--provider-receipt",
            str(ledger),
        )
        assert json.loads(appveyor.stdout)["signed_file_count"] == 7
        appveyor_provenance = json.loads(
            (dist / "release-provenance.json").read_text()
        )
        assert appveyor_provenance["builder"] == "appveyor"
        assert appveyor_provenance["provider_evidence"]["build_id"] == "54588263"
        ledger_output = dist / "release-gate-ledger.json"
        assert ledger_output.is_file()
        public.verify(
            ledger_output.with_name(ledger_output.name + ".sig").read_bytes(),
            ledger_output.read_bytes(),
        )

        missing_receipt = run(*common_args, "--builder", "appveyor", check=False)
        assert missing_receipt.returncode == 1
        assert "requires --provider-receipt" in missing_receipt.stderr

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
