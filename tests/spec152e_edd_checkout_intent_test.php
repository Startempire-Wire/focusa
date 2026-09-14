<?php
// 152E.02.02 Create branded EDD checkout intent from verified registration.
// The activation.checkout surface creates exactly one server-owned EDD checkout intent
// for a mailbox-verified, promoted registration and returns a branded facade checkout
// URL. The intent binds registration, account, EDD customer, product, node request, and
// idempotency. Caller-controlled pricing/grants/products and caller-supplied redirect
// targets are impossible: the product/price come only from the server-owned registry and
// the branded URL comes only from the facade return-handle registry. Repeated canonical
// requests return one intent (idempotent replay and active-intent dedupe). No raw email,
// device key, license key, or secret is stored or returned; the cart/order fixture is
// synthetic with the exact server-owned price relationship. Issuance stays deferred.
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
require_once $root . '/docs/contracts/spec152e-edd-checkout-intent.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_intent(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_intent_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_checkout_intent_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_checkout_intent_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_checkout_intent_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_checkout_intent_test']);
$intentMigration = new FocusaSpec152eEddCheckoutIntentMigration($db, 'wp_');
$intentMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_checkout_intent_test']);

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

$cart = new FocusaSpec152eEddCartSessionAdapter($db, $intentMigration, $clock);
$returnHandles = new FocusaSpec152eFacadeReturnHandleRegistry($facadeRegistry);
$checkoutFrozen = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $frozenRegistry, $clock);
$checkoutFixture = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $fixtureRegistry, $clock);

// Variant with the uiai offer also checkout-enabled: used only to prove the facade
// product allowlist denies a facade that does not serve the bound product.
$activeUiaiRegistry = $fixtureRegistry;
foreach ($activeUiaiRegistry['protected_offers'] as &$offer) {
    if ($offer['public_code'] === 'uiai_operator_lifetime_v1') {
        $offer['mapping_status'] = 'active';
        $offer['sale_status'] = 'enabled';
        $offer['checkout_enabled'] = true;
    }
}
unset($offer);
$checkoutActiveUiai = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $activeUiaiRegistry, $clock);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = false) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.edd.checkout.intent.test',
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
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'checkout-intent-' . $tag . '-' . $seq],
    ]);
    return ['registration_uuid' => $uuid];
};

$transition = static function (string $uuid, string $from, string $to, string $tag) use ($registrations, &$seq): array {
    $seq++;
    $row = $registrations->findByUuid($uuid);
    return $registrations->transition($uuid, $from, $to, (int) $row['state_version'], 'req-tx-' . $tag . '-' . $seq, 'idem-tx-' . $tag . '-' . $seq, ['state_reason' => $to]);
};

$customerOf = static function (string $registrationUuid) use ($registrations): int {
    return (int) $registrations->findByUuid($registrationUuid)['edd_customer_id'];
};

$intentCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_checkout_intents')->fetchColumn();
};

$cartCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_checkout_cart_sessions')->fetchColumn();
};

$counts = static function () use ($db): array {
    return [
        'intents' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_checkout_intents')->fetchColumn(),
        'cart_sessions' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_checkout_cart_sessions')->fetchColumn(),
        'licenses' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn(),
        'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
    ];
};

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$PRODUCT = 'focusa_operator_lifetime_v1';
$UIAPRODUCT = 'uiai_operator_lifetime_v1';
$DOWNLOAD = 1001;
$PRICE = 'price_focusa_op_v1';
$SERVER_PRICE = '697.00';
$DEVICE_KEY = 'ed25519-test-device-public-key-0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b';

// ── Frozen registry invariants (generated contracts remain current) ────

expect_intent($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema');
expect_intent($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_intent($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_intent($offer['mapping_status'] === 'approved_policy_blocked_edd_mapping', 'frozen offer mapping blocked');
    expect_intent($offer['checkout_enabled'] === false, 'frozen offer checkout disabled');
    expect_intent($offer['edd_download_id'] === null, 'frozen offer unassigned download');
}

// ── Facade return-handle registry ──────────────────────────────────────

$returnSuccess = $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'success']);
expect_intent($returnSuccess['return_url'] === 'https://install.focusa.dev/activate/callback/success', 'success handle resolves to branded callback');
expect_intent($returnSuccess['schema'] === 'focusa.spec152e.facade_return_handle_registry.v1', 'return handle registry schema');
$returnCancel = $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'cancel']);
expect_intent($returnCancel['return_url'] === 'https://install.focusa.dev/activate/callback/cancel', 'cancel handle resolves to branded callback');
$returnRecovery = $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'recovery']);
expect_intent($returnRecovery['return_url'] === 'https://install.focusa.dev/activate/callback/recovery', 'recovery handle resolves to branded callback');
$branded = $returnHandles->brandedCheckoutUrl($FACADE, $ORIGIN, 'it_00000000000000000000000000000000');
expect_intent(str_starts_with($branded, 'https://install.focusa.dev/activate/checkout?intent='), 'branded checkout URL uses facade checkout path plus opaque intent token');
$returnHandles->assertFacadeSupports($FACADE, $PRODUCT);
expect_intent(true, 'registered facade supports its server-owned product allowlist');

expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'unknown']),
    'FACADE_REDIRECT_DENIED',
    'unknown return handle is denied',
);
expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'https://evil.example/hook']),
    'FACADE_REDIRECT_DENIED',
    'absolute URL as return handle is denied',
);
expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => '/activate/callback/success']),
    'FACADE_REDIRECT_DENIED',
    'relative path as return handle is denied',
);
expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'success', 'callback_url' => 'https://evil.example/hook']),
    'FACADE_REDIRECT_DENIED',
    'caller-supplied callback URL is denied',
);
expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => 'https://evil.example', 'return_handle' => 'success']),
    'FACADE_ORIGIN_DENIED',
    'wrong facade origin is denied',
);
expect_intent_throws(
    fn() => $returnHandles->resolve(['facade_id' => 'focusa_spoof_v1', 'origin' => $ORIGIN, 'return_handle' => 'success']),
    'FACADE_ORIGIN_DENIED',
    'unregistered facade is denied',
);

// ── Negative: checkout intent creation ─────────────────────────────────

$seq++;
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => '00000000-0000-4000-8000-000000000000',
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-missing-1',
        'idempotency_key' => 'idem-ci-missing-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unknown registration cannot create a checkout intent',
);

$regUnverified = $createRegistration('intent.unverified@example.invalid', $FACADE, $PRODUCT, 'unver', false, false);
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regUnverified['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-unver-1',
        'idempotency_key' => 'idem-ci-unver-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration cannot create a checkout intent',
);

$regEmailOnly = $createRegistration('intent.emailonly@example.invalid', $FACADE, $PRODUCT, 'emailonly', true, false);
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regEmailOnly['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-nopromote-1',
        'idempotency_key' => 'idem-ci-nopromote-1',
    ]),
    'EDD_CUSTOMER_RESOLUTION_FAILED',
    'verified but unpromoted registration cannot create a checkout intent',
);

$regFocusa = $createRegistration('intent.focusa@example.invalid', $FACADE, $PRODUCT, 'focusa', true, true);
$focusaCustomer = $customerOf($regFocusa['registration_uuid']);

// Wrong facade binding: the registration is bound to focusa_install_v1 only.
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => 'focusa_arena_v1',
        'origin' => 'https://arena.focusa.dev',
        'return_handle' => 'success',
        'request_id' => 'req-ci-wrongfac-1',
        'idempotency_key' => 'idem-ci-wrongfac-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'checkout intent cannot be created for a facade the registration is not bound to',
);
// Wrong origin: exact-origin matching only.
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => 'https://evil.example',
        'return_handle' => 'success',
        'request_id' => 'req-ci-wrongorigin-1',
        'idempotency_key' => 'idem-ci-wrongorigin-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'wrong origin is denied',
);
// Unknown product code: the product is resolved from the registration, never the caller.
$regUnknown = $createRegistration('intent.unknown@example.invalid', $FACADE, 'focusa_nonexistent_v1', 'unknown', true, true);
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regUnknown['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-unknownprod-1',
        'idempotency_key' => 'idem-ci-unknownprod-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown product code cannot create a checkout intent',
);
// Frozen registry: no protected offer is checkout-enabled yet.
expect_intent_throws(
    fn() => $checkoutFrozen->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-frozen-1',
        'idempotency_key' => 'idem-ci-frozen-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'frozen registry (no checkout-enabled offer) denies intent creation',
);
// Blocked fixture mapping: approved policy but checkout_enabled false.
$regUiai = $createRegistration('intent.uiai@example.invalid', $FACADE, $UIAPRODUCT, 'uiai', true, true);
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regUiai['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-blocked-1',
        'idempotency_key' => 'idem-ci-blocked-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'approved-but-blocked mapping denies intent creation',
);
// Facade product mismatch: focusa_marketing_v1 does not serve uiai_operator_lifetime_v1.
$regMarketingUiai = $createRegistration('intent.marketing.uiai@example.invalid', 'focusa_marketing_v1', $UIAPRODUCT, 'mktuiai', true, true);
expect_intent_throws(
    fn() => $checkoutActiveUiai->createIntent([
        'registration_uuid' => $regMarketingUiai['registration_uuid'],
        'facade_id' => 'focusa_marketing_v1',
        'origin' => 'https://focusa.dev',
        'return_handle' => 'success',
        'request_id' => 'req-ci-facprod-1',
        'idempotency_key' => 'idem-ci-facprod-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'facade product allowlist mismatch denies intent creation',
);
// Expired registration.
$regExpired = $createRegistration('intent.expired@example.invalid', $FACADE, $PRODUCT, 'expired', true, true);
$nowValue = '2026-08-10T00:00:00Z';
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regExpired['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-expired-1',
        'idempotency_key' => 'idem-ci-expired-1',
    ]),
    'REGISTRATION_EXPIRED',
    'expired registration cannot create a checkout intent',
);
$nowValue = '2026-08-08T00:01:00Z';
// Denied (terminal) registration.
$regDenied = $createRegistration('intent.denied@example.invalid', $FACADE, $PRODUCT, 'denied', true, true);
$transition($regDenied['registration_uuid'], 'account_promoted', 'denied', 'denied');
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regDenied['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-denied-1',
        'idempotency_key' => 'idem-ci-denied-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'denied registration cannot create a checkout intent',
);

// Caller-controlled commerce fields are impossible.
$forbiddenCommerce = [
    'price' => '1.00',
    'amount' => '1.00',
    'total' => '1.00',
    'grants' => ['focusa_operator_lifetime_v1'],
    'features' => ['focusa.core.mission'],
    'limits' => ['nodes' => 99],
    'product_code' => $PRODUCT,
    'edd_download_id' => $DOWNLOAD,
    'edd_price_id' => $PRICE,
    'license_type' => 'focusa_operator_lifetime_v1',
    'node_limit' => 99,
    'sale_status' => 'enabled',
    'commercial_rights' => ['resale'],
];
foreach ($forbiddenCommerce as $field => $value) {
    $seq++;
    expect_intent_throws(
        fn() => $checkoutFixture->createIntent([
            'registration_uuid' => $regFocusa['registration_uuid'],
            'facade_id' => $FACADE,
            'origin' => $ORIGIN,
            'return_handle' => 'success',
            'request_id' => 'req-ci-' . $field . '-1',
            'idempotency_key' => 'idem-ci-' . $field . '-1',
            $field => $value,
        ]),
        'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
        "caller-controlled {$field} is rejected",
    );
}
// Caller-supplied redirect targets are impossible.
$forbiddenRedirects = ['callback_url', 'redirect_url', 'success_url', 'cancel_url', 'return_url'];
foreach ($forbiddenRedirects as $field) {
    $seq++;
    expect_intent_throws(
        fn() => $checkoutFixture->createIntent([
            'registration_uuid' => $regFocusa['registration_uuid'],
            'facade_id' => $FACADE,
            'origin' => $ORIGIN,
            'return_handle' => 'success',
            'request_id' => 'req-ci-' . $field . '-1',
            'idempotency_key' => 'idem-ci-' . $field . '-1',
            $field => 'https://evil.example/hook',
        ]),
        'FACADE_REDIRECT_DENIED',
        "caller-supplied {$field} is rejected",
    );
}

// Input bounds fail closed.
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'short',
        'idempotency_key' => 'idem-ci-badreq-1',
    ]),
    'bounded request ID required',
    'undersized request ID is rejected',
);
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-ci-badidem-1',
        'idempotency_key' => 'short',
    ]),
    'bounded idempotency key required',
    'undersized idempotency key is rejected',
);

// ── Positive: one branded checkout intent ──────────────────────────────

$intentOne = $checkoutFixture->createIntent([
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-ci-one-1',
    'idempotency_key' => 'idem-ci-one-1',
]);
$intentPayload = $intentOne['intent'];
expect_intent($intentOne['schema'] === 'focusa.spec152e.checkout_intent_result.v1', 'checkout intent result schema');
expect_intent(str_starts_with($intentPayload['intent_id'], 'it_'), 'intent id is opaque and prefixed');
expect_intent(strlen($intentPayload['intent_id']) <= 64, 'intent id is bounded');
expect_intent($intentPayload['registration_id'] === $regFocusa['registration_uuid'], 'intent binds the registration');
expect_intent($intentPayload['account_id'] !== null, 'intent binds the promoted account');
expect_intent($intentPayload['customer_id'] === $focusaCustomer, 'intent binds the promoted EDD customer');
expect_intent($intentPayload['facade_id'] === $FACADE, 'intent binds the facade');
expect_intent($intentPayload['product_code'] === $PRODUCT, 'intent binds the registry product code');
expect_intent($intentPayload['state'] === 'checkout_required', 'intent state is checkout_required');
expect_intent($intentPayload['next_action'] === 'open_checkout', 'intent next action is open_checkout');
expect_intent(str_starts_with($intentPayload['branded_checkout_url'], 'https://install.focusa.dev/activate/checkout?intent='), 'branded facade checkout URL returned');
expect_intent($intentPayload['return_handle'] === 'success', 'intent return handle is the allowlisted handle');
expect_intent($intentPayload['return_url'] === 'https://install.focusa.dev/activate/callback/success', 'intent return URL is the branded callback');
expect_intent(str_starts_with($intentPayload['cart_reference'], 'cs_'), 'intent binds an opaque cart reference');
expect_intent(str_starts_with($intentPayload['session_key'], 'sk_'), 'intent binds an opaque session key');
expect_intent($intentPayload['price']['currency'] === 'USD' && $intentPayload['price']['amount_usd'] === $SERVER_PRICE, 'intent price is the server-owned registry price');
expect_intent($intentOne['replayed'] === false && $intentOne['existing'] === false, 'first canonical request creates a fresh intent');
expect_intent($registrations->findByUuid($regFocusa['registration_uuid'])['state'] === 'checkout_pending', 'registration advances to checkout_pending');
expect_intent($intentCount() === 1 && $cartCount() === 1, 'exactly one intent and one cart session created');

// Idempotent replay: the same canonical request returns the same intent.
$intentReplay = $checkoutFixture->createIntent([
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-ci-one-1',
    'idempotency_key' => 'idem-ci-one-1',
]);
expect_intent($intentReplay['intent']['intent_id'] === $intentPayload['intent_id'], 'idempotent replay returns the same intent');
expect_intent($intentReplay['replayed'] === true, 'replay is marked replayed');
expect_intent($intentCount() === 1 && $cartCount() === 1, 'replay creates no new intent or cart');

// Repeated canonical request (new idempotency key) still returns one intent.
$intentRepeat = $checkoutFixture->createIntent([
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-ci-repeat-1',
    'idempotency_key' => 'idem-ci-repeat-1',
]);
expect_intent($intentRepeat['intent']['intent_id'] === $intentPayload['intent_id'], 'repeated canonical request returns the existing intent');
expect_intent($intentRepeat['existing'] === true && $intentRepeat['replayed'] === false, 'repeat is marked existing');
expect_intent($intentCount() === 1 && $cartCount() === 1, 'repeat creates no second intent');

// Idempotency conflict: same key, different request.
expect_intent_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'cancel',
        'request_id' => 'req-ci-conflict-1',
        'idempotency_key' => 'idem-ci-one-1',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'idempotency-key reuse with a different request is rejected',
);

// Node request binding: node UUID and device key are bound; the device key is stored only as a digest.
$regNode = $createRegistration('intent.node@example.invalid', $FACADE, $PRODUCT, 'node', true, true);
$nodeUuid = 'a1111111-2222-4333-8444-555555555555';
$intentNode = $checkoutFixture->createIntent([
    'registration_uuid' => $regNode['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-ci-node-1',
    'idempotency_key' => 'idem-ci-node-1',
    'node_uuid' => $nodeUuid,
    'device_public_key' => $DEVICE_KEY,
]);
expect_intent($intentNode['intent']['node_id'] === $nodeUuid, 'intent binds the node request UUID');
expect_intent($intentNode['intent']['customer_id'] === $customerOf($regNode['registration_uuid']), 'node-bound intent binds its promoted customer');
$nodeIntentRow = $checkoutFixture->findByIntentId($intentNode['intent']['intent_id']);
expect_intent($nodeIntentRow['device_public_key_hash'] === hash('sha256', $DEVICE_KEY), 'device public key is stored only as a digest');
expect_intent(strpos(json_encode($nodeIntentRow, JSON_THROW_ON_ERROR), $DEVICE_KEY) === false, 'raw device public key never stored');

// offer_selected registrations also create exactly one intent and advance to checkout_pending.
$regOffer = $createRegistration('intent.offer@example.invalid', $FACADE, $PRODUCT, 'offer', true, true);
$transition($regOffer['registration_uuid'], 'account_promoted', 'offer_selected', 'offer');
$intentOffer = $checkoutFixture->createIntent([
    'registration_uuid' => $regOffer['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-ci-offer-1',
    'idempotency_key' => 'idem-ci-offer-1',
]);
expect_intent($intentOffer['intent']['product_code'] === $PRODUCT, 'offer-selected registration creates an intent for the bound product');
expect_intent($registrations->findByUuid($regOffer['registration_uuid'])['state'] === 'checkout_pending', 'offer-selected registration advances to checkout_pending');
expect_intent($intentOffer['intent']['intent_id'] !== $intentPayload['intent_id'], 'each promoted registration/product pair gets its own intent');

// Another allowlisted return handle (cancel) on a fresh registration.
$regCancel = $createRegistration('intent.cancel@example.invalid', $FACADE, $PRODUCT, 'cancel', true, true);
$intentCancel = $checkoutFixture->createIntent([
    'registration_uuid' => $regCancel['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'cancel',
    'request_id' => 'req-ci-cancel-1',
    'idempotency_key' => 'idem-ci-cancel-1',
]);
expect_intent($intentCancel['intent']['return_handle'] === 'cancel', 'cancel return handle is honored');
expect_intent($intentCancel['intent']['return_url'] === 'https://install.focusa.dev/activate/callback/cancel', 'cancel return URL is the branded callback');

// Synthetic cart/order fixture: exact server-owned price relationship, no order row, no email.
$fixture = $cart->projectOrderFixture($intentPayload['cart_reference']);
expect_intent($fixture['schema'] === 'focusa.spec152e.edd_order_fixture.v1', 'order fixture schema');
expect_intent($fixture['fixture'] === 'synthetic', 'order fixture is synthetic');
expect_intent($fixture['registration_uuid'] === $regFocusa['registration_uuid'], 'order fixture binds the registration');
expect_intent($fixture['edd_customer_id'] === $focusaCustomer, 'order fixture binds the promoted customer');
expect_intent($fixture['order']['order_id'] === null, 'order fixture carries no real EDD order id');
expect_intent($fixture['order']['status'] === 'checkout_required' && $fixture['order']['email'] === null, 'order fixture is checkout_required with no email');
expect_intent($fixture['items'][0]['download_id'] === $DOWNLOAD, 'order fixture item uses the registry download');
expect_intent($fixture['items'][0]['price_id'] === $PRICE, 'order fixture item uses the registry price id');
expect_intent($fixture['items'][0]['quantity'] === 1, 'order fixture item quantity is one unit');
expect_intent($fixture['items'][0]['unit_amount_usd'] === $SERVER_PRICE && $fixture['items'][0]['total_amount_usd'] === $SERVER_PRICE, 'order fixture uses the exact server-owned price');
expect_intent($fixture['total_amount_usd'] === $SERVER_PRICE, 'order fixture total is the server-owned price');
expect_intent($fixture['entitlement_allowed'] === true, 'order fixture is entitlement-bound but issuance stays deferred');
expect_intent($fixture['order']['email'] === null, 'order fixture never carries a raw email');

// Rollback preservation: intent/cart journals are never deleted.
$preserved = $intentMigration->preserveForRollback('2026-08-08T00:02:00Z', ['source' => 'edd_checkout_intent_test', 'record' => 'rollback']);
expect_intent($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_intent((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_checkout_intent_schema_events')->fetchColumn() === 1, 'exactly one preservation event journaled');

// Redaction: no raw email, raw device key, or secret anywhere in results or journals.
$resultJson = json_encode([
    $intentOne, $intentReplay, $intentRepeat, $intentNode, $intentOffer, $intentCancel, $fixture,
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_intent(strpos($resultJson, '@') === false, 'no raw email in any checkout intent result');
expect_intent(strpos($resultJson, $DEVICE_KEY) === false, 'no raw device public key in any result');
expect_intent(strpos($resultJson, 'fl_') === false, 'no license key in any result');
$intentRows = $db->query('SELECT * FROM wp_wpuiai_edd_checkout_intents')->fetchAll(PDO::FETCH_ASSOC);
$intentJson = json_encode($intentRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_intent(strpos($intentJson, '@') === false, 'no raw email in intent journal');
foreach ($intentRows as $intentRow) {
    expect_intent(preg_match('/^(it_)[0-9a-f]{32}$/D', (string) $intentRow['intent_id']) === 1, 'intent ids are opaque bounded tokens');
    expect_intent(preg_match('/^[0-9a-f]{64}$/D', (string) $intentRow['device_public_key_hash']) === 1
        || $intentRow['device_public_key_hash'] === null, 'device keys are stored only as digests');
}
$cartRows = $db->query('SELECT * FROM wp_wpuiai_edd_checkout_cart_sessions')->fetchAll(PDO::FETCH_ASSOC);
foreach ($cartRows as $cartRow) {
    expect_intent(preg_match('/^(cs_)[0-9a-f]{32}$/D', (string) $cartRow['cart_reference']) === 1, 'cart references are opaque bounded tokens');
    expect_intent(preg_match('/^(sk_)[0-9a-f]{32}$/D', (string) $cartRow['session_key']) === 1, 'session keys are opaque bounded tokens');
}

// ── Summary ───────────────────────────────────────────────────────────

$finalCounts = $counts();
fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_checkout_intent_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'intents_created' => $finalCounts['intents'],
    'cart_sessions' => $finalCounts['cart_sessions'],
    'licenses_created' => $finalCounts['licenses'],
    'customers' => $finalCounts['customers'],
    'protected_offer_fixture' => 'operator_approved_test_mapping_download_1001',
    'branded_checkout_url' => 'facade_origin_plus_allowlisted_checkout_path_plus_opaque_intent_token',
    'entitlement_issuance' => 'deferred',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
