<?php
// 152E.02.08 Add transactional authority outbox and idempotent dispatcher.
// A durable transactional outbox (wp_wpuiai_authority_outbox) records EDD order,
// license, refund, revoke, expiry, customer, node, and lease transitions. Every event
// is appended in the SAME transaction as the canonical EDD/account state change: a
// committed change always carries its outbox row and a rolled-back (crashed)
// transaction carries neither, so an injected crash loses no committed event and
// creates no orphan. Each event is a bounded signed envelope (bounded event schema,
// opaque refs only, envelope digest, server-side HMAC signature with explicit key id);
// the dispatcher verifies the envelope before dispatch and tampered envelopes fail
// closed into the dead-letter state and are never delivered. Delivery is exactly once:
// consumer application, delivery ledger, and dispatched mark commit in one transaction,
// so a crash before the dispatch commit redelivers exactly once and a crash after the
// commit never redelivers — an injected crash/retry duplicates no entitlement. Durable
// failure state is bounded and never blocks canonical EDD commit: failed deliveries
// retry with exponential backoff and a bounded error code; exhausted rows move to
// dead-letter with a bounded repair record; repairState() and retryDeadLetter() expose
// and repair exactly that state; replay() is idempotent. No raw email, payment secret,
// license key, or unmasked real-email evidence is accepted or stored; no client-
// controlled price/amount/grant/feature/limit/tier/download field is accepted.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_outbox(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_outbox_throws(callable $operation, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    try {
        $operation();
    } catch (Throwable $error) {
        if ($error->getMessage() !== $code) {
            fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
            exit(1);
        }
        return;
    }
    fwrite(STDERR, "FAIL: {$message}\n");
    exit(1);
}

/** Hook negative: run inside a caller-owned transaction and roll it back after the fail-closed throw. */
function expect_hook_throws(callable $operation, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    $db = $GLOBALS['db'];
    $db->beginTransaction();
    try {
        $operation();
    } catch (Throwable $error) {
        $db->rollBack();
        if ($error->getMessage() !== $code) {
            fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
            exit(1);
        }
        return;
    }
    $db->rollBack();
    fwrite(STDERR, "FAIL: {$message}\n");
    exit(1);
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$GLOBALS['db'] = $db;

$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_outbox_test']);
$outboxMigration = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
$outboxMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_outbox_test']);

// Canonical EDD fixture tables (mirrors the lifecycle-projection fixtures).
$db->exec("CREATE TABLE wp_edd_customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '',
    purchase_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0,
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status VARCHAR(32) NOT NULL,
    customer_id BIGINT NOT NULL,
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
// Test-owned consumer side-effect table: proves exactly-once application under crashes.
$db->exec("CREATE TABLE wp_outbox_test_applications (
    event_uuid VARCHAR(128) NOT NULL PRIMARY KEY,
    event_type VARCHAR(32) NOT NULL,
    applied_at VARCHAR(32) NOT NULL
)");

$insertCustomer = $db->prepare("INSERT INTO wp_edd_customers (email, name, date_created) VALUES (:email, :name, :created)");
$insertCustomer->execute([':email' => 'customer101@example.test', ':name' => 'Test Customer 101', ':created' => '2026-08-08T00:00:00Z']);
$customerId = (int) $db->lastInsertId();

$orderRows = [];
$licenseRows = [];
for ($orderId = 1001; $orderId <= 1005; $orderId++) {
    $db->prepare("INSERT INTO wp_edd_orders (status, customer_id, date_created) VALUES ('pending', :customer, :created)")
        ->execute([':customer' => $customerId, ':created' => '2026-08-08T00:00:00Z']);
    $orderRows[$orderId] = (int) $db->lastInsertId();
}
for ($licenseId = 1; $licenseId <= 5; $licenseId++) {
    $db->prepare("INSERT INTO wp_edd_licenses (license_key, customer_id, product_id, order_id, status, date_created)
        VALUES (:key, :customer, :product, :order, 'active', :created)")
        ->execute([
            ':key' => strtoupper('FOCUSA-TEST-') . str_pad((string) $licenseId, 4, '0', STR_PAD_LEFT) . '-TESTKEY',
            ':customer' => $customerId,
            ':product' => 1000 + $licenseId,
            ':order' => $orderRows[1000 + $licenseId],
            ':created' => '2026-08-08T00:00:00Z',
        ]);
    $licenseRows[$licenseId] = (int) $db->lastInsertId();
}
$db->prepare("INSERT INTO wp_edd_subscriptions (customer_id, product_id, status, date_created) VALUES (:customer, 1008, 'active', :created)")
    ->execute([':customer' => $customerId, ':created' => '2026-08-08T00:00:00Z']);
$subscriptionRow = (int) $db->lastInsertId();
$historyBaseline = [
    'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
    'orders' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_orders')->fetchColumn(),
    'licenses' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn(),
    'subscriptions' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_subscriptions')->fetchColumn(),
];

$nowValue = '2026-08-08T00:01:00Z';
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};
/** Advance the deterministic clock by whole seconds. */
$tick = static function (int $seconds) use (&$nowValue): void {
    $nowValue = gmdate('Y-m-d\TH:i:s\Z', (int) (new DateTimeImmutable($nowValue))->format('U') + $seconds);
};

$accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
$promoted = $accounts->resolveForPromotionInTransaction($customerId, null, 'cus_test_101', FocusaSpec152eAuthorityAccountMigration::encodeProvenance(['source' => 'authority_outbox_test']), '2026-08-08T00:00:00Z');
$accountUuid = (string) $promoted['account']['account_uuid'];
expect_outbox($accountUuid !== '', 'promoted authority account resolved');

$secret = 'spec152e-outbox-hmac-test-secret-v1';
$signer = new FocusaSpec152eAuthorityEventSigner($secret);
$eventSchema = new FocusaSpec152eAuthorityEventSchema();

// Deterministic consumer with injectable failure modes.
$consumerMode = 'normal';
$deliver = static function (array $event) use ($db, &$consumerMode, &$nowValue): void {
    $statement = $db->prepare('INSERT INTO wp_outbox_test_applications (event_uuid, event_type, applied_at) VALUES (:uuid, :type, :at)');
    $statement->execute([':uuid' => (string) $event['event_uuid'], ':type' => (string) $event['event_type'], ':at' => $nowValue]);
    if ($consumerMode === 'crash') {
        throw new RuntimeException('simulated process crash after applying the consumer effect');
    }
    if ($consumerMode === 'consumer_down') {
        throw new DomainException('DELIVERY_CONSUMER_DOWN');
    }
};

$hook = new FocusaSpec152eEddAuthorityHook($db, $outboxMigration, $eventSchema, $signer, $accounts, 'wp_', $clock);
$dispatcher = new FocusaSpec152eAuthorityOutboxDispatcher($db, $outboxMigration, $signer, $eventSchema, $deliver, $clock, 'wp_', 3, 60);

function outboxCount(PDO $db): int
{
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_outbox')->fetchColumn();
}

function deliveryCount(PDO $db): int
{
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_outbox_deliveries')->fetchColumn();
}

function applicationCount(PDO $db): int
{
    return (int) $db->query('SELECT COUNT(*) FROM wp_outbox_test_applications')->fetchColumn();
}

function accountSequence(PDO $db, string $accountUuid): int
{
    $statement = $db->prepare('SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = :uuid');
    $statement->execute([':uuid' => $accountUuid]);
    return (int) $statement->fetchColumn();
}

function orderStatus(PDO $db, int $orderId): string
{
    $statement = $db->prepare('SELECT status FROM wp_edd_orders WHERE id = :id');
    $statement->execute([':id' => $orderId]);
    return (string) $statement->fetchColumn();
}

// ── Bounded event schema and signer ───────────────────────────────────

expect_outbox(count(FocusaSpec152eAuthorityEventSchema::EVENT_TYPES) === 23, 'event schema registry is bounded at 23 event types');
expect_outbox(FocusaSpec152eAuthorityEventSchema::SURFACES === ['customer', 'order', 'license', 'refund', 'subscription', 'node', 'lease'], 'surfaces are exactly customer/order/license/refund/subscription/node/lease');
expect_outbox(FocusaSpec152eAuthorityEventSchema::KEY_ID === 'wpuiai.spec152e.outbox.v1', 'signing key id is explicit and server-owned');
expect_outbox(FocusaSpec152eAuthorityEventSigner::SIGNATURE_ALGORITHM === 'hmac_sha256.spec152e.outbox.v1', 'signature algorithm is explicit');
expect_outbox($signer->keyId() === 'wpuiai.spec152e.outbox.v1', 'signer owns the canonical key id');

$payloadProbe = FocusaSpec152eAuthorityOutboxMigration::encodeCanonical(['event_type' => 'order_completed', 'nonce' => 'probe']);
$probeDigest = hash('sha256', $payloadProbe);
$signed = $signer->sign($payloadProbe, $probeDigest);
expect_outbox(preg_match('/^sig_v1:[A-Za-z0-9._:-]{1,64}:[0-9a-f]{64}$/D', $signed['signature']) === 1, 'signature is a bounded opaque envelope signature');
expect_outbox($signed['signing_key_id'] === 'wpuiai.spec152e.outbox.v1', 'signed envelope carries the explicit key id');
$signer->verify($payloadProbe, $probeDigest, $signed['signature'], $signed['signing_key_id']);
expect_outbox(true, 'signer verify round-trips a genuine envelope');
expect_outbox_throws(static fn () => $signer->verify($payloadProbe . 'x', $probeDigest, $signed['signature'], $signed['signing_key_id']), 'OUTBOX_ENVELOPE_TAMPERED', 'tampered payload fails verification');
expect_outbox_throws(static fn () => $signer->verify($payloadProbe, hash('sha256', 'other'), $signed['signature'], $signed['signing_key_id']), 'OUTBOX_ENVELOPE_TAMPERED', 'digest mismatch fails verification');
expect_outbox_throws(static fn () => $signer->verify($payloadProbe, $probeDigest, 'sig_v1:attacker:0000', $signed['signing_key_id']), 'INVALID_SIGNATURE', 'malformed signature fails closed');
expect_outbox_throws(static fn () => $signer->verify($payloadProbe, $probeDigest, $signed['signature'], 'attacker.key.v1'), 'UNKNOWN_SIGNING_KEY', 'unknown signing key fails closed');
expect_outbox_throws(static fn () => new FocusaSpec152eAuthorityEventSigner('short'), 'server-side signing secret required', 'weak server secret is rejected');

// Schema positives: derived surface/transition, node and lease events.
$licenseEvent = $eventSchema->validate([
    'event_type' => 'license_expired', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'license_id' => $licenseRows[1], 'surface' => 'license', 'transition' => 'expire',
    'request_id' => 'req_schema_license', 'idempotency_key' => 'idem_schema_license',
]);
expect_outbox($licenseEvent['surface'] === 'license' && $licenseEvent['transition'] === 'expire', 'registry derives the exact surface/transition for license expiry');
$nodeEvent = $eventSchema->validate([
    'event_type' => 'node_registered', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'node_uuid' => '00000000-0000-4000-8000-000000000001',
    'request_id' => 'req_schema_node', 'idempotency_key' => 'idem_schema_node',
]);
expect_outbox($nodeEvent['surface'] === 'node' && $nodeEvent['node_uuid'] !== null, 'node registration event validates');
$leaseEvent = $eventSchema->validate([
    'event_type' => 'lease_issued', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'lease_uuid' => '00000000-0000-4000-8000-000000000002', 'license_id' => $licenseRows[1],
    'request_id' => 'req_schema_lease', 'idempotency_key' => 'idem_schema_lease',
]);
expect_outbox($leaseEvent['surface'] === 'lease' && $leaseEvent['lease_uuid'] !== null && $leaseEvent['license_id'] === $licenseRows[1], 'lease event validates with lease and license references');

// Schema negatives.
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'frobnicate', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'request_id' => 'req_neg_type', 'idempotency_key' => 'idem_neg_type']), 'OUTBOX_EVENT_TYPE_UNKNOWN', 'unknown event type fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'request_id' => 'req_neg_ref', 'idempotency_key' => 'idem_neg_ref']), 'OUTBOX_EVENT_FIELD_MISSING', 'missing required reference fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'surface' => 'license', 'request_id' => 'req_neg_surface', 'idempotency_key' => 'idem_neg_surface']), 'OUTBOX_SURFACE_MISMATCH', 'caller-declared surface mismatch fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'transition' => 'refund', 'request_id' => 'req_neg_transition', 'idempotency_key' => 'idem_neg_transition']), 'OUTBOX_TRANSITION_MISMATCH', 'caller-declared transition mismatch fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'state_reason' => 'contact me@example.test', 'request_id' => 'req_neg_email', 'idempotency_key' => 'idem_neg_email']), 'INPUT_RAW_EMAIL_FORBIDDEN', 'raw email input fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'price' => 9.99, 'request_id' => 'req_neg_price', 'idempotency_key' => 'idem_neg_price']), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled price fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'grants' => ['release' => true], 'request_id' => 'req_neg_grants', 'idempotency_key' => 'idem_neg_grants']), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled grants fail closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => 'not-a-uuid', 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_uuid', 'idempotency_key' => 'idem_neg_uuid']), 'bounded account UUID required', 'malformed account uuid fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'x', 'idempotency_key' => 'idem_neg_request']), 'bounded request ID required', 'malformed request id fails closed');
expect_outbox_throws(static fn () => $eventSchema->validate(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_idem', 'idempotency_key' => 'x']), 'bounded idempotency key required', 'malformed idempotency key fails closed');

// ── Fixture 1: atomic append + injected crash rollback ────────────────

expect_outbox_throws(static fn () => $hook->append(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_txn', 'idempotency_key' => 'idem_neg_txn']), 'OUTBOX_APPEND_REQUIRES_TRANSACTION', 'outbox append outside a transaction fails closed');
expect_outbox(outboxCount($db) === 0, 'no outbox row exists before any committed change');

// Committed append: canonical order change + outbox row in one transaction.
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'completed', ':id' => $orderRows[1001]]);
$e_o1 = $hook->appendFromEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'request_id' => 'req_o1_complete', 'idempotency_key' => 'idem_o1_complete',
]);
$db->commit();
expect_outbox(outboxCount($db) === 1, 'committed canonical change carries exactly one outbox row');
expect_outbox(orderStatus($db, $orderRows[1001]) === 'completed', 'canonical order truth committed with the outbox row');
expect_outbox($e_o1['event_type'] === 'order_completed' && $e_o1['surface'] === 'order' && $e_o1['transition'] === 'complete', 'hook maps order completed to the canonical event');
$rowO1 = $dispatcher->findByEventUuid((string) $e_o1['event_uuid']);
expect_outbox($rowO1 !== null && $rowO1['dispatch_state'] === 'pending' && (int) $rowO1['attempts'] === 0, 'appended event is pending with a zero attempt budget');
expect_outbox((int) $rowO1['authority_sequence'] === 0 && (int) $rowO1['result_sequence'] === 0, 'event snapshots the canonical authority sequence at append time');
expect_outbox(hash_equals((string) $rowO1['envelope_digest'], hash('sha256', (string) $rowO1['payload'])), 'stored envelope digest matches the stored payload');
expect_outbox(preg_match('/^sig_v1:[A-Za-z0-9._:-]{1,64}:[0-9a-f]{64}$/D', (string) $rowO1['signature']) === 1, 'stored signature is bounded and opaque');

// Crash before commit: rollback removes the canonical change AND its outbox row together.
$beforeRollback = outboxCount($db);
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'completed', ':id' => $orderRows[1002]]);
$hook->appendFromEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1002], 'request_id' => 'req_o2_rollback', 'idempotency_key' => 'idem_o2_rollback',
]);
$db->rollBack();
expect_outbox(outboxCount($db) === $beforeRollback, 'injected crash rollback loses no committed event (no orphan outbox row)');
expect_outbox(orderStatus($db, $orderRows[1002]) === 'pending', 'injected crash rollback reverts the canonical change with its outbox row');

// Hook status mapping negatives.
expect_outbox_throws(static fn () => $hook->appendFromEdd(['surface' => 'order', 'status' => 'processing', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_status', 'idempotency_key' => 'idem_neg_status']), 'EDD_STATUS_UNKNOWN', 'unmapped EDD status fails closed');
expect_hook_throws(static fn () => $hook->appendFromEdd(['surface' => 'refund', 'status' => 'refunded', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'order_id' => 99999, 'request_id' => 'req_neg_order', 'idempotency_key' => 'idem_neg_order']), 'EDD_ORDER_RESOLUTION_FAILED', 'missing canonical EDD order fails closed');

// Second committed append plus a customer-promotion append.
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'completed', ':id' => $orderRows[1002]]);
$e_o2 = $hook->appendFromEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1002], 'request_id' => 'req_o2_complete', 'idempotency_key' => 'idem_o2_complete',
]);
$db->commit();
$db->beginTransaction();
$e_cp = $hook->append([
    'event_type' => 'customer_promoted', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'request_id' => 'req_cp_promote', 'idempotency_key' => 'idem_cp_promote',
]);
$db->commit();
expect_outbox(outboxCount($db) === 3, 'three committed events journaled (order x2 + customer promotion)');

// ── Fixture 2: hook snapshots the sequence but never bumps it ─────────

$accounts->advanceSequence($accountUuid, 1, 'idem_seq_snapshot');
expect_outbox(accountSequence($db, $accountUuid) === 1, 'account sequence advanced to 1 by lifecycle truth');
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'completed', ':id' => $orderRows[1003]]);
$e_o3 = $hook->appendFromEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1003], 'request_id' => 'req_o3_complete', 'idempotency_key' => 'idem_o3_complete',
]);
$db->commit();
expect_outbox((int) $e_o3['authority_sequence'] === 1 && (int) $e_o3['result_sequence'] === 1, 'appended event snapshots the canonical sequence at append time');
expect_outbox(accountSequence($db, $accountUuid) === 1, 'outbox append never bumps the authority sequence (lifecycle truth owns advancement)');

// ── Fixture 3: dispatch, injected crash mid-delivery, idempotent replay ─

$consumerMode = 'normal';
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['dispatched'] ?? 0) === 4, 'first dispatch delivers all four due pending events');
expect_outbox(deliveryCount($db) === 4, 'exactly one delivery ledger row per dispatched event');
expect_outbox(applicationCount($db) === 4, 'consumer applied exactly once per dispatched event');
expect_outbox($dispatcher->stateCount('dispatched') === 4 && $dispatcher->stateCount('pending') === 0, 'dispatched events leave the pending queue');

$replayed = $dispatcher->replay((string) $e_o1['event_uuid']);
expect_outbox($replayed['outcome'] === 'replayed', 'replay of a dispatched event returns the existing delivery');
expect_outbox((string) $replayed['delivery']['event_uuid'] === (string) $e_o1['event_uuid'], 'replayed delivery is byte-identical to the original');
expect_outbox(deliveryCount($db) === 4 && applicationCount($db) === 4, 'idempotent replay duplicates no delivery and no entitlement');

// Injected crash mid-delivery: consumer effect applied then process crash.
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'failed', ':id' => $orderRows[1004]]);
$e_o4 = $hook->appendFromEdd([
    'surface' => 'order', 'status' => 'failed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1004], 'request_id' => 'req_o4_failed', 'idempotency_key' => 'idem_o4_failed',
]);
$db->commit();
$consumerMode = 'crash';
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['failed'] ?? 0) === 1, 'crash mid-delivery is recorded as a bounded dispatch failure');
$rowO4 = $dispatcher->findByEventUuid((string) $e_o4['event_uuid']);
expect_outbox($rowO4['dispatch_state'] === 'failed' && (int) $rowO4['attempts'] === 1 && $rowO4['last_error'] === 'DISPATCH_DELIVERY_FAILED', 'crash exposes bounded durable failure state (attempts, bounded error)');
expect_outbox(deliveryCount($db) === 4 && applicationCount($db) === 4, 'crashed dispatch commits nothing (no delivery, no consumer effect)');
$consumerMode = 'normal';
$tick(120);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['dispatched'] ?? 0) === 1, 'retry after the crash redelivers the pending event');
expect_outbox(deliveryCount($db) === 5 && applicationCount($db) === 5, 'crash/retry loses no event and duplicates no entitlement (exactly-once)');
$rowO4 = $dispatcher->findByEventUuid((string) $e_o4['event_uuid']);
expect_outbox($rowO4['dispatch_state'] === 'dispatched' && (int) $rowO4['attempts'] === 2, 'redelivered event is dispatched with a bounded attempt count');

// ── Fixture 4: retry backoff -> dead letter; canonical commit unblocked ─

$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'refunded', ':id' => $orderRows[1001]]);
$e_o5 = $hook->appendFromEdd([
    'surface' => 'refund', 'status' => 'refunded', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'request_id' => 'req_o5_refund', 'idempotency_key' => 'idem_o5_refund',
]);
$db->commit();
expect_outbox(orderStatus($db, $orderRows[1001]) === 'refunded', 'canonical refund truth committed with its outbox event');

$consumerMode = 'consumer_down';
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['failed'] ?? 0) === 1, 'consumer-down first attempt fails bounded');
$rowO5 = $dispatcher->findByEventUuid((string) $e_o5['event_uuid']);
expect_outbox($rowO5['dispatch_state'] === 'failed' && (int) $rowO5['attempts'] === 1 && $rowO5['last_error'] === 'DELIVERY_CONSUMER_DOWN', 'first failure records the bounded consumer error and attempt 1');
expect_outbox((int) (new DateTimeImmutable((string) $rowO5['next_attempt_at']))->format('U') === (int) (new DateTimeImmutable($nowValue))->format('U') + 60, 'first failure backs off 60 seconds');
$tick(60);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['failed'] ?? 0) === 1, 'second attempt retries after the backoff window');
$rowO5 = $dispatcher->findByEventUuid((string) $e_o5['event_uuid']);
expect_outbox($rowO5['dispatch_state'] === 'failed' && (int) $rowO5['attempts'] === 2, 'second failure records attempt 2');
$tick(120);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['dead_lettered'] ?? 0) === 1, 'attempt budget exhausted moves the event to dead letter');
$rowO5 = $dispatcher->findByEventUuid((string) $e_o5['event_uuid']);
expect_outbox($rowO5['dispatch_state'] === 'dead_letter' && (int) $rowO5['attempts'] === 3 && $rowO5['last_error'] === 'DELIVERY_CONSUMER_DOWN', 'dead letter retains bounded durable failure state');
expect_outbox(orderStatus($db, $orderRows[1001]) === 'refunded', 'dispatch failure never blocks or reverts the canonical EDD commit');
expect_outbox(deliveryCount($db) === 5 && applicationCount($db) === 5, 'dead-lettered event was never delivered');
expect_outbox((int) (new DateTimeImmutable((string) $rowO5['retention_until']))->format('U') > (int) (new DateTimeImmutable($nowValue))->format('U'), 'dead-letter row carries a bounded retention horizon');

$repair = $dispatcher->repairState();
expect_outbox($repair['states'] === ['pending' => 0, 'dispatched' => 5, 'failed' => 0, 'dead_letter' => 1] && $repair['total'] === 6, 'repair state exposes bounded counts across dispatch states');
expect_outbox(count($repair['rows']) === 1 && $repair['rows'][0]['dispatch_state'] === 'dead_letter' && $repair['rows'][0]['last_error'] === 'DELIVERY_CONSUMER_DOWN', 'repair state exposes the bounded dead-letter record');
expect_outbox(array_key_exists('event_type', $repair['rows'][0]) && !array_key_exists('payload', $repair['rows'][0]), 'repair state never exposes payload internals');

expect_outbox($dispatcher->retryDeadLetter([(string) $e_o5['event_uuid']]) === 1, 'bounded retry routine re-queues exactly the dead-lettered event');
$rowO5 = $dispatcher->findByEventUuid((string) $e_o5['event_uuid']);
expect_outbox($rowO5['dispatch_state'] === 'pending' && (int) $rowO5['attempts'] === 0 && $rowO5['last_error'] === null, 're-queued event resets the bounded attempt budget');
$consumerMode = 'normal';
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['dispatched'] ?? 0) === 1, 'repaired event dispatches on the next cycle');
expect_outbox(deliveryCount($db) === 6 && applicationCount($db) === 6, 'repair delivers exactly once with no duplicated entitlement');
$repair = $dispatcher->repairState();
expect_outbox($repair['states']['dead_letter'] === 0 && $repair['states']['pending'] === 0 && count($repair['rows']) === 0, 'repair state is clean after bounded repair');

// ── Fixture 5: tampered envelopes -> dead letter, never delivered ─────

$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = :id')->execute([':status' => 'completed', ':id' => $orderRows[1005]]);
$e_o6 = $hook->appendFromEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1005], 'request_id' => 'req_o6_complete', 'idempotency_key' => 'idem_o6_complete',
]);
$db->commit();
$db->prepare('UPDATE wp_wpuiai_authority_outbox SET payload = payload || :junk WHERE event_uuid = :uuid')
    ->execute([':junk' => 'tampered', ':uuid' => (string) $e_o6['event_uuid']]);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['tampered'] ?? 0) === 1, 'tampered payload is quarantined before dispatch');
$rowO6 = $dispatcher->findByEventUuid((string) $e_o6['event_uuid']);
expect_outbox($rowO6['dispatch_state'] === 'dead_letter' && $rowO6['last_error'] === 'OUTBOX_ENVELOPE_TAMPERED', 'tampered payload dead-letters with the bounded tamper code');
expect_outbox(deliveryCount($db) === 6 && applicationCount($db) === 6, 'tampered envelope is never delivered');

$db->beginTransaction();
$e_o7 = $hook->appendFromEdd([
    'surface' => 'subscription', 'status' => 'cancelled', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => $subscriptionRow, 'request_id' => 'req_o7_cancel', 'idempotency_key' => 'idem_o7_cancel',
]);
$db->commit();
$tamperedSignature = (string) $dispatcher->findByEventUuid((string) $e_o7['event_uuid'])['signature'];
$tamperedSignature = substr($tamperedSignature, 0, -1) . ($tamperedSignature[strlen($tamperedSignature) - 1] === 'a' ? 'b' : 'a');
$db->prepare('UPDATE wp_wpuiai_authority_outbox SET signature = :signature WHERE event_uuid = :uuid')
    ->execute([':signature' => $tamperedSignature, ':uuid' => (string) $e_o7['event_uuid']]);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['tampered'] ?? 0) === 1, 'tampered signature is quarantined before dispatch');
expect_outbox($dispatcher->findByEventUuid((string) $e_o7['event_uuid'])['last_error'] === 'OUTBOX_ENVELOPE_TAMPERED', 'tampered signature dead-letters with the bounded tamper code');

$db->beginTransaction();
$e_o8 = $hook->appendFromEdd([
    'surface' => 'license', 'status' => 'expired', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'license_id' => $licenseRows[1], 'request_id' => 'req_o8_expire', 'idempotency_key' => 'idem_o8_expire',
]);
$db->commit();
$db->prepare('UPDATE wp_wpuiai_authority_outbox SET signing_key_id = :key WHERE event_uuid = :uuid')
    ->execute([':key' => 'attacker.key.v1', ':uuid' => (string) $e_o8['event_uuid']]);
$summary = $dispatcher->dispatchReady();
expect_outbox(($summary['tampered'] ?? 0) === 1, 'unknown signing key is quarantined before dispatch');
expect_outbox($dispatcher->findByEventUuid((string) $e_o8['event_uuid'])['last_error'] === 'UNKNOWN_SIGNING_KEY', 'unknown signing key dead-letters with the bounded key code');
expect_outbox(deliveryCount($db) === 6 && applicationCount($db) === 6, 'no tampered envelope ever reaches the consumer');

// ── Fixture 6: fail-closed negatives across hook and dispatcher ───────

expect_hook_throws(static fn () => $hook->append(['event_type' => 'order_completed', 'account_uuid' => '00000000-0000-4000-8000-00000000dead', 'edd_customer_id' => $customerId, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_account', 'idempotency_key' => 'idem_neg_account']), 'ENTITLEMENT_REQUIRED', 'unknown authority account fails closed');
expect_hook_throws(static fn () => $hook->append(['event_type' => 'order_completed', 'account_uuid' => $accountUuid, 'edd_customer_id' => 99999, 'order_id' => $orderRows[1001], 'request_id' => 'req_neg_customer', 'idempotency_key' => 'idem_neg_customer']), 'EDD_CUSTOMER_RESOLUTION_FAILED', 'customer/account mismatch fails closed');
expect_hook_throws(static fn () => $hook->append(['event_type' => 'license_expired', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'license_id' => 99999, 'request_id' => 'req_neg_license', 'idempotency_key' => 'idem_neg_license']), 'EDD_LICENSE_RESOLUTION_FAILED', 'missing canonical EDD license fails closed');
expect_hook_throws(static fn () => $hook->append(['event_type' => 'subscription_cancelled', 'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId, 'subscription_id' => 99999, 'request_id' => 'req_neg_sub', 'idempotency_key' => 'idem_neg_sub']), 'EDD_SUBSCRIPTION_RESOLUTION_FAILED', 'missing canonical EDD subscription fails closed');
expect_outbox_throws(static fn () => $dispatcher->replay('evt_0000000000000000000000000000000000000000'), 'OUTBOX_EVENT_NOT_FOUND', 'replay of an unknown event fails closed');
expect_outbox_throws(static fn () => $dispatcher->retryDeadLetter([(string) $e_o1['event_uuid']]), 'OUTBOX_DEAD_LETTER_REQUIRED', 'retry of a non-dead-letter event fails closed');
expect_outbox_throws(static fn () => $dispatcher->retryDeadLetter(['evt_0000000000000000000000000000000000000000']), 'OUTBOX_EVENT_NOT_FOUND', 'retry of an unknown event fails closed');
expect_outbox_throws(static fn () => new FocusaSpec152eAuthorityOutboxDispatcher($db, $outboxMigration, $signer, $eventSchema, $deliver, $clock, 'wp_', 0, 60), 'positive retry budget required', 'non-positive retry budget fails closed');

// ── Fixture 7: preservation, monotonic state, and redaction ───────────

expect_outbox(outboxCount($db) === 9, 'crash/retry/tamper fixtures lose no committed event (all 9 rows retained)');
$stateCounts = $dispatcher->repairState()['states'];
expect_outbox($stateCounts['dispatched'] === 6 && $stateCounts['dead_letter'] === 3 && $stateCounts['failed'] === 0 && $stateCounts['pending'] === 0, 'final outbox state is bounded: 6 dispatched, 3 dead-lettered, nothing stuck pending');
expect_outbox($stateCounts['dispatched'] + $stateCounts['dead_letter'] + $stateCounts['failed'] + $stateCounts['pending'] === outboxCount($db), 'every committed event is in exactly one bounded dispatch state');
expect_outbox(deliveryCount($db) === 6 && applicationCount($db) === 6, 'exactly one delivery and one consumer application per dispatched event');
expect_outbox((int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn() === $historyBaseline['customers'], 'history preserved: customers');
expect_outbox((int) $db->query('SELECT COUNT(*) FROM wp_edd_orders')->fetchColumn() === $historyBaseline['orders'], 'history preserved: orders');
expect_outbox((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === $historyBaseline['licenses'], 'history preserved: licenses');
expect_outbox((int) $db->query('SELECT COUNT(*) FROM wp_edd_subscriptions')->fetchColumn() === $historyBaseline['subscriptions'], 'history preserved: subscriptions');

// Redaction: no raw email, no payment secret, no key material, no signing secret.
$outboxJson = json_encode($db->query('SELECT * FROM wp_wpuiai_authority_outbox')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
$deliveriesJson = json_encode($db->query('SELECT * FROM wp_wpuiai_outbox_deliveries')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
$repairJson = json_encode($dispatcher->repairState(), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
foreach (['outbox' => $outboxJson, 'deliveries' => $deliveriesJson, 'repair' => $repairJson] as $name => $json) {
    expect_outbox(strpos($json, '@') === false, "no raw email in {$name} storage");
    expect_outbox(strpos($json, 'cus_test_101') === false, "no payment/customer secret in {$name} storage");
    expect_outbox(strpos($json, 'FOCUSA-TEST-') === false, "no license key material in {$name} storage");
    expect_outbox(strpos($json, 'example.test') === false, "no unmasked email evidence in {$name} storage");
    expect_outbox(strpos($json, $secret) === false, "server signing secret never leaves the signer ({$name})");
}
$payloads = $db->query('SELECT payload FROM wp_wpuiai_authority_outbox')->fetchAll(PDO::FETCH_COLUMN);
foreach ($payloads as $payload) {
    expect_outbox(strpos((string) $payload, '@') === false, 'no raw email in any signed payload');
    expect_outbox(strpos((string) $payload, $secret) === false, 'no signing secret in any signed payload');
}
$errorCodes = $db->query('SELECT last_error FROM wp_wpuiai_authority_outbox WHERE last_error IS NOT NULL')->fetchAll(PDO::FETCH_COLUMN);
foreach ($errorCodes as $code) {
    expect_outbox(in_array((string) $code, ['DISPATCH_DELIVERY_FAILED', 'DELIVERY_CONSUMER_DOWN', 'OUTBOX_ENVELOPE_TAMPERED', 'UNKNOWN_SIGNING_KEY'], true), 'persisted failure codes are bounded and enumerable');
}

// Rollback preservation: journals are never deleted.
$preserved = $outboxMigration->preserveForRollback('2026-08-08T00:02:00Z', ['source' => 'authority_outbox_test']);
expect_outbox($preserved['action'] === 'preserve' && $preserved['event_key'] !== '', 'rollback is preservation-only (schema event recorded)');
expect_outbox(outboxCount($db) === 9 && deliveryCount($db) === 6, 'rollback preservation never deletes outbox rows or delivery journals');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.authority_outbox_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'event_types' => count(FocusaSpec152eAuthorityEventSchema::EVENT_TYPES),
    'outbox_rows' => outboxCount($db),
    'pending' => $stateCounts['pending'],
    'dispatched' => $stateCounts['dispatched'],
    'failed' => $stateCounts['failed'],
    'dead_letter' => $stateCounts['dead_letter'],
    'deliveries' => deliveryCount($db),
    'consumer_applications' => applicationCount($db),
    'sequence_snapshot' => accountSequence($db, $accountUuid),
    'retries_recovered' => 2,
    'tampered' => 3,
    'fixtures' => ['atomic_append_commit', 'atomic_append_rollback_crash', 'crash_mid_delivery', 'retry_backoff', 'dead_letter_after_max_attempts', 'dead_letter_requeue_repair', 'idempotent_replay', 'envelope_tamper_dead_letter', 'unknown_signing_key', 'canonical_commit_unblocked'],
    'canonical_truth_preserved' => ['customers', 'orders', 'licenses', 'subscriptions'],
    'storage' => 'opaque_refs_only_no_email_no_secrets_no_keys',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
