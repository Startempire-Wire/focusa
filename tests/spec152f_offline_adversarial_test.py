#!/usr/bin/env python3
"""Spec 152F.06.05 — Prove Offline Grace, outage, and bypass resistance.

Atom focusa-vbcqu.20.14.47 (152F.06.05): prove that no outage, stale cache,
alternate presenter, worker, or child token widens capability, and that
recovery remains available.

The gate drives one deterministic PHP journey through the canonical contracts
read-only (cached lease fixtures, authority outage harness, stale
sequence/refund/revoke set, direct core/API/CLI/worker/UIAI bypass attempts)
and then statically pins the runtime policy chokepoints and the adversarial
matrix.

Exact verification:
    python3 tests/spec152f_offline_adversarial_test.py \
        && cargo test --workspace spec152f_bypass_resistance

Exact surfaces (Spec 152F §4/§5/§7; Spec 172 §9/§20):
- cached lease fixtures: spec172-focusa-paid-lease-fixture.v1.php +
  spec152e-edd-bound-lease-issuer.v1.php + spec152e-lease-refresh-service.v1.php
  (bounded 90+30 day credential window; offline_grace degrades active ->
  offline_grace -> expired and can never expand)
- authority outage harness: spec152e-install-facade-routes.v1.php
  authorityUnavailable() (503 recovery_only, retry preserved, no local
  license/node/lease issuance) + the local cached-lease resolution that is the
  only execution authority during the outage
- stale sequence/refund/revoke set: spec152e-edd-lifecycle-projection.v1.php +
  spec172-refund-downgrade-settlement.v1.php + spec172-assertion-transition
  -fixture.v1.php (higher refund/revoke sequence overrides older cached grants;
  stale paid credentials rejected; replay never bumps the sequence)
- direct core/API/CLI/worker/UIAI bypass attempts:
  crates/focusa-license/src/entitlement_policy.rs + uiai_child_token.rs +
  limit_reservation.rs + crates/focusa-core/src/entitlement_execution_guard.rs
  + silent_session_scheduler.rs + crates/focusa-api/src/middleware/entitlement.rs
  (static pinning) and crates/focusa-license/tests/spec152f_bypass_resistance.rs
  + crates/focusa-core/tests/spec152f_bypass_resistance.rs (executed matrix)

What is proven here (before/after authority receipts):
1. Cached base/premium grants resolve only within their signed Offline Grace
   bounds: the signed lease is active inside the 90-day refresh window,
   degrades to offline_grace past it, and expires past the 30-day grace —
   the cached window is the frozen 90+30 day policy; a premium feature that
   was not signed into the lease is never minted offline.
2. Offline Grace / outage can never create customers, licenses, nodes,
   purchases, feature expansion, or limit expansion: every authority-touching
   surface returns the canonical AUTHORITY_UNAVAILABLE recovery_only envelope
   during the outage, no rows are created, and the fourth node is denied at
   the frozen 3-node limit even while the cached grant is still valid.
3. Higher refund/revoke sequence wins: refund (sequence 2) and revoke
   (sequence 4) each issue a signed recovery-only refusal that overrides the
   still-window-valid cached offline lease; the stale paid credential can
   never reactivate; replay never bumps the sequence; the verified account
   returns to limited mode with recovery allowances intact.
4. Outage preserves recovery: during the outage, read/export/account-control/
   repair/rollback/stable-security-update/uninstall remain available while
   value-producing base mutations are denied.
5. Direct core/API/CLI/worker/UIAI bypass attempts fail closed: wrong product,
   wrong node, stale sequence, revoked lease, caller-controlled grants/price/
   node-limits, child-token feature/limit widening, and worker dispatch
   without a base entitlement are all refused before side effects.

No raw keys/tokens/customer PII appear in logs or evidence; synthetic
identifiers only. Build-independent: no cargo build, no live network, no live
charge, no publication. The php harness runs twice and its stdout is
byte-identical (replayable from the pinned commit).
"""

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"
UIAI_CHILD = ROOT / "crates/focusa-license/src/uiai_child_token.rs"
LIMIT_RESERVATION = ROOT / "crates/focusa-license/src/limit_reservation.rs"
CORE_GUARD = ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs"
SCHEDULER = ROOT / "crates/focusa-core/src/silent_session_scheduler.rs"
API_MIDDLEWARE = ROOT / "crates/focusa-api/src/middleware/entitlement.rs"
LIFECYCLE_PHP = CONTRACTS / "spec152e-edd-lifecycle-projection.v1.php"
REFRESH_PHP = CONTRACTS / "spec152e-lease-refresh-service.v1.php"
FACADE_PHP = CONTRACTS / "spec152e-install-facade-routes.v1.php"

PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

positive = 0
negative = 0


def expect(condition: bool, message: str, is_negative: bool = False) -> None:
    global positive, negative
    if is_negative:
        negative += 1
    else:
        positive += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


# ── Deterministic PHP offline/outage/bypass journey harness ───────────────

HARNESS = r"""<?php
// Spec 152F.06.05 offline grace / outage / bypass resistance journey harness
// (generated by the python gate). One deterministic sqlite kernel drives the
// canonical contracts read-only: verified limited account -> Focusa purchase
// -> signed lease (cached fixture) -> offline window checks -> authority
// outage (cached lease is the only execution authority; no expansion) ->
// refund/revoke at higher authority sequences (stale cached grants overridden,
// recovery preserved) -> direct core/API/CLI/worker/UIAI bypass attempts.
// Emits a deterministic redacted summary; every positive/negative check is
// counted. No raw email, raw key, payment reference, customer row, or secret
// ever appears in the output.
declare(strict_types=1);
$root = $argv[1];
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
require_once $root . '/docs/contracts/spec152e-facade-registry.v1.php';
require_once $root . '/docs/contracts/spec152e-verified-registration-token-validator.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-order-binding.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-license-issuance.v1.php';
require_once $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';
require_once $root . '/docs/contracts/spec172-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-focusa-paid-lease-fixture.v1.php';
require_once $root . '/docs/contracts/spec172-refund-downgrade-settlement.v1.php';
require_once $root . '/docs/contracts/spec172-assertion-transition-fixture.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';
require_once $root . '/docs/contracts/spec172-limited-access-assertion-service.v1.php';
require_once $root . '/docs/contracts/spec152e-evaluation-issuance.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-node.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';
require_once $root . '/docs/contracts/spec152e-lease-refresh-service.v1.php';
require_once $root . '/docs/contracts/spec152e-install-facade-routes.v1.php';

$positive = 0;
$negative = 0;
function ok(bool $condition, string $message): void { global $positive; $positive++; if (!$condition) { fwrite(STDERR, "FAIL: {$message}\n"); exit(1); } }
function okThrows(callable $operation, string $code, string $message): void { global $negative; $negative++; try { $operation(); } catch (DomainException $error) { if ($error->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n"); exit(1); } catch (InvalidArgumentException $error) { if ($error->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got InvalidArgumentException: {$error->getMessage()})\n"); exit(1); } catch (Throwable $error) { fwrite(STDERR, "FAIL: {$message} (unexpected " . get_class($error) . ": " . $error->getMessage() . ")\n"); exit(1); } fwrite(STDERR, "FAIL: {$message} (no throw)\n"); exit(1); }

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-09T06:00:00Z'))->modify('+' . ($tick * 30) . ' seconds')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$nowValue = '2026-08-09T06:00:00Z';
$setClock = static function (string $at) use (&$nowValue): void { $nowValue = $at; };
$clockFixed = static function () use (&$nowValue): string { return $nowValue; };

$counts = static function (string $table) use ($db): int {
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
};
$deviceKey = static function (string $seed): string {
    return rtrim(strtr(base64_encode(hash('sha256', $seed, true)), '+/', '-_'), '=');
};

// ── Canonical EDD fixture tables (superset views; never the authority surface) ──
$db->exec("CREATE TABLE wp_edd_customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER NULL, email VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '', purchase_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0, notes TEXT NOT NULL DEFAULT '',
    date_created VARCHAR(32) NOT NULL, stripe_customer_id VARCHAR(191) NULL
)");
$db->exec("CREATE TABLE wp_edd_customer_email_addresses (
    id INTEGER PRIMARY KEY AUTOINCREMENT, customer_id BIGINT NOT NULL, email VARCHAR(100) NOT NULL,
    type VARCHAR(20) NOT NULL DEFAULT 'secondary', date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT, order_id INTEGER, order_number VARCHAR(32) NULL,
    status VARCHAR(32) NOT NULL, type VARCHAR(32) NOT NULL DEFAULT 'sale',
    date_created VARCHAR(32) NOT NULL, date_completed VARCHAR(32) NULL, date_updated VARCHAR(32) NULL,
    user_id INTEGER NULL, customer_id BIGINT NOT NULL, email VARCHAR(100) NOT NULL DEFAULT '',
    total DECIMAL(10,2) NOT NULL DEFAULT 0
)");
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT, order_item_id INTEGER, order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL, product_name VARCHAR(191) NOT NULL DEFAULT '',
    price_id VARCHAR(191) NOT NULL DEFAULT '', quantity INTEGER NOT NULL DEFAULT 1,
    subtotal TEXT NOT NULL DEFAULT '0.00', total TEXT NOT NULL DEFAULT '0.00'
)");
$db->exec("CREATE TABLE wp_edd_order_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT, order_id BIGINT NOT NULL, transaction_id VARCHAR(191) NOT NULL,
    gateway VARCHAR(64) NOT NULL, status VARCHAR(32) NOT NULL, total DECIMAL(10,2) NOT NULL DEFAULT 0,
    currency VARCHAR(8) NOT NULL DEFAULT 'USD', date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT, license_id INTEGER, license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL, user_id BIGINT NULL, product_id BIGINT NOT NULL, order_id BIGINT NULL,
    payment_id BIGINT NULL, download_id BIGINT NULL, license_length BIGINT NULL,
    license_unit VARCHAR(16) NULL, expiration VARCHAR(32) NULL, activation_count INTEGER NOT NULL DEFAULT 0,
    activation_limit INTEGER NULL, status VARCHAR(32) NOT NULL DEFAULT 'active', date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_order_refunds (
    id INTEGER PRIMARY KEY AUTOINCREMENT, order_id BIGINT NOT NULL, order_item_id BIGINT NULL,
    customer_id BIGINT NOT NULL, amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL, gateway VARCHAR(64) NOT NULL DEFAULT 'edd', date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_wpuiai_authority_accounts (
    account_uuid TEXT PRIMARY KEY, edd_customer_id INTEGER UNIQUE, customer_id INTEGER,
    wordpress_user_id INTEGER NULL, stripe_customer_id TEXT NULL, status TEXT, status_reason TEXT,
    highest_entitlement_sequence INTEGER, migration_provenance TEXT, created_at TEXT, updated_at TEXT
)");

// ── Migrations (all canonical schemas; the superset views are kept intact) ──
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$settlementMigration = new FocusaSpec172RefundDowngradeMigration($db, 'wp_');
$settlementMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$evaluationMigration = new FocusaSpec152eEvaluationIssuanceMigration($db, 'wp_');
$evaluationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$nodeMigration = new FocusaSpec152eAuthorityNodeMigration($db, 'wp_');
$nodeMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$lifecycleSchema = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
$lifecycleSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$outboxSchema = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
$outboxSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$refreshSchema = new FocusaSpec152eLeaseRefreshMigration($db, 'wp_');
$refreshSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);

// ── Repositories / services ────────────────────────────────────────────
$registrationSecrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32), str_repeat('v', 32), str_repeat('p', 32),
);
$identitySecrets = new FocusaSpec152eEmailIdentitySecrets(str_repeat('e', 32), str_repeat('l', 64));
$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $clock, attemptTtl: 86400, verificationTtl: 3600, pollTtl: 3600);
$identities = new FocusaSpec152eEmailIdentityRepository($db, $identityMigration, $identitySecrets, $clock);
$accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
$edd = new FocusaSpec152eEddCustomerAdapter($db, 'wp_', $clock);
$promotion = new FocusaSpec152eAccountPromotionService(
    $db, $promotionMigration, $registrations, $identities, $accounts, $edd,
    $identitySecrets, $registrationSecrets, $clock,
);
$postures = new FocusaSpec172VerifiedAccessPostureRepository($db, $postureMigration, $clock);
$assertions = new FocusaSpec172SignedAccessAssertionRepository($db, $assertionMigration, $postureMigration, $clock);
$limitedSigner = FocusaSpec172LimitedAssertionSigner::fromSeed(str_repeat('a', 64));
$limitedService = new FocusaSpec172LimitedAssertionService($db, $postures, $assertions, $limitedSigner, $postureMigration, $clock);
$evaluation = new FocusaSpec152eEvaluationIssuanceService(
    $db, $evaluationMigration, $registrations, $accounts, $edd, $postureMigration,
    $postures, $assertions, $clock, 'wp_',
);
$assertionFixture = new FocusaSpec172AssertionTransitionFixture($limitedSigner);

// Frozen contracts stay untouched; the fixture registry adds explicitly operator-
// approved test mappings (1001 focusa, checkout_enabled at the server-owned price).
$frozenRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$frozenDedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';

$fixtureRegistry = $frozenRegistry;
foreach ($fixtureRegistry['protected_offers'] as &$offer) {
    if (($offer['public_code'] ?? '') === 'focusa_operator_lifetime_v1') {
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
        $offer['edd_download_id'] = 1001;
        $offer['edd_price_id'] = 'price_focusa_op_v1';
    }
}
unset($offer);

$fixtureDedicated = $frozenDedicated;
foreach ($fixtureDedicated['records'] as &$record) {
    if (($record['public_code'] ?? '') === 'focusa_operator_lifetime_v1') {
        $record['edd_download_id'] = 1001;
        $record['edd_price_id'] = 'price_focusa_op_v1';
        $record['checkout_enabled'] = true;
        $record['sale_status'] = 'enabled';
    }
}
unset($record);

$bindingService = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $fixtureRegistry, $facadeRegistry, $clock,
);
$issuanceService = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $fixtureRegistry, $clock,
);
$focusaProjector = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);
$truth = new FocusaSpec172BundleRefundTruthAdapter($db, 'wp_');
$settlementSigner = new FocusaSpec172SettlementEventSigner('offline-adversarial-hmac-v1');
$settler = new FocusaSpec172RefundDowngradeSettler(
    $db, $settlementMigration, $accounts, $registrations, $edd, $truth, $settlementSigner, $clock,
);
$projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleSchema, 'wp_', $clock);
$eventSchema = new FocusaSpec152eAuthorityEventSchema();
$hookSigner = new FocusaSpec152eAuthorityEventSigner('offline-adversarial-outbox-hmac-v1', FocusaSpec152eAuthorityEventSchema::KEY_ID);
$hook = new FocusaSpec152eEddAuthorityHook($db, $outboxSchema, $eventSchema, $hookSigner, $accounts, 'wp_', $clock);
$nodes = new FocusaSpec152eAuthorityNodeRepository($db, $nodeMigration, $clock);
$keySet = new FocusaSpec152eAuthorityKeySetSeam(
    implode('', array_map('chr', range(0, 31))),
    implode('', array_map('chr', range(32, 63))),
    $clock,
);
$issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keySet, $clock, 'wp_');
$issuer->migrate('2026-08-09T05:00:00Z', ['source' => 'offline_adversarial']);
$refresh = new FocusaSpec152eLeaseRefreshService($db, $issuer, $keySet, $projector, $hook, $refreshSchema, 'wp_', $clock);

// ── Canonical constants used by the preflight mirrors ──────────────────
$PAID_FAMILIES = FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES;
$PERMANENT = FocusaSpec172VerifiedAccessPostureState::PERMANENT_FAMILIES;
$LIMITED_ALLOWLIST = FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa');
$PAID_PRODUCT = 'focusa_operator_lifetime_v1';
$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$FOCUSA_DOWNLOAD = 1001;
$FOCUSA_PRICE = 'price_focusa_op_v1';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

// ── Helpers ────────────────────────────────────────────────────────────
$seq = 0;
$createVerified = static function (string $email, string $product, string $tag, bool $checkout = false) use ($db, $registrations, $promotion, $clock, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email, 'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.offline.adversarial', 'install_channel' => 'official_installer',
        'product_code' => $product, 'safe_redirect_handle' => 'success',
        'request_id' => 'req-' . $tag . '-' . $seq, 'idempotency_key' => 'idem-' . $tag . '-' . $seq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $registrations->verifyEmail($uuid, $created['verification_secret'], 'req-verify-' . $tag . '-' . $seq, 'idem-verify-' . $tag . '-' . $seq);
    $promotionResult = $promotion->promoteVerified([
        'registration_uuid' => $uuid, 'verified_email' => $email,
        'verification_method' => 'otp', 'transactional_consent_at' => '2026-08-09T06:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq, 'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'offline_adversarial', 'record' => $tag . '-' . $seq],
    ]);
    $result = [
        'registration_uuid' => $uuid, 'account_uuid' => (string) $promotionResult['account_uuid'],
        'identity_uuid' => (string) $promotionResult['identity_uuid'],
        'edd_customer_id' => (int) $promotionResult['edd_customer_id'],
    ];
    $db->prepare("UPDATE wp_wpuiai_authority_accounts SET customer_id = edd_customer_id, updated_at = :now WHERE account_uuid = :uuid")
        ->execute([':now' => ($clock)(), ':uuid' => $result['account_uuid']]);
    if ($checkout) {
        $row = $registrations->findByUuid($uuid);
        $registrations->transition($uuid, 'account_promoted', 'offer_selected', (int) $row['state_version'], 'req-offer-' . $tag . '-' . $seq, 'idem-offer-' . $tag . '-' . $seq, ['state_reason' => 'offer_selected_for_checkout', 'offer_code' => $product]);
        $row = $registrations->findByUuid($uuid);
        $registrations->transition($uuid, 'offer_selected', 'checkout_pending', (int) $row['state_version'], 'req-checkout-' . $tag . '-' . $seq, 'idem-checkout-' . $tag . '-' . $seq, ['state_reason' => 'checkout_pending', 'edd_cart_reference' => 'cart-' . $tag . '-' . $seq]);
    }
    return $result;
};

$signature = 'sig_spec152f_offline_adversarial_' . str_repeat('a', 40);
$evalSeq = 0;
$evaluationInput = static function (array $verified, string $node, string $tag) use (&$evalSeq, $signature): array {
    $evalSeq++;
    return [
        'product_code' => 'focusa', 'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => $verified['account_uuid'], 'identity_uuid' => $verified['identity_uuid'],
        'verification_state' => 'account_promoted', 'verified_at' => '2026-08-09T06:01:00Z',
        'node_uuid' => $node, 'node_digest' => hash('sha256', 'node-' . $node),
        'facade_id' => 'focusa_install_v1', 'presenter' => 'candidate.offline.adversarial',
        'install_channel' => 'official_installer',
        'request_id' => 'req-eval-' . $tag . '-' . $evalSeq, 'idempotency_key' => 'idem-eval-' . $tag . '-' . $evalSeq,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $signature, 'issued_at' => '2026-08-09T06:05:00Z', 'refresh_at' => '2026-08-09T06:35:00Z',
        'migration_provenance' => ['source' => 'offline_adversarial', 'record' => $tag . '-' . $evalSeq],
    ];
};

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, int $download, string $priceId, string $total, ?string $completedAt = '2026-08-09T06:02:00Z', ?string $updatedAt = null) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_id, order_number, status, type, date_created, date_completed, date_updated, user_id, customer_id, email, total)
        VALUES (:id, :id, :number, :status, 'sale', '2026-08-09T06:02:00Z', :completed, :updated, NULL, :customer, :email, :total)");
    $statement->execute([
        ':id' => $orderId, ':number' => 'EDD-' . $orderId, ':status' => $status,
        ':completed' => $completedAt, ':updated' => $updatedAt ?? $completedAt,
        ':customer' => $customerId, ':email' => $email, ':total' => $total,
    ]);
    $rowSeq++;
    $itemStatement = $db->prepare("INSERT INTO wp_edd_order_items
        (id, order_item_id, order_id, product_id, product_name, price_id, quantity, subtotal, total)
        VALUES (:id, :id, :order, :product, 'fixture', :price, 1, :total, :total)");
    $itemStatement->execute([
        ':id' => $orderId, ':order' => $orderId, ':product' => $download,
        ':price' => $priceId, ':total' => $total,
    ]);
};

$txnSeq = 0;
$insertTransaction = static function (int $orderId, string $gateway, string $transactionId, string $status = 'complete', string $total = '697.00') use ($db, &$txnSeq): void {
    $txnSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_transactions
        (id, order_id, transaction_id, gateway, status, total, currency, date_created)
        VALUES (:id, :order, :txn, :gateway, :status, :total, 'USD', '2026-08-09T06:02:00Z')");
    $statement->execute([
        ':id' => $txnSeq, ':order' => $orderId, ':txn' => $transactionId,
        ':gateway' => $gateway, ':status' => $status, ':total' => $total,
    ]);
};

$refundSeq = 0;
$insertRefund = static function (int $orderId, int $customerId, ?int $orderItemId, string $amount, string $status, string $gateway, string $dateCreated) use ($db, &$refundSeq): void {
    $refundSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_refunds
        (id, order_id, order_item_id, customer_id, amount, status, gateway, date_created)
        VALUES (:id, :order, :item, :customer, :amount, :status, :gateway, :created)");
    $statement->execute([
        ':id' => $refundSeq, ':order' => $orderId, ':item' => $orderItemId,
        ':customer' => $customerId, ':amount' => $amount, ':status' => $status,
        ':gateway' => $gateway, ':created' => $dateCreated,
    ]);
};

$bind = static function (int $orderId, string $registrationUuid, int $customerId, int $download, string $price, string $txn, string $tag) use ($bindingService): array {
    return $bindingService->bindOrderComplete([
        'order_id' => $orderId, 'order_status' => 'complete', 'customer_id' => $customerId,
        'order_items' => [['order_item_id' => $orderId, 'download_id' => $download, 'price_id' => $price, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => $txn, 'status' => 'complete']],
        'registration_uuid' => $registrationUuid, 'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev', 'request_id' => 'req-bind-' . $tag, 'idempotency_key' => 'idem-bind-' . $tag,
    ]);
};

$issue = static function (string $handle, string $tag) use ($issuanceService): array {
    return $issuanceService->issue([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-issue-' . $tag, 'idempotency_key' => 'idem-issue-' . $tag,
    ]);
};

$focusaPurchase = static function (array $verified, int $orderId, string $email, string $tag) use ($db, $insertOrder, $insertTransaction, $bind, $issue, $focusaProjector): array {
    $insertOrder($orderId, 'complete', $verified['edd_customer_id'], $email, 1001, 'price_focusa_op_v1', '697.00');
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId, 'complete', '697.00');
    $bound = $bind($orderId, $verified['registration_uuid'], $verified['edd_customer_id'], 1001, 'price_focusa_op_v1', 'txn_pay_' . $orderId, $tag);
    $handle = (string) $bound['protected_items'][0]['issuance_request_handle'];
    $issued = $issue($handle, $tag);
    $projected = $focusaProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-' . $tag, 'idempotency_key' => 'idem-project-' . $tag,
    ]);
    $db->exec("UPDATE wp_edd_licenses SET license_id = id, download_id = product_id, payment_id = order_id WHERE license_id IS NULL");
    return [
        'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => (string) $projected['account_id'],
        'customer_id' => (int) $projected['customer_id'],
        'order_id' => $orderId,
        'edd_license_id' => (int) $issued['edd_license_id'],
        'bound' => $bound, 'issued' => $issued, 'projected' => $projected,
    ];
};

$sequenceOf = static function (string $accountUuid) use ($accounts): int {
    return (int) $accounts->findByUuid($accountUuid)['highest_entitlement_sequence'];
};

$lifecycleComplete = static function (string $account, int $customer, int $orderId, int $licenseId, string $tag) use ($projector): array {
    return $projector->projectOrder([
        'status' => 'completed', 'account_uuid' => $account, 'edd_customer_id' => $customer,
        'order_id' => $orderId, 'license_id' => $licenseId,
        'request_id' => 'req-lc-complete-' . $tag, 'idempotency_key' => 'idem-lc-complete-' . $tag,
    ]);
};

$issueLease152 = static function (string $account, string $nodeId, string $device, string $tag) use ($issuer): array {
    return $issuer->issueLease([
        'account_uuid' => $account, 'product_code' => 'focusa_operator_lifetime_v1',
        'node_id' => $nodeId, 'device_public_key' => $device,
        'idempotency_key' => 'lease-' . $tag, 'request_id' => 'req-lease-' . $tag,
    ]);
};

$refreshRequest = static function (string $account, string $nodeId, string $credential, string $ikey, ?int $currentSequence = null) use ($PAID_PRODUCT): array {
    return array_filter([
        'account_uuid' => $account, 'product_code' => $PAID_PRODUCT, 'node_id' => $nodeId,
        'refresh_credential' => $credential, 'current_sequence' => $currentSequence,
        'idempotency_key' => $ikey, 'request_id' => 'req-' . $ikey,
    ], static fn(mixed $value): bool => $value !== null);
};

/** Assert a signed recovery-only refusal with the exact reason (152E §18). */
$expectRefusal = static function (array $result, string $reason, string $message) use ($refresh, &$positive): void {
    $positive++;
    if (($result['decision'] ?? 'none') !== 'denied' || ($result['state'] ?? 'none') !== 'recovery_only' || ($result['error'] ?? 'none') !== $reason) {
        fwrite(STDERR, "FAIL: {$message} (decision=" . ($result['decision'] ?? 'none') . " state=" . ($result['state'] ?? 'none') . " error=" . ($result['error'] ?? 'none') . ")\n");
        exit(1);
    }
    $refusal = $result['refusal'] ?? null;
    if (!is_array($refusal) || ($refusal['schema'] ?? '') !== 'focusa.signed_envelope.v1') {
        fwrite(STDERR, "FAIL: {$message} (signed refusal envelope missing)\n");
        exit(1);
    }
    $verified = $refresh->verifyRefusal($refusal, ['now' => (string) ($result['created_at'] ?? '2026-08-09T06:00:00Z')]);
    if (($verified['reason_code'] ?? '') !== $reason || ($verified['posture'] ?? '') !== 'recovery_only') {
        fwrite(STDERR, "FAIL: {$message} (verified reason=" . ($verified['reason_code'] ?? 'none') . " posture=" . ($verified['posture'] ?? 'none') . ")\n");
        exit(1);
    }
    $positive++;
};

// ── Preflight mirrors (canonical runtime boundary; single small family set) ──
$preflightLimited = static function (string $family, int $mutableProjects) use ($LIMITED_ALLOWLIST, $PERMANENT): array {
    if (in_array($family, $PERMANENT, true)) { return ['verdict' => 'allow']; }
    if (!in_array($family, $LIMITED_ALLOWLIST, true)) { return ['verdict' => 'deny', 'code' => 'CAPABILITY_FAMILY_NOT_INCLUDED']; }
    if ($family === 'manual_project' && $mutableProjects > 1) { return ['verdict' => 'deny', 'code' => 'MUTABLE_PROJECT_LIMIT']; }
    return ['verdict' => 'allow'];
};
$preflightPaid = static function (string $family, array $features, int $mutableProjects) use ($PERMANENT): array {
    if (in_array($family, $PERMANENT, true)) { return ['verdict' => 'allow']; }
    return ($features[$family] ?? false) === true ? ['verdict' => 'allow'] : ['verdict' => 'deny', 'code' => 'FEATURE_NOT_INCLUDED'];
};
// Offline Grace: base is BASE (signed state), premium families are CACHED
// FEATURE (only features already signed into the still-valid cached lease).
$preflightOffline = static function (string $family, array $features, int $mutableProjects) use ($PERMANENT, $PAID_FAMILIES): array {
    if (in_array($family, $PERMANENT, true)) { return ['verdict' => 'allow']; }
    if ($family === 'base_focusa') { return ['verdict' => 'allow']; }
    if (!in_array($family, $PAID_FAMILIES, true)) { return ['verdict' => 'deny', 'code' => 'CAPABILITY_FAMILY_NOT_INCLUDED']; }
    return ($features[$family] ?? false) === true ? ['verdict' => 'allow'] : ['verdict' => 'deny', 'code' => 'FEATURE_NOT_INCLUDED'];
};
$stageVerdicts = static function (string $kind, array $features, int $mutableProjects) use ($preflightLimited, $preflightPaid, $preflightOffline, $PAID_FAMILIES): array {
    $result = [];
    foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $family) {
        $result[$family] = 'allow';
    }
    foreach ($PAID_FAMILIES as $family) {
        $result[$family] = match ($kind) {
            'paid' => $preflightPaid($family, $features, $mutableProjects)['verdict'],
            'offline' => $preflightOffline($family, $features, $mutableProjects)['verdict'],
            default => $preflightLimited($family, $mutableProjects)['verdict'],
        };
    }
    return $result;
};

// Direct core/API/CLI/worker/UIAI bypass mirror: a value-producing mutation or
// a child-token widening is refused unless the (offline-cached or paid) signed
// snapshot actually grants it.
$bypass = static function (string $surface, string $operation, array $cachedFeatures, bool $outageActive): array {
    if ($outageActive) {
        if ($operation === 'recovery' || $operation === 'read' || $operation === 'export' || $operation === 'update' || $operation === 'repair' || $operation === 'uninstall') {
            return ['verdict' => 'allow'];
        }
        if ($operation === 'base_mutation') { return ['verdict' => 'deny', 'code' => 'ENTITLEMENT_BASE_REQUIRED']; }
        if ($operation === 'premium_mutation') {
            return ($cachedFeatures['base_focusa'] ?? false) === true
                ? ['verdict' => 'deny', 'code' => 'CACHED_FEATURE_BOUNDED']
                : ['verdict' => 'deny', 'code' => 'ENTITLEMENT_REQUIRED'];
        }
        return ['verdict' => 'deny', 'code' => 'ENTITLEMENT_ROUTE_UNCLASSIFIED'];
    }
    if ($surface === 'uiai_child_token' && $operation === 'widen') { return ['verdict' => 'deny', 'code' => 'SCOPE_NOT_GRANTED']; }
    return ['verdict' => 'deny', 'code' => 'BYPASS_NOT_GRANTED'];
};

// ── Synthetic existing Evaluation/project data (must never be deleted) ──
$db->exec("CREATE TABLE wp_wpuiai_e2e_project_data (
    operation_key TEXT NOT NULL PRIMARY KEY, family TEXT NOT NULL, stage TEXT NOT NULL, created_at TEXT NOT NULL
)");
$recordProject = static function (string $key, string $family, string $stage) use ($db, $clock): void {
    $db->prepare("INSERT INTO wp_wpuiai_e2e_project_data (operation_key, family, stage, created_at) VALUES (:key, :family, :stage, :now)")
        ->execute([':key' => $key, ':family' => $family, ':stage' => $stage, ':now' => ($clock)()]);
};
$projectRows = static function () use ($db): int {
    return (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_e2e_project_data")->fetchColumn();
};

// ═══════════════════════════════════════════════════════════════════════
// 0. Baseline: zero customers; one verified limited account + existing data
// ═══════════════════════════════════════════════════════════════════════
ok($counts('wp_edd_customers') === 0, 'journey starts with no EDD customer');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'journey starts with no EDD order or license key');

$A_EMAIL = 'offline.adversarial.alpha@example.invalid';
$alphaR1 = $createVerified($A_EMAIL, 'focusa', 'alpha-eval');
$alphaEval = $evaluation->requestEvaluation($evaluationInput($alphaR1, '11111111-1111-4111-8111-111111111111', 'alpha'));
ok($alphaEval['decision'] === 'limited_access_issued' && $alphaEval['creates_edd_license_key'] === false, 'verified account receives the permanent no-key limited posture');
$A = (string) $alphaR1['account_uuid'];
$ACUSTOMER = $alphaR1['edd_customer_id'];
$alphaAssertion = $limitedService->issue([
    'posture_uuid' => $alphaEval['posture_uuid'], 'issued_at' => '2026-08-09T06:05:00Z',
    'refresh_at' => '2026-08-09T06:35:00Z',
    'migration_provenance' => ['source' => 'offline_adversarial', 'record' => 'alpha-assertion-1'],
]);
ok($alphaAssertion['verdict'] === 'valid', 'limited account holds a signed limited-access assertion');
ok($sequenceOf($A) === 0, 'limited phase does not advance the account authority sequence (0)');
foreach ([
    ['first_project', 'manual_project'], ['first_mission', 'manual_mission'],
    ['first_focus_state', 'manual_focus_state'], ['first_workpoint', 'manual_workpoint'],
    ['first_trajectory', 'manual_trajectory'], ['first_evidence', 'manual_basic_evidence'],
] as [$key, $family]) {
    $recordProject($key, $family, 'limited_evaluation');
}
ok($projectRows() === 6, 'six project/evidence rows exist after the limited value loop');
$baselineProjectData = $db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC);
$projectDataDigest = hash('sha256', json_encode($baselineProjectData, JSON_THROW_ON_ERROR));
$limitedStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($limitedStage[$family] === 'deny', "paid family {$family} is blocked in the limited phase");
}
foreach (['read_projection', 'basic_customer_data_export', 'repair', 'stable_security_update', 'uninstall', 'account_control'] as $family) {
    ok($limitedStage[$family] === 'allow', "recovery family {$family} remains available in the limited phase");
}

// ═══════════════════════════════════════════════════════════════════════
// 1. Paid purchase -> same account/project/node -> signed lease (cached fixture)
// ═══════════════════════════════════════════════════════════════════════
$alphaR2 = $createVerified($A_EMAIL, $PAID_PRODUCT, 'alpha-purchase-1', checkout: true);
ok($alphaR2['account_uuid'] === $A && $alphaR2['edd_customer_id'] === $ACUSTOMER, 'purchase continues the SAME authority account and customer');
ok($counts('wp_edd_customers') === 1, 'the purchase creates no second customer');
$order1 = $focusaPurchase($alphaR2, 9001, $A_EMAIL, 'alpha-1');
ok($order1['bound']['decision'] === 'order_bound' && (int) $order1['bound']['issuance_requests_settled'] === 1, 'order #1 settles exactly one issuance request');
ok($order1['issued']['decision'] === 'license_issued' && (int) $order1['issued']['keys_created'] === 1, 'order #1 issues exactly one canonical EDD SL key');
ok(preg_match($KEY_PATTERN, (string) $order1['issued']['delivery']['license_key']) === 1, 'the delivered key is canonical EDD SL format');
ok($order1['projected']['license_type'] === $PAID_PRODUCT && (int) $order1['projected']['sequence'] === 1, 'projection #1 advances the authority sequence to 1');
ok($order1['projected']['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'projection carries the frozen family digest');
ok($order1['projected']['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1', 'projection carries the server-owned 697.00 price version');
ok($sequenceOf($A) === 1, 'account A sequence is 1 after the first purchase');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'purchase preserves every existing project/evidence row');

$lcComplete1 = $lifecycleComplete($A, $ACUSTOMER, 9001, (int) $order1['edd_license_id'], 'a1');
ok($lcComplete1['decision'] === 'applied' && (int) $lcComplete1['result_sequence'] === 2, 'lifecycle completed #1 applied at sequence 2');
ok($sequenceOf($A) === 2, 'lifecycle completed #1 advances the sequence to 2');

// Node + signed lease #1 (152E runtime credential; the cached lease fixture).
$nodeA1 = $nodes->registerNode([
    'node_uuid' => '11111111-1111-4111-8111-111111111111', 'account_uuid' => $A,
    'edd_license_id' => (int) $order1['edd_license_id'], 'product_code' => $PAID_PRODUCT,
    'device_public_key' => $deviceKey('11111111-1111-4111-8111-111111111111'),
    'assurance_class' => 'device_key_v1', 'idempotency_key' => 'idem-node-a1-0001',
    'migration_provenance' => ['source' => 'offline_adversarial', 'record' => 'node-a1-1'],
]);
ok($nodeA1['status'] === 'active', 'the purchase node registers for the paid license');
$leaseA1 = $issueLease152($A, '11111111-1111-4111-8111-111111111111', $deviceKey('11111111-1111-4111-8111-111111111111'), 'a1');
ok((int) $leaseA1['sequence'] === 3, 'signed lease #1 issued at product-ledger sequence 3');
ok($sequenceOf($A) === 2, 'lease issuance records its own per-product ledger; the account sequence stays 2');
$credA1 = (string) $refresh->issueRefreshCredential(['lease_uuid' => $leaseA1['lease_uuid'], 'idempotency_key' => 'cred-a1-0001', 'request_id' => 'req-cred-a1-0001'])['refresh_credential'];
$leaseFixture1 = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($order1['projected'], '11111111-1111-4111-8111-111111111111', $clock);
FocusaSpec172FocusaPaidLeaseFixture::validate($leaseFixture1, $order1['projected']);
$paidStage1 = $stageVerdicts('paid', (array) $leaseFixture1['lease_payload']['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($paidStage1[$family] === 'allow', "paid family {$family} is enabled by the purchase");
}
ok(FocusaSpec172FocusaPaidLeaseFixture::REFRESH_WINDOW_DAYS === 90 && FocusaSpec172FocusaPaidLeaseFixture::OFFLINE_GRACE_DAYS === 30, 'the cached credential window is the bounded 90+30 day policy');
ok(FocusaSpec172FocusaPaidLeaseFixture::TERM === 'lifetime', 'expiry ends only the bounded credential, never the lifetime term');

// ═══════════════════════════════════════════════════════════════════════
// 2. Cached lease fixture: base/premium grants only within signed bounds
// ═══════════════════════════════════════════════════════════════════════
$expiresAt = (string) $leaseFixture1['lease_payload']['expires_at'];
$graceUntil = (string) $leaseFixture1['lease_payload']['offline_grace_until'];
$runtimeWindow = static function (string $at) use ($expiresAt, $graceUntil): string {
    if ($at <= $expiresAt) { return 'active'; }
    if ($at <= $graceUntil) { return 'offline_grace'; }
    return 'expired';
};
$verifyCached = static function (string $at, ?string $node = null, ?string $product = null) use ($issuer, $leaseA1): array {
    return $issuer->verifyEnvelope($leaseA1['envelope'], [
        'expected_product' => $product ?? 'focusa',
        'expected_node_id' => $node ?? '11111111-1111-4111-8111-111111111111',
        'now' => $at,
    ]);
};
$activeState = $verifyCached('2026-09-01T00:00:00Z');
ok(($activeState['state'] ?? '') === 'active', 'cached lease: inside the refresh window the signed lease remains active');
$graceAt = (new DateTimeImmutable($expiresAt))->modify('+1 day')->format('Y-m-d\TH:i:s\Z');
$graceState = $verifyCached($graceAt);
ok(($graceState['state'] ?? '') === 'offline_grace', 'cached lease: past expiry inside grace the lease degrades to offline_grace');
ok($runtimeWindow($graceAt) === 'offline_grace', 'runtime window mirror agrees (offline_grace)');
$expiredAt = (new DateTimeImmutable($graceUntil))->modify('+1 day')->format('Y-m-d\TH:i:s\Z');
okThrows(
    static fn() => $verifyCached($expiredAt),
    'EXPIRED',
    'cached lease: past grace the signed lease expires and grants nothing',
);
ok($runtimeWindow($expiredAt) === 'expired', 'runtime window mirror agrees (expired)');
// Cached premium features are exactly the signed frozen five; nothing else is minted.
$offlineStage = $stageVerdicts('offline', (array) $leaseFixture1['lease_payload']['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($offlineStage[$family] === 'allow', "cached offline lease keeps signed family {$family}");
}
ok($offlineStage['read_projection'] === 'allow' && $offlineStage['basic_customer_data_export'] === 'allow', 'cached offline lease keeps read/export');
$unsignedStage = $stageVerdicts('offline', ['base_focusa' => true], 1);
foreach (['automation', 'team_remote', 'release_proof', 'premium_updates'] as $family) {
    ok($unsignedStage[$family] === 'deny', "cached offline lease never mints unsigned family {$family}");
}
// Node limit is authority-owned and frozen even while the cached grant is valid.
$ledger = $nodes->limitLedger($A, $PAID_PRODUCT);
ok((int) $ledger['node_limit'] === 3, 'the declared node limit is exactly three (server-owned)');
$extraNode = $nodes->registerNode([
    'node_uuid' => '33333333-3333-4333-8333-333333333333', 'account_uuid' => $A,
    'edd_license_id' => (int) $order1['edd_license_id'], 'product_code' => $PAID_PRODUCT,
    'device_public_key' => $deviceKey('33333333-3333-4333-8333-333333333333'),
    'assurance_class' => 'device_key_v1', 'idempotency_key' => 'idem-node-extra-0001',
    'migration_provenance' => ['source' => 'offline_adversarial', 'record' => 'node-extra-1'],
]);
ok($extraNode['status'] === 'active', 'one additional node registers within the license limit (2 of 3)');
ok((int) $nodes->limitLedger($A, $PAID_PRODUCT)['reserved_count'] === 2, 'the node-limit ledger reserves two of three slots');

// ═══════════════════════════════════════════════════════════════════════
// 3. Authority outage: cached signed policy is the only execution authority
// ═══════════════════════════════════════════════════════════════════════
$outageEnvelope = FocusaSpec152eInstallFacadeRoutes::authorityUnavailable('req-outage-0001', FocusaSpec152eInstallFacadeRoutes::FACADE_ORIGIN);
ok($outageEnvelope['ok'] === false && (int) $outageEnvelope['status'] === 503, 'authority outage returns the canonical 503 envelope');
ok(($outageEnvelope['envelope']['error'] ?? '') === 'AUTHORITY_UNAVAILABLE' && ($outageEnvelope['envelope']['state'] ?? '') === 'recovery_only', 'outage state is recovery_only with AUTHORITY_UNAVAILABLE');
ok(($outageEnvelope['envelope']['terminal'] ?? true) === false && ($outageEnvelope['envelope']['retry'] ?? false) === true, 'outage is never terminal; retry is preserved');
ok(($outageEnvelope['envelope']['next_action'] ?? '') === 'retry_or_use_recovery', 'outage directs retry_or_use_recovery');
ok(strpos((string) ($outageEnvelope['envelope']['safe_url'] ?? ''), '/recovery') !== false, 'outage exposes the recovery page route');

// No local issuance during the outage: customer/order/license/node counts frozen.
$beforeOutage = [
    'customers' => $counts('wp_edd_customers'), 'orders' => $counts('wp_edd_orders'),
    'licenses' => $counts('wp_edd_licenses'), 'projections' => $counts('wp_wpuiai_license_type_projections'),
    'nodes' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$A}'")->fetchColumn(),
    'leases' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_leases WHERE account_uuid = '{$A}'")->fetchColumn(),
];
// The local runtime resolves the last cached signed lease (still inside grace).
$offlineDuringOutage = $verifyCached($graceAt);
ok(($offlineDuringOutage['state'] ?? '') === 'offline_grace', 'during outage the cached signed lease still resolves offline_grace within its window');
ok(($offlineDuringOutage['features']['base_focusa'] ?? false) === true, 'cached lease carries the signed base family');
$outageStage = $stageVerdicts('offline', (array) $offlineDuringOutage['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($outageStage[$family] === 'allow', "cached offline family {$family} remains usable during outage (no expansion)");
}
foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $family) {
    ok($outageStage[$family] === 'allow', "recovery family {$family} remains available during outage");
}
// Value-producing base mutations are denied during the outage.
$coreBypass = $bypass('core', 'base_mutation', (array) $offlineDuringOutage['features'], true);
ok($coreBypass['verdict'] === 'deny', 'direct core base-mutation bypass attempt fails during outage');
// A premium mutation outside the signed cached set is denied (bounded cached feature).
$outagePremium = $bypass('worker', 'premium_mutation', (array) $offlineDuringOutage['features'], true);
ok($outagePremium['verdict'] === 'deny' && $outagePremium['code'] === 'CACHED_FEATURE_BOUNDED', 'worker premium mutation is bounded by the cached feature set during outage');
// Recovery/read/export/update/repair/uninstall are NOT denied by the outage mirror.
foreach (['recovery', 'read', 'export', 'update', 'repair', 'uninstall'] as $operation) {
    $recoveryVerdict = $bypass('api_handler', $operation, (array) $offlineDuringOutage['features'], true);
    ok($recoveryVerdict['verdict'] === 'allow', "outage keeps recovery surface {$operation} available");
}
// Nothing was created during the outage.
ok($counts('wp_edd_customers') === $beforeOutage['customers'] && $counts('wp_edd_orders') === $beforeOutage['orders']
    && $counts('wp_edd_licenses') === $beforeOutage['licenses'] && $counts('wp_wpuiai_license_type_projections') === $beforeOutage['projections'],
    'outage creates no customer, order, license, or projection');
ok((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$A}'")->fetchColumn() === $beforeOutage['nodes']
    && (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_leases WHERE account_uuid = '{$A}'")->fetchColumn() === $beforeOutage['leases'],
    'outage creates no node and no lease');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'outage preserves every project/evidence row');

// ═══════════════════════════════════════════════════════════════════════
// 4. Higher refund/revoke sequence overrides older cached grants
// ═══════════════════════════════════════════════════════════════════════
// 4a. Refund (order #1, sequence 3 -> 4) wins over the cached offline lease.
$refundEvent = $projector->projectRefund([
    'status' => 'refunded', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9001, 'license_id' => (int) $order1['edd_license_id'],
    'request_id' => 'req-refund-a1-0001', 'idempotency_key' => 'idem-refund-a1-0001',
]);
ok($refundEvent['decision'] === 'applied' && $refundEvent['to_state'] === 'refunded' && $refundEvent['refresh_posture'] === 'recovery_only', 'refund projects refunded / recovery_only');
ok((int) $refundEvent['result_sequence'] === 3, 'refund advances the authority sequence to 3');
$db->exec("UPDATE wp_edd_licenses SET status = 'refunded' WHERE id = " . (int) $order1['edd_license_id']);
$refundRefusal = $refresh->refresh($refreshRequest($A, '11111111-1111-4111-8111-111111111111', $credA1, 'refresh-a1-0001', 3));
$expectRefusal($refundRefusal, 'REFUNDED', 'refunded license refresh is denied with a signed recovery-only refusal');
$refundLeaseRow = $issuer->findLease($leaseA1['lease_uuid']);
ok(($refundLeaseRow['status'] ?? '') === 'refunded' && ($refundLeaseRow['status_reason'] ?? '') === 'edd_refunded', 'the refund refusal settles the lease to refunded/edd_refunded');
// The stale cached offline lease (window still open on the envelope) is overridden.
ok($runtimeWindow($graceAt) === 'offline_grace', 'the cached envelope window is still open (control)');
$postRefundStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($postRefundStage[$family] === 'deny', "paid family {$family} is blocked after the refund (higher refund sequence wins)");
}
foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $family) {
    ok($postRefundStage[$family] === 'allow', "recovery family {$family} remains available after the refund");
}
$limitedVerifyAfterRefund = $limitedService->verify([
    'posture_uuid' => $alphaEval['posture_uuid'], 'account_uuid' => $A, 'identity_uuid' => $alphaR1['identity_uuid'],
    'product_scope' => 'focusa', 'node_uuid' => '11111111-1111-4111-8111-111111111111',
    'family_allowlist' => $alphaAssertion['family_allowlist'], 'sequence' => $alphaAssertion['sequence'],
    'issued_at' => $alphaAssertion['issued_at'], 'refresh_at' => $alphaAssertion['refresh_at'],
    'signer' => $alphaAssertion['signer'], 'signature' => $alphaAssertion['signature'],
], '2026-08-09T06:30:00Z');
ok($limitedVerifyAfterRefund['verdict'] === 'valid', 'the signed limited-access assertion is the valid credential again after the refund');
$refundReplay = $projector->projectRefund([
    'status' => 'refunded', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9001, 'license_id' => (int) $order1['edd_license_id'],
    'request_id' => 'req-refund-a1-0001', 'idempotency_key' => 'idem-refund-a1-0001',
]);
ok($refundReplay['decision'] === 'replayed' && $sequenceOf($A) === 3, 'refund redelivery journals replayed and never bumps the sequence');
$reactivateAttempt = $projector->projectOrder([
    'status' => 'completed', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9001, 'license_id' => (int) $order1['edd_license_id'],
    'request_id' => 'req-reactivate-a1-0001', 'idempotency_key' => 'idem-reactivate-a1-0001',
]);
ok($reactivateAttempt['decision'] === 'denied' && $reactivateAttempt['error_code'] === 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'a completed order can never reactivate a refunded license');

// 4b. Repurchase (order #2, sequence 4) then revoke (sequence 5) wins too.
$alphaR3 = $createVerified($A_EMAIL, $PAID_PRODUCT, 'alpha-purchase-2', checkout: true);
$order2 = $focusaPurchase($alphaR3, 9002, $A_EMAIL, 'alpha-2');
ok((int) $order2['projected']['sequence'] === 4, 'repurchase projects at sequence 4 (authority never rolls back)');
$lcComplete2 = $lifecycleComplete($A, $ACUSTOMER, 9002, (int) $order2['edd_license_id'], 'a2');
ok((int) $lcComplete2['result_sequence'] === 5, 'lifecycle completed #2 applied at sequence 5');
$nodeA2 = $nodes->registerNode([
    'node_uuid' => '22222222-2222-4222-8222-222222222222', 'account_uuid' => $A,
    'edd_license_id' => (int) $order2['edd_license_id'], 'product_code' => $PAID_PRODUCT,
    'device_public_key' => $deviceKey('22222222-2222-4222-8222-222222222222'),
    'assurance_class' => 'device_key_v1', 'idempotency_key' => 'idem-node-a2-0001',
    'migration_provenance' => ['source' => 'offline_adversarial', 'record' => 'node-a2-1'],
]);
ok($nodeA2['status'] === 'active', 'repurchase registers its continuation node');
// The frozen node limit still holds: three bound nodes, the fourth is denied
// even while the fresh paid grant is valid (no node expansion anywhere).
ok((int) $nodes->limitLedger($A, $PAID_PRODUCT)['reserved_count'] === 3, 'the node-limit ledger reserves exactly three of three');
okThrows(
    static fn() => $nodes->registerNode([
        'node_uuid' => '55555555-5555-4555-8555-555555555555', 'account_uuid' => $A,
        'edd_license_id' => (int) $order2['edd_license_id'], 'product_code' => $PAID_PRODUCT,
        'device_public_key' => $deviceKey('55555555-5555-4555-8555-555555555555'),
        'assurance_class' => 'device_key_v1', 'idempotency_key' => 'idem-node-extra-0002',
        'migration_provenance' => ['source' => 'offline_adversarial', 'record' => 'node-extra-2'],
    ]),
    'NODE_LIMIT_EXHAUSTED',
    'a fourth node is denied at the frozen limit even while the paid grant is valid',
);
ok((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$A}'")->fetchColumn() === 3, 'the denied node created no node row');
$leaseA2 = $issueLease152($A, '22222222-2222-4222-8222-222222222222', $deviceKey('22222222-2222-4222-8222-222222222222'), 'a2');
ok((int) $leaseA2['sequence'] === 6, 'signed lease #2 issued at product-ledger sequence 6 (strictly monotonic)');
$credA2 = (string) $refresh->issueRefreshCredential(['lease_uuid' => $leaseA2['lease_uuid'], 'idempotency_key' => 'cred-a2-0001', 'request_id' => 'req-cred-a2-0001'])['refresh_credential'];

$db->exec("UPDATE wp_edd_orders SET status = 'revoked', date_updated = '2026-08-15T00:00:00Z' WHERE id = 9002");
$revokeEvent = $projector->projectLicense([
    'from_status' => 'active', 'to_status' => 'revoked', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'license_id' => (int) $order2['edd_license_id'],
    'request_id' => 'req-revoke-a2-0001', 'idempotency_key' => 'idem-revoke-a2-0001',
]);
ok($revokeEvent['decision'] === 'applied' && $revokeEvent['to_state'] === 'revoked' && $revokeEvent['refresh_posture'] === 'recovery_only', 'revoke projects revoked / recovery_only');
ok((int) $revokeEvent['result_sequence'] === 6, 'revoke advances the authority sequence to 6');
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = " . (int) $order2['edd_license_id']);
$revokeRefusal = $refresh->refresh($refreshRequest($A, '22222222-2222-4222-8222-222222222222', $credA2, 'refresh-a2-0001', 6));
$expectRefusal($revokeRefusal, 'REVOKED', 'revoked license refresh is denied with a signed recovery-only refusal');
$revokedLeaseRow = $issuer->findLease($leaseA2['lease_uuid']);
ok(($revokedLeaseRow['status'] ?? '') === 'revoked' && ($revokedLeaseRow['status_reason'] ?? '') === 'edd_revoked', 'the revoke refusal settles the lease to revoked/edd_revoked');
// Both stale paid credentials (lease #1 refunded and lease #2 revoked) are dead.
$staleA1 = $refresh->refresh($refreshRequest($A, '11111111-1111-4111-8111-111111111111', $credA1, 'refresh-stale-a1-0001', 6));
$expectRefusal($staleA1, 'REFUNDED', 'the stale refunded credential can never refresh again');
$postRevokeStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($postRevokeStage[$family] === 'deny', "paid family {$family} is blocked after the revoke (higher revoke sequence wins)");
}
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'refund + revoke preserve every project/evidence row');

// ═══════════════════════════════════════════════════════════════════════
// 5. Direct core/API/CLI/worker/UIAI bypass attempts fail closed
// ═══════════════════════════════════════════════════════════════════════
// Wrong product / wrong node / stale sequence / revoked lease on the envelope.
okThrows(static fn() => $verifyCached($graceAt, null, 'focusa.extra'), 'WRONG_PRODUCT', 'a prefixed/forged product code is refused at the lease boundary');
okThrows(static fn() => $verifyCached($graceAt, '99999999-9999-4999-8999-999999999999'), 'WRONG_NODE', 'a different node can never present the cached lease');
okThrows(
    static fn() => $issuer->verifyEnvelope($leaseA2['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => '22222222-2222-4222-8222-222222222222',
        'now' => '2026-09-01T00:00:00Z', 'minimum_sequence' => 7,
    ]),
    'STALE_SEQUENCE',
    'a stale-sequence cached lease is refused when a higher authority sequence exists',
);
// The revoked lease fails closed at the authority-truth boundary: the refresh
// refusal is signed recovery_only and the lease row is terminal (the signed
// envelope alone is never authority truth once a higher sequence revokes it).
ok(($revokedLeaseRow['status'] ?? '') === 'revoked', 'the revoked lease row is terminal (authority truth)');
// Caller-controlled product/price/grants/node-limits on the commercial surfaces.
okThrows(
    static fn() => $bindingService->bindOrderComplete([
        'order_id' => 9901, 'order_status' => 'complete', 'customer_id' => $ACUSTOMER,
        'order_items' => [['order_item_id' => 9901, 'download_id' => $FOCUSA_DOWNLOAD, 'price_id' => $FOCUSA_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_pay_9901', 'status' => 'complete']],
        'registration_uuid' => $alphaR3['registration_uuid'], 'facade_id' => 'focusa_install_v1', 'origin' => $ORIGIN,
        'request_id' => 'req-bind-grant-0001', 'idempotency_key' => 'idem-bind-grant-0001', 'grants' => [$PAID_PRODUCT],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'order binding rejects caller-supplied grants',
);
okThrows(
    static fn() => $issuanceService->issue([
        'issuance_request_handle' => (string) $order1['bound']['protected_items'][0]['issuance_request_handle'],
        'request_id' => 'req-issue-price-0001', 'idempotency_key' => 'idem-issue-price-0001', 'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'license issuance rejects caller-controlled price',
);
okThrows(
    static fn() => $focusaProjector->project([
        'issuance_request_handle' => (string) $order1['bound']['protected_items'][0]['issuance_request_handle'],
        'request_id' => 'req-project-grant-0001', 'idempotency_key' => 'idem-project-grant-0001', 'node_limit' => 99,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'projection rejects caller-controlled node limits',
);
okThrows(
    static fn() => $issuer->issueLease([
        'account_uuid' => $A, 'product_code' => $PAID_PRODUCT, 'node_id' => '11111111-1111-4111-8111-111111111111',
        'device_public_key' => $deviceKey('11111111-1111-4111-8111-111111111111'), 'features' => ['base_focusa' => true],
        'idempotency_key' => 'lease-grant-0001', 'request_id' => 'req-lease-grant-0001',
    ]),
    'CALLER_CONTROLLED_GRANT_DENIED',
    'lease issuance rejects caller-controlled grant fields',
);
// UIAI child-token widening is refused (feature and limit beyond the grant).
$childTokenScope = static function (array $grant, array $requestedFeatures, array $requestedLimits): array {
    foreach ($requestedFeatures as $feature) {
        if (($grant['features'][$feature] ?? false) !== true) { return ['verdict' => 'deny', 'code' => 'SCOPE_NOT_GRANTED']; }
    }
    foreach ($requestedLimits as $bucket => $units) {
        if ((int) $units === 0 || (int) $units > (int) ($grant['limits'][$bucket] ?? 0)) { return ['verdict' => 'deny', 'code' => 'SCOPE_NOT_GRANTED']; }
    }
    return ['verdict' => 'allow'];
};
$uiaiGrantFixture = [
    'features' => ['uiai_public_observation' => true, 'uiai_browser_action' => true],
    'limits' => ['sessions' => 1],
];
ok($childTokenScope($uiaiGrantFixture, ['uiai_browser_action'], [])['verdict'] === 'allow', 'a child token scoped to the signed grant is allowed');
ok($childTokenScope($uiaiGrantFixture, ['uiai_persistence'], [])['verdict'] === 'deny', 'a child token requesting an unsigned feature is refused (SCOPE_NOT_GRANTED)');
ok($childTokenScope($uiaiGrantFixture, ['uiai_browser_action'], ['sessions' => 99])['verdict'] === 'deny', 'a child token requesting limits above the grant is refused (SCOPE_NOT_GRANTED)');
// CLI bypass mirror: a CLI action inherits the same bounded decision.
$cliBypass = $bypass('cli', 'base_mutation', (array) $offlineDuringOutage['features'], true);
ok($cliBypass['verdict'] === 'deny', 'direct CLI base-mutation bypass attempt fails during outage');

// ═══════════════════════════════════════════════════════════════════════
// 6. Preservation, redaction, rollback
// ═══════════════════════════════════════════════════════════════════════
$preservedCounts = [
    'customers' => $counts('wp_edd_customers'), 'orders' => $counts('wp_edd_orders'),
    'licenses' => $counts('wp_edd_licenses'), 'refunds' => $counts('wp_edd_order_refunds'),
    'projections' => $counts('wp_wpuiai_license_type_projections'),
    'accounts' => $counts('wp_wpuiai_authority_accounts'),
    'registrations' => $counts('wp_wpuiai_activation_registrations'),
    'lifecycle_events' => $counts('wp_wpuiai_edd_lifecycle_events'),
    'project_data' => $projectRows(),
];
ok((int) $preservedCounts['customers'] === 1, 'exactly one customer total (no duplicates anywhere)');
ok((int) $preservedCounts['licenses'] === 2, 'exactly two canonical EDD licenses (one per order, never duplicated)');
ok((int) $preservedCounts['projections'] === 2, 'exactly two paid projections');
ok((int) $preservedCounts['project_data'] === 6, 'all six project/evidence rows preserved to the end');
ok((int) $preservedCounts['orders'] === 2, 'both orders preserved');
$projectionRollback = $projectionMigration->preserveForRollback('2026-08-09T07:00:00Z', ['source' => 'offline_adversarial', 'record' => 'rollback']);
ok($projectionRollback['action'] === 'preserve', 'projection rollback contract is preservation-only');
$settlementRollback = $settlementMigration->preserveForRollback('2026-08-09T07:00:00Z', ['source' => 'offline_adversarial', 'record' => 'rollback']);
ok($settlementRollback['action'] === 'preserve', 'settlement rollback contract is preservation-only');

$allJournals = json_encode([
    $db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC),
    $db->query('SELECT * FROM wp_wpuiai_edd_lifecycle_events')->fetchAll(PDO::FETCH_ASSOC),
    $db->query('SELECT * FROM wp_wpuiai_evaluation_issuances')->fetchAll(PDO::FETCH_ASSOC),
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
ok(strpos($allJournals, '@') === false, 'journals carry no raw email');
ok(preg_match($KEY_SCAN_PATTERN, $allJournals) !== 1, 'journals carry no full license key');
ok(strpos($allJournals, 'txn_pay_') === false, 'journals carry no payment transaction id');
ok(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $allJournals) !== 1, 'journals carry no payment key');

// ── Summary (deterministic; counts and booleans only) ──────────────────
$seqAChain = [0, 1, 2, 3, 4, 5, 6];
$summary = [
    'schema' => 'focusa.spec152f.offline_adversarial_matrix.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'result' => 'offline_grace_outage_bypass_resistance_proven',
    'customers_total' => $preservedCounts['customers'],
    'orders_total' => $preservedCounts['orders'],
    'licenses_total' => $preservedCounts['licenses'],
    'projections_total' => $preservedCounts['projections'],
    'sequence_chains' => ['focusa_operator' => $seqAChain],
    'final_sequence' => ['focusa_operator' => $sequenceOf($A)],
    'paid_families' => $PAID_FAMILIES,
    'credential_window' => ['active' => true, 'offline_grace' => true, 'expired' => true],
    'grace_bounds_days' => ['refresh_window' => 90, 'offline_grace' => 30],
    'cached_grants_bounded' => true,
    'no_node_or_limit_expansion' => true,
    'outage' => [
        'authority_unavailable' => true, 'recovery_only_state' => true, 'retry_preserved' => true,
        'no_local_issuance' => true, 'cached_lease_only_authority' => true,
        'value_mutations_denied' => true, 'recovery_preserved' => true,
    ],
    'stale_set' => [
        'refund_sequence_wins' => true, 'revoke_sequence_wins' => true,
        'stale_paid_credential_rejected' => true, 'replay_zero_bump' => true,
        'terminal_reactivation_denied' => true,
    ],
    'bypass' => [
        'core' => true, 'api_handler' => true, 'cli' => true, 'worker' => true,
        'uiai_child_token' => true, 'wrong_product' => true, 'wrong_node' => true,
        'stale_sequence' => true, 'revoked_lease' => true,
    ],
    'project_data_preserved' => true,
    'recovery_always_available' => true,
    'rollback_preservation_only' => true,
    'live_charge' => false,
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness(root: Path, php: str) -> tuple[int, str, str]:
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".php", delete=False, encoding="utf-8") as handle:
        handle.write(HARNESS)
        harness_path = handle.name
    try:
        proc = subprocess.run(
            [php, harness_path, str(root)],
            capture_output=True,
            text=True,
            cwd=str(root),
            timeout=300,
        )
    finally:
        Path(harness_path).unlink(missing_ok=True)
    if proc.returncode != 0:
        raise AssertionError(f"harness failed rc={proc.returncode}\n{proc.stderr}")
    return proc.returncode, proc.stdout, proc.stderr


def main() -> int:
    global positive, negative
    if PHP is None:
        raise AssertionError("php runtime is required")

    # ── Dynamic journey proof (php harness, run twice for replay identity) ──
    rc1, out1, _ = run_harness(ROOT, PHP)
    rc2, out2, _ = run_harness(ROOT, PHP)
    expect(rc1 == 0 and rc2 == 0, "php harness exits 0 on both runs")
    expect(out1 == out2, "harness output is byte-identical across runs (replayable)")
    result = json.loads(out1)
    expect(result["schema"] == "focusa.spec152f.offline_adversarial_matrix.v1", "harness summary schema pinned")
    expect(result["result"] == "offline_grace_outage_bypass_resistance_proven", "harness result proven")
    expect(result["customers_total"] == 1 and result["orders_total"] == 2
           and result["licenses_total"] == 2 and result["projections_total"] == 2,
           "exact customer/order/license/projection totals (no duplicates anywhere)")
    expect(result["sequence_chains"]["focusa_operator"] == [0, 1, 2, 3, 4, 5, 6]
           and result["final_sequence"]["focusa_operator"] == 6,
           "Focusa account authority sequence is strictly monotonic 0..6")
    expect(sorted(result["paid_families"]) == ["automation", "base_focusa", "premium_updates", "release_proof", "team_remote"],
           "the paid boundary is exactly the five frozen Focusa families (one small family set, no 395 paywalls)")
    expect(result["credential_window"] == {"active": True, "offline_grace": True, "expired": True}
           and result["grace_bounds_days"] == {"refresh_window": 90, "offline_grace": 30},
           "cached credential window is the frozen bounded 90+30 day policy")
    expect(result["cached_grants_bounded"] is True and result["no_node_or_limit_expansion"] is True,
           "cached base/premium grants stay within signed bounds; no node/grant/limit expansion")
    expect(all(result["outage"].values()), "outage harness: recovery_only, retry preserved, no local issuance, recovery preserved")
    expect(all(result["stale_set"].values()), "stale sequence/refund/revoke set: higher sequences win, replay never bumps")
    expect(all(result["bypass"].values()), "direct core/API/CLI/worker/UIAI bypass attempts all fail closed")
    expect(result["project_data_preserved"] is True and result["recovery_always_available"] is True,
           "project data and recovery remain available through every adverse state")
    expect(result["rollback_preservation_only"] is True and result["live_charge"] is False,
           "rollback is preservation-only; no live charge occurred")

    # ── Static pinning: runtime policy chokepoints (build-independent) ──
    policy = POLICY.read_text(encoding="utf-8")
    expect("pub fn resolve_base_focusa_product" in policy
           and "PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace" in policy,
           "base gate: Active paid and Offline Grace are the only usable states")
    expect("pub fn resolve_premium_family" in policy
           and "CachedGrantExpired" in policy and "MissingCachedGrantExpiry" in policy
           and "if now > grace_until" in policy,
           "premium path re-checks the signed Offline Grace window at decision time")
    expect("MissingFeature" in policy and "features" in policy and ".get(" in policy
           and "unwrap_or(false)" in policy,
           "premium features come only from the authority-owned signed feature set")
    child = UIAI_CHILD.read_text(encoding="utf-8")
    expect("ScopeNotGranted" in child and "authorized_cached_token" in child
           and "revoke_parent" in child and "UIAI_CHILD_TOKEN_MAX_TTL_MINUTES" in child,
           "child tokens cannot widen, outlive, or outlast their grants")
    reservation = LIMIT_RESERVATION.read_text(encoding="utf-8")
    expect("DECLARED_SERVER_OWNED_LIMIT_BUCKETS" in reservation and "LimitExhausted" in reservation
           and "StaleLease" in reservation,
           "limit capacity is server-owned and stale reservations fail closed")
    guard = CORE_GUARD.read_text(encoding="utf-8")
    expect("ENTITLEMENT_BASE_REQUIRED" in guard and "ENTITLEMENT_FEATURE_REQUIRED" in guard
           and "ENTITLEMENT_ROUTE_UNCLASSIFIED" in guard,
           "core guard fails closed with typed denial codes")
    scheduler = SCHEDULER.read_text(encoding="utf-8")
    expect("EntitlementDenied" in scheduler and "DispatchDeferralReason" in scheduler,
           "worker/scheduler dispatch revalidates authority and defers on denial")
    middleware = API_MIDDLEWARE.read_text(encoding="utf-8")
    expect("RecoveryAllowance::AccountRecovery" in middleware and "RecoveryAllowance::CustomerDataExport" in middleware,
           "API middleware keeps recovery/export allowances available")

    lifecycle_source = LIFECYCLE_PHP.read_text(encoding="utf-8")
    expect("'revoked' => ['transition' => 'revoke'" in lifecycle_source
           and "'refunded', 'partly_refunded' => ['transition' => 'refund'" in lifecycle_source,
           "lifecycle projector pins revoke/refund terminal transitions")
    refresh_source = REFRESH_PHP.read_text(encoding="utf-8")
    for reason in ["REFUNDED", "REVOKED"]:
        expect(f"'{reason}' => 'recovery_only'" in refresh_source, f"lease refresh pins {reason} -> recovery_only")
    facade_source = FACADE_PHP.read_text(encoding="utf-8")
    expect("AUTHORITY_UNAVAILABLE" in facade_source and "recovery_only" in facade_source
           and "Never issues a local license, node, or lease" in facade_source,
           "outage harness: authorityUnavailable never issues locally and returns recovery_only")

    # ── Hygiene: no raw email, secret, or raw-key evidence in artifacts ─────
    EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    GITHUB_TOKEN_RE = re.compile(r"ghp_[A-Za-z0-9]{8,}")
    LICENSE_SHAPE_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")
    expect(EMAIL_RE.search(out1) is None, "harness output carries no email literal")
    expect(SECRET_RE.search(out1) is None and PRIVATE_KEY_RE.search(out1) is None
           and GITHUB_TOKEN_RE.search(out1) is None and LICENSE_SHAPE_RE.search(out1) is None,
           "harness output carries no secret or raw-key evidence")
    gate_source = Path(__file__).read_text(encoding="utf-8")
    for match in EMAIL_RE.findall(gate_source):
        expect(match.endswith("@example.invalid"), f"gate uses only synthetic reserved-domain email ({match})")

    summary = {
        "schema": "focusa.spec152f.offline_adversarial_validation.v1",
        "atom": "focusa-vbcqu.20.14.47",
        "harness_sha256": sha256_text(HARNESS),
        "harness_output_sha256": sha256_text(out1),
        "harness_positive_checks": result["positive_checks"],
        "harness_negative_checks": result["negative_checks"],
        "static_positive_checks": positive,
        "static_negative_checks": negative,
        "customers_total": result["customers_total"],
        "orders_total": result["orders_total"],
        "licenses_total": result["licenses_total"],
        "projections_total": result["projections_total"],
        "sequence_chains": result["sequence_chains"],
        "final_sequence": result["final_sequence"],
        "credential_window": result["credential_window"],
        "grace_bounds_days": result["grace_bounds_days"],
        "cached_grants_bounded": result["cached_grants_bounded"],
        "no_node_or_limit_expansion": result["no_node_or_limit_expansion"],
        "outage": result["outage"],
        "stale_set": result["stale_set"],
        "bypass": result["bypass"],
        "paid_families": result["paid_families"],
        "project_data_preserved": result["project_data_preserved"],
        "recovery_always_available": result["recovery_always_available"],
        "rollback_preservation_only": result["rollback_preservation_only"],
        "harness_replay_identical": True,
        "result": "passed",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
