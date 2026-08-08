<?php
// 152E.02.05 Make EDD Software Licensing key canonical and remove duplicate issuance.
// The canonical issuance adapter consumes exactly one settled issuance request from the
// order-binding journal and produces exactly ONE canonical EDD Software Licensing key per
// eligible order item, linked to the verified account/registration and the canonical EDD
// order/item rows. Duplicate issuance is impossible: replays return the identical decision,
// re-issuance for an already-issued request returns the same key with existing=true and
// zero keys created, synthetic legacy keys (focusa_live_* and prefixes) block issuance and
// are preserved for migration, and the adapter never creates a synthetic-prefixed key.
// Registration fulfillment advances checkout_pending -> entitlement_issued with the
// canonical order/item/license references. Journals store only keyed digests and masked
// keys: the full key appears only inside the bounded fulfillment delivery envelope; no raw
// email, raw payment id, secret, or unmasked real-email evidence is stored or returned.
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

$positiveChecks = 0;
$negativeChecks = 0;

function expect_issuance(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_issuance_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_license_issuance_test']);

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

$issuanceFrozen = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $frozenRegistry, $clock,
);
$issuanceService = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $fixtureRegistry, $clock,
);

// Mutated fixture: the focusa offer exists but its download/license-type mapping no longer
// matches the settled item (PRODUCT_MAPPING_REQUIRED at issuance).
$mismatchedRegistry = $fixtureRegistry;
foreach ($mismatchedRegistry['protected_offers'] as &$offer) {
    if ($offer['public_code'] === 'focusa_operator_lifetime_v1') {
        $offer['edd_download_id'] = 9999;
        $offer['license_type_ref'] = 'uiai_operator_lifetime_v1';
    }
}
unset($offer);
$issuanceMismatched = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $mismatchedRegistry, $clock,
);

// Mutated fixture: the focusa offer is approved-policy but not checkout-enabled
// (EDD_CHECKOUT_REQUIRED at issuance).
$blockedRegistry = $fixtureRegistry;
foreach ($blockedRegistry['protected_offers'] as &$offer) {
    if ($offer['public_code'] === 'focusa_operator_lifetime_v1') {
        $offer['checkout_enabled'] = false;
    }
}
unset($offer);
$issuanceBlocked = new FocusaSpec152eEddLicenseIssuanceService(
    $db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd,
    $blockedRegistry, $clock,
);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true, bool $checkout = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.edd.license.issuance.test',
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
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'license-issuance-' . $tag . '-' . $seq],
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
            ':id' => (int) ($item['item_id'] ?? (200000 + $rowSeq)),
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

$insertLicense = static function (int $customerId, int $downloadId, int $orderId, string $key, string $status = 'active') use ($db): void {
    $statement = $db->prepare("INSERT INTO wp_edd_licenses
        (license_key, customer_id, user_id, product_id, order_id, license_length, license_unit,
         expiration, activation_count, activation_limit, status, date_created)
        VALUES (:key, :customer, NULL, :download, :order, NULL, NULL, NULL, 0, 3, :status, '2026-08-08T00:01:00Z')");
    $statement->execute([
        ':key' => $key,
        ':customer' => $customerId,
        ':download' => $downloadId,
        ':order' => $orderId,
        ':status' => $status,
    ]);
};

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$PRODUCT = 'focusa_operator_lifetime_v1';
$UIAPRODUCT = 'uiai_operator_lifetime_v1';
$DOWNLOAD = 1001;
$PRICE = 'price_focusa_op_v1';
$GATEWAY = 'stripe';
$TXN = 'txn_pay_1001';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';

$bind = static function (int $orderId, string $registrationUuid, int $customerId, array $items, string $txn, string $tag) use ($bindingService, $FACADE, $ORIGIN, $PRICE, $GATEWAY): array {
    return $bindingService->bindOrderComplete([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => array_map(static fn (array $item) => [
            'order_item_id' => (int) $item['item_id'],
            'download_id' => (int) $item['download'],
            'price_id' => $PRICE,
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

$bindingCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_order_bindings')->fetchColumn();
};
$requestCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_issuance_requests')->fetchColumn();
};
$requestStateCount = static function (string $state) use ($db): int {
    $statement = $db->prepare("SELECT COUNT(*) FROM wp_wpuiai_edd_issuance_requests WHERE state = :state");
    $statement->execute([':state' => $state]);
    return (int) $statement->fetchColumn();
};
$licenseCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

// ── Frozen registry invariants (generated contracts remain current) ────

expect_issuance($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema');
expect_issuance($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_issuance($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');
expect_issuance($issuanceMigration::SCHEMA === 'focusa.spec152e.edd_license_issuance.v1', 'issuance schema is canonical');

// ── Positive: one eligible order item -> exactly one canonical EDD SL key ─

$regA = $createRegistration('issue.alpha@example.invalid', $FACADE, $PRODUCT, 'alpha');
$customerA = $customerOf($regA['registration_uuid']);
$insertOrder(3001, 'complete', $customerA, 'issue.alpha@example.invalid', [
    ['item_id' => 3001, 'download' => $DOWNLOAD],
]);
$insertTransaction(3001, $GATEWAY, $TXN);

$bound = $bind(3001, $regA['registration_uuid'], $customerA, [['item_id' => 3001, 'download' => $DOWNLOAD]], $TXN, 'alpha-1');
expect_issuance($bound['decision'] === 'order_bound' && $bound['issuance_requests_settled'] === 1, 'eligible order settles exactly one issuance request');
expect_issuance($requestStateCount('pending') === 1, 'one pending issuance request journaled');
$handleA = $bound['protected_items'][0]['issuance_request_handle'];
expect_issuance(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $handleA) === 1, 'issuance request handle is an opaque bounded token');

$issuedA = $issue($handleA, 'req-issue-alpha-1', 'idem-issue-alpha-1');
expect_issuance($issuedA['schema'] === 'focusa.spec152e.edd_license_issuance_decision.v1', 'issuance decision schema is canonical');
expect_issuance($issuedA['decision'] === 'license_issued', 'first issuance resolves as license_issued');
expect_issuance($issuedA['existing'] === false, 'first issuance is not an existing issuance');
expect_issuance($issuedA['keys_created'] === 1, 'exactly one key created for one item');
expect_issuance($issuedA['registration_id'] === $regA['registration_uuid'], 'issuance is linked to the registration');
expect_issuance($issuedA['account_id'] !== '', 'issuance is linked to the account');
expect_issuance($issuedA['customer_id'] === $customerA, 'issuance is linked to the EDD customer');
expect_issuance($issuedA['order_id'] === 3001 && $issuedA['order_item_id'] === 3001, 'issuance is linked to the canonical order item');
expect_issuance($issuedA['download_id'] === $DOWNLOAD, 'issuance carries the canonical download');
expect_issuance($issuedA['product_code'] === $PRODUCT && $issuedA['license_type_ref'] === $PRODUCT, 'issuance carries the server-owned offer product');
expect_issuance($issuedA['issuance'] === 'canonical_edd_software_licensing', 'issuance is canonical EDD Software Licensing');
expect_issuance(is_int($issuedA['edd_license_id']) && $issuedA['edd_license_id'] > 0, 'canonical EDD license id returned');
expect_issuance(preg_match('/^[0-9a-f]{64}$/D', (string) $issuedA['license_key_digest']) === 1, 'license key digest is a 64-hex keyed digest');
expect_issuance(strpos((string) $issuedA['license_key_mask'], '********-********-********-') === 0, 'license key mask is a non-reversible mask');
expect_issuance(preg_match($KEY_PATTERN, (string) $issuedA['delivery']['license_key']) === 1, 'delivered key is a canonical EDD SL format key');
expect_issuance(str_starts_with((string) $issuedA['delivery']['license_key'], 'focusa_live_') === false, 'delivered key is never a synthetic focusa_live key');
expect_issuance($issuedA['delivery']['schema'] === 'focusa.spec152e.edd_license_delivery.v1', 'delivery envelope uses the canonical delivery schema');
$keyA = $issuedA['delivery']['license_key'];
expect_issuance(hash('sha256', "focusa.spec152e.edd_license_issuance.key.v1\0" . $keyA) === $issuedA['license_key_digest'], 'delivered key matches the journaled digest');

expect_issuance($licenseCount() === 1, 'exactly one canonical license row in wp_edd_licenses');
$licenseRowA = $db->query('SELECT * FROM wp_edd_licenses')->fetch(PDO::FETCH_ASSOC);
expect_issuance($licenseRowA['license_key'] === $keyA, 'the single license row is the issued canonical key');
expect_issuance((int) $licenseRowA['customer_id'] === $customerA && (int) $licenseRowA['order_id'] === 3001 && (int) $licenseRowA['product_id'] === $DOWNLOAD, 'license row links customer/order/download');
expect_issuance($licenseRowA['status'] === 'active', 'issued license is active');
expect_issuance((int) $licenseRowA['activation_limit'] === 3, 'license activation limit comes from the server-owned offer');
expect_issuance($requestStateCount('issued') === 1 && $requestStateCount('pending') === 0, 'issuance request transitions pending -> issued');
expect_issuance($issuanceService->issuanceCount() === 1, 'exactly one issuance journal row');
$issuanceRowA = $issuanceService->findByIssuanceRequestKey($handleA);
expect_issuance($issuanceRowA !== null && $issuanceRowA['state'] === 'issued', 'issuance journal row is issued');
expect_issuance($issuanceRowA['edd_license_id'] == $issuedA['edd_license_id'], 'issuance journal links the canonical EDD license id');
expect_issuance($issuanceRowA['license_key_digest'] === $issuedA['license_key_digest'], 'issuance journal stores only the keyed digest');
expect_issuance(preg_match('/^(ki_)[0-9a-f]{32}$/D', (string) $issuanceRowA['issuance_key']) === 1, 'issuance handles are opaque bounded tokens');
expect_issuance($issuanceService->findByIssuanceKey((string) $issuanceRowA['issuance_key'])['issuance_request_key'] === $handleA, 'issuance lookup by handle resolves the source request');

// Registration fulfillment: checkout_pending -> entitlement_issued with the canonical refs.
$regRowA = $registrations->findByUuid($regA['registration_uuid']);
expect_issuance($regRowA['state'] === 'entitlement_issued', 'registration advances to entitlement_issued');
expect_issuance((int) $regRowA['edd_order_id'] === 3001 && (int) $regRowA['edd_order_item_id'] === 3001, 'registration carries the canonical order item reference');
expect_issuance((int) $regRowA['edd_license_id'] === $issuedA['edd_license_id'], 'registration carries the canonical EDD license reference');

// Idempotent replay: same key returns the identical decision, no second key.
$replayedA = $issue($handleA, 'req-issue-alpha-1', 'idem-issue-alpha-1');
expect_issuance(json_encode($replayedA, JSON_THROW_ON_ERROR) === json_encode($issuedA, JSON_THROW_ON_ERROR), 'idempotency replay returns the identical decision');
expect_issuance($licenseCount() === 1, 'replay creates no second license row');
expect_issuance($issuanceService->issuanceCount() === 1, 'replay creates no second issuance journal row');

// Delivery retry with a new idempotency key: same canonical key, zero keys created.
$retriedA = $issue($handleA, 'req-issue-alpha-retry-1', 'idem-issue-alpha-retry-1');
expect_issuance($retriedA['decision'] === 'license_issued', 'delivery retry resolves the same license');
expect_issuance($retriedA['existing'] === true, 'delivery retry is an existing issuance');
expect_issuance($retriedA['keys_created'] === 0, 'delivery retry creates zero keys');
expect_issuance($retriedA['edd_license_id'] === $issuedA['edd_license_id'], 'delivery retry returns the same canonical license');
expect_issuance($retriedA['delivery']['license_key'] === $keyA, 'delivery retry returns the identical canonical key');
expect_issuance($licenseCount() === 1, 'delivery retry never creates a second key');
expect_issuance($issuanceService->issuanceCount() === 1, 'delivery retry never journals a second issuance');

// ── Positive: two eligible items -> exactly two canonical keys, one per item ─

$regMulti = $createRegistration('issue.multi@example.invalid', $FACADE, $PRODUCT, 'multi');
$customerMulti = $customerOf($regMulti['registration_uuid']);
$insertOrder(3002, 'complete', $customerMulti, 'issue.multi@example.invalid', [
    ['item_id' => 3002, 'download' => $DOWNLOAD],
    ['item_id' => 3003, 'download' => $DOWNLOAD],
]);
$insertTransaction(3002, $GATEWAY, 'txn_pay_1002');
$multi = $bind(3002, $regMulti['registration_uuid'], $customerMulti, [
    ['item_id' => 3002, 'download' => $DOWNLOAD],
    ['item_id' => 3003, 'download' => $DOWNLOAD],
], 'txn_pay_1002', 'multi-1');
expect_issuance($multi['issuance_requests_settled'] === 2, 'two eligible items settle two issuance requests');
expect_issuance($requestStateCount('pending') === 2, 'two pending issuance requests journaled');
$handleMultiA = $multi['protected_items'][0]['issuance_request_handle'];
$handleMultiB = $multi['protected_items'][1]['issuance_request_handle'];
expect_issuance($handleMultiA !== $handleMultiB, 'each order item gets its own issuance request handle');

$issuedMultiA = $issue($handleMultiA, 'req-issue-multi-a-1', 'idem-issue-multi-a-1');
expect_issuance($issuedMultiA['keys_created'] === 1 && $issuedMultiA['order_item_id'] === 3002, 'first item issues exactly one key');
expect_issuance($issuedMultiA['edd_license_id'] !== $issuedA['edd_license_id'], 'first multi-item key is a distinct license');
$issuedMultiB = $issue($handleMultiB, 'req-issue-multi-b-1', 'idem-issue-multi-b-1');
expect_issuance($issuedMultiB['keys_created'] === 1 && $issuedMultiB['order_item_id'] === 3003, 'second item issues exactly one key');
expect_issuance($issuedMultiB['edd_license_id'] !== $issuedMultiA['edd_license_id'], 'second multi-item key is a distinct license');
expect_issuance($issuedMultiB['delivery']['license_key'] !== $issuedMultiA['delivery']['license_key'], 'the two keys are distinct canonical keys');
expect_issuance($licenseCount() === 3, 'two items produce exactly two additional license rows (one per item)');
expect_issuance($requestStateCount('issued') === 3 && $requestStateCount('pending') === 0, 'all settled requests end issued, none pending');
expect_issuance($issuanceService->issuanceCount() === 3, 'three issuance journal rows total');
// The registration advanced once (first item); the sibling item issued without a second
// transition and without disturbing the canonical license reference of the first item.
$regRowMulti = $registrations->findByUuid($regMulti['registration_uuid']);
expect_issuance($regRowMulti['state'] === 'entitlement_issued', 'multi-item registration stays at entitlement_issued');
expect_issuance((int) $regRowMulti['edd_license_id'] === $issuedMultiA['edd_license_id'], 'registration keeps the first item canonical license reference');
// Re-issuance of a sibling item never duplicates either key.
$retriedMultiB = $issue($handleMultiB, 'req-issue-multi-b-retry-1', 'idem-issue-multi-b-retry-1');
expect_issuance($retriedMultiB['existing'] === true && $retriedMultiB['keys_created'] === 0, 'sibling re-issuance returns the same key with zero creation');
expect_issuance($retriedMultiB['delivery']['license_key'] === $issuedMultiB['delivery']['license_key'], 'sibling re-issuance returns the identical key');
expect_issuance($licenseCount() === 3, 'sibling re-issuance creates no third key');

// ── Duplicate-key regression: synthetic legacy keys and parallel issuance ─

// A synthetic focusa_live_* key (legacy custom issuer) for the same customer/download
// blocks canonical issuance; the synthetic row is preserved for migration.
$regSynthetic = $createRegistration('issue.synthetic@example.invalid', $FACADE, $PRODUCT, 'synthetic');
$customerSynthetic = $customerOf($regSynthetic['registration_uuid']);
$insertLicense($customerSynthetic, $DOWNLOAD, 9001, 'focusa_live_1001_' . str_repeat('a', 16), 'inactive');
$insertOrder(3003, 'complete', $customerSynthetic, 'issue.synthetic@example.invalid', [
    ['item_id' => 3004, 'download' => $DOWNLOAD],
]);
$insertTransaction(3003, $GATEWAY, 'txn_pay_1003');
$boundSynthetic = $bind(3003, $regSynthetic['registration_uuid'], $customerSynthetic, [['item_id' => 3004, 'download' => $DOWNLOAD]], 'txn_pay_1003', 'synthetic-1');
expect_issuance($boundSynthetic['issuance_requests_settled'] === 1, 'inactive synthetic key does not block the binding settlement');
$handleSynthetic = $boundSynthetic['protected_items'][0]['issuance_request_handle'];
expect_issuance_throws(
    fn() => $issue($handleSynthetic, 'req-issue-synthetic-1', 'idem-issue-synthetic-1'),
    'EDD_LICENSE_UNUSABLE',
    'canonical issuance is blocked next to a synthetic focusa_live legacy key',
);
expect_issuance($licenseCount() === 4, 'synthetic-key denial creates zero new licenses');
expect_issuance($requestStateCount('pending') === 1 && $requestStateCount('issued') === 3, 'synthetic-key denial leaves the request pending for migration review');
expect_issuance($issuanceService->issuanceCount() === 3, 'synthetic-key denial journals no issuance');
$syntheticRow = $db->query("SELECT * FROM wp_edd_licenses WHERE license_key LIKE 'focusa_live_%'")->fetch(PDO::FETCH_ASSOC);
expect_issuance($syntheticRow !== false && $syntheticRow['status'] === 'inactive', 'legacy synthetic key is preserved, never deleted');

// An ACTIVE synthetic key blocks at the binding boundary (duplicate-key regression): a
// parallel custom entitlement can never coexist with a new issuance request.
$regActiveSynth = $createRegistration('issue.activesynth@example.invalid', $FACADE, $PRODUCT, 'activesynth');
$customerActiveSynth = $customerOf($regActiveSynth['registration_uuid']);
$insertLicense($customerActiveSynth, $DOWNLOAD, 9002, 'focusa_live_1001_' . str_repeat('b', 16), 'active');
$insertOrder(3004, 'complete', $customerActiveSynth, 'issue.activesynth@example.invalid', [
    ['item_id' => 3005, 'download' => $DOWNLOAD],
]);
$insertTransaction(3004, $GATEWAY, 'txn_pay_1004');
expect_issuance_throws(
    fn() => $bind(3004, $regActiveSynth['registration_uuid'], $customerActiveSynth, [['item_id' => 3005, 'download' => $DOWNLOAD]], 'txn_pay_1004', 'activesynth-1'),
    'EDD_LICENSE_UNUSABLE',
    'an active synthetic custom key blocks any new issuance request at the binding boundary',
);
expect_issuance($requestStateCount('pending') === 1, 'active synthetic key creates no issuance request');
expect_issuance($bindingCount() === 4, 'active synthetic key journals no settled binding');

// An active canonical license for the same customer/download from ANOTHER order blocks
// issuance (defense in depth: no parallel entitlement, no second key for the item).
$regParallel = $createRegistration('issue.parallel@example.invalid', $FACADE, $PRODUCT, 'parallel');
$customerParallel = $customerOf($regParallel['registration_uuid']);
$insertOrder(3005, 'complete', $customerParallel, 'issue.parallel@example.invalid', [
    ['item_id' => 3006, 'download' => $DOWNLOAD],
]);
$insertTransaction(3005, $GATEWAY, 'txn_pay_1005');
$boundParallel = $bind(3005, $regParallel['registration_uuid'], $customerParallel, [['item_id' => 3006, 'download' => $DOWNLOAD]], 'txn_pay_1005', 'parallel-1');
expect_issuance($boundParallel['issuance_requests_settled'] === 1, 'parallel-item binding settles before the out-of-band license appears');
$insertLicense($customerParallel, $DOWNLOAD, 9999, '1A2B3C4D-5E6F7890-ABCDEF12-3456WX90', 'active');
$handleParallel = $boundParallel['protected_items'][0]['issuance_request_handle'];
expect_issuance_throws(
    fn() => $issue($handleParallel, 'req-issue-parallel-1', 'idem-issue-parallel-1'),
    'EDD_LICENSE_UNUSABLE',
    'an active license from another order blocks canonical issuance for the item',
);
expect_issuance($licenseCount() === 6, 'parallel-license denial creates zero new licenses');
expect_issuance($requestStateCount('pending') === 2, 'parallel-license denial leaves the request pending');
expect_issuance($issuanceService->issuanceCount() === 3, 'parallel-license denial journals no issuance');

// The adapter never creates synthetic-prefixed keys: every key it issued is canonical.
$issuedKeys = $db->query("SELECT license_key FROM wp_edd_licenses WHERE license_key NOT LIKE 'focusa_live_%' AND license_key <> '1A2B3C4D-5E6F7890-ABCDEF12-3456WX90'")->fetchAll(PDO::FETCH_COLUMN);
expect_issuance(count($issuedKeys) === 3, 'exactly three canonical keys issued across all positive fixtures before the sibling-item fixture');
foreach ($issuedKeys as $issuedKey) {
    expect_issuance(preg_match($KEY_PATTERN, (string) $issuedKey) === 1, 'every adapter-issued key is canonical EDD SL format');
}

// ── Negative: canonical order truth at issuance time ───────────────────

// Refunded canonical order after settlement: fails closed, zero issuance.
$regRefunded = $createRegistration('issue.refunded@example.invalid', $FACADE, $PRODUCT, 'refunded');
$customerRefunded = $customerOf($regRefunded['registration_uuid']);
$insertOrder(3006, 'complete', $customerRefunded, 'issue.refunded@example.invalid', [
    ['item_id' => 3007, 'download' => $DOWNLOAD],
]);
$insertTransaction(3006, $GATEWAY, 'txn_pay_1006');
$boundRefunded = $bind(3006, $regRefunded['registration_uuid'], $customerRefunded, [['item_id' => 3007, 'download' => $DOWNLOAD]], 'txn_pay_1006', 'refunded-1');
$handleRefunded = $boundRefunded['protected_items'][0]['issuance_request_handle'];
$db->exec("UPDATE wp_edd_orders SET status = 'refunded' WHERE id = 3006");
expect_issuance_throws(
    fn() => $issue($handleRefunded, 'req-issue-refunded-1', 'idem-issue-refunded-1'),
    'REFUNDED',
    'a refunded canonical order never issues at issuance time',
);
expect_issuance($requestStateCount('pending') === 3, 'refunded-order denial leaves the request pending');
expect_issuance($licenseCount() === 6, 'refunded-order denial creates zero licenses');

// Revoked canonical order after settlement: fails closed, zero issuance.
$regRevoked = $createRegistration('issue.revoked@example.invalid', $FACADE, $PRODUCT, 'revoked');
$customerRevoked = $customerOf($regRevoked['registration_uuid']);
$insertOrder(3007, 'complete', $customerRevoked, 'issue.revoked@example.invalid', [
    ['item_id' => 3008, 'download' => $DOWNLOAD],
]);
$insertTransaction(3007, $GATEWAY, 'txn_pay_1007');
$boundRevoked = $bind(3007, $regRevoked['registration_uuid'], $customerRevoked, [['item_id' => 3008, 'download' => $DOWNLOAD]], 'txn_pay_1007', 'revoked-1');
$handleRevoked = $boundRevoked['protected_items'][0]['issuance_request_handle'];
$db->exec("UPDATE wp_edd_orders SET status = 'revoked' WHERE id = 3007");
expect_issuance_throws(
    fn() => $issue($handleRevoked, 'req-issue-revoked-1', 'idem-issue-revoked-1'),
    'REVOKED',
    'a revoked canonical order never issues at issuance time',
);
expect_issuance($licenseCount() === 6, 'revoked-order denial creates zero licenses');

// Canonical order row moved back to pending: fails closed with EDD_ORDER_PENDING.
$regPending = $createRegistration('issue.pending@example.invalid', $FACADE, $PRODUCT, 'pendingorder');
$customerPending = $customerOf($regPending['registration_uuid']);
$insertOrder(3008, 'complete', $customerPending, 'issue.pending@example.invalid', [
    ['item_id' => 3009, 'download' => $DOWNLOAD],
]);
$insertTransaction(3008, $GATEWAY, 'txn_pay_1008');
$boundPending = $bind(3008, $regPending['registration_uuid'], $customerPending, [['item_id' => 3009, 'download' => $DOWNLOAD]], 'txn_pay_1008', 'pendingorder-1');
$handlePending = $boundPending['protected_items'][0]['issuance_request_handle'];
$db->exec("UPDATE wp_edd_orders SET status = 'pending' WHERE id = 3008");
expect_issuance_throws(
    fn() => $issue($handlePending, 'req-issue-pending-1', 'idem-issue-pending-1'),
    'EDD_ORDER_PENDING',
    'a pending canonical order fails closed with EDD_ORDER_PENDING',
);
expect_issuance($licenseCount() === 6, 'pending-order denial creates zero licenses');

// Canonical order customer changed after settlement: EDD_ORDER_UNVERIFIED.
$regCustomer = $createRegistration('issue.customer@example.invalid', $FACADE, $PRODUCT, 'customer');
$customerCustomer = $customerOf($regCustomer['registration_uuid']);
$insertOrder(3009, 'complete', $customerCustomer, 'issue.customer@example.invalid', [
    ['item_id' => 3010, 'download' => $DOWNLOAD],
]);
$insertTransaction(3009, $GATEWAY, 'txn_pay_1009');
$boundCustomer = $bind(3009, $regCustomer['registration_uuid'], $customerCustomer, [['item_id' => 3010, 'download' => $DOWNLOAD]], 'txn_pay_1009', 'customer-1');
$handleCustomer = $boundCustomer['protected_items'][0]['issuance_request_handle'];
$db->exec('UPDATE wp_edd_orders SET customer_id = 424242 WHERE id = 3009');
expect_issuance_throws(
    fn() => $issue($handleCustomer, 'req-issue-customer-1', 'idem-issue-customer-1'),
    'EDD_ORDER_UNVERIFIED',
    'an order whose customer changed after settlement fails closed',
);
expect_issuance($licenseCount() === 6, 'customer-mismatch denial creates zero licenses');

// Canonical order email changed after settlement: ACCOUNT_EMAIL_MISMATCH.
$regEmail = $createRegistration('issue.email@example.invalid', $FACADE, $PRODUCT, 'email');
$customerEmail = $customerOf($regEmail['registration_uuid']);
$insertOrder(3010, 'complete', $customerEmail, 'issue.email@example.invalid', [
    ['item_id' => 3011, 'download' => $DOWNLOAD],
]);
$insertTransaction(3010, $GATEWAY, 'txn_pay_1010');
$boundEmail = $bind(3010, $regEmail['registration_uuid'], $customerEmail, [['item_id' => 3011, 'download' => $DOWNLOAD]], 'txn_pay_1010', 'email-1');
$handleEmail = $boundEmail['protected_items'][0]['issuance_request_handle'];
$db->exec("UPDATE wp_edd_orders SET email = 'other@example.invalid' WHERE id = 3010");
expect_issuance_throws(
    fn() => $issue($handleEmail, 'req-issue-email-1', 'idem-issue-email-1'),
    'ACCOUNT_EMAIL_MISMATCH',
    'an order whose canonical email changed after settlement fails closed',
);
expect_issuance($licenseCount() === 6, 'email-mismatch denial creates zero licenses');

// ── Negative: registration/account truth at issuance time ──────────────

// Registration denied after settlement: EMAIL_VERIFICATION_REQUIRED, zero issuance.
$regDenied = $createRegistration('issue.denied@example.invalid', $FACADE, $PRODUCT, 'denied');
$customerDenied = $customerOf($regDenied['registration_uuid']);
$insertOrder(3011, 'complete', $customerDenied, 'issue.denied@example.invalid', [
    ['item_id' => 3012, 'download' => $DOWNLOAD],
]);
$insertTransaction(3011, $GATEWAY, 'txn_pay_1011');
$boundDenied = $bind(3011, $regDenied['registration_uuid'], $customerDenied, [['item_id' => 3012, 'download' => $DOWNLOAD]], 'txn_pay_1011', 'denied-1');
$handleDenied = $boundDenied['protected_items'][0]['issuance_request_handle'];
$deniedRow = $registrations->findByUuid($regDenied['registration_uuid']);
$registrations->transition($regDenied['registration_uuid'], 'checkout_pending', 'denied', (int) $deniedRow['state_version'], 'req-denied-tx-1', 'idem-denied-tx-1', ['state_reason' => 'denied']);
expect_issuance_throws(
    fn() => $issue($handleDenied, 'req-issue-denied-1', 'idem-issue-denied-1'),
    'EMAIL_VERIFICATION_REQUIRED',
    'a denied registration cannot issue after settlement',
);
expect_issuance($licenseCount() === 6, 'denied-registration denial creates zero licenses');

// Registration expired after settlement: REGISTRATION_EXPIRED, zero issuance.
$regExpired = $createRegistration('issue.expired@example.invalid', $FACADE, $PRODUCT, 'expired');
$customerExpired = $customerOf($regExpired['registration_uuid']);
$insertOrder(3012, 'complete', $customerExpired, 'issue.expired@example.invalid', [
    ['item_id' => 3013, 'download' => $DOWNLOAD],
]);
$insertTransaction(3012, $GATEWAY, 'txn_pay_1012');
$boundExpired = $bind(3012, $regExpired['registration_uuid'], $customerExpired, [['item_id' => 3013, 'download' => $DOWNLOAD]], 'txn_pay_1012', 'expired-1');
$handleExpired = $boundExpired['protected_items'][0]['issuance_request_handle'];
$nowValue = '2026-08-10T00:00:00Z';
expect_issuance_throws(
    fn() => $issue($handleExpired, 'req-issue-expired-1', 'idem-issue-expired-1'),
    'REGISTRATION_EXPIRED',
    'an expired registration cannot issue after settlement',
);
expect_issuance($licenseCount() === 6, 'expired-registration denial creates zero licenses');
$nowValue = '2026-08-08T00:01:00Z';

// Registration that never entered checkout (promoted only): EDD_CHECKOUT_REQUIRED.
$regNoCheckout = $createRegistration('issue.nock@example.invalid', $FACADE, $PRODUCT, 'nock', true, true, false);
$customerNoCheckout = $customerOf($regNoCheckout['registration_uuid']);
$insertOrder(3013, 'complete', $customerNoCheckout, 'issue.nock@example.invalid', [
    ['item_id' => 3014, 'download' => $DOWNLOAD],
]);
$insertTransaction(3013, $GATEWAY, 'txn_pay_1013');
$boundNoCheckout = $bind(3013, $regNoCheckout['registration_uuid'], $customerNoCheckout, [['item_id' => 3014, 'download' => $DOWNLOAD]], 'txn_pay_1013', 'nock-1');
expect_issuance($boundNoCheckout['issuance_requests_settled'] === 1, 'promoted-only registration can still settle a binding');
$handleNoCheckout = $boundNoCheckout['protected_items'][0]['issuance_request_handle'];
expect_issuance_throws(
    fn() => $issue($handleNoCheckout, 'req-issue-nock-1', 'idem-issue-nock-1'),
    'EDD_CHECKOUT_REQUIRED',
    'issuance requires the registration to have entered checkout',
);
expect_issuance($licenseCount() === 6, 'no-checkout denial creates zero licenses');
$regRowNoCheckout = $registrations->findByUuid($regNoCheckout['registration_uuid']);
expect_issuance($regRowNoCheckout['state'] === 'account_promoted', 'no-checkout denial leaves the registration state untouched');

// A pending request for a registration that already advanced to entitlement_issued (a
// sibling order item of the same order) still issues its own canonical key and never
// disturbs the registration's existing fulfillment reference.
$regInconsistent = $createRegistration('issue.inconsistent@example.invalid', $FACADE, $PRODUCT, 'inconsistent');
$customerInconsistent = $customerOf($regInconsistent['registration_uuid']);
$insertOrder(3014, 'complete', $customerInconsistent, 'issue.inconsistent@example.invalid', [
    ['item_id' => 3015, 'download' => $DOWNLOAD],
]);
$insertTransaction(3014, $GATEWAY, 'txn_pay_1014');
$boundInconsistent = $bind(3014, $regInconsistent['registration_uuid'], $customerInconsistent, [['item_id' => 3015, 'download' => $DOWNLOAD]], 'txn_pay_1014', 'inconsistent-1');
$handleInconsistent = $boundInconsistent['protected_items'][0]['issuance_request_handle'];
$inconsistentRow = $registrations->findByUuid($regInconsistent['registration_uuid']);
$registrations->transition($regInconsistent['registration_uuid'], 'checkout_pending', 'entitlement_issued', (int) $inconsistentRow['state_version'], 'req-inconsistent-tx-1', 'idem-inconsistent-tx-1', ['state_reason' => 'out_of_band', 'edd_order_id' => 3014, 'edd_order_item_id' => 3015, 'edd_license_id' => 999999]);
$issuedInconsistent = $issue($handleInconsistent, 'req-issue-inconsistent-1', 'idem-issue-inconsistent-1');
expect_issuance($issuedInconsistent['decision'] === 'license_issued' && $issuedInconsistent['keys_created'] === 1, 'a pending request for an already-fulfilled registration still issues its canonical key');
expect_issuance($issuedInconsistent['edd_license_id'] !== 999999 && $issuedInconsistent['edd_license_id'] > 0, 'the sibling issuance creates a real canonical license, never reusing the stale reference');
expect_issuance($licenseCount() === 7, 'already-fulfilled sibling issuance creates exactly one new license');
$regRowInconsistent = $registrations->findByUuid($regInconsistent['registration_uuid']);
expect_issuance($regRowInconsistent['state'] === 'entitlement_issued', 'already-fulfilled registration is left at entitlement_issued');
expect_issuance((int) $regRowInconsistent['edd_license_id'] === 999999, 'the registration fulfillment reference is never overwritten by a sibling issuance');

// ── Negative: registry / offer authority ───────────────────────────────

// A checkout_pending registration with a settled binding for the registry-denial cases.
$regFreeze = $createRegistration('issue.freeze@example.invalid', $FACADE, $PRODUCT, 'freeze');
$customerFreeze = $customerOf($regFreeze['registration_uuid']);
$insertOrder(3015, 'complete', $customerFreeze, 'issue.freeze@example.invalid', [
    ['item_id' => 3016, 'download' => $DOWNLOAD],
]);
$insertTransaction(3015, $GATEWAY, 'txn_pay_1015');
$boundFreeze = $bind(3015, $regFreeze['registration_uuid'], $customerFreeze, [['item_id' => 3016, 'download' => $DOWNLOAD]], 'txn_pay_1015', 'freeze-1');
$handleFreeze = $boundFreeze['protected_items'][0]['issuance_request_handle'];

// Frozen registry (no operator-approved download mapping): PRODUCT_MAPPING_REQUIRED.
expect_issuance_throws(
    fn() => $issuanceFrozen->issue([
        'issuance_request_handle' => $handleFreeze,
        'request_id' => 'req-issue-frozen-1',
        'idempotency_key' => 'idem-issue-frozen-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'frozen registry (no operator-approved download mapping) denies issuance',
);
expect_issuance($licenseCount() === 7, 'frozen-registry denial creates zero licenses');

// Offer download/license-type mapping no longer matches the settled item:
// PRODUCT_MAPPING_REQUIRED.
expect_issuance_throws(
    fn() => $issuanceMismatched->issue([
        'issuance_request_handle' => $handleFreeze,
        'request_id' => 'req-issue-mismatch-1',
        'idempotency_key' => 'idem-issue-mismatch-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'an offer whose download/license-type mapping drifted fails closed',
);
expect_issuance($licenseCount() === 7, 'mismatched-offer denial creates zero licenses');

// Offer no longer checkout-enabled: EDD_CHECKOUT_REQUIRED.
expect_issuance_throws(
    fn() => $issuanceBlocked->issue([
        'issuance_request_handle' => $handleFreeze,
        'request_id' => 'req-issue-blocked-1',
        'idempotency_key' => 'idem-issue-blocked-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'an offer that is no longer checkout-enabled fails closed',
);
expect_issuance($licenseCount() === 7, 'blocked-offer denial creates zero licenses');

// ── Negative: input validation and idempotency ─────────────────────────

$negativeChecks++;
try {
    $issue('not-an-issuance-handle', 'req-issue-malformed-1', 'idem-issue-malformed-1');
    fwrite(STDERR, "FAIL: malformed issuance request handles are rejected\n");
    exit(1);
} catch (InvalidArgumentException) {
    // expected: bounded handle required
}
expect_issuance_throws(
    fn() => $issue('ir_' . str_repeat('0', 32), 'req-issue-unknown-1', 'idem-issue-unknown-1'),
    'EDD_LICENSE_UNUSABLE',
    'unknown issuance request handles fail closed',
);
expect_issuance_throws(
    fn() => $issuanceService->issue([
        'issuance_request_handle' => $handleA,
        'request_id' => 'req-issue-clientfields-1',
        'idempotency_key' => 'idem-issue-clientfields-1',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-controlled commerce fields are forbidden at issuance',
);
expect_issuance_throws(
    fn() => $issue($handleNoCheckout, 'req-issue-conflict-1', 'idem-issue-alpha-1'),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// ── Rollback preservation and redaction ────────────────────────────────

$preserved = $issuanceMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_license_issuance_test', 'record' => 'rollback']);
expect_issuance($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_issuance((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_license_issuance_schema_events')->fetchColumn() === 1, 'exactly one issuance preservation event journaled');

$issuedJson = json_encode([$issuedA, $retriedA, $issuedMultiA, $issuedMultiB, $retriedMultiB], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_issuance(strpos($issuedJson, '@') === false, 'no raw email in any issuance decision');
expect_issuance(preg_match($KEY_SCAN_PATTERN, $issuedJson) === 1, 'the canonical key appears in the decision only inside the delivery envelope');
$decisionWithoutDelivery = $issuedA;
unset($decisionWithoutDelivery['delivery']);
$withoutDeliveryJson = json_encode($decisionWithoutDelivery, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_issuance(preg_match($KEY_SCAN_PATTERN, $withoutDeliveryJson) !== 1, 'no full key outside the bounded delivery envelope');

$issuanceRows = $db->query('SELECT * FROM wp_wpuiai_edd_license_issuances')->fetchAll(PDO::FETCH_ASSOC);
$issuanceJson = json_encode($issuanceRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_issuance(strpos($issuanceJson, '@') === false, 'no raw email in the issuance journal');
expect_issuance(strpos($issuanceJson, 'txn_pay_') === false, 'no raw payment transaction id in the issuance journal');
expect_issuance(preg_match($KEY_SCAN_PATTERN, $issuanceJson) !== 1, 'no full license key in the issuance journal');
expect_issuance(strpos($issuanceJson, 'focusa_live_') === false, 'no synthetic key material in the issuance journal');
foreach ($issuanceRows as $issuanceRow) {
    expect_issuance(preg_match('/^(ki_)[0-9a-f]{32}$/D', (string) $issuanceRow['issuance_key']) === 1, 'issuance handles are opaque bounded tokens');
    expect_issuance(preg_match('/^[0-9a-f]{64}$/D', (string) $issuanceRow['license_key_digest']) === 1, 'license keys are keyed digests only in the journal');
    expect_issuance(preg_match($KEY_SCAN_PATTERN, (string) $issuanceRow['license_key_mask']) !== 1, 'journal masks are never full keys');
    expect_issuance(strpos((string) $issuanceRow['result_payload'], '"license_key"') === false, 'result payloads never contain a raw license_key field');
    expect_issuance(preg_match($KEY_SCAN_PATTERN, (string) $issuanceRow['result_payload']) !== 1, 'result payloads never contain a full key');
}
$requestRows = $db->query('SELECT * FROM wp_wpuiai_edd_issuance_requests')->fetchAll(PDO::FETCH_ASSOC);
$requestJson = json_encode($requestRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_issuance(strpos($requestJson, '@') === false, 'no raw email in the issuance-request journal');
expect_issuance(preg_match($KEY_SCAN_PATTERN, $requestJson) !== 1, 'no full license key in the issuance-request journal');
foreach ($requestRows as $requestRow) {
    expect_issuance(in_array($requestRow['state'], ['pending', 'issued'], true), 'issuance request states are bounded');
}
$licenseRows = $db->query('SELECT * FROM wp_edd_licenses')->fetchAll(PDO::FETCH_ASSOC);
$syntheticCount = 0;
$canonicalCount = 0;
foreach ($licenseRows as $licenseRow) {
    $key = (string) $licenseRow['license_key'];
    if (str_starts_with($key, 'focusa_live_')) {
        $syntheticCount++;
        expect_issuance(in_array($key, ['focusa_live_1001_' . str_repeat('a', 16), 'focusa_live_1001_' . str_repeat('b', 16)], true), 'synthetic keys are only the explicit legacy fixtures');
    } elseif (preg_match($KEY_PATTERN, $key) === 1) {
        $canonicalCount++;
    }
}
expect_issuance($syntheticCount === 2, 'both legacy synthetic fixtures are preserved');
expect_issuance($canonicalCount === 4, 'the only canonical-format keys are the four adapter-issued keys');
$registrationRow = $registrations->findByUuid($regA['registration_uuid']);
expect_issuance(strpos(json_encode($registrationRow, JSON_THROW_ON_ERROR), '@') === false, 'no raw email in the registration journal');
expect_issuance((int) $registrationRow['edd_license_id'] === $issuedA['edd_license_id'], 'registration fulfillment references the canonical license');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_license_issuance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'issuances_journaled' => $issuanceService->issuanceCount(),
    'canonical_licenses_created' => $canonicalCount,
    'synthetic_legacy_preserved' => $syntheticCount,
    'duplicate_key_fixtures' => ['idempotent_replay', 'delivery_retry', 'synthetic_focusa_live_inactive', 'synthetic_focusa_live_active_binding_block', 'parallel_active_license_other_order', 'sibling_item_same_order'],
    'fulfillment' => 'registration_checkout_pending_to_entitlement_issued',
    'key_storage' => 'keyed_digest_journal_bounded_delivery_envelope',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
