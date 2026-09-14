<?php
// 152E.02.04 Bind EDD order completion idempotently to registration.
// The edd_complete_purchase / order-status completion surface accepts only complete,
// eligible orders with the exact account, registration, order item, price, and product
// binding, plus a real order-linked payment transaction; each eligible protected order
// item settles exactly one entitlement issuance request (journaled, deferred to the
// verified issuance service — no license, key, or lease is ever created here).
// Synthetic or unlinked payment IDs (focusa_live_*, synthetic_*, manual/none markers,
// transactions not bound to the exact canonical order) fail closed with
// EDD_ORDER_UNVERIFIED and never issue. Duplicate completion events return the existing
// settlement (existing=true) and never create a second issuance request; idempotency
// replays return the same decision; out-of-order events (canonical order status
// pending/refunded, or a terminal refund/revoke journaled before completion) fail
// closed with zero issuance. Journals store only keyed digests and opaque bounded
// tokens: no raw email, raw payment id, secret, or unmasked real-email evidence.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
require_once $root . '/docs/contracts/spec152e-facade-registry.v1.php';
require_once $root . '/docs/contracts/spec152e-verified-registration-token-validator.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-order-binding.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_binding(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_binding_throws(callable $operation, string $code, string $message): void
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

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_order_binding_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_order_binding_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_order_binding_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_order_binding_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_order_binding_test']);

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
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active'
)");

$nowValue = '2026-08-08T00:01:00Z';
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};

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

// The frozen registry is used by the fail-closed service instance; the fixture registry
// adds an explicitly operator-approved test mapping (download 1001 ->
// focusa_operator_lifetime_v1 active/checkout_enabled, price price_focusa_op_v1) and an
// approved-but-blocked mapping (download 1002 -> uiai_operator_lifetime_v1,
// checkout_enabled false) so positive and blocked paths are exercised without mutating
// the frozen contract.
$frozenRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

$fixtureRegistry = $frozenRegistry;
foreach ($fixtureRegistry['protected_offers'] as &$offer) {
    if ($offer['public_code'] === 'focusa_operator_lifetime_v1') {
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
        $offer['edd_download_id'] = 1001;
        $offer['edd_price_id'] = 'price_focusa_op_v1';
    }
    if ($offer['public_code'] === 'uiai_operator_lifetime_v1') {
        $offer['edd_download_id'] = 1002;
        $offer['edd_price_id'] = 'price_uiai_op_v1';
    }
}
unset($offer);

$bindingFrozen = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $frozenRegistry, $facadeRegistry, $clock,
);
$bindingService = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $fixtureRegistry, $facadeRegistry, $clock,
);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.edd.order.binding.test',
        'install_channel' => 'cli',
        'product_code' => $product,
        'safe_redirect_handle' => 'success',
        'request_id' => 'req-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-' . $tag . '-' . $seq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    if (!$verify) {
        return ['registration_uuid' => $uuid, 'verification_secret' => $created['verification_secret']];
    }
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $seq,
        'idem-verify-' . $tag . '-' . $seq,
    );
    if (!$promote) {
        return ['registration_uuid' => $uuid, 'verified_at' => $verified['registration']['verified_at']];
    }
    $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'order-binding-' . $tag . '-' . $seq],
    ]);
    return ['registration_uuid' => $uuid];
};

$customerOf = static function (string $registrationUuid) use ($registrations): int {
    return (int) $registrations->findByUuid($registrationUuid)['edd_customer_id'];
};

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, array $items = []) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_number, status, type, date_created, date_completed, user_id, customer_id, email)
        VALUES (:id, :number, :status, 'sale', '2026-08-08T00:01:00Z', :completed, NULL, :customer, :email)");
    $statement->execute([
        ':id' => $orderId,
        ':number' => 'EDD-' . $orderId,
        ':status' => $status,
        ':completed' => $status === 'complete' ? '2026-08-08T00:01:00Z' : null,
        ':customer' => $customerId,
        ':email' => $email,
    ]);
    foreach ($items as $item) {
        $rowSeq++;
        $itemStatement = $db->prepare("INSERT INTO wp_edd_order_items
            (id, order_id, product_id, product_name, quantity)
            VALUES (:id, :order, :product, 'fixture', :quantity)");
        $itemStatement->execute([
            ':id' => (int) ($item['item_id'] ?? (100000 + $rowSeq)),
            ':order' => $orderId,
            ':product' => (int) $item['download'],
            ':quantity' => (int) ($item['qty'] ?? 1),
        ]);
    }
};

$txnSeq = 0;
$insertTransaction = static function (int $orderId, string $gateway, string $transactionId, string $status = 'complete', string $total = '697.00') use ($db, &$txnSeq): void {
    $txnSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_transactions
        (id, order_id, transaction_id, gateway, status, total, currency, date_created)
        VALUES (:id, :order, :txn, :gateway, :status, :total, 'USD', '2026-08-08T00:01:00Z')");
    $statement->execute([
        ':id' => $txnSeq,
        ':order' => $orderId,
        ':txn' => $transactionId,
        ':gateway' => $gateway,
        ':status' => $status,
        ':total' => $total,
    ]);
};

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$PRODUCT = 'focusa_operator_lifetime_v1';
$UIAPRODUCT = 'uiai_operator_lifetime_v1';
$DOWNLOAD = 1001;
$PRICE = 'price_focusa_op_v1';
$GATEWAY = 'stripe';
$TXN = 'txn_pay_0001';

$bind = static function (array $overrides = []) use ($bindingService, $FACADE, $ORIGIN, $DOWNLOAD, $PRICE, $GATEWAY, $TXN): array {
    return $bindingService->bindOrderComplete(array_merge([
        'order_id' => 2001,
        'order_status' => 'complete',
        'customer_id' => 1,
        'order_items' => [['order_item_id' => 2001, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => $TXN, 'status' => 'complete']],
        'registration_uuid' => '',
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'request_id' => 'req-bind-2001',
        'idempotency_key' => 'idem-bind-2001',
    ], $overrides));
};

$bindingCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_order_bindings')->fetchColumn();
};
$requestCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_issuance_requests')->fetchColumn();
};
$licenseCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

// ── Frozen registry invariants (generated contracts remain current) ────

expect_binding($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema');
expect_binding($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_binding($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');

// ── Positive: one eligible order item settles one issuance request ─────

$regA = $createRegistration('bind.alpha@example.invalid', $FACADE, $PRODUCT, 'alpha');
$customerA = $customerOf($regA['registration_uuid']);
$insertOrder(2001, 'complete', $customerA, 'bind.alpha@example.invalid', [
    ['item_id' => 2001, 'download' => $DOWNLOAD],
]);
$insertTransaction(2001, $GATEWAY, $TXN);

$bound = $bind([
    'order_id' => 2001,
    'customer_id' => $customerA,
    'registration_uuid' => $regA['registration_uuid'],
    'request_id' => 'req-bind-alpha-1',
    'idempotency_key' => 'idem-bind-alpha-1',
]);
expect_binding($bound['decision'] === 'order_bound', 'eligible complete order settles as order_bound');
expect_binding($bound['entitlement_allowed'] === true, 'settled order is entitlement-allowed');
expect_binding($bound['issuance_requests_settled'] === 1, 'exactly one issuance request settled for one item');
expect_binding($bound['payment_bound'] === true, 'payment is bound to the canonical order');
expect_binding($bound['existing'] === false, 'first settlement is not an existing settlement');
expect_binding(count($bound['protected_items']) === 1, 'one protected item entry in the decision');
$itemEntry = $bound['protected_items'][0];
expect_binding(preg_match('/^(ob_)[0-9a-f]{32}$/D', (string) $itemEntry['binding_key']) === 1, 'binding keys are opaque bounded tokens');
expect_binding(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $itemEntry['issuance_request_handle']) === 1, 'issuance request handles are opaque bounded tokens');
expect_binding($itemEntry['product_code'] === $PRODUCT, 'bound product is the server-owned offer');
expect_binding($itemEntry['price_id'] === $PRICE, 'bound price is the server-owned offer price');
expect_binding($bindingCount() === 1, 'exactly one binding journal row');
expect_binding($requestCount() === 1, 'exactly one issuance request row');
expect_binding($licenseCount() === 0, 'binding settles zero EDD licenses (issuance deferred)');
expect_binding($bindingService->bindingCount() === 1, 'service binding count matches journal');
expect_binding($bindingService->issuanceRequestCount() === 1, 'service issuance-request count matches journal');

$requestRow = $bindingService->findIssuanceRequestByKey($itemEntry['issuance_request_handle']);
expect_binding($requestRow !== null && $requestRow['state'] === 'pending', 'issuance request is pending and deferred');
expect_binding($bindingService->findByBindingKey($itemEntry['binding_key'])['binding_state'] === 'settled_pending_issuance', 'binding row is settled_pending_issuance');

// The frozen registry assigns no downloads and no checkout-enabled offers, so a
// protected order item can never resolve through it: fail closed with
// PRODUCT_MAPPING_REQUIRED before any registration/payment authority is consulted.
expect_binding_throws(
    fn() => $bindingFrozen->bindOrderComplete([
        'order_id' => 2001, 'order_status' => 'complete', 'customer_id' => $customerA,
        'order_items' => [['order_item_id' => 2001, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => $TXN, 'status' => 'complete']],
        'registration_uuid' => '', 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-frozen-1', 'idempotency_key' => 'idem-bind-frozen-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'frozen registry (no operator-approved download mapping) denies settlement',
);
expect_binding($requestCount() === 1, 'frozen registry settlement attempt creates no issuance request');

// Idempotency replay: same key returns the same decision, nothing new.
$replayed = $bind([
    'order_id' => 2001,
    'customer_id' => $customerA,
    'registration_uuid' => $regA['registration_uuid'],
    'request_id' => 'req-bind-alpha-1',
    'idempotency_key' => 'idem-bind-alpha-1',
]);
expect_binding(json_encode($replayed, JSON_THROW_ON_ERROR) === json_encode($bound, JSON_THROW_ON_ERROR), 'idempotency replay returns the identical decision');
expect_binding($requestCount() === 1, 'replay creates no second issuance request');

// Duplicate completion event (new idempotency key): existing settlement, nothing new.
$duplicate = $bind([
    'order_id' => 2001,
    'customer_id' => $customerA,
    'registration_uuid' => $regA['registration_uuid'],
    'request_id' => 'req-bind-alpha-dupe-1',
    'idempotency_key' => 'idem-bind-alpha-dupe-1',
]);
expect_binding($duplicate['decision'] === 'order_bound' && $duplicate['existing'] === true, 'duplicate completion event returns the existing settlement');
expect_binding($duplicate['issuance_requests_settled'] === 0, 'duplicate event settles nothing');
expect_binding($duplicate['protected_items'][0]['issuance_request_handle'] === $itemEntry['issuance_request_handle'], 'duplicate event returns the same issuance request handle');
expect_binding($requestCount() === 1, 'duplicate completion event creates no second issuance request');

// Out-of-order: canonical order status is authoritative. A 'complete' event for an
// order whose canonical row is still pending fails closed with zero issuance...
$regPending = $createRegistration('bind.pending@example.invalid', $FACADE, $PRODUCT, 'pending');
$customerPending = $customerOf($regPending['registration_uuid']);
$insertOrder(2002, 'pending', $customerPending, 'bind.pending@example.invalid', [
    ['item_id' => 2002, 'download' => $DOWNLOAD],
]);
$insertTransaction(2002, $GATEWAY, 'txn_pay_0002');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2002,
        'customer_id' => $customerPending,
        'registration_uuid' => $regPending['registration_uuid'],
        'request_id' => 'req-bind-pending-1',
        'idempotency_key' => 'idem-bind-pending-1',
    ]),
    'EDD_ORDER_PENDING',
    'complete event for a canonical pending order fails closed',
);
expect_binding($requestCount() === 1, 'out-of-order pending completion issues nothing');

// ...and once the canonical row is complete the same order settles.
$db->exec("UPDATE wp_edd_orders SET status = 'complete', date_completed = '2026-08-08T00:02:00Z' WHERE id = 2002");
$pendingRecovered = $bind([
    'order_id' => 2002,
    'customer_id' => $customerPending,
    'order_items' => [['order_item_id' => 2002, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0002', 'status' => 'complete']],
    'registration_uuid' => $regPending['registration_uuid'],
    'request_id' => 'req-bind-pending-2',
    'idempotency_key' => 'idem-bind-pending-2',
]);
expect_binding($pendingRecovered['decision'] === 'order_bound' && $pendingRecovered['issuance_requests_settled'] === 1, 'canonical completion settles after the row is complete');
expect_binding($requestCount() === 2, 'canonical recovery settles exactly one more issuance request');

// A complete event for a canonical refunded order never issues.
$regRefunded = $createRegistration('bind.refunded@example.invalid', $FACADE, $PRODUCT, 'refunded');
$customerRefunded = $customerOf($regRefunded['registration_uuid']);
$insertOrder(2003, 'refunded', $customerRefunded, 'bind.refunded@example.invalid', [
    ['item_id' => 2003, 'download' => $DOWNLOAD],
]);
$insertTransaction(2003, $GATEWAY, 'txn_pay_0003');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2003,
        'customer_id' => $customerRefunded,
        'registration_uuid' => $regRefunded['registration_uuid'],
        'request_id' => 'req-bind-refunded-1',
        'idempotency_key' => 'idem-bind-refunded-1',
    ]),
    'REFUNDED',
    'complete event for a canonical refunded order is denied',
);
expect_binding($requestCount() === 2, 'canonical refunded order issues nothing');
expect_binding($bindingCount() === 3, 'canonical refunded order journals a durable blocked binding');

// ── Out-of-order: terminal refund event journaled before completion ────

$regBlocked = $createRegistration('bind.blocked@example.invalid', $FACADE, $PRODUCT, 'blocked');
$customerBlocked = $customerOf($regBlocked['registration_uuid']);
$insertOrder(2004, 'complete', $customerBlocked, 'bind.blocked@example.invalid', [
    ['item_id' => 2004, 'download' => $DOWNLOAD],
]);
$insertTransaction(2004, $GATEWAY, 'txn_pay_0004');

// Refunded event arrives first (out-of-order): journals a durable blocked binding,
// fails closed, and issues nothing.
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2004,
        'order_status' => 'refunded',
        'customer_id' => $customerBlocked,
        'registration_uuid' => $regBlocked['registration_uuid'],
        'request_id' => 'req-bind-blocked-refund-1',
        'idempotency_key' => 'idem-bind-blocked-refund-1',
    ]),
    'REFUNDED',
    'out-of-order refunded event fails closed',
);
expect_binding($requestCount() === 2, 'out-of-order refunded event issues nothing');
expect_binding($bindingCount() === 4, 'out-of-order refunded event journals a blocked binding');

// A later complete event can never over-settle the journaled terminal block.
$blockedComplete = $bind([
    'order_id' => 2004,
    'customer_id' => $customerBlocked,
    'order_items' => [['order_item_id' => 2004, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0004', 'status' => 'complete']],
    'registration_uuid' => $regBlocked['registration_uuid'],
    'request_id' => 'req-bind-blocked-complete-1',
    'idempotency_key' => 'idem-bind-blocked-complete-1',
]);
expect_binding($blockedComplete['decision'] === 'out_of_order', 'complete event after a journaled terminal block is out_of_order');
expect_binding($blockedComplete['blocked_reason'] === 'REFUNDED', 'out_of_order decision carries the terminal reason');
expect_binding($blockedComplete['issuance_requests_settled'] === 0, 'out_of_order complete event settles nothing');
expect_binding($blockedComplete['entitlement_allowed'] === false, 'out_of_order complete event is not entitlement-allowed');
expect_binding($requestCount() === 2, 'out_of_order complete event creates no issuance request');

// Repeating the refunded event stays deterministic: one blocked binding, still nothing issued.
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2004,
        'order_status' => 'refunded',
        'customer_id' => $customerBlocked,
        'registration_uuid' => $regBlocked['registration_uuid'],
        'request_id' => 'req-bind-blocked-refund-2',
        'idempotency_key' => 'idem-bind-blocked-refund-2',
    ]),
    'REFUNDED',
    'repeated refunded event stays denied',
);
expect_binding($bindingCount() === 4, 'repeated refunded event journals no second blocked binding');

// ── Positive: two eligible items settle two issuance requests, one each ─

$regMulti = $createRegistration('bind.multi@example.invalid', $FACADE, $PRODUCT, 'multi');
$customerMulti = $customerOf($regMulti['registration_uuid']);
$insertOrder(2005, 'complete', $customerMulti, 'bind.multi@example.invalid', [
    ['item_id' => 2005, 'download' => $DOWNLOAD],
    ['item_id' => 2006, 'download' => $DOWNLOAD],
]);
$insertTransaction(2005, $GATEWAY, 'txn_pay_0005');
$multi = $bind([
    'order_id' => 2005,
    'customer_id' => $customerMulti,
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0005', 'status' => 'complete']],
    'order_items' => [
        ['order_item_id' => 2005, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1],
        ['order_item_id' => 2006, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1],
    ],
    'registration_uuid' => $regMulti['registration_uuid'],
    'request_id' => 'req-bind-multi-1',
    'idempotency_key' => 'idem-bind-multi-1',
]);
expect_binding($multi['issuance_requests_settled'] === 2, 'two eligible items settle two issuance requests');
expect_binding(count($multi['protected_items']) === 2, 'two protected item entries returned');
expect_binding($requestCount() === 4, 'two items create exactly two more issuance request rows');

// ── Mixed order: protected item + unrelated item; unrelated never entitles ─

$regMixed = $createRegistration('bind.mixed@example.invalid', $FACADE, $PRODUCT, 'mixed');
$customerMixed = $customerOf($regMixed['registration_uuid']);
$insertOrder(2006, 'complete', $customerMixed, 'bind.mixed@example.invalid', [
    ['item_id' => 2007, 'download' => $DOWNLOAD],
    ['item_id' => 2008, 'download' => 16],
]);
$insertTransaction(2006, $GATEWAY, 'txn_pay_0006');
$mixed = $bind([
    'order_id' => 2006,
    'customer_id' => $customerMixed,
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0006', 'status' => 'complete']],
    'order_items' => [
        ['order_item_id' => 2007, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1],
        ['order_item_id' => 2008, 'download_id' => 16, 'price_id' => 'price_unrelated', 'quantity' => 1],
    ],
    'registration_uuid' => $regMixed['registration_uuid'],
    'request_id' => 'req-bind-mixed-1',
    'idempotency_key' => 'idem-bind-mixed-1',
]);
expect_binding($mixed['decision'] === 'order_bound', 'mixed order settles as order_bound');
expect_binding(count($mixed['excluded_items']) === 1 && $mixed['excluded_items'][0]['disposition'] === 'non_entitlement', 'unrelated item is excluded as non-entitlement');
expect_binding($mixed['issuance_requests_settled'] === 1, 'only the protected item settles an issuance request');
expect_binding($requestCount() === 5, 'mixed order creates exactly one more issuance request');

// ── Unrelated / credit-pack orders: no entitlement, no identity requirement ─

$regUnrelated = $createRegistration('bind.unrelated@example.invalid', $FACADE, $PRODUCT, 'unrelated');
$customerUnrelated = $customerOf($regUnrelated['registration_uuid']);
$insertOrder(2007, 'complete', $customerUnrelated, 'bind.unrelated@example.invalid', [
    ['item_id' => 2009, 'download' => 16],
]);
$unrelated = $bind([
    'order_id' => 2007,
    'customer_id' => $customerUnrelated,
    'order_items' => [['order_item_id' => 2009, 'download_id' => 16, 'price_id' => 'price_unrelated', 'quantity' => 1]],
    'payment_transactions' => [],
    'registration_uuid' => '',
    'request_id' => 'req-bind-unrelated-1',
    'idempotency_key' => 'idem-bind-unrelated-1',
]);
expect_binding($unrelated['decision'] === 'no_entitlement', 'unrelated order is non-entitlement');
expect_binding($unrelated['protected_items'] === 0, 'unrelated order carries zero protected items');
expect_binding($requestCount() === 5, 'unrelated order creates no issuance request');

$regCredit = $createRegistration('bind.credit@example.invalid', $FACADE, $PRODUCT, 'credit');
$customerCredit = $customerOf($regCredit['registration_uuid']);
$insertOrder(2008, 'complete', $customerCredit, 'bind.credit@example.invalid', [
    ['item_id' => 2010, 'download' => 455],
]);
$credit = $bind([
    'order_id' => 2008,
    'customer_id' => $customerCredit,
    'order_items' => [['order_item_id' => 2010, 'download_id' => 455, 'price_id' => 'price_credit', 'quantity' => 1]],
    'payment_transactions' => [],
    'registration_uuid' => '',
    'request_id' => 'req-bind-credit-1',
    'idempotency_key' => 'idem-bind-credit-1',
]);
expect_binding($credit['decision'] === 'no_entitlement', 'credit-pack order is non-entitlement');
expect_binding($credit['excluded_items'][0]['disposition'] === 'credit_pack_excluded', 'credit pack is excluded forever');
expect_binding($requestCount() === 5, 'credit-pack order creates no issuance request');

// ── Negative: registration / account / facade / product / price binding ─

$regUnverified = $createRegistration('bind.unverified@example.invalid', $FACADE, $PRODUCT, 'unver', false, false);
$regEmailOnly = $createRegistration('bind.emailonly@example.invalid', $FACADE, $PRODUCT, 'emailonly', true, false);
$regExpired = $createRegistration('bind.expired@example.invalid', $FACADE, $PRODUCT, 'expired', true, true);
$regWrongProduct = $createRegistration('bind.wrongproduct@example.invalid', $FACADE, $UIAPRODUCT, 'wrongproduct', true, true);
$regOther = $createRegistration('bind.other@example.invalid', $FACADE, $PRODUCT, 'other', true, true);
$regEngine = $createRegistration('bind.engine@example.invalid', 'uiai_engine_v1', $PRODUCT, 'engine', true, true);
$regUiai = $createRegistration('bind.uiai@example.invalid', $FACADE, $UIAPRODUCT, 'uiai', true, true);
$regDenied = $createRegistration('bind.denied@example.invalid', $FACADE, $PRODUCT, 'denied', true, true);
$deniedRow = $registrations->findByUuid($regDenied['registration_uuid']);
$registrations->transition($regDenied['registration_uuid'], 'account_promoted', 'denied', (int) $deniedRow['state_version'], 'req-bind-denied-tx-1', 'idem-bind-denied-tx-1', ['state_reason' => 'denied']);

expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => '00000000-0000-4000-8000-000000000000',
        'request_id' => 'req-bind-unknownreg-1',
        'idempotency_key' => 'idem-bind-unknownreg-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unknown registration cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regUnverified['registration_uuid'],
        'request_id' => 'req-bind-unver-1',
        'idempotency_key' => 'idem-bind-unver-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regEmailOnly['registration_uuid'],
        'request_id' => 'req-bind-nopromote-1',
        'idempotency_key' => 'idem-bind-nopromote-1',
    ]),
    'EDD_CUSTOMER_RESOLUTION_FAILED',
    'verified-but-unpromoted registration cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regDenied['registration_uuid'],
        'request_id' => 'req-bind-denied-1',
        'idempotency_key' => 'idem-bind-denied-2',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'denied registration cannot bind an order',
);
$nowValue = '2026-08-10T00:00:00Z';
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regExpired['registration_uuid'],
        'request_id' => 'req-bind-expired-1',
        'idempotency_key' => 'idem-bind-expired-1',
    ]),
    'REGISTRATION_EXPIRED',
    'expired registration cannot bind an order',
);
$nowValue = '2026-08-08T00:01:00Z';
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'facade_id' => 'focusa_arena_v1',
        'origin' => 'https://arena.focusa.dev',
        'request_id' => 'req-bind-wrongfac-1',
        'idempotency_key' => 'idem-bind-wrongfac-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'facade the registration is not bound to is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'origin' => 'https://evil.example',
        'request_id' => 'req-bind-wrongorigin-1',
        'idempotency_key' => 'idem-bind-wrongorigin-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'wrong facade origin is denied',
);
// Facade product allowlist: uiai_engine_v1 does not serve the focusa offer product.
$customerEngine = $customerOf($regEngine['registration_uuid']);
$insertOrder(2014, 'complete', $customerEngine, 'bind.engine@example.invalid', [
    ['item_id' => 2016, 'download' => $DOWNLOAD],
]);
$insertTransaction(2014, $GATEWAY, 'txn_pay_0012');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2014,
        'customer_id' => $customerEngine,
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0012', 'status' => 'complete']],
        'registration_uuid' => $regEngine['registration_uuid'],
        'facade_id' => 'uiai_engine_v1',
        'origin' => 'https://engine.focusa.dev',
        'request_id' => 'req-bind-facprod-1',
        'idempotency_key' => 'idem-bind-facprod-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'facade that does not serve the offer product is denied',
);
// Registration product not matching the offer product (uiai registration, focusa item).
$customerWrongProduct = $customerOf($regWrongProduct['registration_uuid']);
$insertOrder(2015, 'complete', $customerWrongProduct, 'bind.wrongproduct@example.invalid', [
    ['item_id' => 2017, 'download' => $DOWNLOAD],
]);
$insertTransaction(2015, $GATEWAY, 'txn_pay_0013');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2015,
        'customer_id' => $customerWrongProduct,
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0013', 'status' => 'complete']],
        'registration_uuid' => $regWrongProduct['registration_uuid'],
        'request_id' => 'req-bind-wrongprod-1',
        'idempotency_key' => 'idem-bind-wrongprod-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'registration product not matching the offer product is denied',
);

// ── Negative: item, price, mapping, license, account, email binding ────

$customerOther = $customerOf($regOther['registration_uuid']);
$insertOrder(2009, 'complete', $customerOther, 'bind.other@example.invalid', [
    ['item_id' => 2011, 'download' => $DOWNLOAD],
]);
$insertTransaction(2009, $GATEWAY, 'txn_pay_0007');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'order_items' => [['order_item_id' => 2011, 'download_id' => 9999, 'price_id' => $PRICE, 'quantity' => 1]],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-unknowndl-1',
        'idempotency_key' => 'idem-bind-unknowndl-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown download cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'order_items' => [['order_item_id' => 2011, 'download_id' => $DOWNLOAD, 'price_id' => 'price_wrong', 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0007', 'status' => 'complete']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-wrongprice-1',
        'idempotency_key' => 'idem-bind-wrongprice-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'wrong price cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'order_items' => [['order_item_id' => 9999, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-noitemrow-1',
        'idempotency_key' => 'idem-bind-noitemrow-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'order item with no canonical row cannot bind',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'order_items' => [['order_item_id' => 2001, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-itemotherorder-1',
        'idempotency_key' => 'idem-bind-itemotherorder-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'order item bound to another order cannot bind',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-nopay-1',
        'idempotency_key' => 'idem-bind-nopay-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'missing payment transactions cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'synthetic_pay_1', 'status' => 'complete']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-synthpay-1',
        'idempotency_key' => 'idem-bind-synthpay-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'synthetic payment id cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'payment_transactions' => [['gateway' => 'focusa_live', 'transaction_id' => 'focusa_live_x1', 'status' => 'complete']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-livepay-1',
        'idempotency_key' => 'idem-bind-livepay-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'focusa_live synthetic payment id cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_unlinked_1', 'status' => 'complete']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-unlinkedpay-1',
        'idempotency_key' => 'idem-bind-unlinkedpay-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'payment not bound to this canonical order cannot bind',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0001', 'status' => 'processing']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-payprocessing-1',
        'idempotency_key' => 'idem-bind-payprocessing-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'non-complete payment status cannot bind an order',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerA,
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-custmismatch-1',
        'idempotency_key' => 'idem-bind-custmismatch-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'order customer not matching the canonical order row is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2009,
        'customer_id' => $customerOther,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-accountmismatch-1',
        'idempotency_key' => 'idem-bind-accountmismatch-1',
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'order customer not matching the registration account is denied',
);

// Email binding: canonical order email must be the verified registration email.
$customerChanged = $customerOf($regOther['registration_uuid']);
$insertOrder(2010, 'complete', $customerChanged, 'bind.changed@example.invalid', [
    ['item_id' => 2012, 'download' => $DOWNLOAD],
]);
$insertTransaction(2010, $GATEWAY, 'txn_pay_0008');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2010,
        'customer_id' => $customerChanged,
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-changedemail-1',
        'idempotency_key' => 'idem-bind-changedemail-1',
    ]),
    'ACCOUNT_EMAIL_MISMATCH',
    'order email differing from the verified registration is denied',
);
$customerBlank = $customerOf($regOther['registration_uuid']);
$insertOrder(2011, 'complete', $customerBlank, '', [
    ['item_id' => 2013, 'download' => $DOWNLOAD],
]);
$insertTransaction(2011, $GATEWAY, 'txn_pay_0009');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2011,
        'customer_id' => $customerBlank,
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-blankemail-1',
        'idempotency_key' => 'idem-bind-blankemail-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'blank canonical order email is denied',
);

// Blocked mapping and existing active license.
$customerUiai = $customerOf($regUiai['registration_uuid']);
$insertOrder(2012, 'complete', $customerUiai, 'bind.uiai@example.invalid', [
    ['item_id' => 2014, 'download' => 1002],
]);
$insertTransaction(2012, $GATEWAY, 'txn_pay_0010');
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2012,
        'customer_id' => $customerUiai,
        'order_items' => [['order_item_id' => 2014, 'download_id' => 1002, 'price_id' => 'price_uiai_op_v1', 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0010', 'status' => 'complete']],
        'registration_uuid' => $regUiai['registration_uuid'],
        'request_id' => 'req-bind-blockedmap-1',
        'idempotency_key' => 'idem-bind-blockedmap-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'approved-but-blocked mapping cannot bind an order',
);
$customerLicense = $customerOf($regOther['registration_uuid']);
$insertOrder(2013, 'complete', $customerLicense, 'bind.other@example.invalid', [
    ['item_id' => 2015, 'download' => $DOWNLOAD],
]);
$insertTransaction(2013, $GATEWAY, 'txn_pay_0011');
$db->exec("INSERT INTO wp_edd_licenses (id, license_key, customer_id, order_id, product_id, status)
    VALUES (1, 'FOCUSA-FIXTURE-0001', {$customerLicense}, 2013, {$DOWNLOAD}, 'active')");
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2013,
        'customer_id' => $customerLicense,
        'order_items' => [['order_item_id' => 2015, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_0011', 'status' => 'complete']],
        'registration_uuid' => $regOther['registration_uuid'],
        'request_id' => 'req-bind-license-1',
        'idempotency_key' => 'idem-bind-license-1',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'existing active equivalent license blocks settlement',
);

// ── Negative: statuses and caller-controlled fields ────────────────────

expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'order_status' => 'pending',
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-statuspending-1',
        'idempotency_key' => 'idem-bind-statuspending-1',
    ]),
    'EDD_ORDER_PENDING',
    'pending order status is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'order_status' => 'processing',
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-statusprocessing-1',
        'idempotency_key' => 'idem-bind-statusprocessing-1',
    ]),
    'EDD_ORDER_PENDING',
    'processing order status is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'order_status' => 'revoked',
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-statusrevoked-1',
        'idempotency_key' => 'idem-bind-statusrevoked-1',
    ]),
    'REVOKED',
    'revoked order status is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'order_status' => 'failed',
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-statusfailed-1',
        'idempotency_key' => 'idem-bind-statusfailed-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'failed order status is denied',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'price' => '0.01',
        'request_id' => 'req-bind-commercial-1',
        'idempotency_key' => 'idem-bind-commercial-1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled price is forbidden',
);
expect_binding_throws(
    fn() => $bind([
        'order_id' => 2001,
        'customer_id' => $customerA,
        'registration_uuid' => $regA['registration_uuid'],
        'request_id' => 'req-bind-conflict-1',
        'idempotency_key' => 'idem-bind-alpha-1',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// ── Rollback preservation and redaction ────────────────────────────────

$preserved = $bindingMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_order_binding_test', 'record' => 'rollback']);
expect_binding($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_binding((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_order_binding_schema_events')->fetchColumn() === 1, 'exactly one preservation event journaled');

$resultJson = json_encode([
    $bound, $replayed, $duplicate, $pendingRecovered, $blockedComplete, $multi, $mixed, $unrelated, $credit,
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_binding(strpos($resultJson, '@') === false, 'no raw email in any binding decision');
expect_binding(strpos($resultJson, 'fl_') === false, 'no license key in any binding decision');
$bindingRows = $db->query('SELECT * FROM wp_wpuiai_edd_order_bindings')->fetchAll(PDO::FETCH_ASSOC);
$bindingJson = json_encode($bindingRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_binding(strpos($bindingJson, '@') === false, 'no raw email in the binding journal');
expect_binding(strpos($bindingJson, 'txn_pay_') === false, 'no raw payment transaction id in the binding journal');
foreach ($bindingRows as $bindingRow) {
    expect_binding(preg_match('/^(ob_)[0-9a-f]{32}$/D', (string) $bindingRow['binding_key']) === 1, 'binding keys are opaque bounded tokens');
    expect_binding(in_array($bindingRow['binding_state'], ['settled_pending_issuance', 'blocked'], true), 'binding states are bounded');
    expect_binding($bindingRow['payment_transaction_digest'] === null || preg_match('/^[0-9a-f]{64}$/D', (string) $bindingRow['payment_transaction_digest']) === 1, 'payment transaction identities are keyed digests only');
    expect_binding($bindingRow['issuance_request_key'] === null || preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $bindingRow['issuance_request_key']) === 1, 'issuance request keys are opaque bounded tokens');
}
$requestRows = $db->query('SELECT * FROM wp_wpuiai_edd_issuance_requests')->fetchAll(PDO::FETCH_ASSOC);
$requestJson = json_encode($requestRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_binding(strpos($requestJson, '@') === false, 'no raw email in the issuance-request journal');
foreach ($requestRows as $requestRow) {
    expect_binding(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $requestRow['issuance_request_key']) === 1, 'issuance request handles are opaque bounded tokens');
    expect_binding($requestRow['state'] === 'pending', 'issuance requests stay pending (deferred)');
}

// The only license row is the negative-test fixture (FOCUSA-FIXTURE-0001): the binding
// service itself never creates an EDD license, key, or lease.
$licenseRows = $db->query('SELECT * FROM wp_edd_licenses')->fetchAll(PDO::FETCH_ASSOC);
expect_binding(count($licenseRows) === 1 && str_starts_with((string) $licenseRows[0]['license_key'], 'FOCUSA-FIXTURE-'), 'the only license row is the negative-test fixture; settlement creates zero licenses');
$licensesCreatedByService = count($licenseRows) === 1 ? 0 : count($licenseRows);

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_order_binding_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'bindings_journaled' => $bindingCount(),
    'issuance_requests_settled' => $requestCount(),
    'licenses_created' => $licensesCreatedByService,
    'out_of_order_fixtures' => ['canonical_pending', 'canonical_refunded', 'terminal_block_before_complete'],
    'payment_fixtures' => ['synthetic', 'focusa_live_synthetic', 'unlinked', 'non_complete', 'missing'],
    'entitlement_issuance' => 'deferred_to_verified_issuance_service',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
