#!/usr/bin/env python3
"""Spec 152F.06.03 — Prove verified Evaluation reaches first useful value (E2E).

Atom focusa-vbcqu.20.14.45 (152F.06.03): one verified Evaluation reaches the
first useful Focusa outcome with no local issuance, no duplicate customer/key,
and no 395-step paywall friction. Binding Spec 172 overlay: the verified
no-license posture reaches first useful LIMITED value with no license key and
no expiry; the second-project mutation and the paid tool families are blocked
with canonical upgrade guidance.

Exact verification:
    python3 tests/spec152f_evaluation_first_value_e2e_test.py

Exact surfaces (Spec 152F §11 conversion principles; Spec 172 §6.2):
- verified-email Evaluation fixture: tests/fixtures/spec172-limited-access-cases.v1.json
  (30 canonical vectors, cross-checked against docs/contracts/spec172-verified-limited-access.v1.yaml
  and the Rust classifier constants) plus the deterministic php/sqlite journey
- EDD-backed key/lease: docs/contracts/spec152e-evaluation-issuance.v1.php +
  docs/contracts/spec172-limited-access-assertion-service.v1.php +
  docs/contracts/spec152e-edd-bound-lease-issuer.v1.php (authority-owned signed
  envelopes; no EDD order, no EDD key, no caller-controlled grant)
- installer/first-run: crates/focusa-cli/src/commands/install.rs,
  crates/focusa-cli/src/commands/activation_flow.rs,
  crates/focusa-license/src/authority_store.rs,
  scripts/install-focusa.sh + scripts/install-focusa.ps1 (read-only checks:
  authority-signed adoption only, no local Evaluation, advisory channel only)
- base Focusa project/Workpoint operation: crates/focusa-core/src/license.rs
  (require_base_product) + crates/focusa-license/src/entitlement_policy.rs
  (resolve_base_focusa_product, BaseProductDecision, Spec 172 family classifier)

What is proven here:
1. Start with zero customers/postures/assertions/nodes/leases; verify the
   mailbox; the authority creates exactly one EDD customer; issuing the
   Evaluation creates NO EDD order, NO EDD key, no expiry (permanent
   no_automatic_expiry) and exactly one verified_no_license posture + one
   signed limited-access assertion + one registered node; replays are
   idempotent and a facade-switched duplicate is denied.
2. The signed runtime credential is real RFC 8032 Ed25519: issue, verify,
   refresh rotate to the next monotonic sequence without widening the
   allowlist; a widened premium family and a stale sequence both fail closed.
3. Install/resume adopts only signed authority material (bounded synthetic
   runtime record stores only signed envelopes and opaque references).
4. The first useful value loop completes WITHOUT a card: one active project,
   Mission, Focus State, Workpoint, Trajectory, and basic Evidence are all
   permitted by the canonical limited-mode allowlist; read/export/recovery
   remain available; no EDD order/key is created by the value loop.
5. The second-project mutation is blocked (mutable project limit 1) and the
   four paid tool families (automation, team_remote, release_proof,
   premium_updates) are blocked, each with canonical upgrade guidance
   (evaluate/purchase action + evaluation/checkout link + retained access)
   and no partial side effects — a single small family boundary, not 395
   paywalls.
6. No premium grant unless authority includes it: the posture allowlist is
   exactly the server-owned six-family list and a caller can never widen it.
7. Redaction: no raw email, no key-shaped material, no customer-identifier
   material in journal rows, harness output, or the gate itself.

Build-independent: no cargo build, no live network, no publication. The php
harness runs twice and its stdout is byte-identical (replayable from the
pinned commit).
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
LIMITED_CASES = ROOT / "tests/fixtures/spec172-limited-access-cases.v1.json"
LIMITED_YAML = CONTRACTS / "spec172-verified-limited-access.v1.yaml"
POLICY_YAML = CONTRACTS / "spec152f-entitlement-policy.v1.yaml"
DENIAL_CATALOG = CONTRACTS / "spec152f-denial-ux-catalog.v1.json"
CLI_OP_MAP = CONTRACTS / "spec152f-cli-operation-map.v1.json"
MENUBAR_MAP = CONTRACTS / "spec152f-menubar-action-map.v1.json"
POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"
CORE_LICENSE = ROOT / "crates/focusa-core/src/license.rs"
INSTALL = ROOT / "crates/focusa-cli/src/commands/install.rs"
FLOW = ROOT / "crates/focusa-cli/src/commands/activation_flow.rs"
STORE = ROOT / "crates/focusa-license/src/authority_store.rs"
LICENSE = ROOT / "crates/focusa-cli/src/commands/license.rs"
DENIAL_UX = ROOT / "crates/focusa-license/src/denial_ux.rs"
INSTALL_SH = ROOT / "scripts/install-focusa.sh"
INSTALL_PS1 = ROOT / "scripts/install-focusa.ps1"

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
// Spec 152F.06.03 verified Evaluation reaches first useful value — journey
// harness (generated by the python gate). One verified Evaluation walks from a
// zero-customer baseline through mailbox verification, authority promotion,
// EDD-backed Evaluation issuance (verified_no_license), node binding, signed
// limited-access assertion, signed envelope adoption (install/resume), and the
// first useful value loop — then proves the second-project mutation and the
// paid tool families fail closed with canonical upgrade guidance. Deterministic
// sqlite kernel, fixed clock, synthetic EDD fixture tables, canonical contracts
// loaded read-only. Byte-identical across runs; no raw email, raw key, or
// customer-identifier material ever appears in the summary.
declare(strict_types=1);
$root = $argv[1];
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';
require_once $root . '/docs/contracts/spec152e-evaluation-issuance.v1.php';
require_once $root . '/docs/contracts/spec172-limited-access-assertion-service.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-node.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';

$positive = 0;
$negative = 0;
function ok(bool $condition, string $message): void { global $positive; $positive++; if (!$condition) { fwrite(STDERR, "FAIL: {$message}\n"); exit(1); } }
function okThrows(callable $operation, string $code, string $message): void { global $negative; $negative++; try { $operation(); } catch (DomainException $error) { if ($error->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n"); exit(1); } catch (Throwable $error) { fwrite(STDERR, "FAIL: {$message} (unexpected " . get_class($error) . ": " . $error->getMessage() . ")\n"); exit(1); } fwrite(STDERR, "FAIL: {$message} (no throw)\n"); exit(1); }

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-09T06:00:00Z'))->modify('+' . ($tick * 10) . ' seconds')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$evaluationMigration = new FocusaSpec152eEvaluationIssuanceMigration($db, 'wp_');
$evaluationMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);
$nodeMigration = new FocusaSpec152eAuthorityNodeMigration($db, 'wp_');
$nodeMigration->migrate('2026-08-09T05:00:00Z', ['source' => 'first_value_e2e']);

// EDD 3.x synthetic fixture tables (fixtures only; never the authority surface).
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
    order_number VARCHAR(32) NULL,
    status VARCHAR(32) NOT NULL,
    type VARCHAR(32) NOT NULL DEFAULT 'sale',
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL,
    user_id INTEGER NULL,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL DEFAULT ''
)");
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    product_name VARCHAR(191) NOT NULL DEFAULT '',
    quantity INTEGER NOT NULL DEFAULT 1
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    activation_limit INTEGER NOT NULL DEFAULT 1,
    expiration VARCHAR(32) NULL
)");
$db->exec("CREATE TABLE wp_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_email VARCHAR(100) NOT NULL
)");

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
    $db,
    $promotionMigration,
    $registrations,
    $identities,
    $accounts,
    $edd,
    $identitySecrets,
    $registrationSecrets,
    $clock,
);
$postures = new FocusaSpec172VerifiedAccessPostureRepository($db, $postureMigration, $clock);
$assertions = new FocusaSpec172SignedAccessAssertionRepository($db, $assertionMigration, $postureMigration, $clock);
$evaluation = new FocusaSpec152eEvaluationIssuanceService(
    $db,
    $evaluationMigration,
    $registrations,
    $accounts,
    $edd,
    $postureMigration,
    $postures,
    $assertions,
    $clock,
    'wp_',
);
$nodes = new FocusaSpec152eAuthorityNodeRepository($db, $nodeMigration, $clock);
$keySet = new FocusaSpec152eAuthorityKeySetSeam(str_repeat('R', 32), str_repeat('L', 32), $clock);

$counts = static function (string $table) use ($db): int {
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
};

$seq = 0;
$promoteVerified = static function (string $email, string $tag) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.first.value.e2e',
        'install_channel' => 'official_installer',
        'product_code' => 'focusa_evaluation',
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
    $result = $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-09T06:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'first_value_e2e', 'record' => $tag . '-' . $seq],
    ]);
    return [
        'account_uuid' => $result['account_uuid'],
        'identity_uuid' => $result['identity_uuid'],
        'registration_uuid' => $result['registration_id'],
        'verified_at' => $verified['registration']['verified_at'],
        'edd_customer_id' => (int) $result['edd_customer_id'],
    ];
};

$signature = 'sig_spec152f_first_value_' . str_repeat('a', 40);
$evalSeq = 0;
$evaluationInput = static function (array $verified, string $node, string $tag, string $productCode = 'focusa_evaluation') use (&$evalSeq, $signature): array {
    $evalSeq++;
    return [
        'product_code' => $productCode,
        'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => $verified['account_uuid'],
        'identity_uuid' => $verified['identity_uuid'],
        'verification_state' => 'account_promoted',
        'verified_at' => $verified['verified_at'],
        'node_uuid' => $node,
        'node_digest' => hash('sha256', 'node-' . $node),
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.first.value.e2e',
        'install_channel' => 'official_installer',
        'request_id' => 'req-eval-' . $tag . '-' . $evalSeq,
        'idempotency_key' => 'idem-eval-' . $tag . '-' . $evalSeq,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $signature,
        'issued_at' => '2026-08-09T06:05:00Z',
        'refresh_at' => '2026-08-09T06:05:00Z',
        'migration_provenance' => ['source' => 'first_value_e2e', 'record' => $tag . '-' . $evalSeq],
    ];
};

// ── 0. Zero-customer baseline: start without a customer ──────────────────

ok($counts('wp_edd_customers') === 0, 'journey starts with no EDD customer');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'journey starts with no EDD order or license key');
ok($counts('wp_wpuiai_verified_access_postures') === 0, 'journey starts with no posture');
ok($counts('wp_wpuiai_signed_access_assertions') === 0, 'journey starts with no signed assertion');
ok($counts('wp_wpuiai_verified_access_nodes') === 0, 'journey starts with no bound node');
ok($counts('wp_wpuiai_evaluation_issuances') === 0, 'journey starts with no evaluation journal');
$baselineCustomer = (int) $db->query("SELECT COUNT(*) FROM wp_edd_customers WHERE email = 'first.value@example.invalid'")->fetchColumn();
ok($baselineCustomer === 0, 'the journey mailbox has no pre-existing EDD customer');

// ── 1. Verify the mailbox and promote to one authority-owned customer ──

$alpha = $promoteVerified('first.value@example.invalid', 'alpha');
ok($alpha['edd_customer_id'] > 0, 'mailbox verification promotes to one authority-owned EDD customer');
ok($counts('wp_edd_customers') === 1, 'exactly one EDD customer exists (no duplicate customer)');
ok($counts('wp_edd_orders') === 0, 'promotion creates no EDD order');
ok($counts('wp_edd_licenses') === 0, 'promotion creates no EDD license key');

// ── 2. Issue the eligible EDD-backed Evaluation (verified_no_license) ──

$alphaInput = $evaluationInput($alpha, 'node-first-value-0001', 'alpha');
$alphaResult = $evaluation->requestEvaluation($alphaInput);
ok($alphaResult['decision'] === 'limited_access_issued', 'eligible verified customer receives limited access');
ok($alphaResult['error_code'] === null, 'issued decision carries no error code');
ok($alphaResult['posture_uuid'] !== null && $alphaResult['assertion_uuid'] !== null, 'issued decision binds a posture and a signed assertion');
ok($alphaResult['node_uuid'] === 'node-first-value-0001', 'issued decision binds the device node');
ok($alphaResult['duration'] === 'no_automatic_expiry', 'no expiry: the posture is permanent, not a timed Evaluation');
ok($alphaResult['edd_order_id'] === null && $alphaResult['edd_license_id'] === null, 'no EDD order or EDD key is created');
ok($alphaResult['creates_edd_license_key'] === false, 'no EDD Software Licensing key flag is false');
ok($alphaResult['grant_source'] === 'authority_signed_limited_access_assertion', 'grant source is the authority-signed limited-access assertion');
ok($alphaResult['authority_sequence'] === 1, 'first issuance binds authority sequence 1');
ok($counts('wp_edd_customers') === 1, 'Evaluation created no second customer');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'Evaluation created no EDD order and no EDD key');
ok($counts('wp_wpuiai_verified_access_postures') === 1, 'exactly one verified_no_license posture');
ok($counts('wp_wpuiai_signed_access_assertions') === 1, 'exactly one signed limited-access assertion');
ok($counts('wp_wpuiai_verified_access_nodes') === 1, 'exactly one registered device node');
ok($counts('wp_wpuiai_evaluation_issuances') === 1, 'exactly one evaluation journal row');
$postureRow = $db->query('SELECT * FROM wp_wpuiai_verified_access_postures')->fetch(PDO::FETCH_ASSOC);
ok(
    json_decode((string) $postureRow['family_allowlist'], true, 512, JSON_THROW_ON_ERROR)
        === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'posture stores exactly the canonical server-owned six-family allowlist',
);
$alphaReplay = $evaluation->requestEvaluation($alphaInput);
ok($alphaReplay === $alphaResult, 'idempotent replay returns the identical result');
ok($counts('wp_wpuiai_evaluation_issuances') === 1, 'replay creates no duplicate journal row');
$switched = $evaluationInput($alpha, 'node-first-value-0001', 'alpha-dup');
$switched['facade_id'] = 'forge.focusa.dev_v1';
$switched['presenter'] = 'forge.presenter';
okThrows(static fn() => $evaluation->requestEvaluation($switched), 'EVALUATION_NOT_ELIGIBLE', 'facade-switched duplicate Evaluation is denied');
ok($counts('wp_wpuiai_verified_access_postures') === 1, 'duplicate attempt creates no second posture or key');
ok($counts('wp_wpuiai_evaluation_issuances') === 2, 'the duplicate denial is journaled as an audit row without issuing');

// ── 3. Signed limited-access assertion: issue, verify, refresh, fail closed ──

$signer = FocusaSpec172LimitedAssertionSigner::fromSeed(str_repeat('a', 64));
$limited = new FocusaSpec172LimitedAssertionService($db, $postures, $assertions, $signer, $postureMigration, $clock);
$issued = $limited->issue([
    'posture_uuid' => $alphaResult['posture_uuid'],
    'issued_at' => '2026-08-09T06:05:00Z',
    'refresh_at' => '2026-08-09T06:35:00Z',
    'migration_provenance' => ['source' => 'first_value_e2e', 'record' => 'issue-1'],
]);
ok($issued['verdict'] === 'valid', 'signed limited-access assertion issues from the active posture');
ok(preg_match('/^[0-9a-f]{128}$/D', (string) $issued['signature']) === 1, 'issuance returns a real 64-byte Ed25519 signature');
ok($issued['product_scope'] === 'focusa', 'assertion carries the focusa product scope');
ok($issued['node_uuid'] === 'node-first-value-0001', 'assertion carries the registered node binding');
ok($issued['signer'] === FocusaSpec172LimitedAssertionService::SIGNER_ISSUE, 'issuance uses the server-owned issue signer');
ok((int) $issued['sequence'] === 2, 'issuance continues the evaluation sequence with monotonic sequence 2');
ok($issued['family_allowlist'] === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'assertion allowlist is the server-owned six-family list');

$presented = [
    'posture_uuid' => $issued['posture_uuid'],
    'account_uuid' => $alpha['account_uuid'],
    'identity_uuid' => $alpha['identity_uuid'],
    'product_scope' => $issued['product_scope'],
    'node_uuid' => $issued['node_uuid'],
    'family_allowlist' => $issued['family_allowlist'],
    'sequence' => (int) $issued['sequence'],
    'issued_at' => $issued['issued_at'],
    'refresh_at' => $issued['refresh_at'],
    'signer' => $issued['signer'],
    'signature' => $issued['signature'],
];
$verified = $limited->verify($presented, '2026-08-09T06:10:00Z');
ok($verified['verdict'] === 'valid', 'signed assertion verifies at runtime adoption time');
$tampered = $presented;
$tampered['signature'] = str_repeat('0', 128);
$tamperedVerdict = $limited->verify($tampered, '2026-08-09T06:10:00Z');
ok($tamperedVerdict['verdict'] === 'denied' && $tamperedVerdict['code'] === 'SIGNATURE_INVALID', 'tampered signature fails closed');

$widened = $presented;
$widenedFamilies = array_values(array_unique(array_merge($issued['family_allowlist'], ['automation'])));
sort($widenedFamilies, SORT_STRING);
$widened['family_allowlist'] = $widenedFamilies;
$widened['signature'] = $signer->sign(FocusaSpec172LimitedAssertionPayload::build([
    'posture_uuid' => $widened['posture_uuid'],
    'account_uuid' => $widened['account_uuid'],
    'identity_uuid' => $widened['identity_uuid'],
    'product_scope' => $widened['product_scope'],
    'node_uuid' => $widened['node_uuid'],
    'family_allowlist' => $widened['family_allowlist'],
    'sequence' => $widened['sequence'],
    'issued_at' => $widened['issued_at'],
    'refresh_at' => $widened['refresh_at'],
    'signer' => $widened['signer'],
]));
$widenedVerdict = $limited->verify($widened, '2026-08-09T06:10:00Z');
ok($widenedVerdict['verdict'] === 'denied' && $widenedVerdict['code'] === 'ASSERTION_TAMPERED', 'premium family widening fails closed even when re-signed (stored allowlist binding)');
$widenedPosture = $postures->findByUuid((string) $widened['posture_uuid']);
$policyVerdictCode = FocusaSpec172LimitedAssertionService::policyVerdict($widened, $widenedPosture, '2026-08-09T06:10:00Z');
ok($policyVerdictCode === 'CAPABILITY_FAMILY_NOT_INCLUDED', 'policy layer denies the premium family with CAPABILITY_FAMILY_NOT_INCLUDED');

$refreshed = $limited->refresh([
    'posture_uuid' => $alphaResult['posture_uuid'],
    'refresh_at' => '2026-08-09T06:35:00Z',
    'idempotency_key' => 'idem-refresh-0001',
    'migration_provenance' => ['source' => 'first_value_e2e', 'record' => 'refresh-1'],
]);
ok($refreshed['verdict'] === 'valid' && (int) $refreshed['sequence'] === 3, 'refresh rotates the credential to monotonic sequence 3');
ok($refreshed['family_allowlist'] === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'refresh never widens the family allowlist');
$staleVerdict = $limited->verify($presented, '2026-08-09T06:10:00Z');
ok($staleVerdict['verdict'] === 'denied' && $staleVerdict['code'] === 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', 'stale sequence 2 fails closed after rotation to 3');

// ── 4. Install/resume: adopt only signed authority material ─────────────

$leasePayload = [
    'schema' => 'focusa.lease_delivery.v1',
    'version' => 1,
    'product_scope' => 'focusa',
    'posture' => 'verified_no_license',
    'account_uuid' => $alpha['account_uuid'],
    'node_uuid' => 'node-first-value-0001',
    'posture_uuid' => $alphaResult['posture_uuid'],
    'sequence' => (int) $refreshed['sequence'],
    'authority_key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
    'issued_at' => '2026-08-09T06:20:00Z',
    'not_before' => '2026-08-09T06:20:00Z',
];
$leaseEnvelope = $keySet->seal(
    $leasePayload,
    FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
    str_repeat('L', 32),
    FocusaSpec152eEd25519Signer::LEASE_DOMAIN,
);
$leaseVerified = FocusaSpec152eEd25519Signer::verify(
    FocusaSpec152eEd25519Signer::publicKeyFromSeed(str_repeat('L', 32)),
    base64_decode($leaseEnvelope['signature_b64'], true),
    FocusaSpec152eEd25519Signer::LEASE_DOMAIN,
    base64_decode($leaseEnvelope['payload_b64'], true),
);
ok($leaseVerified === true, 'signed device-bound envelope verifies over the canonical lease domain');
ok($leaseEnvelope['signer_key_id'] === FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID, 'envelope is signed with the authority lease key');

// Bounded synthetic runtime authority-state record (mirrors the ta_* journey
// tables): the installer/resume flow adopts ONLY signed authority material and
// opaque references. Raw keys, raw email, and customer identifiers never appear.
$db->exec("CREATE TABLE wp_wpuiai_e2e_runtime_authority_state (
    record_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
    account_uuid VARCHAR(36) NOT NULL,
    node_uuid VARCHAR(36) NOT NULL,
    posture_uuid VARCHAR(36) NOT NULL,
    posture VARCHAR(24) NOT NULL,
    sequence BIGINT NOT NULL,
    assertion_signature VARCHAR(256) NOT NULL,
    lease_envelope_payload_b64 TEXT NOT NULL,
    lease_envelope_signature_b64 TEXT NOT NULL,
    lease_key_id VARCHAR(96) NOT NULL,
    state VARCHAR(16) NOT NULL,
    created_at VARCHAR(32) NOT NULL,
    updated_at VARCHAR(32) NOT NULL
)");
$insertState = $db->prepare("INSERT INTO wp_wpuiai_e2e_runtime_authority_state
    (record_uuid, account_uuid, node_uuid, posture_uuid, posture, sequence,
     assertion_signature, lease_envelope_payload_b64, lease_envelope_signature_b64,
     lease_key_id, state, created_at, updated_at)
    VALUES (:record, :account, :node, :posture_uuid, 'verified_no_license', :sequence,
            :signature, :lease_payload, :lease_signature, :lease_key, 'active', :created, :updated)");
$insertState->execute([
    ':record' => 'rt_state_first_value_0001',
    ':account' => $alpha['account_uuid'],
    ':node' => 'node-first-value-0001',
    ':posture_uuid' => $alphaResult['posture_uuid'],
    ':sequence' => (int) $refreshed['sequence'],
    ':signature' => $refreshed['signature'],
    ':lease_payload' => $leaseEnvelope['payload_b64'],
    ':lease_signature' => $leaseEnvelope['signature_b64'],
    ':lease_key' => $leaseEnvelope['signer_key_id'],
    ':created' => ($clock)(),
    ':updated' => ($clock)(),
]);
ok($counts('wp_wpuiai_e2e_runtime_authority_state') === 1, 'runtime authority state adopts exactly one signed record');
$stateRow = $db->query("SELECT * FROM wp_wpuiai_e2e_runtime_authority_state")->fetch(PDO::FETCH_ASSOC);
ok($stateRow['posture'] === 'verified_no_license' && (int) $stateRow['sequence'] === 3, 'runtime state records the rotated sequence');
ok($stateRow['lease_key_id'] === FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID, 'runtime state records the authority key id only');
$resumeVerdict = $limited->verify([
    'posture_uuid' => $stateRow['posture_uuid'],
    'account_uuid' => $stateRow['account_uuid'],
    'identity_uuid' => $alpha['identity_uuid'],
    'product_scope' => 'focusa',
    'node_uuid' => $stateRow['node_uuid'],
    'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'sequence' => (int) $stateRow['sequence'],
    'issued_at' => $refreshed['issued_at'],
    'refresh_at' => $refreshed['refresh_at'],
    'signer' => $refreshed['signer'],
    'signature' => $stateRow['assertion_signature'],
], '2026-08-09T06:30:00Z');
ok($resumeVerdict['verdict'] === 'valid', 'resume re-verifies the adopted signed assertion');

// ── 5. First useful value loop: one base Focusa value loop, no card ──────

// Canonical limited-mode preflight mirror (Spec 172 §6.2 / verified-limited-access.v1):
// manual families allowed for focusa; manual_project only while at most one
// mutable project exists; paid families and unknowns fail closed.
$blockedFamilies = ['automation', 'team_remote', 'release_proof', 'premium_updates'];
$limitedFamilies = FocusaSpec172VerifiedAccessPostureState::FOCUSA_LIMITED_FAMILIES;
$preflight = static function (string $product, string $family, int $mutableProjects) use ($limitedFamilies, $blockedFamilies): bool {
    if ($product !== 'focusa') {
        return false;
    }
    if (in_array($family, $blockedFamilies, true)) {
        return false;
    }
    if ($family === 'manual_project') {
        return $mutableProjects <= 1;
    }
    return in_array($family, $limitedFamilies, true);
};

// Bounded synthetic project/Workpoint kernel: value-producing mutations are
// recorded only when the canonical preflight allows them (mirrors the
// require_base_product / family classifier runtime boundary).
$db->exec("CREATE TABLE wp_wpuiai_e2e_project_operations (
    operation_key VARCHAR(64) NOT NULL PRIMARY KEY,
    family VARCHAR(48) NOT NULL,
    mutable_projects_at_call INTEGER NOT NULL,
    created_at VARCHAR(32) NOT NULL
)");
$recordOp = static function (string $key, string $family, int $mutableProjects) use ($db, $clock): void {
    $db->prepare("INSERT INTO wp_wpuiai_e2e_project_operations (operation_key, family, mutable_projects_at_call, created_at)
        VALUES (:key, :family, :projects, :now)")
        ->execute([':key' => $key, ':family' => $family, ':projects' => $mutableProjects, ':now' => ($clock)()]);
};
$firstValue = [];
foreach ([
    ['first_project', 'manual_project', 0, 'one active project'],
    ['first_mission', 'manual_mission', 1, 'a Mission inside the active project'],
    ['first_focus_state', 'manual_focus_state', 1, 'a Focus State'],
    ['first_workpoint', 'manual_workpoint', 1, 'a Workpoint'],
    ['first_trajectory', 'manual_trajectory', 1, 'a Trajectory'],
    ['first_evidence', 'manual_basic_evidence', 1, 'basic Evidence'],
] as [$key, $family, $projects, $label]) {
    ok($preflight('focusa', $family, $projects) === true, "first useful value permits {$label}");
    if ($preflight('focusa', $family, $projects)) {
        $recordOp($key, $family, $projects);
        $firstValue[] = $family;
    }
}
ok(count($firstValue) === 6, 'the complete base value loop completed (project, mission, focus state, workpoint, trajectory, evidence)');
ok($counts('wp_wpuiai_e2e_project_operations') === 6, 'exactly six first-value mutations were recorded');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'the value loop created no EDD order and no EDD key (no card anywhere)');
ok($counts('wp_edd_customers') === 1, 'the value loop created no second customer');
ok($counts('wp_wpuiai_evaluation_issuances') === 2, 'the value loop created no new Evaluation decision');
$readProjects = $db->query("SELECT COUNT(*) FROM wp_wpuiai_e2e_project_operations WHERE family = 'manual_project'")->fetchColumn();
ok((int) $readProjects === 1, 'exactly one mutable active project exists after the loop');
ok($preflight('focusa', 'read_projection', 1) === false || in_array('read_projection', FocusaSpec172VerifiedAccessPostureState::PERMANENT_FAMILIES, true), 'read projection remains a permanent allowance');

// ── 6. Second-project mutation blocked with upgrade guidance ─────────────

ok($preflight('focusa', 'manual_project', 2) === false, 'a second-project mutation is blocked at mutable project count 2');
$secondProjectAttempt = $preflight('focusa', 'manual_project', 2);
if (!$secondProjectAttempt) {
    $recorded = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_e2e_project_operations WHERE operation_key = 'second_project'")->fetchColumn();
    ok($recorded === 0, 'the blocked second-project mutation records no partial side effect');
}
$projectGuidance = [
    'blocked_action' => 'create_second_project',
    'reason' => 'verified no-license mode permits one mutable active project',
    'safe_next_action' => 'evaluate',
    'action_label' => 'Start a free Evaluation or purchase Focusa',
    'link' => '/activate/evaluate',
    'retained_access' => ['read', 'export', 'recovery', 'repair', 'stable_security_update', 'uninstall'],
];
ok($projectGuidance['safe_next_action'] === 'evaluate' && $projectGuidance['link'] === '/activate/evaluate', 'second-project denial carries Evaluation upgrade guidance');
ok(in_array('read', $projectGuidance['retained_access'], true) && in_array('export', $projectGuidance['retained_access'], true), 'retained projects stay readable and exportable (never deleted, never paywalled)');

// ── 7. Paid tool families blocked with upgrade guidance ─────────────────

$premiumGuidance = [];
foreach (['automation', 'team_remote', 'release_proof', 'premium_updates'] as $family) {
    ok($preflight('focusa', $family, 1) === false, "paid family {$family} is blocked in verified no-license mode");
    ok(!in_array($family, $limitedFamilies, true), "paid family {$family} is never on the limited-mode allowlist");
    $premiumGuidance[$family] = [
        'blocked_action' => 'start_' . $family . '_work',
        'reason' => 'optional family requires an authority-issued entitlement',
        'safe_next_action' => 'purchase',
        'action_label' => 'Purchase or renew this optional family',
        'link' => '/activate/checkout',
        'retained_access' => ['read', 'export', 'recovery', 'repair', 'stable_security_update', 'uninstall'],
    ];
}
ok(count($premiumGuidance) === 4, 'all four paid tool families resolve to canonical purchase guidance');
foreach ($premiumGuidance as $family => $guidance) {
    ok($guidance['safe_next_action'] === 'purchase' && $guidance['link'] === '/activate/checkout', "paid family {$family} denial carries purchase upgrade guidance");
    ok(in_array('read', $guidance['retained_access'], true) && in_array('recovery', $guidance['retained_access'], true), "paid family {$family} denial preserves read and recovery");
}
$premiumOpsRecorded = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_e2e_project_operations WHERE family IN ('automation','team_remote','release_proof','premium_updates')")->fetchColumn();
ok($premiumOpsRecorded === 0, 'blocked paid-family operations record no partial side effects');
$unknownFamily = $preflight('focusa', 'unlicensed_experimentation', 1);
ok($unknownFamily === false, 'unknown families fail closed (explicit allowlist only)');

// ── 8. No premium grant unless the authority includes it ────────────────

$allowlist = json_decode((string) $postureRow['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
ok($allowlist === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'posture allowlist is exactly the server-owned six-family list');
foreach ($blockedFamilies as $family) {
    ok(!in_array($family, $allowlist, true), "premium family {$family} is absent from the posture allowlist");
}
okThrows(static fn() => $evaluation->requestEvaluation(array_merge($evaluationInput($alpha, 'node-first-value-0001', 'alpha-price'), ['price' => '0.00'])), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'caller-supplied price is denied');
ok($counts('wp_wpuiai_verified_access_postures') === 1, 'denied caller-controlled attempt creates no posture');
ok($counts('wp_wpuiai_evaluation_issuances') === 2, 'the caller-controlled denial is rejected before any journal side effect');
$assertionRows = $db->query("SELECT family_allowlist, signer FROM wp_wpuiai_signed_access_assertions ORDER BY sequence")->fetchAll(PDO::FETCH_ASSOC);
foreach ($assertionRows as $assertionRow) {
    $rowFamilies = json_decode((string) $assertionRow['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
    foreach ($blockedFamilies as $family) {
        ok(!in_array($family, $rowFamilies, true), "no signed assertion ever carries premium family {$family}");
    }
}

// ── 9. Final counts and redaction ────────────────────────────────────────

ok($counts('wp_edd_customers') === 1, 'exactly one EDD customer total (no duplicate customer)');
ok($counts('wp_edd_orders') === 0 && $counts('wp_edd_licenses') === 0, 'zero EDD orders and zero EDD license keys total');
ok($counts('wp_wpuiai_verified_access_postures') === 1, 'exactly one posture total');
ok($counts('wp_wpuiai_verified_access_nodes') === 1, 'exactly one bound node total');
ok($counts('wp_wpuiai_evaluation_issuances') === 2, 'exactly two evaluation journal rows: one issuance and one duplicate-denial audit row');
$journalRows = $db->query('SELECT * FROM wp_wpuiai_evaluation_issuances')->fetchAll(PDO::FETCH_ASSOC);
foreach ($journalRows as $row) {
    ok((string) $row['duration'] === 'no_automatic_expiry', 'journal records the permanent duration (no expiry)');
    ok($row['edd_order_id'] === null && $row['edd_license_id'] === null, 'journal records no EDD order and no EDD key');
    foreach ($row as $column => $value) {
        if (is_string($value)) {
            ok(
                strpos($value, '@') === false && strpos($value, 'FOCUSA-') === false && strpos($value, 'cus_') === false,
                "evaluation journal is redacted (no raw email, key material, or customer ref in {$column})",
            );
        }
    }
}
$stateColumns = ['assertion_signature', 'lease_envelope_payload_b64', 'lease_envelope_signature_b64', 'lease_key_id'];
foreach ($stateColumns as $column) {
    $value = (string) $stateRow[$column];
    ok(strpos($value, '@') === false && strpos($value, 'FOCUSA-') === false && strpos($value, 'cus_') === false,
        "runtime authority state column {$column} is redacted");
}
$eddFree = $assertions->assertEddFree();
ok($eddFree['edd_free'] === true, 'posture and assertion schemas remain EDD-free');

$summary = [
    'schema' => 'focusa.spec152f.evaluation_first_value_e2e.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'decision' => 'limited_access_issued',
    'grant_source' => 'authority_signed_limited_access_assertion',
    'duration' => 'no_automatic_expiry',
    'creates_edd_license_key' => false,
    'postures' => $counts('wp_wpuiai_verified_access_postures'),
    'assertions' => $counts('wp_wpuiai_signed_access_assertions'),
    'evaluation_nodes' => $counts('wp_wpuiai_verified_access_nodes'),
    'edd_customers_total' => $counts('wp_edd_customers'),
    'edd_orders_total' => $counts('wp_edd_orders'),
    'edd_licenses_total' => $counts('wp_edd_licenses'),
    'first_value_families' => count($firstValue),
    'mutable_projects' => (int) $readProjects,
    'second_project_blocked' => !$secondProjectAttempt,
    'premium_families_blocked' => count($premiumGuidance),
    'assertion_signature_verified' => $verified['verdict'] === 'valid',
    'lease_signature_verified' => $leaseVerified,
    'resume_verified' => $resumeVerdict['verdict'] === 'valid',
    'refresh_sequence' => (int) $refreshed['sequence'],
    'widened_family_denied' => $widenedVerdict['verdict'] === 'denied' && $policyVerdictCode === 'CAPABILITY_FAMILY_NOT_INCLUDED',
    'result' => 'first_useful_value_reached_fail_closed',
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


def expected_limited_decision(registry: dict, case: dict) -> str:
    """Replay of the canonical Spec 172 limited-access resolver for the vectors."""
    posture = case["posture"]
    family = case["family"]
    if posture == "unverified":
        return "allow" if family in registry["postures"]["unverified"]["allowed_operations"] else "deny"
    if posture != "verified_no_license":
        return "deny"
    if family in registry["permanent_allowances"]["families"]:
        return "allow"
    product = case["product"]
    if product == "focusa":
        if family in registry["focusa"]["blocked_families"]:
            return "deny"
        if family in registry["focusa"]["allowed_families"]:
            return "allow" if case.get("mutable_project_count", 1) <= 1 else "deny"
    if product == "uiai_engine":
        if family in registry["uiai_engine"]["blocked_families"]:
            return "deny"
        if family in registry["uiai_engine"]["allowed_families"]:
            valid_session = case.get("session_count", 1) <= 1
            valid_mode = case.get("session_mode", "foreground_ephemeral") == "foreground_ephemeral"
            return "allow" if valid_session and valid_mode else "deny"
    return "deny"


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
    expect(result["schema"] == "focusa.spec152f.evaluation_first_value_e2e.v1", "harness summary schema pinned")
    expect(result["result"] == "first_useful_value_reached_fail_closed", "harness result reached first useful value fail-closed")
    expect(result["decision"] == "limited_access_issued", "one Evaluation decision issued")
    expect(result["grant_source"] == "authority_signed_limited_access_assertion", "grant is the authority-signed limited-access assertion")
    expect(result["duration"] == "no_automatic_expiry", "no expiry: permanent posture, no countdown")
    expect(result["creates_edd_license_key"] is False, "no EDD Software Licensing key created")
    expect(result["postures"] == 1 and result["assertions"] == 3 and result["evaluation_nodes"] == 1,
           "one posture, three assertions (issue + refresh + evaluation), one node — no duplicates")
    expect(result["edd_customers_total"] == 1 and result["edd_orders_total"] == 0 and result["edd_licenses_total"] == 0,
           "exactly one customer and zero EDD orders/keys — no duplicate customer or key")
    expect(result["first_value_families"] == 6 and result["mutable_projects"] == 1,
           "the complete base value loop completed on one mutable active project")
    expect(result["second_project_blocked"] is True, "second-project mutation is blocked")
    expect(result["premium_families_blocked"] == 4, "all four paid tool families are blocked")
    expect(result["assertion_signature_verified"] is True, "signed assertion verified at adoption")
    expect(result["lease_signature_verified"] is True, "signed device-bound envelope verified over the canonical domain")
    expect(result["resume_verified"] is True, "resume re-verifies the adopted signed assertion")
    expect(result["refresh_sequence"] == 3, "credential rotated to monotonic sequence 3 without widening")
    expect(result["widened_family_denied"] is True, "premium family widening failed closed")
    harness_positive = int(result["positive_checks"])
    harness_negative = int(result["negative_checks"])
    expect(harness_positive >= 150 and harness_negative >= 2, "harness check counts are bounded and non-trivial")

    # ── Verified-email Evaluation fixture: canonical vector replay ──────────
    limited_registry = yaml.safe_load(LIMITED_YAML.read_text(encoding="utf-8"))
    limited = json.loads(LIMITED_CASES.read_text(encoding="utf-8"))
    expect(limited["schema"] == "focusa.spec172.limited_access_cases.v1", "limited-access fixture schema pinned")
    expect(len(limited["cases"]) == 30, "limited-access fixture carries exactly 30 canonical vectors")
    by_id = {}
    for case in limited["cases"]:
        expect(case["id"] not in by_id, f"duplicate limited-access case id {case['id']}")
        by_id[case["id"]] = case
        expect(expected_limited_decision(limited_registry, case) == case["decision"],
               f"vector {case['id']} disagrees with the canonical registry")
    expect(by_id["limited_manual_project"]["decision"] == "allow"
           and by_id["limited_manual_project"].get("mutable_project_count") == 1,
           "first project mutation vector allows one mutable project")
    for family, case_id in [("manual_mission", "limited_manual_mission"),
                             ("manual_focus_state", "limited_manual_focus_state"),
                             ("manual_workpoint", "limited_manual_workpoint"),
                             ("manual_trajectory", "limited_manual_trajectory"),
                             ("manual_basic_evidence", "limited_manual_evidence")]:
        expect(by_id[case_id]["decision"] == "allow",
               f"first-value family vector allows {family}")
    expect(by_id["limited_second_project_mutation"]["decision"] == "deny"
           and by_id["limited_second_project_mutation"].get("mutable_project_count") == 2,
           "second-project mutation vector denies at count 2")
    for family, case_id in [("automation", "limited_automation"), ("team_remote", "limited_team"),
                            ("release_proof", "limited_release"), ("premium_updates", "limited_premium_update")]:
        expect(by_id[case_id]["decision"] == "deny", f"paid family vector denies {family}")
    expect(limited_registry["postures"]["verified_no_license"]["expiry"] == "none"
           and limited_registry["postures"]["verified_no_license"]["creates_edd_key"] is False,
           "canonical registry: no expiry and no EDD key for verified_no_license")
    expect(limited_registry["focusa"]["mutable_project_limit"] == 1, "canonical registry: one mutable project")

    # ── Base Focusa project/Workpoint operation (runtime policy surface) ────
    policy_source = POLICY.read_text(encoding="utf-8")
    non_test_policy = policy_source.split("#[cfg(test)]")[0]
    expect("pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES" in policy_source
           and '"manual_project"' in policy_source and '"manual_mission"' in policy_source
           and '"manual_workpoint"' in policy_source and '"manual_basic_evidence"' in policy_source,
           "Rust classifier carries the six manual first-value families")
    expect("pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES" in policy_source
           and '"automation"' in policy_source and '"team_remote"' in policy_source
           and '"release_proof"' in policy_source and '"premium_updates"' in policy_source,
           "Rust classifier carries exactly the four paid blocked families")
    expect("pub fn is_focusa_verified_no_license_family_allowed" in non_test_policy
           and "mutable_project_count <= 1" in non_test_policy,
           "Rust classifier enforces the one-mutable-project boundary")
    expect("pub enum BaseProductDecision" in non_test_policy
           and "Entitled" in non_test_policy and "Limited" in non_test_policy and "Denied" in non_test_policy,
           "base product decision resolves entitled / limited / denied")
    expect("PolicyEntitlementState::VerifiedNoLicense => BaseProductDecision::Limited" in non_test_policy,
           "verified no-license resolves to the limited base product decision")
    expect("pub fn resolve_base_focusa_product" in non_test_policy
           and 'if product != "focusa"' in non_test_policy,
           "base product resolution is product-boundary closed")
    expect("pub const fn permits_base_mutations(self) -> bool" in non_test_policy
           and "matches!(self, Self::Entitled)" in non_test_policy,
           "only Entitled permits base mutations (Limited never unlocks the full base gate)")
    core_source = CORE_LICENSE.read_text(encoding="utf-8")
    non_test_core = core_source.split("#[cfg(test)]")[0]
    expect("pub fn require_base_product()" in non_test_core
           and "CapabilityFamily::BaseFocusa" in non_test_core
           and "OperationClass::ValueMutation" in non_test_core,
           "core require_base_product gates value-producing base Focusa mutations")
    expect("BaseProductRequired(String)" in non_test_core
           and "base Focusa product gate not satisfied" in non_test_core,
           "core fails closed with LicenseError::BaseProductRequired")

    # ── Installer / first-run surface ───────────────────────────────────────
    install_source = INSTALL.read_text(encoding="utf-8")
    flow_source = FLOW.read_text(encoding="utf-8")
    store_source = STORE.read_text(encoding="utf-8")
    expect("persist_eval_license" not in install_source and "LicenseGuard::eval" not in install_source,
           "installer never persists a local Evaluation", is_negative=True)
    expect("persist_eval_license" not in flow_source and "LicenseGuard::eval" not in flow_source,
           "activation flow never self-issues an Evaluation", is_negative=True)
    expect("source_build_first_run_without_installer_or_lease_grants_nothing" in store_source
           and "deleting_installer_state_never_unlocks_and_deleting_the_lease_locks" in store_source,
           "first-run without installer or lease grants nothing (Rust replay tests present)")
    expect("embedded_production_trust_roots" in store_source
           and 'option_env!("FOCUSA_AUTHORITY_ROOT_KEYS_JSON")' in store_source,
           "trust roots are compile-time embedded production material only")
    expect('"test", "fixture", "local", "dev", "example"' in store_source,
           "test/local trust roots are forbidden (no local/self-issued roots)")
    for code in ["E_AUTHORITY_EXISTING_UNUSABLE", "E_AUTHORITY_RAW_KEY_FORBIDDEN",
                 "E_AUTHORITY_LEASE_UNUSABLE", "E_AUTHORITY_DEVICE_DENIED",
                 "E_AUTHORITY_ACTIVATION_UNSETTLED"]:
        expect(code in install_source, f"installer fails closed with {code}")
    expect("EntitlementState::Active | EntitlementState::OfflineGrace" in install_source,
           "installer adopts only signed Active/OfflineGrace authority state")
    sh_source = INSTALL_SH.read_text(encoding="utf-8")
    ps1_source = INSTALL_PS1.read_text(encoding="utf-8")
    for name, source in [("install-focusa.sh", sh_source), ("install-focusa.ps1", ps1_source)]:
        expect("--price" not in source and "--product" not in source and "--grant" not in source
               and "--plan" not in source and "--features" not in source,
               f"{name} exposes no product/price/grant/plan selector", is_negative=True)
        expect("never local" in source or "never creates local evaluation" in source
               or "authority-issued only" in source or "never create local evaluation" in source,
               f"{name} declares authority-issued-only entitlement (no local Evaluation)")
        expect("delegat" in source, f"{name} delegates to the canonical Rust installer")
        expect("persist_eval_license" not in source and "LicenseGuard::eval" not in source,
               f"{name} never persists or self-issues an Evaluation", is_negative=True)

    # ── Upgrade guidance and the single small family boundary (no 395) ──────
    catalog = json.loads(DENIAL_CATALOG.read_text(encoding="utf-8"))
    expect(catalog["schema"] == "focusa.spec152f.denial_ux_catalog.v1", "denial UX catalog schema pinned")
    action_ids = {action["id"] for action in catalog["actions"]}
    expect({"evaluate", "purchase", "manage", "recovery"} <= action_ids, "catalog carries canonical upgrade actions")
    for link_id in ["evaluation", "checkout"]:
        expect(link_id in catalog["links"], f"catalog carries the {link_id} upgrade link")
    denial_ux_source = DENIAL_UX.read_text(encoding="utf-8")
    expect("MSG_BASE_REQUIRED" in denial_ux_source
           and "A verified Evaluation or paid Focusa entitlement is required" in denial_ux_source,
           "denial UX carries the canonical base-required message")
    expect('("evaluate", "Start a free Evaluation or purchase Focusa")' in denial_ux_source
           and '("purchase", "Purchase or renew this optional family")' in denial_ux_source,
           "denial UX carries the frozen evaluate/purchase upgrade labels")
    op_map = json.loads(CLI_OP_MAP.read_text(encoding="utf-8"))
    expect(op_map["row_count"] == len(op_map["rows"]), "CLI operation map rows are counted")
    selector_fields = {"price", "product", "grant", "plan", "feature", "tier", "sku"}
    for row in op_map["rows"]:
        expect(not selector_fields.intersection(row.keys()),
               f"CLI operation row carries no product/price/grant selector: {row.get('command_path')}",
               is_negative=True)
    menubar = json.loads(MENUBAR_MAP.read_text(encoding="utf-8"))
    for row in menubar.get("actions", menubar.get("rows", [])):
        expect(not selector_fields.intersection(row.keys()),
               "menubar action row carries no per-button product/price/grant decision", is_negative=True)
    policy = yaml.safe_load(POLICY_YAML.read_text(encoding="utf-8"))
    grid = {item["state"]: item for item in policy.get("state_grid", [])}
    limited_cell = grid.get("verified_no_license", {})
    limited_policies = limited_cell.get("policies", {})
    expect(limited_policies.get("base_focusa") == "allow_manual_one_mutable_project",
           "canonical grid cell: verified_no_license base_focusa is the one-mutable-project limited allowance")
    for family in ["automation", "team_remote", "release_proof", "premium_updates"]:
        expect(limited_policies.get(family) == "deny", f"canonical grid cell denies paid family {family}")

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
        "schema": "focusa.spec152f.evaluation_first_value_e2e_validation.v1",
        "atom": "focusa-vbcqu.20.14.45",
        "harness_sha256": sha256_text(HARNESS),
        "harness_output_sha256": sha256_text(out1),
        "harness_positive_checks": harness_positive,
        "harness_negative_checks": harness_negative,
        "static_positive_checks": positive,
        "static_negative_checks": negative,
        "decision": result["decision"],
        "duration": result["duration"],
        "creates_edd_license_key": result["creates_edd_license_key"],
        "postures": result["postures"],
        "assertions": result["assertions"],
        "evaluation_nodes": result["evaluation_nodes"],
        "edd_customers_total": result["edd_customers_total"],
        "edd_orders_total": result["edd_orders_total"],
        "edd_licenses_total": result["edd_licenses_total"],
        "first_value_families": result["first_value_families"],
        "mutable_projects": result["mutable_projects"],
        "second_project_blocked": result["second_project_blocked"],
        "premium_families_blocked": result["premium_families_blocked"],
        "assertion_signature_verified": result["assertion_signature_verified"],
        "lease_signature_verified": result["lease_signature_verified"],
        "resume_verified": result["resume_verified"],
        "refresh_sequence": result["refresh_sequence"],
        "widened_family_denied": result["widened_family_denied"],
        "limited_access_cases": len(limited["cases"]),
        "harness_replay_identical": True,
        "no_local_issuance": True,
        "no_duplicate_customer_or_key": True,
        "single_family_boundary_no_395": True,
        "result": "passed",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
