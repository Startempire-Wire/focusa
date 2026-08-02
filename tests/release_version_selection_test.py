#!/usr/bin/env python3
"""Regression tests for monotonic mixed-channel release version selection."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts/select-release-version.py"
SPEC = importlib.util.spec_from_file_location("focusa_release_version_selection", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
select_version = MODULE.select_version


class ReleaseVersionSelectionTests(unittest.TestCase):
    def test_automatic_selection_uses_highest_patch_across_channels(self) -> None:
        result = select_version(
            "0.9",
            None,
            ["v0.9.136-dev", "v0.9.142", "v0.9.140-rc.2", "v0.8.999"],
        )
        self.assertEqual(result["selected_tag"], "v0.9.143-dev")
        self.assertEqual(result["highest_patch"], 142)
        self.assertEqual(
            result["channel_maxima"],
            {"stable": 142, "dev": 136, "rc": 140, "preview": None},
        )
        self.assertTrue(result["monotonic"])

    def test_explicit_next_stable_version_is_accepted(self) -> None:
        result = select_version("0.9", "v0.9.143", ["v0.9.142", "v0.9.136-dev"])
        self.assertEqual(result["selected_channel"], "stable")
        self.assertEqual(result["selected_version"], "0.9.143")

    def test_explicit_lower_or_equal_patch_fails_closed(self) -> None:
        for tag in ("v0.9.141", "v0.9.142", "v0.9.142-rc.1"):
            with self.subTest(tag=tag), self.assertRaisesRegex(
                ValueError, "release version regression"
            ):
                select_version("0.9", tag, ["v0.9.142", "v0.9.136-dev"])

    def test_explicit_tag_must_match_requested_lane(self) -> None:
        with self.assertRaisesRegex(ValueError, "outside requested base"):
            select_version("0.9", "v1.0.1", ["v0.9.142"])

    def test_malformed_tags_cannot_influence_selection(self) -> None:
        result = select_version(
            "0.9", None, ["v0.9.x-dev", "v0.9.0143", "release-v0.9.999", "v0.9.142"]
        )
        self.assertEqual(result["selected_tag"], "v0.9.143-dev")
        self.assertEqual(
            result["ignored_malformed_tags"],
            ["release-v0.9.999", "v0.9.0143", "v0.9.x-dev"],
        )

    def test_empty_lane_preserves_documented_initial_behavior(self) -> None:
        result = select_version("0.9", None, [])
        self.assertEqual(result["selected_tag"], "v0.9.1-dev")
        self.assertEqual(result["highest_patch"], 0)


if __name__ == "__main__":
    unittest.main()
