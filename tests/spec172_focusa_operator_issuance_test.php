<?php
// 172.02.04 Issue Focusa Operator Lifetime v1 from canonical EDD truth.
// The Focusa Operator projector consumes exactly one issued canonical EDD Software
// Licensing key for one verified complete eligible Focusa order and projects
// focusa_operator_lifetime_v1 from canonical EDD truth: the canonical order row
// (complete, exact customer, exact verified-email digest), the canonical order item, the
// active canonical EDD license with the exact journaled key digest, and the server-owned
// dedicated Downloads offer (exact download, public code, License Type ref, price id,
// amount, lifetime, one seat, three shared nodes). One eligible item produces exactly one
// projection with the frozen family digest, seat/node limits, price version, and a
// strictly monotonic per-account sequence; replays return the identical decision and
// duplicate projection calls return the same projection with zero creations. Wrong
// product (uiai), wrong price, wrong account, refunded/revoked/pending orders, unissued
// requests, revoked/tampered licenses, frozen (checkout-disabled) offers, caller
// metadata grants, and idempotency conflicts create none. The Focusa paid lease fixture
// derives the machine credential exclusively from the projection: lifetime entitlement
// with bounded credential lifetime, one seat, three shared nodes, no caller-selected
// commercial right. No raw email, key, token, customer row, credential, or card data is
// stored or returned anywhere.
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

$positiveChecks = 0;
$negativeChecks = 0;

function expect_projection(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_projection_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'focusa_operator_issuance_test']);

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
// test mappings (download 1001 -> focusa_operator_lifetime_v1 and download 1002 ->
// uiai_operator_lifetime_v1, both active/checkout_enabled at the fixed canonical
// prices) so positive and wrong-product paths are exercised without mutating the
// frozen contracts.
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
}
unset($record);

$bindingFrozen = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $frozenRegistry, $facadeRegistry, $clock,
);
$bindingService = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $fixtureRegistry, $facadeRegistry, $clock,
);

$issuanceFrozen = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $frozenRegistry, $clock,
);
$issuanceService = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $fixtureRegistry, $clock,
);

$projectorFrozen = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $frozenDedicated, $clock,
);

// Checkout-disabled fixture: the dedicated offer resolves (download 1001) but checkout
// is not enabled, so projection fails closed with EDD_CHECKOUT_REQUIRED.
$blockedDedicated = $fixtureDedicated;
foreach ($blockedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_operator_lifetime_v1') {
        $record['checkout_enabled'] = false;
        $record['sale_status'] = 'approved_not_yet_enabled';
    }
}
unset($record);
$projectorBlocked = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $blockedDedicated, $clock,
);
$projector = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);

// Mutated fixture: the focusa offer's download/license-type mapping no longer matches
// the settled item (PRODUCT_MAPPING_REQUIRED at projection).
$mismatchedDedicated = $fixtureDedicated;
foreach ($mismatchedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'focusa_operator_lifetime_v1') {
        $record['edd_download_id'] = 9999;
    }
}
unset($record);
$projectorMismatched = new FocusaSpec172FocusaOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $mismatchedDedicated, $clock,
);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true, bool $checkout = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.focusa.operator.issuance.test',
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
        'migration_provenance' => ['source' => 'spec172_candidate', 'record' => 'focusa-operator-' . $tag . '-' . $seq],
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
$FOCUSA_PRODUCT = 'focusa_operator_lifetime_v1';
$UIAI_PRODUCT = 'uiai_operator_lifetime_v1';
$FOCUSA_DOWNLOAD = 1001;
$UIAI_DOWNLOAD = 1002;
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

$project = static function (string $handle, string $requestId, string $idempotencyKey) use ($projector): array {
    return $projector->project([
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

// ── Frozen contracts remain canonical ─────────────────────────────────

expect_projection($frozenDedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1', 'frozen dedicated downloads schema');
expect_projection($frozenDedicated['owner'] === 'WPUIAI/wpuiai', 'frozen dedicated downloads owner');
expect_projection(count($frozenDedicated['records']) === 3, 'frozen dedicated downloads has three records');
expect_projection($frozenDedicated['counts']['checkout_enabled'] === 0, 'frozen dedicated downloads checkout disabled');
$frozenFocusa = null;
foreach ($frozenDedicated['records'] as $record) {
    if ($record['public_code'] === 'focusa_operator_lifetime_v1') {
        $frozenFocusa = $record;
    }
}
expect_projection($frozenFocusa !== null, 'frozen dedicated downloads has the focusa record');
expect_projection((int) $frozenFocusa['amount_minor'] === 69700 && $frozenFocusa['price_usd'] === '697.00', 'frozen focusa offer is 697.00');
expect_projection((int) $frozenFocusa['operator_seats'] === 1 && (int) $frozenFocusa['node_limit'] === 3 && $frozenFocusa['node_set'] === 'operator_shared_v1', 'frozen focusa offer one seat three shared nodes');
expect_projection($frozenFocusa['license_duration'] === 'lifetime', 'frozen focusa offer lifetime');
expect_projection($projectionMigration::SCHEMA === 'focusa.spec172.license_type_projection.v1', 'projection schema is canonical');
expect_projection(FocusaSpec172FocusaOperatorProjector::RESULT_SCHEMA === 'focusa.spec172.focusa_operator_lifetime_projection.v1', 'projection result schema is canonical');
expect_projection(FocusaSpec172FocusaOperatorProjector::familyDigest() !== '', 'frozen family digest is deterministic');
expect_projection(count(FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES) === 5, 'frozen family set has five Focusa Operator v1 families');
expect_projection(FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES === ['base_focusa', 'automation', 'team_remote', 'release_proof', 'premium_updates'], 'frozen family set matches Spec 172 section 7.1');

// ── Positive: one eligible Focusa order -> one key + one projection ───

$regA = $createRegistration('operator.alpha@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'alpha');
$customerA = $customerOf($regA['registration_uuid']);
$insertOrder(4001, 'complete', $customerA, 'operator.alpha@example.invalid', [
    ['item_id' => 4001, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4001, $GATEWAY, 'txn_pay_4001');

$bound = $bind(4001, $regA['registration_uuid'], $customerA, [['item_id' => 4001, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4001', 'alpha-1');
expect_projection($bound['decision'] === 'order_bound' && $bound['issuance_requests_settled'] === 1, 'eligible Focusa order settles exactly one issuance request');
$handleA = $bound['protected_items'][0]['issuance_request_handle'];
expect_projection(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $handleA) === 1, 'issuance request handle is an opaque bounded token');

$issuedA = $issue($handleA, 'req-issue-alpha-1', 'idem-issue-alpha-1');
expect_projection($issuedA['decision'] === 'license_issued' && $issuedA['keys_created'] === 1, 'one eligible Focusa item issues exactly one canonical key');
expect_projection($issuedA['product_code'] === $FOCUSA_PRODUCT && $issuedA['license_type_ref'] === $FOCUSA_PRODUCT, 'issued key carries the server-owned Focusa offer');
$keyA = $issuedA['delivery']['license_key'];
expect_projection(preg_match($KEY_PATTERN, $keyA) === 1, 'issued key is canonical EDD SL format');
expect_projection(str_starts_with($keyA, 'focusa_live_') === false, 'issued key is never a synthetic key');

$projectedA = $project($handleA, 'req-project-alpha-1', 'idem-project-alpha-1');
expect_projection($projectedA['schema'] === 'focusa.spec172.focusa_operator_lifetime_projection.v1', 'projection schema is canonical');
expect_projection($projectedA['decision'] === 'license_type_projected', 'projection decision is license_type_projected');
expect_projection($projectedA['existing'] === false && $projectedA['projections_created'] === 1, 'first projection creates exactly one projection');
expect_projection($projectedA['product'] === 'focusa' && $projectedA['license_type'] === 'focusa_operator_lifetime_v1' && $projectedA['grant'] === 'focusa_operator_lifetime_v1', 'projection carries the Focusa Operator Lifetime v1 grant');
expect_projection($projectedA['registration_id'] === $regA['registration_uuid'], 'projection is linked to the registration');
expect_projection($projectedA['account_id'] !== '', 'projection is linked to the account');
expect_projection($projectedA['customer_id'] === $customerA, 'projection is linked to the EDD customer');
expect_projection($projectedA['order_id'] === 4001 && $projectedA['order_item_id'] === 4001, 'projection is linked to the canonical order item');
expect_projection($projectedA['download_id'] === $FOCUSA_DOWNLOAD, 'projection carries the canonical download');
expect_projection(is_int($projectedA['edd_license_id']) && $projectedA['edd_license_id'] === $issuedA['edd_license_id'], 'projection references the canonical issued license');
expect_projection($projectedA['issuance'] === 'canonical_edd_software_licensing', 'projection derives from canonical EDD Software Licensing');
expect_projection($projectedA['license_key_digest'] === $issuedA['license_key_digest'], 'projection carries the keyed license digest');
expect_projection(strpos((string) $projectedA['license_key_mask'], '********-********-********-') === 0, 'projection carries only a masked license key');

// Frozen family digest, seat/node limits, price version, sequence.
expect_projection($projectedA['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'projection carries the frozen family digest');
expect_projection($projectedA['family_count'] === 5, 'projection freezes five families');
expect_projection($projectedA['operator_seats'] === 1, 'projection freezes one operator seat');
expect_projection($projectedA['node_limit'] === 3 && $projectedA['node_set'] === 'operator_shared_v1', 'projection freezes three shared operator nodes');
expect_projection($projectedA['term'] === 'lifetime' && $projectedA['status'] === 'active', 'projection is an active lifetime grant');
expect_projection($projectedA['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1', 'projection carries the server-owned price version');
expect_projection($projectedA['price_usd'] === '697.00' && $projectedA['amount_minor'] === 69700, 'projection carries the canonical price');
expect_projection(is_int($projectedA['sequence']) && $projectedA['sequence'] === 1, 'projection carries the first monotonic sequence');
expect_projection($accountSequence($regA['registration_uuid']) === 1, 'authority account sequence advanced to 1');

expect_projection($projectionCount() === 1, 'exactly one projection journal row');
$projectionRowA = $projector->findByIssuanceRequestKey($handleA);
expect_projection($projectionRowA !== null && $projectionRowA['status'] === 'active', 'projection journal row is active');
expect_projection(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRowA['projection_key']) === 1, 'projection handles are opaque bounded tokens');
expect_projection($projectionRowA['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1', 'projection journal carries the price version');
expect_projection($projectionRowA['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'projection journal carries the frozen family digest');
expect_projection((int) $projectionRowA['operator_seats'] === 1 && (int) $projectionRowA['node_limit'] === 3 && $projectionRowA['node_set'] === 'operator_shared_v1', 'projection journal carries seat and node limits');
expect_projection((int) $projectionRowA['sequence'] === 1, 'projection journal carries the sequence');
expect_projection($projectionRowA['license_type_ref'] === $FOCUSA_PRODUCT && $projectionRowA['product_code'] === 'focusa', 'projection journal carries the canonical product and License Type');
expect_projection($projector->findByProjectionKey((string) $projectionRowA['projection_key'])['issuance_request_key'] === $handleA, 'projection lookup by handle resolves the source request');

// Registration fulfillment (from SL issuance) is preserved: entitlement_issued.
$regRowA = $registrations->findByUuid($regA['registration_uuid']);
expect_projection($regRowA['state'] === 'entitlement_issued', 'registration is at entitlement_issued');
expect_projection((int) $regRowA['edd_license_id'] === $issuedA['edd_license_id'], 'registration references the canonical issued license');

// Idempotent replay: same key returns the identical decision, no second projection.
$replayedA = $project($handleA, 'req-project-alpha-1', 'idem-project-alpha-1');
expect_projection(json_encode($replayedA, JSON_THROW_ON_ERROR) === json_encode($projectedA, JSON_THROW_ON_ERROR), 'idempotency replay returns the identical decision');
expect_projection($projectionCount() === 1, 'replay creates no second projection row');
expect_projection($accountSequence($regA['registration_uuid']) === 1, 'replay does not bump the sequence');

// Duplicate projection call with a NEW idempotency key: same projection, zero new.
$duplicateA = $project($handleA, 'req-project-alpha-retry-1', 'idem-project-alpha-retry-1');
expect_projection($duplicateA['existing'] === true, 'duplicate projection call is an existing projection');
expect_projection($duplicateA['projections_created'] === 0, 'duplicate projection call creates zero projections');
expect_projection($duplicateA['edd_license_id'] === $issuedA['edd_license_id'], 'duplicate projection call returns the same license reference');
expect_projection($duplicateA['sequence'] === 1 && $duplicateA['family_digest'] === $projectedA['family_digest'], 'duplicate projection call returns the identical grant');
expect_projection($projectionCount() === 1, 'duplicate projection call never creates a second projection');
expect_projection($accountSequence($regA['registration_uuid']) === 1, 'duplicate projection call never bumps the sequence');

// ── Focusa paid lease fixture derives from the projection ─────────────

$fixtureA = FocusaSpec172FocusaPaidLeaseFixture::fromProjection($projectedA, 'node-operator-001', $clock);
expect_projection($fixtureA['schema'] === 'focusa.spec172.focusa_paid_lease_fixture.v1', 'paid lease fixture schema is canonical');
$payloadA = $fixtureA['lease_payload'];
expect_projection($payloadA['schema'] === 'focusa.authority_lease.v1', 'lease payload schema matches the authority lease payload');
expect_projection($payloadA['product'] === 'focusa', 'lease payload product is focusa');
expect_projection($payloadA['subject_id'] === $projectedA['account_id'], 'lease payload subject is the projected account');
expect_projection($payloadA['node_id'] === 'node-operator-001', 'lease payload binds the operator node');
expect_projection((int) $payloadA['sequence'] === 1, 'lease payload carries the projected sequence');
expect_projection($payloadA['status'] === 'active', 'lease payload is active for this sequence');
expect_projection($payloadA['authority_key_id'] !== '', 'lease payload names the authority signing key');
expect_projection(count($payloadA['features']) === 5, 'lease payload carries all five frozen families');
foreach ($payloadA['features'] as $family => $enabled) {
    expect_projection($enabled === true && in_array($family, FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES, true), "lease payload family {$family} is enabled and frozen");
}
expect_projection((int) $payloadA['limits']['operator_seats'] === 1 && (int) $payloadA['limits']['node_limit'] === 3, 'lease payload carries one seat and three nodes');
expect_projection((string) $payloadA['expires_at'] > (string) $payloadA['issued_at'], 'lease credential lifetime is bounded (never perpetual)');
expect_projection((string) $payloadA['offline_grace_until'] > (string) $payloadA['expires_at'], 'offline grace is bounded past the refresh window');
$metaA = $fixtureA['grant_metadata'];
expect_projection($metaA['license_type'] === 'focusa_operator_lifetime_v1' && $metaA['price_version'] === 'focusa_operator_lifetime_v1.697.00.v1', 'lease fixture carries explicit grant metadata');
expect_projection($metaA['family_digest'] === FocusaSpec172FocusaOperatorProjector::familyDigest(), 'lease fixture carries the frozen family digest');
expect_projection($metaA['term'] === 'lifetime' && $metaA['node_set'] === 'operator_shared_v1', 'lease fixture carries lifetime term and shared node set');
expect_projection($metaA['refund_policy'] === 'whole_order_30_days', 'lease fixture carries the whole-order refund policy');

// validate() accepts the derived fixture and rejects tampering.
$validated = FocusaSpec172FocusaPaidLeaseFixture::validate($fixtureA, $projectedA);
expect_projection($validated === null, 'fixture validation passes for the derived fixture');
$tamperedFixture = $fixtureA;
$tamperedFixture['lease_payload']['limits']['node_limit'] = 99;
expect_projection_throws(
    fn() => FocusaSpec172FocusaPaidLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_LIMIT_MISMATCH',
    'tampered node limit fails fixture validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant_metadata']['family_digest'] = str_repeat('0', 64);
expect_projection_throws(
    fn() => FocusaSpec172FocusaPaidLeaseFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_MISMATCH',
    'tampered family digest fails fixture validation',
);
expect_projection_throws(
    fn() => FocusaSpec172FocusaPaidLeaseFixture::fromProjection($issuedA, 'node-operator-001', $clock),
    'LICENSE_TYPE_PROJECTION_REQUIRED',
    'the lease fixture requires exactly an accepted projection, never an issuance decision',
);
expect_projection_throws(
    fn() => FocusaSpec172FocusaPaidLeaseFixture::fromProjection($projectedA, 'node@raw.example', $clock),
    'bounded node id required',
    'raw email node ids are rejected in the lease fixture',
);

// ── Negative: wrong product creates no Focusa projection ──────────────

// A fully eligible UIAI order issues its own canonical UIAI key (generic SL issuance)
// but can never project a Focusa Operator Lifetime v1 grant.
$regUiai = $createRegistration('operator.uiai@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiai');
$customerUiai = $customerOf($regUiai['registration_uuid']);
$insertOrder(4002, 'complete', $customerUiai, 'operator.uiai@example.invalid', [
    ['item_id' => 4002, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(4002, $GATEWAY, 'txn_pay_4002');
$boundUiai = $bind(4002, $regUiai['registration_uuid'], $customerUiai, [['item_id' => 4002, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_4002', 'uiai-1');
expect_projection($boundUiai['issuance_requests_settled'] === 1, 'UIAI order settles its own issuance request');
$handleUiai = $boundUiai['protected_items'][0]['issuance_request_handle'];
$issuedUiai = $issue($handleUiai, 'req-issue-uiai-1', 'idem-issue-uiai-1');
expect_projection($issuedUiai['keys_created'] === 1 && $issuedUiai['license_type_ref'] === $UIAI_PRODUCT, 'UIAI item issues its own canonical UIAI key');
expect_projection_throws(
    fn() => $project($handleUiai, 'req-project-uiai-1', 'idem-project-uiai-1'),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a UIAI license can never project focusa_operator_lifetime_v1',
);
expect_projection($projectionCount() === 1, 'wrong-product denial creates zero Focusa projections');
expect_projection($accountSequence($regUiai['registration_uuid']) === 0, 'wrong-product denial never advances the account sequence');

// ── Negative: wrong price creates no projection ───────────────────────

// The binding/issuance use the correct price; the settled binding price is mutated
// after issuance so the projection-time price check must fail closed.
$regPrice = $createRegistration('operator.price@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'price');
$customerPrice = $customerOf($regPrice['registration_uuid']);
$insertOrder(4003, 'complete', $customerPrice, 'operator.price@example.invalid', [
    ['item_id' => 4003, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4003, $GATEWAY, 'txn_pay_4003');
$boundPrice = $bind(4003, $regPrice['registration_uuid'], $customerPrice, [['item_id' => 4003, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4003', 'price-1');
$handlePrice = $boundPrice['protected_items'][0]['issuance_request_handle'];
$issuedPrice = $issue($handlePrice, 'req-issue-price-1', 'idem-issue-price-1');
expect_projection($issuedPrice['keys_created'] === 1, 'correct-price item issues its key');
$db->exec("UPDATE wp_wpuiai_edd_order_bindings SET price_id = 'price_wrong' WHERE binding_key = (SELECT binding_key FROM wp_wpuiai_edd_issuance_requests WHERE issuance_request_key = '{$handlePrice}')");
expect_projection_throws(
    fn() => $project($handlePrice, 'req-project-price-1', 'idem-project-price-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a settled item whose price no longer matches the dedicated offer fails closed',
);
expect_projection($projectionCount() === 1, 'wrong-price denial creates zero projections');

// A binding that never carried a matching price fails at the binding boundary.
$regPriceBind = $createRegistration('operator.pricebind@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'pricebind');
$customerPriceBind = $customerOf($regPriceBind['registration_uuid']);
$insertOrder(4011, 'complete', $customerPriceBind, 'operator.pricebind@example.invalid', [
    ['item_id' => 4011, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4011, $GATEWAY, 'txn_pay_4011');
expect_projection_throws(
    fn() => $bind(4011, $regPriceBind['registration_uuid'], $customerPriceBind, [['item_id' => 4011, 'download' => $FOCUSA_DOWNLOAD, 'price' => 'price_wrong']], 'txn_pay_4011', 'pricebind-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a wrong price_id can never settle an issuance request',
);

// ── Negative: wrong account creates no projection ─────────────────────

$regAccount = $createRegistration('operator.account@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'account');
$customerAccount = $customerOf($regAccount['registration_uuid']);
$insertOrder(4004, 'complete', $customerAccount, 'operator.account@example.invalid', [
    ['item_id' => 4004, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4004, $GATEWAY, 'txn_pay_4004');
$boundAccount = $bind(4004, $regAccount['registration_uuid'], $customerAccount, [['item_id' => 4004, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4004', 'account-1');
$handleAccount = $boundAccount['protected_items'][0]['issuance_request_handle'];
$issuedAccount = $issue($handleAccount, 'req-issue-account-1', 'idem-issue-account-1');
expect_projection($issuedAccount['keys_created'] === 1, 'account-fixture item issues its key');
$db->exec('UPDATE wp_edd_orders SET customer_id = 424242 WHERE id = 4004');
expect_projection_throws(
    fn() => $project($handleAccount, 'req-project-account-1', 'idem-project-account-1'),
    'EDD_ORDER_UNVERIFIED',
    'an order whose customer changed after settlement fails closed',
);
expect_projection($projectionCount() === 1, 'wrong-account denial creates zero projections');

// ── Negative: canonical order truth at projection time ────────────────

// Refunded canonical order after issuance: fails closed, zero projection.
$regRefunded = $createRegistration('operator.refunded@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'refunded');
$customerRefunded = $customerOf($regRefunded['registration_uuid']);
$insertOrder(4005, 'complete', $customerRefunded, 'operator.refunded@example.invalid', [
    ['item_id' => 4005, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4005, $GATEWAY, 'txn_pay_4005');
$boundRefunded = $bind(4005, $regRefunded['registration_uuid'], $customerRefunded, [['item_id' => 4005, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4005', 'refunded-1');
$handleRefunded = $boundRefunded['protected_items'][0]['issuance_request_handle'];
$issuedRefunded = $issue($handleRefunded, 'req-issue-refunded-1', 'idem-issue-refunded-1');
expect_projection($issuedRefunded['keys_created'] === 1, 'refunded-fixture item issues its key before the order is refunded');
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE id = 4005");
expect_projection_throws(
    fn() => $project($handleRefunded, 'req-project-refunded-1', 'idem-project-refunded-1'),
    'REFUNDED',
    'a refunded canonical order never projects',
);
expect_projection($projectionCount() === 1, 'refunded-order denial creates zero projections');

// Revoked canonical order after issuance: fails closed, zero projection.
$regRevoked = $createRegistration('operator.revoked@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'revoked');
$customerRevoked = $customerOf($regRevoked['registration_uuid']);
$insertOrder(4006, 'complete', $customerRevoked, 'operator.revoked@example.invalid', [
    ['item_id' => 4006, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4006, $GATEWAY, 'txn_pay_4006');
$boundRevoked = $bind(4006, $regRevoked['registration_uuid'], $customerRevoked, [['item_id' => 4006, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4006', 'revoked-1');
$handleRevoked = $boundRevoked['protected_items'][0]['issuance_request_handle'];
$issuedRevoked = $issue($handleRevoked, 'req-issue-revoked-1', 'idem-issue-revoked-1');
expect_projection($issuedRevoked['keys_created'] === 1, 'revoked-fixture item issues its key before the order is revoked');
$db->exec("UPDATE wp_edd_orders SET status = 'revoked' WHERE id = 4006");
expect_projection_throws(
    fn() => $project($handleRevoked, 'req-project-revoked-1', 'idem-project-revoked-1'),
    'REVOKED',
    'a revoked canonical order never projects',
);
expect_projection($projectionCount() === 1, 'revoked-order denial creates zero projections');

// Canonical order row moved back to pending: EDD_ORDER_PENDING.
$regPending = $createRegistration('operator.pending@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'pending');
$customerPending = $customerOf($regPending['registration_uuid']);
$insertOrder(4007, 'complete', $customerPending, 'operator.pending@example.invalid', [
    ['item_id' => 4007, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4007, $GATEWAY, 'txn_pay_4007');
$boundPending = $bind(4007, $regPending['registration_uuid'], $customerPending, [['item_id' => 4007, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4007', 'pending-1');
$handlePending = $boundPending['protected_items'][0]['issuance_request_handle'];
$issuedPending = $issue($handlePending, 'req-issue-pending-1', 'idem-issue-pending-1');
expect_projection($issuedPending['keys_created'] === 1, 'pending-fixture item issues its key before the order moves back to pending');
$db->exec("UPDATE wp_edd_orders SET status = 'pending' WHERE id = 4007");
expect_projection_throws(
    fn() => $project($handlePending, 'req-project-pending-1', 'idem-project-pending-1'),
    'EDD_ORDER_PENDING',
    'a pending canonical order fails closed with EDD_ORDER_PENDING',
);
expect_projection($projectionCount() === 1, 'pending-order denial creates zero projections');

// ── Negative: canonical license truth at projection time ──────────────

// Projection before issuance: the issuance request is still pending, no key, no projection.
$regNoIssue = $createRegistration('operator.noissue@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'noissue');
$customerNoIssue = $customerOf($regNoIssue['registration_uuid']);
$insertOrder(4008, 'complete', $customerNoIssue, 'operator.noissue@example.invalid', [
    ['item_id' => 4008, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4008, $GATEWAY, 'txn_pay_4008');
$boundNoIssue = $bind(4008, $regNoIssue['registration_uuid'], $customerNoIssue, [['item_id' => 4008, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4008', 'noissue-1');
$handleNoIssue = $boundNoIssue['protected_items'][0]['issuance_request_handle'];
expect_projection_throws(
    fn() => $project($handleNoIssue, 'req-project-noissue-1', 'idem-project-noissue-1'),
    'EDD_LICENSE_UNUSABLE',
    'no canonical key, no projection',
);
expect_projection($projectionCount() === 1, 'pre-issuance projection creates zero projections');

// License revoked after issuance: the license row is no longer active, zero projection.
$regLicense = $createRegistration('operator.licenserevoke@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'licenserevoke');
$customerLicense = $customerOf($regLicense['registration_uuid']);
$insertOrder(4009, 'complete', $customerLicense, 'operator.licenserevoke@example.invalid', [
    ['item_id' => 4009, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4009, $GATEWAY, 'txn_pay_4009');
$boundLicense = $bind(4009, $regLicense['registration_uuid'], $customerLicense, [['item_id' => 4009, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4009', 'licenserevoke-1');
$handleLicense = $boundLicense['protected_items'][0]['issuance_request_handle'];
$issuedLicense = $issue($handleLicense, 'req-issue-licenserevoke-1', 'idem-issue-licenserevoke-1');
expect_projection($issuedLicense['keys_created'] === 1, 'license-fixture item issues its key');
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = {$issuedLicense['edd_license_id']}");
expect_projection_throws(
    fn() => $project($handleLicense, 'req-project-licenserevoke-1', 'idem-project-licenserevoke-1'),
    'EDD_LICENSE_UNUSABLE',
    'a revoked canonical license never projects',
);
expect_projection($projectionCount() === 1, 'revoked-license denial creates zero projections');

// License key tampered after issuance: the journaled digest no longer matches.
$regTamper = $createRegistration('operator.tamper@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'tamper');
$customerTamper = $customerOf($regTamper['registration_uuid']);
$insertOrder(4010, 'complete', $customerTamper, 'operator.tamper@example.invalid', [
    ['item_id' => 4010, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4010, $GATEWAY, 'txn_pay_4010');
$boundTamper = $bind(4010, $regTamper['registration_uuid'], $customerTamper, [['item_id' => 4010, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4010', 'tamper-1');
$handleTamper = $boundTamper['protected_items'][0]['issuance_request_handle'];
$issuedTamper = $issue($handleTamper, 'req-issue-tamper-1', 'idem-issue-tamper-1');
expect_projection($issuedTamper['keys_created'] === 1, 'tamper-fixture item issues its key');
$db->exec("UPDATE wp_edd_licenses SET license_key = '11111111-22222222-33333333-44444444' WHERE id = {$issuedTamper['edd_license_id']}");
expect_projection_throws(
    fn() => $project($handleTamper, 'req-project-tamper-1', 'idem-project-tamper-1'),
    'EDD_LICENSE_UNUSABLE',
    'a tampered canonical license never projects',
);
expect_projection($projectionCount() === 1, 'tampered-license denial creates zero projections');

// ── Negative: registry / offer authority ──────────────────────────────

// A checkout-disabled dedicated offer (the mapping resolves but is not enabled):
// EDD_CHECKOUT_REQUIRED.
$regFrozen = $createRegistration('operator.frozen@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'frozen');
$customerFrozen = $customerOf($regFrozen['registration_uuid']);
$insertOrder(4012, 'complete', $customerFrozen, 'operator.frozen@example.invalid', [
    ['item_id' => 4012, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(4012, $GATEWAY, 'txn_pay_4012');
$boundFrozen = $bind(4012, $regFrozen['registration_uuid'], $customerFrozen, [['item_id' => 4012, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_4012', 'frozen-1');
$handleFrozen = $boundFrozen['protected_items'][0]['issuance_request_handle'];
$issuedFrozen = $issue($handleFrozen, 'req-issue-frozen-1', 'idem-issue-frozen-1');
expect_projection($issuedFrozen['keys_created'] === 1, 'frozen-fixture item issues its key');
expect_projection_throws(
    fn() => $projectorBlocked->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-frozen-1',
        'idempotency_key' => 'idem-project-frozen-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'a checkout-disabled dedicated offer denies projection until validation passes',
);
expect_projection($projectionCount() === 1, 'checkout-disabled denial creates zero projections');

// The truly frozen dedicated Downloads contract has no fixture download binding at all:
// PRODUCT_MAPPING_REQUIRED.
expect_projection_throws(
    fn() => $projectorFrozen->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-frozen-real-1',
        'idempotency_key' => 'idem-project-frozen-real-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'the frozen dedicated Downloads contract (no active fixture mapping) denies projection',
);
expect_projection($projectionCount() === 1, 'frozen-offer denial creates zero projections');

// The dedicated offer download mapping drifted: PRODUCT_MAPPING_REQUIRED.
expect_projection_throws(
    fn() => $projectorMismatched->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-mismatch-1',
        'idempotency_key' => 'idem-project-mismatch-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'an offer whose download mapping drifted fails closed',
);
expect_projection($projectionCount() === 1, 'mismatched-offer denial creates zero projections');

// ── Negative: input validation and idempotency ─────────────────────────

$negativeChecks++;
try {
    $project('not-a-handle', 'req-project-malformed-1', 'idem-project-malformed-1');
    fwrite(STDERR, "FAIL: malformed issuance request handles are rejected\n");
    exit(1);
} catch (InvalidArgumentException) {
    // expected: bounded handle required
}
expect_projection_throws(
    fn() => $project('ir_' . str_repeat('0', 32), 'req-project-unknown-1', 'idem-project-unknown-1'),
    'EDD_LICENSE_UNUSABLE',
    'unknown issuance request handles fail closed',
);
expect_projection_throws(
    fn() => $projector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-clientfields-1',
        'idempotency_key' => 'idem-project-clientfields-1',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled commerce fields are forbidden at projection',
);
expect_projection_throws(
    fn() => $projector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-clientfields-2',
        'idempotency_key' => 'idem-project-clientfields-2',
        'license_type_ref' => 'uiai_operator_lifetime_v1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied License Type metadata is forbidden at projection',
);
expect_projection_throws(
    fn() => $project($handleUiai, 'req-project-conflict-1', 'idem-project-alpha-1'),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// ── Rollback preservation and redaction ───────────────────────────────

$preserved = $projectionMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'focusa_operator_issuance_test', 'record' => 'rollback']);
expect_projection($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_projection((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_license_type_projection_schema_events')->fetchColumn() === 1, 'exactly one projection preservation event journaled');

$decisionJson = json_encode([$projectedA, $replayedA, $duplicateA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_projection(strpos($decisionJson, '@') === false, 'no raw email in any projection decision');
expect_projection(preg_match($KEY_SCAN_PATTERN, $decisionJson) !== 1, 'no full license key in any projection decision');

$projectionRows = $db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC);
$projectionJson = json_encode($projectionRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_projection(strpos($projectionJson, '@') === false, 'no raw email in the projection journal');
expect_projection(strpos($projectionJson, 'txn_pay_') === false, 'no raw payment transaction id in the projection journal');
expect_projection(preg_match($KEY_SCAN_PATTERN, $projectionJson) !== 1, 'no full license key in the projection journal');
foreach ($projectionRows as $projectionRow) {
    expect_projection(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRow['projection_key']) === 1, 'projection handles are opaque bounded tokens');
    expect_projection(preg_match('/^[0-9a-f]{64}$/D', (string) $projectionRow['family_digest']) === 1, 'family digest is a 64-hex digest');
    expect_projection((int) $projectionRow['sequence'] === 1, 'the single projection keeps sequence 1');
    expect_projection(strpos((string) $projectionRow['result_payload'], '"license_key"') === false, 'projection payloads never contain a raw license_key field');
    expect_projection(preg_match($KEY_SCAN_PATTERN, (string) $projectionRow['result_payload']) !== 1, 'projection payloads never contain a full key');
    expect_projection(strpos((string) $projectionRow['result_payload'], '@') === false, 'projection payloads never contain raw email');
}
$fixtureJson = json_encode([$fixtureA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_projection(strpos($fixtureJson, '@') === false, 'lease fixture carries no raw email');
expect_projection(preg_match($KEY_SCAN_PATTERN, $fixtureJson) !== 1, 'lease fixture carries no full license key');
expect_projection(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $fixtureJson) !== 1, 'lease fixture carries no payment key');
expect_projection(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $fixtureJson) !== 1, 'lease fixture carries no card data');

// The lease payload itself (outside the fixture envelope) is also clean.
$payloadJson = json_encode($payloadA, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_projection(strpos($payloadJson, '@') === false && preg_match($KEY_SCAN_PATTERN, $payloadJson) !== 1, 'lease payload carries no raw email or key');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.focusa_operator_issuance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'projections_created' => $projectionCount(),
    'canonical_licenses_created' => $licenseCount(),
    'license_type' => 'focusa_operator_lifetime_v1',
    'product' => 'focusa',
    'price_version' => 'focusa_operator_lifetime_v1.697.00.v1',
    'price_usd' => '697.00',
    'family_count' => count(FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES),
    'family_digest' => FocusaSpec172FocusaOperatorProjector::familyDigest(),
    'operator_seats' => 1,
    'node_limit' => 3,
    'node_set' => 'operator_shared_v1',
    'term' => 'lifetime',
    'sequence' => $projectedA['sequence'],
    'paid_lease_fixture' => 'derived_from_projection_bounded_credential',
    'duplicate_issuance_fixtures' => ['idempotent_replay', 'duplicate_projection_call', 'wrong_product_uiai', 'wrong_price', 'wrong_account', 'refunded', 'revoked', 'pending', 'pre_issuance', 'revoked_license', 'tampered_license', 'frozen_checkout_disabled', 'drifted_mapping', 'caller_commerce_fields', 'idempotency_conflict'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
