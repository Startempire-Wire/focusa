#!/usr/bin/env python3
import base64
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest

import yaml
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "canonical_assets", ROOT / "scripts/verify-canonical-release-assets.py"
)
module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(module)


class CanonicalReleaseAssetTests(unittest.TestCase):
    tag = "v9.9.9"

    def populate(self, directory: Path) -> None:
        for name in module.required_exact(self.tag):
            (directory / name).write_bytes(b"asset")
        for name in (
            "Focusa_9.9.9_aarch64.dmg",
            "Focusa_9.9.9_x64.dmg",
            "Focusa_9.9.9_x64-setup.exe",
            "Focusa_9.9.9_x64-setup.exe.sig",
            "Focusa_9.9.9_arm64-setup.exe",
            "Focusa_9.9.9_arm64-setup.exe.sig",
            "Focusa_9.9.9_x64_en-US.msi",
            "Focusa_9.9.9_x64_en-US.msi.sig",
            "Focusa_9.9.9_arm64_en-US.msi",
            "Focusa_9.9.9_arm64_en-US.msi.sig",
        ):
            (directory / name).write_bytes(b"asset")

    def test_complete_all_surface_release_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            self.populate(directory)
            self.assertEqual(module.verify(directory, self.tag), [])

    def test_real_codemagic_dmg_names_pass_same_receipt_contract(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            self.populate(directory)
            for architecture, target in [('aarch64', 'aarch64-apple-darwin'), ('x64', 'x86_64-apple-darwin')]:
                (directory / f'Focusa_9.9.9_{architecture}.dmg').rename(
                    directory / f'Focusa-{self.tag}-{target}.dmg')
            self.assertEqual(module.verify(directory, self.tag), [])
            (directory / f'Focusa-{self.tag}-aarch64-apple-darwin.dmg').unlink()
            self.assertIn('pattern-family:dmg-aarch64', module.verify(directory, self.tag))

    def test_missing_windows_signature_still_blocks(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            self.populate(directory)
            (directory / 'Focusa_9.9.9_arm64_en-US.msi.sig').unlink()
            self.assertIn('pattern-family:Focusa_*arm64*.msi.sig', module.verify(directory, self.tag))

    def test_any_missing_surface_blocks_release(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            self.populate(directory)
            missing = f"focusa-tui-{self.tag}-aarch64-pc-windows-msvc.exe"
            (directory / missing).unlink()
            self.assertIn(missing, module.verify(directory, self.tag))

    def test_missing_generated_or_installer_surface_blocks_release(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            self.populate(directory)
            for missing in (
                f"focusa-generated-clients-{self.tag}.tar.gz",
                f"focusa-installer-{self.tag}.ps1",
            ):
                (directory / missing).unlink()
                self.assertIn(missing, module.verify(directory, self.tag))
                (directory / missing).write_bytes(b"asset")


class ReleasePublicationRegressionTests(unittest.TestCase):
    def workflow(self):
        return yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())

    def test_editorial_override_keeps_substantive_release_gates(self):
        launcher = yaml.safe_load((ROOT / ".github/workflows/dev-release-tag.yml").read_text())
        step = next(s for s in launcher["jobs"]["create-tag"]["steps"] if s.get("name") == "Create approved immutable tag")
        run = step["run"]
        self.assertIn('if [ -n "$RELEASE_REASON" ]; then', run)
        override = run.index("--force-release")
        self.assertLess(run.index("scripts/release-gate.py"), override)
        self.assertLess(run.index("scripts/next-version.py"), override)
        self.assertIn('(.violations | length == 0)', run)
        self.assertIn('scripts/create-dev-release-tag.sh "${args[@]}" --push', run)
        self.assertNotIn("--no-verify", run)

    def test_receipt_gates_and_immutable_validation_cannot_be_skipped(self):
        workflow = self.workflow()
        jobs = workflow["jobs"]
        self.assertIn("windows-ovh-executables", jobs["external-menubar-receipts"]["needs"])
        for gate in ("external-menubar-receipts", "external-rust-binaries", "tag-ci-proof"):
            self.assertIn(gate, jobs["checksums"]["needs"])
        immutable = next(s for s in jobs["rust-check"]["steps"] if s.get("name") == "Verify immutable tag identity")
        self.assertNotIn("if", immutable)
        text = (ROOT / ".github/workflows/release.yml").read_text()
        self.assertNotIn("recover_staged_assets", text)
        self.assertNotIn('"signature": ""', text)

    def test_signing_preserves_intelligence_and_verifies_exact_tag_identity(self):
        steps = self.workflow()["jobs"]["checksums"]["steps"]
        generated = False
        for step in steps:
            if step.get("name") == "Generate typed release intelligence packet and page":
                generated = True
                continue
            if generated:
                self.assertNotIn("rm -f dist/release-intelligence", step.get("run", ""))
        signing = next(s for s in steps if s.get("name") == "Generate detached signatures, manifest, provenance, and trust metadata")
        self.assertIn('"$GITHUB_REF" != "refs/tags/${RELEASE_TAG}"', signing["run"])
        self.assertIn("cosign verify-blob", signing["run"])
        self.assertIn("release.yml@refs/tags/${RELEASE_TAG}", signing["run"])

    def test_real_tauri_sidecar_encoding_survives_updater_generation(self):
        steps = self.workflow()["jobs"]["checksums"]["steps"]
        step = next(s for s in steps if s.get("name") == "Generate signed Tauri updater metadata from provider receipts")
        script = re.search(r"python3 - <<'PY'\n(.*?)\nPY", step["run"], re.S).group(1)
        signature = base64.b64encode(b"untrusted comment: provider fixture\nfixture\n").decode()
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp) / "dist"
            dist.mkdir()
            names = (
                "Focusa_x64.app.tar.gz", "Focusa_aarch64.app.tar.gz",
                "Focusa_9.9.9_x64_en-US.msi", "Focusa_9.9.9_arm64_en-US.msi",
                "Focusa_9.9.9_x64-setup.exe", "Focusa_9.9.9_arm64-setup.exe",
            )
            for name in names:
                (dist / name).write_bytes(b"fixture")
                (dist / (name + ".sig")).write_text(signature + "\n")
            env = {**os.environ, "RELEASE_TAG": "v9.9.9", "RELEASE_REPOSITORY": "owner/repo"}
            result = subprocess.run([sys.executable], input=script, text=True, cwd=tmp, env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            latest = json.loads((dist / "latest.json").read_text())
            self.assertEqual(latest["platforms"]["darwin-aarch64"]["signature"], signature)
            (dist / "Focusa_aarch64.app.tar.gz.sig").write_text(base64.b64encode(b"raw Ed25519 signature").decode())
            rejected = subprocess.run([sys.executable], input=script, text=True, cwd=tmp, env=env, capture_output=True)
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("invalid Tauri provider signature", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
