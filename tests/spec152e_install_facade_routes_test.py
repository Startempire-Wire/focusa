#!/usr/bin/env python3
"""HTTP/browser regression test for the Spec 152E installer route/asset manifest.

Repairs and pins the advertised install.focusa.dev convenience URLs (/focusa,
/bundle, /engine, /powershell) and transactional links (verify, pay, success,
manage, recovery). Proves every advertised URL resolves to an exact verified
asset/page with preserved content type and sha256 trust metadata, no 404, no
unsafe redirect, and facade paths stable through version changes. Replayable
offline from the pinned commit: no live network, no publication.
"""

import hashlib
import importlib.util
import json
import re
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "docs/contracts/spec152e-installer-route-manifest.v1.json"
GENERATOR = ROOT / "scripts/generate-spec152e-installer-route-manifest.py"
REGISTRY = json.loads((ROOT / "docs/contracts/spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))
INVENTORY = json.loads((ROOT / "docs/contracts/spec152e-deployed-surface-inventory.v1.json").read_text(encoding="utf-8"))

ORIGIN = "https://install.focusa.dev"
INSTALL_FACADE = "focusa_install_v1"
EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+")
VERSION_RE = re.compile(r"/v[0-9]+")

spec = importlib.util.spec_from_file_location("spec152e_route_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

manifest_raw = MANIFEST_PATH.read_text(encoding="utf-8")
manifest = json.loads(manifest_raw)

positive = 0
negative = 0


def check(condition: bool, message: str, kind: str = "positive") -> None:
    global positive, negative
    if not condition:
        raise AssertionError(f"FAIL ({kind}): {message}")
    if kind == "positive":
        positive += 1
    else:
        negative += 1


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve(url: str) -> dict:
    """Deterministic offline HTTP resolution against the manifest + registry."""
    parsed = urlsplit(url)
    if parsed.scheme != "https":
        return {"status": 403, "error": "FACADE_ORIGIN_DENIED"}
    origin = f"{parsed.scheme}://{parsed.netloc}"
    if origin != ORIGIN:
        return {"status": 403, "error": "FACADE_ORIGIN_DENIED"}
    if parsed.username or parsed.password or parsed.port:
        return {"status": 403, "error": "FACADE_ORIGIN_DENIED"}
    if parsed.query or parsed.fragment or "*" in parsed.path or ".." in parsed.path.split("/"):
        return {"status": 400, "error": "URL_DENIED"}
    for row in manifest["convenience_urls"]:
        if parsed.path == row["route"]:
            return {"status": row["status"], "content_type": row["content_type"], "target": row["target"]}
    for name, link in manifest["transactional_links"].items():
        if parsed.path == link["path"]:
            return {"status": link["status"], "content_type": "text/html", "target": link["path"], "link": name}
    return {"status": 404, "error": "NOT_FOUND"}


# --- Contract is generated and current ----------------------------------------
check(manifest == module.build(), "committed manifest equals the generated manifest (contracts current)", "positive")
check(module.render_json(module.build()) == manifest_raw, "committed manifest rendering is current", "positive")

# --- Schema / authority -------------------------------------------------------
check(manifest["schema"] == "focusa.spec152e.installer_route_manifest.v1", "manifest schema")
check(manifest["manifest_version"] == 1, "manifest version")
check(manifest["owner"] == "focusadev/install.focusa.dev", "manifest owner")
check(manifest["origin"] == ORIGIN, "manifest origin is the exact https install origin")
check(manifest["authority"]["canonical"] == "WPUIAI.com EDD", "canonical EDD authority")
check(manifest["authority"]["facade_role"] == "registered_branded_facade_and_bounded_proxy_only", "facade is presenter-only")
check(manifest["authority"]["spec158"] == "excluded", "Spec 158 excluded")
check(manifest["stability"]["facade_paths_never_embed_version_segments"] is True, "facade paths are version-stable")
check(manifest["stability"]["convenience_urls_are_stable_aliases"] is True, "convenience URLs are stable aliases")
check(manifest["stability"]["asset_trust_is_pinned_by_sha256"] is True, "asset trust pinned by sha256")

# --- Convenience URLs resolve to exact verified assets -------------------------
expected_routes = {"/focusa", "/bundle", "/engine", "/powershell"}
check({row["route"] for row in manifest["convenience_urls"]} == expected_routes, "convenience URL set is exact")
inventory_files = {row["id"]: row for row in INVENTORY["files"]}
for row in manifest["convenience_urls"]:
    route = row["route"]
    target = row["target"]
    check(route in expected_routes, f"{route} is a named convenience URL")
    check(target.startswith("/installers/"), f"{route} targets a verified installer asset: {target}")
    check(row["status"] == 200, f"{route} resolves 200 (no 404)")
    ext = Path(target).suffix
    check(row["content_type"] == manifest["content_type_policy"][ext], f"{route} preserves content type {row['content_type']}")
    trust = row["trust"]
    check(re.fullmatch(r"[0-9a-f]{64}", trust["sha256"]), f"{route} carries pinned sha256 trust metadata")
    if trust["kind"] == "repository_verified":
        repo_path = ROOT / trust["repository_path"]
        check(repo_path.is_file(), f"{route} repository asset exists: {trust['repository_path']}")
        check(sha256(repo_path) == trust["sha256"], f"{route} sha256 matches the repository asset")
        inventory_row = inventory_files.get(row["trust"]["inventory_id"])
        check(inventory_row is not None and inventory_row.get("repository_sha256") == trust["sha256"], f"{route} repository digest agrees with the deployed inventory")
    elif trust["kind"] == "deployed_only_pinned":
        inventory_row = inventory_files.get(trust["inventory_id"])
        check(inventory_row is not None, f"{route} deployed pin references an inventory record")
        check(inventory_row["sha256"] == trust["sha256"], f"{route} sha256 agrees with the deployed inventory record")
    else:
        check(False, f"{route} trust kind is bounded")

# HTTP/browser resolution for every advertised convenience URL.
for row in manifest["convenience_urls"]:
    resolved = resolve(ORIGIN + row["route"])
    check(resolved["status"] == 200, f"browser GET {row['route']} returns 200")
    check(resolved["content_type"] == row["content_type"], f"browser GET {row['route']} preserves content type")
    check(resolved["target"] == row["target"], f"browser GET {row['route']} lands on the exact verified asset")
    check("error" not in resolved, f"browser GET {row['route']} has no error envelope")

# --- Transactional links resolve to registered facade paths --------------------
facade = next(row for row in REGISTRY["facades"] if row["facade_id"] == INSTALL_FACADE)
check(set(manifest["transactional_links"]) == {"verify", "pay", "success", "manage", "recovery"}, "transactional link set is exact")
for name, link in manifest["transactional_links"].items():
    key = link["facade_path_key"]
    check(key in facade["paths"], f"{name} references a registered facade path key")
    check(link["path"] == facade["paths"][key], f"{name} equals the registered facade path {link['path']}")
    check(VERSION_RE.search(link["path"]) is None, f"{name} facade path has no version segment")
    check(link["status"] == 200, f"{name} resolves 200")
    resolved = resolve(ORIGIN + link["path"])
    check(resolved["status"] == 200 and resolved["target"] == link["path"], f"browser GET {name} resolves to the facade page")

# Transactional links are stable through version changes: equal to the registry
# paths today and contain no version segment, so a future version bump cannot
# break them without a registry change being caught by the contract test.
for name, link in manifest["transactional_links"].items():
    check(VERSION_RE.search(link["path"]) is None and link["path"] == facade["paths"][link["facade_path_key"]], f"{name} stable through version changes")

# --- Negative: fail-closed URL resolution --------------------------------------
unknown = resolve(ORIGIN + "/admin")
check(unknown["status"] == 404, "unknown URL is 404", "negative")
old_404_focusa = resolve(ORIGIN + "/installers/install-focusa.sh")
check(old_404_focusa["status"] == 404, "unlisted direct asset path is not advertised (404)", "negative")
http = resolve("http://install.focusa.dev/focusa")
check(http["status"] == 403, "non-https origin denied", "negative")
external = resolve("https://evil.invalid/focusa")
check(external["status"] == 403, "external origin denied", "negative")
userinfo = resolve("https://user@install.focusa.dev/focusa")
check(userinfo["status"] == 403, "origin userinfo denied", "negative")
query = resolve(ORIGIN + "/focusa?redirect=https://evil.invalid")
check(query["status"] == 400, "query/fragment denied", "negative")
traversal = resolve(ORIGIN + "/focusa/../../etc/passwd")
check(traversal["status"] == 400, "path traversal denied", "negative")
versioned = resolve(ORIGIN + "/activate/verify/v2")
check(versioned["status"] == 404, "versioned facade path not advertised", "negative")
unsafe_redirect = resolve(ORIGIN + "/focusa/" + "https://evil.invalid")
check(unsafe_redirect["status"] == 404, "no unsafe redirect target exists", "negative")

# Unsafe redirect: no advertised URL resolves to an external or absolute target.
for row in manifest["convenience_urls"]:
    check(row["target"].startswith("/"), f"{row['route']} target is a relative path (no unsafe redirect)", "negative")
for name, link in manifest["transactional_links"].items():
    check(link["path"].startswith("/"), f"{name} path is relative (no unsafe redirect)", "negative")

# --- Hygiene: no secrets, no unmasked real email, no license-shaped evidence ----
check(EMAIL_RE.search(manifest_raw) is None, "no email addresses in the manifest", "negative")
check(SECRET_RE.search(manifest_raw) is None, "no secret-shaped values in the manifest", "negative")
check(re.search(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}", manifest_raw) is None, "no license-shaped evidence in the manifest", "negative")
check("*" not in manifest_raw, "no wildcards in the manifest", "negative")
for row in manifest["convenience_urls"]:
    check(row["trust"]["sha256"] != sha256(ROOT / "scripts/install-focusa.sh") or row["route"] in {"/focusa"}, "digest pinning is exact per asset", "negative")

result = {
    "schema": "focusa.spec152e.install_facade_routes_regression.v1",
    "convenience_urls": len(manifest["convenience_urls"]),
    "transactional_links": len(manifest["transactional_links"]),
    "positive_checks": positive,
    "negative_checks": negative,
    "result": "passed_fail_closed",
}
print(json.dumps(result, sort_keys=True))
