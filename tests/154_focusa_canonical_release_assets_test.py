#!/usr/bin/env python3
import importlib.util
import tempfile
import unittest
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


if __name__ == "__main__":
    unittest.main()
