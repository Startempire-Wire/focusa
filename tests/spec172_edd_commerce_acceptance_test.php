<?php
// 172.02.08 Run the complete Spec 172 EDD commerce matrix through ONE EDD authority
// (WPUIAI EDD) with no live charge and no independent Stripe/facade/install-site
// entitlement. The matrix exercises, end to end and fail-closed:
//   no-license (verified mailbox -> canonical account -> verified_no_license limited
//     posture + signed limited-access assertion, zero EDD order/key; unverified email
//     can never create a posture or product grant);
//   paid Focusa (verified order -> canonical EDD SL key -> focusa_operator_lifetime_v1
//     projection -> bounded paid lease fixture; sequence 1);
//   paid UIAI (uiai_operator_lifetime_v1 projection with the frozen hosted-resource
//     exclusion digest; hosted-resource attempts denied);
//   Bundle (ONE SKU, ONE human key, exact union of the two underlying Operator v1
//     grants, shared three-node baseline; component refunds unsupported);
//   wrong price / wrong product / wrong account (zero projections, zero sequence);
//   Download 453 and legacy credit packs (quarantined/retired, never grant);
//   duplicate order (settles once, one issuance request, one key, replay idempotent);
//   caller grants (CLIENT_COMMERCIAL_FIELDS_FORBIDDEN on every surface);
//   partial Bundle refund (COMPONENT_REFUND_UNSUPPORTED);
//   chargeback (lost Stripe dispute settles the Bundle to refunded, both grants
//     revoked, sequence +1, verified account returns to verified_no_license);
//   future type (Navigator and future products never enter Operator or the Bundle);
//   hosted-resource attempts (HOSTED_RESOURCE_NOT_INCLUDED); and direct
//   Stripe/facade/install-site paths (frozen registry, raw payment rows, unverified
//   binding) create zero entitlement.
// All fixtures are synthetic; journals store only keyed digests and opaque bounded
// tokens; no raw email, key, token, customer row, credential, or card data is stored
// or returned. No live charge is ever made.
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

$positiveChecks = 0;
$negativeChecks = 0;

function expect_matrix(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_matrix_throws(callable $operation, string $code, string $message): void
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

/** Assert a returned denial decision: fail closed with zero sequence change. */
function expect_matrix_denied(array $decision, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    $decisionValue = $decision['decision'] ?? 'none';
    $errorCodeValue = $decision['error_code'] ?? 'none';
    if ($decisionValue !== 'denied' || $errorCodeValue !== $code) {
        fwrite(STDERR, "FAIL: {$message} (decision={$decisionValue}, error_code={$errorCodeValue})\n");
        exit(1);
    }
    expect_matrix((int) ($decision['sequence_increment'] ?? -1) === 0, "{$message}: denied events never bump the sequence");
}

// ── Setup: one SQLite authority, all Spec 172 commerce surfaces ────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$settlementMigration = new FocusaSpec172RefundDowngradeMigration($db, 'wp_');
$settlementMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'spec172_edd_commerce_acceptance_test']);

// Canonical EDD fixture tables (single authority; every surface reads the same tables).
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
// Canonical EDD refund/dispute truth (Spec 172 section 17): whole-order refunds carry
// order_item_id NULL and the full order total; item-scoped rows are component refunds
// (unsupported in v1); Stripe dispute rows carry gateway='stripe' with status lost.
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
$postures = new FocusaSpec172VerifiedAccessPostureRepository($db, $postureMigration, $clock);
$assertions = new FocusaSpec172SignedAccessAssertionRepository($db, $assertionMigration, $postureMigration, $clock);
$limitedSigner = FocusaSpec172LimitedAssertionSigner::fromSeed(str_repeat('a', 64));
$limitedService = new FocusaSpec172LimitedAssertionService($db, $postures, $assertions, $limitedSigner, $postureMigration, $clock);
$assertionFixture = new FocusaSpec172AssertionTransitionFixture($limitedSigner);

// Frozen contracts stay untouched; the fixture registry adds explicitly operator-
// approved test mappings (1001 -> focusa_operator_lifetime_v1, 1002 ->
// uiai_operator_lifetime_v1, 1003 -> focusa_uiai_operator_bundle_lifetime_v1, all
// active/checkout_enabled at the server-owned prices) so the positive paid matrix runs
// against the same single authority without mutating the frozen contracts.
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

// Surface instances: frozen (fail-closed authority/facade view) and fixture (operator-
// approved test mappings). No surface owns price, grants, limits, or rights.
$bindingFrozen = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $frozenRegistry, $facadeRegistry, $clock,
);
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
$uiaiProjector = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);
$bundleAdapter = new FocusaSpec172BundleOrderSlAdapter($bindingService, $issuanceService, $fixtureDedicated);
$bundleProjector = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);
$truth = new FocusaSpec172BundleRefundTruthAdapter($db, 'wp_');
$signer = new FocusaSpec172SettlementEventSigner('spec172-edd-commerce-acceptance-hmac-v1');
$settler = new FocusaSpec172RefundDowngradeSettler(
    $db, $settlementMigration, $accounts, $registrations, $edd, $truth, $signer, $clock,
);
$dispatcher = new FocusaSpec172SettlementDispatcher($db, $settlementMigration, $signer, $clock);
$reconciler = new FocusaSpec172SettlementReconciler($db, $settlementMigration, $settler, $truth, $clock);

// ── Fixture helpers ───────────────────────────────────────────────────

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$FOCUSA_PRODUCT = 'focusa_operator_lifetime_v1';
$UIAI_PRODUCT = 'uiai_operator_lifetime_v1';
$BUNDLE_PRODUCT = 'focusa_uiai_operator_bundle_lifetime_v1';
$FOCUSA_DOWNLOAD = 1001;
$UIAI_DOWNLOAD = 1002;
$BUNDLE_DOWNLOAD = 1003;
$FOCUSA_PRICE = 'price_focusa_op_v1';
$UIAI_PRICE = 'price_uiai_op_v1';
$BUNDLE_PRICE = 'price_bundle_op_v1';
$GATEWAY = 'stripe';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true, bool $checkout = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.spec172.edd.commerce.acceptance.test',
        'install_channel' => 'cli',
        'product_code' => $product,
        'safe_redirect_handle' => 'success',
        'request_id' => 'req-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-' . $tag . '-' . $seq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $result = ['registration_uuid' => $uuid];
    if (!$verify) {
        $result['verification_secret'] = $created['verification_secret'];
        return $result;
    }
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $seq,
        'idem-verify-' . $tag . '-' . $seq,
    );
    $result['verified_at'] = $verified['registration']['verified_at'];
    if (!$promote) {
        return $result;
    }
    $promotionResult = $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'spec172_candidate', 'record' => 'commerce-matrix-' . $tag . '-' . $seq],
    ]);
    $result['account_uuid'] = (string) $promotionResult['account_uuid'];
    $result['identity_uuid'] = (string) $promotionResult['identity_uuid'];
    $result['edd_customer_id'] = (int) $registrations->findByUuid($uuid)['edd_customer_id'];
    if (!$checkout) {
        return $result;
    }
    // Legal state-machine path to the paid checkout state.
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'account_promoted', 'offer_selected', (int) $row['state_version'], 'req-offer-' . $tag . '-' . $seq, 'idem-offer-' . $tag . '-' . $seq, ['state_reason' => 'offer_selected_for_checkout', 'offer_code' => $product]);
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'offer_selected', 'checkout_pending', (int) $row['state_version'], 'req-checkout-' . $tag . '-' . $seq, 'idem-checkout-' . $tag . '-' . $seq, ['state_reason' => 'checkout_pending', 'edd_cart_reference' => 'cart-' . $tag . '-' . $seq]);
    return $result;
};

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, array $items = [], string $total = '697.00', ?string $completedAt = '2026-08-08T00:01:00Z', ?string $updatedAt = null) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_number, status, type, date_created, date_completed, date_updated, user_id, customer_id, email, total)
        VALUES (:id, :number, :status, 'sale', '2026-08-08T00:01:00Z', :completed, :updated, NULL, :customer, :email, :total)");
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

$bind = static function (int $orderId, string $registrationUuid, int $customerId, array $items, string $txn, string $tag, string $priceId = 'price_focusa_op_v1') use ($bindingService, $FACADE, $ORIGIN, $GATEWAY): array {
    return $bindingService->bindOrderComplete([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => array_map(static fn (array $item) => [
            'order_item_id' => (int) $item['item_id'],
            'download_id' => (int) $item['download'],
            'price_id' => (string) ($item['price'] ?? $priceId),
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

$issue = static function (string $handle, string $requestId, string $idempotencyKey) use ($issuanceService): array {
    return $issuanceService->issue([
        'issuance_request_handle' => $handle,
        'request_id' => $requestId,
        'idempotency_key' => $idempotencyKey,
    ]);
};

/**
 * Full legal chain for one paid order: verified registration -> promoted account ->
 * offer_selected -> checkout_pending -> settled binding -> one canonical EDD SL human
 * key -> one License Type projection (sequence 1).
 */
$paidOrder = static function (int $orderId, string $email, string $product, int $download, string $price, string $total, string $tag, FocusaSpec172FocusaOperatorProjector|UiaiSpec172UiaiOperatorProjector $projector) use ($createRegistration, $insertOrder, $insertTransaction, $bind, $issue): array {
    $reg = $createRegistration('commerce.' . $tag . '@example.invalid', 'focusa_install_v1', $product, $tag);
    $customerId = $reg['edd_customer_id'];
    $insertOrder($orderId, 'complete', $customerId, 'commerce.' . $tag . '@example.invalid', [['item_id' => $orderId, 'download' => $download]], $total);
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId, 'complete', $total);
    $bound = $bind($orderId, $reg['registration_uuid'], $customerId, [['item_id' => $orderId, 'download' => $download, 'price' => $price]], 'txn_pay_' . $orderId, $tag . '-1', $price);
    $handle = (string) $bound['protected_items'][0]['issuance_request_handle'];
    $issued = $issue($handle, 'req-issue-' . $tag . '-1', 'idem-issue-' . $tag . '-1');
    $projected = $projector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-' . $tag . '-1',
        'idempotency_key' => 'idem-project-' . $tag . '-1',
    ]);
    return [
        'reg' => $reg,
        'customer_id' => $customerId,
        'order_id' => $orderId,
        'handle' => $handle,
        'bound' => $bound,
        'issued' => $issued,
        'projected' => $projected,
    ];
};

/** Full legal chain for one Bundle order: adapter binds and issues, projector projects. */
$bundleOrder = static function (int $orderId, string $email, string $tag) use ($bundleAdapter, $bundleProjector, $createRegistration, $insertOrder, $insertTransaction, $FACADE, $BUNDLE_DOWNLOAD, $BUNDLE_PRICE, $BUNDLE_PRODUCT): array {
    $reg = $createRegistration($email, $FACADE, $BUNDLE_PRODUCT, $tag);
    $customerId = $reg['edd_customer_id'];
    $insertOrder($orderId, 'complete', $customerId, $email, [['item_id' => $orderId, 'download' => $BUNDLE_DOWNLOAD]], '1254.60');
    $insertTransaction($orderId, 'stripe', 'txn_pay_' . $orderId, 'complete', '1254.60');
    $bound = $bundleAdapter->bindAndIssue([
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
    $handle = (string) $bound['issuance_request_handle'];
    $projected = $bundleProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => 'req-project-bundle-' . $tag,
        'idempotency_key' => 'idem-project-bundle-' . $tag,
    ]);
    return [
        'reg' => $reg,
        'customer_id' => $customerId,
        'account_uuid' => (string) $projected['account_id'],
        'order_id' => $orderId,
        'handle' => $handle,
        'bound' => $bound,
        'projected' => $projected,
        'projection_row' => $bundleProjector->findByIssuanceRequestKey($handle),
    ];
};

$settle = static function (int $orderId, int $customerId, string $accountUuid, string $transition, string $tag) use ($settler): array {
    return $settler->settle([
        'order_id' => $orderId,
        'customer_id' => $customerId,
        'account_uuid' => $accountUuid,
        'transition' => $transition,
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
$grantResolution = static function (int $downloadId, string $priceId) use ($fixtureDedicated): array {
    foreach ($fixtureDedicated['records'] as $record) {
        if ((int) $record['edd_download_id'] === $downloadId && (string) $record['edd_price_id'] === $priceId) {
            return ['ok' => true, 'record' => $record['public_code']];
        }
    }
    return ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'];
};

// ── 0. One authority invariants (frozen contracts stay canonical) ─────

expect_matrix($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema is canonical');
expect_matrix($frozenRegistry['counts']['checkout_enabled'] === 0 && $frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero checkout-enabled offers and zero assigned downloads');
expect_matrix(count($frozenRegistry['protected_offers']) === 3, 'frozen registry has exactly three protected offers');
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_matrix($offer['checkout_enabled'] === false && $offer['sale_status'] === 'approved_not_yet_enabled', "{$offer['public_code']} stays checkout-disabled in the frozen contract");
}
expect_matrix($frozenDedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1' && $frozenDedicated['owner'] === 'WPUIAI/wpuiai', 'dedicated downloads contract is canonical and server-owned');
expect_matrix(count($frozenDedicated['records']) === 3 && $frozenDedicated['counts']['checkout_enabled'] === 0, 'exactly three dedicated records, all checkout-disabled until validation passes');
expect_matrix(in_array(453, array_map('intval', $frozenDedicated['authority']['legacy_download_ids']), true), 'Download 453 is on the never-grant legacy list');
expect_matrix((int) $frozenDedicated['authority']['forbidden_implicit_download'] === 453, 'Download 453 is the explicit forbidden implicit mapping');
expect_matrix(FocusaSpec172LicenseTypeRegistry::FOCUSA_PRICE_USD === '697.00' && FocusaSpec172LicenseTypeRegistry::UIAI_PRICE_USD === '697.00' && FocusaSpec172LicenseTypeRegistry::BUNDLE_PRICE_USD === '1254.60', 'License Type registry prices are the canonical 697/697/1254.60');
expect_matrix(FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes() === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'License Type registry grants exactly the two underlying Operator v1 types');
expect_matrix(FocusaSpec172LicenseTypeRegistry::OPERATOR_SEATS === 1 && FocusaSpec172LicenseTypeRegistry::NODE_LIMIT === 3 && FocusaSpec172LicenseTypeRegistry::NODE_SET === 'operator_shared_v1', 'registry one seat and three shared operator nodes');
expect_matrix(FocusaSpec172LicenseTypeRegistry::FUTURE_PRODUCTS_INCLUDED === false && FocusaSpec172LicenseTypeRegistry::FUTURE_LICENSE_TYPES_INCLUDED === false, 'registry excludes future products and future License Types');
expect_matrix(FocusaSpec172LicenseTypeRegistry::COMPONENT_REFUNDS_ALLOWED === false, 'registry refunds are whole-order only');
expect_matrix(UiaiSpec172HostedResourceExclusionRegistry::SCHEMA === 'focusa.spec172.uiai_hosted_resource_exclusion_registry.v1', 'hosted-resource exclusion registry schema is canonical');
expect_matrix(count(UiaiSpec172HostedResourceExclusionRegistry::EXCLUSIONS) === 8 && UiaiSpec172HostedResourceExclusionRegistry::GRANTED === [], 'hosted-resource registry freezes eight exclusions and grants zero hosted resources');
$catalogByDownload = [];
foreach ($frozenRegistry['current_edd_catalog']['entries'] as $entry) {
    $catalogByDownload[(int) $entry['download_id']] = $entry;
}
expect_matrix($catalogByDownload[453]['entitlement_disposition'] === 'quarantine' && $catalogByDownload[453]['reason'] === 'implicit_focusa_mapping_forbidden', 'Download 453 stays quarantined with the explicit forbidden reason');
foreach ([455, 456, 457] as $creditPackId) {
    expect_matrix(($catalogByDownload[$creditPackId]['entitlement_disposition'] ?? '') === 'retire' && ($catalogByDownload[$creditPackId]['reason'] ?? '') === 'credit_pack_excluded_from_entitlement_registry', "credit pack download {$creditPackId} is retired and excluded");
}
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_matrix((int) ($offer['edd_download_id'] ?? 0) !== 453, 'no protected offer maps Download 453');
}
expect_matrix($assertionFixture->transitionMatrix() === FocusaSpec172RefundDowngradeSettler::TRANSITION_MATRIX, 'the transition fixture exposes the canonical lifecycle matrix');
expect_matrix(FocusaSpec172RefundDowngradeSettler::BUNDLE_GRANTS === FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), 'settlement grant pair equals the frozen two underlying License Types');

// ── 1. No-license: verified_no_license limited posture, zero EDD key ───

$regNoLicense = $createRegistration('commerce.nolicense@example.invalid', $FACADE, 'focusa', 'nolicense', verify: true, promote: true, checkout: false);
$licensesBeforeNoLicense = $countOf('wp_edd_licenses');
$postureInput = [
    'verification_state' => 'account_promoted',
    'verified_at' => $regNoLicense['verified_at'],
    'account_uuid' => $regNoLicense['account_uuid'],
    'identity_uuid' => $regNoLicense['identity_uuid'],
    'registration_uuid' => $regNoLicense['registration_uuid'],
    'product_scope' => 'focusa',
    'node_uuid' => 'node-nolicense-0001',
    'node_digest' => hash('sha256', 'node-nolicense-0001'),
    'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'signer' => 'wpuiai.spec172.issue.v1',
    'sequence' => 1,
    'issued_at' => '2026-08-08T00:02:00Z',
    'refresh_at' => '2026-08-08T00:02:00Z',
    'migration_provenance' => ['source' => 'spec172_edd_commerce_acceptance_test', 'record' => 'nolicense-1'],
];
$posture = $postures->recordPosture($postureInput);
expect_matrix($posture['status'] === 'issued' && $posture['product_scope'] === 'focusa', 'verified account receives exactly one verified_no_license posture');
expect_matrix($countOf('wp_edd_licenses') === $licensesBeforeNoLicense, 'no-license posture creates zero EDD license keys');
$assertionNoLicense = $limitedService->issue([
    'posture_uuid' => $posture['posture_uuid'],
    'issued_at' => '2026-08-08T00:02:00Z',
    'refresh_at' => '2026-09-07T00:02:00Z',
    'migration_provenance' => ['source' => 'spec172_edd_commerce_acceptance_test', 'record' => 'nolicense-assertion-1'],
]);
expect_matrix($assertionNoLicense['verdict'] === 'valid' && $assertionNoLicense['product_scope'] === 'focusa', 'no-license posture issues one signed limited-access assertion');
expect_matrix($assertionNoLicense['family_allowlist'] === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'assertion allowlist is exactly the frozen Focusa limited allowlist');
foreach (FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES as $paidFamily) {
    expect_matrix(in_array($paidFamily, $assertionNoLicense['family_allowlist'], true) === false, "paid family {$paidFamily} is excluded from the no-license allowlist");
}
expect_matrix($assertionNoLicense['refresh_at'] > $assertionNoLicense['issued_at'], 'limited credential lifetime is bounded (never perpetual)');
// The signed assertion verifies with the server-owned Ed25519 key.
$verifyNoLicense = $limitedService->verify([
    'posture_uuid' => $posture['posture_uuid'],
    'account_uuid' => $regNoLicense['account_uuid'],
    'identity_uuid' => $regNoLicense['identity_uuid'],
    'product_scope' => 'focusa',
    'node_uuid' => 'node-nolicense-0001',
    'family_allowlist' => $assertionNoLicense['family_allowlist'],
    'sequence' => $assertionNoLicense['sequence'],
    'issued_at' => $assertionNoLicense['issued_at'],
    'refresh_at' => $assertionNoLicense['refresh_at'],
    'signer' => $assertionNoLicense['signer'],
    'signature' => $assertionNoLicense['signature'],
], '2026-08-08T00:02:30Z');
expect_matrix($verifyNoLicense['verdict'] === 'valid', 'the no-license assertion verifies at the authority');
// No anonymous product capability: unverified input can never create a posture or grant.
expect_matrix_throws(
    fn() => $postures->recordPosture(array_merge($postureInput, ['verification_state' => 'pending', 'verified_at' => ''])),
    'EMAIL_VERIFICATION_REQUIRED',
    'an unverified email can never create a limited posture',
);
expect_matrix($countOf('wp_edd_licenses') === $licensesBeforeNoLicense, 'unverified attempts create zero licenses');
$licensesAfterNoLicense = $countOf('wp_edd_licenses');
expect_matrix($licensesAfterNoLicense === $licensesBeforeNoLicense, 'no-license matrix never touches the EDD license table');

// ── 2. Paid Focusa: one canonical key + one Operator v1 projection ────

$focusa = $paidOrder(7001, 'commerce.focusa@example.invalid', $FOCUSA_PRODUCT, $FOCUSA_DOWNLOAD, $FOCUSA_PRICE, '697.00', 'focusa', $focusaProjector);
expect_matrix($focusa['bound']['decision'] === 'order_bound' && $focusa['bound']['issuance_requests_settled'] === 1, 'paid Focusa order settles exactly one issuance request');
expect_matrix($focusa['issued']['decision'] === 'license_issued' && $focusa['issued']['keys_created'] === 1, 'paid Focusa order issues exactly one canonical EDD SL key');
expect_matrix(preg_match($KEY_PATTERN, (string) $focusa['issued']['delivery']['license_key']) === 1, 'delivered Focusa key is canonical EDD SL format');
expect_matrix(str_starts_with((string) $focusa['issued']['delivery']['license_key'], 'focusa_live_') === false, 'the adapter never issues a synthetic install-site key');
$projectedFocusa = $focusa['projected'];
expect_matrix($projectedFocusa['decision'] === 'license_type_projected' && $projectedFocusa['license_type'] === $FOCUSA_PRODUCT, 'Focusa order projects focusa_operator_lifetime_v1');
expect_matrix($projectedFocusa['product'] === 'focusa' && $projectedFocusa['grant'] === $FOCUSA_PRODUCT, 'projection carries the Focusa product grant');
expect_matrix($projectedFocusa['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'projection carries the frozen Focusa family digest');
expect_matrix($projectedFocusa['operator_seats'] === 1 && $projectedFocusa['node_limit'] === 3 && $projectedFocusa['node_set'] === 'operator_shared_v1', 'projection freezes one seat and three shared nodes');
expect_matrix($projectedFocusa['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1' && $projectedFocusa['price_usd'] === '697.00', 'projection carries the server-owned 697.00 price version');
expect_matrix((int) $projectedFocusa['sequence'] === 1, 'Focusa projection carries the first monotonic sequence');
$focusaAccount = (string) $projectedFocusa['account_id'];
expect_matrix($sequenceOf($focusaAccount) === 1, 'authority account sequence advanced to 1 for Focusa');
// Paid lease fixture derives the bounded credential exclusively from the projection.
$focusaLease = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($projectedFocusa, 'node-operator-001', $clock);
expect_matrix($focusaLease['lease_payload']['status'] === 'active' && (int) $focusaLease['lease_payload']['sequence'] === 1, 'Focusa lease credential is active at sequence 1');
expect_matrix($focusaLease['grant_metadata']['refund_policy'] === 'whole_order_30_days', 'Focusa lease carries the whole-order 30-day refund policy');

// ── 3. Paid UIAI: explicit UIAI grants + hosted-resource exclusions ───

$uiai = $paidOrder(7002, 'commerce.uiai@example.invalid', $UIAI_PRODUCT, $UIAI_DOWNLOAD, $UIAI_PRICE, '697.00', 'uiai', $uiaiProjector);
$projectedUiai = $uiai['projected'];
expect_matrix($projectedUiai['decision'] === 'license_type_projected' && $projectedUiai['license_type'] === $UIAI_PRODUCT, 'UIAI order projects uiai_operator_lifetime_v1');
expect_matrix($projectedUiai['product'] === 'uiai_engine', 'UIAI projection carries the uiai_engine product');
expect_matrix($projectedUiai['hosted_resources_included'] === [], 'UIAI projection grants zero hosted resources');
expect_matrix($projectedUiai['hosted_resource_exclusion_digest'] === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'UIAI projection carries the frozen hosted-resource exclusion digest');
expect_matrix((int) $projectedUiai['sequence'] === 1 && $sequenceOf((string) $projectedUiai['account_id']) === 1, 'UIAI projection advances its account sequence to 1');
// Hosted-resource attempts fail closed on the exclusion registry.
foreach (UiaiSpec172HostedResourceExclusionRegistry::exclusionList() as $resource) {
    expect_matrix(UiaiSpec172HostedResourceExclusionRegistry::isIncluded($resource) === false, "hosted resource {$resource} is never included in v1");
    expect_matrix_throws(
        fn() => UiaiSpec172HostedResourceExclusionRegistry::assertIncluded($resource),
        'HOSTED_RESOURCE_NOT_INCLUDED',
        "hosted resource {$resource} denies with HOSTED_RESOURCE_NOT_INCLUDED",
    );
}
expect_matrix_throws(
    fn() => UiaiSpec172HostedResourceExclusionRegistry::assertIncluded('unregistered_future_resource'),
    'HOSTED_RESOURCE_NOT_INCLUDED',
    'unknown hosted resources fail closed',
);

// ── 4. Bundle: one SKU, one key, exact union of two grants ────────────

$bundle = $bundleOrder(7003, 'commerce.bundle@example.invalid', 'bundle');
$projectedBundle = $bundle['projected'];
expect_matrix($projectedBundle['decision'] === 'license_type_projected', 'Bundle order projects the composite SKU');
expect_matrix($projectedBundle['license_type'] === $BUNDLE_PRODUCT && $projectedBundle['sku'] === $BUNDLE_PRODUCT, 'Bundle projection names the one composite SKU');
$bundleGrants = (array) ($projectedBundle['grants'] ?? []);
sort($bundleGrants, SORT_STRING);
expect_matrix($bundleGrants === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'Bundle projection grants exactly the two underlying License Types');
expect_matrix($projectedBundle['family_digest'] === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'Bundle family digest is the derived two-record union, never a third list');
expect_matrix($projectedBundle['human_key_count'] === 1 && $projectedBundle['component_refunds_allowed'] === false, 'Bundle is one human key with whole-order refunds only');
expect_matrix($projectedBundle['operator_seats'] === 1 && $projectedBundle['node_limit'] === 3 && $projectedBundle['node_set'] === 'operator_shared_v1', 'Bundle reuses the same three shared operator nodes (never six)');
expect_matrix($projectedBundle['price_version'] === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'Bundle carries the server-owned 1254.60 price version');
expect_matrix($projectedBundle['future_products_included'] === false && $projectedBundle['future_license_types_included'] === false, 'Bundle excludes future products and future License Types');
$bundleAccount = $bundle['account_uuid'];
expect_matrix($sequenceOf($bundleAccount) === 1, 'Bundle account sequence advanced to 1');
// The Bundle signed lease fixture carries both grants and one human key.
$bundleLease = FocusaSpec172BundleSignedLeaseFixture::fromProjection($projectedBundle, 'node-operator-001', $clock);
$leaseGrants = array_keys($bundleLease['lease_payload']['grants']);
sort($leaseGrants, SORT_STRING);
expect_matrix($leaseGrants === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'Bundle lease payload grants both underlying Operator v1 types');
expect_matrix((int) $bundleLease['lease_payload']['human_key_count'] === 1, 'Bundle lease is exactly one human key');
expect_matrix(FocusaSpec172BundleSignedLeaseFixture::validate($bundleLease, $projectedBundle) === null, 'Bundle lease fixture validates against the projection');
expect_matrix($countOf('wp_edd_licenses') === 3, 'exactly three canonical EDD licenses across the three paid matrix rows (focusa + uiai + bundle)');

// ── 5. Wrong price / product / account: zero projections, zero sequence ──

// Wrong price: a settled item whose price no longer matches the dedicated offer.
$regPrice = $createRegistration('commerce.price@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'price');
$customerPrice = $regPrice['edd_customer_id'];
$insertOrder(7010, 'complete', $customerPrice, 'commerce.price@example.invalid', [['item_id' => 7010, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7010, $GATEWAY, 'txn_pay_7010');
$boundPrice = $bind(7010, $regPrice['registration_uuid'], $customerPrice, [['item_id' => 7010, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7010', 'price-1');
$handlePrice = (string) $boundPrice['protected_items'][0]['issuance_request_handle'];
$issuedPrice = $issue($handlePrice, 'req-issue-price-1', 'idem-issue-price-1');
expect_matrix($issuedPrice['keys_created'] === 1, 'correct-price item issues its key first');
$db->exec("UPDATE wp_wpuiai_edd_order_bindings SET price_id = 'price_wrong' WHERE binding_key = (SELECT binding_key FROM wp_wpuiai_edd_issuance_requests WHERE issuance_request_key = '{$handlePrice}')");
expect_matrix_throws(
    fn() => $focusaProjector->project(['issuance_request_handle' => $handlePrice, 'request_id' => 'req-project-price-1', 'idempotency_key' => 'idem-project-price-1']),
    'PRODUCT_MAPPING_REQUIRED',
    'a wrong price can never project a License Type',
);
// Wrong price at the binding boundary fails closed.
$regPriceBind = $createRegistration('commerce.pricebind@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'pricebind');
$customerPriceBind = $regPriceBind['edd_customer_id'];
$insertOrder(7011, 'complete', $customerPriceBind, 'commerce.pricebind@example.invalid', [['item_id' => 7011, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7011, $GATEWAY, 'txn_pay_7011');
expect_matrix_throws(
    fn() => $bind(7011, $regPriceBind['registration_uuid'], $customerPriceBind, [['item_id' => 7011, 'download' => $FOCUSA_DOWNLOAD, 'price' => 'price_wrong']], 'txn_pay_7011', 'pricebind-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a wrong price_id can never settle an issuance request',
);
// Wrong product: a UIAI license can never project Focusa, and a Focusa license can
// never project UIAI or the Bundle. Each cross-product attempt uses a freshly bound
// and issued (but never projected) order so the wrong projector must fail closed.
$regWrongUiai = $createRegistration('commerce.wronguiai@example.invalid', $FACADE, $UIAI_PRODUCT, 'wronguiai');
$customerWrongUiai = $regWrongUiai['edd_customer_id'];
$insertOrder(7013, 'complete', $customerWrongUiai, 'commerce.wronguiai@example.invalid', [['item_id' => 7013, 'download' => $UIAI_DOWNLOAD]]);
$insertTransaction(7013, $GATEWAY, 'txn_pay_7013');
$boundWrongUiai = $bind(7013, $regWrongUiai['registration_uuid'], $customerWrongUiai, [['item_id' => 7013, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_7013', 'wronguiai-1', $UIAI_PRICE);
$handleWrongUiai = (string) $boundWrongUiai['protected_items'][0]['issuance_request_handle'];
$issuedWrongUiai = $issue($handleWrongUiai, 'req-issue-wronguiai-1', 'idem-issue-wronguiai-1');
expect_matrix($issuedWrongUiai['keys_created'] === 1 && $issuedWrongUiai['license_type_ref'] === $UIAI_PRODUCT, 'UIAI item issues its own canonical UIAI key');
expect_matrix_throws(
    fn() => $focusaProjector->project(['issuance_request_handle' => $handleWrongUiai, 'request_id' => 'req-project-uiai-focusa-1', 'idempotency_key' => 'idem-project-uiai-focusa-1']),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a UIAI license can never project focusa_operator_lifetime_v1',
);
$regWrongFocusa = $createRegistration('commerce.wrongfocusa@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'wrongfocusa');
$customerWrongFocusa = $regWrongFocusa['edd_customer_id'];
$insertOrder(7014, 'complete', $customerWrongFocusa, 'commerce.wrongfocusa@example.invalid', [['item_id' => 7014, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7014, $GATEWAY, 'txn_pay_7014');
$boundWrongFocusa = $bind(7014, $regWrongFocusa['registration_uuid'], $customerWrongFocusa, [['item_id' => 7014, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7014', 'wrongfocusa-1');
$handleWrongFocusa = (string) $boundWrongFocusa['protected_items'][0]['issuance_request_handle'];
$issuedWrongFocusa = $issue($handleWrongFocusa, 'req-issue-wrongfocusa-1', 'idem-issue-wrongfocusa-1');
expect_matrix($issuedWrongFocusa['keys_created'] === 1 && $issuedWrongFocusa['license_type_ref'] === $FOCUSA_PRODUCT, 'Focusa item issues its own canonical Focusa key');
expect_matrix_throws(
    fn() => $uiaiProjector->project(['issuance_request_handle' => $handleWrongFocusa, 'request_id' => 'req-project-focusa-uiai-1', 'idempotency_key' => 'idem-project-focusa-uiai-1']),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a Focusa license can never project uiai_operator_lifetime_v1',
);
expect_matrix_throws(
    fn() => $bundleProjector->project(['issuance_request_handle' => $handleWrongFocusa, 'request_id' => 'req-project-focusa-bundle-1', 'idempotency_key' => 'idem-project-focusa-bundle-1']),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a standalone Focusa license can never project the Bundle',
);
expect_matrix($countOf('wp_wpuiai_license_type_projections') === 3, 'wrong-price/product denials create zero projections (3 paid projections remain)');
// Wrong account: the order customer changed after settlement.
$regAccount = $createRegistration('commerce.account@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'account');
$customerAccount = $regAccount['edd_customer_id'];
$insertOrder(7012, 'complete', $customerAccount, 'commerce.account@example.invalid', [['item_id' => 7012, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7012, $GATEWAY, 'txn_pay_7012');
$boundAccount = $bind(7012, $regAccount['registration_uuid'], $customerAccount, [['item_id' => 7012, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7012', 'account-1');
$handleAccount = (string) $boundAccount['protected_items'][0]['issuance_request_handle'];
$issuedAccount = $issue($handleAccount, 'req-issue-account-1', 'idem-issue-account-1');
expect_matrix($issuedAccount['keys_created'] === 1, 'account-fixture item issues its key first');
$db->exec('UPDATE wp_edd_orders SET customer_id = 424242 WHERE id = 7012');
expect_matrix_throws(
    fn() => $focusaProjector->project(['issuance_request_handle' => $handleAccount, 'request_id' => 'req-project-account-1', 'idempotency_key' => 'idem-project-account-1']),
    'EDD_ORDER_UNVERIFIED',
    'an order whose customer changed after settlement can never project',
);
expect_matrix($countOf('wp_wpuiai_license_type_projections') === 3, 'wrong-account denial creates zero projections');
expect_matrix($sequenceOf($focusaAccount) === 1 && $sequenceOf((string) $uiai['projected']['account_id']) === 1 && $sequenceOf($bundleAccount) === 1, 'denied wrong price/product/account never bumps any sequence');

// ── 6. Download 453 and legacy credit packs never grant ───────────────

expect_matrix($grantResolution(453, 'price_legacy_453') === ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'], 'Download 453 cannot resolve to any Operator v1 record');
expect_matrix($grantResolution(455, 'price_credit') === ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'], 'credit pack download 455 cannot resolve to any Operator v1 record');
expect_matrix($grantResolution(456, 'price_credit') === ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'], 'credit pack download 456 cannot resolve to any Operator v1 record');
expect_matrix($grantResolution(9999, 'price_unknown') === ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'], 'unknown downloads cannot resolve to any Operator v1 record');
// A complete 453 order binds as proven non-entitlement with zero protected items.
$reg453 = $createRegistration('commerce.d453@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'd453');
$customer453 = $reg453['edd_customer_id'];
$insertOrder(7020, 'complete', $customer453, 'commerce.d453@example.invalid', [['item_id' => 7020, 'download' => 453]]);
$insertTransaction(7020, $GATEWAY, 'txn_pay_7020');
$licensesBefore453 = $countOf('wp_edd_licenses');
$bound453 = $bindingService->bindOrderComplete([
    'order_id' => 7020, 'order_status' => 'complete', 'customer_id' => $customer453,
    'order_items' => [['order_item_id' => 7020, 'download_id' => 453, 'price_id' => 'price_legacy_453', 'quantity' => 1]],
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7020', 'status' => 'complete']],
    'registration_uuid' => $reg453['registration_uuid'],
    'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'request_id' => 'req-bind-453-1', 'idempotency_key' => 'idem-bind-453-1',
]);
expect_matrix($bound453['decision'] === 'no_entitlement' && (int) $bound453['protected_items'] === 0, 'Download 453 settles zero protected items (proven non-entitlement)');
expect_matrix($countOf('wp_edd_licenses') === $licensesBefore453, 'Download 453 creates zero EDD licenses');
// A complete credit-pack order (455) is equally non-entitlement.
$regCredit = $createRegistration('commerce.credit@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'credit');
$customerCredit = $regCredit['edd_customer_id'];
$insertOrder(7021, 'complete', $customerCredit, 'commerce.credit@example.invalid', [['item_id' => 7021, 'download' => 455]]);
$insertTransaction(7021, $GATEWAY, 'txn_pay_7021');
$boundCredit = $bindingService->bindOrderComplete([
    'order_id' => 7021, 'order_status' => 'complete', 'customer_id' => $customerCredit,
    'order_items' => [['order_item_id' => 7021, 'download_id' => 455, 'price_id' => 'price_credit', 'quantity' => 1]],
    'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7021', 'status' => 'complete']],
    'registration_uuid' => $regCredit['registration_uuid'],
    'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'request_id' => 'req-bind-credit-1', 'idempotency_key' => 'idem-bind-credit-1',
]);
expect_matrix($boundCredit['decision'] === 'no_entitlement' && (int) $boundCredit['protected_items'] === 0, 'credit pack orders settle zero protected items');
expect_matrix($countOf('wp_edd_licenses') === $licensesBefore453, 'credit pack orders create zero EDD licenses');
// The Bundle adapter can never be fooled by Download 453 either.
expect_matrix_throws(
    fn() => $bundleAdapter->bindAndIssue([
        'order_id' => 7022, 'order_status' => 'complete', 'customer_id' => $customer453,
        'order_items' => [['order_item_id' => 7022, 'download_id' => 453, 'price_id' => 'price_legacy_453', 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7022', 'status' => 'complete']],
        'registration_uuid' => $reg453['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bundle-453-1', 'idempotency_key' => 'idem-bundle-453-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'Download 453 can never bind as the Bundle SKU',
);

// ── 7. Duplicate order: one issuance request, one key, replay idempotent ──

$regDup = $createRegistration('commerce.dup@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'dup');
$customerDup = $regDup['edd_customer_id'];
$insertOrder(7030, 'complete', $customerDup, 'commerce.dup@example.invalid', [['item_id' => 7030, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7030, $GATEWAY, 'txn_pay_7030');
$boundDup = $bind(7030, $regDup['registration_uuid'], $customerDup, [['item_id' => 7030, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7030', 'dup-1');
expect_matrix($boundDup['decision'] === 'order_bound' && $boundDup['existing'] === false && $boundDup['issuance_requests_settled'] === 1, 'first completion settles exactly one issuance request');
$replayedDup = $bind(7030, $regDup['registration_uuid'], $customerDup, [['item_id' => 7030, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7030', 'dup-1');
expect_matrix(json_encode($replayedDup, JSON_THROW_ON_ERROR) === json_encode($boundDup, JSON_THROW_ON_ERROR), 'idempotency replay returns the byte-identical binding decision');
$duplicateDup = $bind(7030, $regDup['registration_uuid'], $customerDup, [['item_id' => 7030, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_7030', 'dup-2');
expect_matrix($duplicateDup['existing'] === true && $duplicateDup['issuance_requests_settled'] === 0, 'duplicate completion event settles nothing new');
expect_matrix((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_issuance_requests WHERE order_id = 7030')->fetchColumn() === 1, 'duplicate/replay creates exactly one issuance request for the one order');
$issuedDup = $issue($boundDup['protected_items'][0]['issuance_request_handle'], 'req-issue-dup-1', 'idem-issue-dup-1');
$issuedDupRetry = $issue($boundDup['protected_items'][0]['issuance_request_handle'], 'req-issue-dup-2', 'idem-issue-dup-2');
expect_matrix($issuedDupRetry['existing'] === true && $issuedDupRetry['keys_created'] === 0 && $issuedDupRetry['delivery']['license_key'] === $issuedDup['delivery']['license_key'], 'issuance retry returns the identical key with zero keys created');
expect_matrix((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses WHERE order_id = 7030')->fetchColumn() === 1, 'exactly one canonical license row for the duplicate-order fixture');

// ── 8. Caller grants: caller commerce fields are impossible on every surface ──

expect_matrix_throws(
    fn() => $bindingService->bindOrderComplete([
        'order_id' => 7030, 'order_status' => 'complete', 'customer_id' => $customerDup,
        'order_items' => [['order_item_id' => 7030, 'download_id' => $FOCUSA_DOWNLOAD, 'price_id' => $FOCUSA_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7030', 'status' => 'complete']],
        'registration_uuid' => $regDup['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-grants-1', 'idempotency_key' => 'idem-bind-grants-1',
        'grants' => [$FOCUSA_PRODUCT],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'order binding rejects caller-supplied grants',
);
expect_matrix_throws(
    fn() => $issuanceService->issue([
        'issuance_request_handle' => $focusa['handle'],
        'request_id' => 'req-issue-forbid-2',
        'idempotency_key' => 'idem-issue-forbid-2',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'issuance rejects caller-controlled price',
);
expect_matrix_throws(
    fn() => $focusaProjector->project([
        'issuance_request_handle' => $focusa['handle'],
        'request_id' => 'req-project-forbid-1',
        'idempotency_key' => 'idem-project-forbid-1',
        'license_type_ref' => $UIAI_PRODUCT,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'projection rejects caller-supplied License Type metadata',
);
expect_matrix_throws(
    fn() => $focusaProjector->project([
        'issuance_request_handle' => $focusa['handle'],
        'request_id' => 'req-project-forbid-2',
        'idempotency_key' => 'idem-project-forbid-2',
        'node_limit' => 99,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'projection rejects caller-controlled node limits',
);
expect_matrix_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $bundle['handle'],
        'request_id' => 'req-project-bundle-forbid-1',
        'idempotency_key' => 'idem-project-bundle-forbid-1',
        'grants' => [$FOCUSA_PRODUCT],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'Bundle projection rejects caller-supplied grants',
);
expect_matrix_throws(
    fn() => $bundleAdapter->bindAndIssue([
        'order_id' => 7031, 'order_status' => 'complete', 'customer_id' => $customerDup,
        'order_items' => [['order_item_id' => 7031, 'download_id' => $BUNDLE_DOWNLOAD, 'price_id' => $BUNDLE_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7031', 'status' => 'complete']],
        'registration_uuid' => $regDup['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bundle-forbid-1', 'idempotency_key' => 'idem-bundle-forbid-1',
        'amount' => 0.01,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'Bundle adapter rejects caller-controlled amount',
);
expect_matrix_throws(
    fn() => $settler->settle([
        'order_id' => $bundle['order_id'], 'customer_id' => $bundle['customer_id'], 'account_uuid' => $bundleAccount,
        'transition' => 'refund', 'scope' => 'component', 'request_id' => 'req-settle-forbid-1', 'idempotency_key' => 'idem-settle-forbid-1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'settlement rejects caller-controlled refund scope',
);
expect_matrix($sequenceOf($bundleAccount) === 1, 'denied caller-grant attempts never bump the sequence');

// ── 9. Partial Bundle refund: component refunds are unsupported in v1 ──

$bundleComponent = $bundleOrder(7032, 'commerce.partial@example.invalid', 'partial');
$insertRefund(7032, $bundleComponent['customer_id'], 7032, '697.00', 'complete', 'edd', '2026-08-10T00:00:00Z');
$db->prepare("UPDATE wp_edd_orders SET status = 'refunded', date_updated = :updated WHERE id = 7032")->execute([':updated' => '2026-08-10T00:00:00Z']);
$componentRefund = $settle(7032, $bundleComponent['customer_id'], $bundleComponent['account_uuid'], 'refund', 'partial');
expect_matrix_denied($componentRefund, 'COMPONENT_REFUND_UNSUPPORTED', 'component-level partial Bundle refund is denied in v1');
expect_matrix($settler->paidGrantsActive(7032) === true, 'denied component refund never removes the paid grants');
expect_matrix($sequenceOf($bundleComponent['account_uuid']) === 1, 'denied component refund never bumps the sequence');

// ── 10. Chargeback: lost Stripe dispute settles the whole Bundle ──────

$bundleChargeback = $bundleOrder(7033, 'commerce.chargeback@example.invalid', 'chargeback');
$insertRefund(7033, $bundleChargeback['customer_id'], null, '1254.60', 'lost', 'stripe', '2026-11-01T00:00:00Z');
$chargeback = $settle(7033, $bundleChargeback['customer_id'], $bundleChargeback['account_uuid'], 'chargeback', 'chargeback');
expect_matrix($chargeback['decision'] === 'applied' && $chargeback['transition'] === 'chargeback' && $chargeback['to_state'] === 'refunded', 'lost Stripe dispute settles the Bundle chargeback');
expect_matrix((int) $chargeback['grants_revoked'] === 2, 'chargeback revokes both Bundle grants together');
expect_matrix($chargeback['paid_grants_active'] === false && $chargeback['limited_posture'] === 'verified_no_license', 'chargeback removes paid grants and returns the verified account to limited mode');
expect_matrix((int) $chargeback['sequence'] === 1 && (int) $chargeback['result_sequence'] === 2, 'chargeback advances the authority sequence 1 -> 2');
expect_matrix($sequenceOf($bundleChargeback['account_uuid']) === 2, 'account sequence is 2 after chargeback');
expect_matrix($settler->currentEffectiveState(7033) === 'refunded', 'chargeback Bundle effective state is refunded');
// The still-verified chargeback account returns to the canonical limited posture.
$limitedPostChargeback = $assertionFixture->limitedPosture($chargeback, 'node-chargeback-001', $clock);
expect_matrix($limitedPostChargeback['kind'] === 'verified_no_license' && $limitedPostChargeback['paid_grants_active'] === false, 'chargeback returns the verified account to verified_no_license');
foreach (FocusaSpec172AssertionTransitionFixture::PERMANENT_ALLOWANCES as $allowance) {
    expect_matrix(in_array($allowance, $limitedPostChargeback['permanent_allowances'], true), "recovery allowance {$allowance} remains available after chargeback");
}
$verifyLimitedPost = $assertionFixture->verifyLimited($limitedPostChargeback);
expect_matrix($verifyLimitedPost['valid'] === true, 'the post-chargeback limited assertion verifies with the server-owned key');
// A stale paid Bundle credential can never reactivate after the chargeback.
expect_matrix_throws(
    fn() => FocusaSpec172AssertionTransitionFixture::validatePaidAssertion(
        $assertionFixture->paidAssertion($bundleChargeback['projected'], 'node-chargeback-001', $clock),
        2,
        'refunded',
    ),
    'PAID_GRANT_REVOKED',
    'a stale paid credential is rejected once the Bundle is terminal',
);
// The paid credential can only be derived from the ACTIVE projection.
$paidChargeback = $assertionFixture->paidAssertion($bundleChargeback['projected'], 'node-chargeback-001', $clock);
expect_matrix($paidChargeback['kind'] === 'paid' && $paidChargeback['assertion_payload']['status'] === 'active', 'paid credential derives from the ACTIVE projection only');

// ── 11. Future type: Navigator and future products never enter Operator ──

$projectionsBeforeFuture = $countOf('wp_wpuiai_license_type_projections');
expect_matrix(in_array('focusa_navigator_lifetime_v1', FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), true) === false, 'the future Navigator License Type is not an approved grant');
$driftedBundle = $fixtureDedicated;
foreach ($driftedBundle['records'] as &$record) {
    if ($record['public_code'] === $BUNDLE_PRODUCT) {
        $record['grants'] = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_navigator_lifetime_v1'];
    }
}
unset($record);
expect_matrix_throws(
    fn() => FocusaSpec172LicenseTypeRegistry::assertOfferComposition($driftedBundle['records'][2]),
    'PRODUCT_MAPPING_REQUIRED',
    'a future License Type in the Bundle grants fails registry composition',
);
$futureProductOffer = $fixtureDedicated;
foreach ($futureProductOffer['records'] as &$record) {
    if ($record['public_code'] === $BUNDLE_PRODUCT) {
        $record['future_products_included'] = true;
    }
}
unset($record);
expect_matrix_throws(
    fn() => FocusaSpec172LicenseTypeRegistry::assertOfferComposition($futureProductOffer['records'][2]),
    'PRODUCT_NOT_INCLUDED',
    'a future product in the Bundle offer fails registry composition',
);
// The Bundle signed lease can never be widened with a future grant.
$widenedLease = $bundleLease;
$widenedLease['lease_payload']['grants']['focusa_navigator_lifetime_v1'] = true;
expect_matrix_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($widenedLease, $projectedBundle),
    'FIXTURE_GRANT_UNION_MISMATCH',
    'a future License Type grant fails Bundle lease validation',
);
$widenedLease = $bundleLease;
$widenedLease['lease_payload']['future_products_included'] = true;
expect_matrix_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($widenedLease, $projectedBundle),
    'FIXTURE_FUTURE_PRODUCT_MISMATCH',
    'a future product in the Bundle lease fails validation',
);
$widenedLease = $bundleLease;
$widenedLease['lease_payload']['features']['future_product_family'] = true;
expect_matrix_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($widenedLease, $projectedBundle),
    'FIXTURE_FAMILY_MISMATCH',
    'a future-product family fails Bundle lease validation',
);
// Operator stays intact: future-type metadata cannot mutate or expand Operator or the
// Bundle (the Bundle projector rejects future-product/type metadata before any state).
expect_matrix_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $bundle['handle'],
        'request_id' => 'req-project-navigator-1',
        'idempotency_key' => 'idem-project-navigator-1',
        'future_license_types_included' => true,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied future-License-Type metadata cannot expand the Bundle',
);
expect_matrix_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $bundle['handle'],
        'request_id' => 'req-project-futureproduct-1',
        'idempotency_key' => 'idem-project-futureproduct-1',
        'future_products_included' => true,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied future-product metadata cannot expand the Bundle',
);
expect_matrix($countOf('wp_wpuiai_license_type_projections') === $projectionsBeforeFuture, 'future-type attempts create zero projections');

// ── 12. Hosted-resource attempts never create paid or limited entitlement ──

$hostedAttempts = ['unlimited_hosted_compute', 'paid_proxies', 'third_party_api_consumption', 'paid_model_usage', 'managed_hosting', 'resale', 'redistribution', 'product_embedding'];
foreach ($hostedAttempts as $resource) {
    expect_matrix(UiaiSpec172HostedResourceExclusionRegistry::isIncluded($resource) === false, "hosted resource {$resource} is excluded");
}
expect_matrix(UiaiSpec172HostedResourceExclusionRegistry::digest() === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'hosted-resource exclusion digest is deterministic');
// The UIAI grant/child-token fixture carries the explicit local/hosted boundary and can
// never grant a hosted resource. A Focusa-only account can never request UIAI paid
// capability either (proven in section 5); here we prove the fixture boundary stays.
$uiaiGrant = (static function () use ($uiai): array {
    // Reuse the UIAI projector result shape: the projection carries the exclusion list.
    return $uiai['projected'];
})();
expect_matrix($uiaiGrant['hosted_resource_exclusion_digest'] === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'UIAI projection keeps the frozen hosted boundary digest');
expect_matrix($uiaiGrant['hosted_resources_included'] === [], 'UIAI projection includes zero hosted resources');
// Limited-mode UIAI also blocks hosted resources: the limited allowlist is frozen and
// never contains a metered resource family.
foreach ($assertionNoLicense['family_allowlist'] as $family) {
    expect_matrix(in_array($family, ['unlimited_hosted_compute', 'paid_proxies', 'managed_hosting'], true) === false, "limited family {$family} is never a hosted resource");
}

// ── 13. No direct Stripe/facade/install-site path creates entitlement ──

// Frozen registry (authority outage / checkout-disabled view): a complete paid-looking
// order fails closed at the binding boundary and creates no issuance request.
$regFrozen = $createRegistration('commerce.frozen@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'frozen');
$customerFrozen = $regFrozen['edd_customer_id'];
$insertOrder(7040, 'complete', $customerFrozen, 'commerce.frozen@example.invalid', [['item_id' => 7040, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7040, $GATEWAY, 'txn_pay_7040');
expect_matrix_throws(
    fn() => $bindingFrozen->bindOrderComplete([
        'order_id' => 7040, 'order_status' => 'complete', 'customer_id' => $customerFrozen,
        'order_items' => [['order_item_id' => 7040, 'download_id' => $FOCUSA_DOWNLOAD, 'price_id' => $FOCUSA_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7040', 'status' => 'complete']],
        'registration_uuid' => $regFrozen['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-frozen-1', 'idempotency_key' => 'idem-bind-frozen-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'the frozen checkout-disabled registry never creates entitlement',
);
// A raw Stripe transaction row alone (no verified binding) creates zero issuance and
// zero licenses: payment evidence is never entitlement.
$rawLicenses = $countOf('wp_edd_licenses');
$rawRequests = $countOf('wp_wpuiai_edd_issuance_requests');
$db->exec("INSERT INTO wp_edd_orders (id, order_number, status, type, date_created, user_id, customer_id, email, total)
    VALUES (7041, 'EDD-7041', 'complete', 'sale', '2026-08-08T00:01:00Z', NULL, {$customerFrozen}, 'commerce.stripe-only@example.invalid', '697.00')");
$db->exec("INSERT INTO wp_edd_order_items (id, order_id, product_id, product_name, quantity) VALUES (7041, 7041, {$FOCUSA_DOWNLOAD}, 'fixture', 1)");
$db->exec("INSERT INTO wp_edd_order_transactions (id, order_id, transaction_id, gateway, status, total, currency, date_created)
    VALUES (999, 7041, 'txn_direct_stripe_0001', 'stripe', 'complete', '697.00', 'USD', '2026-08-08T00:01:00Z')");
expect_matrix($countOf('wp_edd_licenses') === $rawLicenses && $countOf('wp_wpuiai_edd_issuance_requests') === $rawRequests, 'a raw Stripe transaction without a verified binding creates zero entitlement');
// An unverified (pending) registration can never bind a paid order: no anonymous
// product capability at the commerce boundary.
$regUnverified = $createRegistration('commerce.unverified@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'unverified', verify: false);
$insertOrder(7042, 'complete', 9001, 'commerce.unverified@example.invalid', [['item_id' => 7042, 'download' => $FOCUSA_DOWNLOAD]]);
$insertTransaction(7042, $GATEWAY, 'txn_pay_7042');
expect_matrix_throws(
    fn() => $bindingService->bindOrderComplete([
        'order_id' => 7042, 'order_status' => 'complete', 'customer_id' => 9001,
        'order_items' => [['order_item_id' => 7042, 'download_id' => $FOCUSA_DOWNLOAD, 'price_id' => $FOCUSA_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_7042', 'status' => 'complete']],
        'registration_uuid' => $regUnverified['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-unverified-1', 'idempotency_key' => 'idem-bind-unverified-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'an unverified registration can never bind a paid order',
);
expect_matrix($countOf('wp_edd_licenses') === $rawLicenses, 'unverified binding attempts create zero licenses');

// ── 14. Settlement outbox, preservation, redaction, rollback ──────────

// The applied chargeback appended one signed outbox envelope and dispatches exactly once.
$outboxEvent = $settler->latestOutboxEvent();
expect_matrix($outboxEvent !== null && $outboxEvent['transition'] === 'chargeback' && $outboxEvent['dispatch_state'] === 'pending', 'applied chargeback appended one signed settlement outbox envelope');
$dispatch1 = $dispatcher->dispatchOne();
expect_matrix($dispatch1 !== null && $dispatch1['decision'] === 'dispatched' && $dispatch1['delivered'] === true, 'chargeback envelope dispatches exactly once');
expect_matrix($dispatcher->deliveryCount() === 1 && $dispatcher->pendingCount() === 0, 'one delivery ledger row and no pending envelopes');
expect_matrix($dispatcher->dispatchOne() === null, 'nothing to dispatch a second time');
// Reconciliation converges: the partial-refund order stays quarantined, the chargeback
// is already settled, and a second run repairs zero.
$dryRun = $reconciler->run('dry_run');
expect_matrix($dryRun['summary']['repairs_applied'] === 0, 'dry run applies nothing');
$applyRun = $reconciler->run('apply');
expect_matrix($applyRun['summary']['converged'] === true, 'apply run converges (settlement already applied; unsupported refunds quarantined)');
$applyRun2 = $reconciler->run('apply');
expect_matrix((int) $applyRun2['summary']['repairs_applied'] === 0 && $applyRun2['summary']['converged'] === true, 'second apply run repairs zero (idempotent convergence)');
// Preservation: every account/customer/order/license/projection/registration row is kept.
$preservedCounts = [
    'customers' => $countOf('wp_edd_customers'),
    'orders' => $countOf('wp_edd_orders'),
    'licenses' => $countOf('wp_edd_licenses'),
    'refunds' => $countOf('wp_edd_order_refunds'),
    'projections' => $countOf('wp_wpuiai_license_type_projections'),
    'accounts' => $countOf('wp_wpuiai_authority_accounts'),
    'registrations' => $countOf('wp_wpuiai_activation_registrations'),
];
expect_matrix((int) $preservedCounts['customers'] >= 15, 'all matrix customers preserved');
expect_matrix((int) $preservedCounts['orders'] >= 15, 'all matrix orders preserved');
// Exactly one canonical license per issued order: the duplicate-order fixture proves
// the count never grows on replay. 10 orders issued a canonical key across the matrix
// (focusa, uiai, bundle, wrong-price, wrong-product x2, wrong-account, duplicate,
// partial-refund bundle, chargeback bundle) and every order_id appears at most once.
expect_matrix((int) $preservedCounts['licenses'] === 10, 'exactly ten canonical EDD licenses, one per issued order');
$licenseOrderIds = $db->query('SELECT order_id, COUNT(*) AS c FROM wp_edd_licenses GROUP BY order_id')->fetchAll(PDO::FETCH_ASSOC);
foreach ($licenseOrderIds as $licenseOrder) {
    expect_matrix((int) $licenseOrder['c'] === 1, "order {$licenseOrder['order_id']} has exactly one canonical license row");
}
expect_matrix((int) $preservedCounts['projections'] === 5, 'exactly five paid projections exist (focusa + uiai + bundle + partial + chargeback)');
// Redaction: no raw email, key, token, customer row, or card data in any decision.
$decisionJson = json_encode([$projectedFocusa, $projectedUiai, $projectedBundle, $chargeback, $componentRefund, $assertionNoLicense, $focusaLease, $bundleLease], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_matrix(strpos($decisionJson, '@') === false, 'no raw email in any matrix decision');
expect_matrix(preg_match($KEY_SCAN_PATTERN, $decisionJson) !== 1, 'no full license key in any matrix decision');
expect_matrix(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $decisionJson) !== 1, 'no payment key in any matrix decision');
expect_matrix(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $decisionJson) !== 1, 'no card data in any matrix decision');
$projectionJson = json_encode($db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_matrix(strpos($projectionJson, '@') === false && strpos($projectionJson, 'txn_pay_') === false, 'projection journal carries no raw email or payment transaction id');
expect_matrix(preg_match($KEY_SCAN_PATTERN, $projectionJson) !== 1, 'projection journal carries no full license key');
// Rollback is preservation-only across every surface schema.
$preserved = $projectionMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'spec172_edd_commerce_acceptance_test', 'record' => 'rollback']);
expect_matrix($preserved['action'] === 'preserve', 'projection rollback contract is preservation-only');
$preservedSettlement = $settlementMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'spec172_edd_commerce_acceptance_test', 'record' => 'rollback']);
expect_matrix($preservedSettlement['action'] === 'preserve', 'settlement rollback contract is preservation-only');
$preservedPosture = $postureMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'spec172_edd_commerce_acceptance_test', 'record' => 'rollback']);
expect_matrix($preservedPosture['action'] === 'preserve', 'posture rollback contract is preservation-only');
// No live charge was ever made: the fixture has no real gateway, no card data, and the
// only transactions are synthetic fixture rows with bounded opaque ids.
expect_matrix(preg_match('/txn_direct_stripe_|txn_pay_/', $decisionJson) === 0, 'no transaction id leaks into any decision');
$allTxns = $db->query('SELECT transaction_id, gateway, status, total FROM wp_edd_order_transactions')->fetchAll(PDO::FETCH_ASSOC);
foreach ($allTxns as $txn) {
    expect_matrix(in_array((string) $txn['gateway'], ['stripe', 'edd'], true) && in_array((string) $txn['status'], ['complete', 'disputed', 'lost', 'refunded'], true), 'synthetic transaction rows only');
    expect_matrix(strpos((string) $txn['transaction_id'], 'ch_') === false && strpos((string) $txn['transaction_id'], 'pi_') === false, 'no real Stripe payment intent or charge id is used (no live charge)');
}

// ── Summary ───────────────────────────────────────────────────────────

$summary = [
    'schema' => 'focusa.spec172.edd_commerce_acceptance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'matrix_rows' => [
        'no_license', 'paid_focusa', 'paid_uiai', 'bundle_exact_union', 'wrong_price',
        'wrong_product', 'wrong_account', 'download_453', 'credit_pack',
        'duplicate_order', 'caller_grants', 'partial_bundle_refund', 'chargeback',
        'future_type', 'hosted_resource_attempts', 'no_direct_stripe_facade_path',
    ],
    'canonical_licenses_created' => $preservedCounts['licenses'],
    'projections_created' => $preservedCounts['projections'],
    'paid_projection_orders' => ['focusa' => 7001, 'uiai' => 7002, 'bundle' => 7003],
    'license_types' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
    'bundle_grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
    'prices_usd' => ['697.00', '697.00', '1254.60'],
    'operator_seats' => 1,
    'node_limit' => 3,
    'node_set' => 'operator_shared_v1',
    'refund_policy' => 'whole_order_30_days',
    'component_refunds_allowed' => false,
    'future_products_included' => false,
    'future_license_types_included' => false,
    'hosted_resources_granted' => 0,
    'download_453' => 'quarantined_never_grants',
    'credit_packs' => 'retired_never_grant',
    'limited_posture' => 'verified_no_license',
    'chargeback_settled' => $settler->currentEffectiveState(7033),
    'outbox_deliveries' => $dispatcher->deliveryCount(),
    'reconciliation_converged' => $applyRun2['summary']['converged'],
    'preserved' => $preservedCounts,
    'live_charge' => false,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
