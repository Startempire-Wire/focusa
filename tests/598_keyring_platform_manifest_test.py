"""Regression proof: native keyring tables cannot capture shared dependencies."""
import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class PlatformManifestTest(unittest.TestCase):
    def test_shared_dependencies_remain_available_on_every_platform(self):
        expected = {
            "focusa-core": {
                "rand_core", "libc", "chrono-tz", "petgraph", "tree-sitter",
                "rusqlite", "zstd", "sqlite-vec", "fastembed", "mdns-sd",
                "if-addrs", "async-trait", "toml", "focusa-license", "parking_lot",
            },
            "focusa-license": {"reqwest"},
        }
        for crate, required in expected.items():
            with self.subTest(crate=crate):
                data = tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text())
                self.assertFalse(required - data["dependencies"].keys())
                for target, feature in (("macos", "apple-native"),
                                        ("windows", "windows-native"),
                                        ("linux", "linux-native")):
                    dependencies = data["target"][f'cfg(target_os = "{target}")']["dependencies"]
                    self.assertEqual(set(dependencies), {"keyring"})
                    self.assertEqual(dependencies["keyring"]["features"], [feature])


if __name__ == "__main__":
    unittest.main()
