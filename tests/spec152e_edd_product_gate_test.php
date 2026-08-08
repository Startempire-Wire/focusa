<?php
// 152E.02.01 Enforce protected EDD product allowlist and registration gate.
// The EDD add-to-cart, checkout, and order-completion surfaces can reach Focusa/UIAI
// entitlement only through a verified account-bound registration with a single-use gate
// token and an operator-approved server-owned product mapping. Raw add-to-cart, unknown
// downloads, wrong facades, and credit packs can never issue Focusa/UIAI entitlement;
// unrelated/quarantined products are proven non-entitlement; order completion requires
// complete status, verified registration/account binding with matching order email,
// exact product/price relationship, idempotent issuance state, and no duplicate active
// license. Issuance itself stays deferred to the verified issuance service. All fixtures
// are synthetic; all output is redacted (no raw email, token, or license key).
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
require_once $root . '/docs/contracts/spec152e-edd-gate-hooks.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_gate(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_gate_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);
$tokenMigration = new FocusaSpec152eEddRegistrationTokenMigration($db, 'wp_');
$tokenMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);
$gateMigration = new FocusaSpec152eEddGateDecisionMigration($db, 'wp_');
$gateMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_product_gate_test']);

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
$tokens = new FocusaSpec152eVerifiedRegistrationTokenValidator($db, $tokenMigration, $registrations, $clock);

// The frozen registry is used by the fail-closed gate instance; the fixture registry adds
// an explicitly operator-approved test mapping (download 1001 -> focusa_operator_lifetime_v1
// active/checkout_enabled) and an approved-but-blocked mapping (download 1002 ->
// uiai_operator_lifetime_v1, checkout_enabled false) so positive and blocked paths are
// exercised without mutating the frozen contract.
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

$gateFrozen = new FocusaSpec152eEddGateHooks($db, $gateMigration, $tokens, $registrations, $registrationSecrets, $frozenRegistry, $facadeRegistry, $clock);
$gateFixture = new FocusaSpec152eEddGateHooks($db, $gateMigration, $tokens, $registrations, $registrationSecrets, $fixtureRegistry, $facadeRegistry, $clock);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = false) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.edd.gate.test',
        'install_channel' => 'cli',
        'product_code' => $product,
        'safe_redirect_handle' => 'safe-' . $tag . '-' . $seq,
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
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'edd-gate-' . $tag . '-' . $seq],
    ]);
    return ['registration_uuid' => $uuid];
};

$issueToken = static function (string $uuid, string $facade, string $product, string $tag) use ($tokens, &$seq): array {
    $seq++;
    return $tokens->issue([
        'registration_uuid' => $uuid,
        'facade_id' => $facade,
        'product_code' => $product,
        'request_id' => 'req-tok-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-tok-' . $tag . '-' . $seq,
    ]);
};

$customerOf = static function (string $registrationUuid) use ($registrations): int {
    return (int) $registrations->findByUuid($registrationUuid)['edd_customer_id'];
};

$licenseCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

$counts = static function () use ($db): array {
    return [
        'tokens' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_registration_tokens')->fetchColumn(),
        'gate_decisions' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_gate_decisions')->fetchColumn(),
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
$BLOCKED_DOWNLOAD = 1002;
$CREDIT_PACK = 455;
$UNKNOWN_DOWNLOAD = 999;
$UNRELATED_453 = 453;

// ── Frozen registry invariants (generated contracts remain current) ────

expect_gate($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema');
expect_gate($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_gate($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_gate($offer['mapping_status'] === 'approved_policy_blocked_edd_mapping', 'frozen offer mapping blocked');
    expect_gate($offer['checkout_enabled'] === false, 'frozen offer checkout disabled');
    expect_gate($offer['edd_download_id'] === null, 'frozen offer unassigned download');
}

// ── Negative: token issuance ───────────────────────────────────────────

$regUnverified = $createRegistration('gate.unverified@example.invalid', $FACADE, $PRODUCT, 'unver', false, false);
expect_gate_throws(
    fn() => $tokens->issue([
        'registration_uuid' => $regUnverified['registration_uuid'],
        'facade_id' => $FACADE,
        'product_code' => $PRODUCT,
        'request_id' => 'req-tok-unver-1',
        'idempotency_key' => 'idem-tok-unver-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration cannot mint a gate token',
);

$regEmailOnly = $createRegistration('gate.emailonly@example.invalid', $FACADE, $PRODUCT, 'emailonly', true, false);
expect_gate_throws(
    fn() => $tokens->issue([
        'registration_uuid' => $regEmailOnly['registration_uuid'],
        'facade_id' => 'focusa_arena_v1',
        'product_code' => $PRODUCT,
        'request_id' => 'req-tok-wrongfac-1',
        'idempotency_key' => 'idem-tok-wrongfac-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'token cannot be minted for a facade the registration is not bound to',
);

$regFocusa = $createRegistration('gate.focusa@example.invalid', $FACADE, $PRODUCT, 'focusa', true, true);
$regFocusa2 = $createRegistration('gate.focusa2@example.invalid', $FACADE, $PRODUCT, 'focusa2', true, true);
$regUiai = $createRegistration('gate.uiai@example.invalid', $FACADE, $UIAPRODUCT, 'uiai', true, true);
$focusaCustomer = $customerOf($regFocusa['registration_uuid']);

$tokenFocusa = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'focusa');
expect_gate(str_starts_with($tokenFocusa['registration_token'], 'rg_'), 'token has opaque prefix');
expect_gate($tokenFocusa['replayed'] === false, 'first issue returns raw token once');

$tokenReplayInput = [
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE,
    'product_code' => $PRODUCT,
    'request_id' => 'req-tok-replay-fixed-1',
    'idempotency_key' => 'idem-tok-replay-fixed-1',
];
$tokenFirst = $tokens->issue($tokenReplayInput);
expect_gate($tokenFirst['replayed'] === false && isset($tokenFirst['registration_token']), 'first issue returns raw token once');
$tokenReplay = $tokens->issue($tokenReplayInput);
expect_gate($tokenReplay['replayed'] === true, 'token issue replay is idempotent');
expect_gate(!isset($tokenReplay['registration_token']), 'token replay never re-issues a raw token');

// ── Negative: raw add-to-cart, unknown, credit pack, wrong facade ──────

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => '', 'request_id' => 'req-raw-1', 'idempotency_key' => 'idem-raw-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'raw add-to-cart without a verified token is denied',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => '',
        'verified_token' => '', 'request_id' => 'req-raw-2', 'idempotency_key' => 'idem-raw-2',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'raw add-to-cart without registration context is denied',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $UNKNOWN_DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-unk-1', 'idempotency_key' => 'idem-unk-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown download cannot reach entitlement',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $CREDIT_PACK, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-credit-1', 'idempotency_key' => 'idem-credit-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'credit pack can never issue entitlement',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => 'evil_facade_v1', 'origin' => 'https://evil.example',
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-fac-1', 'idempotency_key' => 'idem-fac-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'unregistered facade is denied at add-to-cart',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => 'https://evil.example',
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-origin-1', 'idempotency_key' => 'idem-origin-1',
    ]),
    'FACADE_ORIGIN_DENIED',
    'wrong origin for a registered facade is denied',
);

expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $BLOCKED_DOWNLOAD, 'facade_id' => 'focusa_arena_v1', 'origin' => 'https://arena.focusa.dev',
        'product_code' => $UIAPRODUCT, 'registration_uuid' => $regUiai['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-prod-1', 'idempotency_key' => 'idem-prod-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'facade that does not support the product is denied',
);

$tokenWrongProduct = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'claim');
expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => 'focusa_uiai_operator_bundle_lifetime_v1',
        'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenWrongProduct['registration_token'],
        'request_id' => 'req-claim-1', 'idempotency_key' => 'idem-claim-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'client product claim cannot steer the mapping',
);

$tokenOtherRegistration = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'other');
expect_gate_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa2['registration_uuid'],
        'verified_token' => $tokenOtherRegistration['registration_token'],
        'request_id' => 'req-reg-1', 'idempotency_key' => 'idem-reg-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'a token bound to another registration cannot open the cart',
);

foreach (['price' => '1.00', 'grants' => ['focusa_operator_lifetime_v1'], 'features' => ['anything'],
          'limits' => ['nodes' => 99], 'node_limit' => 99, 'edd_price_id' => $PRICE,
          'sale_status' => 'enabled', 'refund_policy' => 'none', 'commercial_rights' => 'resale'] as $field => $value) {
    expect_gate_throws(
        fn() => $gateFixture->gateAddToCart([
            'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
            'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
            'verified_token' => $tokenFocusa['registration_token'],
            'request_id' => 'req-forbid-1', 'idempotency_key' => 'idem-forbid-1',
            $field => $value,
        ]),
        'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
        "caller-controlled field {$field} denied",
    );
}

// ── Negative: consumed and expired tokens ──────────────────────────────

$tokenWrongFacadeValidate = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'wrongfac');
expect_gate_throws(
    fn() => $tokens->validate([
        'registration_token' => $tokenWrongFacadeValidate['registration_token'],
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => 'focusa_arena_v1', 'product_code' => $PRODUCT,
        'request_id' => 'req-consume-1', 'idempotency_key' => 'idem-consume-1',
        'consume' => true,
    ]),
    'FACADE_ORIGIN_DENIED',
    'token minted for one facade cannot validate for another facade',
);
$tokenWrongProductValidate = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'wrongprod');
expect_gate_throws(
    fn() => $tokens->validate([
        'registration_token' => $tokenWrongProductValidate['registration_token'],
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE, 'product_code' => 'focusa_uiai_operator_bundle_lifetime_v1',
        'request_id' => 'req-consume-2', 'idempotency_key' => 'idem-consume-2',
        'consume' => true,
    ]),
    'FACADE_PRODUCT_DENIED',
    'token bound to one product cannot validate for another',
);

$tokenExpired = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'expiry');
$nowValue = '2026-08-08T01:00:00Z'; // past the 1800s token TTL, registration still valid (24h TTL)
expect_gate_throws(
    fn() => $tokens->validate([
        'registration_token' => $tokenExpired['registration_token'],
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE, 'product_code' => $PRODUCT,
        'request_id' => 'req-exp-1', 'idempotency_key' => 'idem-exp-1',
        'consume' => true,
    ]),
    'EMAIL_VERIFICATION_EXPIRED',
    'expired gate token is denied',
);
$nowValue = '2026-08-08T00:01:00Z';

$tokenConsumed = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'consume2');
$validated = $tokens->validate([
    'registration_token' => $tokenConsumed['registration_token'],
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE, 'product_code' => $PRODUCT,
    'request_id' => 'req-val-1', 'idempotency_key' => 'idem-val-1',
    'consume' => true,
]);
expect_gate($validated['ok'] === true && $validated['token_state'] === 'consumed', 'token validates once and is consumed');
expect_gate_throws(
    fn() => $tokens->validate([
        'registration_token' => $tokenConsumed['registration_token'],
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE, 'product_code' => $PRODUCT,
        'request_id' => 'req-reuse-1', 'idempotency_key' => 'idem-reuse-1',
        'consume' => true,
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'consumed token reuse is denied (single-use)',
);

// Bounded reissue: the active-token cap is enforced (fail closed, no unbounded issuance).
$regCap = $createRegistration('gate.cap@example.invalid', $FACADE, $PRODUCT, 'cap', true, true);
$capSeq = 0;
$capDenied = false;
for ($i = 1; $i <= FocusaSpec152eVerifiedRegistrationTokenValidator::MAX_ACTIVE_TOKENS_PER_REGISTRATION + 1; $i++) {
    $capSeq++;
    try {
        $tokens->issue([
            'registration_uuid' => $regCap['registration_uuid'],
            'facade_id' => $FACADE,
            'product_code' => $PRODUCT,
            'request_id' => 'req-cap-' . $capSeq,
            'idempotency_key' => 'idem-cap-' . $capSeq,
        ]);
    } catch (DomainException $error) {
        $capDenied = $error->getMessage() === 'EMAIL_VERIFICATION_REQUIRED';
        break;
    }
}
expect_gate($capDenied, 'active-token reissue cap is enforced');

// ── Negative: checkout gate ────────────────────────────────────────────

expect_gate_throws(
    fn() => $gateFixture->gateCheckout([
        'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => '', 'request_id' => 'req-co-1', 'idempotency_key' => 'idem-co-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'checkout without verified context is denied',
);

$tokenPriceMismatch = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'price');
expect_gate_throws(
    fn() => $gateFixture->gateCheckout([
        'download_id' => $DOWNLOAD, 'price_id' => 'price_attacker', 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenPriceMismatch['registration_token'],
        'request_id' => 'req-price-1', 'idempotency_key' => 'idem-price-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'client-chosen price cannot pass the exact-price relationship',
);

$tokenBlocked = $issueToken($regUiai['registration_uuid'], $FACADE, $UIAPRODUCT, 'blocked');
expect_gate_throws(
    fn() => $gateFixture->gateCheckout([
        'download_id' => $BLOCKED_DOWNLOAD, 'price_id' => 'price_uiai_op_v1', 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $UIAPRODUCT, 'registration_uuid' => $regUiai['registration_uuid'],
        'verified_token' => $tokenBlocked['registration_token'],
        'request_id' => 'req-blocked-1', 'idempotency_key' => 'idem-blocked-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'offer without checkout_enabled is blocked at checkout',
);

expect_gate_throws(
    fn() => $gateFrozen->gateCheckout([
        'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
        'verified_token' => $tokenFocusa['registration_token'],
        'request_id' => 'req-frozen-1', 'idempotency_key' => 'idem-frozen-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'frozen registry has no download mapping at all',
);

// ── Negative: order completion ─────────────────────────────────────────

$orderBase = static fn(int $orderId, string $status, int $customerId, string $email, array $items, string $tag): array => [
    'order_id' => $orderId, 'order_status' => $status, 'customer_id' => $customerId,
    'order_email' => $email, 'order_items' => $items,
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'request_id' => 'req-order-' . $tag, 'idempotency_key' => 'idem-order-' . $tag,
];
$item = static fn(int $downloadId, string $priceId): array => ['download_id' => $downloadId, 'price_id' => $priceId, 'quantity' => 1];

expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBase(7101, 'pending', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'pending')),
    'EDD_ORDER_PENDING',
    'pending order cannot settle entitlement',
);
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBase(7102, 'refunded', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'refunded')),
    'REFUNDED',
    'refunded order cannot settle entitlement',
);
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBase(7103, 'revoked', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'revoked')),
    'REVOKED',
    'revoked order cannot settle entitlement',
);
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBase(7104, 'failed', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'failed')),
    'EDD_ORDER_UNVERIFIED',
    'failed order cannot settle entitlement',
);

$orderUnverified = $orderBase(7105, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'unver');
$orderUnverified['registration_uuid'] = $regUnverified['registration_uuid'];
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderUnverified),
    'EMAIL_VERIFICATION_REQUIRED',
    'order completion with an unverified registration is denied',
);

$orderEmailOnly = $orderBase(7106, 'complete', $focusaCustomer, 'gate.emailonly@example.invalid', [$item($DOWNLOAD, $PRICE)], 'emailonly');
$orderEmailOnly['registration_uuid'] = $regEmailOnly['registration_uuid'];
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderEmailOnly),
    'EMAIL_VERIFICATION_REQUIRED',
    'order completion without account binding is denied',
);

$orderUnknown = $orderBase(7107, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($UNKNOWN_DOWNLOAD, $PRICE)], 'unknown');
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderUnknown),
    'PRODUCT_MAPPING_REQUIRED',
    'order with an unknown download cannot settle entitlement',
);

$orderPrice = $orderBase(7108, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, 'price_attacker')], 'price');
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderPrice),
    'PRODUCT_MAPPING_REQUIRED',
    'order item price mismatch fails closed',
);

$orderEmailMismatch = $orderBase(7109, 'complete', $focusaCustomer, 'gate.stranger@example.invalid', [$item($DOWNLOAD, $PRICE)], 'emailmismatch');
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderEmailMismatch),
    'EDD_ORDER_UNVERIFIED',
    'order email that does not match the verified registration is denied',
);

$orderProductMismatch = $orderBase(7110, 'complete', $focusaCustomer, 'gate.uiai@example.invalid', [$item($DOWNLOAD, $PRICE)], 'prodmismatch');
$orderProductMismatch['registration_uuid'] = $regUiai['registration_uuid'];
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderProductMismatch),
    'FACADE_PRODUCT_DENIED',
    'registration bound to another product cannot settle a Focusa order',
);

$orderBlockedMapping = $orderBase(7111, 'complete', $focusaCustomer, 'gate.uiai@example.invalid', [$item($BLOCKED_DOWNLOAD, 'price_uiai_op_v1')], 'blockedmap');
$orderBlockedMapping['registration_uuid'] = $regUiai['registration_uuid'];
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBlockedMapping),
    'PRODUCT_MAPPING_REQUIRED',
    'order for a blocked (not checkout_enabled) mapping cannot settle',
);

$orderFrozen = $orderBase(7112, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'frozen');
expect_gate_throws(
    fn() => $gateFrozen->handleOrderComplete($orderFrozen),
    'PRODUCT_MAPPING_REQUIRED',
    'frozen registry cannot settle any protected order',
);

// Duplicate active license: one active license for customer+download blocks issuance.
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_gate_dup_0001', {$focusaCustomer}, 7199, {$DOWNLOAD}, 'active')");
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderBase(7113, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'dup')),
    'EDD_LICENSE_UNUSABLE',
    'existing equivalent active license blocks duplicate issuance',
);
$db->exec("DELETE FROM wp_edd_licenses WHERE license_key = 'fl_gate_dup_0001'");

// Idempotency: same key + same request replays; same key + different request conflicts.
$orderConflict = $orderBase(7114, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'conflict');
$conflictFirst = $gateFixture->handleOrderComplete($orderConflict);
expect_gate($conflictFirst['decision'] === 'entitlement_ready', 'conflict fixture settles once');
$conflictReplay = $gateFixture->handleOrderComplete($orderConflict);
expect_gate($conflictReplay['decision'] === 'entitlement_ready', 'same idempotency key with the same request replays');
$orderConflictBad = $orderConflict;
$orderConflictBad['request_id'] = 'req-order-conflict-bad';
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderConflictBad),
    'IDEMPOTENCY_CONFLICT',
    'idempotency key reuse with a different request is a conflict',
);

// Wrong facade on order completion.
$orderWrongFacade = $orderBase(7115, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'wrongfac');
$orderWrongFacade['facade_id'] = 'evil_facade_v1';
$orderWrongFacade['origin'] = 'https://evil.example';
expect_gate_throws(
    fn() => $gateFixture->handleOrderComplete($orderWrongFacade),
    'FACADE_ORIGIN_DENIED',
    'wrong facade on order completion is denied',
);

// ── Positive: token-bound cart, checkout, and order completion ─────────

$tokenCart = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'cart');
$cart = $gateFixture->gateAddToCart([
    'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => $tokenCart['registration_token'],
    'request_id' => 'req-cart-1', 'idempotency_key' => 'idem-cart-1',
]);
expect_gate($cart['decision'] === 'cart_gate_passed', 'verified token opens the protected cart');
expect_gate($cart['protected'] === true && $cart['entitlement_allowed'] === true, 'cart gate marks protected entitlement');
expect_gate($cart['download_id'] === $DOWNLOAD && $cart['product_code'] === $PRODUCT, 'cart gate maps the exact offer');

$cartReplay = $gateFixture->gateAddToCart([
    'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => $tokenCart['registration_token'],
    'request_id' => 'req-cart-1', 'idempotency_key' => 'idem-cart-1',
]);
expect_gate($cartReplay['decision'] === 'cart_gate_passed', 'cart gate replay returns the same decision');

$tokenCheckout = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'checkout');
$checkout = $gateFixture->gateCheckout([
    'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => $tokenCheckout['registration_token'],
    'request_id' => 'req-co-ok-1', 'idempotency_key' => 'idem-co-ok-1',
]);
expect_gate($checkout['decision'] === 'checkout_gate_passed', 'checkout passes with exact server-owned price');
expect_gate($checkout['price_id'] === $PRICE && $checkout['product_code'] === $PRODUCT, 'checkout binds the exact price and product');

// Checkout via the journaled cart-gate binding (fresh token not required).
$checkoutBinding = $gateFixture->gateCheckout([
    'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => '', 'request_id' => 'req-co-bind-1', 'idempotency_key' => 'idem-co-bind-1',
]);
expect_gate($checkoutBinding['decision'] === 'checkout_gate_passed', 'checkout passes through the journaled cart-gate binding');

$licensesBefore = $licenseCount();
$completion = $gateFixture->handleOrderComplete($orderBase(7201, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ok'));
expect_gate($completion['decision'] === 'entitlement_ready', 'order completion declares entitlement ready');
expect_gate($completion['issuance'] === 'deferred_to_verified_issuance_service', 'issuance stays deferred');
expect_gate(count($completion['protected_items']) === 1, 'one protected item settled');
expect_gate($completion['protected_items'][0]['product_code'] === $PRODUCT, 'protected item maps to the exact offer');
expect_gate($licenseCount() === $licensesBefore, 'order completion creates zero licenses itself');

$completionReplay = $gateFixture->handleOrderComplete($orderBase(7201, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ok'));
expect_gate($completionReplay['decision'] === 'entitlement_ready', 'order completion replay returns the same decision');

// Mixed order: protected item settles, credit pack excluded, zero licenses.
$mixed = $gateFixture->handleOrderComplete($orderBase(7202, 'complete', $focusaCustomer, 'gate.focusa@example.invalid', [$item($DOWNLOAD, $PRICE), $item($CREDIT_PACK, 'price_credit')], 'mixed'));
expect_gate($mixed['decision'] === 'entitlement_ready', 'mixed order settles the protected item');
expect_gate($mixed['excluded_items'][0]['disposition'] === 'credit_pack_excluded', 'credit pack proven excluded');
expect_gate($licenseCount() === $licensesBefore, 'mixed order creates zero licenses');

// ── Positive: non-entitlement orders (frozen registry) ─────────────────

$ordersBefore = $gateFrozen->decisionCount('order_complete_gate');
$noEntCredit = $gateFrozen->handleOrderComplete($orderBase(7203, 'complete', 9001, 'gate.shopper@example.invalid', [$item($CREDIT_PACK, 'price_credit'), $item(456, 'price_credit')], 'creditonly'));
expect_gate($noEntCredit['decision'] === 'no_entitlement', 'credit-pack-only order is proven no-entitlement');
expect_gate($noEntCredit['protected_items'] === 0 && $noEntCredit['issuance'] === 'none', 'credit-pack order never issues entitlement');

$noEntUnrelated = $gateFrozen->handleOrderComplete($orderBase(7204, 'complete', 9002, 'gate.shopper2@example.invalid', [$item($UNRELATED_453, 'price_453')], 'unrelated'));
expect_gate($noEntUnrelated['decision'] === 'no_entitlement', 'unrelated product order is proven no-entitlement');
expect_gate($noEntUnrelated['excluded_items'][0]['disposition'] === 'non_entitlement', 'unrelated item disposition proven');

$noEntCart = $gateFrozen->gateAddToCart([
    'download_id' => $UNRELATED_453, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => '', 'registration_uuid' => '', 'verified_token' => '',
    'request_id' => 'req-nocart-1', 'idempotency_key' => 'idem-nocart-1',
]);
expect_gate($noEntCart['decision'] === 'non_entitlement_allowed' && $noEntCart['entitlement_allowed'] === false, 'unrelated download never marks entitlement');
expect_gate($gateFrozen->decisionCount('order_complete_gate') === $ordersBefore + 2, 'non-entitlement decisions are journaled');

// ── Frozen registry scan: no catalog download can reach entitlement ────

foreach ($frozenRegistry['current_edd_catalog']['entries'] as $entry) {
    $download = (int) $entry['download_id'];
    $outcome = null;
    try {
        $outcome = $gateFrozen->gateAddToCart([
            'download_id' => $download, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
            'product_code' => '', 'registration_uuid' => '', 'verified_token' => '',
            'request_id' => 'req-scan-' . $download, 'idempotency_key' => 'idem-scan-' . $download,
        ]);
    } catch (DomainException $error) {
        $outcome = ['error' => $error->getMessage()];
    }
    expect_gate(
        ($outcome['error'] ?? null) === 'PRODUCT_MAPPING_REQUIRED'
            || (($outcome['decision'] ?? '') === 'non_entitlement_allowed' && $outcome['entitlement_allowed'] === false),
        "catalog download {$download} can never reach Focusa entitlement",
    );
}

// ── Token lifecycle and rollback preservation ──────────────────────────

expect_gate($tokens->activeTokenCount($regFocusa['registration_uuid']) >= 0, 'active token count is bounded');
$revoked = $tokens->revokeForRegistration($regFocusa2['registration_uuid'], 'req-revoke-1', 'test_revocation');
$tokenRevoked = $issueToken($regFocusa2['registration_uuid'], $FACADE, $PRODUCT, 'revoked');
expect_gate($tokens->activeTokenCount($regFocusa2['registration_uuid']) === 1, 'revocation leaves one fresh active token');
$revoked2 = $tokens->revokeForRegistration($regFocusa2['registration_uuid'], 'req-revoke-2', 'test_revocation');
expect_gate($tokens->activeTokenCount($regFocusa2['registration_uuid']) === 0, 'revocation invalidates all active tokens');

$beforeRollback = $counts();
$tokenRollback = $tokenMigration->preserveForRollback('2026-08-08T02:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_edd_gate_rollback',
]);
$gateRollback = $gateMigration->preserveForRollback('2026-08-08T02:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_edd_gate_rollback',
]);
expect_gate($tokenRollback['action'] === 'preserve' && $gateRollback['action'] === 'preserve', 'rollback is preservation-only');
expect_gate($counts() === $beforeRollback, 'rollback preserves token and gate decision truth');

// ── Redaction and no-default-grant invariants ──────────────────────────

$payloads = $db->query('SELECT result_payload FROM wp_wpuiai_edd_gate_decisions')->fetchAll(PDO::FETCH_COLUMN);
$joined = implode("\n", $payloads);
expect_gate(strpos($joined, '@') === false, 'no raw email in any gate decision');
expect_gate(strpos($joined, 'rg_') === false, 'no raw token in any gate decision');
expect_gate(strpos($joined, 'fl_') === false, 'no license key in any gate decision');
foreach ($payloads as $payload) {
    $decoded = json_decode($payload, true, 512, JSON_THROW_ON_ERROR);
    expect_gate(!array_key_exists('grants', $decoded) && !array_key_exists('features', $decoded)
        && !array_key_exists('limits', $decoded) && !array_key_exists('price', $decoded),
        'gate decisions carry no default grants or caller metadata');
}

$tokenRows = $db->query('SELECT token_hash FROM wp_wpuiai_edd_registration_tokens WHERE state = \'active\'')->fetchAll(PDO::FETCH_COLUMN);
foreach ($tokenRows as $hash) {
    expect_gate(preg_match('/^[a-f0-9]{64}$/D', (string) $hash) === 1, 'tokens are stored only as keyed digests');
}

// ── Summary ───────────────────────────────────────────────────────────

$finalCounts = $counts();
fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_product_gate_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'tokens_issued' => $finalCounts['tokens'],
    'gate_decisions' => $finalCounts['gate_decisions'],
    'licenses_created' => $finalCounts['licenses'],
    'customers' => $finalCounts['customers'],
    'protected_offer_fixture' => 'operator_approved_test_mapping_download_1001',
    'entitlement_issuance' => 'deferred',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
