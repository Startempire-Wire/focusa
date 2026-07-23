#!/usr/bin/env python3
"""Generate a signed, machine-verifiable Focusa production deploy-success proof."""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import pathlib
from typing import Any

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

SCHEMA = "focusa.deploy_success.v1"
WORKFLOW = ".github/workflows/deploy-live-daemon.yml"


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_private_key(path: pathlib.Path) -> Ed25519PrivateKey:
    key = serialization.load_pem_private_key(path.read_bytes(), password=None)
    if not isinstance(key, Ed25519PrivateKey):
        raise ValueError("release signing key must be Ed25519")
    return key


def write_json(path: pathlib.Path, payload: dict[str, Any]) -> bytes:
    encoded = (json.dumps(payload, indent=2, sort_keys=True) + "\n").encode()
    path.write_bytes(encoded)
    return encoded


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--run-url", required=True)
    parser.add_argument("--asset", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--manifest-signature", type=pathlib.Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--private-key", type=pathlib.Path, required=True)
    args = parser.parse_args()

    if not args.tag.startswith("v") or len(args.commit) != 40:
        raise ValueError("canonical release tag and 40-character commit are required")
    if not args.run_url.startswith("https://github.com/"):
        raise ValueError("deploy proof run URL must be a GitHub Actions URL")
    if (
        not args.asset.is_file()
        or not args.manifest.is_file()
        or not args.manifest_signature.is_file()
    ):
        raise ValueError(
            "deployed asset, release manifest, and manifest signature must exist"
        )

    private_key = load_private_key(args.private_key)
    public_key = private_key.public_key()
    public_raw = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    manifest_bytes = args.manifest.read_bytes()
    try:
        public_key.verify(args.manifest_signature.read_bytes(), manifest_bytes)
    except Exception as error:
        raise ValueError("release manifest detached signature is invalid") from error
    manifest = json.loads(manifest_bytes)
    if manifest.get("schema") != "focusa.release_manifest.v1":
        raise ValueError("release manifest schema is not canonical")
    if manifest.get("tag") != args.tag or manifest.get("commit") != args.commit:
        raise ValueError("release manifest identity does not match deployed release")
    trust = manifest.get("trust", {})
    if (
        trust.get("public_key_fingerprint") != hashlib.sha256(public_raw).hexdigest()
        or trust.get("signing_algorithm") != "ed25519"
        or trust.get("revoked_at") is not None
    ):
        raise ValueError(
            "release manifest trust root does not match active signing key"
        )
    manifest_asset = manifest.get("assets", {}).get(args.asset.name)
    if not isinstance(manifest_asset, dict):
        raise ValueError("deployed daemon asset is absent from release manifest")
    asset_sha256 = sha256(args.asset)
    if manifest_asset.get("sha256") != asset_sha256:
        raise ValueError(
            "deployed daemon asset checksum does not match release manifest"
        )

    payload = {
        "schema": SCHEMA,
        "tag": args.tag,
        "commit": args.commit,
        "version": args.version,
        "environment": "production",
        "workflow": WORKFLOW,
        "run_url": args.run_url,
        "success": True,
        "smoke_success": True,
        "asset_name": args.asset.name,
        "asset_sha256": asset_sha256,
        "release_manifest_sha256": sha256(args.manifest),
        "deployed_at": dt.datetime.now(dt.UTC)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    encoded = write_json(args.output, payload)
    signature = private_key.sign(encoded)
    args.output.with_name(args.output.name + ".sig").write_bytes(signature)
    print(
        json.dumps(
            {
                "schema": "focusa.deploy_success_generation.v1",
                "status": "generated",
                "proof": str(args.output),
                "signature": str(args.output.with_name(args.output.name + ".sig")),
                "signature_base64": base64.b64encode(signature).decode(),
                "asset_sha256": asset_sha256,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
