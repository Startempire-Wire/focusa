#!/usr/bin/env python3
"""Regression tests for the one canonical distribution-manifest contract."""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts/distribution_manifest.py"
spec = importlib.util.spec_from_file_location("focusa_distribution_manifest", MODULE_PATH)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

manifest_path = ROOT / module.MANIFEST_REL
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
failures = module.verify_manifest(ROOT, manifest)
assert not failures, failures

assert manifest["digest_contract"] == "sha256-tree-v1"
assert len(manifest["components"]["rust_runtime"]["sha256"]) == len("sha256:") + 64
assert manifest["components"]["agent_skills"]["file_count"] > 0
assert manifest["components"]["capability_surfaces"]["contract_count"] > 0
runtime = manifest["components"]["runtime_contract"]
assert runtime["installed_manifest_path"] == "/usr/local/lib/focusa/distribution-manifest.json"
assert runtime["manifest_required_from"] == "0.9.188"
assert set(runtime["binary_paths"]) == {"cli", "daemon", "tui", "session_runner"}
assert all(len(digest) == len("sha256:") + 64 for digest in manifest["artifacts"].values())

with tempfile.TemporaryDirectory(prefix="focusa-distribution-tree-") as temporary:
    root = Path(temporary)
    (root / "component").mkdir()
    (root / "component/b.txt").write_text("b", encoding="utf-8")
    (root / "component/a.txt").write_text("a", encoding="utf-8")
    first = module.tree_contract(root, ("component",))
    (root / "component").rename(root / "moved")
    (root / "component").mkdir()
    (root / "component/a.txt").write_text("a", encoding="utf-8")
    (root / "component/b.txt").write_text("b", encoding="utf-8")
    second = module.tree_contract(root, ("component",))
    assert first == second, "tree digest must be deterministic across creation order"
    (root / "component/a.txt").write_text("changed", encoding="utf-8")
    assert module.tree_contract(root, ("component",)) != second

    link = root / "component/link"
    link.symlink_to(root / "component/a.txt")
    try:
        module.tree_contract(root, ("component",))
        raise AssertionError("component symlinks must be rejected")
    except ValueError as error:
        assert "symlink" in str(error)
    link.unlink()

    fifo = root / "component/special"
    os.mkfifo(fifo)
    try:
        module.tree_contract(root, ("component",))
        raise AssertionError("component special entries must be rejected")
    except ValueError as error:
        assert "special" in str(error)
    fifo.unlink()

    try:
        module.artifacts(root, {"../outside": "sha256:ignored"})
        raise AssertionError("unsafe artifact paths must be rejected")
    except ValueError as error:
        assert "unsafe" in str(error)

print("distribution manifest contract: PASS")
