#!/usr/bin/env python3
"""Spec 152F.06.04 — Prove paid continuation and adverse lifecycle (E2E).

Atom focusa-vbcqu.20.14.46 (152F.06.04): purchase continues the same verified
account/project/node without reinstall, then expiry, refund, revoke, renewal,
and node removal are exercised on that same account. Payment enables the
base/purchased families; every adverse state removes value capability without
deleting customer data and without rolling the authority sequence backward;
a still-mailbox-verified account returns to `verified_no_license` limited
mode with read/export/recovery/repair/stable-security-update/uninstall intact.

Binding Spec 172 overlay: prove the upgrade from a verified limited posture to
the Focusa Operator and the Focusa+UIAI Operator Bundle while preserving data;
adverse lifecycle returns verified users to limited mode.

Exact verification:
    python3 tests/spec152f_paid_lifecycle_e2e_test.py

Exact surfaces (Spec 152F §2/§4/§9; Spec 172 §9/§17/§21; 152E §17/§23):
- branded EDD purchase: docs/contracts/spec152e-edd-order-binding.v1.php +
  spec152e-edd-license-issuance.v1.php + spec172-edd-license-type-projector
  .v1.php + spec172-bundle-edd-license-type-projector.v1.php +
  spec172-edd-operator-v1-downloads.v1.php (server-owned fixture mappings only;
  frozen contracts are read-only and stay checkout-disabled)
- canonical customer/order/key/node/lease:
  spec152e-edd-customer-adapter.v1.php + spec152e-activation-registration
  .v1.php + spec152e-account-promotion.v1.php + spec152e-authority-account
  .v1.php + spec152e-authority-node.v1.php + spec172-focusa-paid-lease-fixture
  .v1.php + spec172-bundle-signed-lease-fixture.v1.php +
  spec152e-edd-bound-lease-issuer.v1.php
- existing Evaluation/project data: spec152e-evaluation-issuance.v1.php +
  spec172-verified-access-posture.v1.php + spec172-signed-access-assertion
  .v1.php + spec172-limited-access-assertion-service.v1.php (verified_no_license
  posture, signed limited assertion, first-value project/evidence rows)
- refund/revoke/expiry transitions: spec152e-edd-lifecycle-projection.v1.php +
  spec152e-lease-refresh-service.v1.php + spec172-refund-downgrade-settlement
  .v1.php + spec172-assertion-transition-fixture.v1.php (terminal states,
  signed recovery-only refusals, whole-order Bundle refund/revoke returning the
  verified account to limited mode, stale paid credentials rejected)
- base Focusa product/family resolution (static cross-check):
  crates/focusa-license/src/entitlement_policy.rs +
  crates/focusa-core/src/license.rs +
  docs/contracts/spec152f-entitlement-policy.v1.yaml +
  docs/contracts/spec172-verified-limited-access.v1.yaml +
  docs/contracts/spec152f-denial-ux-catalog.v1.json

What is proven here (before/after authority receipts):
1. Zero-customer baseline; one verified limited account reaches the first
   useful value (6 manual project/evidence rows) with a signed limited
   assertion; account sequence 0.
2. Paid continuation: the SAME account/customer/node buys Focusa through the
   branded EDD purchase (offer_selected -> checkout_pending -> bound order ->
   one canonical EDD SL key -> focusa_operator_lifetime_v1 projection).
   No new customer, no reinstall, no duplicate key; the projection advances the
   authority sequence 0 -> 1; the paid lease fixture carries the frozen five
   families (base_focusa, automation, team_remote, release_proof,
   premium_updates); the limited-mode paid-family denials flip to allow;
   the 6 project/evidence rows are byte-preserved; an Evaluation retry is
   refused with PAID_POSTURE_PRESERVED (paid posture is never downgraded).
3. Expiry: the bounded credential window degrades active -> offline_grace ->
   expired while the lifetime entitlement is preserved (the projection stays
   active and renewal issues a replacement credential); the EDD license expiry
   projection is applied at sequence 4 with refresh denied by a signed
   recovery-only EXPIRED refusal.
4. Renewal: order #2 issues a new canonical key + projection at sequence 6 and
   a new signed lease; the old lease is superseded (never rolled back).
5. Node removal: three concurrent nodes bind within the license limit, an
   explicit deactivation releases the slot exactly once, the freed slot is
   re-reserved/settled; project data is preserved.
6. Refund: order #2 refunds -> applied at sequence 8, refresh denied with a
   signed REFUNDED recovery-only refusal; the verified account returns to
   limited mode (paid families blocked again, recovery allowances intact,
   project data preserved, stale paid credential rejected).
7. Repurchase: order #3 binds a fresh key + projection at sequence 10 and paid
   families are enabled again — authority never rolls backward.
8. Bundle continuation: a second verified account goes limited -> Bundle order
   #1 (one key, exact union of the two Operator grants) -> whole-order refund
   (both grants revoked together, sequence 2, verified_no_license limited
   posture with frozen limited families and permanent allowances, stale paid
   assertion rejected) -> Bundle order #2 renewal (sequence 3, paid again) ->
   revoke (sequence 4, limited again).
9. No caller-controlled product/price/grants on any surface
   (CLIENT_COMMERCIAL_FIELDS_FORBIDDEN / CALLER_CONTROLLED_GRANT_DENIED);
   no 395 paywalls (one small family boundary); no raw email/key/token/card in
   journals, decisions, or the summary; rollback is preservation-only.

Build-independent: no cargo build, no live network, no live charge, no
publication. The php harness runs twice and its stdout is byte-identical
(replayable from the pinned commit).
"""

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
POLICY_YAML = CONTRACTS / "spec152f-entitlement-policy.v1.yaml"
LIMITED_YAML = CONTRACTS / "spec172-verified-limited-access.v1.yaml"
DENIAL_CATALOG = CONTRACTS / "spec152f-denial-ux-catalog.v1.json"
POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"
CORE_LICENSE = ROOT / "crates/focusa-core/src/license.rs"
SETTLEMENT_PHP = CONTRACTS / "spec172-refund-downgrade-settlement.v1.php"
TRANSITION_PHP = CONTRACTS / "spec172-assertion-transition-fixture.v1.php"
PROJECTOR_PHP = CONTRACTS / "spec172-edd-license-type-projector.v1.php"
BUNDLE_PROJECTOR_PHP = CONTRACTS / "spec172-bundle-edd-license-type-projector.v1.php"
LIFECYCLE_PHP = CONTRACTS / "spec152e-edd-lifecycle-projection.v1.php"
REFRESH_PHP = CONTRACTS / "spec152e-lease-refresh-service.v1.php"

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


# ── Deterministic PHP journey harness ───────────────────────────────────────

HARNESS = r"""<?php
// Spec 152F.06.04 paid continuation and adverse lifecycle journey harness
// (generated by the python gate). One deterministic sqlite kernel drives the
// canonical contracts read-only: verified limited account -> Focusa purchase
// (same account/project/node) -> expiry -> renewal -> node removal -> refund ->
// repurchase, plus a Bundle account (purchase -> refund -> renewal -> revoke).
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
require_once $root . '/docs/contracts/spec172-uiai-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php';
require_once $root . '/docs/contracts/spec172-bundle-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-bundle-signed-lease-fixture.v1.php';
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
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NULL,
    email VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '',
    purchase_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    date_created VARCHAR(32) NOT NULL,
    stripe_customer_id VARCHAR(191) NULL
)");
$db->exec("CREATE TABLE wp_edd_customer_email_addresses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL,
    type VARCHAR(20) NOT NULL DEFAULT 'secondary',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER,
    order_number VARCHAR(32) NULL,
    status VARCHAR(32) NOT NULL,
    type VARCHAR(32) NOT NULL DEFAULT 'sale',
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL,
    date_updated VARCHAR(32) NULL,
    user_id INTEGER NULL,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL DEFAULT '',
    total DECIMAL(10,2) NOT NULL DEFAULT 0
)");
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_item_id INTEGER,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    product_name VARCHAR(191) NOT NULL DEFAULT '',
    price_id VARCHAR(191) NOT NULL DEFAULT '',
    quantity INTEGER NOT NULL DEFAULT 1,
    subtotal TEXT NOT NULL DEFAULT '0.00',
    total TEXT NOT NULL DEFAULT '0.00'
)");
$db->exec("CREATE TABLE wp_edd_order_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    transaction_id VARCHAR(191) NOT NULL,
    gateway VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL,
    total DECIMAL(10,2) NOT NULL DEFAULT 0,
    currency VARCHAR(8) NOT NULL DEFAULT 'USD',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_id INTEGER,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    user_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    payment_id BIGINT NULL,
    download_id BIGINT NULL,
    license_length BIGINT NULL,
    license_unit VARCHAR(16) NULL,
    expiration VARCHAR(32) NULL,
    activation_count INTEGER NOT NULL DEFAULT 0,
    activation_limit INTEGER NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_order_refunds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    order_item_id BIGINT NULL,
    customer_id BIGINT NOT NULL,
    amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL,
    gateway VARCHAR(64) NOT NULL DEFAULT 'edd',
    date_created VARCHAR(32) NOT NULL
)");
// Authority account superset view: canonical account repository columns plus the
// customer_id alias the 152E lease issuer resolves (customer_id mirrors the EDD
// customer id in this EDD-backed model; the promotion path maintains it).
$db->exec("CREATE TABLE wp_wpuiai_authority_accounts (
    account_uuid TEXT PRIMARY KEY,
    edd_customer_id INTEGER UNIQUE,
    customer_id INTEGER,
    wordpress_user_id INTEGER NULL,
    stripe_customer_id TEXT NULL,
    status TEXT,
    status_reason TEXT,
    highest_entitlement_sequence INTEGER,
    migration_provenance TEXT,
    created_at TEXT,
    updated_at TEXT
)");

// ── Migrations (all canonical schemas; the superset views are kept intact) ──
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$settlementMigration = new FocusaSpec172RefundDowngradeMigration($db, 'wp_');
$settlementMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$evaluationMigration = new FocusaSpec152eEvaluationIssuanceMigration($db, 'wp_');
$evaluationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$nodeMigration = new FocusaSpec152eAuthorityNodeMigration($db, 'wp_');
$nodeMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$lifecycleSchema = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
$lifecycleSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$outboxSchema = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
$outboxSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$refreshSchema = new FocusaSpec152eLeaseRefreshMigration($db, 'wp_');
$refreshSchema->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);

// ── Repositories / services ────────────────────────────────────────────
$registrationSecrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);
$identitySecrets = new FocusaSpec152eEmailIdentitySecrets(
    str_repeat('e', 32),
    str_repeat('l', 64),
);
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
// approved test mappings (1001 focusa / 1002 uiai / 1003 bundle, checkout_enabled
// at the server-owned prices) so the positive paid matrix runs against the same
// single authority without mutating the frozen contracts.
$frozenRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$frozenDedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';

$fixtureRegistry = $frozenRegistry;
foreach ($fixtureRegistry['protected_offers'] as &$offer) {
    if (in_array((string) ($offer['public_code'] ?? ''), ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1'], true)) {
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
        $offer['edd_download_id'] = match ($offer['public_code']) {
            'focusa_operator_lifetime_v1' => 1001,
            'uiai_operator_lifetime_v1' => 1002,
            default => 1003,
        };
        $offer['edd_price_id'] = match ($offer['public_code']) {
            'focusa_operator_lifetime_v1' => 'price_focusa_op_v1',
            'uiai_operator_lifetime_v1' => 'price_uiai_op_v1',
            default => 'price_bundle_op_v1',
        };
    }
}
unset($offer);

$fixtureDedicated = $frozenDedicated;
foreach ($fixtureDedicated['records'] as &$record) {
    if (in_array((string) ($record['public_code'] ?? ''), ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1'], true)) {
        $record['edd_download_id'] = match ($record['public_code']) {
            'focusa_operator_lifetime_v1' => 1001,
            'uiai_operator_lifetime_v1' => 1002,
            default => 1003,
        };
        $record['edd_price_id'] = match ($record['public_code']) {
            'focusa_operator_lifetime_v1' => 'price_focusa_op_v1',
            'uiai_operator_lifetime_v1' => 'price_uiai_op_v1',
            default => 'price_bundle_op_v1',
        };
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
$bundleAdapter = new FocusaSpec172BundleOrderSlAdapter($bindingService, $issuanceService, $fixtureDedicated);
$bundleProjector = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);
$truth = new FocusaSpec172BundleRefundTruthAdapter($db, 'wp_');
$settlementSigner = new FocusaSpec172SettlementEventSigner('paid-lifecycle-e2e-hmac-v1');
$settler = new FocusaSpec172RefundDowngradeSettler(
    $db, $settlementMigration, $accounts, $registrations, $edd, $truth, $settlementSigner, $clock,
);

$projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleSchema, 'wp_', $clock);
$eventSchema = new FocusaSpec152eAuthorityEventSchema();
$hookSigner = new FocusaSpec152eAuthorityEventSigner('paid-lifecycle-e2e-outbox-hmac-v1', FocusaSpec152eAuthorityEventSchema::KEY_ID);
$hook = new FocusaSpec152eEddAuthorityHook($db, $outboxSchema, $eventSchema, $hookSigner, $accounts, 'wp_', $clock);
$nodes = new FocusaSpec152eAuthorityNodeRepository($db, $nodeMigration, $clock);
$keySet = new FocusaSpec152eAuthorityKeySetSeam(
    implode('', array_map('chr', range(0, 31))),
    implode('', array_map('chr', range(32, 63))),
    $clock,
);
$issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keySet, $clock, 'wp_');
$issuer->migrate('2026-08-09T05:00:00Z', ['source' => 'paid_lifecycle_e2e']);
$refresh = new FocusaSpec152eLeaseRefreshService($db, $issuer, $keySet, $projector, $hook, $refreshSchema, 'wp_', $clock);

// ── Canonical constants used by the preflight mirrors ──────────────────
$PAID_FAMILIES = FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES;
$PERMANENT = FocusaSpec172VerifiedAccessPostureState::PERMANENT_FAMILIES;
$LIMITED_ALLOWLIST = FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa');
$PAID_PRODUCT = 'focusa_operator_lifetime_v1';
$BUNDLE_PRODUCT = 'focusa_uiai_operator_bundle_lifetime_v1';
$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$FOCUSA_DOWNLOAD = 1001;
$BUNDLE_DOWNLOAD = 1003;
$FOCUSA_PRICE = 'price_focusa_op_v1';
$BUNDLE_PRICE = 'price_bundle_op_v1';
$GATEWAY = 'stripe';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

// ── Helpers ────────────────────────────────────────────────────────────
$seq = 0;
$createVerified = static function (string $email, string $product, string $tag, bool $checkout = false) use ($db, $registrations, $promotion, $clock, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.paid.lifecycle.e2e',
        'install_channel' => 'official_installer',
        'product_code' => $product,
        'safe_redirect_handle' => 'success',
        'request_id' => 'req-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-' . $tag . '-' . $seq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $seq,
        'idem-verify-' . $tag . '-' . $seq,
    );
    $promotionResult = $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-09T06:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => $tag . '-' . $seq],
    ]);
    $result = [
        'registration_uuid' => $uuid,
        'verified_at' => $verified['registration']['verified_at'],
        'account_uuid' => (string) $promotionResult['account_uuid'],
        'identity_uuid' => (string) $promotionResult['identity_uuid'],
        'edd_customer_id' => (int) $promotionResult['edd_customer_id'],
    ];
    // Superset-view maintenance: the 152E lease issuer resolves customer_id.
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

$signature = 'sig_spec152f_paid_lifecycle_' . str_repeat('a', 40);
$evalSeq = 0;
$evaluationInput = static function (array $verified, string $node, string $tag) use (&$evalSeq, $signature): array {
    $evalSeq++;
    return [
        'product_code' => 'focusa',
        'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => $verified['account_uuid'],
        'identity_uuid' => $verified['identity_uuid'],
        'verification_state' => 'account_promoted',
        'verified_at' => $verified['verified_at'],
        'node_uuid' => $node,
        'node_digest' => hash('sha256', 'node-' . $node),
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.paid.lifecycle.e2e',
        'install_channel' => 'official_installer',
        'request_id' => 'req-eval-' . $tag . '-' . $evalSeq,
        'idempotency_key' => 'idem-eval-' . $tag . '-' . $evalSeq,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $signature,
        'issued_at' => '2026-08-09T06:05:00Z',
        'refresh_at' => '2026-08-09T06:35:00Z',
        'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => $tag . '-' . $evalSeq],
    ];
};

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, int $download, string $priceId, string $total, ?string $completedAt = '2026-08-09T06:02:00Z', ?string $updatedAt = null) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_id, order_number, status, type, date_created, date_completed, date_updated, user_id, customer_id, email, total)
        VALUES (:id, :id, :number, :status, 'sale', '2026-08-09T06:02:00Z', :completed, :updated, NULL, :customer, :email, :total)");
    $statement->execute([
        ':id' => $orderId,
        ':number' => 'EDD-' . $orderId,
        ':status' => $status,
        ':completed' => $completedAt,
        ':updated' => $updatedAt ?? $completedAt,
        ':customer' => $customerId,
        ':email' => $email,
        ':total' => $total,
    ]);
    $rowSeq++;
    $itemStatement = $db->prepare("INSERT INTO wp_edd_order_items
        (id, order_item_id, order_id, product_id, product_name, price_id, quantity, subtotal, total)
        VALUES (:id, :id, :order, :product, 'fixture', :price, 1, :total, :total)");
    $itemStatement->execute([
        ':id' => $orderId,
        ':order' => $orderId,
        ':product' => $download,
        ':price' => $priceId,
        ':total' => $total,
    ]);
};

$txnSeq = 0;
$insertTransaction = static function (int $orderId, string $gateway, string $transactionId, string $status = 'complete', string $total = '697.00') use ($db, &$txnSeq): void {
    $txnSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_transactions
        (id, order_id, transaction_id, gateway, status, total, currency, date_created)
        VALUES (:id, :order, :txn, :gateway, :status, :total, 'USD', '2026-08-09T06:02:00Z')");
    $statement->execute([
        ':id' => $txnSeq,
        ':order' => $orderId,
        ':txn' => $transactionId,
        ':gateway' => $gateway,
        ':status' => $status,
        ':total' => $total,
    ]);
};

$refundSeq = 0;
$insertRefund = static function (int $orderId, int $customerId, ?int $orderItemId, string $amount, string $status, string $gateway, string $dateCreated) use ($db, &$refundSeq): void {
    $refundSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_refunds
        (id, order_id, order_item_id, customer_id, amount, status, gateway, date_created)
        VALUES (:id, :order, :item, :customer, :amount, :status, :gateway, :created)");
    $statement->execute([
        ':id' => $refundSeq,
        ':order' => $orderId,
        ':item' => $orderItemId,
        ':customer' => $customerId,
        ':amount' => $amount,
        ':status' => $status,
        ':gateway' => $gateway,
        ':created' => $dateCreated,
    ]);
};

$bind = static function (int $orderId, string $registrationUuid, int $customerId, int $download, string $price, string $txn, string $tag) use ($bindingService): array {
    return $bindingService->bindOrderComplete([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => [['order_item_id' => $orderId, 'download_id' => $download, 'price_id' => $price, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => $txn, 'status' => 'complete']],
        'registration_uuid' => $registrationUuid,
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'request_id' => 'req-bind-' . $tag,
        'idempotency_key' => 'idem-bind-' . $tag,
    ]);
};

$issue = static function (string $handle, string $tag) use ($issuanceService): array {
    return $issuanceService->issue([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-issue-' . $tag,
        'idempotency_key' => 'idem-issue-' . $tag,
    ]);
};

/** One canonical Focusa purchase chain: bind -> issue -> project. */
$focusaPurchase = static function (array $verified, int $orderId, string $email, int $download, string $price, string $tag) use ($db, $insertOrder, $insertTransaction, $bind, $issue, $focusaProjector): array {
    $total = $price === 'price_bundle_op_v1' ? '1254.60' : '697.00';
    $insertOrder($orderId, 'complete', $verified['edd_customer_id'], $email, $download, $price, $total);
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId, 'complete', $total);
    $bound = $bind($orderId, $verified['registration_uuid'], $verified['edd_customer_id'], $download, $price, 'txn_pay_' . $orderId, $tag);
    $handle = (string) $bound['protected_items'][0]['issuance_request_handle'];
    $issued = $issue($handle, $tag);
    $projected = $focusaProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-' . $tag,
        'idempotency_key' => 'idem-project-' . $tag,
    ]);
    // Superset-view maintenance: the 152E lease issuer resolves license_id,
    // download_id, and payment_id from the canonical EDD license row.
    $db->exec("UPDATE wp_edd_licenses SET license_id = id, download_id = product_id, payment_id = order_id WHERE license_id IS NULL");
    return [
        'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => (string) $projected['account_id'],
        'customer_id' => (int) $projected['customer_id'],
        'order_id' => $orderId,
        'edd_license_id' => (int) $issued['edd_license_id'],
        'bound' => $bound,
        'issued' => $issued,
        'projected' => $projected,
    ];
};

/** One canonical Bundle purchase chain through the adapter + composite projector. */
$bundlePurchase = static function (array $verified, int $orderId, string $email, string $tag) use ($db, $insertOrder, $insertTransaction, $bundleAdapter, $bundleProjector): array {
    $insertOrder($orderId, 'complete', $verified['edd_customer_id'], $email, 1003, 'price_bundle_op_v1', '1254.60');
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId, 'complete', '1254.60');
    $bound = $bundleAdapter->bindAndIssue([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $verified['edd_customer_id'],
        'order_items' => [['order_item_id' => $orderId, 'download_id' => 1003, 'price_id' => 'price_bundle_op_v1', 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_pay_' . $orderId, 'status' => 'complete']],
        'registration_uuid' => $verified['registration_uuid'],
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'request_id' => 'req-bind-bundle-' . $tag,
        'idempotency_key' => 'idem-bind-bundle-' . $tag,
    ]);
    $handle = (string) $bound['issuance_request_handle'];
    $projected = $bundleProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-bundle-' . $tag,
        'idempotency_key' => 'idem-project-bundle-' . $tag,
    ]);
    $db->exec("UPDATE wp_edd_licenses SET license_id = id, download_id = product_id, payment_id = order_id WHERE license_id IS NULL");
    return [
        'account_uuid' => (string) $projected['account_id'],
        'customer_id' => (int) $projected['customer_id'],
        'order_id' => $orderId,
        'edd_license_id' => (int) $bound['edd_license_id'],
        'handle' => $handle,
        'bound' => $bound,
        'projected' => $projected,
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
    if (in_array($family, $PERMANENT, true)) {
        return ['verdict' => 'allow'];
    }
    if (!in_array($family, $LIMITED_ALLOWLIST, true)) {
        return ['verdict' => 'deny', 'code' => 'CAPABILITY_FAMILY_NOT_INCLUDED'];
    }
    if ($family === 'manual_project' && $mutableProjects > 1) {
        return ['verdict' => 'deny', 'code' => 'MUTABLE_PROJECT_LIMIT'];
    }
    return ['verdict' => 'allow'];
};
$preflightPaid = static function (string $family, array $features, int $mutableProjects) use ($PERMANENT): array {
    if (in_array($family, $PERMANENT, true)) {
        return ['verdict' => 'allow'];
    }
    return ($features[$family] ?? false) === true ? ['verdict' => 'allow'] : ['verdict' => 'deny', 'code' => 'FEATURE_NOT_INCLUDED'];
};
$stageVerdicts = static function (string $kind, array $features, int $mutableProjects) use ($preflightLimited, $preflightPaid, $PAID_FAMILIES): array {
    $result = [];
    foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $family) {
        $result[$family] = 'allow';
    }
    foreach ($PAID_FAMILIES as $family) {
        $result[$family] = $kind === 'paid'
            ? $preflightPaid($family, $features, $mutableProjects)['verdict']
            : $preflightLimited($family, $mutableProjects)['verdict'];
    }
    return $result;
};

// ── Synthetic existing Evaluation/project data (must never be deleted) ──
$db->exec("CREATE TABLE wp_wpuiai_e2e_project_data (
    operation_key TEXT NOT NULL PRIMARY KEY,
    family TEXT NOT NULL,
    stage TEXT NOT NULL,
    created_at TEXT NOT NULL
)");
$db->exec("CREATE TABLE wp_wpuiai_e2e_paid_work_log (
    family TEXT NOT NULL PRIMARY KEY,
    recorded_at TEXT NOT NULL
)");
$recordProject = static function (string $key, string $family, string $stage) use ($db, $clock): void {
    $db->prepare("INSERT INTO wp_wpuiai_e2e_project_data (operation_key, family, stage, created_at) VALUES (:key, :family, :stage, :now)")
        ->execute([':key' => $key, ':family' => $family, ':stage' => $stage, ':now' => ($clock)()]);
};
$projectRows = static function () use ($db): int {
    return (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_e2e_project_data")->fetchColumn();
};

// ═══════════════════════════════════════════════════════════════════════
// 0. Zero-customer baseline + verified limited account reaches first value
// ═══════════════════════════════════════════════════════════════════════
ok($counts('wp_edd_customers') === 0, 'journey starts with no EDD customer');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'journey starts with no EDD order or license key');
ok($counts('wp_wpuiai_verified_access_postures') === 0, 'journey starts with no posture');

// Account A: verified limited user with existing Evaluation/project data.
$A_EMAIL = 'paid.continuation.alpha@example.invalid';
$alphaR1 = $createVerified($A_EMAIL, 'focusa', 'alpha-eval');
$alphaEval = $evaluation->requestEvaluation($evaluationInput($alphaR1, '11111111-1111-4111-8111-111111111111', 'alpha'));
ok($alphaEval['decision'] === 'limited_access_issued', 'verified account receives the limited Evaluation posture');
ok($alphaEval['duration'] === 'no_automatic_expiry' && $alphaEval['creates_edd_license_key'] === false, 'Evaluation is a permanent no-key limited posture');
$A = (string) $alphaR1['account_uuid'];
$ACUSTOMER = $alphaR1['edd_customer_id'];
$alphaAssertion = $limitedService->issue([
    'posture_uuid' => $alphaEval['posture_uuid'],
    'issued_at' => '2026-08-09T06:05:00Z',
    'refresh_at' => '2026-08-09T06:35:00Z',
    'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'alpha-assertion-1'],
]);
ok($alphaAssertion['verdict'] === 'valid', 'limited account holds a signed limited-access assertion');
ok($sequenceOf($A) === 0, 'limited phase does not advance the account authority sequence (0)');
// First useful value loop (existing Evaluation/project data).
foreach ([
    ['first_project', 'manual_project'], ['first_mission', 'manual_mission'],
    ['first_focus_state', 'manual_focus_state'], ['first_workpoint', 'manual_workpoint'],
    ['first_trajectory', 'manual_trajectory'], ['first_evidence', 'manual_basic_evidence'],
] as [$key, $family]) {
    ok($preflightLimited($family, 1)['verdict'] === 'allow', "limited phase permits first-value family {$family}");
    $recordProject($key, $family, 'limited_evaluation');
}
ok($projectRows() === 6, 'six project/evidence rows exist after the limited value loop');
$baselineProjectData = $db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC);
$limitedStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($limitedStage[$family] === 'deny', "paid family {$family} is blocked in the limited phase");
}
foreach (['read_projection', 'basic_customer_data_export', 'repair', 'stable_security_update', 'uninstall', 'account_control'] as $family) {
    ok($limitedStage[$family] === 'allow', "recovery family {$family} remains available in the limited phase");
}
$projectDataDigest = hash('sha256', json_encode($baselineProjectData, JSON_THROW_ON_ERROR));

// ═══════════════════════════════════════════════════════════════════════
// 1. Paid continuation: same account/project/node buys Focusa (no reinstall)
// ═══════════════════════════════════════════════════════════════════════
$alphaR2 = $createVerified($A_EMAIL, $PAID_PRODUCT, 'alpha-purchase-1', checkout: true);
ok($alphaR2['account_uuid'] === $A && $alphaR2['edd_customer_id'] === $ACUSTOMER, 'purchase journey continues the SAME authority account and customer');
ok($counts('wp_edd_customers') === 1, 'the purchase creates no second customer');
$order1 = $focusaPurchase($alphaR2, 9001, $A_EMAIL, $FOCUSA_DOWNLOAD, $FOCUSA_PRICE, 'alpha-1');
ok($order1['bound']['decision'] === 'order_bound' && (int) $order1['bound']['issuance_requests_settled'] === 1, 'order #1 settles exactly one issuance request');
ok($order1['issued']['decision'] === 'license_issued' && (int) $order1['issued']['keys_created'] === 1, 'order #1 issues exactly one canonical EDD SL key');
ok(preg_match($KEY_PATTERN, (string) $order1['issued']['delivery']['license_key']) === 1, 'the delivered key is canonical EDD SL format');
ok($order1['projected']['decision'] === 'license_type_projected' && $order1['projected']['license_type'] === $PAID_PRODUCT, 'order #1 projects focusa_operator_lifetime_v1');
ok((int) $order1['projected']['sequence'] === 1, 'projection #1 advances the authority sequence to 1');
ok($order1['projected']['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'projection carries the frozen family digest');
ok($order1['projected']['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1', 'projection carries the server-owned 697.00 price version');
ok($sequenceOf($A) === 1, 'account A sequence is 1 after the first purchase');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'purchase preserves every existing project/evidence row (no reinstall wipe)');

// Paid lease fixture: bounded lifetime credential carrying the frozen five families.
$leaseFixture1 = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($order1['projected'], '11111111-1111-4111-8111-111111111111', $clock);
FocusaSpec172FocusaPaidLeaseFixture::validate($leaseFixture1, $order1['projected']);
$leaseFeatures1 = (array) $leaseFixture1['lease_payload']['features'];
$paidStage1 = $stageVerdicts('paid', $leaseFeatures1, 1);
foreach ($PAID_FAMILIES as $family) {
    ok($paidStage1[$family] === 'allow', "paid family {$family} is enabled by the purchase");
}
foreach (['read_projection', 'basic_customer_data_export', 'repair', 'stable_security_update', 'uninstall'] as $family) {
    ok($paidStage1[$family] === 'allow', "recovery family {$family} remains available while paid");
}
// Payment enables base/purchased families without 395 paywalls: one family set.
ok(count($PAID_FAMILIES) === 5, 'the paid boundary is exactly five families (no per-surface paywalls)');
// A paid customer is never downgraded to the limited Evaluation posture.
okThrows(
    static fn() => $evaluation->requestEvaluation($evaluationInput($alphaR1, '11111111-1111-4111-8111-111111111111', 'alpha-retry')),
    'PAID_POSTURE_PRESERVED',
    'an Evaluation retry after purchase preserves the paid posture',
);

// ═══════════════════════════════════════════════════════════════════════
// 2. Expiry: bounded credential window + licensed-state expiry -> limited mode
// ═══════════════════════════════════════════════════════════════════════
$expiresAt = (string) $leaseFixture1['lease_payload']['expires_at'];
$graceUntil = (string) $leaseFixture1['lease_payload']['offline_grace_until'];
$runtimeWindow = static function (string $at) use ($expiresAt, $graceUntil): string {
    if ($at <= $expiresAt) {
        return 'active';
    }
    if ($at <= $graceUntil) {
        return 'offline_grace';
    }
    return 'expired';
};
ok($runtimeWindow($leaseFixture1['lease_payload']['issued_at']) === 'active', 'credential is active inside the refresh window');
ok($runtimeWindow((new DateTimeImmutable($expiresAt))->modify('+1 day')->format('Y-m-d\TH:i:s\Z')) === 'offline_grace', 'credential degrades to offline_grace past the refresh window');
ok($runtimeWindow((new DateTimeImmutable($graceUntil))->modify('+1 day')->format('Y-m-d\TH:i:s\Z')) === 'expired', 'credential expires past the offline grace');
ok(FocusaSpec172FocusaPaidLeaseFixture::REFRESH_WINDOW_DAYS === 90 && FocusaSpec172FocusaPaidLeaseFixture::OFFLINE_GRACE_DAYS === 30, 'the credential window is the bounded 90+30 day policy');
ok(FocusaSpec172FocusaPaidLeaseFixture::TERM === 'lifetime', 'expiry ends only the bounded credential, never the lifetime term');

// Licensed-state expiry through the 152E lifecycle projector (same account).
$license1Id = $order1['edd_license_id'];
$lcComplete1 = $lifecycleComplete($A, $ACUSTOMER, 9001, $license1Id, 'a1');
ok($lcComplete1['decision'] === 'applied' && (int) $lcComplete1['result_sequence'] === 2, 'lifecycle completed #1 applied at sequence 2');
ok($sequenceOf($A) === 2, 'lifecycle completed #1 advances the sequence to 2');
// Node + signed lease for order #1 (152E runtime credential).
$nodeA1 = $nodes->registerNode([
    'node_uuid' => '11111111-1111-4111-8111-111111111111',
    'account_uuid' => $A,
    'edd_license_id' => $license1Id,
    'product_code' => $PAID_PRODUCT,
    'device_public_key' => $deviceKey('11111111-1111-4111-8111-111111111111'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-node-a1-0001',
    'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'node-a1-1'],
]);
ok($nodeA1['status'] === 'active', 'the purchase node registers for the paid license');
$leaseA1 = $issueLease152($A, '11111111-1111-4111-8111-111111111111', $deviceKey('11111111-1111-4111-8111-111111111111'), 'a1');
ok((int) $leaseA1['sequence'] === 3, 'signed lease #1 issued at product-ledger sequence 3');
ok($sequenceOf($A) === 2, 'lease issuance records its own per-product ledger; the account sequence stays 2');
$credA1 = (string) $refresh->issueRefreshCredential(['lease_uuid' => $leaseA1['lease_uuid'], 'idempotency_key' => 'cred-a1-0001', 'request_id' => 'req-cred-a1-0001'])['refresh_credential'];
$offlineActive = $issuer->verifyEnvelope($leaseA1['envelope'], [
    'expected_product' => 'focusa', 'expected_node_id' => '11111111-1111-4111-8111-111111111111',
    'now' => '2026-10-01T00:00:00Z',
]);
ok(($offlineActive['state'] ?? '') === 'active', 'offline policy: inside expiry the signed lease remains active');
$offlineGrace = $issuer->verifyEnvelope($leaseA1['envelope'], [
    'expected_product' => 'focusa', 'expected_node_id' => '11111111-1111-4111-8111-111111111111',
    'now' => '2026-11-20T00:00:00Z',
]);
ok(($offlineGrace['state'] ?? '') === 'offline_grace', 'offline policy: past expiry inside grace the lease degrades to offline_grace');
okThrows(
    static fn() => $issuer->verifyEnvelope($leaseA1['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => '11111111-1111-4111-8111-111111111111',
        'now' => '2027-01-01T00:00:00Z',
    ]),
    'EXPIRED',
    'offline policy: past grace the signed lease expires and grants nothing',
);
// Licensed-state expiry projection (EDD truth) -> recovery_only + signed refusal.
$expiryEvent = $projector->projectLicense([
    'from_status' => 'active', 'to_status' => 'expired', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'license_id' => $license1Id, 'request_id' => 'req-expiry-a1-0001', 'idempotency_key' => 'idem-expiry-a1-0001',
]);
ok($expiryEvent['decision'] === 'applied' && $expiryEvent['to_state'] === 'expired' && $expiryEvent['refresh_posture'] === 'recovery_only', 'expiry projects expired / recovery_only at sequence 3');
ok((int) $expiryEvent['result_sequence'] === 3, 'expiry advances the authority sequence to 3');
$db->exec("UPDATE wp_edd_licenses SET status = 'expired' WHERE id = {$license1Id}");
$expiryRefusal = $refresh->refresh($refreshRequest($A, '11111111-1111-4111-8111-111111111111', $credA1, 'refresh-a1-0001', 3));
$expectRefusal($expiryRefusal, 'EXPIRED', 'expired license refresh is denied with a signed recovery-only refusal');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'expiry preserves every project/evidence row');
$expiredStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($expiredStage[$family] === 'deny', "paid family {$family} is blocked again after expiry");
}
foreach (['read_projection', 'basic_customer_data_export', 'repair', 'stable_security_update', 'uninstall'] as $family) {
    ok($expiredStage[$family] === 'allow', "recovery family {$family} remains available after expiry");
}
// The lifetime entitlement survives credential expiry: the projection is still
// active and a replacement credential can be derived (renewal).
$renewalDerivation = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($order1['projected'], '11111111-1111-4111-8111-111111111111', $clock);
ok($renewalDerivation['lease_payload']['status'] === 'active', 'expiry does not destroy the underlying lifetime entitlement (renewal can issue a replacement)');

// ═══════════════════════════════════════════════════════════════════════
// 3. Renewal: order #2 issues a fresh key + projection + lease (no reinstall)
// ═══════════════════════════════════════════════════════════════════════
$alphaR3 = $createVerified($A_EMAIL, $PAID_PRODUCT, 'alpha-renewal-2', checkout: true);
ok($alphaR3['account_uuid'] === $A, 'renewal continues the SAME authority account');
$order2 = $focusaPurchase($alphaR3, 9002, $A_EMAIL, $FOCUSA_DOWNLOAD, $FOCUSA_PRICE, 'alpha-2');
ok((int) $order2['issued']['keys_created'] === 1 && (int) $order2['projected']['sequence'] === 4, 'renewal order #2 issues a fresh key and projection at sequence 4');
ok($sequenceOf($A) === 4, 'account A sequence is 4 after renewal');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'renewal preserves every project/evidence row');
$lcComplete2 = $lifecycleComplete($A, $ACUSTOMER, 9002, $order2['edd_license_id'], 'a2');
ok($lcComplete2['decision'] === 'applied' && (int) $lcComplete2['result_sequence'] === 5, 'lifecycle completed #2 applied at sequence 5');
$nodeA2 = $nodes->registerNode([
    'node_uuid' => '22222222-2222-4222-8222-222222222222',
    'account_uuid' => $A,
    'edd_license_id' => (int) $order2['edd_license_id'],
    'product_code' => $PAID_PRODUCT,
    'device_public_key' => $deviceKey('22222222-2222-4222-8222-222222222222'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-node-a2-0001',
    'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'node-a2-1'],
]);
ok($nodeA2['status'] === 'active', 'renewal registers its continuation node');
$leaseA2 = $issueLease152($A, '22222222-2222-4222-8222-222222222222', $deviceKey('22222222-2222-4222-8222-222222222222'), 'a2');
ok((int) $leaseA2['sequence'] === 6, 'signed lease #2 issued at product-ledger sequence 6');
ok($sequenceOf($A) === 5, 'lease #2 records its own ledger; the account sequence stays 5');
$credA2 = (string) $refresh->issueRefreshCredential(['lease_uuid' => $leaseA2['lease_uuid'], 'idempotency_key' => 'cred-a2-0001', 'request_id' => 'req-cred-a2-0001'])['refresh_credential'];
$highestLeaseSeq = $refresh->highestSequence($A, 'focusa_operator_lifetime_v1');
ok((int) $highestLeaseSeq['highest_sequence'] === 6, 'the per-product lease ledger is strictly monotonic (3 -> 6), never rolled back');
$lease1Row = $issuer->findLease($leaseA1['lease_uuid']);
ok((int) $lease1Row['sequence'] === 3 && (int) $leaseA2['sequence'] === 6, 'lease #1 (sequence 3) is superseded by lease #2 (sequence 6) in the authority ledger');
// The pre-renewal lease (sequence 3) is superseded by the higher sequence in the
// authority ledger; it is never rolled backward (proven above via the ledger).
$leaseFixture2 = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($order2['projected'], '22222222-2222-4222-8222-222222222222', $clock);
$paidStage2 = $stageVerdicts('paid', (array) $leaseFixture2['lease_payload']['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($paidStage2[$family] === 'allow', "paid family {$family} is enabled again after renewal");
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Node removal: limit, explicit deactivation, slot reuse, data preserved
// ═══════════════════════════════════════════════════════════════════════
$nodeSeq = 0;
$register = static function (string $nodeUuid, string $device) use ($nodes, $A, $order2, &$nodeSeq): array {
    $nodeSeq++;
    return $nodes->registerNode([
        'node_uuid' => $nodeUuid,
        'account_uuid' => $A,
        'edd_license_id' => (int) $order2['edd_license_id'],
        'product_code' => 'focusa_operator_lifetime_v1',
        'device_public_key' => $device,
        'assurance_class' => 'device_key_v1',
        'idempotency_key' => 'idem-node-extra-' . $nodeSeq,
        'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'node-extra-' . $nodeSeq],
    ]);
};
$extra1 = $register('33333333-3333-4333-8333-333333333333', $deviceKey('33333333-3333-4333-8333-333333333333'));
ok($extra1['status'] === 'active', 'one additional node registers within the license limit');
$ledgerFull = $nodes->limitLedger($A, 'focusa_operator_lifetime_v1');
ok((int) $ledgerFull['node_limit'] === 3 && (int) $ledgerFull['reserved_count'] === 3, 'the node-limit ledger reserves exactly three slots of three');
okThrows(
    static fn() => $register('44444444-4444-4444-8444-444444444444', $deviceKey('44444444-4444-4444-8444-444444444444')),
    'NODE_LIMIT_EXHAUSTED',
    'a fourth node is denied at the limit',
);
okThrows(
    static fn() => $register('55555555-5555-4555-8555-555555555555', $deviceKey('55555555-5555-4555-8555-555555555555')),
    'NODE_LIMIT_EXHAUSTED',
    'a fifth concurrent node is also denied',
);
ok((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$A}'")->fetchColumn() === 3, 'the denied fourth/fifth nodes created no node rows for the account');
$deactivated = $nodes->deactivateNode([
    'node_uuid' => '33333333-3333-4333-8333-333333333333',
    'account_uuid' => $A,
    'status_reason' => 'user_requested',
    'idempotency_key' => 'idem-node-deactivate-0001',
]);
ok($deactivated['status'] === 'deactivated', 'explicit node removal (deactivation) succeeds');
$ledgerAfter = $nodes->limitLedger($A, 'focusa_operator_lifetime_v1');
ok((int) $ledgerAfter['reserved_count'] === 2, 'deactivation releases the node slot exactly once');
$reservation = $nodes->reserve([
    'node_uuid' => '44444444-4444-4444-8444-444444444444',
    'account_uuid' => $A,
    'edd_license_id' => (int) $order2['edd_license_id'],
    'product_code' => 'focusa_operator_lifetime_v1',
    'device_public_key' => $deviceKey('44444444-4444-4444-8444-444444444444'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-node-reserve-0001',
    'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'reserve-0001'],
]);
ok(($reservation['state'] ?? '') === 'reserved', 'the freed slot is reserved by the explicit two-phase flow');
$settled = $nodes->settleReservation((string) $reservation['reservation_id'], 'ns_reuse_0001', 'idem-node-settle-0001');
ok($settled['status'] === 'active', 'the reserved slot settles to an active node');
$ledgerFinal = $nodes->limitLedger($A, 'focusa_operator_lifetime_v1');
ok((int) $ledgerFinal['reserved_count'] === 3, 'the ledger holds exactly three live slots after removal + reuse');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'node removal preserves every project/evidence row');

// ═══════════════════════════════════════════════════════════════════════
// 5. Refund: order #2 refunds -> recovery_only -> verified user to limited mode
// ═══════════════════════════════════════════════════════════════════════
$refundEvent = $projector->projectRefund([
    'status' => 'refunded', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9002, 'license_id' => (int) $order2['edd_license_id'],
    'request_id' => 'req-refund-a2-0001', 'idempotency_key' => 'idem-refund-a2-0001',
]);
ok($refundEvent['decision'] === 'applied' && $refundEvent['to_state'] === 'refunded' && $refundEvent['refresh_posture'] === 'recovery_only', 'refund projects refunded / recovery_only');
ok((int) $refundEvent['result_sequence'] === 6, 'refund advances the authority sequence to 6');
$db->exec("UPDATE wp_edd_licenses SET status = 'refunded' WHERE id = " . (int) $order2['edd_license_id']);
$refundRefusal = $refresh->refresh($refreshRequest($A, '22222222-2222-4222-8222-222222222222', $credA2, 'refresh-a2-0001', 6));
$expectRefusal($refundRefusal, 'REFUNDED', 'refunded license refresh is denied with a signed recovery-only refusal');
$refundLeaseRow = $issuer->findLease($leaseA2['lease_uuid']);
ok(($refundLeaseRow['status'] ?? '') === 'refunded' && ($refundLeaseRow['status_reason'] ?? '') === 'edd_refunded', 'the refund refusal settles the lease to refunded/edd_refunded');
// The still-verified account returns to limited mode: paid families blocked,
// recovery allowances intact, project data preserved, stale paid credential dead.
$postRefundStage = $stageVerdicts('limited', [], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($postRefundStage[$family] === 'deny', "paid family {$family} is blocked after the refund");
}
foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $family) {
    ok($postRefundStage[$family] === 'allow', "recovery family {$family} remains available after the refund");
}
$limitedVerifyAfterRefund = $limitedService->verify([
    'posture_uuid' => $alphaEval['posture_uuid'],
    'account_uuid' => $A,
    'identity_uuid' => $alphaR1['identity_uuid'],
    'product_scope' => 'focusa',
    'node_uuid' => '11111111-1111-4111-8111-111111111111',
    'family_allowlist' => $alphaAssertion['family_allowlist'],
    'sequence' => $alphaAssertion['sequence'],
    'issued_at' => $alphaAssertion['issued_at'],
    'refresh_at' => $alphaAssertion['refresh_at'],
    'signer' => $alphaAssertion['signer'],
    'signature' => $alphaAssertion['signature'],
], '2026-08-09T06:30:00Z');
ok($limitedVerifyAfterRefund['verdict'] === 'valid', 'the signed limited-access assertion is the valid credential again after the refund');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'refund preserves every project/evidence row');
// Refund replay never bumps the sequence; terminal refund never reactivates.
$refundReplay = $projector->projectRefund([
    'status' => 'refunded', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9002, 'license_id' => (int) $order2['edd_license_id'],
    'request_id' => 'req-refund-a2-0001', 'idempotency_key' => 'idem-refund-a2-0001',
]);
ok($refundReplay['decision'] === 'replayed', 'refund redelivery journals as replayed');
ok($sequenceOf($A) === 6, 'replayed refund never bumps the sequence');
$reactivateAttempt = $projector->projectOrder([
    'status' => 'completed', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
    'order_id' => 9002, 'license_id' => (int) $order2['edd_license_id'],
    'request_id' => 'req-reactivate-a2-0001', 'idempotency_key' => 'idem-reactivate-a2-0001',
]);
ok($reactivateAttempt['decision'] === 'denied' && $reactivateAttempt['error_code'] === 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'a completed order can never reactivate a refunded license');

// ═══════════════════════════════════════════════════════════════════════
// 6. Repurchase: order #3 restores paid capability (authority never rolls back)
// ═══════════════════════════════════════════════════════════════════════
$alphaR4 = $createVerified($A_EMAIL, $PAID_PRODUCT, 'alpha-repurchase-3', checkout: true);
$order3 = $focusaPurchase($alphaR4, 9003, $A_EMAIL, $FOCUSA_DOWNLOAD, $FOCUSA_PRICE, 'alpha-3');
ok((int) $order3['issued']['keys_created'] === 1 && (int) $order3['projected']['sequence'] === 7, 'repurchase order #3 issues a fresh key and projection at sequence 7');
ok($sequenceOf($A) === 7, 'account A sequence is 7 after the repurchase (strictly monotonic, never rolled back)');
$lcComplete3 = $lifecycleComplete($A, $ACUSTOMER, 9003, $order3['edd_license_id'], 'a3');
ok($lcComplete3['decision'] === 'applied' && (int) $lcComplete3['result_sequence'] === 8, 'lifecycle completed #3 applied at sequence 8');
$leaseFixture3 = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($order3['projected'], '11111111-1111-4111-8111-111111111111', $clock);
$paidStage3 = $stageVerdicts('paid', (array) $leaseFixture3['lease_payload']['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($paidStage3[$family] === 'allow', "paid family {$family} is enabled again after the repurchase");
}
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'repurchase preserves every project/evidence row');
$seqAChain = [0, 1, 2, 3, 4, 5, 6, 7, 8];

// ═══════════════════════════════════════════════════════════════════════
// 7. Bundle continuation: limited -> Bundle -> refund -> limited -> renewal -> revoke
// ═══════════════════════════════════════════════════════════════════════
$B_EMAIL = 'bundle.continuation.beta@example.invalid';
$betaR1 = $createVerified($B_EMAIL, 'focusa', 'beta-eval');
$betaEval = $evaluation->requestEvaluation($evaluationInput($betaR1, '77777777-7777-4777-8777-777777777777', 'beta'));
ok($betaEval['decision'] === 'limited_access_issued', 'Bundle account starts verified limited');
$B = (string) $betaR1['account_uuid'];
ok($counts('wp_edd_customers') === 2, 'Bundle account is the second (and only second) customer');
$betaAssertion = $limitedService->issue([
    'posture_uuid' => $betaEval['posture_uuid'],
    'issued_at' => '2026-08-09T06:05:00Z',
    'refresh_at' => '2026-08-09T06:35:00Z',
    'migration_provenance' => ['source' => 'paid_lifecycle_e2e', 'record' => 'beta-assertion-1'],
]);
ok($betaAssertion['verdict'] === 'valid', 'Bundle account holds its signed limited assertion');
// Bundle order #1: one SKU, one human key, exact union of the two grants.
$betaR2 = $createVerified($B_EMAIL, $BUNDLE_PRODUCT, 'beta-bundle-1', checkout: true);
ok($betaR2['account_uuid'] === $B && $betaR2['edd_customer_id'] === $betaR1['edd_customer_id'], 'Bundle purchase continues the SAME account and customer');
$bundle1 = $bundlePurchase($betaR2, 9101, $B_EMAIL, 'beta-1');
ok($bundle1['bound']['decision'] === 'bundle_bound_and_issued', 'Bundle order #1 binds and issues through the adapter');
ok((int) $bundle1['bound']['human_key_count'] === 1, 'Bundle order #1 is exactly one human key');
ok((int) $bundle1['projected']['sequence'] === 1, 'Bundle projection #1 advances account B to sequence 1');
$bundleGrants = (array) $bundle1['projected']['grants'];
sort($bundleGrants, SORT_STRING);
ok($bundleGrants === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'Bundle projection grants exactly the two underlying Operator types');
ok($bundle1['projected']['family_digest'] === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'Bundle family digest is the derived two-record union');
ok($bundle1['projected']['price_version'] === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'Bundle carries the server-owned 1254.60 price version');
$bundleLease1 = FocusaSpec172BundleSignedLeaseFixture::fromProjection($bundle1['projected'], '77777777-7777-4777-8777-777777777777', $clock);
FocusaSpec172BundleSignedLeaseFixture::validate($bundleLease1, $bundle1['projected']);
$bundleLeaseGrants = array_keys($bundleLease1['lease_payload']['grants']);
sort($bundleLeaseGrants, SORT_STRING);
ok($bundleLeaseGrants === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'Bundle signed lease grants both underlying grants');
$bundlePaidStage = $stageVerdicts('paid', (array) $bundleLease1['lease_payload']['features'], 1);
foreach ($PAID_FAMILIES as $family) {
    ok($bundlePaidStage[$family] === 'allow', "Bundle enables paid family {$family}");
}
$paidAssertionB1 = $assertionFixture->paidAssertion($bundle1['projected'], '77777777-7777-4777-8777-777777777777', $clock);
ok($paidAssertionB1['kind'] === 'paid' && $paidAssertionB1['assertion_payload']['status'] === 'active', 'the Bundle derives a paid credential from the ACTIVE projection');
// Bundle refund: whole order only, both grants revoked, verified account to limited.
$db->exec("UPDATE wp_edd_orders SET status = 'refunded', date_updated = '2026-08-14T00:00:00Z' WHERE id = 9101");
$insertRefund(9101, $bundle1['customer_id'], null, '1254.60', 'complete', 'edd', '2026-08-14T00:00:00Z');
$bundleRefund = $settler->settle([
    'order_id' => 9101, 'customer_id' => $bundle1['customer_id'], 'account_uuid' => $B,
    'transition' => 'refund', 'request_id' => 'req-settle-refund-b1', 'idempotency_key' => 'idem-settle-refund-b1',
]);
ok($bundleRefund['decision'] === 'applied' && $bundleRefund['to_state'] === 'refunded', 'Bundle refund settles applied to refunded');
ok((int) $bundleRefund['grants_revoked'] === 2, 'Bundle refund revokes both grants together');
ok($bundleRefund['limited_posture'] === 'verified_no_license' && $bundleRefund['paid_grants_active'] === false, 'Bundle refund returns the verified account to limited mode');
ok((int) $bundleRefund['result_sequence'] === 2, 'Bundle refund advances account B to sequence 2');
$db->exec("UPDATE wp_edd_licenses SET status = 'refunded' WHERE id = " . (int) $bundle1['edd_license_id']);
$limitedB1 = $assertionFixture->limitedPosture($bundleRefund, '77777777-7777-4777-8777-777777777777', $clock);
ok($limitedB1['kind'] === 'verified_no_license' && $limitedB1['paid_grants_active'] === false, 'post-refund posture is verified_no_license with zero paid grants');
$verifyLimitedB1 = $assertionFixture->verifyLimited($limitedB1);
ok($verifyLimitedB1['valid'] === true, 'the post-refund limited assertion verifies with the server-owned key');
foreach (FocusaSpec172AssertionTransitionFixture::PERMANENT_ALLOWANCES as $allowance) {
    ok(in_array($allowance, $limitedB1['permanent_allowances'], true), "Bundle refund keeps recovery allowance {$allowance}");
}
foreach ($PAID_FAMILIES as $family) {
    ok(!in_array($family, $limitedB1['families_allowed'], true), "Bundle refund excludes paid family {$family} from the limited allowlist");
}
okThrows(
    static fn() => FocusaSpec172AssertionTransitionFixture::validatePaidAssertion(
        $assertionFixture->paidAssertion($bundle1['projected'], '77777777-7777-4777-8777-777777777777', $clock),
        2,
        'refunded',
    ),
    'PAID_GRANT_REVOKED',
    'a stale paid Bundle credential is rejected once the Bundle is terminal',
);
// Bundle renewal: order #2 restores paid capability on the same account.
$betaR3 = $createVerified($B_EMAIL, $BUNDLE_PRODUCT, 'beta-bundle-2', checkout: true);
$bundle2 = $bundlePurchase($betaR3, 9102, $B_EMAIL, 'beta-2');
ok((int) $bundle2['projected']['sequence'] === 3, 'Bundle renewal projects at sequence 3 (paid again)');
$paidAssertionB2 = $assertionFixture->paidAssertion($bundle2['projected'], '77777777-7777-4777-8777-777777777777', $clock);
ok($paidAssertionB2['assertion_payload']['status'] === 'active', 'Bundle renewal reactivates the paid credential');
// Bundle revoke: order #2 revoked, both grants removed, verified account limited again.
$db->exec("UPDATE wp_edd_orders SET status = 'revoked', date_updated = '2026-08-15T00:00:00Z' WHERE id = 9102");
$bundleRevoke = $settler->settle([
    'order_id' => 9102, 'customer_id' => $bundle2['customer_id'], 'account_uuid' => $B,
    'transition' => 'revoke', 'request_id' => 'req-settle-revoke-b2', 'idempotency_key' => 'idem-settle-revoke-b2',
]);
ok($bundleRevoke['decision'] === 'applied' && $bundleRevoke['to_state'] === 'revoked' && (int) $bundleRevoke['grants_revoked'] === 2, 'Bundle revoke removes both grants');
ok((int) $bundleRevoke['result_sequence'] === 4, 'Bundle revoke advances account B to sequence 4');
$limitedB2 = $assertionFixture->limitedPosture($bundleRevoke, '77777777-7777-4777-8777-777777777777', $clock);
ok($limitedB2['kind'] === 'verified_no_license' && $assertionFixture->verifyLimited($limitedB2)['valid'] === true, 'post-revoke posture returns to verified_no_license');
okThrows(
    static fn() => FocusaSpec172AssertionTransitionFixture::validatePaidAssertion(
        $assertionFixture->paidAssertion($bundle2['projected'], '77777777-7777-4777-8777-777777777777', $clock),
        4,
        'revoked',
    ),
    'PAID_GRANT_REVOKED',
    'the revoked Bundle paid credential can never reactivate',
);
// A second adverse event on the already-terminal Bundle journals replayed (zero bump).
$bundleRevokeReplay = $settler->settle([
    'order_id' => 9102, 'customer_id' => $bundle2['customer_id'], 'account_uuid' => $B,
    'transition' => 'revoke', 'request_id' => 'req-settle-revoke-b2r', 'idempotency_key' => 'idem-settle-revoke-b2r',
]);
ok($bundleRevokeReplay['decision'] === 'replayed' && (int) $bundleRevokeReplay['sequence_increment'] === 0, 'a duplicate adverse event on a terminal Bundle never bumps the sequence');
ok($sequenceOf($B) === 4, 'account B sequence stays 4 after the replay');
ok($projectRows() === 6 && hash('sha256', json_encode($db->query("SELECT operation_key, family FROM wp_wpuiai_e2e_project_data ORDER BY operation_key")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)) === $projectDataDigest, 'the Bundle lifecycle preserves every account-A project/evidence row');
$seqBChain = [0, 1, 2, 3, 4];

// ═══════════════════════════════════════════════════════════════════════
// 8. No caller control, preservation, redaction, rollback
// ═══════════════════════════════════════════════════════════════════════
okThrows(
    static fn() => $bindingService->bindOrderComplete([
        'order_id' => 9901, 'order_status' => 'complete', 'customer_id' => $ACUSTOMER,
        'order_items' => [['order_item_id' => 9901, 'download_id' => $FOCUSA_DOWNLOAD, 'price_id' => $FOCUSA_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_pay_9901', 'status' => 'complete']],
        'registration_uuid' => $alphaR2['registration_uuid'],
        'facade_id' => 'focusa_install_v1', 'origin' => $ORIGIN,
        'request_id' => 'req-bind-grant-0001', 'idempotency_key' => 'idem-bind-grant-0001',
        'grants' => [$PAID_PRODUCT],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'order binding rejects caller-supplied grants',
);
okThrows(
    static fn() => $issuanceService->issue([
        'issuance_request_handle' => (string) $order1['bound']['protected_items'][0]['issuance_request_handle'],
        'request_id' => 'req-issue-price-0001', 'idempotency_key' => 'idem-issue-price-0001',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'license issuance rejects caller-controlled price',
);
okThrows(
    static fn() => $focusaProjector->project([
        'issuance_request_handle' => (string) $order1['bound']['protected_items'][0]['issuance_request_handle'],
        'request_id' => 'req-project-grant-0001', 'idempotency_key' => 'idem-project-grant-0001',
        'node_limit' => 99,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'projection rejects caller-controlled node limits',
);
okThrows(
    static fn() => $settler->settle([
        'order_id' => 9101, 'customer_id' => $bundle1['customer_id'], 'account_uuid' => $B,
        'transition' => 'refund', 'scope' => 'component', 'request_id' => 'req-settle-forbid-0001', 'idempotency_key' => 'idem-settle-forbid-0001',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'settlement rejects caller-controlled refund scope',
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
okThrows(
    static fn() => $projector->projectOrder([
        'status' => 'completed', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
        'order_id' => 9001, 'license_id' => $license1Id, 'price' => '0.00',
        'request_id' => 'req-price-0001', 'idempotency_key' => 'idem-price-0001',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'lifecycle projection rejects caller-controlled price',
);
okThrows(
    static fn() => $projector->projectOrder([
        'status' => 'completed', 'account_uuid' => $A, 'edd_customer_id' => $ACUSTOMER,
        'order_id' => 9001, 'license_id' => $license1Id, 'state_reason' => 'owner at x@example.invalid',
        'request_id' => 'req-email-0001', 'idempotency_key' => 'idem-email-0001',
    ]),
    'INPUT_RAW_EMAIL_FORBIDDEN',
    'raw email on a lifecycle event fails closed',
);

// Preservation: every account/customer/order/license/refund/projection row is kept.
$preservedCounts = [
    'customers' => $counts('wp_edd_customers'),
    'orders' => $counts('wp_edd_orders'),
    'licenses' => $counts('wp_edd_licenses'),
    'refunds' => $counts('wp_edd_order_refunds'),
    'projections' => $counts('wp_wpuiai_license_type_projections'),
    'accounts' => $counts('wp_wpuiai_authority_accounts'),
    'registrations' => $counts('wp_wpuiai_activation_registrations'),
    'settlements' => $counts('wp_wpuiai_spec172_settlements'),
    'lifecycle_events' => $counts('wp_wpuiai_edd_lifecycle_events'),
    'project_data' => $projectRows(),
];
ok((int) $preservedCounts['customers'] === 2, 'exactly two customers total (no duplicates across the whole lifecycle)');
ok((int) $preservedCounts['licenses'] === 5, 'exactly five canonical EDD licenses (one per order, never duplicated)');
ok((int) $preservedCounts['projections'] === 5, 'exactly five paid projections (three Focusa + two Bundle)');
ok((int) $preservedCounts['settlements'] === 3, 'exactly three settlement journal rows (Bundle refund + revoke + replay)');
ok((int) $preservedCounts['project_data'] === 6, 'all six project/evidence rows preserved to the end');
ok((int) $preservedCounts['orders'] === 5, 'all five orders preserved');

// Rollback is preservation-only on the projection and settlement schemas.
$projectionRollback = $projectionMigration->preserveForRollback('2026-08-09T07:00:00Z', ['source' => 'paid_lifecycle_e2e', 'record' => 'rollback']);
ok($projectionRollback['action'] === 'preserve', 'projection rollback contract is preservation-only');
$settlementRollback = $settlementMigration->preserveForRollback('2026-08-09T07:00:00Z', ['source' => 'paid_lifecycle_e2e', 'record' => 'rollback']);
ok($settlementRollback['action'] === 'preserve', 'settlement rollback contract is preservation-only');

// Redaction: no raw email, key, payment reference, or secret in any decision or
// journal that the summary would ever expose; every journal row is scanned.
$allJournals = json_encode([
    $db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC),
    $db->query('SELECT * FROM wp_wpuiai_edd_lifecycle_events')->fetchAll(PDO::FETCH_ASSOC),
    $db->query('SELECT * FROM wp_wpuiai_spec172_settlements')->fetchAll(PDO::FETCH_ASSOC),
    $db->query('SELECT * FROM wp_wpuiai_evaluation_issuances')->fetchAll(PDO::FETCH_ASSOC),
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
ok(strpos($allJournals, '@') === false, 'journals carry no raw email');
ok(preg_match($KEY_SCAN_PATTERN, $allJournals) !== 1, 'journals carry no full license key');
ok(strpos($allJournals, 'txn_pay_') === false, 'journals carry no payment transaction id');
ok(strpos($allJournals, 'cus_') === false, 'journals carry no customer reference');
ok(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $allJournals) !== 1, 'journals carry no payment key');
$allTxns = $db->query('SELECT transaction_id, gateway, status FROM wp_edd_order_transactions')->fetchAll(PDO::FETCH_ASSOC);
foreach ($allTxns as $txn) {
    ok(strpos((string) $txn['transaction_id'], 'ch_') === false && strpos((string) $txn['transaction_id'], 'pi_') === false, 'no real Stripe payment intent or charge id is used (no live charge)');
}

// ── Summary (deterministic; counts and booleans only) ──────────────────
$summary = [
    'schema' => 'focusa.spec152f.paid_lifecycle_e2e.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'result' => 'paid_continuation_and_adverse_lifecycle_proven',
    'customers_total' => $preservedCounts['customers'],
    'registrations_total' => $preservedCounts['registrations'],
    'orders_total' => $preservedCounts['orders'],
    'licenses_total' => $preservedCounts['licenses'],
    'projections_total' => $preservedCounts['projections'],
    'sequence_chains' => ['focusa_operator' => $seqAChain, 'bundle' => $seqBChain],
    'final_sequence' => ['focusa_operator' => $sequenceOf($A), 'bundle' => $sequenceOf($B)],
    'paid_families' => $PAID_FAMILIES,
    'limited_allowlist_count' => count($LIMITED_ALLOWLIST),
    'permanent_allowances_count' => count($PERMANENT),
    'project_data_rows' => $projectRows(),
    'project_data_preserved' => true,
    'no_reinstall_same_account' => true,
    'same_customer_no_duplicate' => true,
    'no_second_key_per_order' => true,
    'paid_families_enabled' => true,
    'paid_families_blocked_in_limited' => true,
    'credential_window' => ['active' => true, 'offline_grace' => true, 'expired' => true],
    'lifetime_term_preserved_on_expiry' => true,
    'expiry_refusal_verified' => true,
    'refund_refusal_verified' => true,
    'renewal_reactivated_paid' => true,
    'repurchase_restored_paid' => true,
    'node_removal' => ['limit' => 3, 'reserved' => (int) $ledgerFinal['reserved_count'], 'released_once' => true, 'slot_reused' => true],
    'bundle' => ['refund_limited' => true, 'renewal_paid' => true, 'revoke_limited' => true, 'stale_paid_rejected' => true, 'replay_zero_bump' => true],
    'paid_posture_preserved_on_eval_retry' => true,
    'caller_controlled_denied' => true,
    'recovery_always_available' => true,
    'rollback_preservation_only' => true,
    'preserved' => $preservedCounts,
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
    expect(result["schema"] == "focusa.spec152f.paid_lifecycle_e2e.v1", "harness summary schema pinned")
    expect(result["result"] == "paid_continuation_and_adverse_lifecycle_proven", "harness result proven")
    expect(result["customers_total"] == 2 and result["registrations_total"] == 7
           and result["orders_total"] == 5 and result["licenses_total"] == 5
           and result["projections_total"] == 5,
           "exact customer/registration/order/license/projection totals (no duplicates anywhere)")
    expect(result["sequence_chains"]["focusa_operator"] == [0, 1, 2, 3, 4, 5, 6, 7, 8]
           and result["final_sequence"]["focusa_operator"] == 8,
           "Focusa account authority sequence is strictly monotonic 0..8 (before/after receipts)")
    expect(result["sequence_chains"]["bundle"] == [0, 1, 2, 3, 4]
           and result["final_sequence"]["bundle"] == 4,
           "Bundle account authority sequence is strictly monotonic 0..4")
    expect(sorted(result["paid_families"]) == ["automation", "base_focusa", "premium_updates", "release_proof", "team_remote"],
           "the paid boundary is exactly the five frozen Focusa families (one small family set, no 395 paywalls)")
    expect(result["paid_families_enabled"] is True and result["paid_families_blocked_in_limited"] is True,
           "payment enables base/purchased families and limited mode blocks them")
    expect(result["no_reinstall_same_account"] is True and result["same_customer_no_duplicate"] is True,
           "purchase continues the same account/customer without reinstall and without duplicate customers")
    expect(result["project_data_rows"] == 6 and result["project_data_preserved"] is True,
           "existing Evaluation/project data is preserved through the entire lifecycle")
    expect(result["credential_window"]["active"] is True and result["credential_window"]["offline_grace"] is True
           and result["credential_window"]["expired"] is True and result["lifetime_term_preserved_on_expiry"] is True,
           "expiry degrades the bounded credential and never destroys the lifetime entitlement")
    expect(result["expiry_refusal_verified"] is True and result["refund_refusal_verified"] is True,
           "expiry/refund refresh is denied with verified signed recovery-only refusals")
    expect(result["renewal_reactivated_paid"] is True and result["repurchase_restored_paid"] is True,
           "renewal and repurchase restore paid capability without rolling authority backward")
    expect(result["node_removal"]["limit"] == 3 and result["node_removal"]["reserved"] == 3
           and result["node_removal"]["released_once"] is True and result["node_removal"]["slot_reused"] is True,
           "node removal releases the slot exactly once and the explicit flow reuses it")
    expect(result["bundle"]["refund_limited"] is True and result["bundle"]["renewal_paid"] is True
           and result["bundle"]["revoke_limited"] is True and result["bundle"]["stale_paid_rejected"] is True
           and result["bundle"]["replay_zero_bump"] is True,
           "Bundle adverse lifecycle returns verified users to limited mode; stale paid credentials are rejected")
    expect(result["paid_posture_preserved_on_eval_retry"] is True, "a paid posture is never downgraded by an Evaluation retry")
    expect(result["caller_controlled_denied"] is True, "caller-controlled product/price/grants fail closed on every surface")
    expect(result["recovery_always_available"] is True, "read/export/recovery/repair/stable update/uninstall remain in every state")
    expect(result["rollback_preservation_only"] is True and result["live_charge"] is False,
           "rollback is preservation-only and no live charge was made")
    harness_positive = int(result["positive_checks"])
    harness_negative = int(result["negative_checks"])
    expect(harness_positive >= 180 and harness_negative >= 10, "harness check counts are bounded and non-trivial")

    # ── Canonical policy grid: paid and adverse cells ──────────────────────
    policy = yaml.safe_load(POLICY_YAML.read_text(encoding="utf-8"))
    grid = {item["state"]: item for item in policy.get("state_grid", [])}
    active_cell = grid.get("active_paid", {}).get("policies", {})
    expect(active_cell.get("base_focusa") == "require_base", "canonical grid: active_paid base_focusa requires the base product")
    for family in ["automation", "team_remote", "release_proof", "premium_updates"]:
        expect(active_cell.get(family) == "require_feature", f"canonical grid: active_paid {family} is require_feature")
    expect(active_cell.get("account_recovery") == "allow" and active_cell.get("read_projection") == "read"
           and active_cell.get("customer_data_export") == "allow",
           "canonical grid: active_paid keeps recovery/read/export available")
    for state in ["expired", "refunded_or_revoked"]:
        cell = grid.get(state, {}).get("policies", {})
        expect(cell.get("base_focusa") == "deny", f"canonical grid: {state} denies base_focusa")
        for family in ["automation", "team_remote", "release_proof", "premium_updates"]:
            expect(cell.get(family) == "deny", f"canonical grid: {state} denies paid family {family}")
        expect(cell.get("account_recovery") == "allow" and cell.get("read_projection") == "read"
               and cell.get("customer_data_export") == "allow",
               f"canonical grid: {state} keeps recovery/read/export available")

    # ── Verified limited-access registry: frozen limited boundary ──────────
    limited_registry = yaml.safe_load(LIMITED_YAML.read_text(encoding="utf-8"))
    focusa_limited = limited_registry["focusa"]
    expect("expiry" not in focusa_limited or focusa_limited["expiry"] == "none"
           or limited_registry["postures"]["verified_no_license"]["expiry"] == "none",
           "canonical registry: verified_no_license has no automatic expiry")
    for family in ["automation", "team_remote", "release_proof", "premium_updates"]:
        expect(family in focusa_limited["blocked_families"], f"canonical registry blocks paid family {family}")
    expect("manual_project" in focusa_limited["allowed_families"], "canonical registry allows the first-value manual project")
    expect("read_projection" in limited_registry["permanent_allowances"]["families"]
           and "basic_customer_data_export" in limited_registry["permanent_allowances"]["families"],
           "canonical registry keeps read and basic export permanent")

    # ── Rust base-product and family-classifier surfaces (runtime policy) ──
    policy_source = POLICY.read_text(encoding="utf-8")
    non_test_policy = policy_source.split("#[cfg(test)]")[0]
    expect("PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace => {" in non_test_policy
           and "PolicyEntitlementState::VerifiedNoLicense => BaseProductDecision::Limited" in non_test_policy
           and "PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace => {" in non_test_policy
           and "BaseProductDecision::Entitled" in non_test_policy,
           "Rust base product resolution: paid/offline-grace -> Entitled, verified_no_license -> Limited")
    expect("if product != \"focusa\"" in non_test_policy, "Rust base product resolution is product-boundary closed")
    expect("Self::Expired => \"expired\"" in non_test_policy
           and "Self::RefundedOrRevoked => \"refunded_or_revoked\"" in non_test_policy,
           "Rust policy models the adverse refunded/revoked/expired states")
    expect("pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES" in policy_source
           and '"automation"' in policy_source and '"team_remote"' in policy_source
           and '"release_proof"' in policy_source and '"premium_updates"' in policy_source,
           "Rust classifier carries exactly the four paid blocked families")
    expect("ActiveLeaseExpired" in policy_source and "CachedGrantExpired" in policy_source,
           "Rust premium-family resolver models credential expiry as a bounded denial")
    core_source = CORE_LICENSE.read_text(encoding="utf-8")
    non_test_core = core_source.split("#[cfg(test)]")[0]
    expect("pub fn require_base_product()" in non_test_core
           and "CapabilityFamily::BaseFocusa" in non_test_core
           and "OperationClass::ValueMutation" in non_test_core,
           "core require_base_product gates value-producing base Focusa mutations")
    expect("BaseProductRequired(String)" in non_test_core, "core fails closed with LicenseError::BaseProductRequired")

    # ── Spec 172 contracts: paid/adverse authority behavior ────────────────
    settlement_source = SETTLEMENT_PHP.read_text(encoding="utf-8")
    expect("'refund' => ['to_state' => 'refunded'" in settlement_source
           and "'revoke' => ['to_state' => 'revoked'" in settlement_source
           and "'chargeback' => ['to_state' => 'refunded'" in settlement_source,
           "settlement transition matrix pins refund/chargeback/revoke to terminal states")
    expect("'refund' => ['to_state' => 'refunded', 'sequence_increment' => 1, 'terminal' => true" in settlement_source,
           "each adverse settlement advances the sequence by exactly one and is terminal")
    expect("verified_no_license" in settlement_source and "unverified" in settlement_source,
           "settlement returns the account to verified_no_license / unverified limited posture")
    expect("PAID_GRANT_REVOKED" in settlement_source and "STALE_CREDENTIAL_SUPERSEDED" in settlement_source,
           "stale paid credentials can never reactivate after an adverse settlement")
    transition_source = TRANSITION_PHP.read_text(encoding="utf-8")
    expect("public const TERMINAL_STATES = ['refunded', 'revoked', 'expired', 'superseded', 'cancelled', 'denied']" in settlement_source,
           "settlement pins the six terminal states (refunded/revoked/expired/superseded/cancelled/denied)")
    for family in ["manual_project", "manual_mission", "manual_focus_state", "manual_workpoint",
                   "manual_trajectory", "manual_basic_evidence"]:
        expect(f"'{family}'" in transition_source, f"transition fixture carries the frozen limited family {family}")
    for family in ["read_projection", "basic_customer_data_export", "account_control", "device_control",
                   "license_status", "diagnostics", "repair", "rollback", "stable_security_update", "uninstall"]:
        expect(f"'{family}'" in transition_source, f"transition fixture keeps the permanent allowance {family}")
    expect("'paid_families_excluded' => true" in transition_source, "post-settlement limited posture excludes paid families")
    projector_source = PROJECTOR_PHP.read_text(encoding="utf-8")
    expect("'refunded', 'revoked'" in projector_source or "in_array($status, ['refunded', 'revoked'], true)" in projector_source,
           "projector fails closed on refunded/revoked canonical orders")
    expect("CLIENT_COMMERCIAL_FIELDS_FORBIDDEN" in projector_source, "projection rejects caller commerce fields")
    bundle_source = BUNDLE_PROJECTOR_PHP.read_text(encoding="utf-8")
    expect("BUNDLE_ITEM_COUNT_REQUIRED" in bundle_source and "BUNDLE_ORDER_INELIGIBLE" in bundle_source
           and "BUNDLE_KEY_ISSUANCE_FAILED" in bundle_source,
           "Bundle adapter adds only its three Bundle-scoped fail-closed codes")
    expect("public const FOCUSA_LICENSE_TYPE = 'focusa_operator_lifetime_v1'" in bundle_source
           and "public const UIAI_LICENSE_TYPE = 'uiai_operator_lifetime_v1'" in bundle_source
           and "return [self::FOCUSA_LICENSE_TYPE, self::UIAI_LICENSE_TYPE]" in bundle_source,
           "Bundle grants are exactly the two underlying Operator types")
    lifecycle_source = LIFECYCLE_PHP.read_text(encoding="utf-8")
    expect("'refunded', 'revoked', 'expired', 'superseded', 'cancelled', 'denied'" in lifecycle_source,
           "lifecycle projector pins the six terminal states")
    expect("'refresh_posture' => 'recovery_only'" in lifecycle_source
           and "LICENSE_TERMINAL_REACTIVATION_DENIED" in lifecycle_source
           and "ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED" in lifecycle_source,
           "lifecycle projector fails closed: recovery_only, no reactivation, no rollback")
    refresh_source = REFRESH_PHP.read_text(encoding="utf-8")
    for reason in ["REFUNDED", "REVOKED", "EXPIRED"]:
        expect(f"'{reason}' => 'recovery_only'" in refresh_source, f"lease refresh pins {reason} -> recovery_only")

    # ── No 395 paywalls: one canonical purchase/denial UX surface ──────────
    catalog = json.loads(DENIAL_CATALOG.read_text(encoding="utf-8"))
    expect(catalog["schema"] == "focusa.spec152f.denial_ux_catalog.v1", "denial UX catalog schema pinned")
    action_ids = {action["id"] for action in catalog["actions"]}
    expect({"evaluate", "purchase", "manage", "recovery"} <= action_ids, "catalog carries the canonical upgrade actions")
    selector_fields = {"price", "product", "grant", "plan", "feature", "tier", "sku"}
    for action in catalog["actions"]:
        expect(not selector_fields.intersection(action.keys()),
               f"denial action carries no product/price/grant selector: {action.get('id')}",
               is_negative=True)

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
        "schema": "focusa.spec152f.paid_lifecycle_e2e_validation.v1",
        "atom": "focusa-vbcqu.20.14.46",
        "harness_sha256": sha256_text(HARNESS),
        "harness_output_sha256": sha256_text(out1),
        "harness_positive_checks": harness_positive,
        "harness_negative_checks": harness_negative,
        "static_positive_checks": positive,
        "static_negative_checks": negative,
        "customers_total": result["customers_total"],
        "registrations_total": result["registrations_total"],
        "orders_total": result["orders_total"],
        "licenses_total": result["licenses_total"],
        "projections_total": result["projections_total"],
        "sequence_chains": result["sequence_chains"],
        "final_sequence": result["final_sequence"],
        "paid_families": result["paid_families"],
        "limited_allowlist_count": result["limited_allowlist_count"],
        "permanent_allowances_count": result["permanent_allowances_count"],
        "project_data_rows": result["project_data_rows"],
        "project_data_preserved": result["project_data_preserved"],
        "no_reinstall_same_account": result["no_reinstall_same_account"],
        "paid_families_enabled": result["paid_families_enabled"],
        "credential_window": result["credential_window"],
        "lifetime_term_preserved_on_expiry": result["lifetime_term_preserved_on_expiry"],
        "expiry_refusal_verified": result["expiry_refusal_verified"],
        "refund_refusal_verified": result["refund_refusal_verified"],
        "renewal_reactivated_paid": result["renewal_reactivated_paid"],
        "repurchase_restored_paid": result["repurchase_restored_paid"],
        "node_removal": result["node_removal"],
        "bundle": result["bundle"],
        "paid_posture_preserved_on_eval_retry": result["paid_posture_preserved_on_eval_retry"],
        "caller_controlled_denied": result["caller_controlled_denied"],
        "recovery_always_available": result["recovery_always_available"],
        "rollback_preservation_only": result["rollback_preservation_only"],
        "harness_replay_identical": True,
        "result": "passed",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
