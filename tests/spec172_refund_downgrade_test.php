<?php
// 172.02.07 Settle refund, chargeback, revoke, and whole-Bundle downgrade.
// The Bundle settles as a WHOLE ORDER only: a Bundle refund is a 30-day whole-order
// refund derived from canonical EDD truth (never caller input); component-level partial
// refunds are not supported in v1 (COMPONENT_REFUND_UNSUPPORTED). Refund/chargeback/
// revoke each settle EXACTLY ONCE against the accepted composite Bundle projection:
// both underlying Operator v1 grants revoke together (grants_revoked=2), the account's
// monotonic authority sequence increments by exactly one, the account/customer/order/
// license/refund/projection/audit history is fully preserved, and a still-mailbox-
// verified account returns to `verified_no_license` limited mode. Duplicate
// redeliveries and second adverse events are journaled `replayed` with zero sequence
// bump; stale `complete`/`unsuspend` cache events fail closed with
// LICENSE_TERMINAL_REACTIVATION_DENIED; out-of-order events fail closed with
// ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED; out-of-window refunds fail closed with
// REFUND_WINDOW_EXPIRED. Applied settlements append a signed transactional outbox row
// in the same transaction, dispatch exactly once through the unique delivery ledger,
// and the bounded reconciler proves idempotent convergence (a second apply run repairs
// zero). The paid -> limited assertion transition fixture derives the paid credential
// from the ACTIVE projection and the limited-mode posture (verified_no_license, limited
// families + permanent safety allowances only) from the applied terminal settlement; a
// stale paid credential can never reactivate (PAID_GRANT_REVOKED /
// STALE_CREDENTIAL_SUPERSEDED). No raw email, key, token, customer row, credential, or
// card data is stored or returned anywhere.
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
require_once $root . '/docs/contracts/spec152e-edd-license-issuance.v1.php';
require_once $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';
require_once $root . '/docs/contracts/spec172-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-uiai-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-bundle-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-bundle-signed-lease-fixture.v1.php';
require_once $root . '/docs/contracts/spec172-limited-access-assertion-service.v1.php';
require_once $root . '/docs/contracts/spec172-refund-downgrade-settlement.v1.php';
require_once $root . '/docs/contracts/spec172-assertion-transition-fixture.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_settlement(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_settlement_throws(callable $operation, string $code, string $message): void
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
function expect_settlement_denied(array $decision, string $code, string $message): void
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
    expect_settlement((int) ($decision['sequence_increment'] ?? -1) === 0, "{$message}: denied events never bump the sequence");
    expect_settlement((int) ($decision['result_sequence'] ?? -1) === (int) ($decision['sequence'] ?? -2), "{$message}: denied events never change the sequence");
}

/** Assert a replay decision: settles once, never bumps the sequence. */
function expect_settlement_replayed(array $decision, int $expectedSequence, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    $decisionValue = $decision['decision'] ?? 'none';
    if ($decisionValue !== 'replayed') {
        fwrite(STDERR, "FAIL: {$message} (decision={$decisionValue})\n");
        exit(1);
    }
    expect_settlement((int) ($decision['sequence_increment'] ?? -1) === 0, "{$message}: replay never bumps the sequence");
    expect_settlement((int) ($decision['result_sequence'] ?? -1) === $expectedSequence, "{$message}: replay keeps the settled sequence");
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);
$settlementMigration = new FocusaSpec172RefundDowngradeMigration($db, 'wp_');
$settlementMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'refund_downgrade_test']);

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
    date_updated VARCHAR(32) NULL,
    user_id INTEGER NULL,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL DEFAULT '',
    total DECIMAL(10,2) NOT NULL DEFAULT 0
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
    user_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    license_length BIGINT NULL,
    license_unit VARCHAR(16) NULL,
    expiration VARCHAR(32) NULL,
    activation_count INTEGER NOT NULL DEFAULT 0,
    activation_limit INTEGER NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
// Canonical EDD refund/dispute truth table (Spec 172 section 17): whole-order refunds
// carry order_item_id NULL and the full order total; item-scoped rows are component
// refunds (unsupported in v1); Stripe dispute rows carry gateway='stripe' with
// status disputed/lost.
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

$frozenRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$frozenDedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';

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
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
        $offer['edd_download_id'] = 1002;
        $offer['edd_price_id'] = 'price_uiai_op_v1';
    }
    if ($offer['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
        $offer['edd_download_id'] = 1003;
        $offer['edd_price_id'] = 'price_bundle_op_v1';
    }
}
unset($offer);

$fixtureDedicated = $frozenDedicated;
foreach ($fixtureDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_operator_lifetime_v1') {
        $record['edd_download_id'] = 1001;
        $record['edd_price_id'] = 'price_focusa_op_v1';
        $record['checkout_enabled'] = true;
        $record['sale_status'] = 'enabled';
    }
    if ($record['public_code'] === 'uiai_operator_lifetime_v1') {
        $record['edd_download_id'] = 1002;
        $record['edd_price_id'] = 'price_uiai_op_v1';
        $record['checkout_enabled'] = true;
        $record['sale_status'] = 'enabled';
    }
    if ($record['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $record['edd_download_id'] = 1003;
        $record['edd_price_id'] = 'price_bundle_op_v1';
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

$adapter = new FocusaSpec172BundleOrderSlAdapter($bindingService, $issuanceService, $fixtureDedicated);

$bundleProjector = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);

$truth = new FocusaSpec172BundleRefundTruthAdapter($db, 'wp_');
$signer = new FocusaSpec172SettlementEventSigner('spec172-refund-downgrade-test-secret');
$settler = new FocusaSpec172RefundDowngradeSettler(
    $db, $settlementMigration, $accounts, $registrations, $edd, $truth, $signer, $clock,
);
$dispatcher = new FocusaSpec172SettlementDispatcher($db, $settlementMigration, $signer, $clock);
$reconciler = new FocusaSpec172SettlementReconciler($db, $settlementMigration, $settler, $truth, $clock);

$limitedSigner = FocusaSpec172LimitedAssertionSigner::fromSeed(str_repeat('a', 64));
$assertionFixture = new FocusaSpec172AssertionTransitionFixture($limitedSigner);

// ── Fixture helpers ────────────────────────────────────────────────────

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$BUNDLE_PRODUCT = 'focusa_uiai_operator_bundle_lifetime_v1';
$BUNDLE_DOWNLOAD = 1003;
$BUNDLE_PRICE = 'price_bundle_op_v1';
$GATEWAY = 'stripe';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.refund.downgrade.test',
        'install_channel' => 'cli',
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
    $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'spec172_candidate', 'record' => 'refund-' . $tag . '-' . $seq],
    ]);
    // Legal state-machine path to the paid checkout state.
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'account_promoted', 'offer_selected', (int) $row['state_version'], 'req-offer-' . $tag . '-' . $seq, 'idem-offer-' . $tag . '-' . $seq, ['state_reason' => 'offer_selected_for_checkout', 'offer_code' => $product]);
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'offer_selected', 'checkout_pending', (int) $row['state_version'], 'req-checkout-' . $tag . '-' . $seq, 'idem-checkout-' . $tag . '-' . $seq, ['state_reason' => 'checkout_pending', 'edd_cart_reference' => 'cart-' . $tag . '-' . $seq]);
    return ['registration_uuid' => $uuid, 'verified_at' => $verified['registration']['verified_at']];
};

$customerOf = static function (string $registrationUuid) use ($registrations): int {
    return (int) $registrations->findByUuid($registrationUuid)['edd_customer_id'];
};

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, array $items = [], ?string $completedAt = '2026-08-08T00:01:00Z', ?string $updatedAt = null) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_number, status, type, date_created, date_completed, date_updated, user_id, customer_id, email, total)
        VALUES (:id, :number, :status, 'sale', '2026-08-08T00:01:00Z', :completed, :updated, NULL, :customer, :email, '1254.60')");
    $statement->execute([
        ':id' => $orderId,
        ':number' => 'EDD-' . $orderId,
        ':status' => $status,
        ':completed' => $completedAt,
        ':updated' => $updatedAt ?? $completedAt,
        ':customer' => $customerId,
        ':email' => $email,
    ]);
    foreach ($items as $item) {
        $rowSeq++;
        $itemStatement = $db->prepare("INSERT INTO wp_edd_order_items
            (id, order_id, product_id, product_name, quantity)
            VALUES (:id, :order, :product, 'fixture', :quantity)");
        $itemStatement->execute([
            ':id' => (int) ($item['item_id'] ?? (300000 + $rowSeq)),
            ':order' => $orderId,
            ':product' => (int) $item['download'],
            ':quantity' => (int) ($item['qty'] ?? 1),
        ]);
    }
};

$txnSeq = 0;
$insertTransaction = static function (int $orderId, string $gateway, string $transactionId, string $status = 'complete', string $total = '1254.60') use ($db, &$txnSeq): void {
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

$bind = static function (int $orderId, string $registrationUuid, int $customerId, array $items, string $txn, string $tag) use ($bindingService, $FACADE, $ORIGIN, $GATEWAY): array {
    return $bindingService->bindOrderComplete([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => array_map(static fn (array $item) => [
            'order_item_id' => (int) $item['item_id'],
            'download_id' => (int) $item['download'],
            'price_id' => (string) $item['price'],
            'quantity' => 1,
        ], $items),
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => $txn, 'status' => 'complete']],
        'registration_uuid' => $registrationUuid,
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'request_id' => 'req-bind-' . $tag,
        'idempotency_key' => 'idem-bind-' . $tag,
    ]);
};

/**
 * Full legal chain for one eligible Bundle order: registration -> verified mailbox ->
 * promoted account -> offer_selected -> checkout_pending -> settled binding -> one
 * canonical EDD SL human key -> one composite Bundle projection (sequence 1).
 */
$bundleOrder = static function (int $orderId, string $email, string $tag) use ($adapter, $bundleProjector, $createRegistration, $customerOf, $insertOrder, $insertTransaction, $bind, $FACADE, $BUNDLE_DOWNLOAD, $BUNDLE_PRICE, $BUNDLE_PRODUCT): array {
    $reg = $createRegistration($email, $FACADE, $BUNDLE_PRODUCT, $tag);
    $customerId = $customerOf($reg['registration_uuid']);
    $insertOrder($orderId, 'complete', $customerId, $email, [['item_id' => $orderId, 'download' => $BUNDLE_DOWNLOAD]]);
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId);
    $bound = $adapter->bindAndIssue([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => [['order_item_id' => $orderId, 'download_id' => $BUNDLE_DOWNLOAD, 'price_id' => $BUNDLE_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_pay_' . $orderId, 'status' => 'complete']],
        'registration_uuid' => $reg['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => 'https://install.focusa.dev',
        'request_id' => 'req-bind-bundle-' . $tag,
        'idempotency_key' => 'idem-bind-bundle-' . $tag,
    ]);
    if (($bound['decision'] ?? '') !== 'bundle_bound_and_issued') {
        throw new RuntimeException('bundle bind failed for ' . $tag);
    }
    $handle = (string) $bound['issuance_request_handle'];
    $projected = $bundleProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-bundle-' . $tag,
        'idempotency_key' => 'idem-project-bundle-' . $tag,
    ]);
    if (($projected['decision'] ?? '') !== 'license_type_projected') {
        throw new RuntimeException('bundle projection failed for ' . $tag);
    }
    $accountUuid = (string) $projected['account_id'];
    return [
        'registration_uuid' => $reg['registration_uuid'],
        'customer_id' => $customerId,
        'account_uuid' => $accountUuid,
        'order_id' => $orderId,
        'handle' => $handle,
        'projection' => $projected,
        'projection_row' => $bundleProjector->findByIssuanceRequestKey($handle),
    ];
};

$settle = static function (int $orderId, int $customerId, string $accountUuid, string $transition, string $tag, ?int $authoritySequence = null) use ($settler): array {
    return $settler->settle([
        'order_id' => $orderId,
        'customer_id' => $customerId,
        'account_uuid' => $accountUuid,
        'transition' => $transition,
        'authority_sequence' => $authoritySequence,
        'request_id' => 'req-settle-' . $transition . '-' . $tag,
        'idempotency_key' => 'idem-settle-' . $transition . '-' . $tag,
    ]);
};

$sequenceOf = static function (string $accountUuid) use ($accounts): int {
    return (int) $accounts->findByUuid($accountUuid)['highest_entitlement_sequence'];
};
$countOf = static function (string $table) use ($db): int {
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
};
$appliedSettlements = static function () use ($settler): int {
    return $settler->appliedSettlementCount();
};

// ── Frozen constants and transition matrix ─────────────────────────────

expect_settlement(FocusaSpec172RefundDowngradeSettler::BUNDLE_SKU === FocusaSpec172LicenseTypeRegistry::BUNDLE_SKU, 'settlement SKU equals the frozen composite Bundle SKU');
expect_settlement(FocusaSpec172RefundDowngradeSettler::BUNDLE_GRANTS === FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), 'settlement grant pair equals the frozen two underlying Operator v1 License Types');
expect_settlement(FocusaSpec172RefundDowngradeSettler::GRANTS_REVOKED === 2, 'a Bundle settlement revokes exactly two grants');
expect_settlement(FocusaSpec172RefundDowngradeSettler::REFUND_WINDOW_DAYS === 30, 'the Bundle refund window is 30 days');
expect_settlement(FocusaSpec172RefundDowngradeSettler::BUNDLE_RESULT_SCHEMA === FocusaSpec172BundleOperatorProjector::RESULT_SCHEMA, 'settlement consumes the composite Bundle projection schema');

// The canonical lifecycle transition matrix: whole-order 30-day refund only; chargeback
// and revoke are adverse authority events outside the customer refund window; terminal
// states never reactivate.
$matrix = FocusaSpec172AssertionTransitionFixture::transitionMatrix();
expect_settlement($matrix === FocusaSpec172RefundDowngradeSettler::TRANSITION_MATRIX, 'the fixture exposes the canonical transition matrix');
expect_settlement($matrix['refund']['to_state'] === 'refunded' && $matrix['refund']['terminal'] === true && $matrix['refund']['adverse'] === true, 'refund is terminal and adverse');
expect_settlement((int) $matrix['refund']['refund_window_days'] === 30 && $matrix['refund']['whole_order_only'] === true, 'refund is the only 30-day whole-order transition');
expect_settlement($matrix['chargeback']['to_state'] === 'refunded' && $matrix['chargeback']['terminal'] === true && $matrix['chargeback']['adverse'] === true, 'chargeback is terminal and adverse');
expect_settlement((int) $matrix['chargeback']['refund_window_days'] === 0, 'chargeback is never bounded by the customer refund window');
expect_settlement($matrix['revoke']['to_state'] === 'revoked' && $matrix['revoke']['terminal'] === true && $matrix['revoke']['adverse'] === true, 'revoke is terminal and adverse');
expect_settlement((int) $matrix['revoke']['refund_window_days'] === 0, 'revoke is never bounded by the customer refund window');
expect_settlement($matrix['complete']['terminal'] === false && $matrix['complete']['adverse'] === false, 'complete is never an adverse settlement event');
expect_settlement($matrix['unsuspend']['terminal'] === false && $matrix['unsuspend']['adverse'] === false, 'unsuspend can never settle or reactivate a terminal Bundle');
foreach (['refund', 'chargeback', 'revoke'] as $adverse) {
    expect_settlement((int) $matrix[$adverse]['sequence_increment'] === 1, "{$adverse} increments the authority sequence by exactly one");
    expect_settlement($matrix[$adverse]['refresh_posture'] === 'recovery_only', "{$adverse} drops refresh posture to recovery_only");
}

// ── Positive: 30-day whole-order Bundle refund ─────────────────────────

$alpha = $bundleOrder(6001, 'refund.alpha@example.invalid', 'alpha');
expect_settlement($alpha['projection']['status'] === 'active', 'alpha Bundle projection is active before settlement');
expect_settlement($sequenceOf($alpha['account_uuid']) === 1, 'alpha account sequence is 1 after projection');
// Canonical EDD truth: whole-order refund of the full 1254.60 within the 30-day window.
$insertRefund(6001, $alpha['customer_id'], null, '1254.60', 'complete', 'edd', '2026-08-10T00:00:00Z');
$markRefunded = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6001");
$markRefunded->execute([':updated' => '2026-08-10T00:00:00Z']);

$refundA = $settle(6001, $alpha['customer_id'], $alpha['account_uuid'], 'refund', 'alpha');
expect_settlement($refundA['schema'] === 'focusa.spec172.bundle_settlement.v1', 'refund decision uses the canonical settlement schema');
expect_settlement($refundA['decision'] === 'applied', '30-day whole-order Bundle refund is applied');
expect_settlement($refundA['transition'] === 'refund' && $refundA['to_state'] === 'refunded', 'refund settles the Bundle to refunded');
expect_settlement($refundA['from_state'] === 'active', 'refund transitions from active');
expect_settlement((int) $refundA['grants_revoked'] === 2, 'refund revokes BOTH Bundle grants together');
expect_settlement($refundA['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'refund removes exactly the two underlying Operator v1 grants');
expect_settlement($refundA['scope'] === 'whole_order', 'refund scope is whole_order (never component)');
expect_settlement((int) $refundA['refund_window_days'] === 30, 'refund decision carries the 30-day window');
expect_settlement($refundA['paid_grants_active'] === false, 'refund removes the paid grants');
expect_settlement($refundA['limited_posture'] === 'verified_no_license', 'still-verified account returns to verified_no_license limited mode');
expect_settlement($refundA['refresh_posture'] === 'recovery_only', 'refund drops refresh posture to recovery_only');
expect_settlement((int) $refundA['sequence'] === 1 && (int) $refundA['result_sequence'] === 2, 'refund advances the authority sequence 1 -> 2');
expect_settlement((int) $refundA['sequence_increment'] === 1, 'refund increments the sequence by exactly one');
expect_settlement($sequenceOf($alpha['account_uuid']) === 2, 'account sequence is 2 after the refund');
expect_settlement($settler->currentEffectiveState(6001) === 'refunded', 'Bundle effective state is refunded');
expect_settlement($settler->paidGrantsActive(6001) === false, 'paid grants are inactive after the refund');
$settlementRowA = $settler->settlementForOrder(6001, 'refund');
expect_settlement($settlementRowA !== null && $settlementRowA['to_state'] === 'refunded', 'refund journal row is applied refunded');
expect_settlement(preg_match('/^(stl_)[0-9a-f]{32}$/D', (string) $refundA['settlement_uuid']) === 1, 'settlement handles are opaque bounded tokens');

// Both Bundle grants revoke together in the effective resolution: the accepted
// projection remains active in its immutable journal, but the settlement makes both
// grants unusable (paid_grants_active=false, grants_revoked=2).
$projectionRowA = $alpha['projection_row'];
expect_settlement((string) $projectionRowA['status'] === 'active', 'projection journal row is never mutated (preservation-only)');
expect_settlement($settler->paidGrantsActive(6001) === false, 'the settled Bundle exposes zero paid grants');
expect_settlement((string) $refundA['license_type_ref'] === 'focusa_uiai_operator_bundle_lifetime_v1', 'settlement names the one composite SKU');

// ── Idempotent outbox: one applied settlement -> one signed envelope -> exactly-once ──

$outboxEventA = $settler->latestOutboxEvent();
expect_settlement($outboxEventA !== null && $outboxEventA['transition'] === 'refund', 'applied refund appended one settlement outbox event');
expect_settlement($outboxEventA['dispatch_state'] === 'pending', 'settlement outbox event starts pending');
expect_settlement((int) $outboxEventA['authority_sequence'] === 1 && (int) $outboxEventA['result_sequence'] === 2, 'outbox envelope carries the exact authority/result sequence pair');
expect_settlement($dispatcher->pendingCount() === 1, 'exactly one pending outbox envelope after one applied settlement');

$dispatch1 = $dispatcher->dispatchOne();
expect_settlement($dispatch1 !== null && $dispatch1['decision'] === 'dispatched' && $dispatch1['delivered'] === true, 'first dispatch delivers the envelope');
expect_settlement($dispatcher->deliveryCount() === 1, 'exactly one delivery ledger row');
expect_settlement($dispatcher->pendingCount() === 0, 'no pending envelopes after dispatch');
$dispatch2 = $dispatcher->dispatchOne();
expect_settlement($dispatch2 === null, 'nothing to dispatch a second time');
expect_settlement($dispatcher->deliveryCount() === 1, 'exactly-once: the delivery ledger never duplicates');

// ── Settle once: duplicate and cross-transition adverse events ─────────

$refundARetry = $settle(6001, $alpha['customer_id'], $alpha['account_uuid'], 'refund', 'alpha-retry');
expect_settlement_replayed($refundARetry, 2, 'duplicate refund redelivery is journaled replayed with zero bump');
expect_settlement($settler->paidGrantsActive(6001) === false, 'replayed duplicate refund never reactivates paid grants');
$refundAReplay = $settler->settle([
    'order_id' => 6001,
    'customer_id' => $alpha['customer_id'],
    'account_uuid' => $alpha['account_uuid'],
    'transition' => 'refund',
    'request_id' => 'req-settle-refund-alpha',
    'idempotency_key' => 'idem-settle-refund-alpha',
]);
expect_settlement_replayed($refundAReplay, 2, 'idempotency-key replay returns the stored decision replayed');
expect_settlement((int) $refundAReplay['result_sequence'] === 2 && (int) $refundAReplay['sequence_increment'] === 0, 'idempotency replay never bumps the sequence');
expect_settlement($appliedSettlements() === 1, 'the adverse refund event settled exactly once (applied count 1)');
expect_settlement($sequenceOf($alpha['account_uuid']) === 2, 'account sequence stays 2 after replay');

// A second adverse event (revoke) on the already-refunded Bundle: both grants were
// already revoked together; journaled replayed, zero bump.
$revokeAfterRefund = $settle(6001, $alpha['customer_id'], $alpha['account_uuid'], 'revoke', 'alpha-revoke');
expect_settlement_replayed($revokeAfterRefund, 2, 'revoke after refund is replayed (both grants already revoked)');
expect_settlement($settler->currentEffectiveState(6001) === 'refunded', 'effective state stays refunded');
expect_settlement($sequenceOf($alpha['account_uuid']) === 2, 'sequence stays 2 after the replayed second adverse event');

// ── Stale cache cannot reactivate ─────────────────────────────────────

$guardComplete = $settler->guardReactivation([
    'order_id' => 6001,
    'customer_id' => $alpha['customer_id'],
    'account_uuid' => $alpha['account_uuid'],
    'transition' => 'complete',
    'request_id' => 'req-stale-complete-alpha',
    'idempotency_key' => 'idem-stale-complete-alpha',
]);
expect_settlement_denied($guardComplete, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'stale complete cache event cannot reactivate a refunded Bundle');
$guardUnsuspend = $settler->guardReactivation([
    'order_id' => 6001,
    'customer_id' => $alpha['customer_id'],
    'account_uuid' => $alpha['account_uuid'],
    'transition' => 'unsuspend',
    'request_id' => 'req-stale-unsuspend-alpha',
    'idempotency_key' => 'idem-stale-unsuspend-alpha',
]);
expect_settlement_denied($guardUnsuspend, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'stale unsuspend cache event cannot reactivate a refunded Bundle');
expect_settlement($settler->currentEffectiveState(6001) === 'refunded' && $sequenceOf($alpha['account_uuid']) === 2, 'reactivation guards never change state or sequence');

// Out-of-order delivery: a genuinely new adverse event whose authority ordinal is not
// newer than the account's highest sequence can never roll the sequence back.
$rollbackRefund = $settle(6001, $alpha['customer_id'], $alpha['account_uuid'], 'refund', 'alpha-rollback', authoritySequence: 2);
expect_settlement_denied($rollbackRefund, 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', 'out-of-order event with a stale authority ordinal is denied');
expect_settlement($sequenceOf($alpha['account_uuid']) === 2, 'sequence rollback denial never changes the sequence');

// ── Component refunds are not supported in v1 ──────────────────────────

$beta = $bundleOrder(6002, 'refund.component@example.invalid', 'beta');
$insertRefund(6002, $beta['customer_id'], 6002, '697.00', 'complete', 'edd', '2026-08-10T00:00:00Z');
$markRefunded2 = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6002");
$markRefunded2->execute([':updated' => '2026-08-10T00:00:00Z']);
$componentRefund = $settle(6002, $beta['customer_id'], $beta['account_uuid'], 'refund', 'beta');
expect_settlement_denied($componentRefund, 'COMPONENT_REFUND_UNSUPPORTED', 'component-level partial Bundle refund is denied in v1');
expect_settlement($settler->paidGrantsActive(6002) === true, 'denied component refund never removes the paid grants');
expect_settlement($sequenceOf($beta['account_uuid']) === 1, 'denied component refund never bumps the sequence');

// ── Refunds outside the 30-day window are denied ───────────────────────

$gamma = $bundleOrder(6003, 'refund.late@example.invalid', 'gamma');
$insertRefund(6003, $gamma['customer_id'], null, '1254.60', 'complete', 'edd', '2026-09-20T00:00:00Z');
$markRefunded3 = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6003");
$markRefunded3->execute([':updated' => '2026-09-20T00:00:00Z']);
$lateRefund = $settle(6003, $gamma['customer_id'], $gamma['account_uuid'], 'refund', 'gamma');
expect_settlement_denied($lateRefund, 'REFUND_WINDOW_EXPIRED', 'Bundle refund after the 30-day window is denied');
expect_settlement($settler->paidGrantsActive(6003) === true, 'denied late refund never removes the paid grants');
expect_settlement($sequenceOf($gamma['account_uuid']) === 1, 'denied late refund never bumps the sequence');

// ── Refund truth must exist: absent canonical refund rows deny ─────────

$zeta = $bundleOrder(6006, 'refund.notruth@example.invalid', 'zeta');
$zetaOrder = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6006");
$zetaOrder->execute([':updated' => '2026-08-10T00:00:00Z']);
$noTruthRefund = $settle(6006, $zeta['customer_id'], $zeta['account_uuid'], 'refund', 'zeta');
expect_settlement_denied($noTruthRefund, 'REFUND_TRUTH_UNKNOWN', 'refund without canonical refund rows is denied');
expect_settlement($settler->paidGrantsActive(6006) === true, 'denied no-truth refund never removes the paid grants');
expect_settlement($sequenceOf($zeta['account_uuid']) === 1, 'denied no-truth refund never bumps the sequence');

// ── Chargeback settles to refunded (no customer refund window) ─────────

$delta = $bundleOrder(6004, 'chargeback.delta@example.invalid', 'delta');
$insertRefund(6004, $delta['customer_id'], null, '1254.60', 'lost', 'stripe', '2026-11-01T00:00:00Z');
$chargebackD = $settle(6004, $delta['customer_id'], $delta['account_uuid'], 'chargeback', 'delta');
expect_settlement($chargebackD['decision'] === 'applied', 'lost Stripe dispute settles the Bundle chargeback');
expect_settlement($chargebackD['transition'] === 'chargeback' && $chargebackD['to_state'] === 'refunded', 'chargeback settles to refunded');
expect_settlement((int) $chargebackD['grants_revoked'] === 2, 'chargeback revokes both Bundle grants together');
expect_settlement($chargebackD['paid_grants_active'] === false && $chargebackD['limited_posture'] === 'verified_no_license', 'chargeback removes paid grants and returns the verified account to limited mode');
expect_settlement((int) $chargebackD['sequence'] === 1 && (int) $chargebackD['result_sequence'] === 2, 'chargeback advances the sequence 1 -> 2');
expect_settlement($sequenceOf($delta['account_uuid']) === 2, 'account sequence is 2 after chargeback');
expect_settlement($settler->currentEffectiveState(6004) === 'refunded', 'chargeback Bundle effective state is refunded');
$chargebackRetry = $settle(6004, $delta['customer_id'], $delta['account_uuid'], 'chargeback', 'delta-retry');
expect_settlement_replayed($chargebackRetry, 2, 'duplicate chargeback redelivery is replayed with zero bump');

// ── Manual revoke settles to revoked ───────────────────────────────────

$epsilon = $bundleOrder(6005, 'revoke.epsilon@example.invalid', 'epsilon');
$epsilonOrder = $db->prepare("UPDATE wp_edd_orders SET status = 'revoked', date_updated = :updated WHERE id = 6005");
$epsilonOrder->execute([':updated' => '2026-08-09T00:00:00Z']);
$revokeE = $settle(6005, $epsilon['customer_id'], $epsilon['account_uuid'], 'revoke', 'epsilon');
expect_settlement($revokeE['decision'] === 'applied', 'manual revoke settles the Bundle');
expect_settlement($revokeE['transition'] === 'revoke' && $revokeE['to_state'] === 'revoked', 'revoke settles to revoked');
expect_settlement((int) $revokeE['grants_revoked'] === 2, 'revoke revokes both Bundle grants together');
expect_settlement($revokeE['paid_grants_active'] === false && $revokeE['limited_posture'] === 'verified_no_license', 'revoke removes paid grants and returns the verified account to limited mode');
expect_settlement((int) $revokeE['sequence'] === 1 && (int) $revokeE['result_sequence'] === 2, 'revoke advances the sequence 1 -> 2');
expect_settlement($sequenceOf($epsilon['account_uuid']) === 2, 'account sequence is 2 after revoke');
expect_settlement($settler->currentEffectiveState(6005) === 'revoked', 'revoked Bundle effective state is revoked');
$revokeGuard = $settler->guardReactivation([
    'order_id' => 6005,
    'customer_id' => $epsilon['customer_id'],
    'account_uuid' => $epsilon['account_uuid'],
    'transition' => 'complete',
    'request_id' => 'req-stale-complete-epsilon',
    'idempotency_key' => 'idem-stale-complete-epsilon',
]);
expect_settlement_denied($revokeGuard, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'revoked Bundle cannot reactivate from a stale complete event');

// ── Chargeback truth missing denies ────────────────────────────────────

$eta = $bundleOrder(6007, 'chargeback.notruth@example.invalid', 'eta');
$chargebackMissing = $settle(6007, $eta['customer_id'], $eta['account_uuid'], 'chargeback', 'eta');
expect_settlement_denied($chargebackMissing, 'CHARGEBACK_TRUTH_UNKNOWN', 'chargeback without canonical dispute truth is denied');
expect_settlement($settler->paidGrantsActive(6007) === true, 'denied chargeback never removes the paid grants');

// ── Paid -> limited assertion transition fixtures ──────────────────────

// Paid credential derives from the ACTIVE projection only.
$paidAlpha = $assertionFixture->paidAssertion($alpha['projection'], 'node-bundle-001', $clock);
expect_settlement($paidAlpha['schema'] === 'focusa.spec172.assertion_transition_fixture.v1' && $paidAlpha['kind'] === 'paid', 'paid assertion fixture schema is canonical');
expect_settlement($paidAlpha['assertion_payload']['schema'] === 'focusa.bundle_signed_lease.v1', 'paid credential uses the bundle signed lease payload schema');
expect_settlement($paidAlpha['assertion_payload']['status'] === 'active' && (int) $paidAlpha['assertion_payload']['sequence'] === 1, 'paid credential is active at sequence 1');
$paidGrants = array_keys($paidAlpha['assertion_payload']['grants']);
sort($paidGrants, SORT_STRING);
expect_settlement($paidGrants === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'paid credential carries both underlying Operator v1 grants');
expect_settlement((int) $paidAlpha['assertion_payload']['human_key_count'] === 1 && $paidAlpha['assertion_payload']['component_refunds_allowed'] === false, 'paid credential is one human key, whole-order refund only');
expect_settlement($paidAlpha['grant_metadata']['refund_policy'] === 'whole_order_30_days', 'paid credential carries the whole-order 30-day refund policy');
$paidExpires = new DateTimeImmutable($paidAlpha['assertion_payload']['expires_at'], new DateTimeZone('UTC'));
$paidIssued = new DateTimeImmutable($paidAlpha['assertion_payload']['issued_at'], new DateTimeZone('UTC'));
expect_settlement($paidExpires <= $paidIssued->modify('+90 days'), 'paid credential refresh window is bounded (90 days)');

// After the terminal settlement the paid assertion can no longer be derived (the paid
// grants are removed) and the stale cached paid credential can never reactivate.
$staleProjection = $alpha['projection'];
$staleProjection['status'] = 'refunded';
expect_settlement_throws(
    fn() => $assertionFixture->paidAssertion($staleProjection, 'node-bundle-001', $clock),
    'PAID_GRANT_REVOKED',
    'a stale refunded projection never yields a paid credential',
);
expect_settlement_throws(
    fn() => FocusaSpec172AssertionTransitionFixture::validatePaidAssertion($paidAlpha, 2, 'refunded'),
    'PAID_GRANT_REVOKED',
    'stale paid credential is rejected once the Bundle is terminal',
);
expect_settlement_throws(
    fn() => FocusaSpec172AssertionTransitionFixture::validatePaidAssertion($paidAlpha, 3, 'active'),
    'STALE_CREDENTIAL_SUPERSEDED',
    'stale paid credential is superseded by the higher authority sequence',
);

// Limited-mode posture derives from the applied terminal settlement: verified_no_license
// with ONLY the frozen limited families and the permanent safety allowances.
$limitedAlpha = $assertionFixture->limitedPosture($refundA, 'node-bundle-001', $clock);
expect_settlement($limitedAlpha['kind'] === 'verified_no_license', 'refunded verified account returns to verified_no_license');
expect_settlement($limitedAlpha['paid_grants_active'] === false && (int) $limitedAlpha['grants_revoked'] === 2, 'limited posture carries the removed paid grants');
expect_settlement((int) $limitedAlpha['sequence'] === 2, 'limited assertion sequence is the settlement result sequence');
$expectedLimited = array_values(array_unique(array_merge(
    FocusaSpec172AssertionTransitionFixture::FOCUSA_LIMITED_FAMILIES,
    FocusaSpec172AssertionTransitionFixture::UIAI_LIMITED_FAMILIES,
    FocusaSpec172AssertionTransitionFixture::PERMANENT_ALLOWANCES,
)));
sort($expectedLimited, SORT_STRING);
expect_settlement($limitedAlpha['families_allowed'] === $expectedLimited, 'limited allowlist is exactly the frozen limited families plus permanent allowances');
expect_settlement($limitedAlpha['paid_families_excluded'] === true, 'paid families are excluded from the limited posture');
foreach (FocusaSpec172LicenseTypeRegistry::underlyingFamilies() as $paidFamily) {
    expect_settlement(in_array($paidFamily, $limitedAlpha['families_allowed'], true) === false, "paid family {$paidFamily} is excluded from limited mode");
}
expect_settlement($limitedAlpha['permanent_allowances'] === FocusaSpec172AssertionTransitionFixture::PERMANENT_ALLOWANCES, 'recovery/export/repair/rollback/security allowances remain available');
$verifyLimited = $assertionFixture->verifyLimited($limitedAlpha);
expect_settlement($verifyLimited['valid'] === true, 'limited assertion verifies with the server-owned Ed25519 key');
expect_settlement($limitedAlpha['assertion']['schema'] === 'focusa.spec172.limited_access_assertion.v1', 'limited assertion payload schema is canonical');
// A widened (paid) family in a validly-signed limited assertion still fails verification
// closed: the allowlist is server-frozen and can never widen into paid families.
$widened = $limitedAlpha;
$widened['assertion']['family_allowlist'] = array_merge($limitedAlpha['families_allowed'], ['automation']);
$widened['signature'] = $limitedSigner->sign($widened['assertion']);
$verifyWidened = $assertionFixture->verifyLimited($widened);
expect_settlement($verifyWidened['valid'] === false && $verifyWidened['error_code'] === 'LIMITED_FAMILY_WIDENING_DENIED', 'limited assertion can never widen into paid families');
$tampered = $limitedAlpha;
$tampered['signature'] = str_repeat('0', 128);
$verifyTampered = $assertionFixture->verifyLimited($tampered);
expect_settlement($verifyTampered['valid'] === false && $verifyTampered['error_code'] === 'LIMITED_SIGNATURE_INVALID', 'tampered limited assertion fails verification');
$unverifiedPosture = $assertionFixture->limitedPosture([
    'decision' => 'applied',
    'to_state' => 'refunded',
    'limited_posture' => 'unverified',
    'result_sequence' => 2,
    'grants_revoked' => 2,
    'created_at' => '2026-08-10T00:00:00Z',
], 'node-unverified-001', $clock);
expect_settlement($unverifiedPosture['kind'] === 'unverified' && $unverifiedPosture['product_access'] === 'registration_only', 'unverified accounts return to registration-only, never a grant');

// ── Idempotent outbox continues for each applied settlement ────────────

expect_settlement($dispatcher->pendingCount() === 2, 'two applied settlements (chargeback + revoke) remain pending; the refund envelope was already dispatched');
// delta (chargeback) and epsilon (revoke) each appended one envelope; the denied
// chargeback for eta appended none.
expect_settlement($dispatcher->deliveryCount() === 1, 'only the refund envelope was dispatched so far');

$dispatchDelta = $dispatcher->dispatchOne();
expect_settlement($dispatchDelta !== null && $dispatchDelta['transition'] === 'chargeback', 'chargeback envelope dispatches next');
expect_settlement($dispatchDelta['delivered'] === true && $dispatcher->deliveryCount() === 2, 'chargeback delivered exactly once');
$dispatchEpsilon = $dispatcher->dispatchOne();
expect_settlement($dispatchEpsilon !== null && $dispatchEpsilon['transition'] === 'revoke', 'revoke envelope dispatches next');
expect_settlement($dispatchEpsilon['delivered'] === true && $dispatcher->deliveryCount() === 3, 'revoke delivered exactly once');
expect_settlement($dispatcher->pendingCount() === 0, 'all applied settlements dispatched');
$dispatchAgain = $dispatcher->dispatchOne();
expect_settlement($dispatchAgain === null, 'no pending envelopes remain');

// ── Tampered outbox envelopes dead-letter (never delivered) ────────────

$theta = $bundleOrder(6008, 'outbox.tamper@example.invalid', 'theta');
$insertRefund(6008, $theta['customer_id'], null, '1254.60', 'complete', 'edd', '2026-08-10T00:00:00Z');
$markRefunded8 = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6008");
$markRefunded8->execute([':updated' => '2026-08-10T00:00:00Z']);
$refundT = $settle(6008, $theta['customer_id'], $theta['account_uuid'], 'refund', 'theta');
expect_settlement($refundT['decision'] === 'applied', 'theta refund applied (for the tamper fixture)');
$tamperEvent = $db->query("SELECT event_uuid FROM wp_wpuiai_spec172_settlement_outbox WHERE dispatch_state = 'pending' ORDER BY created_at DESC LIMIT 1")->fetch(PDO::FETCH_ASSOC);
expect_settlement($tamperEvent !== false, 'theta outbox envelope is the pending tamper target');
$tamperUpdate = $db->prepare("UPDATE wp_wpuiai_spec172_settlement_outbox SET payload = :payload WHERE event_uuid = :event");
$tamperUpdate->execute([':payload' => '{"tampered":true}', ':event' => (string) $tamperEvent['event_uuid']]);
$tamperDispatch = $dispatcher->dispatchOne();
expect_settlement($tamperDispatch !== null && $tamperDispatch['decision'] === 'dead_letter' && $tamperDispatch['error_code'] === 'OUTBOX_DIGEST_INVALID', 'tampered envelope dead-letters on digest verification');
expect_settlement($dispatcher->deadLetterCount() === 1, 'exactly one dead-lettered envelope');
expect_settlement($dispatcher->deliveryCount() === 3, 'tampered envelope is never delivered');
expect_settlement($settler->currentEffectiveState(6008) === 'refunded', 'dead-lettered dispatch never changes the settlement truth');

// ── Reconciler: missing settlements are repaired and converge ──────────

$iota = $bundleOrder(6009, 'reconcile.iota@example.invalid', 'iota');
$insertRefund(6009, $iota['customer_id'], null, '1254.60', 'complete', 'edd', '2026-08-10T00:00:00Z');
$iotaOrder = $db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 6009");
$iotaOrder->execute([':updated' => '2026-08-10T00:00:00Z']);
expect_settlement($settler->currentEffectiveState(6009) === 'active', 'iota Bundle is active before reconciliation');
expect_settlement($sequenceOf($iota['account_uuid']) === 1, 'iota account sequence is 1 before reconciliation');

$dryRun = $reconciler->run('dry_run');
expect_settlement($dryRun['mode'] === 'dry_run' && $dryRun['summary']['repairs_applied'] === 0, 'dry run applies nothing');
expect_settlement((int) $dryRun['summary']['would_repair'] >= 1, 'dry run finds the missing iota settlement');
expect_settlement($settler->currentEffectiveState(6009) === 'active' && $sequenceOf($iota['account_uuid']) === 1, 'dry run never changes state');

$applyRun = $reconciler->run('apply');
expect_settlement($applyRun['mode'] === 'apply', 'apply run executed');
expect_settlement((int) $applyRun['summary']['repairs_applied'] === 1, 'apply repaired exactly one missing settlement');
expect_settlement($applyRun['summary']['converged'] === true, 'apply run converged');
expect_settlement($settler->currentEffectiveState(6009) === 'refunded', 'iota Bundle settled refunded by reconciliation');
expect_settlement($sequenceOf($iota['account_uuid']) === 2, 'iota account sequence advanced by reconciliation');
expect_settlement($settler->paidGrantsActive(6009) === false, 'iota paid grants removed by reconciliation');

$applyRun2 = $reconciler->run('apply');
expect_settlement((int) $applyRun2['summary']['repairs_applied'] === 0 && $applyRun2['summary']['converged'] === true, 'second apply run repairs zero (idempotent convergence)');
expect_settlement($settler->currentEffectiveState(6009) === 'refunded' && $sequenceOf($iota['account_uuid']) === 2, 'converged reconciliation never re-settles or bumps again');
$reconcilerDispatch = $dispatcher->dispatchOne();
expect_settlement($reconcilerDispatch !== null && $reconcilerDispatch['decision'] === 'dispatched', 'reconciliation repair appended and dispatched its outbox envelope');

// ── Client-controlled commerce fields and raw email are forbidden ──────

expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'scope' => 'component', 'request_id' => 'req-bad-scope', 'idempotency_key' => 'idem-bad-scope',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled refund scope is forbidden',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'amount' => 100, 'request_id' => 'req-bad-amount', 'idempotency_key' => 'idem-bad-amount',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled refund amount is forbidden',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'refund_date' => '2026-08-10T00:00:00Z', 'request_id' => 'req-bad-date', 'idempotency_key' => 'idem-bad-date',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled refund date is forbidden',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'grants' => ['focusa_operator_lifetime_v1'], 'request_id' => 'req-bad-grants', 'idempotency_key' => 'idem-bad-grants',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled grant selection is forbidden',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'email' => 'someone@example.test', 'request_id' => 'req-raw-email', 'idempotency_key' => 'idem-raw-email',
    ]),
    'INPUT_RAW_EMAIL_FORBIDDEN',
    'raw email input is forbidden',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'upgrade', 'request_id' => 'req-bad-transition', 'idempotency_key' => 'idem-bad-transition',
    ]),
    'EDD_TRANSITION_UNKNOWN',
    'non-adverse transitions cannot settle a Bundle',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 9999, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'refund', 'request_id' => 'req-no-entitlement', 'idempotency_key' => 'idem-no-entitlement',
    ]),
    'ENTITLEMENT_REQUIRED',
    'a Bundle with no accepted projection can never settle',
);
expect_settlement_throws(
    fn() => $settler->settle([
        'order_id' => 6001, 'customer_id' => $alpha['customer_id'], 'account_uuid' => $alpha['account_uuid'],
        'transition' => 'chargeback', 'request_id' => 'req-conflict', 'idempotency_key' => 'idem-settle-refund-alpha',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'an idempotency-key conflict fails closed',
);

// ── Customer data and recovery remain accessible (preservation) ────────

$preservedCounts = [
    'customers' => $countOf('wp_edd_customers'),
    'orders' => $countOf('wp_edd_orders'),
    'order_items' => $countOf('wp_edd_order_items'),
    'licenses' => $countOf('wp_edd_licenses'),
    'refunds' => $countOf('wp_edd_order_refunds'),
    'projections' => $countOf('wp_wpuiai_license_type_projections'),
    'accounts' => $countOf('wp_wpuiai_authority_accounts'),
    'registrations' => $countOf('wp_wpuiai_activation_registrations'),
];
expect_settlement((int) $preservedCounts['customers'] === 9, 'all nine customers preserved');
expect_settlement((int) $preservedCounts['orders'] === 9, 'all nine orders preserved');
expect_settlement((int) $preservedCounts['licenses'] === 9, 'all nine canonical licenses preserved');
expect_settlement((int) $preservedCounts['refunds'] === 6, 'all six canonical refund rows preserved');
expect_settlement((int) $preservedCounts['projections'] === 9, 'all nine Bundle projections preserved');
expect_settlement((int) $preservedCounts['accounts'] === 9, 'all nine authority accounts preserved');
expect_settlement((int) $preservedCounts['registrations'] === 9, 'all nine verified registrations preserved');

// The still-verified registrations remain mailbox_verified after settlement.
$regAlpha = $registrations->findByUuid($alpha['registration_uuid']);
expect_settlement((string) $regAlpha['verification_state'] === 'mailbox_verified' && $regAlpha['verified_at'] !== null, 'refunded account stays mailbox-verified');
expect_settlement((string) $regAlpha['state'] === 'entitlement_issued', 'registration fulfillment state is preserved (never downgraded or deleted)');
// Recovery, account control, basic export, repair, rollback, stable security update,
// and uninstall stay available through the permanent allowances.
foreach (['read_projection', 'basic_customer_data_export', 'account_control', 'device_control', 'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update', 'uninstall'] as $allowance) {
    expect_settlement(in_array($allowance, $limitedAlpha['permanent_allowances'], true), "recovery allowance {$allowance} remains available");
}
// Account rows, orders, licenses, projections are never deleted by any settlement.
$activeAccounts = $db->prepare('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts WHERE status = :s');
$activeAccounts->execute([':s' => 'active']);
expect_settlement((int) $activeAccounts->fetchColumn() === 9, 'all accounts remain active and preserved');
$storedDecisions = $db->query('SELECT result_payload FROM wp_wpuiai_spec172_settlements')->fetchAll(PDO::FETCH_COLUMN);
expect_settlement(count($storedDecisions) >= 10, 'settlement journal preserves every applied/replayed/denied event');

// ── Redaction: no raw email, key, token, customer row, or card data ────

$decisionJson = json_encode([$refundA, $chargebackD, $revokeE, $componentRefund, $lateRefund, $noTruthRefund, $chargebackMissing, $refundARetry, $revokeAfterRefund, $guardComplete, $rollbackRefund], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_settlement(strpos($decisionJson, '@') === false, 'no raw email in any settlement decision');
expect_settlement(preg_match($KEY_SCAN_PATTERN, $decisionJson) !== 1, 'no full license key in any settlement decision');
expect_settlement(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $decisionJson) !== 1, 'no payment key in any settlement decision');
expect_settlement(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $decisionJson) !== 1, 'no card data in any settlement decision');
expect_settlement(strpos($decisionJson, 'txn_pay_') === false, 'no raw payment transaction id in any settlement decision');

$settlementJson = json_encode($db->query('SELECT * FROM wp_wpuiai_spec172_settlements')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_settlement(strpos($settlementJson, '@') === false, 'no raw email in the settlement journal');
expect_settlement(preg_match($KEY_SCAN_PATTERN, $settlementJson) !== 1, 'no full license key in the settlement journal');
$outboxJson = json_encode($db->query('SELECT * FROM wp_wpuiai_spec172_settlement_outbox')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_settlement(strpos($outboxJson, '@') === false, 'no raw email in the settlement outbox');
expect_settlement(preg_match($KEY_SCAN_PATTERN, $outboxJson) !== 1, 'no full license key in the settlement outbox');
$fixtureJson = json_encode([$paidAlpha, $limitedAlpha], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_settlement(strpos($fixtureJson, '@') === false, 'no raw email in the assertion transition fixtures');
expect_settlement(preg_match($KEY_SCAN_PATTERN, $fixtureJson) !== 1, 'no full license key in the assertion transition fixtures');
expect_settlement(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $fixtureJson) !== 1, 'no payment key in the assertion transition fixtures');

// ── Rollback preservation ──────────────────────────────────────────────

$preserved = $settlementMigration->preserveForRollback('2026-08-10T00:03:00Z', ['source' => 'refund_downgrade_test', 'record' => 'rollback']);
expect_settlement($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_settlement((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_spec172_settlement_schema_events')->fetchColumn() === 1, 'exactly one settlement preservation event journaled');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.refund_downgrade_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'applied_settlements' => $appliedSettlements(),
    'orders_settled' => ['refunded' => $settler->currentEffectiveState(6001), 'chargeback' => $settler->currentEffectiveState(6004), 'revoked' => $settler->currentEffectiveState(6005), 'reconciled' => $settler->currentEffectiveState(6009)],
    'grants_revoked_per_settlement' => 2,
    'grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
    'refund_policy' => 'whole_order_30_days',
    'component_refunds_allowed' => false,
    'limited_posture' => 'verified_no_license',
    'outbox_deliveries' => $dispatcher->deliveryCount(),
    'outbox_dead_letters' => $dispatcher->deadLetterCount(),
    'reconciliation_converged' => $applyRun2['summary']['converged'],
    'preserved' => $preservedCounts,
    'transition_matrix' => array_map(static fn (array $spec): array => [
        'to_state' => $spec['to_state'],
        'terminal' => $spec['terminal'],
        'adverse' => $spec['adverse'],
        'sequence_increment' => $spec['sequence_increment'],
        'refund_window_days' => $spec['refund_window_days'],
        'whole_order_only' => $spec['whole_order_only'],
    ], $matrix),
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
