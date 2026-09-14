#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "scripts" / "verify-embedded-authority-root.py"
KEY_ID = "authority-root-2026-01"
PUBLIC_KEY = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="


class ReleaseAuthorityRootEmbeddingTest(unittest.TestCase):
    def run_probe(self, binary: Path, roots: object | None) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        if roots is None:
            env.pop("FOCUSA_AUTHORITY_ROOT_KEYS_JSON", None)
        else:
            env["FOCUSA_AUTHORITY_ROOT_KEYS_JSON"] = json.dumps(roots)
        return subprocess.run(
            [sys.executable, str(PROBE), str(binary)],
            cwd=ROOT,
            env=env,
            check=False,
            text=True,
            capture_output=True,
        )

    def test_probe_requires_production_configuration_and_binary_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "focusa-daemon"
            binary.write_bytes(b"release-prefix")

            missing = self.run_probe(binary, None)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn("is required", missing.stderr)

            forbidden = self.run_probe(binary, {"test-root": PUBLIC_KEY})
            self.assertNotEqual(forbidden.returncode, 0)
            self.assertIn("forbidden non-production", forbidden.stderr)

            malformed = self.run_probe(binary, {KEY_ID: "not-base64"})
            self.assertNotEqual(malformed.returncode, 0)
            self.assertIn("not valid Base64", malformed.stderr)

            wrong_length = self.run_probe(binary, {KEY_ID: "c2hvcnQ="})
            self.assertNotEqual(wrong_length.returncode, 0)
            self.assertIn("must decode to 32 bytes", wrong_length.stderr)

            missing_embedding = self.run_probe(binary, {KEY_ID: PUBLIC_KEY})
            self.assertNotEqual(missing_embedding.returncode, 0)
            self.assertIn("lacks authority root key ID", missing_embedding.stderr)
            self.assertNotIn(PUBLIC_KEY, missing_embedding.stdout + missing_embedding.stderr)

            binary.write_bytes(
                b"release-prefix\x00" + KEY_ID.encode() + b"\x00" + PUBLIC_KEY.encode() + b"\x00suffix"
            )
            accepted = self.run_probe(binary, {KEY_ID: PUBLIC_KEY})
            self.assertEqual(accepted.returncode, 0, accepted.stderr)
            self.assertIn("authority_root_embedding=passed", accepted.stdout)
            self.assertNotIn(PUBLIC_KEY, accepted.stdout + accepted.stderr)

    def test_every_rust_release_provider_sets_and_verifies_authority_roots(self) -> None:
        github = (ROOT / ".github/workflows/release.yml").read_text()
        appveyor = (ROOT / ".appveyor.yml").read_text()
        codemagic = (ROOT / "codemagic.yaml").read_text()
        license_build = (ROOT / "crates/focusa-license/build.rs").read_text()

        for name, source in {
            "github": github,
            "appveyor": appveyor,
            "codemagic": codemagic,
        }.items():
            with self.subTest(provider=name):
                self.assertIn("FOCUSA_AUTHORITY_ROOT_KEYS_JSON", source)
                self.assertIn("scripts/verify-embedded-authority-root.py", source)

        self.assertIn('test -s "$src"', github)
        self.assertGreaterEqual(github.count("FOCUSA_AUTHORITY_ROOT_KEYS_JSON"), 3)
        self.assertIn(
            'passthrough = ["FOCUSA_AUTHORITY_ROOT_KEYS_JSON"]', github
        )
        self.assertIn('if [ "$bin" = "focusa" ] || [ "$bin" = "focusa-daemon" ]', github)
        self.assertIn("skip_branch_with_pr: true", appveyor)
        self.assertIn('@("focusa-daemon", "focusa")', appveyor)
        self.assertIn("production authority root embedding proof failed", appveyor)
        self.assertIn('if [[ "$bin" == "focusa" || "$bin" == "focusa-daemon" ]]', codemagic)
        self.assertIn('"${authority_binaries[@]}"', codemagic)
        self.assertIn(
            "cargo:rerun-if-env-changed=FOCUSA_AUTHORITY_ROOT_KEYS_JSON",
            license_build,
        )

    def test_probe_rejects_missing_and_empty_binaries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            missing = Path(directory) / "missing"
            result = self.run_probe(missing, {KEY_ID: PUBLIC_KEY})
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("missing or empty", result.stderr)

            empty = Path(directory) / "empty"
            empty.touch()
            result = self.run_probe(empty, {KEY_ID: PUBLIC_KEY})
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("missing or empty", result.stderr)


if __name__ == "__main__":
    unittest.main()
