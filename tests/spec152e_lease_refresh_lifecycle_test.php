<?php
// Exact verification for focusa-vbcqu.20.13.36: refresh, sequence, refund, revoke, and
// expiry settlement (spec 152E §7.5, §10 POST /v1/lease/refresh, §14.2 refresh credential,
// §17 lifecycle sequence increments, §18 refund/revoke/expiry -> sequence increment ->
// refresh denied/recovery-only, §19.9 idempotent refresh, §20 stable failure semantics,
// §23 acceptance matrix "Refund", "Revocation", "Authority outage" rows).
//
// The refresh service re-reads canonical EDD/account/node state, enforces the monotonic
// sequence and signed offline bounds, rotates the signed lease and the bounded refresh
// credential safely (hash-at-rest, plaintext exactly once), and denies refund/revoke/
// expiry/node removal with a signed recovery-only refusal so stale software/state cannot
// restore access. Every settlement is idempotent, outbox-journaled, and preservation-only.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';
require_once $root . '/docs/contracts/spec152e-lease-refresh-service.v1.php';

const NOW = '2026-08-08T18:30:00Z';
const PAID_DEVICE_KEY = 'AbCdEfGhIjKlMnOpQrStUvWxYz0123456789ab_CDef';
const EVAL_DEVICE_KEY = 'bMgpSjs62S8N-Lb9mJHdwWjFQkoy7Pk5eVAzRZVpr1s';
const PRODUCT_PAID = 'focusa_operator_lifetime_v1';
const PRODUCT_EVAL = 'focusa_evaluation';

$positive = 0;
$negative = 0;

function expect_refresh(bool $condition, string $message): void
{
    global $positive;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
    $positive++;
}

function expect_refresh_throws(callable $operation, string $code, string $message): void
{
    global $negative;
    $negative++;
    try {
        $operation();
    } catch (Throwable $error) {
        if ($error->getMessage() === $code) {
            return;
        }
        fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
        exit(1);
    }
    fwrite(STDERR, "FAIL: {$message} (no exception thrown)\n");
    exit(1);
}

function expect_refresh_invalid(callable $operation, string $message): void
{
    global $negative;
    $negative++;
    try {
        $operation();
    } catch (InvalidArgumentException $error) {
        return;
    } catch (Throwable $error) {
        fwrite(STDERR, "FAIL: {$message} (unexpected " . get_class($error) . ": " . $error->getMessage() . ")\n");
        exit(1);
    }
    fwrite(STDERR, "FAIL: {$message} (no exception thrown)\n");
    exit(1);
}

/** Assert a signed refusal result: decision denied, recovery-only, signed and verifiable. */
function expect_refusal(array $result, string $reason, FocusaSpec152eLeaseRefreshService $service, string $message): void
{
    global $positive;
    $decisionValue = $result['decision'] ?? 'none';
    $stateValue = $result['state'] ?? 'none';
    $errorValue = $result['error'] ?? 'none';
    if ($decisionValue !== 'denied') {
        fwrite(STDERR, "FAIL: {$message} (decision={$decisionValue})\n");
        exit(1);
    }
    if ($stateValue !== 'recovery_only') {
        fwrite(STDERR, "FAIL: {$message} (state={$stateValue})\n");
        exit(1);
    }
    if ($errorValue !== $reason) {
        fwrite(STDERR, "FAIL: {$message} (error={$errorValue})\n");
        exit(1);
    }
    $refusal = $result['refusal'] ?? null;
    if (!is_array($refusal) || ($refusal['schema'] ?? '') !== 'focusa.signed_envelope.v1') {
        fwrite(STDERR, "FAIL: {$message} (refusal envelope missing)\n");
        exit(1);
    }
    $verified = $service->verifyRefusal($refusal, ['now' => $result['created_at']]);
    if (($verified['reason_code'] ?? '') !== $reason) {
        fwrite(STDERR, "FAIL: {$message} (verified reason=" . ($verified['reason_code'] ?? 'none') . ")\n");
        exit(1);
    }
    if (($verified['posture'] ?? '') !== 'recovery_only') {
        fwrite(STDERR, "FAIL: {$message} (posture=" . ($verified['posture'] ?? 'none') . ")\n");
        exit(1);
    }
    if ((int) ($verified['authority_sequence'] ?? 0) < (int) ($verified['presented_sequence'] ?? 0)) {
        fwrite(STDERR, "FAIL: {$message} (refusal is internally stale)\n");
        exit(1);
    }
    $positive++;
}

// ── Fixture ────────────────────────────────────────────────────────────

function seed_fixture(PDO $db): void
{
    $db->exec('CREATE TABLE wp_edd_customers (customer_id INTEGER PRIMARY KEY, email TEXT, name TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_orders (id INTEGER PRIMARY KEY, order_id INTEGER, customer_id INTEGER, status TEXT, total TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_order_items (order_item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER, price_id INTEGER, quantity INTEGER, subtotal TEXT, total TEXT)');
    $db->exec('CREATE TABLE wp_edd_licenses (id INTEGER PRIMARY KEY, license_id INTEGER, customer_id INTEGER, download_id INTEGER, payment_id INTEGER, license_key TEXT, status TEXT, activation_limit INTEGER, expiration TEXT, date_created TEXT)');
    // Superset authority-account view: the EDD-bound issuer reads customer_id; the account
    // repository / outbox / projector read edd_customer_id (same EDD customer).
    $db->exec('CREATE TABLE wp_wpuiai_authority_accounts (
        account_uuid TEXT PRIMARY KEY, edd_customer_id INTEGER UNIQUE, customer_id INTEGER,
        wordpress_user_id INTEGER NULL, stripe_customer_id TEXT NULL, status TEXT, status_reason TEXT,
        highest_entitlement_sequence INTEGER, migration_provenance TEXT, created_at TEXT, updated_at TEXT)');
    $db->exec('CREATE TABLE wp_wpuiai_authority_nodes (node_uuid TEXT PRIMARY KEY, account_uuid TEXT, edd_license_id INTEGER, product_code TEXT, device_public_key TEXT, assurance_class TEXT, status TEXT)');
}

/** Seed one paid account: customer + order/item + license + account + node (idempotent). */
function seed_paid(PDO $db, int $customerId, string $accountUuid, int $highest, int $orderId, int $itemId, int $licenseId, string $nodeId, string $status = 'active', string $orderStatus = 'complete'): void
{
    $db->exec("INSERT OR IGNORE INTO wp_edd_customers VALUES ({$customerId}, 'c{$customerId}@example.invalid', 'Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT OR IGNORE INTO wp_wpuiai_authority_accounts VALUES ('{$accountUuid}', {$customerId}, {$customerId}, NULL, NULL, 'active', 'mailbox_verified', {$highest}, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES ({$orderId}, {$orderId}, {$customerId}, '{$orderStatus}', '697.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES ({$itemId}, {$orderId}, 1001, 0, 1, '697.00', '697.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES ({$licenseId}, {$licenseId}, {$customerId}, 1001, {$orderId}, 'F0C15A-{$customerId}-0001-0001-0001', '{$status}', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('{$nodeId}', '{$accountUuid}', {$licenseId}, '" . PRODUCT_PAID . "', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
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
    $accountMigration->migrate('2026-08-08T05:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
    $accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
    $lifecycleSchema = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
    $lifecycleSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
    $projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleSchema, 'wp_', $clock);
    $outboxSchema = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
    $outboxSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
    $eventSchema = new FocusaSpec152eAuthorityEventSchema();
    $signer = new FocusaSpec152eAuthorityEventSigner('test-server-side-secret-for-spec152e-outbox-v1!', FocusaSpec152eAuthorityEventSchema::KEY_ID);
    $outboxHook = new FocusaSpec152eEddAuthorityHook($db, $outboxSchema, $eventSchema, $signer, $accounts, 'wp_', $clock);
    $issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keySet, $clock, 'wp_');
    $issuer->migrate('2026-08-08T05:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
    $refreshSchema = new FocusaSpec152eLeaseRefreshMigration($db, 'wp_');
    $refreshSchema->migrate('2026-08-08T05:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
    $service = new FocusaSpec152eLeaseRefreshService($db, $issuer, $keySet, $projector, $outboxHook, $refreshSchema, 'wp_', $clock);
    return compact('clock', 'keySet', 'accounts', 'projector', 'outboxHook', 'signer', 'issuer', 'service');
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
seed_fixture($db);

// ── Fixture accounts (paid unless noted) ──
$A1 = 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d';   // rotation / replay / conflict / highest-sequence
$F1 = 'f1a2b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c';   // offline-grace rotation + NOT_YET_VALID + past-grace expiry
$R1 = 'e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091';   // refund settlement
$V1 = 'c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f';   // revoke settlement
$S1 = 'd4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f81';   // advanceSequence staleness
$M1 = 'b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6f';   // current_sequence mismatch
$C1 = 'c5d6e7f8-a9b0-4c1d-2e3f-4a5b6c7d8e91';   // credential invalid
$N1 = 'd6e7f8a9-b0c1-4d2e-3f4a-5b6c7d8e9f02';   // node removal + node deactivation
$B1 = 'e7f8a9b0-c1d2-4e3f-4a5b-6c7d8e9f0a13';   // EDD-truth refusals (4 leases)
$X1 = 'f8a9b0c1-d2e3-4f4a-5b6c-7d8e9f0a1b24';   // no leases -> LEASE_NOT_FOUND
$E1 = 'b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e';   // eval, no credential -> REFRESH_CREDENTIAL_REQUIRED
$E2 = 'a9b0c1d2-e3f4-4a5b-6c7d-8e9f0a1b2c35';   // eval expiry settlement
$P1 = 'd4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f80';   // pending/unverified -> EMAIL_VERIFICATION_REQUIRED

// Paid accounts
seed_paid($db, 1001, $A1, 41, 9001, 90011, 7001, 'node-paid-golden-001');
seed_paid($db, 5005, $F1, 10, 9101, 91011, 7101, 'node-grace-001');
seed_paid($db, 6006, $R1, 41, 9201, 92011, 7201, 'node-refund-001');
seed_paid($db, 7007, $V1, 41, 9301, 93011, 7301, 'node-revoke-001');
seed_paid($db, 8008, $S1, 41, 9401, 94011, 7401, 'node-stale-001');
seed_paid($db, 9009, $M1, 41, 9501, 95011, 7501, 'node-seq-001');
seed_paid($db, 1101, $C1, 41, 9601, 96011, 7601, 'node-cred-001');
seed_paid($db, 1201, $N1, 41, 9701, 97011, 7701, 'node-removed-001');
seed_paid($db, 1201, $N1, 41, 9702, 97022, 7702, 'node-deactivated-001');
// B1: four leases for the EDD-truth refusal matrix
seed_paid($db, 2001, $B1, 41, 9801, 98011, 8101, 'node-lic-001');
seed_paid($db, 2001, $B1, 41, 9802, 98022, 8102, 'node-cust-001');
seed_paid($db, 2001, $B1, 41, 9803, 98033, 8103, 'node-pending-001');
seed_paid($db, 2001, $B1, 41, 9804, 98044, 8104, 'node-download-001');
seed_paid($db, 1501, $X1, 0, 9901, 99011, 7901, 'node-none-001');
// Eval accounts
$db->exec("INSERT INTO wp_edd_customers VALUES (2002, 'c2002@example.invalid', 'Eval Fixture', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_edd_orders VALUES (9002, 9002, 2002, 'complete', '0.00', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_edd_order_items VALUES (90022, 9002, 1004, 0, 1, '0.00', '0.00')");
$db->exec("INSERT INTO wp_edd_licenses VALUES (7002, 7002, 2002, 1004, 9002, 'E5A10000-0002-0002-0002-0002', 'active', 1, '2026-09-07T18:30:00Z', '2026-08-08T18:30:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('{$E1}', 2002, 2002, NULL, NULL, 'active', 'account_promoted', 6, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-eval-golden-001', '{$E1}', 7002, '" . PRODUCT_EVAL . "', '" . EVAL_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_edd_customers VALUES (2302, 'c2302@example.invalid', 'Eval Expire', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_edd_orders VALUES (9102, 9102, 2302, 'complete', '0.00', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_edd_order_items VALUES (91022, 9102, 1004, 0, 1, '0.00', '0.00')");
$db->exec("INSERT INTO wp_edd_licenses VALUES (7003, 7003, 2302, 1004, 9102, 'E5A10000-0003-0003-0003-0003', 'active', 1, '2026-09-07T18:30:00Z', '2026-08-08T18:30:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('{$E2}', 2302, 2302, NULL, NULL, 'active', 'account_promoted', 6, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-eval-expire-001', '{$E2}', 7003, '" . PRODUCT_EVAL . "', '" . EVAL_DEVICE_KEY . "', 'device_key_v1', 'active')");
// Unverified account (pending) for hard-failure tests
$db->exec("INSERT INTO wp_edd_customers VALUES (4004, 'c4004@example.invalid', 'Pending Fixture', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('{$P1}', 4004, 4004, NULL, NULL, 'pending', 'email_challenge_sent', 0, '{}', '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-pending-000', '{$P1}', 7999, '" . PRODUCT_PAID . "', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");

$c = build_components($db, NOW);
$keySet = $c['keySet'];
$accounts = $c['accounts'];
$issuer = $c['issuer'];
$service = $c['service'];
$projector = $c['projector'];
$signer = $c['signer'];

// ── Migration idempotency + rollback preservation ──
$service->migrate('2026-08-08T05:01:00Z', ['source' => 'repeat_must_preserve_first_schema_application']);
$migrationRows = $db->query('SELECT * FROM wp_wpuiai_lease_refresh_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_refresh(count($migrationRows) === 1, 'repeated refresh migration records one schema version');
expect_refresh($migrationRows[0]['applied_at'] === '2026-08-08T05:00:00Z', 'repeated refresh migration preserves first applied timestamp');
$preserved = $service->preserveForRollback('2026-08-08T06:00:00Z', ['source' => 'lease_refresh_lifecycle_test']);
expect_refresh(($preserved['action'] ?? '') === 'preserve' && ($preserved['event_key'] ?? '') !== '', 'refresh rollback preservation journaled');

// ── Issue the A1 paid lease + refresh credential ──
$request = static fn(string $account, string $node, string $ikey, string $credential, ?int $currentSequence = null, string $product = PRODUCT_PAID): array => array_filter([
    'account_uuid' => $account,
    'product_code' => $product,
    'node_id' => $node,
    'refresh_credential' => $credential,
    'current_sequence' => $currentSequence,
    'idempotency_key' => $ikey,
    'request_id' => 'req-' . $ikey,
], static fn(mixed $value): bool => $value !== null);

$paidLease = $issuer->issueLease([
    'account_uuid' => $A1,
    'product_code' => PRODUCT_PAID,
    'node_id' => 'node-paid-golden-001',
    'device_public_key' => PAID_DEVICE_KEY,
    'idempotency_key' => 'lease-refresh-a1-0001',
    'request_id' => 'req-lease-refresh-a1-0001',
]);
expect_refresh($paidLease['sequence'] === 42, 'A1 lease issued at server sequence 42');

$credentialSeed = $service->issueRefreshCredential(['lease_uuid' => $paidLease['lease_uuid'], 'idempotency_key' => 'cred-a1-0001', 'request_id' => 'req-cred-a1-0001']);
$credA1 = (string) $credentialSeed['refresh_credential'];
expect_refresh(str_starts_with($credA1, 'rc_') && strlen($credA1) === 51, 'refresh credential is an opaque bounded token');
expect_refresh(hash('sha256', $credA1) === (string) $credentialSeed['credential_digest'], 'credential digest matches the returned plaintext');
$credRowA1 = $service->credentialRow($paidLease['lease_uuid']);
expect_refresh($credRowA1 !== null && ($credRowA1['status'] ?? '') === 'current', 'credential stored current for the lease');
expect_refresh((string) $credRowA1['credential_digest'] === hash('sha256', $credA1), 'only the digest is stored at rest');
$replaySeed = $service->issueRefreshCredential(['lease_uuid' => $paidLease['lease_uuid'], 'idempotency_key' => 'cred-a1-0001', 'request_id' => 'req-cred-a1-0001']);
expect_refresh(($replaySeed['replayed'] ?? false) === true, 'credential issuance is idempotent under the same key');
expect_refresh_throws(
    static fn() => $service->issueRefreshCredential(['lease_uuid' => $paidLease['lease_uuid'], 'idempotency_key' => 'cred-a1-0002', 'request_id' => 'req-cred-a1-0002']),
    'REFRESH_CREDENTIAL_ALREADY_ISSUED',
    'a current credential is never silently re-issued',
);

// ── Positive: rotation ──
$rotation = $service->refresh($request($A1, 'node-paid-golden-001', 'refresh-a1-0001', $credA1, 42));
expect_refresh(($rotation['decision'] ?? '') === 'rotated' && ($rotation['state'] ?? '') === 'activated', 'refresh rotates to activated');
expect_refresh((int) $rotation['presented_sequence'] === 42, 'rotation presents sequence 42');
$rotatedLease = $rotation['lease'];
expect_refresh((int) $rotatedLease['sequence'] === 43, 'rotated lease sequence is strictly monotonic (43)');
expect_refresh((string) $rotatedLease['claims']['previous_lease_digest'] === 'sha256:' . hash('sha256', FocusaSpec152eAuthorityKeySetSeam::decodePayload($paidLease['envelope']['payload_b64'])), 'rotated lease chains the exact prior payload digest');
expect_refresh((string) $rotation['previous_lease_uuid'] === (string) $paidLease['lease_uuid'], 'rotation records the superseded lease uuid');
$oldRow = $issuer->findLease($paidLease['lease_uuid']);
expect_refresh(($oldRow['status'] ?? '') === 'superseded' && ($oldRow['status_reason'] ?? '') === 'refresh_rotated', 'presented lease settled to superseded/refresh_rotated');
$rotCred = (string) $rotation['refresh_credential'];
expect_refresh(str_starts_with($rotCred, 'rc_'), 'rotation returns a fresh one-time refresh credential');
$credRowNew = $service->credentialRow($rotatedLease['lease_uuid']);
expect_refresh($credRowNew !== null && (string) $credRowNew['credential_digest'] === hash('sha256', $rotCred), 'rotated credential digest stored bound to the new lease');
expect_refresh(($service->credentialRow($paidLease['lease_uuid'])['status'] ?? '') === 'superseded', 'old credential superseded');
$verifyState = $issuer->verifyEnvelope($rotatedLease['envelope'], [
    'expected_product' => 'focusa',
    'expected_node_id' => 'node-paid-golden-001',
    'now' => NOW,
    'minimum_sequence' => 43,
]);
expect_refresh(($verifyState['state'] ?? '') === 'active', 'rotated envelope verifies as Active at the new sequence');
$highest = $service->highestSequence($A1, PRODUCT_PAID);
expect_refresh((int) $highest['highest_sequence'] === 43, 'highest sequence surface reports 43 after rotation');
$outboxRows = $db->query("SELECT event_type FROM wp_wpuiai_authority_outbox ORDER BY created_at")->fetchAll(PDO::FETCH_ASSOC);
$eventTypes = array_column($outboxRows, 'event_type');
expect_refresh(in_array('lease_superseded', $eventTypes, true) && in_array('lease_issued', $eventTypes, true), 'rotation journals lease_superseded + lease_issued outbox events');
expect_refresh($service->refreshCount() === 1, 'rotation records one refresh log row');
$logRow = $service->findByRefreshUuid((string) $db->query("SELECT refresh_uuid FROM wp_wpuiai_lease_refresh_log LIMIT 1")->fetchColumn());
expect_refresh(($logRow['decision'] ?? '') === 'rotated' && ($logRow['posture'] ?? '') === 'activated', 'refresh log records rotated/activated');
expect_refresh(strpos((string) $logRow['result_payload'], $rotCred) === false, 'refresh log never stores the plaintext credential');
// Outbox envelope signature verifies (HMAC seam regression).
$outboxEnvelope = $db->query("SELECT payload, envelope_digest, signature, signing_key_id FROM wp_wpuiai_authority_outbox WHERE event_type = 'lease_issued' LIMIT 1")->fetch(PDO::FETCH_ASSOC);
$signer->verify((string) $outboxEnvelope['payload'], (string) $outboxEnvelope['envelope_digest'], (string) $outboxEnvelope['signature'], (string) $outboxEnvelope['signing_key_id']);
expect_refresh(true, 'outbox lease_issued envelope verifies with the authority event signer');

// ── Positive: idempotent replay of the rotation ──
$replayRefresh = $service->refresh($request($A1, 'node-paid-golden-001', 'refresh-a1-0001', $credA1, 42));
expect_refresh(($replayRefresh['replayed'] ?? false) === true && ($replayRefresh['decision'] ?? '') === 'rotated', 'refresh replay returns the same rotation decision');
expect_refresh((string) $replayRefresh['lease']['lease_uuid'] === (string) $rotatedLease['lease_uuid'], 'refresh replay returns the byte-identical rotated lease');
expect_refresh(str_starts_with((string) $replayRefresh['refresh_credential'], 'rc_'), 'refresh replay re-seeds a one-time credential for the rotated lease');
expect_refresh($service->refreshCount() === 1, 'refresh replay does not add a log row');
expect_refresh_throws(
    static fn() => $service->refresh($request($A1, 'node-other-001', 'refresh-a1-0001', $credA1, 42)),
    'IDEMPOTENCY_CONFLICT',
    'changed reuse of a refresh idempotency key fails closed',
);

// ── Positive: offline-grace rotation re-extends (bounds) ──
$f1Lease = $issuer->issueLease([
    'account_uuid' => $F1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-grace-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-f1-0001', 'request_id' => 'req-lease-refresh-f1-0001',
]);
expect_refresh((int) $f1Lease['sequence'] === 11, 'F1 lease issued at sequence 11');
$credF1 = (string) $service->issueRefreshCredential(['lease_uuid' => $f1Lease['lease_uuid'], 'idempotency_key' => 'cred-f1-0001', 'request_id' => 'req-cred-f1-0001'])['refresh_credential'];
// NOT_YET_VALID: a refresh clock before the lease not_before is refused without settlement.
$preService = build_components($db, '2026-08-01T00:00:00Z')['service'];
$notYetValid = $preService->refresh($request($F1, 'node-grace-001', 'refresh-f1-prenb-0001', $credF1, 11));
expect_refusal($notYetValid, 'NOT_YET_VALID', $preService, 'refresh before not_before is denied');
expect_refresh(($issuer->findLease($f1Lease['lease_uuid'])['status'] ?? '') === 'active', 'NOT_YET_VALID does not settle the lease');
// Grace window rotation: within expiry+offline grace the refresh re-extends.
$grace = build_components($db, '2026-12-01T00:00:00Z');
$graceRotation = $grace['service']->refresh($request($F1, 'node-grace-001', 'refresh-f1-grace-0001', $credF1, 11));
expect_refresh(($graceRotation['decision'] ?? '') === 'rotated' && (int) $graceRotation['lease']['sequence'] === 12, 'refresh inside offline grace rotates to sequence 12');
expect_refresh((string) $graceRotation['lease']['claims']['expires_at'] === '2027-03-01T00:00:00Z', 'grace rotation re-extends expiry +90d from the refresh time');
$f1NewCred = (string) $graceRotation['refresh_credential'];
$f1NewLeaseUuid = (string) $graceRotation['lease']['lease_uuid'];

// ── Settlement: refund → signed refusal, lease refunded, outbox journaled, stable re-denial ──
$completeDecision = $projector->projectOrder([
    'account_uuid' => $R1, 'edd_customer_id' => 6006, 'order_id' => 9201, 'license_id' => 7201,
    'status' => 'completed', 'request_id' => 'req-project-complete-r1', 'idempotency_key' => 'project-complete-r1-0001',
]);
expect_refresh(($completeDecision['decision'] ?? '') === 'applied' && (int) $completeDecision['result_sequence'] === 42, 'order completion projects the account to sequence 42');
$r1Lease = $issuer->issueLease([
    'account_uuid' => $R1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-refund-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-r1-0001', 'request_id' => 'req-lease-refresh-r1-0001',
]);
expect_refresh((int) $r1Lease['sequence'] === 43, 'R1 lease issued at sequence 43 after completion');
$credR1 = (string) $service->issueRefreshCredential(['lease_uuid' => $r1Lease['lease_uuid'], 'idempotency_key' => 'cred-r1-0001', 'request_id' => 'req-cred-r1-0001'])['refresh_credential'];
$refundDecision = $projector->projectRefund([
    'account_uuid' => $R1, 'edd_customer_id' => 6006, 'order_id' => 9201, 'license_id' => 7201,
    'status' => 'refunded', 'request_id' => 'req-project-refund-r1', 'idempotency_key' => 'project-refund-r1-0001',
]);
expect_refresh(($refundDecision['decision'] ?? '') === 'applied' && ($refundDecision['license_state'] ?? '') === 'refunded' && (int) $refundDecision['result_sequence'] === 43, 'refund projects the license to refunded at sequence 43');
$db->exec("UPDATE wp_edd_licenses SET status = 'refunded' WHERE license_id = 7201");
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE order_id = 9201");
$refundRefusal = $service->refresh($request($R1, 'node-refund-001', 'refresh-r1-refund-0001', $credR1, 43));
expect_refusal($refundRefusal, 'REFUNDED', $service, 'refunded license denies refresh with a signed recovery-only refusal');
expect_refresh((int) $refundRefusal['authority_sequence'] === 43, 'refund refusal carries the post-refund authority sequence');
$r1Row = $issuer->findLease($r1Lease['lease_uuid']);
expect_refresh(($r1Row['status'] ?? '') === 'refunded' && ($r1Row['status_reason'] ?? '') === 'edd_refunded', 'refund settles the lease to refunded/edd_refunded');
$refundEvents = array_column($db->query("SELECT event_type FROM wp_wpuiai_authority_outbox")->fetchAll(PDO::FETCH_ASSOC), 'event_type');
expect_refresh(in_array('lease_superseded', $refundEvents, true), 'refund journals a lease_superseded outbox event');
$stableDenial = $service->refresh($request($R1, 'node-refund-001', 'refresh-r1-refund-0002', $credR1, 43));
expect_refusal($stableDenial, 'REFUNDED', $service, 'refund denial is stable across refresh attempts (no local extension)');

// ── Settlement: revoke → signed refusal, lease revoked, lease_revoked outbox ──
$completeV1 = $projector->projectOrder([
    'account_uuid' => $V1, 'edd_customer_id' => 7007, 'order_id' => 9301, 'license_id' => 7301,
    'status' => 'completed', 'request_id' => 'req-project-complete-v1', 'idempotency_key' => 'project-complete-v1-0001',
]);
expect_refresh(($completeV1['decision'] ?? '') === 'applied', 'V1 order completion projected');
$v1Lease = $issuer->issueLease([
    'account_uuid' => $V1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-revoke-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-v1-0001', 'request_id' => 'req-lease-refresh-v1-0001',
]);
expect_refresh((int) $v1Lease['sequence'] === 43, 'V1 lease issued at sequence 43');
$credV1 = (string) $service->issueRefreshCredential(['lease_uuid' => $v1Lease['lease_uuid'], 'idempotency_key' => 'cred-v1-0001', 'request_id' => 'req-cred-v1-0001'])['refresh_credential'];
$revokeDecision = $projector->projectLicense([
    'account_uuid' => $V1, 'edd_customer_id' => 7007, 'license_id' => 7301,
    'from_status' => 'active', 'to_status' => 'revoked', 'request_id' => 'req-project-revoke-v1', 'idempotency_key' => 'project-revoke-v1-0001',
]);
expect_refresh(($revokeDecision['decision'] ?? '') === 'applied' && ($revokeDecision['license_state'] ?? '') === 'revoked', 'revoke projects the license to revoked');
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE license_id = 7301");
$revokeRefusal = $service->refresh($request($V1, 'node-revoke-001', 'refresh-v1-revoke-0001', $credV1, 43));
expect_refusal($revokeRefusal, 'REVOKED', $service, 'revoked license denies refresh with a signed refusal');
expect_refresh(($issuer->findLease($v1Lease['lease_uuid'])['status'] ?? '') === 'revoked', 'revoke settles the lease to revoked');
$revokeEvents = array_column($db->query("SELECT event_type FROM wp_wpuiai_authority_outbox")->fetchAll(PDO::FETCH_ASSOC), 'event_type');
expect_refresh(in_array('lease_revoked', $revokeEvents, true), 'revoke journals a lease_revoked outbox event');

// ── Settlement: stale sequence → signed refusal + verifier rejects the old lease ──
$s1Lease = $issuer->issueLease([
    'account_uuid' => $S1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-stale-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-s1-0001', 'request_id' => 'req-lease-refresh-s1-0001',
]);
$credS1 = (string) $service->issueRefreshCredential(['lease_uuid' => $s1Lease['lease_uuid'], 'idempotency_key' => 'cred-s1-0001', 'request_id' => 'req-cred-s1-0001'])['refresh_credential'];
$accounts->advanceSequence($S1, 45, 'advance-stale-s1-0001');
$staleRefusal = $service->refresh($request($S1, 'node-stale-001', 'refresh-s1-stale-0001', $credS1, 42));
expect_refusal($staleRefusal, 'STALE_SEQUENCE', $service, 'lease below the account highest sequence is denied');
expect_refresh((int) $staleRefusal['authority_sequence'] === 45, 'stale refusal carries the advanced authority sequence 45');
expect_refresh(($issuer->findLease($s1Lease['lease_uuid'])['status'] ?? '') === 'superseded', 'stale lease settled to superseded/stale_sequence');
expect_refresh_throws(
    static fn() => $issuer->verifyEnvelope($s1Lease['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => 'node-stale-001',
        'now' => NOW, 'minimum_sequence' => (int) $staleRefusal['authority_sequence'],
    ]),
    'STALE_SEQUENCE',
    'stale software/state cannot restore access: the verifier rejects the old lease at the refusal authority sequence',
);

// ── Settlement: current_sequence mismatch → STALE_SEQUENCE (authoritative, not stale) ──
$m1Lease = $issuer->issueLease([
    'account_uuid' => $M1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-seq-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-m1-0001', 'request_id' => 'req-lease-refresh-m1-0001',
]);
$credM1 = (string) $service->issueRefreshCredential(['lease_uuid' => $m1Lease['lease_uuid'], 'idempotency_key' => 'cred-m1-0001', 'request_id' => 'req-cred-m1-0001'])['refresh_credential'];
$seqRefusal = $service->refresh($request($M1, 'node-seq-001', 'refresh-m1-seq-0001', $credM1, 7));
expect_refusal($seqRefusal, 'STALE_SEQUENCE', $service, 'client-presented current_sequence mismatch denies refresh');
expect_refresh((int) $seqRefusal['authority_sequence'] >= (int) $seqRefusal['presented_sequence'], 'sequence refusal is authoritative (never internally stale)');

// ── Settlement: credential invalid → refusal without lease mutation ──
$c1Lease = $issuer->issueLease([
    'account_uuid' => $C1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-cred-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-c1-0001', 'request_id' => 'req-lease-refresh-c1-0001',
]);
$service->issueRefreshCredential(['lease_uuid' => $c1Lease['lease_uuid'], 'idempotency_key' => 'cred-c1-0001', 'request_id' => 'req-cred-c1-0001']);
$wrongCred = 'rc_' . str_repeat('ab', 24);
$credRefusal = $service->refresh($request($C1, 'node-cred-001', 'refresh-c1-cred-0001', $wrongCred, 42));
expect_refusal($credRefusal, 'REFRESH_CREDENTIAL_INVALID', $service, 'wrong refresh credential is denied');
expect_refresh(($issuer->findLease($c1Lease['lease_uuid'])['status'] ?? '') === 'active', 'credential-invalid denial never mutates the lease row');

// ── Settlement: node removal and node deactivation ──
$n1LeaseRemoved = $issuer->issueLease([
    'account_uuid' => $N1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-removed-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-n1-0001', 'request_id' => 'req-lease-refresh-n1-0001',
]);
$credN1a = (string) $service->issueRefreshCredential(['lease_uuid' => $n1LeaseRemoved['lease_uuid'], 'idempotency_key' => 'cred-n1-0001', 'request_id' => 'req-cred-n1-0001'])['refresh_credential'];
$n1LeaseDeactivated = $issuer->issueLease([
    'account_uuid' => $N1, 'product_code' => PRODUCT_PAID, 'node_id' => 'node-deactivated-001',
    'device_public_key' => PAID_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-n1-0002', 'request_id' => 'req-lease-refresh-n1-0002',
]);
$credN1b = (string) $service->issueRefreshCredential(['lease_uuid' => $n1LeaseDeactivated['lease_uuid'], 'idempotency_key' => 'cred-n1-0002', 'request_id' => 'req-cred-n1-0002'])['refresh_credential'];
$db->exec("UPDATE wp_wpuiai_authority_nodes SET status = 'deactivated' WHERE node_uuid = 'node-deactivated-001'");
$db->exec("DELETE FROM wp_wpuiai_authority_nodes WHERE node_uuid = 'node-removed-001'");
$nodeRemoval = $service->refresh($request($N1, 'node-removed-001', 'refresh-n1-removed-0001', $credN1a, 42));
expect_refusal($nodeRemoval, 'NODE_NOT_FOUND', $service, 'node removal denies refresh');
expect_refresh(($issuer->findLease($n1LeaseRemoved['lease_uuid'])['status'] ?? '') === 'superseded', 'node-removed lease settled to superseded/node_removed');
$nodeDeactivated = $service->refresh($request($N1, 'node-deactivated-001', 'refresh-n1-deactivated-0001', $credN1b, 43));
expect_refusal($nodeDeactivated, 'NODE_NOT_ACTIVE', $service, 'node deactivation denies refresh');

// ── Settlement: EDD-truth re-reads (license/order/download) ──
$issueForNode = static function (string $node, string $ikey) use ($issuer): array {
    return $issuer->issueLease([
        'account_uuid' => $GLOBALS['b1Uuid'],
        'product_code' => PRODUCT_PAID,
        'node_id' => $node,
        'device_public_key' => PAID_DEVICE_KEY,
        'idempotency_key' => $ikey,
        'request_id' => 'req-' . $ikey,
    ]);
};
$GLOBALS['b1Uuid'] = $B1;
$b1Seeds = [];
$b1Creds = [];
foreach (['node-lic-001', 'node-cust-001', 'node-pending-001', 'node-download-001'] as $index => $node) {
    $b1Seeds[$node] = $issueForNode($node, 'lease-refresh-b1-' . str_pad((string) ($index + 1), 4, '0', STR_PAD_LEFT));
    $b1Creds[$node] = (string) $service->issueRefreshCredential([
        'lease_uuid' => $b1Seeds[$node]['lease_uuid'],
        'idempotency_key' => 'cred-b1-' . str_pad((string) ($index + 1), 4, '0', STR_PAD_LEFT),
        'request_id' => 'req-cred-b1-' . str_pad((string) ($index + 1), 4, '0', STR_PAD_LEFT),
    ])['refresh_credential'];
}
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE license_id = 8101");
$db->exec("UPDATE wp_edd_licenses SET customer_id = 9999 WHERE license_id = 8102");
$db->exec("UPDATE wp_edd_orders SET status = 'pending' WHERE order_id = 9803");
$db->exec("UPDATE wp_edd_licenses SET download_id = 1004 WHERE license_id = 8104");
$eddLicRefusal = $service->refresh($request($B1, 'node-lic-001', 'refresh-b1-lic-0001', $b1Creds['node-lic-001'], 42));
expect_refusal($eddLicRefusal, 'EDD_LICENSE_UNUSABLE', $service, 'unusable EDD license denies refresh');
$eddCustRefusal = $service->refresh($request($B1, 'node-cust-001', 'refresh-b1-cust-0001', $b1Creds['node-cust-001'], 43));
expect_refusal($eddCustRefusal, 'LICENSE_ACCOUNT_MISMATCH', $service, 'license of another customer denies refresh');
$eddPendingRefusal = $service->refresh($request($B1, 'node-pending-001', 'refresh-b1-pending-0001', $b1Creds['node-pending-001'], 44));
expect_refusal($eddPendingRefusal, 'EDD_ORDER_PENDING', $service, 'unsatisfied EDD order denies refresh');
$eddDownloadRefusal = $service->refresh($request($B1, 'node-download-001', 'refresh-b1-download-0001', $b1Creds['node-download-001'], 45));
expect_refusal($eddDownloadRefusal, 'EDD_ORDER_UNVERIFIED', $service, 'download/price mismatch denies refresh');
expect_refresh(($issuer->findLease($b1Seeds['node-lic-001']['lease_uuid'])['status'] ?? '') === 'superseded', 'EDD-truth refusal settles the lease to superseded');

// ── Settlement: expiry (evaluation, no offline grace; paid past grace) ──
$e2Lease = $issuer->issueLease([
    'account_uuid' => $E2, 'product_code' => PRODUCT_EVAL, 'node_id' => 'node-eval-expire-001',
    'device_public_key' => EVAL_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-e2-0001', 'request_id' => 'req-lease-refresh-e2-0001',
]);
$credE2 = (string) $service->issueRefreshCredential(['lease_uuid' => $e2Lease['lease_uuid'], 'idempotency_key' => 'cred-e2-0001', 'request_id' => 'req-cred-e2-0001'])['refresh_credential'];
$expiryService = build_components($db, '2026-09-08T18:30:00Z')['service'];
$expiryRefusal = $expiryService->refresh($request($E2, 'node-eval-expire-001', 'refresh-e2-expired-0001', $credE2, 7, PRODUCT_EVAL));
expect_refusal($expiryRefusal, 'EXPIRED', $expiryService, 'evaluation lease past expiry denies refresh (no local extension)');
expect_refresh(($issuer->findLease($e2Lease['lease_uuid'])['status'] ?? '') === 'superseded', 'expired lease settled to superseded/lease_expired');
$pastGraceService = build_components($db, '2027-06-01T00:00:00Z')['service'];
$pastGraceRefusal = $pastGraceService->refresh($request($F1, 'node-grace-001', 'refresh-f1-expired-0001', $f1NewCred, 12));
expect_refusal($pastGraceRefusal, 'EXPIRED', $pastGraceService, 'paid lease past offline grace denies refresh (no local extension)');

// ── Hard failures (endpoint-level DomainException codes) ──
expect_refresh_throws(
    static fn() => $service->refresh($request('00000000-0000-4000-8000-000000000000', 'node-paid-golden-001', 'refresh-unknown-0001', $credA1, 42)),
    'ACCOUNT_NOT_FOUND',
    'unknown account fails closed',
);
expect_refresh_throws(
    static fn() => $service->refresh($request($P1, 'node-pending-000', 'refresh-pending-0001', 'rc_' . str_repeat('cd', 24), null)),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified account fails closed',
);
expect_refresh_throws(
    static fn() => $service->refresh(array_merge($request($A1, 'node-paid-golden-001', 'refresh-product-0001', $credA1, 42), ['product_code' => 'focusa_unknown_product'])),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown product code fails closed',
);
expect_refresh_throws(
    static fn() => $service->refresh($request($X1, 'node-none-001', 'refresh-none-0001', 'rc_' . str_repeat('ef', 24), null)),
    'LEASE_NOT_FOUND',
    'refresh without a current active lease fails closed',
);
$e1Lease = $issuer->issueLease([
    'account_uuid' => $E1, 'product_code' => PRODUCT_EVAL, 'node_id' => 'node-eval-golden-001',
    'device_public_key' => EVAL_DEVICE_KEY, 'idempotency_key' => 'lease-refresh-e1-0001', 'request_id' => 'req-lease-refresh-e1-0001',
]);
expect_refresh_throws(
    static fn() => $service->refresh($request($E1, 'node-eval-golden-001', 'refresh-e1-nocred-0001', 'rc_' . str_repeat('01', 24), 7, PRODUCT_EVAL)),
    'REFRESH_CREDENTIAL_REQUIRED',
    'refresh without an issued credential fails closed',
);
expect_refresh_throws(
    static fn() => $service->refresh(array_merge($request($A1, 'node-paid-golden-001', 'refresh-grants-0001', $credA1, 43), ['price' => '9.99'])),
    'CALLER_CONTROLLED_GRANT_DENIED',
    'caller-supplied price is never accepted on refresh',
);
expect_refresh_throws(
    static fn() => $service->refresh(array_merge($request($A1, 'node-paid-golden-001', 'refresh-email-0001', $credA1, 43), ['email' => 'leak@example.invalid'])),
    'INPUT_RAW_EMAIL_FORBIDDEN',
    'raw email input fails closed',
);
expect_refresh_invalid(
    static fn() => $service->refresh(['account_uuid' => 'not-a-uuid', 'product_code' => PRODUCT_PAID, 'node_id' => 'node-paid-golden-001', 'refresh_credential' => $credA1, 'idempotency_key' => 'refresh-baduuid-0001', 'request_id' => 'req-refresh-baduuid-0001']),
    'malformed account uuid fails closed',
);
expect_refresh_invalid(
    static fn() => $service->refresh($request($A1, 'node-paid-golden-001', 'refresh-emptycred-0001', '', 43)),
    'empty refresh credential fails closed',
);

// ── verifyRefusal negative matrix (signed but invalid) ──
$refusalPayload = json_decode(FocusaSpec152eAuthorityKeySetSeam::decodePayload($refundRefusal['refusal']['payload_b64']), true, 512, JSON_THROW_ON_ERROR);
$tampered = $refusalPayload;
$tampered['presented_sequence'] = 99;
$tamperedRefusal = $keySet->seal($tampered, FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID, $keySet->leaseSeed(), FocusaSpec152eEd25519Signer::LEASE_DOMAIN);
expect_refresh_throws(
    static fn() => $service->verifyRefusal($tamperedRefusal, ['now' => NOW]),
    'REFUSAL_STALE',
    'a refusal with authority below the presented sequence is stale',
);
$expiredRefusalPayload = $refusalPayload;
$expiredRefusalPayload['expires_at'] = '2026-08-01T00:00:00Z';
$expiredRefusal = $keySet->seal($expiredRefusalPayload, FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID, $keySet->leaseSeed(), FocusaSpec152eEd25519Signer::LEASE_DOMAIN);
expect_refresh_throws(
    static fn() => $service->verifyRefusal($expiredRefusal, ['now' => NOW]),
    'EXPIRED',
    'an expired refusal is rejected',
);
$unknownReason = $refusalPayload;
$unknownReason['reason_code'] = 'CLIENT_INVENTED_REASON';
$unknownReasonEnvelope = $keySet->seal($unknownReason, FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID, $keySet->leaseSeed(), FocusaSpec152eEd25519Signer::LEASE_DOMAIN);
expect_refresh_throws(
    static fn() => $service->verifyRefusal($unknownReasonEnvelope, ['now' => NOW]),
    'REFUSAL_REASON_UNKNOWN',
    'a refusal with an unbounded reason code is rejected',
);
$wrongKeyEnvelope = $refundRefusal['refusal'];
$wrongKeyEnvelope['signer_key_id'] = 'unknown-key-0001';
expect_refresh_throws(
    static fn() => $service->verifyRefusal($wrongKeyEnvelope, ['now' => NOW]),
    'UNKNOWN_KEY',
    'a refusal under an unknown authority key is rejected',
);
$tamperedSignature = $refundRefusal['refusal'];
$tamperedSignature['signature_b64'] = base64_encode(str_repeat("\x00", 64));
expect_refresh_throws(
    static fn() => $service->verifyRefusal($tamperedSignature, ['now' => NOW]),
    'INVALID_SIGNATURE',
    'a refusal with an invalid signature is rejected',
);
$wrongAccount = $refundRefusal['refusal'];
expect_refresh_throws(
    static fn() => $service->verifyRefusal($wrongAccount, ['now' => NOW, 'expected_account_uuid' => '00000000-0000-4000-8000-000000000000']),
    'WRONG_ACCOUNT',
    'a refusal bound to another account is rejected',
);

// ── Routes seam: bounded POST /v1/lease/refresh ──
$routeOk = FocusaSpec152eLeaseRefreshRoutes::resolveRoute('POST', '/v1/lease/refresh');
expect_refresh(($routeOk['ok'] ?? false) === true && ($routeOk['authority_route'] ?? '') === '/v1/lease/refresh', 'route seam resolves POST /v1/lease/refresh');
$routeBad = FocusaSpec152eLeaseRefreshRoutes::resolveRoute('GET', '/v1/lease/refresh');
expect_refresh(($routeBad['ok'] ?? false) === false && ($routeBad['error'] ?? '') === 'FACADE_ROUTE_DENIED', 'route seam fails closed on wrong method');
$maskedRotated = FocusaSpec152eLeaseRefreshRoutes::maskedResponse($rotation);
expect_refresh(($maskedRotated['state'] ?? '') === 'activated' && isset($maskedRotated['lease_envelope']), 'masked rotation response carries state + lease envelope');
expect_refresh(!array_key_exists('refresh_credential', $maskedRotated), 'masked response never exposes the refresh credential');
$maskedRefusal = FocusaSpec152eLeaseRefreshRoutes::maskedResponse($refundRefusal);
expect_refresh(($maskedRefusal['state'] ?? '') === 'recovery_only' && ($maskedRefusal['next_action'] ?? '') === 'use_recovery', 'masked refusal response carries recovery posture');
$handledRefund = FocusaSpec152eLeaseRefreshRoutes::handle($service, $request($R1, 'node-refund-001', 'refresh-r1-http-0001', $credR1, 42));
expect_refresh(($handledRefund['ok'] ?? false) === true && ($handledRefund['status'] ?? 0) === 200, 'route handle returns the refusal as a 200 recovery result');
$handledInvalid = FocusaSpec152eLeaseRefreshRoutes::handle($service, $request($A1, 'node-other-001', 'refresh-a1-0001', $credA1, 43));
expect_refresh(($handledInvalid['ok'] ?? false) === false && ($handledInvalid['error'] ?? '') === 'IDEMPOTENCY_CONFLICT', 'route handle maps hard failures to bounded error envelopes');

// ── Hygiene: no email, license key, secret, synthetic key, or plaintext credential anywhere ──
$raw = '';
foreach (['wp_wpuiai_lease_refresh_log', 'wp_wpuiai_lease_refresh_credentials', 'wp_wpuiai_lease_refresh_idempotency',
          'wp_wpuiai_authority_outbox', 'wp_wpuiai_outbox_deliveries', 'wp_wpuiai_edd_lifecycle_events',
          'wp_wpuiai_authority_leases', 'wp_wpuiai_authority_lease_sequences', 'wp_wpuiai_authority_lease_idempotency'] as $table) {
    $rows = $db->query("SELECT * FROM {$table}")->fetchAll(PDO::FETCH_ASSOC);
    $raw .= json_encode($rows);
}
$results = [$rotation, $replayRefresh, $refundRefusal, $revokeRefusal, $staleRefusal, $seqRefusal, $credRefusal,
            $nodeRemoval, $nodeDeactivated, $eddLicRefusal, $eddCustRefusal, $eddPendingRefusal, $eddDownloadRefusal,
            $expiryRefusal, $pastGraceRefusal, $graceRotation];
foreach ($results as $result) {
    $raw .= json_encode($result['refusal'] ?? []);
    if (($result['lease'] ?? null) !== null) {
        $raw .= json_encode($result['lease']['envelope'] ?? []) . json_encode($result['lease']['claims'] ?? []);
    }
}
expect_refresh(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $raw) !== 1, 'hygiene: no unmasked email in refresh outputs or journals');
expect_refresh(preg_match('/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/', $raw) !== 1, 'hygiene: no license key material in refresh outputs');
expect_refresh(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $raw) !== 1, 'hygiene: no secret prefixes in refresh outputs');
expect_refresh(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/i', $raw) !== 1, 'hygiene: no synthetic focusa_live keys in refresh outputs');
$dbDump = json_encode($db->query('SELECT * FROM wp_wpuiai_lease_refresh_log')->fetchAll(PDO::FETCH_ASSOC))
    . json_encode($db->query('SELECT * FROM wp_wpuiai_lease_refresh_credentials')->fetchAll(PDO::FETCH_ASSOC))
    . json_encode($db->query('SELECT * FROM wp_wpuiai_authority_outbox')->fetchAll(PDO::FETCH_ASSOC));
foreach ([$credA1, $rotCred, $f1NewCred, $credR1, $credV1, $credS1, $credM1, $credE2] as $plaintext) {
    expect_refresh(strpos($dbDump, $plaintext) === false, 'hygiene: plaintext refresh credential is never stored at rest');
}
expect_refresh(preg_match('/rc_[0-9a-f]{48}/', $dbDump) !== 1, 'hygiene: no refresh credential token appears in any stored row');

echo json_encode([
    'schema' => 'focusa.spec152e.lease_refresh_lifecycle_validation.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'rotations' => 2,
    'refusals' => 15,
    'settlements' => ['refunded', 'revoked', 'expired', 'superseded_stale', 'superseded_node', 'superseded_edd'],
    'highest_sequence_after_rotation' => 43,
    'result' => 'passed_fail_closed',
], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), "\n";
