#!/usr/bin/env python3
"""Runtime regression for signed compatibility-canary completion evidence."""

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
SCRIPT = ROOT / "scripts/compatibility-canary-receipt.py"
TAG = "v0.9.188"
COMMIT = "1" * 40


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="focusa-canary-sign-") as raw:
        root = pathlib.Path(raw)
        private = Ed25519PrivateKey.generate()
        private_path = root / "private.pem"
        private_path.write_bytes(
            private.private_bytes(
                serialization.Encoding.PEM,
                serialization.PrivateFormat.PKCS8,
                serialization.NoEncryption(),
            )
        )
        private_path.chmod(0o600)
        public_raw = private.public_key().public_bytes(
            serialization.Encoding.Raw, serialization.PublicFormat.Raw
        )
        trusted = root / "trusted.json"
        trusted.write_text(
            json.dumps(
                {
                    "keys": [
                        {
                            "key_id": "test",
                            "signing_algorithm": "ed25519",
                            "public_key_base64": base64.b64encode(public_raw).decode(),
                            "public_key_fingerprint": hashlib.sha256(
                                public_raw
                            ).hexdigest(),
                            "revoked_at": None,
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )
        receipt = root / "compatibility-canary-success.json"
        payload = {
            "schema": "focusa.compatibility_canary_success.v1",
            "status": "passed",
            "candidate": {"tag": TAG, "commit": COMMIT},
            "previous_release_tag": "v0.9.177",
            "environment": "isolated_preproduction",
            "sequence": [
                "prior_release_healthy",
                "candidate_manifest_bound_apply_healthy",
                "prior_release_full_rollback_healthy",
                "candidate_manifest_bound_reapply_healthy",
            ],
            "database_quick_check": "ok",
            "database_evidence": {
                phase: {"schema_sha256": "a" * 64, "row_counts_sha256": "b" * 64}
                for phase in (
                    "prior_initial",
                    "prior_interrupted_recovery",
                    "candidate_first",
                    "prior_rollback",
                    "candidate_reapply",
                )
            },
            "distribution_parity": {
                "candidate_first_sha256": "c" * 64,
                "candidate_reapply_sha256": "d" * 64,
                "status": "passed",
            },
            "signed_lease_preserved": True,
            "user_sentinel_preserved": True,
            "production_runtime_preserved": True,
            "system_install_performed": False,
            "service_mutation_performed": False,
            "automatic_apply_performed": False,
            "interrupted_install_recovered": True,
            "run_url": "https://github.com/Startempire-Wire/focusa/actions/runs/1",
        }
        receipt.write_text(json.dumps(payload), encoding="utf-8")
        command = [
            "python3",
            str(SCRIPT),
            "sign",
            "--receipt",
            str(receipt),
            "--private-key",
            str(private_path),
            "--trusted-keys",
            str(trusted),
            "--tag",
            TAG,
            "--commit",
            COMMIT,
            "--previous-tag",
            "v0.9.177",
        ]
        accepted = subprocess.run(command, text=True, capture_output=True)
        assert accepted.returncode == 0, accepted.stderr
        signature = receipt.with_name(receipt.name + ".sig").read_bytes()
        assert len(signature) == 64
        private.public_key().verify(signature, receipt.read_bytes())
        verified = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "verify",
                "--receipt",
                str(receipt),
                "--signature",
                str(receipt) + ".sig",
                "--trusted-keys",
                str(trusted),
                "--tag",
                TAG,
                "--commit",
                COMMIT,
                "--previous-tag",
                "v0.9.177",
            ],
            text=True,
            capture_output=True,
        )
        assert verified.returncode == 0, verified.stderr
        assert json.loads(verified.stdout)["status"] == "verified"

        payload["production_runtime_preserved"] = False
        receipt.write_text(json.dumps(payload), encoding="utf-8")
        rejected = subprocess.run(command, text=True, capture_output=True)
        assert rejected.returncode != 0
        assert "incomplete or unsafe" in rejected.stderr

    print("compatibility canary receipt signer: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
