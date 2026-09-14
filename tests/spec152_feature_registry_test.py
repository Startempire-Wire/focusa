#!/usr/bin/env python3
"""Validate canonical Spec 152 feature registry completeness and fail-closed shape."""

import copy
import hashlib
import importlib.util
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "docs/contracts/spec152-feature-registry.v1.yaml"
DIGEST_PATH = ROOT / "docs/contracts/spec152-feature-registry.v1.sha256"
GENERATOR_PATH = ROOT / "scripts/generate-spec152-feature-registry.py"

spec = importlib.util.spec_from_file_location("feature_registry_generator", GENERATOR_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

registry = yaml.safe_load(REGISTRY_PATH.read_text())
canonical = module.validate(registry)
expected = hashlib.sha256(canonical).hexdigest() + "  " + REGISTRY_PATH.name + "\n"
assert DIGEST_PATH.read_text() == expected, "registry digest is stale"

keys = {feature["key"] for feature in registry["features"]}
spec_files = list((ROOT / "docs").glob("152*.md")) + [ROOT / "docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md"]
required = set()
for path in spec_files:
    # Spec 152E names the branded domain `focusa.dev`; domain names are not
    # entitlement feature keys even though they share the `focusa.` prefix.
    required.update(
        token
        for token in re.findall(r"`(focusa\.[a-z0-9_.-]+)`", path.read_text())
        if token != "focusa.dev"
    )
required.update({
    "focusa.install.channel.stable",
    "focusa.install.channel.preview",
    "focusa.install.channel.nightly",
    "focusa.repair.execute",
})
assert required <= keys, f"registry omits features: {sorted(required - keys)}"

source_keys = set()
for path in [
    ROOT / "crates/focusa-cli/src/commands/install.rs",
    ROOT / "crates/focusa-core/src/install_lifecycle/transactions.rs",
]:
    source_keys.update(re.findall(r'"(focusa\.(?:agent|core|export|install\.channel|release|remote|repair|team|update)[a-z0-9_.-]*)"', path.read_text()))
assert source_keys <= keys, f"runtime uses unregistered features: {sorted(source_keys - keys)}"

for mutation, message in [
    ((lambda value: value["features"].append(copy.deepcopy(value["features"][0]))), "duplicate"),
    ((lambda value: value["features"][0].update(operation_class="invented")), "operation"),
    ((lambda value: value["features"][0].update(product="other")), "product"),
    ((lambda value: value["features"][0].pop("owner")), "fields"),
]:
    candidate = copy.deepcopy(registry)
    mutation(candidate)
    try:
        module.validate(candidate)
    except ValueError:
        pass
    else:
        raise AssertionError(f"registry accepted invalid {message} mutation")

print(json.dumps({
    "schema": "focusa.feature_registry_validation.v1",
    "feature_count": len(keys),
    "digest": expected.split()[0],
    "unknown_keys": "fail_closed",
    "result": "passed",
}, sort_keys=True))
