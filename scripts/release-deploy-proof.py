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
    parser.add_argument("--distribution-parity", type=pathlib.Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--private-key", type=pathlib.Path, required=True)
    parser.add_argument(
        "--settle",
        action="store_true",
        help="re-sign the deployed candidate after OTA acceptance for stable promotion",
    )
    args = parser.parse_args()

    if not args.tag.startswith("v") or len(args.commit) != 40:
        raise ValueError("canonical release tag and 40-character commit are required")
    if not args.run_url.startswith("https://github.com/"):
        raise ValueError("deploy proof run URL must be a GitHub Actions URL")
    if (
        not args.asset.is_file()
        or not args.manifest.is_file()
        or not args.manifest_signature.is_file()
        or not args.distribution_parity.is_file()
    ):
        raise ValueError(
            "deployed asset, release manifest/signature, and distribution parity proof must exist"
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
    parity = json.loads(args.distribution_parity.read_bytes())
    installed_parity = parity.get("installed", {})
    binary_versions = installed_parity.get("binary_versions", {})
    distribution_manifest_sha256 = installed_parity.get("manifest_sha256")
    if (
        parity.get("schema") != "focusa.distribution_parity.v1"
        or parity.get("parity_ok") is not True
        or parity.get("drift") != []
        or installed_parity.get("manifest_version") != args.version
        or (
            not args.settle
            and installed_parity.get("release_manifest_sha256")
            != "sha256:" + hashlib.sha256(manifest_bytes).hexdigest()
        )
        or (
            args.settle
            and manifest.get("publication_status") != "deployed_candidate"
        )
        or not isinstance(distribution_manifest_sha256, str)
        or len(distribution_manifest_sha256) != len("sha256:") + 64
        or not distribution_manifest_sha256.startswith("sha256:")
        or distribution_manifest_sha256
        != parity.get("source_manifest", {}).get("sha256")
        or set(binary_versions) != {"cli", "daemon", "tui", "session_runner"}
        or set(binary_versions.values()) != {args.version}
    ):
        raise ValueError("installed distribution parity proof is not accepted")
    manifest_asset = manifest.get("assets", {}).get(args.asset.name)
    if not isinstance(manifest_asset, dict):
        raise ValueError("deployed daemon asset is absent from release manifest")
    asset_sha256 = sha256(args.asset)
    if manifest_asset.get("sha256") != asset_sha256:
        raise ValueError(
            "deployed daemon asset checksum does not match release manifest"
        )

    candidate_run_urls = {
        asset_contract.get("url", "").split("#artifact-", 1)[0]
        for asset_contract in manifest.get("assets", {}).values()
        if isinstance(asset_contract, dict) and "#artifact-" in asset_contract.get("url", "")
    }
    release_run_url = (
        next(iter(candidate_run_urls))
        if len(candidate_run_urls) == 1
        else manifest.get("gates", {}).get("release_run_url", "")
    )
    if "/actions/runs/" not in release_run_url:
        raise ValueError("candidate manifest does not bind one canonical release run")
    repository_url, separator, _ = args.run_url.partition("/actions/runs/")
    if not separator or not repository_url.startswith("https://github.com/"):
        raise ValueError("deploy run URL does not identify a canonical GitHub repository")
    for asset_name, asset_contract in manifest.get("assets", {}).items():
        if not isinstance(asset_contract, dict):
            raise ValueError(f"release manifest asset contract is invalid: {asset_name}")
        asset_contract["url"] = (
            f"{repository_url}/releases/download/{args.tag}/{asset_name}"
        )

    manifest["publication_status"] = (
        "published" if args.settle else "deployed_candidate"
    )
    canary_authority = manifest.get("compatibility_canary")
    if isinstance(canary_authority, dict):
        canary_authority["status"] = "superseded_by_production_deploy"
        canary_authority["production_apply_authorized"] = False
        canary_authority["system_install_authorized"] = False
        canary_authority["service_mutation_authorized"] = False
        canary_authority["automatic_apply_authorized"] = False
    manifest.setdefault("gates", {})["release_success"] = True
    manifest["gates"]["deploy_success"] = True
    manifest["gates"]["release_run_url"] = release_run_url
    manifest["gates"]["deploy_run_url"] = args.run_url
    if args.settle:
        manifest["gates"]["ota_success"] = True
    else:
        manifest["gates"].pop("ota_success", None)
    args.manifest.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    args.manifest_signature.write_bytes(private_key.sign(args.manifest.read_bytes()))

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
        "distribution_parity_sha256": sha256(args.distribution_parity),
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
