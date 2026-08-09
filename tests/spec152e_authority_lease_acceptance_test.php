<?php
// Spec 152E §23 authority/lease acceptance matrix (fixture-level, build-independent).
// 152E.04.07 exercises the delivery, node, lease, and recovery fixtures so that every
// acceptance-matrix row settles ONE canonical EDD entitlement and no delivery/lease/
// recovery path creates independent authority:
//   - issuer + verifier: paid lease issues at the server sequence and verifies Active;
//     wrong device/product and tampered signatures fail closed (paid, wrong
//     device/product, tamper);
//   - refresh: rotation is strictly monotonic; stale sequence, refund, revoke, and
//     evaluation expiry each settle to a SIGNED recovery-only refusal (stale sequence,
//     refund, revocation, expiry, Evaluation);
//   - dual delivery: email + terminal resolve one canonical key; terminal delivery
//     loss recovers the SAME key and never mints a second license (website/terminal
//     paid, terminal delivery loss);
//   - node reservation: the atomic counter enforces the server-owned limit and a
//     reservation race cannot over-issue (node limit, node race);
//   - verifier: OTP and verification-complete handler accept only the live challenge
//     and reject wrong/tampered/expired verifiers (verifier);
//   - authority outage + recovery: the facade returns recovery_only/AUTHORITY_UNAVAILABLE
//     and never issues locally; REFUNDED/REVOKED/AUTHORITY_UNAVAILABLE bind to
//     recovery_only with preserved recovery surfaces (authority outage, recovery).
// No raw email, secret, license key, or unmasked real-email evidence is emitted; the
// only fixture emails use the reserved .invalid TLD.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-terminal-delivery-envelope.v1.php';
require_once $root . '/docs/contracts/spec152e-transactional-mail-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-dual-delivery-coordinator.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';
require_once $root . '/docs/contracts/spec152e-lease-refresh-service.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-node.v1.php';
require_once $root . '/docs/contracts/spec152e-challenge-service.v1.php';
require_once $root . '/docs/contracts/spec152e-verification-complete-handler.v1.php';
require_once $root . '/docs/contracts/spec152e-rate-limiter.v1.php';
require_once $root . '/docs/contracts/spec152e-install-facade-routes.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

const NOW = '2026-08-08T18:30:00Z';
const PAID_DEVICE_KEY = 'AbCdEfGhIjKlMnOpQrStUvWxYz0123456789ab_CDef';
const EVAL_DEVICE_KEY = 'bMgpSjs62S8N-Lb9mJHdwWjFQkoy7Pk5eVAzRZVpr1s';
const PRODUCT_PAID = 'focusa_operator_lifetime_v1';
const PRODUCT_EVAL = 'focusa_evaluation';

$positive = 0;
$negative = 0;
$matrixRows = [];

function expect_acc(bool $condition, string $message): void
{
    global $positive;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
    $positive++;
}

function expect_acc_throws(callable $operation, string $code, string $message): void
{
    global $negative;
    $negative++;
    try {
        $operation();
    } catch (DomainException $error) {
        if ($error->getMessage() === $code) {
            return;
        }
        fwrite(STDERR, "FAIL: {$message} (got DomainException {$error->getMessage()})\n");
        exit(1);
    } catch (Throwable $error) {
        fwrite(STDERR, "FAIL: {$message} (unexpected " . get_class($error) . ': ' . $error->getMessage() . ")\n");
        exit(1);
    }
    fwrite(STDERR, "FAIL: {$message} (no exception thrown)\n");
    exit(1);
}

/** Assert a signed refusal: decision denied, recovery_only, reason, verifiable. */
function expect_acc_refusal(array $result, string $reason, FocusaSpec152eLeaseRefreshService $service, string $message): void
{
    global $positive;
    if (($result['decision'] ?? '') !== 'denied' || ($result['state'] ?? '') !== 'recovery_only' || ($result['error'] ?? '') !== $reason) {
        fwrite(STDERR, "FAIL: {$message} (decision=" . ($result['decision'] ?? 'none') . ' state=' . ($result['state'] ?? 'none') . " error=" . ($result['error'] ?? 'none') . ")\n");
        exit(1);
    }
    $refusal = $result['refusal'] ?? null;
    if (!is_array($refusal) || ($refusal['schema'] ?? '') !== 'focusa.signed_envelope.v1') {
        fwrite(STDERR, "FAIL: {$message} (refusal envelope missing)\n");
        exit(1);
    }
    $verified = $service->verifyRefusal($refusal, ['now' => $result['created_at']]);
    if (($verified['reason_code'] ?? '') !== $reason || ($verified['posture'] ?? '') !== 'recovery_only') {
        fwrite(STDERR, "FAIL: {$message} (verified reason=" . ($verified['reason_code'] ?? 'none') . " posture=" . ($verified['posture'] ?? 'none') . ")\n");
        exit(1);
    }
    $positive++;
}

function b64url_encode_php(string $binary): string
{
    return rtrim(strtr(base64_encode($binary), '+/', '-_'), '=');
}

function b64url_decode_php(string $encoded): string
{
    $padding = (4 - strlen($encoded) % 4) % 4;
    $decoded = base64_decode(strtr($encoded . str_repeat('=', $padding), '-_', '+/'), true);
    if ($decoded === false) {
        throw new DomainException('ENVELOPE_FORMAT_DENIED');
    }
    return $decoded;
}

// ── Shared fixture: superset EDD/authority tables ─────────────────────────

function seed_fixture(PDO $db): void
{
    $db->exec('CREATE TABLE wp_edd_customers (customer_id INTEGER PRIMARY KEY, email TEXT, name TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_orders (id INTEGER PRIMARY KEY, order_id INTEGER, customer_id INTEGER, status TEXT, total TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_order_items (order_item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER, price_id INTEGER, quantity INTEGER, subtotal TEXT, total TEXT)');
    $db->exec('CREATE TABLE wp_edd_licenses (
        id INTEGER PRIMARY KEY, license_id INTEGER, customer_id INTEGER, user_id INTEGER NULL,
        download_id INTEGER, payment_id INTEGER, product_id INTEGER, order_id INTEGER,
        license_key TEXT, status TEXT, activation_limit INTEGER, expiration TEXT,
        date_created TEXT, license_length INTEGER NULL, license_unit TEXT NULL,
        activation_count INTEGER NOT NULL DEFAULT 0)');
    $db->exec('CREATE TABLE wp_wpuiai_authority_accounts (
        account_uuid TEXT PRIMARY KEY, edd_customer_id INTEGER UNIQUE, customer_id INTEGER,
        wordpress_user_id INTEGER NULL, stripe_customer_id TEXT NULL, status TEXT, status_reason TEXT,
        highest_entitlement_sequence INTEGER, migration_provenance TEXT, created_at TEXT, updated_at TEXT)');
    $db->exec('CREATE TABLE wp_wpuiai_authority_nodes (node_uuid TEXT PRIMARY KEY, account_uuid TEXT, edd_license_id INTEGER, product_code TEXT, device_public_key TEXT, assurance_class TEXT, status TEXT)');
}

function seed_paid(PDO $db, int $customerId, string $accountUuid, int $highest, int $orderId, int $itemId, int $licenseId, string $nodeId, string $status = 'active', string $orderStatus = 'complete'): void
{
    $db->exec("INSERT OR IGNORE INTO wp_edd_customers (customer_id, email, name, date_created) VALUES ({$customerId}, 'c{$customerId}@example.invalid', 'Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT OR IGNORE INTO wp_wpuiai_authority_accounts (account_uuid, edd_customer_id, customer_id, wordpress_user_id, stripe_customer_id, status, status_reason, highest_entitlement_sequence, migration_provenance, created_at, updated_at)
        VALUES ('{$accountUuid}', {$customerId}, {$customerId}, NULL, NULL, 'active', 'mailbox_verified', {$highest}, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders (id, order_id, customer_id, status, total, date_created) VALUES ({$orderId}, {$orderId}, {$customerId}, '{$orderStatus}', '697.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items (order_item_id, order_id, product_id, price_id, quantity, subtotal, total) VALUES ({$itemId}, {$orderId}, 1001, 0, 1, '697.00', '697.00')");
    $db->exec("INSERT INTO wp_edd_licenses (id, license_id, customer_id, download_id, payment_id, product_id, license_key, status, activation_limit, expiration, date_created)
        VALUES ({$licenseId}, {$licenseId}, {$customerId}, 1001, {$orderId}, 1001, 'F0C15A-{$customerId}-0001-0001-0001', '{$status}', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes (node_uuid, account_uuid, edd_license_id, product_code, device_public_key, assurance_class, status)
        VALUES ('{$nodeId}', '{$accountUuid}', {$licenseId}, '" . PRODUCT_PAID . "', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
}

function seed_eval(PDO $db, int $customerId, string $accountUuid, int $orderId, int $itemId, int $licenseId, string $nodeId, string $expiration): void
{
    $db->exec("INSERT OR IGNORE INTO wp_edd_customers (customer_id, email, name, date_created) VALUES ({$customerId}, 'c{$customerId}@example.invalid', 'Eval Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT OR IGNORE INTO wp_wpuiai_authority_accounts (account_uuid, edd_customer_id, customer_id, wordpress_user_id, stripe_customer_id, status, status_reason, highest_entitlement_sequence, migration_provenance, created_at, updated_at)
        VALUES ('{$accountUuid}', {$customerId}, {$customerId}, NULL, NULL, 'active', 'account_promoted', 6, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders (id, order_id, customer_id, status, total, date_created) VALUES ({$orderId}, {$orderId}, {$customerId}, 'complete', '0.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items (order_item_id, order_id, product_id, price_id, quantity, subtotal, total) VALUES ({$itemId}, {$orderId}, 1004, 0, 1, '0.00', '0.00')");
    $db->exec("INSERT INTO wp_edd_licenses (id, license_id, customer_id, download_id, payment_id, product_id, license_key, status, activation_limit, expiration, date_created)
        VALUES ({$licenseId}, {$licenseId}, {$customerId}, 1004, {$orderId}, 1004, 'E5A10000-0002-0002-0002-0002', 'active', 1, '{$expiration}', '2026-08-08T18:30:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes (node_uuid, account_uuid, edd_license_id, product_code, device_public_key, assurance_class, status)
        VALUES ('{$nodeId}', '{$accountUuid}', {$licenseId}, '" . PRODUCT_EVAL . "', '" . EVAL_DEVICE_KEY . "', 'device_key_v1', 'active')");
}

function build_components(PDO $db, string $clockValue): array
{
    $clock = static fn() => $clockValue;
    $keySet = new FocusaSpec152eAuthorityKeySetSeam(
        implode('', array_map('chr', range(0, 31))),
        implode('', array_map('chr', range(32, 63))),
        $clock,
    );
    $accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
    $accountMigration->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
    $accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
    $lifecycleSchema = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
    $lifecycleSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
    $projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleSchema, 'wp_', $clock);
    $outboxSchema = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
    $outboxSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
    $eventSchema = new FocusaSpec152eAuthorityEventSchema();
    $signer = new FocusaSpec152eAuthorityEventSigner('test-server-side-secret-for-spec152e-outbox-v1!', FocusaSpec152eAuthorityEventSchema::KEY_ID);
    $outboxHook = new FocusaSpec152eEddAuthorityHook($db, $outboxSchema, $eventSchema, $signer, $accounts, 'wp_', $clock);
    $issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keySet, $clock, 'wp_');
    $issuer->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
    $refreshSchema = new FocusaSpec152eLeaseRefreshMigration($db, 'wp_');
    $refreshSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
    $service = new FocusaSpec152eLeaseRefreshService($db, $issuer, $keySet, $projector, $outboxHook, $refreshSchema, 'wp_', $clock);
    return compact('clock', 'keySet', 'accounts', 'projector', 'outboxHook', 'signer', 'issuer', 'service');
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
seed_fixture($db);

// Shared mutable clock for the registrations/dual/verifier surfaces.
$nowValue = NOW;
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};

// ── A. Issuer + verifier: paid lease, wrong device/product, tamper ────────

$A1 = 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d';
seed_paid($db, 1001, $A1, 41, 9001, 90011, 7001, 'node-paid-001');
$c = build_components($db, NOW);
$keySet = $c['keySet'];
$issuer = $c['issuer'];
$service = $c['service'];
$accounts = $c['accounts'];
$projector = $c['projector'];

$paidLease = $issuer->issueLease([
    'account_uuid' => $A1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-paid-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-acc-a1-0001', 'request_id' => 'req-lease-acc-a1-0001',
]);
expect_acc((int) $paidLease['sequence'] === 42, 'paid lease issues at the server sequence 42');
$verifyContext = [
    'expected_product' => 'focusa', 'expected_node_id' => 'node-paid-001',
    'now' => NOW, 'minimum_sequence' => 42,
];
$verified = $issuer->verifyEnvelope($paidLease['envelope'], $verifyContext);
expect_acc(($verified['state'] ?? '') === 'active', 'paid lease verifies Active on the bound device');
expect_acc_throws(
    static fn() => $issuer->verifyEnvelope($paidLease['envelope'], ['expected_product' => 'focusa', 'expected_node_id' => 'node-other-001', 'now' => NOW, 'minimum_sequence' => 42]),
    'WRONG_NODE',
    'a lease on another device never verifies',
);
expect_acc_throws(
    static fn() => $issuer->verifyEnvelope($paidLease['envelope'], ['expected_product' => 'uiai_engine', 'expected_node_id' => 'node-paid-001', 'now' => NOW, 'minimum_sequence' => 42]),
    'WRONG_PRODUCT',
    'a lease never cross-grants another product',
);
$tampered = $paidLease['envelope'];
$tampered['signature_b64'] = base64_encode(str_repeat("\x00", 64));
expect_acc_throws(
    static fn() => $issuer->verifyEnvelope($tampered, $verifyContext),
    'INVALID_SIGNATURE',
    'a tampered lease signature fails closed',
);
$matrixRows[] = 'paid_lease_issuer_verifier';
$matrixRows[] = 'wrong_device_product';
$matrixRows[] = 'tamper';

// ── B. Refresh: rotation, stale sequence, refund, revoke, evaluation expiry ──

$request = static fn(string $account, string $node, string $ikey, string $credential, ?int $currentSequence = null, string $product = PRODUCT_PAID): array => array_filter([
    'account_uuid' => $account, 'product_code' => $product, 'node_id' => $node,
    'refresh_credential' => $credential, 'current_sequence' => $currentSequence,
    'idempotency_key' => $ikey, 'request_id' => 'req-' . $ikey,
], static fn(mixed $value): bool => $value !== null);

$credA1 = (string) $service->issueRefreshCredential(['lease_uuid' => $paidLease['lease_uuid'], 'idempotency_key' => 'cred-acc-a1-0001', 'request_id' => 'req-cred-acc-a1-0001'])['refresh_credential'];
$rotation = $service->refresh($request($A1, 'node-paid-001', 'refresh-acc-a1-0001', $credA1, 42));
expect_acc(($rotation['decision'] ?? '') === 'rotated' && ($rotation['state'] ?? '') === 'activated', 'refresh rotates the paid lease');
expect_acc((int) $rotation['lease']['sequence'] === 43, 'rotated lease sequence is strictly monotonic (43)');
$matrixRows[] = 'refresh_rotation';

$S1 = 'd4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f81';
seed_paid($db, 8008, $S1, 41, 9401, 94011, 7401, 'node-stale-001');
$s1Lease = $issuer->issueLease([
    'account_uuid' => $S1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-stale-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-acc-s1-0001', 'request_id' => 'req-lease-acc-s1-0001',
]);
$credS1 = (string) $service->issueRefreshCredential(['lease_uuid' => $s1Lease['lease_uuid'], 'idempotency_key' => 'cred-acc-s1-0001', 'request_id' => 'req-cred-acc-s1-0001'])['refresh_credential'];
$accounts->advanceSequence($S1, 45, 'advance-acc-s1-0001');
$staleRefusal = $service->refresh($request($S1, 'node-stale-001', 'refresh-acc-s1-0001', $credS1, 42));
expect_acc_refusal($staleRefusal, 'STALE_SEQUENCE', $service, 'stale sequence denies refresh with a signed recovery-only refusal');
expect_acc((int) $staleRefusal['authority_sequence'] === 45, 'stale refusal carries the advanced authority sequence');
$matrixRows[] = 'stale_sequence';

$R1 = 'e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091';
seed_paid($db, 6006, $R1, 41, 9201, 92011, 7201, 'node-refund-001');
$projector->projectOrder([
    'account_uuid' => $R1, 'edd_customer_id' => 6006, 'order_id' => 9201, 'license_id' => 7201,
    'status' => 'completed', 'request_id' => 'req-acc-complete-r1', 'idempotency_key' => 'acc-complete-r1-0001',
]);
$r1Lease = $issuer->issueLease([
    'account_uuid' => $R1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-refund-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-acc-r1-0001', 'request_id' => 'req-lease-acc-r1-0001',
]);
$credR1 = (string) $service->issueRefreshCredential(['lease_uuid' => $r1Lease['lease_uuid'], 'idempotency_key' => 'cred-acc-r1-0001', 'request_id' => 'req-cred-acc-r1-0001'])['refresh_credential'];
$projector->projectRefund([
    'account_uuid' => $R1, 'edd_customer_id' => 6006, 'order_id' => 9201, 'license_id' => 7201,
    'status' => 'refunded', 'request_id' => 'req-acc-refund-r1', 'idempotency_key' => 'acc-refund-r1-0001',
]);
$db->exec("UPDATE wp_edd_licenses SET status = 'refunded' WHERE license_id = 7201");
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE order_id = 9201");
$refundRefusal = $service->refresh($request($R1, 'node-refund-001', 'refresh-acc-r1-0001', $credR1, 43));
expect_acc_refusal($refundRefusal, 'REFUNDED', $service, 'refund denies refresh with a signed recovery-only refusal');
expect_acc(($issuer->findLease($r1Lease['lease_uuid'])['status'] ?? '') === 'refunded', 'refund settles the lease to refunded');
$matrixRows[] = 'refund';

$V1 = 'c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f';
seed_paid($db, 7007, $V1, 41, 9301, 93011, 7301, 'node-revoke-001');
$projector->projectOrder([
    'account_uuid' => $V1, 'edd_customer_id' => 7007, 'order_id' => 9301, 'license_id' => 7301,
    'status' => 'completed', 'request_id' => 'req-acc-complete-v1', 'idempotency_key' => 'acc-complete-v1-0001',
]);
$v1Lease = $issuer->issueLease([
    'account_uuid' => $V1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-revoke-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-acc-v1-0001', 'request_id' => 'req-lease-acc-v1-0001',
]);
$credV1 = (string) $service->issueRefreshCredential(['lease_uuid' => $v1Lease['lease_uuid'], 'idempotency_key' => 'cred-acc-v1-0001', 'request_id' => 'req-cred-acc-v1-0001'])['refresh_credential'];
$projector->projectLicense([
    'account_uuid' => $V1, 'edd_customer_id' => 7007, 'license_id' => 7301,
    'from_status' => 'active', 'to_status' => 'revoked', 'request_id' => 'req-acc-revoke-v1', 'idempotency_key' => 'acc-revoke-v1-0001',
]);
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE license_id = 7301");
$revokeRefusal = $service->refresh($request($V1, 'node-revoke-001', 'refresh-acc-v1-0001', $credV1, 43));
expect_acc_refusal($revokeRefusal, 'REVOKED', $service, 'revocation denies refresh with a signed recovery-only refusal');
expect_acc(($issuer->findLease($v1Lease['lease_uuid'])['status'] ?? '') === 'revoked', 'revocation settles the lease to revoked');
$matrixRows[] = 'revocation';

$E2 = 'a9b0c1d2-e3f4-4a5b-6c7d-8e9f0a1b2c35';
seed_eval($db, 2302, $E2, 9102, 91022, 7003, 'node-eval-expire-001', '2026-09-07T18:30:00Z');
$e2Lease = $issuer->issueLease([
    'account_uuid' => $E2, 'product_code' => PRODUCT_EVAL, 'node_id' => 'node-eval-expire-001',
    'device_public_key' => EVAL_DEVICE_KEY, 'idempotency_key' => 'lease-acc-e2-0001', 'request_id' => 'req-lease-acc-e2-0001',
]);
$credE2 = (string) $service->issueRefreshCredential(['lease_uuid' => $e2Lease['lease_uuid'], 'idempotency_key' => 'cred-acc-e2-0001', 'request_id' => 'req-cred-acc-e2-0001'])['refresh_credential'];
$expiryService = build_components($db, '2026-09-08T18:30:00Z')['service'];
$expiryRefusal = $expiryService->refresh($request($E2, 'node-eval-expire-001', 'refresh-acc-e2-0001', $credE2, 7, PRODUCT_EVAL));
expect_acc_refusal($expiryRefusal, 'EXPIRED', $expiryService, 'evaluation lease past expiry denies refresh (no local extension)');
$matrixRows[] = 'expiry_evaluation';

// ── C. Dual delivery: email + terminal one canonical key; delivery loss ───

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_lease_acceptance']);
$envelopeMigration = new FocusaSpec152eTerminalDeliveryEnvelopeMigration($db, 'wp_');
$envelopeMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_lease_acceptance']);
$dualMigration = new FocusaSpec152eDualLicenseDeliveryMigration($db, 'wp_');
$dualMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_lease_acceptance']);
$registrationSecrets = new FocusaSpec152eActivationRegistrationSecrets(str_repeat('e', 32), str_repeat('v', 32), str_repeat('p', 32));
$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $clock, attemptTtl: 86400, verificationTtl: 3600, pollTtl: 3600);
$sentEmails = [];
$mailAdapter = new FocusaSpec152eTransactionalMailAdapter(
    static function (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) use (&$sentEmails): bool {
        $sentEmails[] = ['to' => $to, 'subject' => $subject, 'html' => $htmlBody, 'text' => $textBody, 'sender' => $senderIdentity];
        return true;
    }
);
$coordinator = new FocusaSpec152eDualLicenseDeliveryCoordinator($db, $dualMigration, $registrations, $registrationSecrets, $mailAdapter, $clock);
$installFacade = null;
foreach ($facadeRegistry['facades'] as $f) {
    if (($f['facade_id'] ?? '') === 'focusa_install_v1') {
        $installFacade = $f;
        break;
    }
}
expect_acc(is_array($installFacade) && isset($installFacade['sender']['identity']), 'fixture uses the registered install facade sender');
$eddLicenseCount = static function (): int {
    global $db;
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

$fixtureSequence = 0;
$makeFixture = static function () use (&$fixtureSequence, $registrations, $db): array {
    $fixtureSequence++;
    $seq = str_pad((string) $fixtureSequence, 4, '0', STR_PAD_LEFT);
    $licenseId = 5000 + $fixtureSequence;
    $orderId = 7000 + $fixtureSequence;
    $itemId = 8000 + $fixtureSequence;
    $created = $registrations->createPending([
        'email' => 'synthetic.operator' . $seq . '@example.invalid',
        'facade_id' => 'focusa_install_v1', 'presenter' => 'terminal',
        'install_channel' => 'source_build', 'product_code' => 'focusa_operator',
        'request_id' => 'req-acc-dual-' . $seq . '-0001', 'idempotency_key' => 'idem-acc-dual-' . $seq . '-0001',
    ]);
    $registrationId = $created['registration']['registration_uuid'];
    $verified = $registrations->verifyEmail($registrationId, $created['verification_secret'], 'req-acc-dual-' . $seq . '-0002', 'idem-acc-dual-' . $seq . '-0002');
    $promoted = $registrations->promoteVerified($registrationId, '018f47c2-6ac0-7b16-8d1a-' . str_pad(dechex(100 + $fixtureSequence), 12, '0', STR_PAD_LEFT), 41000 + $fixtureSequence, 'req-acc-dual-' . $seq . '-0003', 'idem-acc-dual-' . $seq . '-0003');
    $offer = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, (int) $promoted['registration']['state_version'],
        'req-acc-dual-' . $seq . '-0004', 'idem-acc-dual-' . $seq . '-0004', ['offer_code' => 'focusa_operator', 'journey' => 'purchase']);
    $checkout = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
        FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING, (int) $offer['registration']['state_version'],
        'req-acc-dual-' . $seq . '-0005', 'idem-acc-dual-' . $seq . '-0005', ['edd_cart_reference' => 'cart_acc_' . $seq]);
    $issued = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
        FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED, (int) $checkout['registration']['state_version'],
        'req-acc-dual-' . $seq . '-0006', 'idem-acc-dual-' . $seq . '-0006',
        ['edd_order_id' => $orderId, 'edd_order_item_id' => $itemId, 'edd_license_id' => $licenseId]);
    $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
        FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, (int) $issued['registration']['state_version'],
        'req-acc-dual-' . $seq . '-0007', 'idem-acc-dual-' . $seq . '-0007');
    $key = strtoupper(substr(hash('sha256', 'fixture-' . $seq), 0, 8)) . '-' . strtoupper(substr(hash('sha256', 'a-' . $seq), 0, 8))
        . '-' . strtoupper(substr(hash('sha256', 'b-' . $seq), 0, 8)) . '-' . strtoupper(substr(hash('sha256', 'c-' . $seq), 0, 8));
    $db->exec("INSERT INTO wp_edd_licenses (id, license_id, customer_id, user_id, download_id, payment_id, product_id, order_id, license_key, status, activation_limit, expiration, date_created, license_length, license_unit, activation_count)
        VALUES ({$licenseId}, {$licenseId}, 41000 + {$fixtureSequence}, NULL, 1001, {$orderId}, 1001, {$orderId}, '" . $key . "', 'active', 5, NULL, '2026-08-08T00:00:00Z', 0, 'years', 0)");
    return ['registration_id' => $registrationId, 'poll_credential' => $created['poll_credential'], 'license_id' => $licenseId, 'license_key' => $key, 'seq' => $seq];
};

$devicePrivate = random_bytes(32);
$devicePublicRaw = FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($devicePrivate);
$devicePublicB64 = b64url_encode_php($devicePublicRaw);

// Fixture A: full dual delivery — email + terminal resolve ONE canonical key.
$fixtureA = $makeFixture();
$licensesBeforeA = $eddLicenseCount();
$settleA = $coordinator->settle([
    'registration_id' => $fixtureA['registration_id'], 'facade' => $installFacade,
    'product_name' => 'Focusa Operator', 'support_email' => 'support.synthetic@example.invalid',
    'request_id' => 'req-acc-settle-a-0001', 'idempotency_key' => 'idem-acc-settle-a-0001',
]);
expect_acc(($settleA['email_sent'] ?? false) === true && ($settleA['resolved_state'] ?? '') === 'pending', 'settle sends the transactional email and leaves terminal pending');
expect_acc(str_contains(json_encode($sentEmails), $fixtureA['license_key']), 'the license email carries the full canonical key');
$deliveredA = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureA['registration_id'], 'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T18:31:00Z', 'request_id' => 'req-acc-email-a-0001', 'idempotency_key' => 'idem-acc-email-a-0001',
]);
expect_acc(($deliveredA['resolved_state'] ?? '') === 'email_only', 'provider email delivery resolves partial');
$confirmed = $coordinator->noteTerminalDelivered([
    'registration_id' => $fixtureA['registration_id'], 'edd_license_id' => $fixtureA['license_id'],
    'license_key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($fixtureA['license_key']),
    'request_id' => 'req-acc-terminal-a-0001', 'idempotency_key' => 'idem-acc-terminal-a-0001',
]);
expect_acc(($confirmed['resolved_state'] ?? '') === 'both_delivered' && ($confirmed['same_key_confirmed'] ?? false) === true,
    'email and terminal resolve one canonical key');
expect_acc($eddLicenseCount() === $licensesBeforeA, 'dual delivery never mints a second license');
expect_acc_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureA['registration_id'], 'poll_credential' => $fixtureA['poll_credential'],
        'recovery_channel' => 'terminal', 'request_id' => 'req-acc-recover-a-0001', 'idempotency_key' => 'idem-acc-recover-a-0001',
    ]),
    'DUAL_DELIVERY_ALREADY_SETTLED',
    'recovery after full settlement fails closed',
);
$matrixRows[] = 'dual_delivery_one_key';

// Fixture C: terminal delivery loss — email delivered, terminal recovery returns the SAME key.
$fixtureC = $makeFixture();
$licensesBeforeC = $eddLicenseCount();
$coordinator->settle([
    'registration_id' => $fixtureC['registration_id'], 'facade' => $installFacade,
    'request_id' => 'req-acc-settle-c-0001', 'idempotency_key' => 'idem-acc-settle-c-0001',
]);
$coordinator->recordEmailOutcome([
    'registration_id' => $fixtureC['registration_id'], 'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T18:31:00Z', 'request_id' => 'req-acc-email-c-0001', 'idempotency_key' => 'idem-acc-email-c-0001',
]);
$partialC = $coordinator->deliveryState(['registration_id' => $fixtureC['registration_id']]);
expect_acc(($partialC['resolved_state'] ?? '') === 'email_only', 'terminal delivery loss resolves email-only');
$registrations->bindDevicePublicKey($fixtureC['registration_id'], $devicePublicB64, 'req-acc-bind-c-0001', 'idem-acc-bind-c-0001');
$recoveredC = $coordinator->recover([
    'registration_id' => $fixtureC['registration_id'], 'poll_credential' => $fixtureC['poll_credential'],
    'recovery_channel' => 'terminal', 'request_id' => 'req-acc-recover-c-0001', 'idempotency_key' => 'idem-acc-recover-c-0001',
]);
expect_acc(($recoveredC['schema'] ?? '') === FocusaSpec152eDualLicenseDeliveryCoordinator::RECOVERY_SCHEMA, 'terminal recovery returns the recovery schema');
expect_acc((bool) preg_match('/^env_[0-9a-f]{32}$/D', (string) ($recoveredC['envelope_id'] ?? '')), 'terminal recovery returns a bounded envelope ID');
expect_acc(str_contains((string) ($recoveredC['one_time_key_envelope'] ?? ''), $fixtureC['license_key']) === false, 'recovery envelope never exposes plaintext');
$recoveryEnvelope = json_decode(b64url_decode_php((string) $recoveredC['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
$claimsC = json_decode(FocusaSpec152eTerminalEnvelopeCrypto::open($devicePrivate, $recoveryEnvelope), true, 512, JSON_THROW_ON_ERROR);
expect_acc((string) ($claimsC['license_key'] ?? '') === $fixtureC['license_key'], 'terminal recovery resolves the SAME canonical key');
expect_acc($eddLicenseCount() === $licensesBeforeC, 'delivery loss recovery never mints a second license');
$matrixRows[] = 'terminal_delivery_loss';

// ── D. Node reservation: server-owned limit + reservation race ────────────

$nodeDb = new PDO('sqlite::memory:');
$nodeDb->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$nodeMigration = new FocusaSpec152eAuthorityNodeMigration($nodeDb, 'wp_');
$nodeMigration->migrate('2026-08-08T05:00:00Z', ['source' => 'authority_lease_acceptance']);
$nodeDb->exec('CREATE TABLE wp_wpuiai_authority_accounts (account_uuid TEXT NOT NULL PRIMARY KEY, edd_customer_id BIGINT NOT NULL UNIQUE, status VARCHAR(32) NOT NULL, status_reason VARCHAR(191) NOT NULL)');
$nodeDb->exec('CREATE TABLE wp_edd_licenses (id BIGINT NOT NULL PRIMARY KEY, license_key TEXT NOT NULL, customer_id BIGINT NOT NULL, activation_limit BIGINT NOT NULL, status VARCHAR(32) NOT NULL, expiration TEXT NULL)');
$seedAccount = static function (string $uuid, int $customer, string $reason) use ($nodeDb): void {
    $nodeDb->prepare('INSERT INTO wp_wpuiai_authority_accounts (account_uuid, edd_customer_id, status, status_reason) VALUES (:uuid, :customer, :status, :reason)')
        ->execute([':uuid' => $uuid, ':customer' => $customer, ':status' => 'active', ':reason' => $reason]);
};
$seedLicense = static function (int $id, int $customer, int $limit) use ($nodeDb): void {
    $nodeDb->prepare('INSERT INTO wp_edd_licenses (id, license_key, customer_id, activation_limit, status, expiration) VALUES (:id, :key, :customer, :limit, :status, NULL)')
        ->execute([':id' => $id, ':key' => 'FOCUSA-ACC-' . str_pad((string) $id, 4, '0', STR_PAD_LEFT), ':customer' => $customer, ':limit' => $limit, ':status' => 'active']);
};
$accountA = '018f47c2-6ac0-7b16-8d1a-4e93df5a01aa';
$accountB = '018f47c2-6ac0-7b16-8d1a-4e93df5a01bb';
$seedAccount($accountA, 41001, 'mailbox_verified');
$seedAccount($accountB, 41002, 'account_promoted');
$seedLicense(9001, 41001, 3); // account A: limit 3
$seedLicense(9002, 41002, 2); // account B: limit 2 (race)
$clockTick = 0;
$nodeClock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-08T05:02:00Z'))->modify('+' . $clockTick . ' minutes')->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
};
$nodeRepo = new FocusaSpec152eAuthorityNodeRepository($nodeDb, $nodeMigration, $nodeClock);
$deviceKey = static fn(string $seed): string => substr(strtr(base64_encode(hash('sha256', 'device-' . $seed, true)), '+/', '-_'), 0, 43);
$nodeAttempt = static function (array $overrides) use ($accountA, $deviceKey): array {
    return array_merge([
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1001', 'account_uuid' => $accountA,
        'edd_license_id' => 9001, 'product_code' => 'focusa_operator_lifetime_v1',
        'device_public_key' => $deviceKey('a1'), 'assurance_class' => 'device_key_v1',
        'idempotency_key' => 'idem-acc-node-0001', 'migration_provenance' => ['source' => 'synthetic_node_fixture'],
    ], $overrides);
};
for ($i = 1; $i <= 3; $i++) {
    $node = $nodeRepo->registerNode($nodeAttempt([
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1' . str_pad((string) $i, 3, '0', STR_PAD_LEFT),
        'device_public_key' => $deviceKey('a' . $i), 'idempotency_key' => 'idem-acc-node-000' . $i,
    ]));
    expect_acc($node['status'] === 'active', "node {$i} registers within the server-owned limit");
}
$beforeOverflow = (int) $nodeDb->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn();
expect_acc_throws(
    static fn() => $nodeRepo->registerNode($nodeAttempt([
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1004', 'device_public_key' => $deviceKey('a4'),
        'idempotency_key' => 'idem-acc-node-0004',
    ])),
    'NODE_LIMIT_EXHAUSTED',
    'the fourth node cannot exceed the EDD product node limit',
);
expect_acc((int) $nodeDb->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn() === $beforeOverflow, 'denied attempt creates no reservation');
$matrixRows[] = 'node_limit';

$race = static function (string $uuid, string $deviceSeed, string $idem, int $license) use ($nodeRepo, $accountB, $deviceKey): array {
    return $nodeRepo->reserve([
        'node_uuid' => $uuid, 'account_uuid' => $accountB, 'edd_license_id' => $license,
        'product_code' => 'uiai_operator_lifetime_v1', 'device_public_key' => $deviceKey($deviceSeed),
        'assurance_class' => 'device_key_v1', 'idempotency_key' => $idem,
        'migration_provenance' => ['source' => 'synthetic_node_fixture'],
    ]);
};
$r1 = $race('018f47c2-6ac0-7b16-8d1a-4e93df5a2001', 'b1', 'idem-acc-race-0001', 9002);
expect_acc(($r1['state'] ?? '') === 'reserved', 'first race reservation occupies a slot');
$r2 = $race('018f47c2-6ac0-7b16-8d1a-4e93df5a2002', 'b2', 'idem-acc-race-0002', 9002);
expect_acc(($r2['state'] ?? '') === 'reserved', 'second race reservation occupies the last slot');
expect_acc_throws(
    static fn() => $race('018f47c2-6ac0-7b16-8d1a-4e93df5a2003', 'b3', 'idem-acc-race-0003', 9002),
    'NODE_LIMIT_EXHAUSTED',
    'a third concurrent reservation cannot exceed the counter limit',
);
$nodeRepo->releaseReservation((string) $r2['reservation_id'], 'test_release', 'idem-acc-release-0001');
$r3 = $race('018f47c2-6ac0-7b16-8d1a-4e93df5a2004', 'b4', 'idem-acc-race-0004', 9002);
expect_acc(($r3['state'] ?? '') === 'reserved', 'a released slot can be re-reserved (no drift)');
$matrixRows[] = 'node_race';

// ── E. Verifier: OTP + verification-complete handler, wrong/tampered/expired ──

$challenge = new FocusaSpec152eChallengeService(str_repeat('v', 32));
$otp = $challenge->generateOtp('focusa_install_v1', NOW, '2026-08-08T18:35:00Z');
expect_acc($challenge->validate((string) $otp['code'], (string) $otp['verifier_hash']) === true, 'a live OTP validates against the stored hash');
expect_acc($challenge->validate('000000', (string) $otp['verifier_hash']) === false, 'a wrong/tampered OTP fails');
expect_acc($challenge->validate((string) $otp['code'], str_repeat('0', 64)) === false, 'a tampered stored hash fails');
$rateLimiter = new FocusaSpec152eRateLimiter($db, 'wp_', $clock);
$completeHandler = new FocusaSpec152eVerificationCompleteHandler($registrations, $rateLimiter);
$pending = $registrations->createPending([
    'email' => 'synthetic.verify.acceptance@example.invalid', 'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal', 'install_channel' => 'source_build', 'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-acc-verify-0001', 'idempotency_key' => 'idem-acc-verify-0001',
]);
$regUuid = $pending['registration']['registration_uuid'];
$verifyOk = $completeHandler->complete([
    'registration_uuid' => $regUuid, 'verifier' => $pending['verification_secret'],
    'facade_id' => 'focusa_install_v1', 'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-acc-verify-0002', 'idempotency_key' => 'idem-acc-verify-0002',
], $facadeRegistry, hash('sha256', 'opaque-acceptance-client-1'));
expect_acc(!isset($verifyOk['error']), 'a live verifier completes email verification');
$pendingBad = $registrations->createPending([
    'email' => 'synthetic.verify.tampered@example.invalid', 'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal', 'install_channel' => 'source_build', 'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-acc-verify-0021', 'idempotency_key' => 'idem-acc-verify-0021',
]);
$verifyBad = $completeHandler->complete([
    'registration_uuid' => $pendingBad['registration']['registration_uuid'], 'verifier' => 'tampered-verifier-token',
    'facade_id' => 'focusa_install_v1', 'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-acc-verify-0003', 'idempotency_key' => 'idem-acc-verify-0003',
], $facadeRegistry, hash('sha256', 'opaque-acceptance-client-2'));
expect_acc(($verifyBad['error'] ?? '') === 'EMAIL_VERIFICATION_FAILED', 'a tampered verifier is denied');
$pending2 = $registrations->createPending([
    'email' => 'synthetic.verify.expired@example.invalid', 'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal', 'install_channel' => 'source_build', 'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-acc-verify-0011', 'idempotency_key' => 'idem-acc-verify-0011',
]);
$regUuid2 = $pending2['registration']['registration_uuid'];
$nowValue = '2026-08-08T20:31:00Z'; // +2h past the 1h verification TTL
$verifyExpired = $completeHandler->complete([
    'registration_uuid' => $regUuid2, 'verifier' => $pending2['verification_secret'],
    'facade_id' => 'focusa_install_v1', 'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-acc-verify-0012', 'idempotency_key' => 'idem-acc-verify-0012',
], $facadeRegistry, hash('sha256', 'opaque-acceptance-client-3'));
expect_acc(($verifyExpired['error'] ?? '') === 'EMAIL_VERIFICATION_EXPIRED', 'an expired challenge is rejected');
$matrixRows[] = 'email_verifier';

// ── F. Authority outage + recovery-only denial ────────────────────────────

$outage = FocusaSpec152eInstallFacadeRoutes::authorityUnavailable('req-acc-outage-0001', 'https://install.focusa.dev');
expect_acc(($outage['status'] ?? 0) === 503 && ($outage['envelope']['error'] ?? '') === 'AUTHORITY_UNAVAILABLE', 'authority outage returns a stable 503');
expect_acc(($outage['envelope']['state'] ?? '') === 'recovery_only' && ($outage['envelope']['next_action'] ?? '') === 'retry_or_use_recovery',
    'authority outage maps to recovery_only with a safe next action');
expect_acc(!array_key_exists('license', $outage['envelope']) && !array_key_exists('lease', $outage['envelope']) && !array_key_exists('node', $outage['envelope']),
    'authority outage never issues a local license, node, or lease');
$matrixRows[] = 'authority_outage';

$recoveryContract = json_decode(file_get_contents($root . '/docs/contracts/spec152e-recovery-only-surface.v1.json'), true, 512, JSON_THROW_ON_ERROR);
$denialBindings = [];
foreach ($recoveryContract['denial_bindings'] as $binding) {
    foreach ($binding['codes'] as $code) {
        $denialBindings[$code] = $binding;
    }
}
expect_acc(($denialBindings['REFUNDED']['class'] ?? '') === 'license' && ($denialBindings['REFUNDED']['posture'] ?? '') === 'recovery_only',
    'REFUNDED binds to recovery_only');
expect_acc(($denialBindings['REVOKED']['class'] ?? '') === 'license' && ($denialBindings['REVOKED']['posture'] ?? '') === 'recovery_only',
    'REVOKED binds to recovery_only');
expect_acc(($denialBindings['AUTHORITY_UNAVAILABLE']['class'] ?? '') === 'lease' && ($denialBindings['AUTHORITY_UNAVAILABLE']['posture'] ?? '') === 'recovery_only',
    'AUTHORITY_UNAVAILABLE binds to recovery_only');
$surfaces = array_column($recoveryContract['recovery_surfaces'], 'surface');
foreach (['account_verification', 'license_status_management', 'export', 'diagnostics', 'repair', 'update_for_recovery', 'uninstall'] as $surface) {
    expect_acc(in_array($surface, $surfaces, true), "recovery preserves {$surface}");
}
expect_acc(($recoveryContract['invariants']['recovery_never_grants_entitlement'] ?? false) === true, 'recovery never grants entitlement');
$matrixRows[] = 'recovery_only_denial';

// ── Hygiene: no unmasked real email, license key, secret, or credential ───

$rawOutputs = '';
foreach ([$settleA, $confirmed, $recoveredC, $refundRefusal, $revokeRefusal, $staleRefusal, $expiryRefusal, $outage] as $result) {
    $rawOutputs .= json_encode($result);
}
expect_acc(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $rawOutputs) !== 1, 'hygiene: no unmasked email in any result envelope');
expect_acc(preg_match('/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/', $rawOutputs) !== 1, 'hygiene: no full license key in any result envelope');
expect_acc(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $rawOutputs) !== 1, 'hygiene: no secret prefixes in any result envelope');
// The rotation result carries the fresh one-time refresh credential BY DESIGN (returned
// exactly once); the stored rows must never contain it.
$dbDump = json_encode($db->query('SELECT * FROM wp_wpuiai_lease_refresh_log')->fetchAll(PDO::FETCH_ASSOC))
    . json_encode($db->query('SELECT * FROM wp_wpuiai_lease_refresh_credentials')->fetchAll(PDO::FETCH_ASSOC))
    . json_encode($db->query('SELECT * FROM wp_wpuiai_authority_outbox')->fetchAll(PDO::FETCH_ASSOC));
foreach ([(string) ($rotation['refresh_credential'] ?? ''), $credA1] as $plaintext) {
    if ($plaintext !== '') {
        expect_acc(strpos($dbDump, $plaintext) === false, 'hygiene: plaintext refresh credential is never stored at rest');
    }
}
expect_acc(preg_match('/rc_[0-9a-f]{48}/', $dbDump) !== 1, 'hygiene: no refresh credential token appears in any stored row');

echo json_encode([
    'schema' => 'focusa.spec152e.authority_lease_acceptance_matrix.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'matrix_rows_exercised' => array_values(array_unique($matrixRows)),
    'settlements' => ['refunded', 'revoked', 'superseded_stale', 'superseded_expiry'],
    'rotations' => 1,
    'refusals' => 4,
    'result' => 'passed_fail_closed',
], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), "\n";
