#!/usr/bin/env python3
"""Spec 152E multi-domain facade acceptance matrix (build-independent).

Exercises every registered facade domain and its exact origin, the allowed and
denied product bindings, verification/checkout/success/cancel/manage/recovery
links, checkout return, polling, recovery, spoofing, timeout, and upstream
authority outage behavior — entirely offline and replayable from the pinned
commit. No cargo build, no live network, no publication. The matrix is driven
by the generated facade registry plus the activation call stack, error
registry, public OpenAPI, golden vectors, installer route manifest, deployed
surface inventory, the browser security/registration components, and the
executable PHP facade contracts probed through the CLI.
"""

import base64
import hashlib
import hmac
import json
import re
import subprocess
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"

REGISTRY = json.loads((CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))
PRODUCTS = json.loads((CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8"))
CALL_STACK = yaml.safe_load((CONTRACTS / "spec152e-activation-call-stack.v1.yaml").read_text(encoding="utf-8"))
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
ERRORS = json.loads((CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"))
OPENAPI = json.loads((CONTRACTS / "spec152e-activation-public-openapi.v1.json").read_text(encoding="utf-8"))
GOLDEN = json.loads((CONTRACTS / "spec152e-facade-golden-vectors.v1.json").read_text(encoding="utf-8"))
MANIFEST = json.loads((CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8"))
INVENTORY = json.loads((CONTRACTS / "spec152e-deployed-surface-inventory.v1.json").read_text(encoding="utf-8"))

REGISTRY_JSON = (CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8")
REGISTRY_YAML = (CONTRACTS / "spec152e-facade-registry.v1.yaml").read_text(encoding="utf-8")
REGISTRY_PHP = (CONTRACTS / "spec152e-facade-registry.v1.php").read_text(encoding="utf-8")
PROTOCOL_PHP = (CONTRACTS / "spec152e-facade-protocol.v1.php").read_text(encoding="utf-8")
SECURITY_PHP = (CONTRACTS / "spec152e-facade-security.v1.php").read_text(encoding="utf-8")
INSTALL_PHP = (CONTRACTS / "spec152e-install-facade-routes.v1.php").read_text(encoding="utf-8")
BROWSER_SOURCE = (ROOT / "public/activation/focusa-facade-security.mjs").read_text(encoding="utf-8")
REGISTRATION_SOURCE = (ROOT / "public/activation/focusa-registration.mjs").read_text(encoding="utf-8")
PAGE_HTML = (ROOT / "public/activation/page.html").read_text(encoding="utf-8")

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
LIVE_RE = re.compile(r"(?i)focusa_live_[0-9]+_[0-9a-f]+")
VERSION_RE = re.compile(r"/v[0-9]+")

SIGNED_FIELDS = [
    "schema", "credential_id", "timestamp", "nonce", "request_id",
    "idempotency_key", "registration_id", "facade_id", "origin",
    "product_code", "action", "redirect_handle", "continuation_token",
    "body_sha256",
]

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


def facade_by_id(facade_id: str) -> dict:
    matches = [row for row in REGISTRY["facades"] if row["facade_id"] == facade_id]
    assert len(matches) == 1, facade_id
    return matches[0]


def valid_request(overrides=None) -> dict:
    request = {
        "facade_id": "focusa_install_v1",
        "origin": "https://install.focusa.dev",
        "product_code": "focusa_operator_lifetime_v1",
        "route": "activation_start",
        "callback_handle": "success",
        "locale": "en-US",
        "timestamp": "2026-08-07T00:00:00Z",
        "request_id": "req_synthetic_acceptance_01",
        "idempotency_key": "idem_synthetic_acceptance_01",
    }
    if overrides:
        request.update(overrides)
    return request


def deny(request: dict) -> str:
    """Contract-driven fail-closed resolver using the registry request contract."""
    for field in REGISTRY["request_contract"]["forbidden"]:
        if field in request:
            return "FACADE_REQUEST_FIELD_DENIED"
    for field in REGISTRY["request_contract"]["required"]:
        if field not in request or not isinstance(request[field], str) or request[field] == "":
            return "FACADE_REQUEST_INVALID"
    matches = [row for row in REGISTRY["facades"] if row["facade_id"] == request["facade_id"]]
    if len(matches) != 1:
        return REGISTRY["request_contract"]["unknown_facade"]
    facade = matches[0]
    if request["origin"] not in facade["exact_origins"]:
        return REGISTRY["request_contract"]["unknown_origin"]
    if request["product_code"] not in facade["products"]:
        return REGISTRY["request_contract"]["unknown_product"]
    if request["route"] not in REGISTRY["proxy_routes"]:
        return REGISTRY["request_contract"]["unknown_route"]
    if request["callback_handle"] not in facade["callbacks"]:
        return REGISTRY["request_contract"]["unknown_callback"]
    if request["locale"] not in facade["locale"]["allowed"]:
        return REGISTRY["request_contract"]["unknown_locale"]
    return "ok"


def php_probe(code: str) -> str:
    proc = subprocess.run(
        ["php", "-d", "log_errors=0", "-d", "error_log=/dev/null", "-r", code],
        capture_output=True, text=True, cwd=str(ROOT),
    )
    check(proc.returncode == 0, f"php probe exited 0: {proc.stderr[:240]}")
    return proc.stdout.strip()


# --- A. All registered domains -------------------------------------------------
check(REGISTRY["schema"] == "focusa.spec152e.facade_registry.v1", "registry schema")
check(REGISTRY["registry_version"] == 1, "registry version")
check(REGISTRY["owner"] == "WPUIAI/wpuiai", "registry owner")
expected_facades = {
    "focusa_install_v1", "focusa_marketing_v1", "focusa_forge_v1",
    "focusa_arena_v1", "uiai_engine_v1", "wpuiai_public_v1",
}
check({row["facade_id"] for row in REGISTRY["facades"]} == expected_facades, "six registered facade domains")
expected_origins = {
    "https://install.focusa.dev", "https://focusa.dev", "https://forge.focusa.dev",
    "https://arena.focusa.dev", "https://engine.focusa.dev", "https://wpuiai.com",
}
check({origin for row in REGISTRY["facades"] for origin in row["exact_origins"]} == expected_origins, "six exact origins")
check(REGISTRY["counts"] == {
    "facades": 6, "exact_origins": 6, "product_bindings": 14,
    "sender_identities": 6, "callback_handles": 18, "proxy_routes": 11,
}, "registry counts are exact")
for row in REGISTRY["facades"]:
    check(row["status"] == "registered_presenter", f"{row['facade_id']} is a registered presenter")
    check(len(row["exact_origins"]) == 1 and re.fullmatch(r"https://[a-z0-9.-]+", row["exact_origins"][0]),
          f"{row['facade_id']} exact https origin")
    check("bounded_authority_proxy" in row["presenter_capabilities"], f"{row['facade_id']} is a bounded proxy")
    check(row["locale"]["default"] == "en-US" and row["locale"]["allowed"] == ["en-US"],
          f"{row['facade_id']} locale allowlist")
    check("@" not in row["sender"]["identity"] and row["sender"]["display_name"] != "",
          f"{row['facade_id']} sender identity is registry-owned")
    check(set(row["callbacks"]) == {"success", "cancel", "recovery"}, f"{row['facade_id']} callback handles")
    check(set(row["paths"]) == {"verification", "checkout", "success", "cancel", "manage", "recovery"},
          f"{row['facade_id']} facade paths")
check(len({row["sender"]["identity"] for row in REGISTRY["facades"]}) == 6, "sender identities unique per domain")
check(len(REGISTRY["proxy_routes"]) == 11 and all(path.startswith("/v1/") for path in REGISTRY["proxy_routes"].values()),
      "eleven authority proxy routes")
check("authority_issue" not in REGISTRY["proxy_routes"], "no issuance route in any facade")
check(REGISTRY["authority"] == {
    "canonical": "WPUIAI.com EDD", "facade_role": "presenter_and_bounded_proxy_only",
    "entitlement_issuance": "forbidden", "customer_or_commerce_truth": "forbidden",
    "wildcard_authority": "forbidden", "spec158": "excluded",
}, "authority posture forbids facade issuance")

# --- B. Allowed and denied products --------------------------------------------
protected = {row["public_code"] for row in PRODUCTS["protected_offers"]}
check(len(protected) == 3, "three protected EDD offers")
bindings = 0
for row in REGISTRY["facades"]:
    for product in row["products"]:
        bindings += 1
        check(product in protected, f"{row['facade_id']} {product} is a registered protected offer")
        check(deny(valid_request({
            "facade_id": row["facade_id"], "origin": row["exact_origins"][0],
            "product_code": product, "route": "activation_start", "callback_handle": "success",
        })) == "ok", f"{row['facade_id']} allowed product {product} accepted")
check(bindings == REGISTRY["counts"]["product_bindings"] == 14, "fourteen product bindings all exercised")
check(protected <= {product for row in REGISTRY["facades"] for product in row["products"]},
      "every protected offer is sellable on at least one facade")
check("focusa_operator_lifetime_v1" not in facade_by_id("uiai_engine_v1")["products"],
      "engine facade never carries the base Focusa product")
denied_cross = {
    "uiai_operator_lifetime_v1": ["focusa_marketing_v1", "focusa_forge_v1", "focusa_arena_v1"],
    "focusa_operator_lifetime_v1": ["uiai_engine_v1"],
}
for product, denied_facades in denied_cross.items():
    for facade_id in denied_facades:
        row = facade_by_id(facade_id)
        check(product not in row["products"], f"{facade_id} must not carry {product}")
        check(deny(valid_request({
            "facade_id": facade_id, "origin": row["exact_origins"][0], "product_code": product,
        })) == "FACADE_PRODUCT_DENIED", f"{facade_id} denied {product}")
for row in REGISTRY["facades"]:
    check(deny(valid_request({
        "facade_id": row["facade_id"], "origin": row["exact_origins"][0],
        "product_code": "attacker_product_v1",
    })) == "FACADE_PRODUCT_DENIED", f"{row['facade_id']} denies attacker product")

# --- C. Verification, checkout, and transactional links -------------------------
install_facade = facade_by_id("focusa_install_v1")
link_roles = {
    "verify": "verification", "pay": "checkout", "success": "success",
    "manage": "manage", "recovery": "recovery",
}
for role, path_key in link_roles.items():
    entry = MANIFEST["transactional_links"][role]
    check(entry["status"] == 200, f"advertised {role} link resolves 200")
    check(entry["facade_path_key"] == path_key, f"{role} link maps facade path key {path_key}")
    check(entry["path"] == install_facade["paths"][path_key], f"{role} link equals the registered facade path")
    check(entry["path"].startswith("/") and not VERSION_RE.search(entry["path"]), f"{role} link is relative and version-stable")
for row in REGISTRY["facades"]:
    origin = row["exact_origins"][0]
    for path_key, path in row["paths"].items():
        check(path.startswith("/") and "?" not in path and "#" not in path and not VERSION_RE.search(path),
              f"{row['facade_id']} {path_key} path is a stable relative page")
        check(origin + path == "https://" + (origin + path)[8:], f"{row['facade_id']} {path_key} link is https")
    for handle, callback in row["callbacks"].items():
        check(callback.startswith("/") and "?" not in callback and "#" not in callback,
              f"{row['facade_id']} {handle} callback is a named relative path")
        check((origin + callback).startswith("https://"), f"{row['facade_id']} {handle} callback resolves https")
check(GOLDEN["expected"]["safe_redirect"] == "https://install.focusa.dev" + install_facade["callbacks"]["success"],
      "golden vector safe redirect equals the registered success callback")

# --- D. Checkout return ---------------------------------------------------------
check(REGISTRY["proxy_routes"]["activation_checkout"] == "/v1/activation/checkout", "checkout proxy route")
check("activation.checkout" in {op["operationId"] for op in OPENAPI["paths"]["/v1/activation/checkout"].values()},
      "checkout is a public OpenAPI operation")
check(REGISTRY["proxy_routes"]["activation_select_offer"] == "/v1/activation/select-offer", "select-offer proxy route")
transitions = CALL_STACK["registration_states"]["transitions"]
check("checkout_pending" in transitions["offer_selected"] and "entitlement_issued" in transitions["checkout_pending"],
      "select-offer -> checkout -> entitlement chain")
for row in REGISTRY["facades"]:
    check(row["paths"]["checkout"] == "/activate/checkout" and row["callbacks"]["success"] == "/activate/callback/success",
          f"{row['facade_id']} checkout return path")
check("'activation_checkout' => ['method' => 'POST', 'page' => '/activate/checkout', 'facade_path' => 'checkout'" in INSTALL_PHP,
      "install facade checkout page proxies to the registered facade path")

# --- E. Polling -----------------------------------------------------------------
polling = CALL_STACK["polling"]
check(polling["credential"] == "opaque_poll_credential" and polling["stored_as"] == "keyed_hash_only",
      "poll credential is opaque and hash-only at rest")
check(set(polling["binding"]) == {"registration_id", "facade_id", "action", "expiry"}, "poll credential binding")
check(polling["default_retry_after_seconds"] == 3 and polling["maximum_retry_after_seconds"] == 30,
      "poll retry-after is bounded 3..30s")
check(set(polling["terminal_states"]) == {"activated", "denied", "recovery_only"}, "poll terminal states")
rules = polling["retry_rules"]
check("EDD_ORDER_PENDING" in rules["safe_retry"] and "EDD_LICENSE_PENDING" in rules["safe_retry"]
      and "LICENSE_DELIVERY_PENDING" in rules["safe_retry"] and "AUTHORITY_UNAVAILABLE" in rules["safe_retry"],
      "safe retry rules")
check("REQUEST_IN_PROGRESS" in rules["retry_with_same_idempotency_key"], "idempotent retry rule")
check("EMAIL_VERIFICATION_EXPIRED" in rules["restart_verification"], "verification restart rule")
for code in ("FACADE_ORIGIN_DENIED", "FACADE_PRODUCT_DENIED", "PRODUCT_MAPPING_REQUIRED",
             "EVALUATION_NOT_ELIGIBLE", "LICENSE_ACCOUNT_MISMATCH", "NODE_LIMIT_EXHAUSTED"):
    check(code in rules["do_not_retry_unchanged"], f"no-unchanged-retry rule {code}")
for code in ("EDD_LICENSE_UNUSABLE", "REFUNDED", "REVOKED"):
    check(code in rules["recovery_only"], f"recovery-only poll rule {code}")
check(INTERNAL["polling"] == polling, "internal activation contract agrees with the call stack")
check(REGISTRY["proxy_routes"]["activation_poll"] == "/v1/activation/poll", "poll proxy route")
check("activation.poll" in {op["operationId"] for op in OPENAPI["paths"]["/v1/activation/poll"].values()},
      "poll is a public OpenAPI operation")
for code in ("EDD_ORDER_PENDING", "EDD_LICENSE_PENDING", "LICENSE_DELIVERY_PENDING"):
    row = next(error for error in ERRORS["errors"] if error["code"] == code)
    check(row["http_status"] == 202 and row["retryable"] is True and row["safe_next_action"] == "poll_after_retry_after",
          f"{code} is a retryable poll error")
canonical_output = INTERNAL["canonical_output"]
check("poll_credential_hash" in canonical_output["forbidden"] and "verification_hash" in canonical_output["forbidden"],
      "poll and verification hashes are never exposed")
check("masked_email" in canonical_output["optional"] and "email" in canonical_output["forbidden"],
      "presenter output is masked email only")

# --- F. Recovery ----------------------------------------------------------------
for row in REGISTRY["facades"]:
    check(row["paths"]["recovery"] == "/activate/recovery", f"{row['facade_id']} recovery page")
    check(row["callbacks"]["recovery"] == "/activate/callback/recovery", f"{row['facade_id']} recovery callback")
check("/activate/recovery" in INSTALL_PHP and "/activate/callback/recovery" in INSTALL_PHP,
      "install facade renders recovery pages")
check("recovery_only" in CALL_STACK["registration_states"]["terminal"], "recovery_only is terminal")
for state in ("denied", "refunded", "revoked", "superseded", "expired"):
    check("recovery_only" in transitions[state], f"{state} resolves to recovery_only")
check(transitions["recovery_only"] == [], "recovery_only is a terminal sink")
for code in ("EDD_LICENSE_UNUSABLE", "REFUNDED", "REVOKED"):
    row = next(error for error in ERRORS["errors"] if error["code"] == code)
    check(row["http_status"] == 403 and row["retryable"] is False and row["safe_next_action"] == "recovery_only",
          f"{code} is recovery-only")
check("'safe_url' => $origin . self::RENDER_PAGES['recovery']" in INSTALL_PHP,
      "outage recovery link is the registered recovery page")
check("$maskedEnvelope['state'] = 'recovery_only'" in INSTALL_PHP, "recovery pages force recovery-only posture")

# --- G. Spoofing matrix ---------------------------------------------------------
spoof_cases = [
    ("unknown facade", {"facade_id": "attacker_v1"}, "FACADE_ORIGIN_DENIED"),
    ("http origin", {"origin": "http://install.focusa.dev"}, "FACADE_ORIGIN_DENIED"),
    ("suffix spoof", {"origin": "https://install.focusa.dev.evil.invalid"}, "FACADE_ORIGIN_DENIED"),
    ("subdomain widening", {"origin": "https://child.install.focusa.dev"}, "FACADE_ORIGIN_DENIED"),
    ("wildcard origin", {"origin": "https://*.focusa.dev"}, "FACADE_ORIGIN_DENIED"),
    ("userinfo origin", {"origin": "https://user@install.focusa.dev"}, "FACADE_ORIGIN_DENIED"),
    ("port origin", {"origin": "https://install.focusa.dev:8443"}, "FACADE_ORIGIN_DENIED"),
    ("path origin", {"origin": "https://install.focusa.dev/extra"}, "FACADE_ORIGIN_DENIED"),
    ("cross-facade origin", {"facade_id": "focusa_marketing_v1", "origin": "https://install.focusa.dev"},
     "FACADE_ORIGIN_DENIED"),
    ("invented product", {"product_code": "invented_product_v1"}, "FACADE_PRODUCT_DENIED"),
    ("issuance route", {"route": "authority_issue"}, "FACADE_ROUTE_DENIED"),
    ("unknown route", {"route": "activation_bless"}, "FACADE_ROUTE_DENIED"),
    ("absolute callback", {"callback_handle": "https://evil.invalid/callback"}, "FACADE_CALLBACK_DENIED"),
    ("unknown callback", {"callback_handle": "attacker"}, "FACADE_CALLBACK_DENIED"),
    ("unknown locale", {"locale": "en-GB"}, "FACADE_LOCALE_DENIED"),
]
for name, overrides, expected in spoof_cases:
    check(deny(valid_request(overrides)) == expected, f"{name} fails closed", kind="negative")
for field in REGISTRY["request_contract"]["forbidden"]:
    check(deny(valid_request({field: "attacker-controlled"})) == "FACADE_REQUEST_FIELD_DENIED",
          f"caller authority field {field} denied", kind="negative")
for field in REGISTRY["request_contract"]["required"]:
    request = valid_request()
    del request[field]
    check(deny(request) == "FACADE_REQUEST_INVALID", f"missing required field {field}", kind="negative")
check("sender_id" in REGISTRY["request_contract"]["forbidden"] and "sender_email" in REGISTRY["request_contract"]["forbidden"],
      "sender identity is never caller-controlled")
check(REGISTRY["request_contract"]["unknown_sender"] == "FACADE_SENDER_DENIED", "sender denial code")

# --- H. Timeout -----------------------------------------------------------------
check("public const MAX_SKEW_SECONDS = 300" in PROTOCOL_PHP, "protocol timestamp skew bound is 300s")
check("FACADE_TIMESTAMP_DENIED" in PROTOCOL_PHP, "skewed requests fail closed")
check("public const SESSION_TTL_SECONDS = 1800" in SECURITY_PHP, "presenter session TTL is 1800s")
check("public const CSRF_TTL_SECONDS = 600" in SECURITY_PHP, "CSRF TTL is 600s")
check("FACADE_SESSION_DENIED" in SECURITY_PHP and "FACADE_CSRF_DENIED" in SECURITY_PHP,
      "expired session and CSRF deny")
check(GOLDEN["now"] + 300 == 1786061100, "golden now is fixed")
payload_now, _ = GOLDEN["request"]["continuation_token"].split(".")
claims_now = base64.urlsafe_b64decode(payload_now + "=" * ((4 - len(payload_now) % 4) % 4)).decode().split("\n")
check(claims_now[-1] == str(GOLDEN["now"] + 300), "continuation token expires now+300")
for code in ("EMAIL_VERIFICATION_EXPIRED", "EMAIL_VERIFICATION_FAILED", "EMAIL_VERIFICATION_REQUIRED"):
    row = next(error for error in ERRORS["errors"] if error["code"] == code)
    check(row["retryable"] is False, f"{code} is not retryable")
expired = next(error for error in ERRORS["errors"] if error["code"] == "EMAIL_VERIFICATION_EXPIRED")
check(expired["http_status"] == 410 and expired["safe_next_action"] == "restart_verification",
      "verification expiry restarts verification")

# --- I. Upstream authority outage -----------------------------------------------
outage_error = next(error for error in ERRORS["errors"] if error["code"] == "AUTHORITY_UNAVAILABLE")
check(outage_error["http_status"] == 503 and outage_error["retryable"] is True
      and outage_error["safe_next_action"] == "retry_or_use_recovery", "authority outage contract")
check("function authorityUnavailable" in INSTALL_PHP, "install facade implements outage handling")
check("'status' => 503" in INSTALL_PHP and "'state' => 'recovery_only'" in INSTALL_PHP
      and "'error' => 'AUTHORITY_UNAVAILABLE'" in INSTALL_PHP, "outage envelope is 503 recovery-only")
check("public const AUTHORITY = 'WPUIAI.com EDD'" in INSTALL_PHP, "install facade declares WPUIAI authority")
for surface in ("activation_start", "activation_verify", "activation_offers", "activation_checkout",
                "activation_existing_license", "activation_poll", "lease_refresh", "nodes",
                "account_manage_link"):
    check(f"'{surface}' =>" in INSTALL_PHP, f"install facade surface {surface}")
for field in ("edd_download_id", "edd_price_id", "price", "grants", "features", "limits",
              "sender_email", "redirect_url", "credential", "secret"):
    check(f"'{field}'" in INSTALL_PHP, f"install facade forbids caller field {field}")
check(OPENAPI["x-focusa-facade-authority"] == "proxy_only" and OPENAPI["x-focusa-spec158"] == "excluded",
      "OpenAPI facade posture is proxy-only")
check(OPENAPI["servers"][0]["variables"]["registeredFacade"]["default"] == "facade.invalid",
      "OpenAPI never names a real facade server")
check(CALL_STACK["authority"]["facade"] == "registered_authenticated_bounded_proxy_only", "call stack facade role")
check(INTERNAL["authority"]["facade"] == "registered_authenticated_bounded_proxy_only", "internal contract facade role")
deployment_roles = {row["id"]: row["target_role"] for row in INVENTORY["deployments"]}
check(deployment_roles["wpuiai_com"] == "sole canonical authority kernel", "WPUIAI is the sole authority kernel")
check(deployment_roles["install_focusa_dev"] == "registered branded facade installer host and bounded WPUIAI proxy",
      "install site is a bounded facade")

outage = json.loads(php_probe(
    'require "docs/contracts/spec152e-install-facade-routes.v1.php";'
    'echo json_encode(FocusaSpec152eInstallFacadeRoutes::authorityUnavailable('
    '"req_acceptance_outage", "https://install.focusa.dev"));'
))
check(outage["ok"] is False and outage["status"] == 503, "authority outage executes as 503")
check(outage["envelope"]["state"] == "recovery_only" and outage["envelope"]["terminal"] is False
      and outage["envelope"]["retry"] is True and outage["envelope"]["error"] == "AUTHORITY_UNAVAILABLE",
      "executed outage envelope is recovery-only and retryable")
check(outage["envelope"]["safe_url"] == "https://install.focusa.dev/activate/recovery",
      "executed outage envelope links the registered recovery page")
check(not any(key in outage["envelope"] for key in ("license_key", "lease_envelope", "node_id", "credential")),
      "outage envelope never issues license, lease, node, or credential")

routes = json.loads(php_probe(
    'require "docs/contracts/spec152e-install-facade-routes.v1.php";'
    '$registry = require "docs/contracts/spec152e-facade-registry.v1.php";'
    '$out = [];'
    'foreach (FocusaSpec152eInstallFacadeRoutes::pageRoutes() as $action => $route) {'
    '  $d = FocusaSpec152eInstallFacadeRoutes::resolveRoute($route["page"], $route["method"], '
    '"https://install.focusa.dev", "focusa_operator_lifetime_v1", $registry);'
    '  $out[$action] = [$d["ok"] ?? false, $d["authority_route"] ?? $d["error"] ?? null];'
    '}'
    'echo json_encode($out);'
))
check(len(routes) == 11, "all eleven install page routes resolve")
for action, (ok, authority_route) in routes.items():
    check(ok is True, f"{action} install page route resolves")
    check(authority_route == REGISTRY["proxy_routes"][action], f"{action} maps to the registered proxy route")

timeouts = json.loads(php_probe(
    'require "docs/contracts/spec152e-facade-security.v1.php";'
    '$registry = require "docs/contracts/spec152e-facade-registry.v1.php";'
    '$now = 1786060800;'
    '$secret = "synthetic-acceptance-matrix-secret-not-for-runtime";'
    '$base = ["facade_id" => "focusa_install_v1", "origin" => "https://install.focusa.dev",'
    '  "route" => "activation_start", "method" => "POST",'
    '  "product_code" => "focusa_operator_lifetime_v1", "redirect_handle" => "success",'
    '  "client_key" => "synthetic-browser-client"];'
    '$consumed = [];'
    '$consume = static function (string $f, string $s, string $n, int $e) use (&$consumed): bool {'
    '  $k = $f . ":" . $s . ":" . $n; if (isset($consumed[$k])) { return false; } $consumed[$k] = true; return true; };'
    '$rate = static fn(string $f, string $c, string $r): bool => $c !== "";'
    '$expiredSession = FocusaSpec152eFacadeSecurity::issueSession('
    '$registry, $secret, "focusa_install_v1", "https://install.focusa.dev", "session_expired_probe", $now - 1800);'
    '$d1 = FocusaSpec152eFacadeSecurity::verifyBrowserRequest('
    '$base + ["session_token" => $expiredSession["token"], "csrf_token" => "unused"],'
    '$registry, $secret, $consume, $rate, $now);'
    '$freshSession = FocusaSpec152eFacadeSecurity::issueSession('
    '$registry, $secret, "focusa_install_v1", "https://install.focusa.dev", "session_fresh_probe", $now);'
    '$expiredCsrf = FocusaSpec152eFacadeSecurity::issueCsrf('
    '$secret, "focusa_install_v1", "https://install.focusa.dev", "session_fresh_probe", "activation_start",'
    '"csrf_expired_probe", $now - 600);'
    '$d2 = FocusaSpec152eFacadeSecurity::verifyBrowserRequest('
    '$base + ["session_token" => $freshSession["token"], "csrf_token" => $expiredCsrf],'
    '$registry, $secret, $consume, $rate, $now);'
    'echo json_encode(["session_expired" => $d1["error"], "csrf_expired" => $d2["error"]]);'
))
check(timeouts["session_expired"] == "FACADE_SESSION_DENIED", "expired presenter session is denied")
check(timeouts["csrf_expired"] == "FACADE_CSRF_DENIED", "expired CSRF token is denied")

# --- J. Golden vector and hygiene -----------------------------------------------
request = GOLDEN["request"]
credential = GOLDEN["credential"]
check(GOLDEN["schema"] == "focusa.spec152e.facade_golden_vectors.v1", "golden vector schema")
check(request["schema"] == "focusa.spec152e.facade_protocol.v1", "golden request protocol schema")
check(request["body_sha256"] == hashlib.sha256(b"{}").hexdigest(), "golden body digest")
canonical = "\n".join(str(request[field]) for field in SIGNED_FIELDS)
check(canonical == GOLDEN["canonical_request"], "golden canonical request")
key = credential["key_utf8"].encode()
check(hmac.compare_digest(
    hmac.new(key, canonical.encode(), hashlib.sha256).hexdigest(), request["signature"]), "golden signature")
payload, signature = request["continuation_token"].split(".")
check(hmac.compare_digest(
    hmac.new(key, f"continuation-v1\n{payload}".encode(), hashlib.sha256).hexdigest(), signature),
    "golden continuation signature")
claims = base64.urlsafe_b64decode(payload + "=" * ((4 - len(payload) % 4) % 4)).decode().split("\n")
check(claims == [request["registration_id"], request["facade_id"], request["action"],
                 request["nonce"], str(GOLDEN["now"] + 300)], "continuation claims bind and expire")
check(GOLDEN["expected"] == {"authority_route": "/v1/activation/start",
                             "safe_redirect": "https://install.focusa.dev/activate/callback/success"},
      "golden expected authority routing")

openapi_paths = {
    operation["operationId"].replace(".", "_"): path
    for path, operations in OPENAPI["paths"].items()
    for operation in operations.values()
}
check(openapi_paths == REGISTRY["proxy_routes"], "OpenAPI proxy paths equal the registry proxy routes")
call_ids = {operation["id"].replace(".", "_") for operation in CALL_STACK["operations"]}
check(set(REGISTRY["proxy_routes"]) <= call_ids, "every proxy route has a call-stack operation")

for row in REGISTRY["facades"]:
    check(row["facade_id"] in BROWSER_SOURCE, f"browser security binds {row['facade_id']}")
    check(row["exact_origins"][0] in BROWSER_SOURCE, f"browser security binds {row['facade_id']} origin")
    for product in row["products"]:
        check(product in BROWSER_SOURCE, f"browser security binds {product}")
for themed in ("focusa_marketing_v1", "focusa_forge_v1", "focusa_arena_v1", "wpuiai_public_v1"):
    check(themed in REGISTRATION_SOURCE, f"website registration themes {themed}")
check("focusa_install_v1" not in REGISTRATION_SOURCE and "uiai_engine_v1" not in REGISTRATION_SOURCE,
      "installer surfaces are not website-themed registration pages")
check('<meta name="referrer" content="no-referrer">' in PAGE_HTML, "branded page leaks no referrer")

public_texts = {
    "registry.json": REGISTRY_JSON, "registry.yaml": REGISTRY_YAML, "registry.php": REGISTRY_PHP,
    "products.json": (CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8"),
    "call_stack.yaml": (CONTRACTS / "spec152e-activation-call-stack.v1.yaml").read_text(encoding="utf-8"),
    "errors.json": (CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"),
    "openapi.json": (CONTRACTS / "spec152e-activation-public-openapi.v1.json").read_text(encoding="utf-8"),
    "golden.json": (CONTRACTS / "spec152e-facade-golden-vectors.v1.json").read_text(encoding="utf-8"),
    "manifest.json": (CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8"),
    "install.php": INSTALL_PHP, "security.php": SECURITY_PHP, "protocol.php": PROTOCOL_PHP,
    "browser.mjs": BROWSER_SOURCE, "registration.mjs": REGISTRATION_SOURCE, "page.html": PAGE_HTML,
}
for name, text in public_texts.items():
    check(not EMAIL_RE.search(text), f"{name} contains no unmasked email")
    check(not SECRET_RE.search(text), f"{name} contains no stripe-shaped secret")
    check(not LIVE_RE.search(text), f"{name} contains no live license key")
check("*.focusa.dev" not in REGISTRY_YAML and "*.focusa.dev" not in REGISTRY_JSON and "*.focusa.dev" not in REGISTRY_PHP,
      "no wildcard authority in the registry")

print(json.dumps({
    "schema": "focusa.spec152e.facade_acceptance_matrix.v1",
    "facades": len(REGISTRY["facades"]),
    "exact_origins": REGISTRY["counts"]["exact_origins"],
    "product_bindings": bindings,
    "proxy_routes": len(REGISTRY["proxy_routes"]),
    "php_probes": 3,
    "positive_checks": positive,
    "negative_checks": negative,
    "result": "passed_fail_closed",
}, sort_keys=True))
