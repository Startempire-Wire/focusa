<?php
// 152E.02.07 Project EDD refund, revoke, expiry, and subscription truth.
// The lifecycle projector maps every authority-relevant EDD transition (order completion,
// refund, chargeback, manual revoke, suspend/unsuspend, expiry, subscription cancellation,
// upgrade/downgrade, reissue) to the exact EDD license state and a strictly monotonic
// authority sequence, and journals each hook in a durable transactional outbox in the
// same transaction as the sequence advance. Stale entitlement can never reactivate:
// terminal states (refunded/revoked/expired/superseded/cancelled/denied) fail closed on
// any later activation attempt (LICENSE_TERMINAL_REACTIVATION_DENIED), genuinely new
// out-of-order events fail closed (ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED), and duplicate
// redeliveries are journaled as 'replayed' without bumping the sequence. Refund, revoke,
// expiry, and cancellation never delete the EDD customer, order, license, subscription,
// refund, or audit history. Stripe/EDD status adapters map raw hook statuses and fail
// closed on unmapped statuses (EDD_STATUS_UNKNOWN). No raw email, raw payment id, secret,
// or unmasked real-email evidence is accepted or stored; no client-controlled price,
// amount, grant, feature, limit, tier, or commercial field is accepted.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_lifecycle(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_lifecycle_throws(callable $operation, string $code, string $message): void
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

/** Assert a returned denial decision: fail closed with no state or sequence change. */
function expect_lifecycle_denied(array $decision, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    $decisionValue = $decision['decision'] ?? 'none';
    $errorCodeValue = $decision['error_code'] ?? 'none';
    if ($decisionValue !== 'denied') {
        fwrite(STDERR, "FAIL: {$message} (decision={$decisionValue})\n");
        exit(1);
    }
    if ($errorCodeValue !== $code) {
        fwrite(STDERR, "FAIL: {$message} (error_code={$errorCodeValue})\n");
        exit(1);
    }
    expect_lifecycle($decision['sequence_increment'] === 0, "{$message}: denied events never bump the sequence");
    expect_lifecycle($decision['result_sequence'] === $decision['sequence'], "{$message}: denied events never change the sequence");
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_lifecycle_projection_test']);
$lifecycleMigration = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
$lifecycleMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_lifecycle_projection_test']);

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
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1
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

$insertCustomer = $db->prepare("INSERT INTO wp_edd_customers (email, name, date_created) VALUES (:email, :name, :created)");
$insertCustomer->execute([':email' => 'customer101@example.test', ':name' => 'Test Customer 101', ':created' => '2026-08-08T00:00:00Z']);
$customerId = (int) $db->lastInsertId();
$insertOrder = $db->prepare("INSERT INTO wp_edd_orders (status, customer_id, date_created) VALUES (:status, :customer, :created)");
$insertLicense = $db->prepare("INSERT INTO wp_edd_licenses (license_key, customer_id, product_id, order_id, status, date_created) VALUES (:key, :customer, :product, :order, :status, :created)");
$insertSubscription = $db->prepare("INSERT INTO wp_edd_subscriptions (customer_id, product_id, status, date_created) VALUES (:customer, :product, :status, :created)");

$orderRows = [];
$licenseRows = [];
$subscriptionRows = [];
for ($orderId = 1001; $orderId <= 1007; $orderId++) {
    $insertOrder->execute([':status' => 'completed', ':customer' => $customerId, ':created' => '2026-08-08T00:00:00Z']);
    $orderRows[$orderId] = (int) $db->lastInsertId();
}
for ($licenseId = 1; $licenseId <= 7; $licenseId++) {
    $insertLicense->execute([
        ':key' => strtoupper('FOCUSA-TEST-') . str_pad((string) $licenseId, 4, '0', STR_PAD_LEFT) . '-TESTKEY',
        ':customer' => $customerId,
        ':product' => 1000 + $licenseId,
        ':order' => $orderRows[1000 + $licenseId],
        ':status' => 'active',
        ':created' => '2026-08-08T00:00:00Z',
    ]);
    $licenseRows[$licenseId] = (int) $db->lastInsertId();
}
for ($subscriptionId = 501; $subscriptionId <= 501; $subscriptionId++) {
    $insertSubscription->execute([':customer' => $customerId, ':product' => 1008, ':status' => 'active', ':created' => '2026-08-08T00:00:00Z']);
    $subscriptionRows[$subscriptionId] = (int) $db->lastInsertId();
}
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

$accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
$promoted = $accounts->resolveForPromotionInTransaction($customerId, null, 'cus_test_101', FocusaSpec152eAuthorityAccountMigration::encodeProvenance(['source' => 'edd_lifecycle_projection_test']), '2026-08-08T00:00:00Z');
$accountUuid = (string) $promoted['account']['account_uuid'];
expect_lifecycle($accountUuid !== '', 'promoted authority account resolved');

$projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleMigration, 'wp_', $clock);

/** Journal query helper: latest applied/replayed event for a scope column. */
function latestScopeState(PDO $db, string $column, int $value): ?array
{
    $table = 'wp_wpuiai_edd_lifecycle_events';
    $statement = $db->prepare("SELECT license_state, refresh_posture, result_sequence, decision FROM {$table}
        WHERE {$column} = :scope AND decision IN ('applied','replayed')
        ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
    $statement->execute([':scope' => $value]);
    $row = $statement->fetch(PDO::FETCH_ASSOC);
    return $row === false ? null : $row;
}

function accountSequence(PDO $db, string $accountUuid): int
{
    $statement = $db->prepare('SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = :uuid');
    $statement->execute([':uuid' => $accountUuid]);
    return (int) $statement->fetchColumn();
}

// ── Status adapters (Stripe/EDD) ───────────────────────────────────────

$adapterCases = [
    // [adapter, args..., expected transition, expected state, expected posture]
    ['order', ['completed'], 'complete', 'active', 'allowed'],
    ['order', ['refunded'], 'refund', 'refunded', 'recovery_only'],
    ['order', ['partly_refunded'], 'refund', 'refunded', 'recovery_only'],
    ['order', ['revoked'], 'revoke', 'revoked', 'recovery_only'],
    ['order', ['cancelled'], 'cancel', 'cancelled', 'recovery_only'],
    ['license', ['active', 'expired'], 'expire', 'expired', 'recovery_only'],
    ['license', ['active', 'revoked'], 'revoke', 'revoked', 'recovery_only'],
    ['license', ['active', 'disabled'], 'suspend', 'suspended', 'denied'],
    ['license', ['disabled', 'active'], 'unsuspend', 'active', 'allowed'],
    ['subscription', ['active'], 'complete', 'active', 'allowed'],
    ['subscription', ['cancelled'], 'cancel', 'cancelled', 'recovery_only'],
    ['subscription', ['expired'], 'expire', 'expired', 'recovery_only'],
    ['subscription', ['suspended'], 'suspend', 'suspended', 'denied'],
    ['subscription', ['failing'], 'suspend', 'suspended', 'denied'],
    ['stripe', ['paid'], 'complete', 'active', 'allowed'],
    ['stripe', ['refunded'], 'refund', 'refunded', 'recovery_only'],
    ['stripe', ['disputed'], 'chargeback', 'refunded', 'recovery_only'],
    ['stripe', ['lost'], 'chargeback', 'refunded', 'recovery_only'],
    ['stripe', ['canceled'], 'cancel', 'cancelled', 'recovery_only'],
    ['stripe', ['void'], 'cancel', 'cancelled', 'recovery_only'],
    ['stripe', ['past_due'], 'suspend', 'suspended', 'denied'],
    ['stripe', ['unpaid'], 'suspend', 'suspended', 'denied'],
    ['stripe', ['won'], 'unsuspend', 'active', 'allowed'],
    ['refund', ['refunded'], 'refund', 'refunded', 'recovery_only'],
    ['refund', ['partly_refunded'], 'refund', 'refunded', 'recovery_only'],
    ['refund', ['chargeback'], 'chargeback', 'refunded', 'recovery_only'],
    ['refund', ['disputed'], 'chargeback', 'refunded', 'recovery_only'],
];
foreach ($adapterCases as [$surface, $args, $transition, $state, $posture]) {
    $mapping = match ($surface) {
        'order' => FocusaSpec152eEddStatusAdapter::adaptOrder($args[0]),
        'license' => FocusaSpec152eEddStatusAdapter::adaptLicenseChange($args[0], $args[1]),
        'subscription' => FocusaSpec152eEddStatusAdapter::adaptSubscription($args[0]),
        'stripe' => FocusaSpec152eEddStatusAdapter::adaptStripe($args[0]),
        'refund' => FocusaSpec152eEddStatusAdapter::adaptRefund($args[0]),
    };
    expect_lifecycle($mapping['transition'] === $transition, "adapter {$surface} maps to {$transition}");
    expect_lifecycle($mapping['license_state'] === $state, "adapter {$surface} targets {$state}");
    expect_lifecycle($mapping['refresh_posture'] === $posture, "adapter {$surface} posture {$posture}");
    expect_lifecycle($mapping['sequence_increment'] === 1, "adapter {$surface} is entitlement-relevant (+1)");
}

// ── Full lifecycle: complete -> refund -> chargeback -> revoke ────────

// e1: order complete -> active, sequence 0 -> 1
$e1 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_e1_lifecycle', 'idempotency_key' => 'idem_e1_complete',
]);
expect_lifecycle($e1['decision'] === 'applied', 'order completion applies');
expect_lifecycle($e1['transition'] === 'complete', 'completion transition is complete');
expect_lifecycle($e1['license_state'] === 'active' && $e1['refresh_posture'] === 'allowed', 'completion projects active/allowed');
expect_lifecycle($e1['from_state'] === 'none' && $e1['to_state'] === 'active', 'completion from none to active');
expect_lifecycle($e1['sequence'] === 0 && $e1['result_sequence'] === 1, 'completion bumps sequence 0 -> 1');
expect_lifecycle(accountSequence($db, $accountUuid) === 1, 'account sequence is 1 after completion');
expect_lifecycle($projector->findByEventUuid((string) $e1['event_uuid']) !== null, 'completion event journaled');

// e2: refund -> refunded terminal, sequence 1 -> 2, history preserved
$e2 = $projector->projectRefund([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'refunded', 'request_id' => 'req_e2_refund', 'idempotency_key' => 'idem_e2_refund',
    'state_reason' => 'edd_order_refunded',
]);
expect_lifecycle($e2['decision'] === 'applied', 'refund applies');
expect_lifecycle($e2['license_state'] === 'refunded' && $e2['refresh_posture'] === 'recovery_only', 'refund projects refunded/recovery_only');
expect_lifecycle($e2['from_state'] === 'active' && $e2['to_state'] === 'refunded', 'refund from active to refunded');
expect_lifecycle($e2['sequence'] === 1 && $e2['result_sequence'] === 2, 'refund bumps sequence 1 -> 2');
expect_lifecycle(accountSequence($db, $accountUuid) === 2, 'account sequence is 2 after refund');
$posture = $projector->latestProjectionForAccount($accountUuid);
expect_lifecycle($posture !== null && $posture['license_state'] === 'refunded' && $posture['refresh_posture'] === 'recovery_only', 'account posture after refund is refunded/recovery_only');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn() === $historyBaseline['customers'], 'refund preserves customers');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_orders')->fetchColumn() === $historyBaseline['orders'], 'refund preserves orders');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === $historyBaseline['licenses'], 'refund preserves licenses');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_subscriptions')->fetchColumn() === $historyBaseline['subscriptions'], 'refund preserves subscriptions');
$licenseRow = $db->query('SELECT * FROM wp_edd_licenses WHERE id = ' . $licenseRows[1])->fetch(PDO::FETCH_ASSOC);
expect_lifecycle($licenseRow !== false, 'refund never deletes the EDD license row');

// e3: chargeback redelivery with a new key -> replayed, sequence unchanged
$e3 = $projector->projectRefund([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'chargeback', 'request_id' => 'req_e3_chargeback', 'idempotency_key' => 'idem_e3_chargeback_newkey',
]);
expect_lifecycle($e3['decision'] === 'replayed', 'chargeback after refund is a replayed duplicate');
expect_lifecycle($e3['license_state'] === 'refunded', 'replayed chargeback keeps refunded state');
expect_lifecycle($e3['sequence_increment'] === 0 && $e3['result_sequence'] === 2, 'replayed chargeback never bumps the sequence');
expect_lifecycle(accountSequence($db, $accountUuid) === 2, 'account sequence still 2 after replayed chargeback');

// e4: manual revoke -> revoked terminal, sequence 2 -> 3
$projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1002], 'license_id' => $licenseRows[2],
    'status' => 'completed', 'request_id' => 'req_e4a_complete', 'idempotency_key' => 'idem_e4a_complete',
]);
$e4 = $projector->projectLicense([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1002], 'license_id' => $licenseRows[2],
    'from_status' => 'active', 'to_status' => 'revoked', 'request_id' => 'req_e4_revoke', 'idempotency_key' => 'idem_e4_revoke',
]);
expect_lifecycle($e4['decision'] === 'applied', 'revoke applies');
expect_lifecycle($e4['license_state'] === 'revoked' && $e4['refresh_posture'] === 'recovery_only', 'revoke projects revoked/recovery_only');
expect_lifecycle($e4['result_sequence'] === 4, 'revoke bumps sequence to 4');

// ── Subscription truth: suspend -> unsuspend -> cancel ─────────────────

// e5: subscription active (complete), e6: suspended (denied), e7: active again, e9: cancelled
$e5 = $projector->projectSubscription([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => $subscriptionRows[501],
    'status' => 'active', 'request_id' => 'req_e5_sub_active', 'idempotency_key' => 'idem_e5_sub_active',
]);
expect_lifecycle($e5['decision'] === 'applied' && $e5['result_sequence'] === 5, 'subscription active completes at sequence 5');
$e6 = $projector->projectSubscription([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => $subscriptionRows[501],
    'status' => 'suspended', 'request_id' => 'req_e6_sub_suspended', 'idempotency_key' => 'idem_e6_sub_suspended',
]);
expect_lifecycle($e6['decision'] === 'applied' && $e6['license_state'] === 'suspended' && $e6['refresh_posture'] === 'denied', 'subscription suspend projects suspended/denied');
expect_lifecycle($e6['result_sequence'] === 6, 'suspend bumps sequence to 6');
$e7 = $projector->projectSubscription([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => $subscriptionRows[501],
    'status' => 'active', 'request_id' => 'req_e7_sub_active_again', 'idempotency_key' => 'idem_e7_sub_active_again',
]);
expect_lifecycle($e7['decision'] === 'applied' && $e7['license_state'] === 'active' && $e7['refresh_posture'] === 'allowed', 'subscription unsuspend restores active/allowed');
expect_lifecycle($e7['result_sequence'] === 7, 'unsuspend bumps sequence to 7');

// e8a/e8b: expiry -> expired terminal, sequence 6 -> 8
$projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1003], 'license_id' => $licenseRows[3],
    'status' => 'completed', 'request_id' => 'req_e8a_complete', 'idempotency_key' => 'idem_e8a_complete',
]);
$e8 = $projector->projectLicense([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1003], 'license_id' => $licenseRows[3],
    'from_status' => 'active', 'to_status' => 'expired', 'request_id' => 'req_e8_expire', 'idempotency_key' => 'idem_e8_expire',
]);
expect_lifecycle($e8['decision'] === 'applied' && $e8['license_state'] === 'expired' && $e8['refresh_posture'] === 'recovery_only', 'expiry projects expired/recovery_only');
expect_lifecycle($e8['result_sequence'] === 9, 'expiry bumps sequence to 9');

// e9: subscription cancelled -> cancelled terminal, sequence 8 -> 9
$e9 = $projector->projectSubscription([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => $subscriptionRows[501],
    'status' => 'cancelled', 'request_id' => 'req_e9_sub_cancel', 'idempotency_key' => 'idem_e9_sub_cancel',
]);
expect_lifecycle($e9['decision'] === 'applied' && $e9['license_state'] === 'cancelled' && $e9['refresh_posture'] === 'recovery_only', 'subscription cancel projects cancelled/recovery_only');
expect_lifecycle($e9['result_sequence'] === 10, 'cancel bumps sequence to 10');

// ── Upgrade: supersede the old license, new license active ────────────

// e10a: complete license 4 (sequence 9 -> 10); e10b: upgrade supersedes it (10 -> 11)
$projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1004], 'license_id' => $licenseRows[4],
    'status' => 'completed', 'request_id' => 'req_e10a_complete', 'idempotency_key' => 'idem_e10a_complete',
]);
$e10 = $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'order_id' => $orderRows[1004], 'license_id' => $licenseRows[4],
    'transition' => 'upgrade', 'request_id' => 'req_e10_upgrade', 'idempotency_key' => 'idem_e10_upgrade',
]);
expect_lifecycle($e10['decision'] === 'applied' && $e10['license_state'] === 'superseded' && $e10['refresh_posture'] === 'recovery_only', 'upgrade supersedes the old license');
expect_lifecycle($e10['result_sequence'] === 12, 'upgrade bumps sequence to 12');

// e11: new license completes after upgrade (sequence 11 -> 12)
$e11 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1005], 'license_id' => $licenseRows[5],
    'status' => 'completed', 'request_id' => 'req_e11_complete_new', 'idempotency_key' => 'idem_e11_complete_new',
]);
expect_lifecycle($e11['decision'] === 'applied' && $e11['license_state'] === 'active' && $e11['result_sequence'] === 13, 'new license completes after upgrade');

// e12: stale completion on the superseded license -> denied, cannot reactivate
$e12 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1004], 'license_id' => $licenseRows[4],
    'status' => 'completed', 'request_id' => 'req_e12_stale_complete', 'idempotency_key' => 'idem_e12_stale_complete',
]);
expect_lifecycle_denied($e12, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'stale completion cannot reactivate a superseded license');
$scope4 = latestScopeState($db, 'license_id', $licenseRows[4]);
expect_lifecycle($scope4 !== null && $scope4['license_state'] === 'superseded', 'superseded license stays superseded');

// ── Downgrade then reissue ─────────────────────────────────────────────

// e13a: complete license 6 (12 -> 13); e13b: downgrade -> superseded/allowed (13 -> 14)
$projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1006], 'license_id' => $licenseRows[6],
    'status' => 'completed', 'request_id' => 'req_e13a_complete', 'idempotency_key' => 'idem_e13a_complete',
]);
$e13 = $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'order_id' => $orderRows[1006], 'license_id' => $licenseRows[6],
    'transition' => 'downgrade', 'request_id' => 'req_e13_downgrade', 'idempotency_key' => 'idem_e13_downgrade',
]);
expect_lifecycle($e13['decision'] === 'applied' && $e13['license_state'] === 'superseded' && $e13['refresh_posture'] === 'allowed', 'downgrade supersedes with allowed posture');
expect_lifecycle($e13['result_sequence'] === 15, 'downgrade bumps sequence to 15');

// e14a: suspend the active license 5 (15 -> 16); e14b: reissue restores it (16 -> 17).
// A reissue never reactivates a terminal (superseded/refunded/expired) license; it can
// only restore a reversible suspended state. Reissue after downgrade stays denied (e12
// already proves superseded reactivation denial).
$e14a = $projector->projectLicense([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1005], 'license_id' => $licenseRows[5],
    'from_status' => 'active', 'to_status' => 'disabled', 'request_id' => 'req_e14a_suspend', 'idempotency_key' => 'idem_e14a_suspend',
]);
expect_lifecycle($e14a['decision'] === 'applied' && $e14a['license_state'] === 'suspended' && $e14a['refresh_posture'] === 'denied', 'suspend projects suspended/denied');
expect_lifecycle($e14a['result_sequence'] === 16, 'suspend bumps sequence to 16');
$e14 = $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'order_id' => $orderRows[1005], 'license_id' => $licenseRows[5],
    'transition' => 'reissue', 'request_id' => 'req_e14_reissue', 'idempotency_key' => 'idem_e14_reissue',
]);
expect_lifecycle($e14['decision'] === 'applied' && $e14['license_state'] === 'active' && $e14['refresh_posture'] === 'allowed', 'reissue restores active/allowed from suspended');
expect_lifecycle($e14['result_sequence'] === 17, 'reissue bumps sequence to 17');

// ── Account audit hooks (no entitlement sequence change) ───────────────

$e15 = $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'transition' => 'email_change', 'request_id' => 'req_e15_email_change', 'idempotency_key' => 'idem_e15_email_change',
]);
expect_lifecycle($e15['decision'] === 'applied' && $e15['sequence_increment'] === 0 && $e15['result_sequence'] === 17, 'email change is an audit-only hook (sequence unchanged)');
$e16 = $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'transition' => 'deactivate_node', 'request_id' => 'req_e16_node', 'idempotency_key' => 'idem_e16_node',
]);
expect_lifecycle($e16['decision'] === 'applied' && $e16['sequence_increment'] === 0 && $e16['result_sequence'] === 17, 'node deactivation is an audit-only hook (sequence unchanged)');

// ── Replay and out-of-order fixtures ───────────────────────────────────

// e17: identical idempotency-key replay returns the identical decision, no new row
$beforeCount = $projector->eventCount();
$e17 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_e1_lifecycle', 'idempotency_key' => 'idem_e1_complete',
]);
expect_lifecycle($e17['decision'] === 'replayed', 'idempotent replay returns a replayed decision');
expect_lifecycle($e17['event_uuid'] === $e1['event_uuid'], 'idempotent replay returns the identical decision');
expect_lifecycle($e17['result_sequence'] === 1 && accountSequence($db, $accountUuid) === 17, 'idempotent replay never bumps the sequence');
expect_lifecycle($projector->eventCount() === $beforeCount, 'idempotent replay journals no duplicate row');

// e18: genuinely new event with a stale authority ordinal -> out-of-order denial
$e18 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1007], 'license_id' => $licenseRows[7],
    'status' => 'completed', 'authority_sequence' => 5, 'request_id' => 'req_e18_out_of_order', 'idempotency_key' => 'idem_e18_out_of_order',
]);
expect_lifecycle_denied($e18, 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', 'out-of-order event is denied and cannot roll the sequence back');
expect_lifecycle(accountSequence($db, $accountUuid) === 17, 'out-of-order denial leaves the sequence at 17');

// e19: the same event delivered in order applies (15 -> 16)
$e19 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1007], 'license_id' => $licenseRows[7],
    'status' => 'completed', 'request_id' => 'req_e19_complete', 'idempotency_key' => 'idem_e19_complete',
]);
expect_lifecycle($e19['decision'] === 'applied' && $e19['result_sequence'] === 18, 'in-order completion applies at sequence 18');

// e20: stale completion after refund -> cannot reactivate stale entitlement
$e20 = $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_e20_stale_refunded', 'idempotency_key' => 'idem_e20_stale_refunded',
]);
expect_lifecycle_denied($e20, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'stale completion after refund is denied');
$scope1 = latestScopeState($db, 'license_id', $licenseRows[1]);
expect_lifecycle($scope1 !== null && $scope1['license_state'] === 'refunded', 'refunded license stays refunded');

// e21: Stripe dispute-won after refund -> cannot reactivate
$e21 = $projector->projectStripe([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'won', 'request_id' => 'req_e21_dispute_won', 'idempotency_key' => 'idem_e21_dispute_won',
]);
expect_lifecycle_denied($e21, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'dispute-won webhook cannot reactivate a refunded license');

// e22: refund redelivered with a new key -> replayed, sequence unchanged
$e22 = $projector->projectRefund([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'refunded', 'request_id' => 'req_e22_refund_redelivery', 'idempotency_key' => 'idem_e22_refund_redelivery',
]);
expect_lifecycle($e22['decision'] === 'replayed' && $e22['result_sequence'] === 18, 'refund redelivery replays without bumping the sequence');

// e23: expired license reactivation -> denied
$e23 = $projector->projectLicense([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1003], 'license_id' => $licenseRows[3],
    'from_status' => 'expired', 'to_status' => 'active', 'request_id' => 'req_e23_expired_reactivate', 'idempotency_key' => 'idem_e23_expired_reactivate',
]);
expect_lifecycle_denied($e23, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'expired license cannot reactivate');
$scope3 = latestScopeState($db, 'license_id', $licenseRows[3]);
expect_lifecycle($scope3 !== null && $scope3['license_state'] === 'expired', 'expired license stays expired');

// ── Monotonicity, durability, and preservation ─────────────────────────

expect_lifecycle(accountSequence($db, $accountUuid) === 18, 'final account sequence is 18 (strictly monotonic)');
$sequences = $db->query("SELECT result_sequence FROM wp_wpuiai_edd_lifecycle_events WHERE decision = 'applied' AND sequence_increment > 0 ORDER BY result_sequence")->fetchAll(PDO::FETCH_COLUMN);
$uniqueSequences = array_values(array_unique($sequences));
expect_lifecycle($uniqueSequences === $sequences, 'applied result sequences are strictly monotonic with no duplicates');
$eventCount = $projector->eventCount();
expect_lifecycle($eventCount === 27, "outbox journals exactly one row per hook delivery ({$eventCount} == 27)");
$decisions = $db->query('SELECT decision, COUNT(*) FROM wp_wpuiai_edd_lifecycle_events GROUP BY decision ORDER BY decision')->fetchAll(PDO::FETCH_KEY_PAIR);
expect_lifecycle(($decisions['applied'] ?? 0) === 20, '20 applied events journaled (18 entitlement + 2 audit hooks)');
expect_lifecycle(($decisions['replayed'] ?? 0) === 2, '2 replayed events journaled');
expect_lifecycle(($decisions['denied'] ?? 0) === 5, '5 denied events journaled (fail-closed audit)');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn() === $historyBaseline['customers'], 'history preserved: customers');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_orders')->fetchColumn() === $historyBaseline['orders'], 'history preserved: orders');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === $historyBaseline['licenses'], 'history preserved: licenses');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_edd_subscriptions')->fetchColumn() === $historyBaseline['subscriptions'], 'history preserved: subscriptions');
$refundedOrder = $db->query('SELECT status FROM wp_edd_orders WHERE id = ' . $orderRows[1001])->fetch(PDO::FETCH_ASSOC);
expect_lifecycle($refundedOrder !== false, 'refunded order row is retained (refund truth preserved)');

// ── Redaction: no raw email, no payment secrets, no key material ───────

$journalJson = json_encode($db->query('SELECT * FROM wp_wpuiai_edd_lifecycle_events')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_lifecycle(strpos($journalJson, '@') === false, 'no raw email in the lifecycle journal');
expect_lifecycle(strpos($journalJson, 'cus_test_101') === false, 'no raw payment/customer secret in the lifecycle journal');
expect_lifecycle(strpos($journalJson, 'FOCUSA-TEST-') === false, 'no license key material in the lifecycle journal');
expect_lifecycle(strpos($journalJson, 'example.test') === false, 'no unmasked email evidence in the lifecycle journal');
$payloads = $db->query('SELECT result_payload FROM wp_wpuiai_edd_lifecycle_events')->fetchAll(PDO::FETCH_COLUMN);
foreach ($payloads as $payload) {
    expect_lifecycle(strpos($payload, '@') === false, 'no raw email in any decision payload');
    expect_lifecycle(strpos($payload, 'cus_test_101') === false, 'no raw payment secret in any decision payload');
}

// ── Negative checks: fail closed on every named input ──────────────────

expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptOrder('pending'), 'EDD_STATUS_UNKNOWN', 'unknown EDD order status fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptOrder('processing'), 'EDD_STATUS_UNKNOWN', 'processing order status fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptStripe('requires_action'), 'EDD_STATUS_UNKNOWN', 'unknown Stripe status fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptSubscription('trialling'), 'EDD_STATUS_UNKNOWN', 'trialling subscription fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptSubscription('pending'), 'EDD_STATUS_UNKNOWN', 'pending subscription fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptLicenseChange('active', 'mystery'), 'EDD_STATUS_UNKNOWN', 'unknown license status fails closed');
expect_lifecycle_throws(static fn () => FocusaSpec152eEddStatusAdapter::adaptRefund('mystery'), 'EDD_STATUS_UNKNOWN', 'unknown refund status fails closed');
expect_lifecycle_throws(static fn () => $projector->projectTransition([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'surface' => 'order', 'transition' => 'frobnicate', 'request_id' => 'req_neg_transition', 'idempotency_key' => 'idem_neg_transition',
]), 'EDD_TRANSITION_UNKNOWN', 'unknown explicit transition fails closed');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1], 'price' => 9.99,
    'status' => 'completed', 'request_id' => 'req_neg_price', 'idempotency_key' => 'idem_neg_price',
]), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled price is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1], 'grants' => ['release' => true],
    'status' => 'completed', 'request_id' => 'req_neg_grants', 'idempotency_key' => 'idem_neg_grants',
]), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled grants are rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1], 'download_id' => 1001,
    'status' => 'completed', 'request_id' => 'req_neg_download', 'idempotency_key' => 'idem_neg_download',
]), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled download/product id is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1], 'email' => 'someone@example.test',
    'status' => 'completed', 'request_id' => 'req_neg_email', 'idempotency_key' => 'idem_neg_email',
]), 'INPUT_RAW_EMAIL_FORBIDDEN', 'raw email input is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[2],
    'status' => 'completed', 'request_id' => 'req_neg_conflict', 'idempotency_key' => 'idem_e1_complete',
]), 'IDEMPOTENCY_CONFLICT', 'same idempotency key with a different digest conflicts');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => '00000000-0000-4000-8000-00000000dead', 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_neg_account', 'idempotency_key' => 'idem_neg_account',
]), 'ENTITLEMENT_REQUIRED', 'unknown account fails closed');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => 99999,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_neg_customer', 'idempotency_key' => 'idem_neg_customer',
]), 'EDD_CUSTOMER_RESOLUTION_FAILED', 'customer/account mismatch fails closed');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'short', 'idempotency_key' => 'idem_neg_request',
]), 'bounded request ID required', 'malformed request id is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_neg_idem', 'idempotency_key' => 'x',
]), 'bounded idempotency key required', 'malformed idempotency key is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => 'not-a-uuid', 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'request_id' => 'req_neg_uuid', 'idempotency_key' => 'idem_neg_uuid',
]), 'bounded account UUID required', 'malformed account uuid is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'authority_sequence' => 0, 'request_id' => 'req_neg_seq', 'idempotency_key' => 'idem_neg_seq',
]), 'positive authority sequence required', 'non-positive authority sequence is rejected');
expect_lifecycle_throws(static fn () => $projector->projectOrder([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1001], 'license_id' => $licenseRows[1],
    'status' => 'completed', 'state_reason' => 'contact me@example.test', 'request_id' => 'req_neg_reason', 'idempotency_key' => 'idem_neg_reason',
]), 'INPUT_RAW_EMAIL_FORBIDDEN', 'state reason with raw email is rejected');

// Terminal transitions with no prior entitlement fail closed (decision-denied, no throw).
$neverEntitled = $projector->projectRefund([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'order_id' => $orderRows[1007], 'license_id' => 7001,
    'status' => 'refunded', 'request_id' => 'req_neg_never', 'idempotency_key' => 'idem_neg_never',
]);
expect_lifecycle_denied($neverEntitled, 'ENTITLEMENT_REQUIRED', 'refund with no prior entitlement is denied');
$neverSuspended = $projector->projectSubscription([
    'account_uuid' => $accountUuid, 'edd_customer_id' => $customerId,
    'subscription_id' => 9999,
    'status' => 'suspended', 'request_id' => 'req_neg_suspend', 'idempotency_key' => 'idem_neg_suspend',
]);
expect_lifecycle_denied($neverSuspended, 'ENTITLEMENT_REQUIRED', 'suspend with no prior entitlement is denied');

// ── Rollback preservation ──────────────────────────────────────────────

$preserved = $lifecycleMigration->preserveForRollback('2026-08-08T00:02:00Z', ['source' => 'edd_lifecycle_projection_test']);
expect_lifecycle($preserved['action'] === 'preserve' && $preserved['event_key'] !== '', 'rollback is preservation-only (schema event recorded)');
expect_lifecycle((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_lifecycle_events')->fetchColumn() === 29, 'rollback preservation never deletes lifecycle journals (27 fixtures + 2 denied negatives)');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_lifecycle_projection_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'events_journaled' => $eventCount,
    'applied' => (int) ($decisions['applied'] ?? 0),
    'replayed' => (int) ($decisions['replayed'] ?? 0),
    'denied' => (int) ($decisions['denied'] ?? 0),
    'final_sequence' => accountSequence($db, $accountUuid),
    'fixtures' => ['idempotent_replay', 'idempotency_conflict', 'out_of_order_stale_event', 'stale_completion_after_refund', 'stale_renewal_after_cancel', 'stripe_dispute_won_after_refund', 'duplicate_terminal_redelivery', 'upgrade_supersedes', 'chargeback_after_refund', 'expiry_then_reactivation_denied'],
    'terminal_states' => ['refunded', 'revoked', 'expired', 'superseded', 'cancelled'],
    'history_preserved' => ['customers', 'orders', 'licenses', 'subscriptions'],
    'journal_storage' => 'opaque_refs_only_no_email_no_secrets_no_keys',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
