#!/usr/bin/env python3
"""Spec 152E.07.06 — unverified-email, facade, checkout, and credential adversarial matrix.

Attacks every named surface of the Spec 152E addendum — Verification, facade,
checkout, agent, terminal, logs, and the redirect/origin/token/product/key
boundaries — with adversarial inputs and asserts the safe expected errors:

  - invalid / unreachable / disposable-policy email (provider-neutral, pending-only)
  - wrong / expired / replayed verification codes and gate tokens
  - enumeration resistance (identical safe responses for known vs unknown state)
  - changed checkout email (fulfillment held until verified-link review)
  - raw cart / direct cart-session access (server-owned fields only)
  - spoofed facade / origin / redirect (sessions, CSRF, return handles)
  - arbitrary grants, prices, downloads, and caller redirects (fail closed)
  - token / key / log leakage (masked output, hashed at rest, no plaintext in logs)

The matrix is replayable from the pinned commit: it re-runs the existing
build-independent PHP adversarial suites offline, probes the executable PHP
contracts through the CLI with the frozen registries, and statically binds the
Rust/TS/browser surfaces to the frozen contracts. No network, no cargo build,
no publication. Every adversarial case must fail closed without creating any
customer entitlement, payment fulfillment, key, node, lease, or secret
disclosure outside policy.

Exact verification: python3 tests/spec152e_adversarial_identity_facade_test.py
"""

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
TESTS = ROOT / "tests"

ERRORS = json.loads((CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"))
FACADES = json.loads((CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))
PRODUCTS = json.loads((CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8"))
AGENT = json.loads((CONTRACTS / "spec152e-agent-activation.v1.json").read_text(encoding="utf-8"))
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
OPENAPI = json.loads((CONTRACTS / "spec152e-activation-public-openapi.v1.json").read_text(encoding="utf-8"))

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
LIVE_KEY_RE = re.compile(r"(?i)focusa_live_[0-9]+_[0-9a-f]+")
FULL_KEY_RE = re.compile(r"[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}")

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


def php_probe(code: str) -> dict:
    proc = subprocess.run(
        ["php", "-d", "log_errors=0", "-d", "error_log=/dev/null", "-r", code],
        capture_output=True, text=True, cwd=str(ROOT),
    )
    check(proc.returncode == 0, f"php probe exited 0: {proc.stderr[:240]}", kind="negative")
    return json.loads(proc.stdout.strip())


def run_php_suite(name: str) -> None:
    proc = subprocess.run(["php", str(TESTS / name)], capture_output=True, text=True, cwd=str(ROOT))
    check(proc.returncode == 0, f"{name} exited 0: {proc.stderr[:240]}", kind="negative")
    result = json.loads(proc.stdout.strip())
    check(result["result"] == "passed_fail_closed", f"{name} result passed_fail_closed", kind="negative")
    global positive, negative
    positive += int(result.get("positive_checks", result.get("positive_round_trips", result.get("assertion_groups", 0))))
    negative += int(result.get("negative_checks", 0))


def run_python_suite(name: str) -> None:
    proc = subprocess.run(["python3", str(TESTS / name)], capture_output=True, text=True, cwd=str(ROOT))
    check(proc.returncode == 0, f"{name} exited 0: {proc.stderr[:240]}", kind="negative")
    result = json.loads(proc.stdout.strip())
    check(result["result"] == "passed_fail_closed", f"{name} result passed_fail_closed", kind="negative")
    global positive, negative
    positive += int(result["positive_checks"])
    negative += int(result["negative_checks"])


# ════════════════════════════════════════════════════════════════════════
# Part A — replay the build-independent adversarial suites (offline)
# ════════════════════════════════════════════════════════════════════════

for suite in [
    # Verification: pending-only invalid/unreachable email, enumeration-safe start.
    "spec152e_verification_start_test.php",
    # Verification: wrong/expired/replayed code, attempts, enumeration, cross-facade.
    "spec152e_verification_complete_test.php",
    # Checkout: changed/blank/conflicting email holds fulfillment until review.
    "spec152e_checkout_email_integrity_test.php",
    # Checkout: raw cart access and caller fields are impossible; branded return only.
    "spec152e_edd_checkout_intent_test.php",
    # Facade: spoofed session/CSRF/origin/product/redirect fail closed.
    "spec152e_facade_security_test.php",
    # Facade: origin/timestamp/redirect boundaries and protocol binding.
    "spec152e_facade_protocol_test.php",
    # Terminal: envelope key boundaries, one-time delivery, no key leakage.
    "spec152e_terminal_envelope_test.php",
    # Identity: email schema, normalization, and keyed lookup policy.
    "spec152e_email_identity_schema_test.php",
    # Product: client-controlled price/grants/limits never accepted.
    "spec152e_edd_product_gate_test.php",
    # Commerce: no card data, no credential leakage across the EDD boundary.
    "spec152e_edd_commerce_acceptance_test.php",
]:
    run_php_suite(suite)

# Agent surface: typed human-action envelope, masked by default, never invents
# an email/code/consent/payment/license, no raw secret fields.
run_python_suite("spec152e_agent_json_contract_test.py")


# ════════════════════════════════════════════════════════════════════════
# Part B — targeted adversarial probes against the executable PHP contracts
# ════════════════════════════════════════════════════════════════════════

# B1. Email policy: invalid / unreachable / disposable-aliasing email.
# Provider-neutral normalization: dot/plus aliases are never merged, pending-only.
PROBE_EMAIL = r'''
require "docs/contracts/spec152e-email-identity.v1.php";
$out = [];
$out["dot_alias_distinct"] = FocusaSpec152eEmailNormalizer::exact("alice.lee@Example.COM") !== FocusaSpec152eEmailNormalizer::exact("alicelee@example.com");
$out["plus_alias_distinct"] = FocusaSpec152eEmailNormalizer::exact("alice+spam@example.com") !== FocusaSpec152eEmailNormalizer::exact("alice@example.com");
$out["domain_lowercased"] = FocusaSpec152eEmailNormalizer::exact("alice@EXAMPLE.COM") === "alice@example.com";
$out["local_case_preserved"] = FocusaSpec152eEmailNormalizer::exact("Alice@example.com") === "Alice@example.com";
$out["surrounding_whitespace_stripped"] = FocusaSpec152eEmailNormalizer::exact("  alice@example.com\t") === "alice@example.com";
$invalid = ["", "no-at-sign", "a@", "@example.com", "alice@exa\nmple.com", "alice@example.com\x00x", str_repeat("a", 64) . "@" . str_repeat("b", 191) . ".com"];
$rejected = 0;
foreach ($invalid as $candidate) { try { FocusaSpec152eEmailNormalizer::exact($candidate); } catch (InvalidArgumentException) { $rejected++; } }
$out["all_invalid_rejected"] = $rejected === count($invalid);
$secrets = new FocusaSpec152eEmailIdentitySecrets(str_repeat("e", 32), str_repeat("l", 64));
$out["digest_dot_alias_distinct"] = $secrets->digest("alice.lee@example.com") !== $secrets->digest("alicelee@example.com");
$out["digest_plus_alias_distinct"] = $secrets->digest("alice+spam@example.com") !== $secrets->digest("alice@example.com");
$out["digest_exact_repeatable"] = $secrets->digest("alice@example.com") === $secrets->digest("alice@example.com");
$cipher = $secrets->encrypt("alice@example.com");
$out["encrypt_not_plaintext"] = strpos($cipher, "alice@example.com") === false;
$out["decrypt_roundtrip"] = $secrets->decrypt($cipher) === "alice@example.com";
try { $secrets->decrypt("tampered-envelope"); $out["tamper_denied"] = false; } catch (DomainException) { $out["tamper_denied"] = true; }
echo json_encode($out);
'''

# B2. Challenge codes: wrong / expired / replayed / injected verifiers.
PROBE_CHALLENGE = r'''
require "docs/contracts/spec152e-activation-registration.v1.php";
require "docs/contracts/spec152e-challenge-service.v1.php";
$svc = new FocusaSpec152eChallengeService(str_repeat("v", 32));
$out = [];
$otp = $svc->generateOtp("focusa_install_v1", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z");
$out["otp_is_6_digits"] = preg_match("/^\d{6}$/D", $otp["code"]) === 1;
$out["otp_hash_matches"] = hash_equals($otp["verifier_hash"], $svc->hash($otp["code"]));
$out["wrong_code_fails"] = $svc->validate("000000", $otp["verifier_hash"]) === false;
$out["crlf_injection_fails"] = $svc->validate("123456\r\n", $otp["verifier_hash"]) === false;
$out["oversized_verifier_fails"] = $svc->validate(str_repeat("x", 300), $otp["verifier_hash"]) === false;
$out["tampered_hash_fails"] = $svc->validate("483921", str_repeat("0", 64)) === false;
$link = $svc->generateMagicLink("focusa_install_v1", "/activate/verify", "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "https://install.focusa.dev", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z");
$out["magic_link_binds_registration"] = strpos($link["magic_link"], "registration=018f47c2-6ac0-7b16-8d1a-4e93df5a0101") !== false;
$out["magic_verifier_urlsafe"] = preg_match("/^[A-Za-z0-9_-]{43}$/D", $link["verifier"]) === 1;
$out["magic_hash_matches"] = hash_equals($link["verifier_hash"], $svc->hash($link["verifier"]));
$threw = 0;
try { $svc->generateOtp("evil_facade\nid", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z"); } catch (InvalidArgumentException) { $threw++; }
try { $svc->generateMagicLink("focusa_install_v1", "/activate/verify", "not-a-uuid", "https://install.focusa.dev", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z"); } catch (InvalidArgumentException) { $threw++; }
try { $svc->generateMagicLink("focusa_install_v1", "/activate/verify", "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "http://install.focusa.dev", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z"); } catch (InvalidArgumentException) { $threw++; }
try { $svc->generateOtp("focusa_install_v1", "2026-08-08", "2026-08-08T00:15:00Z"); } catch (InvalidArgumentException) { $threw++; }
$out["invalid_inputs_rejected"] = $threw === 4;
echo json_encode($out);
'''

# B3. Facade spoof: forged/cross-facade/expired sessions, expired/replayed CSRF,
# wrong origin (including suffix spoof), unknown product, absolute redirect.
PROBE_FACADE = r'''
require "docs/contracts/spec152e-facade-security.v1.php";
$registry = require "docs/contracts/spec152e-facade-registry.v1.php";
$out = [];
$now = 1786060800;
$secret = "synthetic-adversarial-matrix-secret-not-for-runtime";
$consumed = [];
$consume = static function (string $f, string $s, string $n, int $e) use (&$consumed): bool {
  $k = $f . ":" . $s . ":" . $n; if (isset($consumed[$k])) { return false; } $consumed[$k] = true; return true; };
$rate = static fn(string $f, string $c, string $r): bool => $c !== "";
$base = ["facade_id" => "focusa_install_v1", "origin" => "https://install.focusa.dev", "route" => "activation_start", "method" => "POST", "product_code" => "focusa_operator_lifetime_v1", "redirect_handle" => "success"];
$verify = static function (array $req) use ($registry, $secret, $consume, $rate, $now): array {
  return FocusaSpec152eFacadeSecurity::verifyBrowserRequest($req, $registry, $secret, $consume, $rate, $now); };
$forged = FocusaSpec152eFacadeSecurity::issueSession($registry, str_repeat("a", 32), "focusa_install_v1", "https://install.focusa.dev", "victim-session", $now);
$out["forged_session_denied"] = ($verify($base + ["session_token" => $forged["token"], "csrf_token" => "unused"])["error"] ?? "") === "FACADE_SESSION_DENIED";
$good = FocusaSpec152eFacadeSecurity::issueSession($registry, $secret, "focusa_install_v1", "https://install.focusa.dev", "matrix-session", $now);
$out["cross_facade_session_denied"] = ($verify(["facade_id" => "focusa_marketing_v1", "origin" => "https://focusa.dev", "route" => "activation_start", "method" => "POST", "product_code" => "focusa_operator_lifetime_v1", "redirect_handle" => "success", "session_token" => $good["token"], "csrf_token" => "x"])["error"] ?? "") === "FACADE_SESSION_DENIED";
$expired = FocusaSpec152eFacadeSecurity::issueSession($registry, $secret, "focusa_install_v1", "https://install.focusa.dev", "expired-session", $now - 1800);
$out["expired_session_denied"] = ($verify($base + ["session_token" => $expired["token"], "csrf_token" => "x"])["error"] ?? "") === "FACADE_SESSION_DENIED";
$csrf = FocusaSpec152eFacadeSecurity::issueCsrf($secret, "focusa_install_v1", "https://install.focusa.dev", "matrix-session", "activation_start", "csrf-nonce-matrix", $now);
$first = $verify($base + ["session_token" => $good["token"], "csrf_token" => $csrf]);
$second = $verify($base + ["session_token" => $good["token"], "csrf_token" => $csrf]);
$out["csrf_replay_denied"] = ($first["ok"] ?? false) === true && ($second["error"] ?? "") === "FACADE_CSRF_DENIED";
$expiredCsrf = FocusaSpec152eFacadeSecurity::issueCsrf($secret, "focusa_install_v1", "https://install.focusa.dev", "matrix-session", "activation_start", "csrf-expired-matrix", $now - 600);
$out["expired_csrf_denied"] = ($verify($base + ["session_token" => $good["token"], "csrf_token" => $expiredCsrf])["error"] ?? "") === "FACADE_CSRF_DENIED";
$out["wrong_origin_denied"] = ($verify(array_replace($base, ["origin" => "https://evil.invalid"], ["session_token" => $good["token"], "csrf_token" => "x"]))["error"] ?? "") === "FACADE_ORIGIN_DENIED";
$out["suffix_origin_denied"] = ($verify(array_replace($base, ["origin" => "https://install.focusa.dev.evil.invalid"], ["session_token" => $good["token"], "csrf_token" => "x"]))["error"] ?? "") === "FACADE_ORIGIN_DENIED";
$out["unknown_product_denied"] = ($verify(array_replace($base, ["product_code" => "attacker_product_v1"], ["session_token" => $good["token"], "csrf_token" => "x"]))["error"] ?? "") === "FACADE_PRODUCT_DENIED";
$out["absolute_redirect_denied"] = ($verify(array_replace($base, ["redirect_handle" => "https://evil.invalid/hook"], ["session_token" => $good["token"], "csrf_token" => "x"]))["error"] ?? "") === "FACADE_REDIRECT_DENIED";
$out["unknown_redirect_denied"] = ($verify(array_replace($base, ["redirect_handle" => "attacker"], ["session_token" => $good["token"], "csrf_token" => "x"]))["error"] ?? "") === "FACADE_REDIRECT_DENIED";
$noSession = $base; unset($noSession["session_token"]);
$out["missing_session_denied"] = ($verify($noSession + ["csrf_token" => "x"])["error"] ?? "") === "FACADE_REQUEST_DENIED";
$out["safe_redirect_bounded"] = ($first["safe_redirect"] ?? "") === "https://install.focusa.dev/activate/callback/success";
echo json_encode($out);
'''

# B4. Checkout / raw cart / arbitrary grants: caller-controlled commercial and
# redirect fields are impossible at both the intent and the cart-session layer;
# unknown raw cart/intent references resolve nowhere.
PROBE_CHECKOUT = r'''
require "docs/contracts/spec152e-activation-registration.v1.php";
require "docs/contracts/spec152e-email-identity.v1.php";
require "docs/contracts/spec152e-authority-account.v1.php";
require "docs/contracts/spec152e-account-promotion.v1.php";
require "docs/contracts/spec152e-edd-customer-adapter.v1.php";
require "docs/contracts/spec152e-edd-product-registry.v1.php";
require "docs/contracts/spec152e-facade-registry.v1.php";
require "docs/contracts/spec152e-verified-registration-token-validator.v1.php";
require "docs/contracts/spec152e-edd-checkout-intent.v1.php";
$db = new PDO("sqlite::memory:");
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$nowValue = "2026-08-08T00:01:00Z";
$clock = static function () use (&$nowValue): string { return $nowValue; };
$intentMigration = new FocusaSpec152eEddCheckoutIntentMigration($db, "wp_");
$intentMigration->migrate("2026-08-08T00:00:00Z", ["source" => "adversarial_matrix"]);
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, "wp_");
$registrationMigration->migrate("2026-08-08T00:00:00Z", ["source" => "adversarial_matrix"]);
$secrets = new FocusaSpec152eActivationRegistrationSecrets(str_repeat("e", 32), str_repeat("v", 32), str_repeat("p", 32));
$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $secrets, $clock);
$cart = new FocusaSpec152eEddCartSessionAdapter($db, $intentMigration, $clock);
$facadeRegistry = require "docs/contracts/spec152e-facade-registry.v1.php";
$returnHandles = new FocusaSpec152eFacadeReturnHandleRegistry($facadeRegistry);
$frozenRegistry = require "docs/contracts/spec152e-edd-product-registry.v1.php";
$checkout = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $frozenRegistry, $clock);
$probe = static function (callable $fn): string {
  try { $fn(); return "NO_ERROR"; } catch (DomainException $e) { return $e->getMessage(); }
  catch (InvalidArgumentException $e) { return "INVALID_ARGUMENT"; }
  catch (OutOfBoundsException $e) { return "NOT_FOUND"; } };
$out = [];
$baseIntent = ["facade_id" => "focusa_install_v1", "origin" => "https://install.focusa.dev", "return_handle" => "success", "request_id" => "req-adv-0001", "idempotency_key" => "idem-adv-0001"];
$out["client_price_forbidden"] = $probe(fn() => $checkout->createIntent($baseIntent + ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "price" => "1.00"])) === "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN";
$out["client_grants_forbidden"] = $probe(fn() => $checkout->createIntent($baseIntent + ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "grants" => ["focusa.core.all"]])) === "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN";
$out["client_download_forbidden"] = $probe(fn() => $checkout->createIntent($baseIntent + ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "edd_download_id" => 1001])) === "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN";
$out["client_redirect_forbidden"] = $probe(fn() => $checkout->createIntent($baseIntent + ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "callback_url" => "https://evil.invalid/hook"])) === "FACADE_REDIRECT_DENIED";
$cartBase = ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "edd_customer_id" => 1, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "edd_download_id" => 1001, "edd_price_id" => "price_focusa_op_v1", "price_usd" => "697.00", "request_id" => "req-adv-cart-1", "idempotency_key" => "idem-adv-cart-1"];
$out["cart_grants_forbidden"] = $probe(fn() => $cart->openSession($cartBase + ["limits" => ["nodes" => 99]])) === "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN";
$out["cart_caller_redirect_forbidden"] = $probe(fn() => $cart->openSession($cartBase + ["redirect_url" => "https://evil.invalid"])) === "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN";
$out["unknown_cart_reference_not_found"] = $probe(fn() => $cart->findByCartReference("cs_00000000000000000000000000000000")) === "NOT_FOUND";
$out["unknown_intent_reference_not_found"] = $probe(fn() => $checkout->findByIntentId("it_00000000000000000000000000000000")) === "NOT_FOUND";
$out["unknown_registration_fails_closed"] = $probe(fn() => $checkout->createIntent($baseIntent + ["registration_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101"])) === "EMAIL_VERIFICATION_REQUIRED";
echo json_encode($out);
'''

# B5. Verified-registration gate token: unverified issue, wrong token/product,
# single-use consumption, replay never re-returns the raw token.
PROBE_TOKEN = r'''
require "docs/contracts/spec152e-activation-registration.v1.php";
require "docs/contracts/spec152e-verified-registration-token-validator.v1.php";
$db = new PDO("sqlite::memory:");
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$nowValue = "2026-08-08T00:01:00Z";
$clock = static function () use (&$nowValue): string { return $nowValue; };
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, "wp_");
$registrationMigration->migrate("2026-08-08T00:00:00Z", ["source" => "adversarial_matrix"]);
$tokenMigration = new FocusaSpec152eEddRegistrationTokenMigration($db, "wp_");
$tokenMigration->migrate("2026-08-08T00:00:00Z", ["source" => "adversarial_matrix"]);
$secrets = new FocusaSpec152eActivationRegistrationSecrets(str_repeat("e", 32), str_repeat("v", 32), str_repeat("p", 32));
$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $secrets, $clock);
$validator = new FocusaSpec152eVerifiedRegistrationTokenValidator($db, $tokenMigration, $registrations, $clock);
$probe = static function (callable $fn): string {
  try { $fn(); return "NO_ERROR"; } catch (DomainException $e) { return $e->getMessage(); }
  catch (InvalidArgumentException $e) { return "INVALID_ARGUMENT"; } };
$out = [];
$pending = $registrations->createPending(["email" => "synthetic.token.adv@example.invalid", "facade_id" => "focusa_install_v1", "presenter" => "terminal", "install_channel" => "source_build", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0001", "idempotency_key" => "idem-token-0001"]);
$uuid = $pending["registration"]["registration_uuid"];
$out["unverified_issue_denied"] = $probe(fn() => $validator->issue(["registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0002", "idempotency_key" => "idem-token-0002"])) === "EMAIL_VERIFICATION_REQUIRED";
$registrations->verifyEmail($uuid, $pending["verification_secret"], "req-token-verify-0001", "idem-token-verify-0001");
$issued = $validator->issue(["registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0003", "idempotency_key" => "idem-token-0003"]);
$raw = $issued["registration_token"] ?? "";
$out["token_prefix_opaque"] = str_starts_with($raw, "rg_");
$replay = $validator->issue(["registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0003", "idempotency_key" => "idem-token-0003"]);
$out["token_issue_replay_never_rereturns_raw"] = ($replay["replayed"] ?? false) === true && !isset($replay["registration_token"]);
$out["wrong_token_denied"] = $probe(fn() => $validator->validate(["registration_token" => "rg_" . str_repeat("0", 43), "registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0004", "idempotency_key" => "idem-token-0004"])) === "EMAIL_VERIFICATION_REQUIRED";
$out["wrong_product_denied"] = $probe(fn() => $validator->validate(["registration_token" => $raw, "registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "uiai_operator_lifetime_v1", "request_id" => "req-token-0005", "idempotency_key" => "idem-token-0005"])) === "FACADE_PRODUCT_DENIED";
$ok = $validator->validate(["registration_token" => $raw, "registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0006", "idempotency_key" => "idem-token-0006"]);
$out["valid_token_consumed"] = ($ok["ok"] ?? false) === true && ($ok["token_state"] ?? "") === "consumed";
$out["consumed_replay_denied"] = $probe(fn() => $validator->validate(["registration_token" => $raw, "registration_uuid" => $uuid, "facade_id" => "focusa_install_v1", "product_code" => "focusa_operator_lifetime_v1", "request_id" => "req-token-0007", "idempotency_key" => "idem-token-0007"])) === "EMAIL_VERIFICATION_REQUIRED";
echo json_encode($out);
'''

# B6. Terminal delivery / key boundaries: canonical key pattern only, masked
# output, keyed digest, forged/expired/binding-mismatched envelopes fail closed.
PROBE_ENVELOPE = r'''
require "docs/contracts/spec152e-terminal-delivery-envelope.v1.php";
$out = [];
$key = "0123ABCD-4567EFAB-89CD0123-4567EFAB";
$binding = ["registration_id" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0101", "account_uuid" => "018f47c2-6ac0-7b16-8d1a-4e93df5a0102", "customer_id" => 42, "edd_license_id" => 7, "product_code" => "focusa_operator_lifetime_v1"];
$claims = FocusaSpec152eTerminalDeliveryEnvelope::buildClaims($binding, $key, "env_" . str_repeat("a", 32), "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z");
$out["claims_built_one_time"] = ($claims["one_time"] ?? null) === true;
$out["masked_key_format"] = FocusaSpec152eTerminalDeliveryEnvelope::maskKey($key) === "********-********-********-EFAB";
$digest = FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($key);
$out["key_digest_64hex"] = preg_match("/^[a-f0-9]{64}$/D", $digest) === 1;
$out["digest_never_plaintext"] = strpos($digest, "0123ABCD") === false;
$probe = static function (callable $fn): string {
  try { $fn(); return "NO_ERROR"; } catch (DomainException $e) { return $e->getMessage(); }
  catch (InvalidArgumentException $e) { return "INVALID_ARGUMENT"; } };
$out["synthetic_prefix_key_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::buildClaims($binding, "focusa_live_1234_abcd", "env_" . str_repeat("a", 32), "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z")) === "EDD_LICENSE_UNUSABLE";
$out["lowercase_key_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::buildClaims($binding, "0123abcd-4567efab-89cd0123-4567efab", "env_" . str_repeat("a", 32), "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z")) === "EDD_LICENSE_UNUSABLE";
$out["bad_envelope_id_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::buildClaims($binding, $key, "env_tampered", "2026-08-08T00:00:00Z", "2026-08-08T00:15:00Z")) === "INVALID_ARGUMENT";
$bad = $claims; $bad["one_time"] = false;
$out["non_one_time_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($bad, "2026-08-08T00:01:00Z", null)) === "ENVELOPE_FORMAT_DENIED";
$bad2 = $claims; $bad2["schema"] = "focusa.spec152e.forged.v1";
$out["forged_schema_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($bad2, "2026-08-08T00:01:00Z", null)) === "ENVELOPE_FORMAT_DENIED";
$bad3 = $claims; $bad3["registration_id"] = "018f47c2-6ac0-7b16-8d1a-4e93df5a0999";
$out["binding_mismatch_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($bad3, "2026-08-08T00:01:00Z", "018f47c2-6ac0-7b16-8d1a-4e93df5a0101")) === "ENVELOPE_BINDING_MISMATCH";
$out["expired_envelope_denied"] = $probe(fn() => FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, "2026-08-08T00:20:00Z", null)) === "ENVELOPE_EXPIRED";
echo json_encode($out);
'''

probe_results = {
    "email_policy": php_probe(PROBE_EMAIL),
    "challenge_codes": php_probe(PROBE_CHALLENGE),
    "facade_spoof": php_probe(PROBE_FACADE),
    "checkout_raw_cart_grants": php_probe(PROBE_CHECKOUT),
    "gate_token": php_probe(PROBE_TOKEN),
    "terminal_envelope_keys": php_probe(PROBE_ENVELOPE),
}
for probe_name, results in probe_results.items():
    for case, outcome in results.items():
        check(outcome is True, f"{probe_name}: {case}", kind="negative")


# ════════════════════════════════════════════════════════════════════════
# Part C — static binding to frozen contracts and leakage scans
# ════════════════════════════════════════════════════════════════════════

# C1. Frozen public error registry: stable, public-safe, and complete for every
# public error the adversarial flows can surface.
public_codes = {row["code"] for row in ERRORS["errors"]}
check(len(public_codes) == 33, "frozen public error registry has 33 stable codes")
check(ERRORS["rules"] == {
    "codes_are_stable": True, "messages_are_public_safe": True,
    "presenters_must_not_rewrite": True, "unknown_codes_fail_closed": True,
}, "error registry rules are fail-closed")
for code in ["EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_EXPIRED",
             "EMAIL_VERIFICATION_FAILED", "FACADE_ORIGIN_DENIED", "FACADE_PRODUCT_DENIED",
             "ACCOUNT_EMAIL_MISMATCH", "ACCOUNT_MERGE_REVIEW_REQUIRED", "EDD_ORDER_UNVERIFIED",
             "EDD_CHECKOUT_REQUIRED", "PRODUCT_MAPPING_REQUIRED", "EDD_LICENSE_UNUSABLE",
             "REFUNDED", "REVOKED", "AUTHORITY_UNAVAILABLE", "EMAIL_DELIVERY_FAILED"]:
    check(code in public_codes, f"public error {code} is registered")
for row in ERRORS["errors"]:
    check(row["http_status"] in {400, 401, 403, 409, 410, 429, 503, 202}, f"{row['code']} safe http status")
    check(row["safe_next_action"] != "" and "raw" not in row["public_message"].lower(),
          f"{row['code']} has a safe next action and public message")
check(INTERNAL["error_registry"] == "docs/contracts/spec152e-activation-errors.v1.json",
      "internal contract pins the frozen public error registry")

# C2. Facade / product authority posture: no facade issues entitlement, no
# caller-controlled grant fields, no wildcard authority.
check(FACADES["authority"]["entitlement_issuance"] == "forbidden", "facade issuance forbidden")
check("authority_issue" not in FACADES["proxy_routes"], "no issuance route on any facade")
check("*.focusa.dev" not in json.dumps(FACADES), "no wildcard facade authority")
for field in ["grants", "features", "limits", "price", "edd_download_id", "edd_price_id",
              "sender_email", "redirect_url", "credential", "secret"]:
    check(field in FACADES["request_contract"]["forbidden"], f"caller grant/redirect field {field} forbidden")
check(OPENAPI["x-focusa-facade-authority"] == "proxy_only", "OpenAPI facade posture is proxy-only")
check(OPENAPI["x-focusa-spec158"] == "excluded", "Spec 158 remains excluded")
check(PRODUCTS["counts"]["checkout_enabled"] == 0 and PRODUCTS["counts"]["assigned_edd_downloads"] == 0,
      "frozen product registry has zero checkout-enabled and zero assigned downloads")
for offer in PRODUCTS["protected_offers"]:
    check(offer["mapping_status"] == "approved_policy_blocked_edd_mapping"
          and offer["checkout_enabled"] is False and offer["edd_download_id"] is None,
          f"frozen offer {offer['public_code']} is policy-blocked and unassigned")

# C3. Agent surface: frozen envelope never carries raw email, key, or secrets.
envelope = AGENT["envelope"]
check(envelope["schema"] == "focusa.agent_activation_envelope.v1", "agent envelope schema")
for field in ["email", "normalized_email", "raw_email", "full_license_key",
              "one_time_key_envelope", "lease_envelope", "poll_credential",
              "poll_credential_hash", "verification_hash", "server_credential",
              "signing_key", "card_pan", "card_expiry", "card_cvc", "edd_internal_record"]:
    check(field in set(envelope["forbidden"]), f"agent envelope forbids {field}")
check(AGENT["secret_masking"]["email"] == "masked_by_default_masked_email_only"
      and AGENT["secret_masking"]["key"] == "masked_by_default", "agent masks email and key by default")
check(AGENT["secret_masking"]["reveal_policy"]["reveal_requires_both"] is True,
      "agent key reveal requires BOTH opt-in and confirmation")
check(AGENT["bounded_poll"]["timeout_settles_fail_closed"] == "cancel_to_recovery_only",
      "agent poll timeout settles fail-closed")

# C4. Leakage scan across every named surface (PHP contracts, frozen JSON,
# browser sources, Rust agent/CLI/daemon, Pi TS envelope). Only reserved
# @example.com/@example.invalid fixtures and the public support address may
# appear; no stripe-shaped secret, no synthetic legacy key, no full key.
surface_texts = {}
for path in sorted((CONTRACTS / "spec152e-facade-registry.v1.php").parent.glob("spec152e-*.v1.php")):
    surface_texts[path.name] = path.read_text(encoding="utf-8")
for path in sorted(CONTRACTS.glob("spec152e-*.v1.json")):
    surface_texts[path.name] = path.read_text(encoding="utf-8")
for path in sorted((ROOT / "public/activation").glob("*")):
    surface_texts[f"public/activation/{path.name}"] = path.read_text(encoding="utf-8")
for rel in ["crates/focusa-license/src/activation_agent.rs",
            "crates/focusa-cli/src/commands/activation_flow.rs",
            "crates/focusa-cli/src/commands/license.rs",
            "crates/focusa-api/src/routes/license.rs",
            "apps/pi-extension/src/activation-envelope.ts"]:
    surface_texts[rel] = (ROOT / rel).read_text(encoding="utf-8")

for name, text in surface_texts.items():
    for match in EMAIL_RE.findall(text):
        if match.endswith("@example.com") or match.endswith("@example.invalid") or match == "support@focusa.dev":
            continue
        raise AssertionError(f"unmasked email in {name}: {match}")
    check(not SECRET_RE.search(text), f"{name} contains no stripe-shaped secret", kind="negative")
    check(not LIVE_KEY_RE.search(text), f"{name} contains no synthetic live key", kind="negative")
    # The terminal-envelope golden-vector fixture intentionally carries one bounded
    # synthetic canonical key for byte-exact cross-language verification; every other
    # surface must never contain a full license key pattern.
    if name != "spec152e-terminal-envelope-golden-vectors.v1.json":
        check(not FULL_KEY_RE.search(text), f"{name} contains no full license key", kind="negative")
    check("*.focusa.dev" not in text, f"{name} contains no wildcard facade authority", kind="negative")

# Logs surface: the terminal/CLI presenter emits masked email only, never a raw
# key; the agent protocol module never prints; the daemon masks identity.
flow = surface_texts["crates/focusa-cli/src/commands/activation_flow.rs"]
check("A copy was emailed to {masked}" in flow, "terminal logs masked email only")
check("full_license_key" not in flow and "println!(\"License:" not in flow,
      "terminal never prints a raw license key", kind="negative")
agent_rs = surface_texts["crates/focusa-license/src/activation_agent.rs"]
check("println!" not in agent_rs, "agent protocol module never prints", kind="negative")
check("mask_key_prefix" in agent_rs and "masked_email_or_none" in agent_rs,
      "agent surface has key and email maskers")
daemon = surface_texts["crates/focusa-api/src/routes/license.rs"]
check("masked_identity" in daemon, "daemon masks customer identity")

# C5. Enumerated safe failures: the raw-cart / arbitrary-grant / spoof errors
# stay internal fail-closed decisions, never public presenter messages.
internal_only = {"ACTIVATION_REQUEST_ACCEPTED", "CLIENT_COMMERCIAL_FIELDS_FORBIDDEN",
                 "FACADE_SESSION_DENIED", "FACADE_CSRF_DENIED", "FACADE_REDIRECT_DENIED",
                 "FACADE_REQUEST_DENIED", "FACADE_METHOD_DENIED", "FACADE_CALLBACK_DENIED",
                 "FACADE_LOCALE_DENIED", "FACADE_TIMESTAMP_DENIED", "FACADE_SENDER_DENIED",
                 "FACADE_ROUTE_DENIED", "ENVELOPE_FORMAT_DENIED", "ENVELOPE_EXPIRED",
                 "ENVELOPE_BINDING_MISMATCH", "EMAIL_IDENTITY_DECRYPTION_FAILED"}
check(internal_only.isdisjoint(public_codes), "internal fail-closed codes stay internal")

# ════════════════════════════════════════════════════════════════════════
# Bounded result
# ════════════════════════════════════════════════════════════════════════

print(json.dumps({
    "schema": "focusa.spec152e.adversarial_identity_facade_matrix.v1",
    "positive_checks": positive,
    "negative_checks": negative,
    "replayed_php_suites": 10,
    "replayed_python_suites": 1,
    "php_probes": 6,
    "probe_cases": sum(len(r) for r in probe_results.values()),
    "leakage_scanned_surfaces": len(surface_texts),
    "internal_fail_closed_codes": sorted(internal_only),
    "result": "passed_fail_closed",
}, sort_keys=True))
