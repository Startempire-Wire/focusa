#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
source = (R / "crates/focusa-core/src/software_domain.rs").read_text()
root_manifest = (R / "Cargo.toml").read_text()
core_manifest = (R / "crates/focusa-core/Cargo.toml").read_text()
contract = json.loads((R / "docs/contracts/spec135-v4-software-domain.v1.yaml").read_text())
for dependency in ('petgraph = "0.8"', 'tree-sitter = "0.25"'):
    assert dependency in root_manifest
for dependency in ("petgraph = { workspace = true }", "tree-sitter = { workspace = true }"):
    assert dependency in core_manifest
for marker in (
    "StableDiGraph",
    "Parser::new()",
    "parser.set_language",
    'args(["scan", "--json=stream", "--pattern", pattern])',
    "Sha256::digest",
    "MAX_CHANGED_FILES",
    "remove_path",
    "revision.saturating_add(1)",
    "canonical_state_unchanged: true",
):
    assert marker in source
assert contract["adopted_frameworks"] == ["petgraph", "Tree-sitter", "ast-grep"]
assert all(contract["acceptance"].values())
print("Spec 135 V4 evidence-backed software domain strict lint: PASS")
