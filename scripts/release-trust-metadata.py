#!/usr/bin/env python3
"""Build signed Focusa OTA release metadata from a completed asset directory."""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import pathlib
import re
import shutil
import stat
import sys
from typing import Any

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

METADATA_NAMES = {
    "SHA256SUMS.txt",
    "SHA256SUMS.txt.sig",
    "release-manifest.json",
    "release-manifest.json.sig",
    "release-provenance.json",
    "release-provenance.json.sig",
    "focusa-trusted-release-keys.json",
    "focusa-trusted-release-keys.json.sig",
}


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: pathlib.Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sign_and_verify(
    path: pathlib.Path,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
) -> pathlib.Path:
    signature = path.with_name(path.name + ".sig")
    payload = path.read_bytes()
    signature.write_bytes(private_key.sign(payload))
    public_key.verify(signature.read_bytes(), payload)
    if signature.stat().st_size != 64:
        raise ValueError(f"invalid Ed25519 signature length for {path.name}")
    return signature


def platform_for(name: str) -> str:
    mappings = (
        ("aarch64-apple-darwin", "macos-aarch64"),
        ("x86_64-apple-darwin", "macos-x86_64"),
        ("aarch64-pc-windows-msvc", "windows-aarch64"),
        ("x86_64-pc-windows-msvc", "windows-x86_64"),
        ("x86_64-unknown-linux-musl", "linux-x86_64-musl"),
        ("x86_64-unknown-linux-gnu", "linux-x86_64-gnu"),
    )
    for marker, platform in mappings:
        if marker in name:
            return platform
    if name.endswith(".dmg") or ".app." in name:
        return "macos"
    return "all"


def channel_for(tag: str) -> str:
    lowered = tag.lower()
    if "preview" in lowered or "rc" in lowered:
        return "preview"
    if "dev" in lowered or "nightly" in lowered:
        return "nightly"
    return "stable"


def active_key(metadata: dict[str, Any]) -> dict[str, Any]:
    keys = metadata.get("keys")
    if not isinstance(keys, list):
        raise ValueError("trusted key metadata requires a keys array")
    active = [key for key in keys if key.get("revoked_at") is None]
    if len(active) != 1:
        raise ValueError("exactly one active release signing key is required")
    key = active[0]
    required = {
        "key_id",
        "signing_algorithm",
        "public_key_fingerprint",
        "public_key_base64",
    }
    if not required.issubset(key):
        raise ValueError("active release key metadata is incomplete")
    if key["signing_algorithm"] != "ed25519":
        raise ValueError("active release key must use ed25519")
    raw = base64.b64decode(key["public_key_base64"], validate=True)
    if (
        len(raw) != 32
        or hashlib.sha256(raw).hexdigest() != key["public_key_fingerprint"]
    ):
        raise ValueError("trusted public key fingerprint mismatch")
    return key


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", required=True, type=pathlib.Path)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--run-url", required=True)
    parser.add_argument("--workflow", default=".github/workflows/release.yml")
    parser.add_argument("--candidate", action="store_true")
    parser.add_argument(
        "--compatibility-from-tag",
        help="authorize only an isolated preproduction canary from this exact prior stable tag",
    )
    parser.add_argument("--private-key", required=True, type=pathlib.Path)
    parser.add_argument("--trusted-keys", required=True, type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.dist.is_dir():
        raise ValueError(f"asset directory not found: {args.dist}")
    if args.compatibility_from_tag:
        if not args.candidate:
            raise ValueError("compatibility canary authority requires --candidate")
        if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.compatibility_from_tag):
            raise ValueError("compatibility canary prior tag must be exact stable SemVer")
        if args.compatibility_from_tag == args.tag:
            raise ValueError("compatibility canary prior and candidate tags must differ")
    key_mode = stat.S_IMODE(args.private_key.stat().st_mode)
    if key_mode & 0o077:
        raise ValueError("private signing key permissions must be 0600 or stricter")

    trusted_metadata = json.loads(args.trusted_keys.read_text())
    key = active_key(trusted_metadata)
    trusted_output = args.dist / "focusa-trusted-release-keys.json"
    shutil.copyfile(args.trusted_keys, trusted_output)

    loaded_private_key = serialization.load_pem_private_key(
        args.private_key.read_bytes(), password=None
    )
    if not isinstance(loaded_private_key, Ed25519PrivateKey):
        raise ValueError("release signing key is not Ed25519")
    public_key = loaded_private_key.public_key()
    public_raw = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    if base64.b64encode(public_raw).decode("ascii") != key["public_key_base64"]:
        raise ValueError(
            "private signing key does not match trusted public key metadata"
        )
    try:
        assets = sorted(
            path
            for path in args.dist.iterdir()
            if path.is_file()
            and path.name not in METADATA_NAMES
            and not path.name.endswith(".sig")
            and not path.name.startswith(".")
        )
        if not assets:
            raise ValueError("no release assets found")

        checksums = args.dist / "SHA256SUMS.txt"
        checksums.write_text(
            "".join(f"{sha256(path)}  {path.name}\n" for path in assets)
        )

        manifest_assets: dict[str, Any] = {}
        subjects = []
        for asset in assets:
            signature = sign_and_verify(asset, loaded_private_key, public_key)
            digest = sha256(asset)
            manifest_assets[asset.name] = {
                "platform": platform_for(asset.name),
                "name": asset.name,
                "sha256": digest,
                "size_bytes": asset.stat().st_size,
                "url": (
                    f"{args.run_url}#artifact-{asset.name}"
                    if args.candidate
                    else f"https://github.com/{args.repo}/releases/download/{args.tag}/{asset.name}"
                ),
                "signature": {
                    "algorithm": "ed25519",
                    "key_id": key["key_id"],
                    "signature": base64.b64encode(signature.read_bytes()).decode(
                        "ascii"
                    ),
                    "certificate_sha256": None,
                },
            }
            subjects.append(
                {
                    "name": asset.name,
                    "sha256": digest,
                    "size_bytes": asset.stat().st_size,
                }
            )

        published_at = (
            dt.datetime.now(dt.timezone.utc)
            .replace(microsecond=0)
            .isoformat()
            .replace("+00:00", "Z")
        )
        provenance = {
            "schema": "focusa.release_provenance.v1",
            "tag": args.tag,
            "commit": args.commit,
            "builder": "github-actions",
            "workflow": args.workflow,
            "run_url": args.run_url,
            "artifact_digest": sha256(checksums),
            "subjects": subjects,
            "generated_at": published_at,
            "slsa_attestation": None,
        }
        provenance_path = args.dist / "release-provenance.json"
        write_json(provenance_path, provenance)

        compatibility_canary = None
        if args.compatibility_from_tag:
            compatibility_canary = {
                "schema": "focusa.compatibility_canary_authorization.v1",
                "status": "authorized",
                "environment": "isolated_preproduction",
                "allowed_install_scope": "non_root_ephemeral_home",
                "required_previous_tag": args.compatibility_from_tag,
                "required_sequence": [
                    "prior_release",
                    "candidate_manifest_bound_apply",
                    "prior_release_full_rollback",
                    "candidate_manifest_bound_reapply",
                ],
                "production_apply_authorized": False,
                "system_install_authorized": False,
                "service_mutation_authorized": False,
                "automatic_apply_authorized": False,
            }

        manifest = {
            "schema": "focusa.release_manifest.v1",
            "tag": args.tag,
            "commit": args.commit,
            "channel": channel_for(args.tag),
            "published_at": published_at,
            "publication_status": "candidate_only" if args.candidate else "published",
            "yanked": False,
            "revoked": False,
            "superseded_by": None,
            "gates": {
                "ci_success": True,
                "release_success": not args.candidate,
                "deploy_success": None,
                "smoke_success": True,
                "installer_proof_success": True,
                "ci_run_url": args.run_url,
                "release_run_url": None if args.candidate else args.run_url,
                "deploy_run_url": None,
            },
            "compatibility_canary": compatibility_canary,
            "trust": {
                "signing_algorithm": key["signing_algorithm"],
                "key_id": key["key_id"],
                "public_key_fingerprint": key["public_key_fingerprint"],
                "valid_from": key.get("valid_from"),
                "valid_until": key.get("valid_until"),
                "revoked_at": key.get("revoked_at"),
            },
            "provenance": {
                "builder": provenance["builder"],
                "workflow": provenance["workflow"],
                "run_url": provenance["run_url"],
                "artifact_digest": provenance["artifact_digest"],
                "slsa_attestation": provenance["slsa_attestation"],
            },
            "compatibility": {
                "min_installed_version": "0.9.94-dev",
                "daemon_api_contract": "v1",
                "pi_tool_contract": "focusa.tool_result.v1",
                "data_schema": "backward-compatible",
                "requires_migration": False,
                "downgrade_supported": False,
                "requires_restart": ["focusa-daemon", "focusa-tui", "focusa-menubar"],
                "incompatible_if_features_missing": [],
            },
            "assets": manifest_assets,
            "requires_license_features": [],
            "dev_mode_features": ["nightly_updates"],
            "rollback_supported": True,
        }
        manifest_path = args.dist / "release-manifest.json"
        write_json(manifest_path, manifest)

        for metadata_path in (
            checksums,
            provenance_path,
            manifest_path,
            trusted_output,
        ):
            sign_and_verify(metadata_path, loaded_private_key, public_key)

        result = {
            "status": "completed",
            "tag": args.tag,
            "key_id": key["key_id"],
            "asset_count": len(assets),
            "signed_file_count": len(assets) + 4,
            "manifest": str(manifest_path),
            "provenance": str(provenance_path),
            "checksums": str(checksums),
            "trusted_keys": str(trusted_output),
        }
        print(json.dumps(result, sort_keys=True))
        return 0
    finally:
        pass


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"release trust metadata failed: {error}", file=sys.stderr)
        raise SystemExit(1)
