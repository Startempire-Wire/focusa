#!/usr/bin/env python3
"""Validate the Spec 152 feature registry and emit its canonical digest."""

import argparse
import hashlib
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REGISTRY = ROOT / "docs/contracts/spec152-feature-registry.v1.yaml"
DEFAULT_DIGEST = ROOT / "docs/contracts/spec152-feature-registry.v1.sha256"
OPERATION_CLASSES = {"read", "write", "execute", "export", "admin", "update", "install", "recovery"}
RECOVERY_POSTURES = {"always_available", "entitlement_required"}
DISCOVERABILITY = {"visible", "advanced", "internal"}
REQUIRED = {"key", "product", "operation_class", "recovery_posture", "limit_bucket", "limit_unit", "discoverability", "owner"}


def validate(registry: dict) -> bytes:
    if registry.get("schema") != "focusa.feature_registry.v1" or registry.get("product") != "focusa":
        raise ValueError("unsupported registry identity")
    features = registry.get("features")
    if not isinstance(features, list) or not features:
        raise ValueError("features must be a non-empty list")
    keys = []
    for index, feature in enumerate(features):
        if set(feature) != REQUIRED:
            raise ValueError(f"feature[{index}] fields differ: {sorted(set(feature) ^ REQUIRED)}")
        key = feature["key"]
        if not isinstance(key, str) or not key.startswith(f'{feature["product"]}.'):
            raise ValueError(f"feature[{index}] is not product-qualified")
        if feature["product"] != "focusa":
            raise ValueError(f"feature[{index}] has unknown product")
        if feature["operation_class"] not in OPERATION_CLASSES:
            raise ValueError(f"feature[{index}] has unknown operation class")
        if feature["recovery_posture"] not in RECOVERY_POSTURES:
            raise ValueError(f"feature[{index}] has unknown recovery posture")
        if feature["discoverability"] not in DISCOVERABILITY:
            raise ValueError(f"feature[{index}] has unknown discoverability")
        bucket, unit = feature["limit_bucket"], feature["limit_unit"]
        if (bucket is None) != (unit is None):
            raise ValueError(f"feature[{index}] limit bucket/unit must both be null or strings")
        if bucket is not None and (not isinstance(bucket, str) or not isinstance(unit, str)):
            raise ValueError(f"feature[{index}] limit bucket/unit are invalid")
        if not isinstance(feature["owner"], str) or not feature["owner"].strip():
            raise ValueError(f"feature[{index}] owner is empty")
        keys.append(key)
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate feature key")
    if keys != sorted(keys):
        raise ValueError("features must be sorted by stable key")
    return json.dumps(registry, sort_keys=True, separators=(",", ":")).encode()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument("--digest", type=Path, default=DEFAULT_DIGEST)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    registry = yaml.safe_load(args.registry.read_text())
    digest = hashlib.sha256(validate(registry)).hexdigest() + "  " + args.registry.name + "\n"
    if args.check:
        if not args.digest.exists() or args.digest.read_text() != digest:
            raise SystemExit("feature registry digest is stale")
    else:
        args.digest.write_text(digest)
    print(digest, end="")


if __name__ == "__main__":
    main()
