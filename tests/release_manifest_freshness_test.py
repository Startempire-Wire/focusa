#!/usr/bin/env python3
"""Verify exact bytes and ancestry against real isolated Git histories."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("distribution_manifest", ROOT / "scripts/distribution_manifest.py")
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)
MANIFEST = owner.MANIFEST_REL


class ManifestFreshnessTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="focusa-manifest-git-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "source"
        self.root.mkdir()
        self.env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        fixture_paths = ("component",)
        fixture = patch.multiple(
            owner, LEGACY_ARTIFACT_PATHS=("artifact",),
            source_components=lambda root: {"fixture": owner.tree_contract(root, fixture_paths)},
        )
        fixture.start()
        self.addCleanup(fixture.stop)
        self.env["GIT_CONFIG_NOSYSTEM"] = "1"
        self.env["GIT_CONFIG_GLOBAL"] = os.devnull
        self.git("init", "--quiet", "--initial-branch=main")
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.9.198"\n')
        (self.root / "artifact").write_text("protected payload\n")
        (self.root / "component").write_text("protected component\n")
        self.initial = self.commit("initial")
        self.write_manifest(self.initial[:11])
        self.stamped = self.commit("stamp manifest")

    def git(self, *args, root=None, input=None):
        return subprocess.check_output(
            ["git", "-c", "core.hooksPath=" + os.devnull,
             "-c", "commit.gpgsign=false", "-c", "user.name=Fixture",
             "-c", "user.email=fixture@example.invalid", *args],
            cwd=root or self.root, env=self.env, input=input, text=True,
            stderr=subprocess.PIPE, timeout=15,
        ).strip()

    def commit(self, message):
        self.git("add", ".")
        self.git("commit", "--quiet", "-m", message)
        return self.git("rev-parse", "HEAD")

    def write_manifest(self, source, **changes):
        manifest = owner.build_manifest(self.root, {}, "0.9.198", source, "2000-01-01T00:00:00Z")
        manifest.update(changes)
        path = self.root / MANIFEST
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(manifest))

    def check_gate(self, passes, *, root=None, error=None):
        root = root or self.root
        source = json.loads((root / MANIFEST).read_text())
        with patch.dict(os.environ, self.env, clear=True):
            failures = owner.verify_manifest(root, source)
            failures += owner.verify_source_ancestor(root, source["source_commit"])
        self.assertEqual(not failures, passes, failures)
        if error:
            self.assertIn(error, "\n".join(failures))

    def test_head_and_parent_resolve_independently_of_display_width(self):
        for commit in (self.initial, self.stamped):
            for width in (7, 8, 11, 40):
                for display in (7, 11, 40):
                    with self.subTest(commit=commit, width=width, display=display):
                        self.git("config", "core.abbrev", str(display))
                        self.write_manifest(commit[:width])
                        self.check_gate(True)

    def test_unrelated_commits_do_not_invalidate_unchanged_covered_bytes(self):
        for n in range(3):
            (self.root / "notes").write_text(str(n))
            self.commit("unrelated change")
        self.check_gate(True)
        (self.root / "component").write_text("covered change")
        self.commit("change shipped component")
        self.check_gate(False, error="component digest/runtime contract is stale")

    def test_missing_and_non_oid_sources_fail_closed(self):
        for source in (None, [], {}, "HEAD", "--help", "not-a-commit", "f" * 40):
            with self.subTest(source=source):
                self.write_manifest(source)
                self.check_gate(False, error="source_commit")

    def test_blob_oid_is_not_a_commit(self):
        blob = self.git("hash-object", "-w", "--stdin", input="not a commit\n")
        self.write_manifest(blob)
        self.check_gate(False, error="source_commit")

    def test_hex_named_ref_cannot_shadow_missing_object_identity(self):
        name = "f" * 40
        self.git("update-ref", "refs/heads/" + name, self.stamped)
        self.write_manifest(name)
        self.check_gate(False, error="source_commit")

    def test_unrelated_commit_fails_even_with_matching_bytes(self):
        tree = self.git("rev-parse", "HEAD^{tree}")
        unrelated = self.git("commit-tree", tree, input="unrelated root\n")
        self.write_manifest(unrelated)
        self.check_gate(False, error="cannot be verified as an ancestor")

    def test_ambiguous_oid_fails_closed(self):
        tree = self.git("rev-parse", "HEAD^{tree}")
        seen = {}
        for n in range(250000):
            body = (f"tree {tree}\nauthor Fixture <fixture@example.invalid> 1 +0000\n"
                    f"committer Fixture <fixture@example.invalid> 1 +0000\n\n{n}\n")
            payload = body.encode()
            oid = hashlib.sha1(b"commit " + str(len(payload)).encode() + b"\0" + payload).hexdigest()
            prefix = oid[:7]
            if prefix in seen:
                self.git("hash-object", "-t", "commit", "-w", "--stdin", input=seen[prefix])
                self.git("hash-object", "-t", "commit", "-w", "--stdin", input=body)
                self.write_manifest(prefix)
                self.check_gate(False, error="ambiguous")
                return
            seen[prefix] = body
        self.fail("deterministic collision fixture exhausted its bound")

    def test_full_and_blobless_clones_accept_the_same_parent_stamp(self):
        self.git("config", "uploadpack.allowFilter", "true")
        for kind, options in (("full", []), ("blobless", ["--filter=blob:none", "--depth=2"])):
            clone = Path(self.temporary.name) / kind
            self.git("clone", "--quiet", "--no-local", *options, self.root.as_uri(), str(clone))
            self.git("config", "core.abbrev", "7", root=clone)
            if kind == "blobless":
                self.assertEqual(self.git("config", "--get", "remote.origin.promisor", root=clone), "true")
            self.check_gate(True, root=clone)

    def test_shallow_clone_without_source_commit_fails_closed(self):
        clone = Path(self.temporary.name) / "shallow"
        self.git("clone", "--quiet", "--depth=1", self.root.as_uri(), str(clone))
        self.check_gate(False, root=clone, error="source_commit")

    def test_initial_commit_has_no_implicit_parent(self):
        self.git("checkout", "--quiet", "--detach", self.initial)
        self.write_manifest(self.initial[:11])
        self.check_gate(True)

    def test_existing_artifact_digest_gate_is_preserved(self):
        self.write_manifest(self.stamped)
        (self.root / "artifact").write_text("tampered\n")
        self.check_gate(False, error="artifact SHA-256 map is stale")

    def test_timestamp_is_provenance_not_expiry(self):
        self.check_gate(True)
        self.write_manifest(self.stamped, generated_at="2026-02-30T00:00:00Z")
        self.check_gate(False, error="generated_at")
        self.write_manifest(self.stamped, release_version="wrong")
        self.check_gate(False, error="release_version")

    def test_preflight_calls_the_canonical_validator(self):
        self.assertIn('scripts/distribution_manifest.py --check --source-ancestor',
                      (ROOT / "scripts/local-release-preflight.sh").read_text())


if __name__ == "__main__":
    unittest.main(verbosity=2)
