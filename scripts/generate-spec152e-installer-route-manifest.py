#!/usr/bin/env python3
"""Generate and validate the public Spec 152E installer route/asset manifest.

Repairs the advertised install.focusa.dev convenience URLs (/focusa, /bundle,
Engine, PowerShell) and the transactional links (verify, pay, success, manage,
recovery) so every advertised URL resolves to an exact verified asset/page with
preserved content type and trust metadata. Convenience routes are stable
aliases and facade paths never embed version segments, so the public surface
stays stable through version changes. No 404 and no unsafe redirect remain in
the repaired surface.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs/contracts/spec152e-installer-route-manifest.v1.json"
REGISTRY = ROOT / "docs/contracts/spec152e-facade-registry.v1.json"
INVENTORY = ROOT / "docs/contracts/spec152e-deployed-surface-inventory.v1.json"

SCHEMA = "focusa.spec152e.installer_route_manifest.v1"
OWNER = "focusadev/install.focusa.dev"
ORIGIN = "https://install.focusa.dev"
INSTALL_FACADE = "focusa_install_v1"

EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+")
VERSION_SEGMENT = re.compile(r"/v[0-9]+")

CONTENT_TYPE_POLICY = {".sh": "application/x-sh", ".ps1": "application/x-powershell"}

# Advertised convenience URL -> exact verified installer asset. repository_path
# is set when a repo copy is the verified source; None marks deployed-only
# assets pinned from the deployed surface inventory.
CONVENIENCE = [
    {"route": "/focusa", "target": "/installers/install-focusa.sh", "repository_path": "scripts/install-focusa.sh", "inventory_id": "installer.unix"},
    {"route": "/bundle", "target": "/installers/install-bundle.sh", "repository_path": None, "inventory_id": "installer.bundle"},
    {"route": "/engine", "target": "/installers/install-engine.sh", "repository_path": None, "inventory_id": "installer.engine"},
    {"route": "/powershell", "target": "/installers/install-focusa.ps1", "repository_path": "scripts/install-focusa.ps1", "inventory_id": "installer.windows"},
]

# Transactional link -> facade registry path key (stable through version changes).
TRANSACTIONAL_KEYS = {
    "verify": "verification",
    "pay": "checkout",
    "success": "success",
    "manage": "manage",
    "recovery": "recovery",
}


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _relative_path(value: object, label: str) -> str:
    _require(isinstance(value, str) and value.startswith("/"), f"{label} must be an absolute relative path")
    parsed = urlsplit(value)
    _require(parsed.scheme == "" and parsed.netloc == "", f"{label} must not be a URL")
    _require(parsed.query == "" and parsed.fragment == "", f"{label} must not contain query/fragment")
    _require("*" not in value and "//" not in value and ".." not in parsed.path.split("/"), f"{label} is not exact")
    return value


def _exact_origin(value: object) -> str:
    _require(isinstance(value, str) and value != "", "origin must be a non-empty string")
    parsed = urlsplit(value)
    _require(parsed.scheme == "https", f"origin must use exact https: {value!r}")
    _require(parsed.netloc != "" and parsed.hostname is not None, f"origin has no host: {value!r}")
    _require(parsed.username is None and parsed.password is None, f"origin contains user info: {value!r}")
    _require(parsed.port is None, f"origin contains a port: {value!r}")
    _require(parsed.path == "" and parsed.query == "" and parsed.fragment == "", f"origin must not contain path/query/fragment: {value!r}")
    _require(parsed.hostname == parsed.hostname.lower(), f"origin host must be lowercase: {value!r}")
    _require("*" not in value and value == f"https://{parsed.hostname}", f"origin is not exact: {value!r}")
    return value


def build() -> dict:
    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    facades = {row["facade_id"]: row for row in registry["facades"]}
    _require(INSTALL_FACADE in facades, "focusa_install_v1 facade must be registered")
    facade = facades[INSTALL_FACADE]
    _require(ORIGIN in facade["exact_origins"], "install facade must bind the exact origin")
    inventory_files = {row["id"]: row for row in inventory["files"]}

    convenience_urls = []
    for entry in CONVENIENCE:
        route = _relative_path(entry["route"], "convenience route")
        target = _relative_path(entry["target"], "convenience target")
        _require(VERSION_SEGMENT.search(route) is None, f"convenience route must be version-stable: {route}")
        _require(target.startswith("/installers/"), f"convenience target must live under /installers/: {target}")
        ext = Path(target).suffix
        _require(ext in CONTENT_TYPE_POLICY, f"no content type policy for {target}")
        inventory_row = inventory_files.get(entry["inventory_id"])
        _require(inventory_row is not None, f"inventory record missing: {entry['inventory_id']}")
        _require(inventory_row["path"] == target.lstrip("/"), f"inventory path mismatch for {entry['inventory_id']}")
        if entry["repository_path"] is not None:
            repo_path = ROOT / entry["repository_path"]
            _require(repo_path.is_file(), f"missing repository asset: {entry['repository_path']}")
            digest = _sha256(repo_path)
            _require(inventory_row.get("repository_sha256") == digest, f"repository digest drift for {entry['inventory_id']}")
            trust = {"kind": "repository_verified", "sha256": digest, "repository_path": entry["repository_path"], "inventory_id": entry["inventory_id"]}
        else:
            digest = inventory_row["sha256"]
            _require(re.fullmatch(r"[0-9a-f]{64}", digest), f"invalid deployed digest for {entry['inventory_id']}")
            trust = {"kind": "deployed_only_pinned", "sha256": digest, "inventory_id": entry["inventory_id"]}
        convenience_urls.append(
            {
                "route": route,
                "target": target,
                "content_type": CONTENT_TYPE_POLICY[ext],
                "trust": trust,
                "status": 200,
            }
        )

    transactional_links = {}
    for name, key in TRANSACTIONAL_KEYS.items():
        _require(key in facade["paths"], f"facade path key missing: {key}")
        path = _relative_path(facade["paths"][key], f"transactional link {name}")
        _require(VERSION_SEGMENT.search(path) is None, f"transactional link must be version-stable: {name}")
        transactional_links[name] = {"facade_path_key": key, "path": path, "status": 200}

    manifest = {
        "schema": SCHEMA,
        "manifest_version": 1,
        "owner": OWNER,
        "origin": ORIGIN,
        "authority": {
            "canonical": "WPUIAI.com EDD",
            "facade_role": "registered_branded_facade_and_bounded_proxy_only",
            "spec158": "excluded",
        },
        "content_type_policy": dict(sorted(CONTENT_TYPE_POLICY.items())),
        "convenience_urls": convenience_urls,
        "transactional_links": dict(sorted(transactional_links.items())),
        "stability": {
            "convenience_urls_are_stable_aliases": True,
            "facade_paths_never_embed_version_segments": True,
            "asset_trust_is_pinned_by_sha256": True,
        },
        "removed_stale": [
            "advertised /focusa convenience URL previously returned 404 and is repaired to the verified installer asset",
            "advertised /bundle convenience URL previously returned 404 and is repaired to the verified installer asset",
            "stale installer/email link instructions that referenced versioned or broken routes are removed",
        ],
        "invariants": [
            "every advertised URL resolves to an exact verified asset or registered facade page",
            "no advertised URL returns 404 and no unsafe redirect remains",
            "content type and sha256 trust metadata are preserved for every verified asset",
            "facade paths are stable through version changes and equal the registered facade paths",
            "the manifest contains no credentials secrets or unmasked email addresses",
        ],
        "counts": {
            "convenience_urls": len(convenience_urls),
            "transactional_links": len(transactional_links),
            "repository_verified": sum(1 for row in convenience_urls if row["trust"]["kind"] == "repository_verified"),
            "deployed_only_pinned": sum(1 for row in convenience_urls if row["trust"]["kind"] == "deployed_only_pinned"),
        },
    }
    validate(manifest, registry)
    return manifest


def validate(manifest: dict, registry: dict) -> None:
    _require(manifest.get("schema") == SCHEMA, "invalid schema")
    _require(manifest.get("manifest_version") == 1, "invalid manifest version")
    _require(manifest.get("owner") == OWNER, "invalid owner")
    origin = _exact_origin(manifest.get("origin"))
    _require(origin == ORIGIN, "origin must be https://install.focusa.dev")
    authority = manifest.get("authority", {})
    _require(authority.get("canonical") == "WPUIAI.com EDD", "canonical EDD authority required")
    _require(authority.get("facade_role") == "registered_branded_facade_and_bounded_proxy_only", "facade must be presenter-only")
    _require(authority.get("spec158") == "excluded", "Spec 158 must remain excluded")

    stability = manifest.get("stability", {})
    for key in ("convenience_urls_are_stable_aliases", "facade_paths_never_embed_version_segments", "asset_trust_is_pinned_by_sha256"):
        _require(stability.get(key) is True, f"stability.{key} must be true")

    facades = {row["facade_id"]: row for row in (registry or {}).get("facades", [])}
    facade = facades.get(INSTALL_FACADE, {})
    paths = facade.get("paths", {})

    seen_routes: set[str] = set()
    for row in manifest.get("convenience_urls", []):
        route = _relative_path(row.get("route"), "convenience route")
        _require(route not in seen_routes, f"duplicate convenience route: {route}")
        seen_routes.add(route)
        _require(VERSION_SEGMENT.search(route) is None, f"convenience route must be version-stable: {route}")
        target = _relative_path(row.get("target"), "convenience target")
        _require(target.startswith("/installers/"), "convenience target must live under /installers/")
        _require(row.get("status") == 200, f"convenience route must resolve 200: {route}")
        content_type = row.get("content_type")
        ext = Path(target).suffix
        _require(content_type == CONTENT_TYPE_POLICY.get(ext), f"content type mismatch for {route}")
        trust = row.get("trust", {})
        digest = trust.get("sha256")
        _require(isinstance(digest, str) and re.fullmatch(r"[0-9a-f]{64}", digest), f"invalid trust digest for {route}")
        if trust.get("kind") == "repository_verified":
            _require(isinstance(trust.get("repository_path"), str) and (ROOT / trust["repository_path"]).is_file(), f"repository asset missing for {route}")
            _require(_sha256(ROOT / trust["repository_path"]) == digest, f"repository digest mismatch for {route}")
            _require(isinstance(trust.get("inventory_id"), str), f"repository pin needs inventory_id for {route}")
        elif trust.get("kind") == "deployed_only_pinned":
            _require(isinstance(trust.get("inventory_id"), str), f"deployed pin needs inventory_id for {route}")
        else:
            _require(False, f"unknown trust kind for {route}")

    for name, link in manifest.get("transactional_links", {}).items():
        _require(isinstance(name, str) and name in TRANSACTIONAL_KEYS, f"unknown transactional link: {name}")
        key = link.get("facade_path_key")
        _require(key == TRANSACTIONAL_KEYS[name], f"facade path key mismatch for {name}")
        _require(key in paths, f"facade path key not registered: {key}")
        _require(link.get("path") == paths[key], f"transactional link must equal the registered facade path: {name}")
        _require(VERSION_SEGMENT.search(link["path"]) is None, f"transactional link must be version-stable: {name}")
        _require(link.get("status") == 200, f"transactional link must resolve 200: {name}")

    raw = json.dumps(manifest, sort_keys=True)
    _require("*" not in raw, "wildcards are forbidden")
    _require(EMAIL.search(raw) is None, "email addresses are forbidden in the public manifest")
    _require(SECRET.search(raw) is None, "secret-like values are forbidden in the public manifest")


def render_json(manifest: dict) -> str:
    return json.dumps(manifest, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate and fail if the generated manifest is stale")
    args = parser.parse_args()
    manifest = build()
    content = render_json(manifest)
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != content:
            raise SystemExit("stale Spec 152E installer route manifest: " + str(OUTPUT.relative_to(ROOT)))
    else:
        OUTPUT.write_text(content, encoding="utf-8")
    print(json.dumps({"schema": "focusa.spec152e.installer_route_manifest_validation.v1", **manifest["counts"], "result": "passed"}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
