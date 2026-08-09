<?php
// 172.02.06 Implement Bundle as one SKU/key and two underlying grants.
// The Bundle is ONE commerce SKU (`focusa_uiai_operator_bundle_lifetime_v1`) and ONE
// canonical EDD Software Licensing human key that resolves to the exact union of the
// two underlying Operator v1 License Types. The WPUIAI Bundle order/SL adapter binds
// one verified complete $1,254.60 Bundle order/item (exactly one item; two standalone
// items are never folded into a Bundle) and issues exactly ONE canonical human key. The
// composite Bundle projector consumes that issued key and projects the Bundle from
// canonical EDD truth (order row, order item, active canonical license with the exact
// journaled key digest, server-owned dedicated Downloads offer). The projection and the
// signed lease fixture carry BOTH underlying Operator v1 grants (exact union), the
// DERIVED twelve-family union digest (5 Focusa + 7 UIAI, never a third hand-copied
// list), one operator seat, the SAME three shared operator_shared_v1 node identities
// (never six unrelated activations), the server-owned 1254.60 USD price version, one
// human key, whole-order refunds, and zero future products. Wrong product (standalone
// Focusa or UIAI orders can never project the Bundle), wrong price, wrong account,
// refunded/revoked/pending orders, unissued requests, revoked/tampered licenses,
// checkout-disabled/drifted/frozen offers, non-exact-union bundle offers, caller
// metadata grants, and idempotency conflicts create zero projections and never advance
// the authority sequence. No raw email, key, token, customer row, credential, or card
// data is stored or returned anywhere.
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

$positiveChecks = 0;
$negativeChecks = 0;

function expect_bundle(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_bundle_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'bundle_composition_test']);

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

// The frozen contracts are used by the fail-closed service instances. The fixture
// registry and fixture dedicated-downloads contract add explicitly operator-approved
// test mappings (download 1001 -> focusa_operator_lifetime_v1, download 1002 ->
// uiai_operator_lifetime_v1, and download 1003 ->
// focusa_uiai_operator_bundle_lifetime_v1, all active/checkout_enabled at the fixed
// canonical prices) so positive and wrong-product paths are exercised without mutating
// the frozen contracts.
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

// Checkout-disabled fixture: the dedicated Bundle offer resolves (download 1003) but
// checkout is not enabled, so Bundle projection fails closed with EDD_CHECKOUT_REQUIRED.
$blockedDedicated = $fixtureDedicated;
foreach ($blockedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $record['checkout_enabled'] = false;
        $record['sale_status'] = 'approved_not_yet_enabled';
    }
}
unset($record);
$bundleProjectorBlocked = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $blockedDedicated, $clock,
);

// Mutated fixture: the bundle offer's download mapping no longer matches the settled
// item (PRODUCT_MAPPING_REQUIRED at projection).
$mismatchedDedicated = $fixtureDedicated;
foreach ($mismatchedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $record['edd_download_id'] = 9999;
    }
}
unset($record);
$bundleProjectorMismatched = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $mismatchedDedicated, $clock,
);

// Non-exact-union fixture: the bundle offer's grants are NOT the exact two Operator
// records (a future License Type was appended) — fails closed as a mapping error.
$nonUnionDedicated = $fixtureDedicated;
foreach ($nonUnionDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $record['grants'] = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_navigator_lifetime_v1'];
    }
}
unset($record);
$bundleProjectorNonUnion = new FocusaSpec172BundleOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $nonUnionDedicated, $clock,
);

// The prior atoms' projectors are reused as the cross-product guards: a Bundle license
// can never project focusa_operator_lifetime_v1 or uiai_operator_lifetime_v1 and vice
// versa.
$focusaProjector = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);
$uiaiProjector = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true, bool $checkout = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.bundle.composition.test',
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
        'migration_provenance' => ['source' => 'spec172_candidate', 'record' => 'bundle-' . $tag . '-' . $seq],
    ]);
    if (!$checkout) {
        return ['registration_uuid' => $uuid];
    }
    // Legal state-machine path to the paid checkout state.
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'account_promoted', 'offer_selected', (int) $row['state_version'], 'req-offer-' . $tag . '-' . $seq, 'idem-offer-' . $tag . '-' . $seq, ['state_reason' => 'offer_selected_for_checkout', 'offer_code' => $product]);
    $row = $registrations->findByUuid($uuid);
    $registrations->transition($uuid, 'offer_selected', 'checkout_pending', (int) $row['state_version'], 'req-checkout-' . $tag . '-' . $seq, 'idem-checkout-' . $tag . '-' . $seq, ['state_reason' => 'checkout_pending', 'edd_cart_reference' => 'cart-' . $tag . '-' . $seq]);
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

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$BUNDLE_PRODUCT = 'focusa_uiai_operator_bundle_lifetime_v1';
$FOCUSA_PRODUCT = 'focusa_operator_lifetime_v1';
$UIAI_PRODUCT = 'uiai_operator_lifetime_v1';
$BUNDLE_DOWNLOAD = 1003;
$FOCUSA_DOWNLOAD = 1001;
$UIAI_DOWNLOAD = 1002;
$BUNDLE_PRICE = 'price_bundle_op_v1';
$FOCUSA_PRICE = 'price_focusa_op_v1';
$UIAI_PRICE = 'price_uiai_op_v1';
$GATEWAY = 'stripe';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

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

$issue = static function (string $handle, string $requestId, string $idempotencyKey) use ($issuanceService): array {
    return $issuanceService->issue([
        'issuance_request_handle' => $handle,
        'request_id' => $requestId,
        'idempotency_key' => $idempotencyKey,
    ]);
};

// The WPUIAI Bundle order/SL adapter: one verified $1,254.60 Bundle order/item -> one
// canonical EDD human key carrying both underlying Operator v1 grants.
$bundleBindAndIssue = static function (int $orderId, string $registrationUuid, int $customerId, array $items, string $txn, string $tag) use ($adapter, $FACADE, $ORIGIN, $GATEWAY): array {
    return $adapter->bindAndIssue([
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
        'request_id' => 'req-bind-bundle-' . $tag,
        'idempotency_key' => 'idem-bind-bundle-' . $tag,
    ]);
};

$bundleProject = static function (string $handle, string $requestId, string $idempotencyKey) use ($bundleProjector): array {
    return $bundleProjector->project([
        'issuance_request_handle' => $handle,
        'request_id' => $requestId,
        'idempotency_key' => $idempotencyKey,
    ]);
};

$projectionCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_license_type_projections')->fetchColumn();
};
$licenseCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};
$accountSequence = static function (string $registrationUuid) use ($registrations, $accounts): int {
    $accountUuid = (string) $registrations->findByUuid($registrationUuid)['account_uuid'];
    return (int) $accounts->findByUuid($accountUuid)['highest_entitlement_sequence'];
};

// ── Frozen contracts and License Type registry remain canonical ────────

expect_bundle($frozenDedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1', 'frozen dedicated downloads schema');
expect_bundle($frozenDedicated['owner'] === 'WPUIAI/wpuiai', 'frozen dedicated downloads owner');
expect_bundle(count($frozenDedicated['records']) === 3, 'frozen dedicated downloads has three records');
expect_bundle($frozenDedicated['counts']['checkout_enabled'] === 0, 'frozen dedicated downloads checkout disabled');
$frozenBundle = null;
foreach ($frozenDedicated['records'] as $record) {
    if ($record['public_code'] === 'focusa_uiai_operator_bundle_lifetime_v1') {
        $frozenBundle = $record;
    }
}
expect_bundle($frozenBundle !== null, 'frozen dedicated downloads has the bundle record');
expect_bundle((int) $frozenBundle['amount_minor'] === 125460 && $frozenBundle['price_usd'] === '1254.60', 'frozen bundle offer is 1254.60');
expect_bundle($frozenBundle['composite_sku_ref'] === $frozenBundle['public_code'] && $frozenBundle['composite_sku_ref'] === 'focusa_uiai_operator_bundle_lifetime_v1', 'frozen bundle offer is one composite SKU');
expect_bundle($frozenBundle['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'frozen bundle grants exactly the two underlying License Types');
expect_bundle($frozenBundle['grant_composition'] === 'exact_union', 'frozen bundle composition is exact_union');
expect_bundle($frozenBundle['component_refunds_allowed'] === false, 'frozen bundle refunds are whole-order only');
expect_bundle($frozenBundle['products'] === ['focusa', 'uiai_engine'], 'frozen bundle product scope is exactly focusa and uiai_engine');
expect_bundle($frozenBundle['future_products_included'] === false && $frozenBundle['future_license_types_included'] === false, 'frozen bundle includes no future product');
expect_bundle((int) $frozenBundle['operator_seats'] === 1 && (int) $frozenBundle['node_limit'] === 3 && $frozenBundle['node_set'] === 'operator_shared_v1', 'frozen bundle one seat three shared nodes');
expect_bundle($frozenBundle['license_duration'] === 'lifetime', 'frozen bundle offer lifetime');

// License Type registry: one SKU, two underlying grants, derived family union.
expect_bundle(FocusaSpec172LicenseTypeRegistry::SCHEMA === 'focusa.spec172.license_types.v1', 'license type registry schema is canonical');
expect_bundle(FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes() === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'registry grants exactly the two underlying Operator v1 License Types');
expect_bundle(FocusaSpec172LicenseTypeRegistry::BUNDLE_AMOUNT_MINOR === 125460 && FocusaSpec172LicenseTypeRegistry::BUNDLE_PRICE_USD === '1254.60', 'registry bundle price is the canonical 1254.60');
expect_bundle(FocusaSpec172LicenseTypeRegistry::priceVersion() === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'registry price version is canonical');
expect_bundle(FocusaSpec172LicenseTypeRegistry::OPERATOR_SEATS === 1 && FocusaSpec172LicenseTypeRegistry::NODE_LIMIT === 3 && FocusaSpec172LicenseTypeRegistry::NODE_SET === 'operator_shared_v1', 'registry one seat and three shared nodes');
expect_bundle(FocusaSpec172LicenseTypeRegistry::FUTURE_PRODUCTS_INCLUDED === false && FocusaSpec172LicenseTypeRegistry::FUTURE_LICENSE_TYPES_INCLUDED === false && FocusaSpec172LicenseTypeRegistry::COMPONENT_REFUNDS_ALLOWED === false, 'registry excludes future products and component refunds');
// The Bundle family set is DERIVED from the two underlying records — never a third
// hand-copied family list.
expect_bundle(FocusaSpec172LicenseTypeRegistry::focusaFamilies() === FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES, 'bundle focusa families ARE the underlying Focusa record');
expect_bundle(FocusaSpec172LicenseTypeRegistry::uiaiFamilies() === UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES, 'bundle uiai families ARE the underlying UIAI record');
expect_bundle(FocusaSpec172LicenseTypeRegistry::underlyingFamilies() === array_merge(FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES, UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES), 'bundle family union is the exact merge of the two underlying records');
expect_bundle(FocusaSpec172LicenseTypeRegistry::familySets() === ['focusa' => FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES, 'uiai_engine' => UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES], 'bundle family sets are per underlying product');
expect_bundle(count(FocusaSpec172LicenseTypeRegistry::focusaFamilies()) === 5 && count(FocusaSpec172LicenseTypeRegistry::uiaiFamilies()) === 7 && count(FocusaSpec172LicenseTypeRegistry::underlyingFamilies()) === 12, 'bundle union is 5 Focusa + 7 UIAI = 12 families');
expect_bundle(FocusaSpec172LicenseTypeRegistry::familyDigest() !== '', 'bundle family digest is deterministic');
expect_bundle(FocusaSpec172LicenseTypeRegistry::familyDigest() === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'bundle family digest is stable across calls');
// The registry's fail-closed composition check accepts the frozen exact-union offer and
// rejects a future-License-Type drift and a future product.
expect_bundle((static function () use ($frozenBundle): bool {
    FocusaSpec172LicenseTypeRegistry::assertOfferComposition($frozenBundle);
    return true;
})(), 'registry accepts the frozen exact-union bundle offer');
$driftedGrants = $frozenBundle;
$driftedGrants['grants'] = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_navigator_lifetime_v1'];
expect_bundle_throws(
    fn() => FocusaSpec172LicenseTypeRegistry::assertOfferComposition($driftedGrants),
    'PRODUCT_MAPPING_REQUIRED',
    'a future License Type in the bundle grants fails registry composition',
);
$futureProductOffer = $frozenBundle;
$futureProductOffer['future_products_included'] = true;
expect_bundle_throws(
    fn() => FocusaSpec172LicenseTypeRegistry::assertOfferComposition($futureProductOffer),
    'PRODUCT_NOT_INCLUDED',
    'a future product in the bundle offer fails registry composition',
);

// ── Positive: one Bundle order -> one adapter key -> one composite projection ──

$regA = $createRegistration('operator.bundle@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundle-alpha');
$customerA = $customerOf($regA['registration_uuid']);
$insertOrder(6001, 'complete', $customerA, 'operator.bundle@example.invalid', [
    ['item_id' => 6001, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6001, $GATEWAY, 'txn_pay_6001');

$boundA = $bundleBindAndIssue(6001, $regA['registration_uuid'], $customerA, [['item_id' => 6001, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6001', 'alpha');
expect_bundle($boundA['decision'] === 'bundle_bound_and_issued', 'Bundle adapter binds and issues the Bundle');
expect_bundle($boundA['human_key_count'] === 1, 'Bundle uses exactly one canonical human key');
expect_bundle($boundA['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'adapter carries both underlying Operator v1 grants');
expect_bundle($boundA['grant_composition'] === 'exact_union' && $boundA['sku'] === $BUNDLE_PRODUCT, 'adapter carries the exact-union composite SKU');
expect_bundle($boundA['price_usd'] === '1254.60' && $boundA['amount_minor'] === 125460, 'adapter binds the canonical 1254.60 price');
$handleA = $boundA['issuance_request_handle'];
expect_bundle(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $handleA) === 1, 'Bundle issuance request handle is an opaque bounded token');
$keyA = $boundA['delivery']['license_key'] ?? '';
expect_bundle(preg_match($KEY_PATTERN, (string) $keyA) === 1, 'Bundle issues one canonical EDD SL human key');
expect_bundle(str_starts_with((string) $keyA, 'focusa_live_') === false, 'Bundle key is never a synthetic key');
expect_bundle($boundA['edd_license_id'] > 0, 'Bundle key references a canonical EDD license row');

$projectedA = $bundleProject($handleA, 'req-project-bundle-alpha-1', 'idem-project-bundle-alpha-1');
expect_bundle($projectedA['schema'] === 'focusa.spec172.bundle_operator_lifetime_projection.v1', 'bundle projection schema is canonical');
expect_bundle($projectedA['decision'] === 'license_type_projected', 'bundle projection decision is license_type_projected');
expect_bundle($projectedA['existing'] === false && $projectedA['projections_created'] === 1, 'first bundle projection creates exactly one projection');
expect_bundle($projectedA['sku'] === $BUNDLE_PRODUCT && $projectedA['license_type'] === $BUNDLE_PRODUCT && $projectedA['grant'] === $BUNDLE_PRODUCT, 'bundle projection carries the one composite SKU');
expect_bundle($projectedA['products'] === ['focusa', 'uiai_engine'], 'bundle projection product scope is exactly focusa and uiai_engine');
expect_bundle($projectedA['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle semantic grants equal the exact union of both Operator records');
expect_bundle($projectedA['grants_union'] === 'exact_union', 'bundle grant composition is the exact union');
expect_bundle($projectedA['human_key_count'] === 1, 'bundle projection carries one human key');
expect_bundle($projectedA['registration_id'] === $regA['registration_uuid'], 'bundle projection is linked to the registration');
expect_bundle($projectedA['account_id'] !== '', 'bundle projection is linked to the account');
expect_bundle($projectedA['customer_id'] === $customerA, 'bundle projection is linked to the EDD customer');
expect_bundle($projectedA['order_id'] === 6001 && $projectedA['order_item_id'] === 6001, 'bundle projection is linked to the canonical order item');
expect_bundle($projectedA['download_id'] === $BUNDLE_DOWNLOAD, 'bundle projection carries the canonical bundle download');
expect_bundle(is_int($projectedA['edd_license_id']) && $projectedA['edd_license_id'] === $boundA['edd_license_id'], 'bundle projection references the one canonical issued license');
expect_bundle($projectedA['issuance'] === 'canonical_edd_software_licensing', 'bundle projection derives from canonical EDD Software Licensing');
expect_bundle($projectedA['license_key_digest'] === $boundA['license_key_digest'], 'bundle projection carries the keyed license digest');
expect_bundle(strpos((string) $projectedA['license_key_mask'], '********-********-********-') === 0, 'bundle projection carries only a masked license key');
expect_bundle($projectedA['price_version'] === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'bundle projection carries the server-owned price version');
expect_bundle($projectedA['price_usd'] === '1254.60' && $projectedA['amount_minor'] === 125460, 'bundle projection carries the canonical price');
// Derived union families and digest — never a third hand-copied family list.
expect_bundle($projectedA['family_digest'] === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'bundle projection carries the derived union family digest');
expect_bundle($projectedA['family_count'] === 12, 'bundle projection freezes the 12-family union');
expect_bundle($projectedA['families'] === FocusaSpec172LicenseTypeRegistry::underlyingFamilies(), 'bundle projection family list IS the derived union of both Operator records');
expect_bundle($projectedA['family_sets'] === FocusaSpec172LicenseTypeRegistry::familySets(), 'bundle projection carries the per-product family sets');
// Three shared node identities for both products — never six unrelated activations.
expect_bundle($projectedA['operator_seats'] === 1, 'bundle projection freezes one operator seat');
expect_bundle($projectedA['node_limit'] === 3 && $projectedA['node_set'] === 'operator_shared_v1', 'bundle projection freezes three shared operator nodes');
expect_bundle($projectedA['node_set'] === $frozenBundle['node_set'] && (int) $projectedA['node_limit'] === (int) $frozenBundle['node_limit'], 'bundle shares the same node set as the dedicated offer');
expect_bundle($projectedA['term'] === 'lifetime' && $projectedA['status'] === 'active', 'bundle projection is an active lifetime grant');
expect_bundle(is_int($projectedA['sequence']) && $projectedA['sequence'] === 1, 'bundle projection carries the first monotonic sequence');
expect_bundle($accountSequence($regA['registration_uuid']) === 1, 'authority account sequence advanced to 1');
expect_bundle($projectedA['component_refunds_allowed'] === false, 'bundle projection is whole-order refund only');
expect_bundle($projectedA['future_products_included'] === false && $projectedA['future_license_types_included'] === false, 'bundle projection contains no additional/future product');

expect_bundle($projectionCount() === 1, 'exactly one bundle projection journal row');
$projectionRowA = $bundleProjector->findByIssuanceRequestKey($handleA);
expect_bundle($projectionRowA !== null && $projectionRowA['status'] === 'active', 'bundle projection journal row is active');
expect_bundle(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRowA['projection_key']) === 1, 'bundle projection handles are opaque bounded tokens');
expect_bundle($projectionRowA['product_code'] === $BUNDLE_PRODUCT && $projectionRowA['license_type_ref'] === $BUNDLE_PRODUCT, 'bundle projection journal carries the one composite SKU');
expect_bundle($projectionRowA['price_version'] === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'bundle projection journal carries the price version');
expect_bundle($projectionRowA['family_digest'] === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'bundle projection journal carries the derived union digest');
expect_bundle((int) $projectionRowA['operator_seats'] === 1 && (int) $projectionRowA['node_limit'] === 3 && $projectionRowA['node_set'] === 'operator_shared_v1', 'bundle projection journal carries seat and node limits');
expect_bundle((int) $projectionRowA['sequence'] === 1, 'bundle projection journal carries the sequence');
expect_bundle($bundleProjector->findByProjectionKey((string) $projectionRowA['projection_key'])['issuance_request_key'] === $handleA, 'bundle projection lookup by handle resolves the source request');

// Registration fulfillment (from SL issuance) is preserved: entitlement_issued.
$regRowA = $registrations->findByUuid($regA['registration_uuid']);
expect_bundle($regRowA['state'] === 'entitlement_issued', 'bundle registration is at entitlement_issued');
expect_bundle((int) $regRowA['edd_license_id'] === $boundA['edd_license_id'], 'bundle registration references the one canonical issued license');

// Idempotent replay: same key returns the identical decision, no second projection.
$replayedA = $bundleProject($handleA, 'req-project-bundle-alpha-1', 'idem-project-bundle-alpha-1');
expect_bundle(json_encode($replayedA, JSON_THROW_ON_ERROR) === json_encode($projectedA, JSON_THROW_ON_ERROR), 'bundle idempotency replay returns the identical decision');
expect_bundle($projectionCount() === 1, 'bundle replay creates no second projection row');
expect_bundle($accountSequence($regA['registration_uuid']) === 1, 'bundle replay does not bump the sequence');

// Duplicate projection call with a NEW idempotency key: same projection, zero new.
$duplicateA = $bundleProject($handleA, 'req-project-bundle-alpha-retry-1', 'idem-project-bundle-alpha-retry-1');
expect_bundle($duplicateA['existing'] === true, 'duplicate bundle projection call is an existing projection');
expect_bundle($duplicateA['projections_created'] === 0, 'duplicate bundle projection call creates zero projections');
expect_bundle($duplicateA['edd_license_id'] === $boundA['edd_license_id'], 'duplicate bundle projection call returns the same license reference');
expect_bundle($duplicateA['grants'] === $projectedA['grants'] && $duplicateA['sequence'] === 1 && $duplicateA['family_digest'] === $projectedA['family_digest'], 'duplicate bundle projection call returns the identical grant');
expect_bundle($projectionCount() === 1, 'duplicate bundle projection call never creates a second projection');
expect_bundle($accountSequence($regA['registration_uuid']) === 1, 'duplicate bundle projection call never bumps the sequence');

// ── Bundle signed lease claims derive from the composite projection ────

$fixtureA = FocusaSpec172BundleSignedLeaseFixture::fromProjection($projectedA, 'node-bundle-001', $clock);
expect_bundle($fixtureA['schema'] === 'focusa.spec172.bundle_signed_lease_fixture.v1', 'bundle lease fixture schema is canonical');
$leaseA = $fixtureA['lease_payload'];
expect_bundle($leaseA['schema'] === 'focusa.bundle_signed_lease.v1', 'bundle lease payload schema is canonical');
expect_bundle($leaseA['product'] === $BUNDLE_PRODUCT, 'bundle lease payload product is the one SKU');
expect_bundle($leaseA['subject_id'] === $projectedA['account_id'], 'bundle lease payload subject is the projected account');
expect_bundle($leaseA['node_id'] === 'node-bundle-001', 'bundle lease payload binds the operator node');
expect_bundle((int) $leaseA['sequence'] === 1, 'bundle lease payload carries the projected sequence');
expect_bundle($leaseA['status'] === 'active', 'bundle lease payload is active for this sequence');
expect_bundle($leaseA['authority_key_id'] !== '', 'bundle lease payload names the authority signing key');
$leaseGrantCodes = array_keys((array) $leaseA['grants']);
sort($leaseGrantCodes, SORT_STRING);
expect_bundle($leaseGrantCodes === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle lease claims carry exactly the two underlying Operator grants');
foreach ((array) $leaseA['grants'] as $code => $granted) {
    expect_bundle($granted === true && in_array($code, FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), true), "bundle lease grant {$code} is enabled and underlying");
}
$leaseFamilies = array_keys((array) $leaseA['features']);
sort($leaseFamilies, SORT_STRING);
$expectedFamilies = FocusaSpec172LicenseTypeRegistry::underlyingFamilies();
sort($expectedFamilies, SORT_STRING);
expect_bundle($leaseFamilies === $expectedFamilies, 'bundle lease features are exactly the derived 12-family union');
foreach ((array) $leaseA['features'] as $family => $enabled) {
    expect_bundle($enabled === true, "bundle lease family {$family} is enabled");
}
expect_bundle((array) $leaseA['family_sets'] === FocusaSpec172LicenseTypeRegistry::familySets(), 'bundle lease carries the per-product family sets');
expect_bundle((int) $leaseA['limits']['operator_seats'] === 1 && (int) $leaseA['limits']['node_limit'] === 3, 'bundle lease carries one seat and three nodes');
expect_bundle($leaseA['node_set'] === 'operator_shared_v1', 'bundle lease uses the shared three-node set, never six activations');
expect_bundle((int) $leaseA['human_key_count'] === 1, 'bundle lease carries one human key');
expect_bundle($leaseA['future_products_included'] === false && $leaseA['future_license_types_included'] === false, 'bundle lease excludes future products');
expect_bundle($leaseA['component_refunds_allowed'] === false, 'bundle lease is whole-order refund only');
expect_bundle((string) $leaseA['expires_at'] > (string) $leaseA['issued_at'], 'bundle lease credential lifetime is bounded (never perpetual)');
expect_bundle((string) $leaseA['offline_grace_until'] > (string) $leaseA['expires_at'], 'bundle lease offline grace is bounded past the refresh window');
$leaseIssued = new DateTimeImmutable($leaseA['issued_at'], new DateTimeZone('UTC'));
$leaseExpires = new DateTimeImmutable($leaseA['expires_at'], new DateTimeZone('UTC'));
$leaseGrace = new DateTimeImmutable($leaseA['offline_grace_until'], new DateTimeZone('UTC'));
expect_bundle($leaseExpires <= $leaseIssued->modify('+90 days'), 'bundle lease refresh window never exceeds 90 days');
expect_bundle($leaseGrace <= $leaseExpires->modify('+30 days'), 'bundle lease offline grace never exceeds 30 days past refresh');
$metaA = $fixtureA['grant_metadata'];
expect_bundle($metaA['license_type'] === $BUNDLE_PRODUCT && $metaA['price_version'] === 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1', 'bundle lease fixture carries explicit grant metadata');
expect_bundle($metaA['family_digest'] === FocusaSpec172LicenseTypeRegistry::familyDigest(), 'bundle lease fixture carries the derived union digest');
expect_bundle($metaA['term'] === 'lifetime' && $metaA['node_set'] === 'operator_shared_v1', 'bundle lease fixture carries lifetime term and shared node set');
expect_bundle($metaA['refund_policy'] === 'whole_order_30_days', 'bundle lease fixture carries the whole-order refund policy');
expect_bundle($metaA['grants'] === FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), 'bundle lease fixture carries the exact two underlying grants');

// validate() accepts the derived fixture and rejects tampering.
$validated = FocusaSpec172BundleSignedLeaseFixture::validate($fixtureA, $projectedA);
expect_bundle($validated === null, 'bundle lease fixture validation passes for the derived fixture');
$tamperedFixture = $fixtureA;
unset($tamperedFixture['lease_payload']['grants']['uiai_operator_lifetime_v1']);
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_UNION_MISMATCH',
    'a missing underlying grant fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['grants']['focusa_navigator_lifetime_v1'] = true;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_UNION_MISMATCH',
    'a future License Type grant fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant_metadata']['grants'] = ['focusa_operator_lifetime_v1'];
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_UNION_MISMATCH',
    'a truncated metadata grant list fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['features']['future_product_family'] = true;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_FAMILY_MISMATCH',
    'a future-product family fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['limits']['node_limit'] = 99;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_LIMIT_MISMATCH',
    'tampered node limit fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant_metadata']['family_digest'] = str_repeat('0', 64);
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_MISMATCH',
    'tampered family digest fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['human_key_count'] = 2;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_HUMAN_KEY_MISMATCH',
    'a second human key fails bundle lease validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['future_products_included'] = true;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_FUTURE_PRODUCT_MISMATCH',
    'a future product in the bundle lease fails validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['component_refunds_allowed'] = true;
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_COMPONENT_REFUND_MISMATCH',
    'component refunds in the bundle lease fail validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['offline_grace_until'] = $tamperedFixture['lease_payload']['expires_at'];
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_CREDENTIAL_WINDOW_INVALID',
    'a collapsed credential window fails bundle lease validation',
);
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::fromProjection($boundA, 'node-bundle-001', $clock),
    'LICENSE_TYPE_PROJECTION_REQUIRED',
    'the bundle lease fixture requires exactly an accepted composite projection, never an adapter decision',
);
expect_bundle_throws(
    fn() => FocusaSpec172BundleSignedLeaseFixture::fromProjection($projectedA, 'node@raw.example', $clock),
    'bounded node id required',
    'raw email node ids are rejected in the bundle lease fixture',
);

// ── Negative: wrong product creates no Bundle projection ───────────────

// A fully eligible standalone Focusa order issues its own canonical Focusa key (generic
// SL issuance) but can never project the Bundle: Focusa-only orders cannot obtain the
// Bundle SKU.
$regFocusa = $createRegistration('operator.focusabundle@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'focusabundle');
$customerFocusa = $customerOf($regFocusa['registration_uuid']);
$insertOrder(6002, 'complete', $customerFocusa, 'operator.focusabundle@example.invalid', [
    ['item_id' => 6002, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(6002, $GATEWAY, 'txn_pay_6002');
$boundFocusa = $bind(6002, $regFocusa['registration_uuid'], $customerFocusa, [['item_id' => 6002, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_6002', 'focusabundle-1');
expect_bundle($boundFocusa['issuance_requests_settled'] === 1, 'Focusa order settles its own issuance request');
$handleFocusa = $boundFocusa['protected_items'][0]['issuance_request_handle'];
$issuedFocusa = $issue($handleFocusa, 'req-issue-focusabundle-1', 'idem-issue-focusabundle-1');
expect_bundle($issuedFocusa['keys_created'] === 1 && $issuedFocusa['license_type_ref'] === $FOCUSA_PRODUCT, 'Focusa item issues its own canonical Focusa key');
expect_bundle_throws(
    fn() => $bundleProject($handleFocusa, 'req-project-bundle-from-focusa-1', 'idem-project-bundle-from-focusa-1'),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a Focusa license can never project the Bundle',
);
expect_bundle($projectionCount() === 1, 'Focusa denial creates zero Bundle projections');
expect_bundle($accountSequence($regFocusa['registration_uuid']) === 0, 'Focusa denial never advances the account sequence');

// A fully eligible standalone UIAI order can never project the Bundle either.
$regUiai = $createRegistration('operator.uiaibundle@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaibundle');
$customerUiai = $customerOf($regUiai['registration_uuid']);
$insertOrder(6003, 'complete', $customerUiai, 'operator.uiaibundle@example.invalid', [
    ['item_id' => 6003, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(6003, $GATEWAY, 'txn_pay_6003');
$boundUiai = $bind(6003, $regUiai['registration_uuid'], $customerUiai, [['item_id' => 6003, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_6003', 'uiaibundle-1');
expect_bundle($boundUiai['issuance_requests_settled'] === 1, 'UIAI order settles its own issuance request');
$handleUiai = $boundUiai['protected_items'][0]['issuance_request_handle'];
$issuedUiai = $issue($handleUiai, 'req-issue-uiaibundle-1', 'idem-issue-uiaibundle-1');
expect_bundle($issuedUiai['keys_created'] === 1 && $issuedUiai['license_type_ref'] === $UIAI_PRODUCT, 'UIAI item issues its own canonical UIAI key');
expect_bundle_throws(
    fn() => $bundleProject($handleUiai, 'req-project-bundle-from-uiai-1', 'idem-project-bundle-from-uiai-1'),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a UIAI license can never project the Bundle',
);
expect_bundle($projectionCount() === 1, 'UIAI denial creates zero Bundle projections');

// Cross-product guard (reusing the prior atoms' projectors): a Bundle license can never
// project focusa_operator_lifetime_v1 or uiai_operator_lifetime_v1. An issued-but-
// not-yet-projected Bundle handle fails closed in both standalone projectors with
// LICENSE_TYPE_NOT_INCLUDED.
$regBundleFresh = $createRegistration('operator.bundlefresh@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlefresh');
$customerBundleFresh = $customerOf($regBundleFresh['registration_uuid']);
$insertOrder(6021, 'complete', $customerBundleFresh, 'operator.bundlefresh@example.invalid', [
    ['item_id' => 6021, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6021, $GATEWAY, 'txn_pay_6021');
$boundBundleFresh = $bind(6021, $regBundleFresh['registration_uuid'], $customerBundleFresh, [['item_id' => 6021, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6021', 'bundlefresh-1');
expect_bundle($boundBundleFresh['issuance_requests_settled'] === 1, 'fresh Bundle order settles its own issuance request');
$handleBundleFresh = $boundBundleFresh['protected_items'][0]['issuance_request_handle'];
$issuedBundleFresh = $issue($handleBundleFresh, 'req-issue-bundlefresh-1', 'idem-issue-bundlefresh-1');
expect_bundle($issuedBundleFresh['keys_created'] === 1 && $issuedBundleFresh['license_type_ref'] === '', 'fresh Bundle item issues its one canonical key');
expect_bundle_throws(
    fn() => $focusaProjector->project([
        'issuance_request_handle' => $handleBundleFresh,
        'request_id' => 'req-project-focusa-from-bundle-1',
        'idempotency_key' => 'idem-project-focusa-from-bundle-1',
    ]),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a Bundle license can never project focusa_operator_lifetime_v1',
);
expect_bundle_throws(
    fn() => $uiaiProjector->project([
        'issuance_request_handle' => $handleBundleFresh,
        'request_id' => 'req-project-uiai-from-bundle-1',
        'idempotency_key' => 'idem-project-uiai-from-bundle-1',
    ]),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a Bundle license can never project uiai_operator_lifetime_v1',
);
expect_bundle($projectionCount() === 1, 'cross-product denials create zero projections');

// The shared projection journal never fabricates a standalone grant: replaying the
// already-projected Bundle handle through the Focusa projector returns the existing
// Bundle projection (product focusa_uiai_operator_bundle_lifetime_v1), never a Focusa
// grant, and creates no second row.
$focusaReplayOfBundle = $focusaProjector->project([
    'issuance_request_handle' => $handleA,
    'request_id' => 'req-project-focusa-from-bundle-2',
    'idempotency_key' => 'idem-project-focusa-from-bundle-2',
]);
expect_bundle($focusaReplayOfBundle['existing'] === true, 'shared journal replay marks the bundle projection existing');
expect_bundle(($focusaReplayOfBundle['product'] ?? '') === $BUNDLE_PRODUCT && ($focusaReplayOfBundle['license_type'] ?? '') === $BUNDLE_PRODUCT, 'replay returns the existing Bundle projection, never a Focusa grant');
expect_bundle($projectionCount() === 1, 'replay creates no second projection row');

// ── Negative: Bundle adapter shape and product guards ──────────────────

// Two standalone items in one order are NEVER folded into a Bundle: the Bundle is one
// SKU, one order item, one human key.
$regTwoItem = $createRegistration('operator.twoitem@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'twoitem');
$customerTwoItem = $customerOf($regTwoItem['registration_uuid']);
$insertOrder(6011, 'complete', $customerTwoItem, 'operator.twoitem@example.invalid', [
    ['item_id' => 6011, 'download' => $FOCUSA_DOWNLOAD],
    ['item_id' => 6012, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(6011, $GATEWAY, 'txn_pay_6011');
expect_bundle_throws(
    fn() => $bundleBindAndIssue(6011, $regTwoItem['registration_uuid'], $customerTwoItem, [
        ['item_id' => 6011, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE],
        ['item_id' => 6012, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE],
    ], 'txn_pay_6011', 'twoitem-1'),
    'BUNDLE_ITEM_COUNT_REQUIRED',
    'two standalone items can never be folded into a Bundle',
);
expect_bundle($projectionCount() === 1, 'two-item denial creates zero Bundle projections');

// A standalone Focusa item can never bind through the Bundle adapter.
$regStandalone = $createRegistration('operator.standalone@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'standalone');
$customerStandalone = $customerOf($regStandalone['registration_uuid']);
$insertOrder(6012, 'complete', $customerStandalone, 'operator.standalone@example.invalid', [
    ['item_id' => 6013, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(6012, $GATEWAY, 'txn_pay_6012');
expect_bundle_throws(
    fn() => $bundleBindAndIssue(6012, $regStandalone['registration_uuid'], $customerStandalone, [['item_id' => 6013, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_6012', 'standalone-1'),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a standalone Operator item can never bind as a Bundle',
);

// Caller-controlled commerce fields are forbidden at the adapter boundary.
expect_bundle_throws(
    fn() => $adapter->bindAndIssue([
        'order_id' => 6001,
        'order_status' => 'complete',
        'customer_id' => $customerA,
        'order_items' => [['order_item_id' => 6001, 'download_id' => $BUNDLE_DOWNLOAD, 'price_id' => $BUNDLE_PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => $GATEWAY, 'transaction_id' => 'txn_pay_6001', 'status' => 'complete']],
        'registration_uuid' => $regA['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'request_id' => 'req-bind-bundle-clientfields',
        'idempotency_key' => 'idem-bind-bundle-clientfields',
        'grants' => ['focusa_operator_lifetime_v1'],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied grants are forbidden at the Bundle adapter',
);

// ── Negative: wrong price creates no Bundle projection ─────────────────

// The binding/issuance use the correct price; the settled binding price is mutated
// after issuance so the projection-time price check must fail closed.
$regPrice = $createRegistration('operator.bundleprice@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundleprice');
$customerPrice = $customerOf($regPrice['registration_uuid']);
$insertOrder(6004, 'complete', $customerPrice, 'operator.bundleprice@example.invalid', [
    ['item_id' => 6004, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6004, $GATEWAY, 'txn_pay_6004');
$boundPrice = $bind(6004, $regPrice['registration_uuid'], $customerPrice, [['item_id' => 6004, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6004', 'bundleprice-1');
$handlePrice = $boundPrice['protected_items'][0]['issuance_request_handle'];
$issuedPrice = $issue($handlePrice, 'req-issue-bundleprice-1', 'idem-issue-bundleprice-1');
expect_bundle($issuedPrice['keys_created'] === 1, 'correct-price Bundle item issues its key');
$db->exec("UPDATE wp_wpuiai_edd_order_bindings SET price_id = 'price_wrong' WHERE binding_key = (SELECT binding_key FROM wp_wpuiai_edd_issuance_requests WHERE issuance_request_key = '{$handlePrice}')");
expect_bundle_throws(
    fn() => $bundleProject($handlePrice, 'req-project-bundleprice-1', 'idem-project-bundleprice-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a settled Bundle item whose price no longer matches the dedicated offer fails closed',
);
expect_bundle($projectionCount() === 1, 'wrong-price denial creates zero Bundle projections');

// A Bundle binding that never carried a matching price fails at the binding boundary.
$regPriceBind = $createRegistration('operator.bundlepricebind@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlepricebind');
$customerPriceBind = $customerOf($regPriceBind['registration_uuid']);
$insertOrder(6013, 'complete', $customerPriceBind, 'operator.bundlepricebind@example.invalid', [
    ['item_id' => 6014, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6013, $GATEWAY, 'txn_pay_6013');
expect_bundle_throws(
    fn() => $bind(6013, $regPriceBind['registration_uuid'], $customerPriceBind, [['item_id' => 6014, 'download' => $BUNDLE_DOWNLOAD, 'price' => 'price_wrong']], 'txn_pay_6013', 'bundlepricebind-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a wrong Bundle price_id can never settle an issuance request',
);

// ── Negative: wrong account creates no Bundle projection ───────────────

$regAccount = $createRegistration('operator.bundleaccount@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundleaccount');
$customerAccount = $customerOf($regAccount['registration_uuid']);
$insertOrder(6005, 'complete', $customerAccount, 'operator.bundleaccount@example.invalid', [
    ['item_id' => 6005, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6005, $GATEWAY, 'txn_pay_6005');
$boundAccount = $bind(6005, $regAccount['registration_uuid'], $customerAccount, [['item_id' => 6005, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6005', 'bundleaccount-1');
$handleAccount = $boundAccount['protected_items'][0]['issuance_request_handle'];
$issuedAccount = $issue($handleAccount, 'req-issue-bundleaccount-1', 'idem-issue-bundleaccount-1');
expect_bundle($issuedAccount['keys_created'] === 1, 'account-fixture Bundle item issues its key');
$db->exec('UPDATE wp_edd_orders SET customer_id = 424242 WHERE id = 6005');
expect_bundle_throws(
    fn() => $bundleProject($handleAccount, 'req-project-bundleaccount-1', 'idem-project-bundleaccount-1'),
    'EDD_ORDER_UNVERIFIED',
    'a Bundle order whose customer changed after settlement fails closed',
);
expect_bundle($projectionCount() === 1, 'wrong-account denial creates zero Bundle projections');

// ── Negative: canonical order truth at projection time ────────────────

// Refunded canonical Bundle order after issuance: fails closed, zero projection.
$regRefunded = $createRegistration('operator.bundlerefunded@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlerefunded');
$customerRefunded = $customerOf($regRefunded['registration_uuid']);
$insertOrder(6006, 'complete', $customerRefunded, 'operator.bundlerefunded@example.invalid', [
    ['item_id' => 6006, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6006, $GATEWAY, 'txn_pay_6006');
$boundRefunded = $bind(6006, $regRefunded['registration_uuid'], $customerRefunded, [['item_id' => 6006, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6006', 'bundlerefunded-1');
$handleRefunded = $boundRefunded['protected_items'][0]['issuance_request_handle'];
$issuedRefunded = $issue($handleRefunded, 'req-issue-bundlerefunded-1', 'idem-issue-bundlerefunded-1');
expect_bundle($issuedRefunded['keys_created'] === 1, 'refunded-fixture Bundle item issues its key before the order is refunded');
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE id = 6006");
expect_bundle_throws(
    fn() => $bundleProject($handleRefunded, 'req-project-bundlerefunded-1', 'idem-project-bundlerefunded-1'),
    'REFUNDED',
    'a refunded canonical Bundle order never projects',
);
expect_bundle($projectionCount() === 1, 'refunded-order denial creates zero Bundle projections');

// Revoked canonical Bundle order after issuance: fails closed, zero projection.
$regRevoked = $createRegistration('operator.bundlerevoked@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlerevoked');
$customerRevoked = $customerOf($regRevoked['registration_uuid']);
$insertOrder(6007, 'complete', $customerRevoked, 'operator.bundlerevoked@example.invalid', [
    ['item_id' => 6007, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6007, $GATEWAY, 'txn_pay_6007');
$boundRevoked = $bind(6007, $regRevoked['registration_uuid'], $customerRevoked, [['item_id' => 6007, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6007', 'bundlerevoked-1');
$handleRevoked = $boundRevoked['protected_items'][0]['issuance_request_handle'];
$issuedRevoked = $issue($handleRevoked, 'req-issue-bundlerevoked-1', 'idem-issue-bundlerevoked-1');
expect_bundle($issuedRevoked['keys_created'] === 1, 'revoked-fixture Bundle item issues its key before the order is revoked');
$db->exec("UPDATE wp_edd_orders SET status = 'revoked' WHERE id = 6007");
expect_bundle_throws(
    fn() => $bundleProject($handleRevoked, 'req-project-bundlerevoked-1', 'idem-project-bundlerevoked-1'),
    'REVOKED',
    'a revoked canonical Bundle order never projects',
);
expect_bundle($projectionCount() === 1, 'revoked-order denial creates zero Bundle projections');

// Canonical Bundle order row moved back to pending: EDD_ORDER_PENDING.
$regPending = $createRegistration('operator.bundlepending@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlepending');
$customerPending = $customerOf($regPending['registration_uuid']);
$insertOrder(6008, 'complete', $customerPending, 'operator.bundlepending@example.invalid', [
    ['item_id' => 6008, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6008, $GATEWAY, 'txn_pay_6008');
$boundPending = $bind(6008, $regPending['registration_uuid'], $customerPending, [['item_id' => 6008, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6008', 'bundlepending-1');
$handlePending = $boundPending['protected_items'][0]['issuance_request_handle'];
$issuedPending = $issue($handlePending, 'req-issue-bundlepending-1', 'idem-issue-bundlepending-1');
expect_bundle($issuedPending['keys_created'] === 1, 'pending-fixture Bundle item issues its key before the order moves back to pending');
$db->exec("UPDATE wp_edd_orders SET status = 'pending' WHERE id = 6008");
expect_bundle_throws(
    fn() => $bundleProject($handlePending, 'req-project-bundlepending-1', 'idem-project-bundlepending-1'),
    'EDD_ORDER_PENDING',
    'a pending canonical Bundle order fails closed with EDD_ORDER_PENDING',
);
expect_bundle($projectionCount() === 1, 'pending-order denial creates zero Bundle projections');

// ── Negative: canonical license truth at projection time ──────────────

// Projection before issuance: the issuance request is still pending, no key, no projection.
$regNoIssue = $createRegistration('operator.bundlenoissue@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlenoissue');
$customerNoIssue = $customerOf($regNoIssue['registration_uuid']);
$insertOrder(6009, 'complete', $customerNoIssue, 'operator.bundlenoissue@example.invalid', [
    ['item_id' => 6009, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6009, $GATEWAY, 'txn_pay_6009');
$boundNoIssue = $bind(6009, $regNoIssue['registration_uuid'], $customerNoIssue, [['item_id' => 6009, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6009', 'bundlenoissue-1');
$handleNoIssue = $boundNoIssue['protected_items'][0]['issuance_request_handle'];
expect_bundle_throws(
    fn() => $bundleProject($handleNoIssue, 'req-project-bundlenoissue-1', 'idem-project-bundlenoissue-1'),
    'EDD_LICENSE_UNUSABLE',
    'no canonical Bundle key, no Bundle projection',
);
expect_bundle($projectionCount() === 1, 'pre-issuance projection creates zero Bundle projections');

// License revoked after issuance: the license row is no longer active, zero projection.
$regLicense = $createRegistration('operator.bundlelicense@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlelicense');
$customerLicense = $customerOf($regLicense['registration_uuid']);
$insertOrder(6010, 'complete', $customerLicense, 'operator.bundlelicense@example.invalid', [
    ['item_id' => 6010, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6010, $GATEWAY, 'txn_pay_6010');
$boundLicense = $bind(6010, $regLicense['registration_uuid'], $customerLicense, [['item_id' => 6010, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6010', 'bundlelicense-1');
$handleLicense = $boundLicense['protected_items'][0]['issuance_request_handle'];
$issuedLicense = $issue($handleLicense, 'req-issue-bundlelicense-1', 'idem-issue-bundlelicense-1');
expect_bundle($issuedLicense['keys_created'] === 1, 'license-fixture Bundle item issues its key');
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = {$issuedLicense['edd_license_id']}");
expect_bundle_throws(
    fn() => $bundleProject($handleLicense, 'req-project-bundlelicense-1', 'idem-project-bundlelicense-1'),
    'EDD_LICENSE_UNUSABLE',
    'a revoked canonical Bundle license never projects',
);
expect_bundle($projectionCount() === 1, 'revoked-license denial creates zero Bundle projections');

// License key tampered after issuance: the journaled digest no longer matches.
$regTamper = $createRegistration('operator.bundletamper@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundletamper');
$customerTamper = $customerOf($regTamper['registration_uuid']);
$insertOrder(6014, 'complete', $customerTamper, 'operator.bundletamper@example.invalid', [
    ['item_id' => 6015, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6014, $GATEWAY, 'txn_pay_6014');
$boundTamper = $bind(6014, $regTamper['registration_uuid'], $customerTamper, [['item_id' => 6015, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6014', 'bundletamper-1');
$handleTamper = $boundTamper['protected_items'][0]['issuance_request_handle'];
$issuedTamper = $issue($handleTamper, 'req-issue-bundletamper-1', 'idem-issue-bundletamper-1');
expect_bundle($issuedTamper['keys_created'] === 1, 'tamper-fixture Bundle item issues its key');
$db->exec("UPDATE wp_edd_licenses SET license_key = '11111111-22222222-33333333-44444444' WHERE id = {$issuedTamper['edd_license_id']}");
expect_bundle_throws(
    fn() => $bundleProject($handleTamper, 'req-project-bundletamper-1', 'idem-project-bundletamper-1'),
    'EDD_LICENSE_UNUSABLE',
    'a tampered canonical Bundle license never projects',
);
expect_bundle($projectionCount() === 1, 'tampered-license denial creates zero Bundle projections');

// ── Negative: registry / offer authority ──────────────────────────────

// A checkout-disabled dedicated Bundle offer (the mapping resolves but is not enabled):
// EDD_CHECKOUT_REQUIRED.
$regFrozen = $createRegistration('operator.bundlefrozen@example.invalid', $FACADE, $BUNDLE_PRODUCT, 'bundlefrozen');
$customerFrozen = $customerOf($regFrozen['registration_uuid']);
$insertOrder(6015, 'complete', $customerFrozen, 'operator.bundlefrozen@example.invalid', [
    ['item_id' => 6016, 'download' => $BUNDLE_DOWNLOAD],
]);
$insertTransaction(6015, $GATEWAY, 'txn_pay_6015');
$boundFrozen = $bind(6015, $regFrozen['registration_uuid'], $customerFrozen, [['item_id' => 6016, 'download' => $BUNDLE_DOWNLOAD, 'price' => $BUNDLE_PRICE]], 'txn_pay_6015', 'bundlefrozen-1');
$handleFrozen = $boundFrozen['protected_items'][0]['issuance_request_handle'];
$issuedFrozen = $issue($handleFrozen, 'req-issue-bundlefrozen-1', 'idem-issue-bundlefrozen-1');
expect_bundle($issuedFrozen['keys_created'] === 1, 'frozen-fixture Bundle item issues its key');
expect_bundle_throws(
    fn() => $bundleProjectorBlocked->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-bundlefrozen-1',
        'idempotency_key' => 'idem-project-bundlefrozen-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'a checkout-disabled dedicated Bundle offer denies projection until validation passes',
);
expect_bundle($projectionCount() === 1, 'checkout-disabled denial creates zero Bundle projections');

// The truly frozen dedicated Downloads contract has no fixture download binding at all
// (the fixture binding used download 1003): PRODUCT_MAPPING_REQUIRED.
expect_bundle_throws(
    fn() => (new FocusaSpec172BundleOperatorProjector(
        $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
        $registrationSecrets, $accounts, $edd, $frozenDedicated, $clock,
    ))->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-bundlefrozen-real-1',
        'idempotency_key' => 'idem-project-bundlefrozen-real-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'the frozen dedicated Downloads contract (no active fixture mapping) denies Bundle projection',
);
expect_bundle($projectionCount() === 1, 'frozen-offer denial creates zero Bundle projections');

// The dedicated Bundle offer download mapping drifted: PRODUCT_MAPPING_REQUIRED.
expect_bundle_throws(
    fn() => $bundleProjectorMismatched->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-bundlemismatch-1',
        'idempotency_key' => 'idem-project-bundlemismatch-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'a Bundle offer whose download mapping drifted fails closed',
);
expect_bundle($projectionCount() === 1, 'mismatched-offer denial creates zero Bundle projections');

// The dedicated Bundle offer grants are NOT the exact union (a future License Type was
// appended): PRODUCT_MAPPING_REQUIRED, never a widened Bundle grant.
expect_bundle_throws(
    fn() => $bundleProjectorNonUnion->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-bundlenonunion-1',
        'idempotency_key' => 'idem-project-bundlenonunion-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'a Bundle offer whose grants are not the exact two Operator records fails closed',
);
expect_bundle($projectionCount() === 1, 'non-union denial creates zero Bundle projections');

// ── Negative: input validation and idempotency ─────────────────────────

$negativeChecks++;
try {
    $bundleProject('not-a-handle', 'req-project-bundlemalformed-1', 'idem-project-bundlemalformed-1');
    fwrite(STDERR, "FAIL: malformed issuance request handles are rejected\n");
    exit(1);
} catch (InvalidArgumentException) {
    // expected: bounded handle required
}
expect_bundle_throws(
    fn() => $bundleProject('ir_' . str_repeat('0', 32), 'req-project-bundleunknown-1', 'idem-project-bundleunknown-1'),
    'EDD_LICENSE_UNUSABLE',
    'unknown issuance request handles fail closed',
);
expect_bundle_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-bundleclientfields-1',
        'idempotency_key' => 'idem-project-bundleclientfields-1',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled commerce fields are forbidden at Bundle projection',
);
expect_bundle_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-bundleclientfields-2',
        'idempotency_key' => 'idem-project-bundleclientfields-2',
        'grants' => ['focusa_operator_lifetime_v1'],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied grant metadata is forbidden at Bundle projection',
);
expect_bundle_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-bundleclientfields-3',
        'idempotency_key' => 'idem-project-bundleclientfields-3',
        'products' => ['focusa'],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied product scope is forbidden at Bundle projection',
);
expect_bundle_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-bundleclientfields-4',
        'idempotency_key' => 'idem-project-bundleclientfields-4',
        'family_sets' => ['focusa' => ['future_product_family']],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied family sets are forbidden at Bundle projection',
);
expect_bundle_throws(
    fn() => $bundleProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-bundleclientfields-5',
        'idempotency_key' => 'idem-project-bundleclientfields-5',
        'future_products_included' => true,
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied future-product metadata is forbidden at Bundle projection',
);
expect_bundle_throws(
    fn() => $bundleProject($handleFocusa, 'req-project-bundleconflict-1', 'idem-project-bundle-alpha-1'),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// ── Rollback preservation and redaction ───────────────────────────────

$preserved = $projectionMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'bundle_composition_test', 'record' => 'rollback']);
expect_bundle($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_bundle((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_license_type_projection_schema_events')->fetchColumn() === 1, 'exactly one projection preservation event journaled');

$decisionJson = json_encode([$projectedA, $replayedA, $duplicateA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_bundle(strpos($decisionJson, '@') === false, 'no raw email in any Bundle projection decision');
expect_bundle(preg_match($KEY_SCAN_PATTERN, $decisionJson) !== 1, 'no full license key in any Bundle projection decision');

$projectionRows = $db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC);
$projectionJson = json_encode($projectionRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_bundle(strpos($projectionJson, '@') === false, 'no raw email in the projection journal');
expect_bundle(strpos($projectionJson, 'txn_pay_') === false, 'no raw payment transaction id in the projection journal');
expect_bundle(preg_match($KEY_SCAN_PATTERN, $projectionJson) !== 1, 'no full license key in the projection journal');
foreach ($projectionRows as $projectionRow) {
    expect_bundle(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRow['projection_key']) === 1, 'projection handles are opaque bounded tokens');
    expect_bundle(preg_match('/^[0-9a-f]{64}$/D', (string) $projectionRow['family_digest']) === 1, 'family digest is a 64-hex digest');
    expect_bundle((int) $projectionRow['sequence'] === 1, 'the single Bundle projection keeps sequence 1');
    expect_bundle(strpos((string) $projectionRow['result_payload'], '"license_key"') === false, 'projection payloads never contain a raw license_key field');
    expect_bundle(preg_match($KEY_SCAN_PATTERN, (string) $projectionRow['result_payload']) !== 1, 'projection payloads never contain a full key');
    expect_bundle(strpos((string) $projectionRow['result_payload'], '@') === false, 'projection payloads never contain raw email');
}
$fixtureJson = json_encode([$fixtureA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_bundle(strpos($fixtureJson, '@') === false, 'Bundle lease fixture carries no raw email');
expect_bundle(preg_match($KEY_SCAN_PATTERN, $fixtureJson) !== 1, 'Bundle lease fixture carries no full license key');
expect_bundle(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $fixtureJson) !== 1, 'Bundle lease fixture carries no payment key');
expect_bundle(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $fixtureJson) !== 1, 'Bundle lease fixture carries no card data');

$leaseJson = json_encode($leaseA, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_bundle(strpos($leaseJson, '@') === false && preg_match($KEY_SCAN_PATTERN, $leaseJson) !== 1, 'Bundle lease payload carries no raw email or key');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.bundle_composition_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'projections_created' => $projectionCount(),
    'canonical_licenses_created' => $licenseCount(),
    'sku' => 'focusa_uiai_operator_bundle_lifetime_v1',
    'grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
    'grants_union' => 'exact_union',
    'human_key_count' => 1,
    'price_version' => 'focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1',
    'price_usd' => '1254.60',
    'amount_minor' => 125460,
    'family_count' => count(FocusaSpec172LicenseTypeRegistry::underlyingFamilies()),
    'family_sets' => array_map('count', FocusaSpec172LicenseTypeRegistry::familySets()),
    'family_digest' => FocusaSpec172LicenseTypeRegistry::familyDigest(),
    'operator_seats' => 1,
    'node_limit' => 3,
    'node_set' => 'operator_shared_v1',
    'term' => 'lifetime',
    'sequence' => $projectedA['sequence'],
    'bundle_signed_lease_fixture' => 'derived_from_composite_projection_exact_grant_union_shared_nodes',
    'duplicate_issuance_fixtures' => ['idempotent_replay', 'duplicate_projection_call', 'two_item_order', 'standalone_focusa', 'standalone_uiai', 'wrong_product_guard', 'cross_product_guard', 'wrong_price', 'wrong_account', 'refunded', 'revoked', 'pending', 'pre_issuance', 'revoked_license', 'tampered_license', 'frozen_checkout_disabled', 'drifted_mapping', 'non_exact_union_grants', 'caller_commerce_fields', 'future_product_excluded', 'idempotency_conflict'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
