<?php
// 172.02.05 Issue UIAI Operator Lifetime v1 with hosted-resource exclusions.
// The UIAI Operator projector consumes exactly one issued canonical EDD Software
// Licensing key for one verified complete eligible UIAI order and projects
// uiai_operator_lifetime_v1 from canonical EDD truth: the canonical order row
// (complete, exact customer, exact verified-email digest), the canonical order item, the
// active canonical EDD license with the exact journaled key digest, and the server-owned
// dedicated Downloads offer (exact download, public code, License Type ref, price id,
// amount, lifetime, one seat, three shared nodes). One eligible item produces exactly one
// projection with the frozen local family digest, seat/node limits, price version, the
// frozen hosted-resource exclusion digest, and a strictly monotonic per-account
// sequence; replays return the identical decision and duplicate projection calls return
// the same projection with zero creations. Wrong product (focusa), wrong price, wrong
// account, refunded/revoked/pending orders, unissued requests, revoked/tampered
// licenses, frozen (checkout-disabled) offers, caller metadata grants, and idempotency
// conflicts create none. Focusa-only orders cannot obtain UIAI. The hosted-resource
// exclusion registry denies unlimited hosted compute, paid proxies, third-party API
// consumption, paid model usage, managed hosting, resale, redistribution, and product
// embedding with HOSTED_RESOURCE_NOT_INCLUDED; the UIAI grant/child-token fixture
// derives the bounded machine credential and the bounded node child token exclusively
// from the projection, carrying the explicit local/hosted boundary. No raw email, key,
// token, customer row, credential, or card data is stored or returned anywhere.
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
require_once $root . '/docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php';
require_once $root . '/docs/contracts/spec172-uiai-edd-license-type-projector.v1.php';
require_once $root . '/docs/contracts/spec172-uiai-grant-child-token-fixture.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_uiai(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_uiai_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);
$projectionMigration = new FocusaSpec172LicenseTypeProjectionMigration($db, 'wp_');
$projectionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'uiai_operator_issuance_test']);

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

$bindingService = new FocusaSpec152eEddOrderBindingService(
    $db, $bindingMigration, $registrations, $registrationSecrets, $accounts,
    $fixtureRegistry, $facadeRegistry, $clock,
);

$issuanceService = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $fixtureRegistry, $clock,
);

$uiaiProjectorFrozen = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $frozenDedicated, $clock,
);

// Checkout-disabled fixture: the dedicated offer resolves (download 1002) but checkout
// is not enabled, so UIAI projection fails closed with EDD_CHECKOUT_REQUIRED.
$blockedDedicated = $fixtureDedicated;
foreach ($blockedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'uiai_operator_lifetime_v1') {
        $record['checkout_enabled'] = false;
        $record['sale_status'] = 'approved_not_yet_enabled';
    }
}
unset($record);
$uiaiProjectorBlocked = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $blockedDedicated, $clock,
);
$uiaiProjector = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $fixtureDedicated, $clock,
);

// Mutated fixture: the uiai offer's download/license-type mapping no longer matches
// the settled item (PRODUCT_MAPPING_REQUIRED at projection).
$mismatchedDedicated = $fixtureDedicated;
foreach ($mismatchedDedicated['records'] as &$record) {
    if ($record['public_code'] === 'uiai_operator_lifetime_v1') {
        $record['edd_download_id'] = 9999;
    }
}
unset($record);
$uiaiProjectorMismatched = new UiaiSpec172UiaiOperatorProjector(
    $db, $projectionMigration, $issuanceMigration, $bindingMigration, $registrations,
    $registrationSecrets, $accounts, $edd, $mismatchedDedicated, $clock,
);

// The Focusa projector (prior atom) is reused as the cross-product guard: a UIAI
// license can never project focusa_operator_lifetime_v1 and vice versa.
$focusaProjector = new FocusaSpec172FocusaOperatorProjector(
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
        'presenter' => 'candidate.uiai.operator.issuance.test',
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
        'migration_provenance' => ['source' => 'spec172_candidate', 'record' => 'uiai-operator-' . $tag . '-' . $seq],
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

$uiaiProject = static function (string $handle, string $requestId, string $idempotencyKey) use ($uiaiProjector): array {
    return $uiaiProjector->project([
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

// ── Frozen contracts and exclusion registry remain canonical ──────────

expect_uiai($frozenDedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1', 'frozen dedicated downloads schema');
expect_uiai($frozenDedicated['owner'] === 'WPUIAI/wpuiai', 'frozen dedicated downloads owner');
expect_uiai(count($frozenDedicated['records']) === 3, 'frozen dedicated downloads has three records');
expect_uiai($frozenDedicated['counts']['checkout_enabled'] === 0, 'frozen dedicated downloads checkout disabled');
$frozenUiai = null;
foreach ($frozenDedicated['records'] as $record) {
    if ($record['public_code'] === 'uiai_operator_lifetime_v1') {
        $frozenUiai = $record;
    }
}
expect_uiai($frozenUiai !== null, 'frozen dedicated downloads has the uiai record');
expect_uiai((int) $frozenUiai['amount_minor'] === 69700 && $frozenUiai['price_usd'] === '697.00', 'frozen uiai offer is 697.00');
expect_uiai((int) $frozenUiai['operator_seats'] === 1 && (int) $frozenUiai['node_limit'] === 3 && $frozenUiai['node_set'] === 'operator_shared_v1', 'frozen uiai offer one seat three shared nodes');
expect_uiai($frozenUiai['license_duration'] === 'lifetime', 'frozen uiai offer lifetime');
expect_uiai($projectionMigration::SCHEMA === 'focusa.spec172.license_type_projection.v1', 'projection schema is canonical');
expect_uiai(UiaiSpec172UiaiOperatorProjector::RESULT_SCHEMA === 'focusa.spec172.uiai_operator_lifetime_projection.v1', 'uiai projection result schema is canonical');
expect_uiai(UiaiSpec172UiaiOperatorProjector::familyDigest() !== '', 'frozen uiai family digest is deterministic');
expect_uiai(count(UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES) === 7, 'frozen family set has seven UIAI Operator v1 local families');
expect_uiai(UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES === ['uiai_public_observation', 'uiai_browser_action', 'uiai_persistence', 'uiai_diagnostics', 'uiai_proof_packets', 'uiai_batch_responsive', 'uiai_supported_integrations'], 'frozen family set matches Spec 172 section 7.2');

// Hosted-resource exclusion registry: explicit frozen deny list, fail-closed checks.
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::SCHEMA === 'focusa.spec172.uiai_hosted_resource_exclusion_registry.v1', 'hosted-resource exclusion registry schema is canonical');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::PRODUCT === 'uiai_engine', 'exclusion registry product is uiai_engine');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::LICENSE_TYPE === 'uiai_operator_lifetime_v1', 'exclusion registry license type is canonical');
expect_uiai(count(UiaiSpec172HostedResourceExclusionRegistry::EXCLUSIONS) === 8, 'exclusion registry freezes eight hosted/metered exclusions');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::exclusionList() === ['unlimited_hosted_compute', 'paid_proxies', 'third_party_api_consumption', 'paid_model_usage', 'managed_hosting', 'resale', 'redistribution', 'product_embedding'], 'exclusion registry lists the exact Spec 172 section 7.2 exclusions');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::GRANTED === [], 'exclusion registry grants zero hosted resources in v1');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::digest() !== '', 'exclusion digest is deterministic');
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::digest() === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'exclusion digest is stable across calls');

// ── Positive: one eligible UIAI order -> one key + one projection ─────

$regA = $createRegistration('operator.uiai@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiai-alpha');
$customerA = $customerOf($regA['registration_uuid']);
$insertOrder(5001, 'complete', $customerA, 'operator.uiai@example.invalid', [
    ['item_id' => 5001, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5001, $GATEWAY, 'txn_pay_5001');

$bound = $bind(5001, $regA['registration_uuid'], $customerA, [['item_id' => 5001, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5001', 'uiai-alpha-1');
expect_uiai($bound['decision'] === 'order_bound' && $bound['issuance_requests_settled'] === 1, 'eligible UIAI order settles exactly one issuance request');
$handleA = $bound['protected_items'][0]['issuance_request_handle'];
expect_uiai(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $handleA) === 1, 'issuance request handle is an opaque bounded token');

$issuedA = $issue($handleA, 'req-issue-uiai-alpha-1', 'idem-issue-uiai-alpha-1');
expect_uiai($issuedA['decision'] === 'license_issued' && $issuedA['keys_created'] === 1, 'one eligible UIAI item issues exactly one canonical key');
expect_uiai($issuedA['product_code'] === $UIAI_PRODUCT && $issuedA['license_type_ref'] === $UIAI_PRODUCT, 'issued key carries the server-owned UIAI offer');
$keyA = $issuedA['delivery']['license_key'];
expect_uiai(preg_match($KEY_PATTERN, $keyA) === 1, 'issued key is canonical EDD SL format');
expect_uiai(str_starts_with($keyA, 'focusa_live_') === false, 'issued key is never a synthetic key');

$projectedA = $uiaiProject($handleA, 'req-project-uiai-alpha-1', 'idem-project-uiai-alpha-1');
expect_uiai($projectedA['schema'] === 'focusa.spec172.uiai_operator_lifetime_projection.v1', 'projection schema is canonical');
expect_uiai($projectedA['decision'] === 'license_type_projected', 'projection decision is license_type_projected');
expect_uiai($projectedA['existing'] === false && $projectedA['projections_created'] === 1, 'first projection creates exactly one projection');
expect_uiai($projectedA['product'] === 'uiai_engine' && $projectedA['license_type'] === 'uiai_operator_lifetime_v1' && $projectedA['grant'] === 'uiai_operator_lifetime_v1', 'projection carries the UIAI Operator Lifetime v1 grant');
expect_uiai($projectedA['registration_id'] === $regA['registration_uuid'], 'projection is linked to the registration');
expect_uiai($projectedA['account_id'] !== '', 'projection is linked to the account');
expect_uiai($projectedA['customer_id'] === $customerA, 'projection is linked to the EDD customer');
expect_uiai($projectedA['order_id'] === 5001 && $projectedA['order_item_id'] === 5001, 'projection is linked to the canonical order item');
expect_uiai($projectedA['download_id'] === $UIAI_DOWNLOAD, 'projection carries the canonical download');
expect_uiai(is_int($projectedA['edd_license_id']) && $projectedA['edd_license_id'] === $issuedA['edd_license_id'], 'projection references the canonical issued license');
expect_uiai($projectedA['issuance'] === 'canonical_edd_software_licensing', 'projection derives from canonical EDD Software Licensing');
expect_uiai($projectedA['license_key_digest'] === $issuedA['license_key_digest'], 'projection carries the keyed license digest');
expect_uiai(strpos((string) $projectedA['license_key_mask'], '********-********-********-') === 0, 'projection carries only a masked license key');

// Frozen family digest, seat/node limits, price version, sequence, hosted boundary.
expect_uiai($projectedA['family_digest'] === UiaiSpec172UiaiOperatorProjector::familyDigest(), 'projection carries the frozen family digest');
expect_uiai($projectedA['family_count'] === 7, 'projection freezes seven families');
expect_uiai($projectedA['operator_seats'] === 1, 'projection freezes one operator seat');
expect_uiai($projectedA['node_limit'] === 3 && $projectedA['node_set'] === 'operator_shared_v1', 'projection freezes three shared operator nodes');
expect_uiai($projectedA['term'] === 'lifetime' && $projectedA['status'] === 'active', 'projection is an active lifetime grant');
expect_uiai($projectedA['price_version'] === 'uiai_operator_lifetime_v1.697.00.v1', 'projection carries the server-owned price version');
expect_uiai($projectedA['price_usd'] === '697.00' && $projectedA['amount_minor'] === 69700, 'projection carries the canonical price');
expect_uiai(is_int($projectedA['sequence']) && $projectedA['sequence'] === 1, 'projection carries the first monotonic sequence');
expect_uiai($accountSequence($regA['registration_uuid']) === 1, 'authority account sequence advanced to 1');
expect_uiai($projectedA['hosted_resource_exclusions'] === UiaiSpec172HostedResourceExclusionRegistry::exclusionList(), 'projection carries the frozen hosted-resource exclusion list');
expect_uiai($projectedA['hosted_resource_exclusion_digest'] === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'projection carries the frozen hosted-resource exclusion digest');
expect_uiai($projectedA['hosted_resources_included'] === [], 'projection grants zero hosted resources');

expect_uiai($projectionCount() === 1, 'exactly one projection journal row');
$projectionRowA = $uiaiProjector->findByIssuanceRequestKey($handleA);
expect_uiai($projectionRowA !== null && $projectionRowA['status'] === 'active', 'projection journal row is active');
expect_uiai(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRowA['projection_key']) === 1, 'projection handles are opaque bounded tokens');
expect_uiai($projectionRowA['price_version'] === 'uiai_operator_lifetime_v1.697.00.v1', 'projection journal carries the price version');
expect_uiai($projectionRowA['family_digest'] === UiaiSpec172UiaiOperatorProjector::familyDigest(), 'projection journal carries the frozen family digest');
expect_uiai((int) $projectionRowA['operator_seats'] === 1 && (int) $projectionRowA['node_limit'] === 3 && $projectionRowA['node_set'] === 'operator_shared_v1', 'projection journal carries seat and node limits');
expect_uiai((int) $projectionRowA['sequence'] === 1, 'projection journal carries the sequence');
expect_uiai($projectionRowA['license_type_ref'] === $UIAI_PRODUCT && $projectionRowA['product_code'] === 'uiai_engine', 'projection journal carries the canonical product and License Type');
expect_uiai($uiaiProjector->findByProjectionKey((string) $projectionRowA['projection_key'])['issuance_request_key'] === $handleA, 'projection lookup by handle resolves the source request');

// Registration fulfillment (from SL issuance) is preserved: entitlement_issued.
$regRowA = $registrations->findByUuid($regA['registration_uuid']);
expect_uiai($regRowA['state'] === 'entitlement_issued', 'registration is at entitlement_issued');
expect_uiai((int) $regRowA['edd_license_id'] === $issuedA['edd_license_id'], 'registration references the canonical issued license');

// Idempotent replay: same key returns the identical decision, no second projection.
$replayedA = $uiaiProject($handleA, 'req-project-uiai-alpha-1', 'idem-project-uiai-alpha-1');
expect_uiai(json_encode($replayedA, JSON_THROW_ON_ERROR) === json_encode($projectedA, JSON_THROW_ON_ERROR), 'idempotency replay returns the identical decision');
expect_uiai($projectionCount() === 1, 'replay creates no second projection row');
expect_uiai($accountSequence($regA['registration_uuid']) === 1, 'replay does not bump the sequence');

// Duplicate projection call with a NEW idempotency key: same projection, zero new.
$duplicateA = $uiaiProject($handleA, 'req-project-uiai-alpha-retry-1', 'idem-project-uiai-alpha-retry-1');
expect_uiai($duplicateA['existing'] === true, 'duplicate projection call is an existing projection');
expect_uiai($duplicateA['projections_created'] === 0, 'duplicate projection call creates zero projections');
expect_uiai($duplicateA['edd_license_id'] === $issuedA['edd_license_id'], 'duplicate projection call returns the same license reference');
expect_uiai($duplicateA['sequence'] === 1 && $duplicateA['family_digest'] === $projectedA['family_digest'], 'duplicate projection call returns the identical grant');
expect_uiai($projectionCount() === 1, 'duplicate projection call never creates a second projection');
expect_uiai($accountSequence($regA['registration_uuid']) === 1, 'duplicate projection call never bumps the sequence');

// ── UIAI grant/child-token fixture derives from the projection ─────────

$fixtureA = UiaiSpec172UiaiGrantChildTokenFixture::fromProjection($projectedA, 'node-uiai-001', 'client-uiai-001', $clock);
expect_uiai($fixtureA['schema'] === 'focusa.spec172.uiai_grant_child_token_fixture.v1', 'grant/child-token fixture schema is canonical');
$grantA = $fixtureA['grant'];
expect_uiai($grantA['schema'] === 'focusa.uiai_grant.v1', 'grant payload schema is canonical');
expect_uiai($grantA['product'] === 'uiai_engine', 'grant payload product is uiai_engine');
expect_uiai($grantA['subject_id'] === $projectedA['account_id'], 'grant payload subject is the projected account');
expect_uiai($grantA['node_id'] === 'node-uiai-001', 'grant payload binds the operator node');
expect_uiai((int) $grantA['sequence'] === 1, 'grant payload carries the projected sequence');
expect_uiai($grantA['status'] === 'active', 'grant payload is active for this sequence');
expect_uiai($grantA['authority_key_id'] !== '', 'grant payload names the authority signing key');
expect_uiai(count($grantA['features']) === 7, 'grant payload carries all seven frozen local families');
foreach ($grantA['features'] as $family => $enabled) {
    expect_uiai($enabled === true && in_array($family, UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES, true), "grant payload family {$family} is enabled and frozen");
}
expect_uiai((int) $grantA['limits']['operator_seats'] === 1 && (int) $grantA['limits']['node_limit'] === 3, 'grant payload carries one seat and three nodes');
expect_uiai((string) $grantA['expires_at'] > (string) $grantA['issued_at'], 'grant credential lifetime is bounded (never perpetual)');
expect_uiai((string) $grantA['offline_grace_until'] > (string) $grantA['expires_at'], 'offline grace is bounded past the refresh window');
expect_uiai(count($grantA['hosted_resources']) === 8, 'grant payload carries all eight hosted/metered exclusions');
foreach ($grantA['hosted_resources'] as $resource => $granted) {
    expect_uiai($granted === false && in_array($resource, UiaiSpec172HostedResourceExclusionRegistry::exclusionList(), true), "hosted resource {$resource} is explicitly denied");
}
expect_uiai($grantA['hosted_resource_exclusion_digest'] === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'grant payload carries the frozen exclusion digest');
$childTokenA = $fixtureA['child_token'];
expect_uiai($childTokenA['schema'] === 'focusa.uiai_child_token.v1', 'child token schema matches the runtime child-token schema');
expect_uiai($childTokenA['node_id'] === 'node-uiai-001' && $childTokenA['client_id'] === 'client-uiai-001', 'child token is node/client-scoped');
expect_uiai($childTokenA['grant_lease_id'] === $grantA['grant_id'] && (int) $childTokenA['grant_sequence'] === (int) $grantA['sequence'], 'child token binds the exact grant lease and sequence');
expect_uiai(count($childTokenA['features']) === 7 && count(array_diff($childTokenA['features'], UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES)) === 0, 'child token features are an exact subset of the grant');
expect_uiai((int) $childTokenA['limits']['operator_seats'] === 1 && (int) $childTokenA['limits']['node_limit'] === 3, 'child token carries the same seat/node limits');
expect_uiai($childTokenA['hosted_resource_exclusion_digest'] === UiaiSpec172HostedResourceExclusionRegistry::digest(), 'child token carries the frozen exclusion digest');
expect_uiai((string) $childTokenA['expires_at'] > (string) $childTokenA['issued_at'], 'child token has a positive bounded lifetime');
$childIssued = new DateTimeImmutable($childTokenA['issued_at'], new DateTimeZone('UTC'));
$childExpires = new DateTimeImmutable($childTokenA['expires_at'], new DateTimeZone('UTC'));
$childMax = $childIssued->modify('+15 minutes');
expect_uiai($childExpires <= $childMax, 'child token TTL never exceeds the 15-minute runtime bound');
$metaA = $fixtureA['grant_metadata'];
expect_uiai($metaA['license_type'] === 'uiai_operator_lifetime_v1' && $metaA['price_version'] === 'uiai_operator_lifetime_v1.697.00.v1', 'grant fixture carries explicit grant metadata');
expect_uiai($metaA['family_digest'] === UiaiSpec172UiaiOperatorProjector::familyDigest(), 'grant fixture carries the frozen family digest');
expect_uiai($metaA['term'] === 'lifetime' && $metaA['node_set'] === 'operator_shared_v1', 'grant fixture carries lifetime term and shared node set');
expect_uiai($metaA['refund_policy'] === 'whole_order_30_days', 'grant fixture carries the whole-order refund policy');
expect_uiai($fixtureA['hosted_resource_exclusions'] === UiaiSpec172HostedResourceExclusionRegistry::exclusionList(), 'fixture carries the canonical exclusion list');

// validate() accepts the derived fixture and rejects tampering.
$validated = UiaiSpec172UiaiGrantChildTokenFixture::validate($fixtureA, $projectedA);
expect_uiai($validated === null, 'fixture validation passes for the derived fixture');
$tamperedFixture = $fixtureA;
$tamperedFixture['grant']['limits']['node_limit'] = 99;
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_LIMIT_MISMATCH',
    'tampered node limit fails fixture validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant_metadata']['family_digest'] = str_repeat('0', 64);
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_GRANT_MISMATCH',
    'tampered family digest fails fixture validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant']['hosted_resources']['unlimited_hosted_compute'] = true;
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_HOSTED_RESOURCE_MISMATCH',
    'a granted hosted resource fails fixture validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['grant']['hosted_resource_exclusion_digest'] = str_repeat('0', 64);
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_HOSTED_RESOURCE_MISMATCH',
    'a tampered hosted-resource exclusion digest fails fixture validation',
);
$tamperedFixture = $fixtureA;
$tamperedFixture['child_token']['expires_at'] = $childIssued->modify('+16 minutes')->format('Y-m-d\TH:i:s\Z');
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::validate($tamperedFixture, $projectedA),
    'FIXTURE_CREDENTIAL_WINDOW_INVALID',
    'a child token past the 15-minute bound fails fixture validation',
);
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::fromProjection($issuedA, 'node-uiai-001', 'client-uiai-001', $clock),
    'LICENSE_TYPE_PROJECTION_REQUIRED',
    'the grant fixture requires exactly an accepted projection, never an issuance decision',
);
expect_uiai_throws(
    fn() => UiaiSpec172UiaiGrantChildTokenFixture::fromProjection($projectedA, 'node@raw.example', 'client-uiai-001', $clock),
    'bounded node id required',
    'raw email node ids are rejected in the grant fixture',
);

// ── Hosted-resource exclusion registry: excluded rights remain denied ──

foreach (UiaiSpec172HostedResourceExclusionRegistry::EXCLUSIONS as $resource => $reason) {
    expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::isIncluded($resource) === false, "hosted resource {$resource} is never included");
    expect_uiai_throws(
        fn() => UiaiSpec172HostedResourceExclusionRegistry::assertIncluded($resource),
        'HOSTED_RESOURCE_NOT_INCLUDED',
        "hosted resource {$resource} denies with HOSTED_RESOURCE_NOT_INCLUDED",
    );
}
expect_uiai(UiaiSpec172HostedResourceExclusionRegistry::isIncluded('unknown_metered_capacity') === false, 'unknown hosted resources are denied by default');
expect_uiai_throws(
    fn() => UiaiSpec172HostedResourceExclusionRegistry::assertIncluded('unknown_metered_capacity'),
    'HOSTED_RESOURCE_NOT_INCLUDED',
    'unknown hosted resources fail closed',
);
expect_uiai_throws(
    fn() => UiaiSpec172HostedResourceExclusionRegistry::assertIncluded('paid_model_usage'),
    'HOSTED_RESOURCE_NOT_INCLUDED',
    'paid model usage is never included in the UIAI Operator Lifetime v1 License Type',
);

// ── Negative: wrong product creates no UIAI projection ────────────────

// A fully eligible Focusa order issues its own canonical Focusa key (generic SL
// issuance) but can never project a UIAI Operator Lifetime v1 grant: Focusa-only
// orders cannot obtain UIAI.
$regFocusa = $createRegistration('operator.focusa@example.invalid', $FACADE, $FOCUSA_PRODUCT, 'focusa');
$customerFocusa = $customerOf($regFocusa['registration_uuid']);
$insertOrder(5002, 'complete', $customerFocusa, 'operator.focusa@example.invalid', [
    ['item_id' => 5002, 'download' => $FOCUSA_DOWNLOAD],
]);
$insertTransaction(5002, $GATEWAY, 'txn_pay_5002');
$boundFocusa = $bind(5002, $regFocusa['registration_uuid'], $customerFocusa, [['item_id' => 5002, 'download' => $FOCUSA_DOWNLOAD, 'price' => $FOCUSA_PRICE]], 'txn_pay_5002', 'focusa-1');
expect_uiai($boundFocusa['issuance_requests_settled'] === 1, 'Focusa order settles its own issuance request');
$handleFocusa = $boundFocusa['protected_items'][0]['issuance_request_handle'];
$issuedFocusa = $issue($handleFocusa, 'req-issue-focusa-1', 'idem-issue-focusa-1');
expect_uiai($issuedFocusa['keys_created'] === 1 && $issuedFocusa['license_type_ref'] === $FOCUSA_PRODUCT, 'Focusa item issues its own canonical Focusa key');
expect_uiai_throws(
    fn() => $uiaiProject($handleFocusa, 'req-project-focusa-1', 'idem-project-focusa-1'),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a Focusa license can never project uiai_operator_lifetime_v1',
);
expect_uiai($projectionCount() === 1, 'wrong-product denial creates zero UIAI projections');
expect_uiai($accountSequence($regFocusa['registration_uuid']) === 0, 'wrong-product denial never advances the account sequence');

// Cross-product guard (reusing the prior atom's Focusa projector): a UIAI license can
// never project focusa_operator_lifetime_v1. An issued-but-not-yet-projected UIAI order
// fails closed in the Focusa projector with LICENSE_TYPE_NOT_INCLUDED.
$regUiaiFresh = $createRegistration('operator.uiaifresh@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaifresh');
$customerUiaiFresh = $customerOf($regUiaiFresh['registration_uuid']);
$insertOrder(5021, 'complete', $customerUiaiFresh, 'operator.uiaifresh@example.invalid', [
    ['item_id' => 5021, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5021, $GATEWAY, 'txn_pay_5021');
$boundUiaiFresh = $bind(5021, $regUiaiFresh['registration_uuid'], $customerUiaiFresh, [['item_id' => 5021, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5021', 'uiaifresh-1');
expect_uiai($boundUiaiFresh['issuance_requests_settled'] === 1, 'fresh UIAI order settles its own issuance request');
$handleUiaiFresh = $boundUiaiFresh['protected_items'][0]['issuance_request_handle'];
$issuedUiaiFresh = $issue($handleUiaiFresh, 'req-issue-uiaifresh-1', 'idem-issue-uiaifresh-1');
expect_uiai($issuedUiaiFresh['keys_created'] === 1 && $issuedUiaiFresh['license_type_ref'] === $UIAI_PRODUCT, 'fresh UIAI item issues its own canonical UIAI key');
expect_uiai_throws(
    fn() => $focusaProjector->project([
        'issuance_request_handle' => $handleUiaiFresh,
        'request_id' => 'req-project-focusa-from-uiai-2',
        'idempotency_key' => 'idem-project-focusa-from-uiai-2',
    ]),
    'LICENSE_TYPE_NOT_INCLUDED',
    'a UIAI license can never project focusa_operator_lifetime_v1',
);
expect_uiai($projectionCount() === 1, 'cross-product denial creates zero projections');

// The shared projection journal never fabricates the other product's grant: replaying
// the already-projected UIAI handle through the Focusa projector returns the existing
// UIAI projection (product uiai_engine, license type uiai_operator_lifetime_v1), never
// a Focusa grant, and creates no second row.
$focusaReplayOfUiai = $focusaProjector->project([
    'issuance_request_handle' => $handleA,
    'request_id' => 'req-project-focusa-from-uiai-3',
    'idempotency_key' => 'idem-project-focusa-from-uiai-3',
]);
expect_uiai($focusaReplayOfUiai['existing'] === true, 'shared journal replay marks the projection existing');
expect_uiai(($focusaReplayOfUiai['product'] ?? '') === 'uiai_engine' && ($focusaReplayOfUiai['license_type'] ?? '') === 'uiai_operator_lifetime_v1', 'replay returns the existing UIAI projection, never a Focusa grant');
expect_uiai($projectionCount() === 1, 'replay creates no second projection row');

// ── Negative: wrong price creates no projection ───────────────────────

// The binding/issuance use the correct price; the settled binding price is mutated
// after issuance so the projection-time price check must fail closed.
$regPrice = $createRegistration('operator.uiaiprice@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaiprice');
$customerPrice = $customerOf($regPrice['registration_uuid']);
$insertOrder(5003, 'complete', $customerPrice, 'operator.uiaiprice@example.invalid', [
    ['item_id' => 5003, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5003, $GATEWAY, 'txn_pay_5003');
$boundPrice = $bind(5003, $regPrice['registration_uuid'], $customerPrice, [['item_id' => 5003, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5003', 'uiaiprice-1');
$handlePrice = $boundPrice['protected_items'][0]['issuance_request_handle'];
$issuedPrice = $issue($handlePrice, 'req-issue-uiaiprice-1', 'idem-issue-uiaiprice-1');
expect_uiai($issuedPrice['keys_created'] === 1, 'correct-price item issues its key');
$db->exec("UPDATE wp_wpuiai_edd_order_bindings SET price_id = 'price_wrong' WHERE binding_key = (SELECT binding_key FROM wp_wpuiai_edd_issuance_requests WHERE issuance_request_key = '{$handlePrice}')");
expect_uiai_throws(
    fn() => $uiaiProject($handlePrice, 'req-project-uiaiprice-1', 'idem-project-uiaiprice-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a settled item whose price no longer matches the dedicated offer fails closed',
);
expect_uiai($projectionCount() === 1, 'wrong-price denial creates zero projections');

// A binding that never carried a matching price fails at the binding boundary.
$regPriceBind = $createRegistration('operator.uiaipricebind@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaipricebind');
$customerPriceBind = $customerOf($regPriceBind['registration_uuid']);
$insertOrder(5011, 'complete', $customerPriceBind, 'operator.uiaipricebind@example.invalid', [
    ['item_id' => 5011, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5011, $GATEWAY, 'txn_pay_5011');
expect_uiai_throws(
    fn() => $bind(5011, $regPriceBind['registration_uuid'], $customerPriceBind, [['item_id' => 5011, 'download' => $UIAI_DOWNLOAD, 'price' => 'price_wrong']], 'txn_pay_5011', 'uiaipricebind-1'),
    'PRODUCT_MAPPING_REQUIRED',
    'a wrong price_id can never settle an issuance request',
);

// ── Negative: wrong account creates no projection ─────────────────────

$regAccount = $createRegistration('operator.uiaiaccount@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaiaccount');
$customerAccount = $customerOf($regAccount['registration_uuid']);
$insertOrder(5004, 'complete', $customerAccount, 'operator.uiaiaccount@example.invalid', [
    ['item_id' => 5004, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5004, $GATEWAY, 'txn_pay_5004');
$boundAccount = $bind(5004, $regAccount['registration_uuid'], $customerAccount, [['item_id' => 5004, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5004', 'uiaiaccount-1');
$handleAccount = $boundAccount['protected_items'][0]['issuance_request_handle'];
$issuedAccount = $issue($handleAccount, 'req-issue-uiaiaccount-1', 'idem-issue-uiaiaccount-1');
expect_uiai($issuedAccount['keys_created'] === 1, 'account-fixture item issues its key');
$db->exec('UPDATE wp_edd_orders SET customer_id = 424242 WHERE id = 5004');
expect_uiai_throws(
    fn() => $uiaiProject($handleAccount, 'req-project-uiaiaccount-1', 'idem-project-uiaiaccount-1'),
    'EDD_ORDER_UNVERIFIED',
    'an order whose customer changed after settlement fails closed',
);
expect_uiai($projectionCount() === 1, 'wrong-account denial creates zero projections');

// ── Negative: canonical order truth at projection time ────────────────

// Refunded canonical order after issuance: fails closed, zero projection.
$regRefunded = $createRegistration('operator.uiairefunded@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiairefunded');
$customerRefunded = $customerOf($regRefunded['registration_uuid']);
$insertOrder(5005, 'complete', $customerRefunded, 'operator.uiairefunded@example.invalid', [
    ['item_id' => 5005, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5005, $GATEWAY, 'txn_pay_5005');
$boundRefunded = $bind(5005, $regRefunded['registration_uuid'], $customerRefunded, [['item_id' => 5005, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5005', 'uiairefunded-1');
$handleRefunded = $boundRefunded['protected_items'][0]['issuance_request_handle'];
$issuedRefunded = $issue($handleRefunded, 'req-issue-uiairefunded-1', 'idem-issue-uiairefunded-1');
expect_uiai($issuedRefunded['keys_created'] === 1, 'refunded-fixture item issues its key before the order is refunded');
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE id = 5005");
expect_uiai_throws(
    fn() => $uiaiProject($handleRefunded, 'req-project-uiairefunded-1', 'idem-project-uiairefunded-1'),
    'REFUNDED',
    'a refunded canonical order never projects',
);
expect_uiai($projectionCount() === 1, 'refunded-order denial creates zero projections');

// Revoked canonical order after issuance: fails closed, zero projection.
$regRevoked = $createRegistration('operator.uiairevoked@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiairevoked');
$customerRevoked = $customerOf($regRevoked['registration_uuid']);
$insertOrder(5006, 'complete', $customerRevoked, 'operator.uiairevoked@example.invalid', [
    ['item_id' => 5006, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5006, $GATEWAY, 'txn_pay_5006');
$boundRevoked = $bind(5006, $regRevoked['registration_uuid'], $customerRevoked, [['item_id' => 5006, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5006', 'uiairevoked-1');
$handleRevoked = $boundRevoked['protected_items'][0]['issuance_request_handle'];
$issuedRevoked = $issue($handleRevoked, 'req-issue-uiairevoked-1', 'idem-issue-uiairevoked-1');
expect_uiai($issuedRevoked['keys_created'] === 1, 'revoked-fixture item issues its key before the order is revoked');
$db->exec("UPDATE wp_edd_orders SET status = 'revoked' WHERE id = 5006");
expect_uiai_throws(
    fn() => $uiaiProject($handleRevoked, 'req-project-uiairevoked-1', 'idem-project-uiairevoked-1'),
    'REVOKED',
    'a revoked canonical order never projects',
);
expect_uiai($projectionCount() === 1, 'revoked-order denial creates zero projections');

// Canonical order row moved back to pending: EDD_ORDER_PENDING.
$regPending = $createRegistration('operator.uiaipending@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaipending');
$customerPending = $customerOf($regPending['registration_uuid']);
$insertOrder(5007, 'complete', $customerPending, 'operator.uiaipending@example.invalid', [
    ['item_id' => 5007, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5007, $GATEWAY, 'txn_pay_5007');
$boundPending = $bind(5007, $regPending['registration_uuid'], $customerPending, [['item_id' => 5007, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5007', 'uiaipending-1');
$handlePending = $boundPending['protected_items'][0]['issuance_request_handle'];
$issuedPending = $issue($handlePending, 'req-issue-uiaipending-1', 'idem-issue-uiaipending-1');
expect_uiai($issuedPending['keys_created'] === 1, 'pending-fixture item issues its key before the order moves back to pending');
$db->exec("UPDATE wp_edd_orders SET status = 'pending' WHERE id = 5007");
expect_uiai_throws(
    fn() => $uiaiProject($handlePending, 'req-project-uiaipending-1', 'idem-project-uiaipending-1'),
    'EDD_ORDER_PENDING',
    'a pending canonical order fails closed with EDD_ORDER_PENDING',
);
expect_uiai($projectionCount() === 1, 'pending-order denial creates zero projections');

// ── Negative: canonical license truth at projection time ──────────────

// Projection before issuance: the issuance request is still pending, no key, no projection.
$regNoIssue = $createRegistration('operator.uiainoissue@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiainoissue');
$customerNoIssue = $customerOf($regNoIssue['registration_uuid']);
$insertOrder(5008, 'complete', $customerNoIssue, 'operator.uiainoissue@example.invalid', [
    ['item_id' => 5008, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5008, $GATEWAY, 'txn_pay_5008');
$boundNoIssue = $bind(5008, $regNoIssue['registration_uuid'], $customerNoIssue, [['item_id' => 5008, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5008', 'uiainoissue-1');
$handleNoIssue = $boundNoIssue['protected_items'][0]['issuance_request_handle'];
expect_uiai_throws(
    fn() => $uiaiProject($handleNoIssue, 'req-project-uiainoissue-1', 'idem-project-uiainoissue-1'),
    'EDD_LICENSE_UNUSABLE',
    'no canonical key, no projection',
);
expect_uiai($projectionCount() === 1, 'pre-issuance projection creates zero projections');

// License revoked after issuance: the license row is no longer active, zero projection.
$regLicense = $createRegistration('operator.uiailicense@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiailicense');
$customerLicense = $customerOf($regLicense['registration_uuid']);
$insertOrder(5009, 'complete', $customerLicense, 'operator.uiailicense@example.invalid', [
    ['item_id' => 5009, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5009, $GATEWAY, 'txn_pay_5009');
$boundLicense = $bind(5009, $regLicense['registration_uuid'], $customerLicense, [['item_id' => 5009, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5009', 'uiailicense-1');
$handleLicense = $boundLicense['protected_items'][0]['issuance_request_handle'];
$issuedLicense = $issue($handleLicense, 'req-issue-uiailicense-1', 'idem-issue-uiailicense-1');
expect_uiai($issuedLicense['keys_created'] === 1, 'license-fixture item issues its key');
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = {$issuedLicense['edd_license_id']}");
expect_uiai_throws(
    fn() => $uiaiProject($handleLicense, 'req-project-uiailicense-1', 'idem-project-uiailicense-1'),
    'EDD_LICENSE_UNUSABLE',
    'a revoked canonical license never projects',
);
expect_uiai($projectionCount() === 1, 'revoked-license denial creates zero projections');

// License key tampered after issuance: the journaled digest no longer matches.
$regTamper = $createRegistration('operator.uiaitamper@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaitamper');
$customerTamper = $customerOf($regTamper['registration_uuid']);
$insertOrder(5010, 'complete', $customerTamper, 'operator.uiaitamper@example.invalid', [
    ['item_id' => 5010, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5010, $GATEWAY, 'txn_pay_5010');
$boundTamper = $bind(5010, $regTamper['registration_uuid'], $customerTamper, [['item_id' => 5010, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5010', 'uiaitamper-1');
$handleTamper = $boundTamper['protected_items'][0]['issuance_request_handle'];
$issuedTamper = $issue($handleTamper, 'req-issue-uiaitamper-1', 'idem-issue-uiaitamper-1');
expect_uiai($issuedTamper['keys_created'] === 1, 'tamper-fixture item issues its key');
$db->exec("UPDATE wp_edd_licenses SET license_key = '11111111-22222222-33333333-44444444' WHERE id = {$issuedTamper['edd_license_id']}");
expect_uiai_throws(
    fn() => $uiaiProject($handleTamper, 'req-project-uiaitamper-1', 'idem-project-uiaitamper-1'),
    'EDD_LICENSE_UNUSABLE',
    'a tampered canonical license never projects',
);
expect_uiai($projectionCount() === 1, 'tampered-license denial creates zero projections');

// ── Negative: registry / offer authority ──────────────────────────────

// A checkout-disabled dedicated offer (the mapping resolves but is not enabled):
// EDD_CHECKOUT_REQUIRED.
$regFrozen = $createRegistration('operator.uiaifrozen@example.invalid', $FACADE, $UIAI_PRODUCT, 'uiaifrozen');
$customerFrozen = $customerOf($regFrozen['registration_uuid']);
$insertOrder(5012, 'complete', $customerFrozen, 'operator.uiaifrozen@example.invalid', [
    ['item_id' => 5012, 'download' => $UIAI_DOWNLOAD],
]);
$insertTransaction(5012, $GATEWAY, 'txn_pay_5012');
$boundFrozen = $bind(5012, $regFrozen['registration_uuid'], $customerFrozen, [['item_id' => 5012, 'download' => $UIAI_DOWNLOAD, 'price' => $UIAI_PRICE]], 'txn_pay_5012', 'uiaifrozen-1');
$handleFrozen = $boundFrozen['protected_items'][0]['issuance_request_handle'];
$issuedFrozen = $issue($handleFrozen, 'req-issue-uiaifrozen-1', 'idem-issue-uiaifrozen-1');
expect_uiai($issuedFrozen['keys_created'] === 1, 'frozen-fixture item issues its key');
expect_uiai_throws(
    fn() => $uiaiProjectorBlocked->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-uiaifrozen-1',
        'idempotency_key' => 'idem-project-uiaifrozen-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'a checkout-disabled dedicated offer denies projection until validation passes',
);
expect_uiai($projectionCount() === 1, 'checkout-disabled denial creates zero projections');

// The truly frozen dedicated Downloads contract has no fixture download binding at all:
// PRODUCT_MAPPING_REQUIRED.
expect_uiai_throws(
    fn() => $uiaiProjectorFrozen->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-uiaifrozen-real-1',
        'idempotency_key' => 'idem-project-uiaifrozen-real-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'the frozen dedicated Downloads contract (no active fixture mapping) denies projection',
);
expect_uiai($projectionCount() === 1, 'frozen-offer denial creates zero projections');

// The dedicated offer download mapping drifted: PRODUCT_MAPPING_REQUIRED.
expect_uiai_throws(
    fn() => $uiaiProjectorMismatched->project([
        'issuance_request_handle' => $handleFrozen,
        'request_id' => 'req-project-uiaimismatch-1',
        'idempotency_key' => 'idem-project-uiaimismatch-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'an offer whose download mapping drifted fails closed',
);
expect_uiai($projectionCount() === 1, 'mismatched-offer denial creates zero projections');

// ── Negative: input validation and idempotency ─────────────────────────

$negativeChecks++;
try {
    $uiaiProject('not-a-handle', 'req-project-uiaimalformed-1', 'idem-project-uiaimalformed-1');
    fwrite(STDERR, "FAIL: malformed issuance request handles are rejected\n");
    exit(1);
} catch (InvalidArgumentException) {
    // expected: bounded handle required
}
expect_uiai_throws(
    fn() => $uiaiProject('ir_' . str_repeat('0', 32), 'req-project-uiaiunknown-1', 'idem-project-uiaiunknown-1'),
    'EDD_LICENSE_UNUSABLE',
    'unknown issuance request handles fail closed',
);
expect_uiai_throws(
    fn() => $uiaiProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-uiaiclientfields-1',
        'idempotency_key' => 'idem-project-uiaiclientfields-1',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled commerce fields are forbidden at projection',
);
expect_uiai_throws(
    fn() => $uiaiProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-uiaiclientfields-2',
        'idempotency_key' => 'idem-project-uiaiclientfields-2',
        'license_type_ref' => 'focusa_operator_lifetime_v1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied License Type metadata is forbidden at projection',
);
expect_uiai_throws(
    fn() => $uiaiProjector->project([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-project-uiaiclientfields-3',
        'idempotency_key' => 'idem-project-uiaiclientfields-3',
        'hosted_resources' => ['unlimited_hosted_compute'],
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied hosted-resource metadata is forbidden at projection',
);
expect_uiai_throws(
    fn() => $uiaiProject($handleFocusa, 'req-project-uiaiconflict-1', 'idem-project-uiai-alpha-1'),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// ── Rollback preservation and redaction ───────────────────────────────

$preserved = $projectionMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'uiai_operator_issuance_test', 'record' => 'rollback']);
expect_uiai($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_uiai((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_license_type_projection_schema_events')->fetchColumn() === 1, 'exactly one projection preservation event journaled');

$decisionJson = json_encode([$projectedA, $replayedA, $duplicateA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_uiai(strpos($decisionJson, '@') === false, 'no raw email in any projection decision');
expect_uiai(preg_match($KEY_SCAN_PATTERN, $decisionJson) !== 1, 'no full license key in any projection decision');

$projectionRows = $db->query('SELECT * FROM wp_wpuiai_license_type_projections')->fetchAll(PDO::FETCH_ASSOC);
$projectionJson = json_encode($projectionRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_uiai(strpos($projectionJson, '@') === false, 'no raw email in the projection journal');
expect_uiai(strpos($projectionJson, 'txn_pay_') === false, 'no raw payment transaction id in the projection journal');
expect_uiai(preg_match($KEY_SCAN_PATTERN, $projectionJson) !== 1, 'no full license key in the projection journal');
foreach ($projectionRows as $projectionRow) {
    expect_uiai(preg_match('/^(pr_)[0-9a-f]{32}$/D', (string) $projectionRow['projection_key']) === 1, 'projection handles are opaque bounded tokens');
    expect_uiai(preg_match('/^[0-9a-f]{64}$/D', (string) $projectionRow['family_digest']) === 1, 'family digest is a 64-hex digest');
    expect_uiai((int) $projectionRow['sequence'] === 1, 'the single projection keeps sequence 1');
    expect_uiai(strpos((string) $projectionRow['result_payload'], '"license_key"') === false, 'projection payloads never contain a raw license_key field');
    expect_uiai(preg_match($KEY_SCAN_PATTERN, (string) $projectionRow['result_payload']) !== 1, 'projection payloads never contain a full key');
    expect_uiai(strpos((string) $projectionRow['result_payload'], '@') === false, 'projection payloads never contain raw email');
}
$fixtureJson = json_encode([$fixtureA], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_uiai(strpos($fixtureJson, '@') === false, 'grant/child-token fixture carries no raw email');
expect_uiai(preg_match($KEY_SCAN_PATTERN, $fixtureJson) !== 1, 'grant/child-token fixture carries no full license key');
expect_uiai(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $fixtureJson) !== 1, 'grant/child-token fixture carries no payment key');
expect_uiai(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $fixtureJson) !== 1, 'grant/child-token fixture carries no card data');

// The grant payload itself (outside the fixture envelope) is also clean.
$grantJson = json_encode($grantA, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_uiai(strpos($grantJson, '@') === false && preg_match($KEY_SCAN_PATTERN, $grantJson) !== 1, 'grant payload carries no raw email or key');
$childTokenJson = json_encode($childTokenA, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_uiai(strpos($childTokenJson, '@') === false && preg_match($KEY_SCAN_PATTERN, $childTokenJson) !== 1, 'child token carries no raw email or key');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.uiai_operator_issuance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'projections_created' => $projectionCount(),
    'canonical_licenses_created' => $licenseCount(),
    'license_type' => 'uiai_operator_lifetime_v1',
    'product' => 'uiai_engine',
    'price_version' => 'uiai_operator_lifetime_v1.697.00.v1',
    'price_usd' => '697.00',
    'family_count' => count(UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES),
    'family_digest' => UiaiSpec172UiaiOperatorProjector::familyDigest(),
    'hosted_resource_exclusions' => count(UiaiSpec172HostedResourceExclusionRegistry::EXCLUSIONS),
    'hosted_resource_exclusion_digest' => UiaiSpec172HostedResourceExclusionRegistry::digest(),
    'operator_seats' => 1,
    'node_limit' => 3,
    'node_set' => 'operator_shared_v1',
    'term' => 'lifetime',
    'sequence' => $projectedA['sequence'],
    'grant_child_token_fixture' => 'derived_from_projection_bounded_credential_explicit_hosted_boundary',
    'duplicate_issuance_fixtures' => ['idempotent_replay', 'duplicate_projection_call', 'wrong_product_focusa', 'cross_product_guard', 'wrong_price', 'wrong_account', 'refunded', 'revoked', 'pending', 'pre_issuance', 'revoked_license', 'tampered_license', 'frozen_checkout_disabled', 'drifted_mapping', 'caller_commerce_fields', 'hosted_resource_exclusion', 'idempotency_conflict'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
