<?php
// 152E.02.03 Hold fulfillment on checkout email change or account mismatch.
// The EDD checkout validation/completion surface locks the checkout email to the
// mailbox-verified registration identity: a protected order proceeds only when the
// order email matches the verified registration digest, or when it is an already
// mailbox-verified identity safely linked to the same promoted authority account.
// Changed, blank, or conflicting checkout emails (and order customer/account
// mismatches) journal a bounded fulfillment hold with an opaque review handle,
// entitlement_allowed false, and zero licenses — payment success alone can never
// promote the new email or issue a key. A hold is released only by a separately
// verified link review (mailbox-verified identity of the exact same account plus an
// opaque review evidence handle); release marks entitlement_ready but issuance stays
// deferred. Journals store only keyed digests and opaque handles: no raw email.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-verified-registration-token-validator.v1.php';
require_once $root . '/docs/contracts/spec152e-checkout-email-integrity.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_integrity(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_integrity_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'checkout_email_integrity_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'checkout_email_integrity_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'checkout_email_integrity_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'checkout_email_integrity_test']);
$integrityMigration = new FocusaSpec152eCheckoutEmailIntegrityMigration($db, 'wp_');
$integrityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'checkout_email_integrity_test']);

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

$integrityFrozen = new FocusaSpec152eCheckoutEmailIntegrityService(
    $db, $integrityMigration, $registrations, $registrationSecrets, $identities, $accounts,
    $frozenRegistry, $facadeRegistry, $clock,
);
$integrityFixture = new FocusaSpec152eCheckoutEmailIntegrityService(
    $db, $integrityMigration, $registrations, $registrationSecrets, $identities, $accounts,
    $fixtureRegistry, $facadeRegistry, $clock,
);

// ── Fixture helpers ────────────────────────────────────────────────────

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = false) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.checkout.email.integrity.test',
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
    $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $seq,
        'idem-verify-' . $tag . '-' . $seq,
    );
    if (!$promote) {
        return ['registration_uuid' => $uuid];
    }
    $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'checkout-email-integrity-' . $tag . '-' . $seq],
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

$accountOf = static function (string $registrationUuid) use ($registrations): string {
    return (string) $registrations->findByUuid($registrationUuid)['account_uuid'];
};

$licenseCount = static function () use ($db): int {
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

$orderBase = static fn(int $orderId, string $status, int $customerId, string $email, array $items, string $tag, string $registrationUuid, string $facade = 'focusa_install_v1', string $origin = 'https://install.focusa.dev'): array => [
    'order_id' => $orderId,
    'order_status' => $status,
    'customer_id' => $customerId,
    'order_email' => $email,
    'order_items' => $items,
    'facade_id' => $facade,
    'origin' => $origin,
    'registration_uuid' => $registrationUuid,
    'request_id' => 'req-' . $tag,
    'idempotency_key' => 'idem-' . $tag,
];

$item = static fn(int $download, string $price): array => [
    'download_id' => $download,
    'price_id' => $price,
    'quantity' => 1,
];

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$PRODUCT = 'focusa_operator_lifetime_v1';
$UIAPRODUCT = 'uiai_operator_lifetime_v1';
$DOWNLOAD = 1001;
$PRICE = 'price_focusa_op_v1';
$NON_ENTITLEMENT_DOWNLOAD = 16;
$REG_EMAIL = 'integrity.owner@example.invalid';

// ── Frozen registry invariants (generated contracts remain current) ────

expect_integrity($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema');
expect_integrity($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_integrity($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_integrity($offer['mapping_status'] === 'approved_policy_blocked_edd_mapping', 'frozen offer mapping blocked');
    expect_integrity($offer['checkout_enabled'] === false, 'frozen offer checkout disabled');
    expect_integrity($offer['edd_download_id'] === null, 'frozen offer unassigned download');
}

// ── Negative: order completion preconditions ───────────────────────────

$seq++;
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7101, 'complete', 1, $REG_EMAIL, [$item($DOWNLOAD, $PRICE)], 'ci-missing-1', '00000000-0000-4000-8000-000000000000')),
    'EMAIL_VERIFICATION_REQUIRED',
    'unknown registration cannot assess a protected order',
);

$regUnverified = $createRegistration('integrity.unverified@example.invalid', $FACADE, $PRODUCT, 'unver', false, false);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7102, 'complete', 1, 'integrity.unverified@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-unver-1', $regUnverified['registration_uuid'])),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration cannot assess a protected order',
);

$regEmailOnly = $createRegistration('integrity.emailonly@example.invalid', $FACADE, $PRODUCT, 'emailonly', true, false);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7103, 'complete', 1, 'integrity.emailonly@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-nopromote-1', $regEmailOnly['registration_uuid'])),
    'EDD_CUSTOMER_RESOLUTION_FAILED',
    'verified but unpromoted registration cannot assess a protected order',
);

// Order status gate.
$regStatus = $createRegistration('integrity.status@example.invalid', $FACADE, $PRODUCT, 'status', true, true);
$statusCustomer = $customerOf($regStatus['registration_uuid']);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7104, 'pending', $statusCustomer, 'integrity.status@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-pending-1', $regStatus['registration_uuid'])),
    'EDD_ORDER_PENDING',
    'pending order cannot assess as complete',
);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7105, 'refunded', $statusCustomer, 'integrity.status@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-refunded-1', $regStatus['registration_uuid'])),
    'REFUNDED',
    'refunded order is refused before any integrity decision',
);

// Wrong facade binding.
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7106, 'complete', $statusCustomer, 'integrity.status@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-wrongfac-1', $regStatus['registration_uuid'], 'focusa_arena_v1', 'https://arena.focusa.dev')),
    'FACADE_ORIGIN_DENIED',
    'wrong facade binding is denied',
);

// Expired registration.
$regExpired = $createRegistration('integrity.expired@example.invalid', $FACADE, $PRODUCT, 'expired', true, true);
$nowValue = '2026-08-10T00:00:00Z';
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7107, 'complete', $customerOf($regExpired['registration_uuid']), 'integrity.expired@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-expired-1', $regExpired['registration_uuid'])),
    'REGISTRATION_EXPIRED',
    'expired registration cannot assess a protected order',
);
$nowValue = '2026-08-08T00:01:00Z';

// Denied (terminal) registration.
$regDenied = $createRegistration('integrity.denied@example.invalid', $FACADE, $PRODUCT, 'denied', true, true);
$transition($regDenied['registration_uuid'], 'account_promoted', 'denied', 'denied');
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7108, 'complete', $customerOf($regDenied['registration_uuid']), 'integrity.denied@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-denied-1', $regDenied['registration_uuid'])),
    'EMAIL_VERIFICATION_REQUIRED',
    'denied registration cannot assess a protected order',
);

// Unknown download, blocked mapping, wrong price, frozen registry.
$regBadItem = $createRegistration('integrity.baditem@example.invalid', $FACADE, $PRODUCT, 'baditem', true, true);
$badItemCustomer = $customerOf($regBadItem['registration_uuid']);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7109, 'complete', $badItemCustomer, 'integrity.baditem@example.invalid', [$item(7777, 'price_x')], 'ci-unknown-1', $regBadItem['registration_uuid'])),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown download id is refused',
);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7110, 'complete', $badItemCustomer, 'integrity.baditem@example.invalid', [$item($DOWNLOAD, 'price_wrong')], 'ci-wrongprice-1', $regBadItem['registration_uuid'])),
    'PRODUCT_MAPPING_REQUIRED',
    'wrong price id is refused',
);
$regUiai = $createRegistration('integrity.uiai@example.invalid', $FACADE, $UIAPRODUCT, 'uiai', true, true);
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7111, 'complete', $customerOf($regUiai['registration_uuid']), 'integrity.uiai@example.invalid', [$item(1002, 'price_uiai_op_v1')], 'ci-blocked-1', $regUiai['registration_uuid'])),
    'EDD_CHECKOUT_REQUIRED',
    'approved-but-blocked mapping refuses a protected order',
);
$regFrozen = $createRegistration('integrity.frozen@example.invalid', $FACADE, $PRODUCT, 'frozen', true, true);
expect_integrity_throws(
    fn() => $integrityFrozen->assessOrder($orderBase(7112, 'complete', $customerOf($regFrozen['registration_uuid']), 'integrity.frozen@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-frozen-1', $regFrozen['registration_uuid'])),
    'PRODUCT_MAPPING_REQUIRED',
    'frozen registry (no assigned checkout-enabled download) refuses a protected order',
);

// Existing equivalent active license.
$regDuplicate = $createRegistration('integrity.duplicate@example.invalid', $FACADE, $PRODUCT, 'duplicate', true, true);
$duplicateCustomer = $customerOf($regDuplicate['registration_uuid']);
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_duplicate_fixture_key_000000000000000000000000', {$duplicateCustomer}, 1, {$DOWNLOAD}, 'active')");
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7113, 'complete', $duplicateCustomer, 'integrity.duplicate@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-duplicate-1', $regDuplicate['registration_uuid'])),
    'EDD_LICENSE_UNUSABLE',
    'existing equivalent active license refuses a duplicate protected order',
);
$db->exec("DELETE FROM wp_edd_licenses");
expect_integrity((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === 0, 'license fixture cleaned before the integrity matrix');

// Caller-controlled commerce fields are impossible.
$forbiddenCommerce = [
    'price' => '1.00', 'amount' => '1.00', 'grants' => [$PRODUCT], 'features' => ['focusa.core.mission'],
    'limits' => ['nodes' => 99], 'product_code' => $PRODUCT, 'edd_download_id' => $DOWNLOAD,
    'edd_price_id' => $PRICE, 'license_type' => $PRODUCT, 'node_limit' => 99,
];
foreach ($forbiddenCommerce as $field => $value) {
    $seq++;
    $input = $orderBase(7114, 'complete', $duplicateCustomer, 'integrity.duplicate@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-forbidden-' . $field, $regDuplicate['registration_uuid']);
    $input[$field] = $value;
    expect_integrity_throws(
        fn() => $integrityFixture->assessOrder($input),
        'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
        "caller-controlled {$field} is rejected",
    );
}

// Input bounds fail closed.
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder(['order_id' => 1, 'order_status' => 'complete', 'customer_id' => 1, 'order_email' => 'x@example.invalid', 'order_items' => [$item($DOWNLOAD, $PRICE)], 'facade_id' => $FACADE, 'origin' => $ORIGIN, 'registration_uuid' => $regDuplicate['registration_uuid'], 'request_id' => 'short', 'idempotency_key' => 'idem-ci-badreq-1']),
    'bounded request ID required',
    'undersized request ID is rejected',
);

// ── Positive: matching verified checkout email proceeds ───────────────

$regOwner = $createRegistration($REG_EMAIL, $FACADE, $PRODUCT, 'owner', true, true);
$ownerCustomer = $customerOf($regOwner['registration_uuid']);
$ownerAccount = $accountOf($regOwner['registration_uuid']);

$assessPass = $integrityFixture->assessOrder($orderBase(7201, 'complete', $ownerCustomer, $REG_EMAIL, [$item($DOWNLOAD, $PRICE)], 'ci-pass-1', $regOwner['registration_uuid']));
expect_integrity($assessPass['schema'] === 'focusa.spec152e.checkout_email_integrity_decision.v1', 'assessment decision schema');
expect_integrity($assessPass['decision'] === 'email_integrity_passed', 'matching checkout email passes integrity');
expect_integrity($assessPass['entitlement_allowed'] === true, 'matching checkout email allows entitlement settlement');
expect_integrity($assessPass['issuance'] === 'deferred_to_verified_issuance_service', 'matching assessment defers issuance');
expect_integrity($assessPass['order_id'] === 7201, 'assessment binds the order id');
expect_integrity($assessPass['registration_id'] === $regOwner['registration_uuid'], 'assessment binds the registration');
expect_integrity($assessPass['account_id'] === $ownerAccount, 'assessment binds the promoted account');
expect_integrity($assessPass['customer_id'] === $ownerCustomer, 'assessment binds the promoted customer');
expect_integrity($assessPass['product_code'] === $PRODUCT, 'assessment binds the server-owned product code');
expect_integrity($assessPass['protected_items'][0]['download_id'] === $DOWNLOAD, 'assessment item uses the registry download');
expect_integrity($assessPass['email_matches_verified_identity'] === true, 'matching assessment marks the verified identity match');
expect_integrity($assessPass['replayed'] === false && $assessPass['existing'] === false, 'first matching assessment is fresh');
expect_integrity($integrityFixture->holdCount() === 1, 'matching assessment journals one hold row');
expect_integrity($licenseCount() === 0, 'matching assessment creates no license');
expect_integrity($integrityFixture->releaseCount() === 0, 'matching assessment creates no release');

// Idempotent replay returns the same decision.
$assessPassReplay = $integrityFixture->assessOrder($orderBase(7201, 'complete', $ownerCustomer, $REG_EMAIL, [$item($DOWNLOAD, $PRICE)], 'ci-pass-1', $regOwner['registration_uuid']));
expect_integrity($assessPassReplay['decision'] === 'email_integrity_passed' && $assessPassReplay['replayed'] === true, 'matching assessment replay is idempotent');
expect_integrity($integrityFixture->holdCount() === 1, 'replay creates no second journal row');

// Repeated canonical request (new idempotency key) returns the existing decision.
$assessPassRepeat = $integrityFixture->assessOrder($orderBase(7201, 'complete', $ownerCustomer, $REG_EMAIL, [$item($DOWNLOAD, $PRICE)], 'ci-pass-repeat-1', $regOwner['registration_uuid']));
expect_integrity($assessPassRepeat['decision'] === 'email_integrity_passed' && $assessPassRepeat['existing'] === true, 'repeated matching request returns the existing decision');
expect_integrity($integrityFixture->holdCount() === 1, 'repeated request creates no second journal row');

// ── Negative: changed, blank, conflicting, and account-mismatch holds ──

// Changed email: syntactically valid but not verified anywhere.
$regChanged = $createRegistration('integrity.changed@example.invalid', $FACADE, $PRODUCT, 'changed', true, true);
$changedCustomer = $customerOf($regChanged['registration_uuid']);
$assessChanged = $integrityFixture->assessOrder($orderBase(7202, 'complete', $changedCustomer, 'integrity.stranger@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-changed-1', $regChanged['registration_uuid']));
expect_integrity($assessChanged['decision'] === 'fulfillment_held', 'changed checkout email holds fulfillment');
expect_integrity($assessChanged['mismatch_kind'] === 'changed', 'changed checkout email mismatch kind is changed');
expect_integrity($assessChanged['error_code'] === 'ACCOUNT_EMAIL_MISMATCH', 'changed checkout email fails with ACCOUNT_EMAIL_MISMATCH');
expect_integrity($assessChanged['entitlement_allowed'] === false, 'changed checkout email creates no deliverable entitlement');
expect_integrity($assessChanged['issuance'] === 'held_until_email_verified', 'changed checkout email issuance is held');
expect_integrity(str_starts_with((string) $assessChanged['hold_key'], 'fh_'), 'hold key is opaque and prefixed');
expect_integrity(str_starts_with((string) $assessChanged['review_handle'], 'hr_'), 'review handle is opaque and prefixed');
expect_integrity($assessChanged['product_code'] === $PRODUCT, 'held assessment binds the server-owned product code');
expect_integrity($licenseCount() === 0, 'changed checkout email creates no license');
$changedHold = $integrityFixture->findByHoldKey((string) $assessChanged['hold_key']);
expect_integrity($changedHold !== null && $changedHold['hold_state'] === 'held', 'changed email hold is journaled as held');
expect_integrity($changedHold['order_email_lookup_digest'] === hash_hmac('sha256', "focusa.spec152e.registration.email.lookup.v1\0integrity.stranger@example.invalid", str_repeat('v', 32)), 'hold stores only the keyed digest of the changed email');
expect_integrity($changedHold['expected_email_lookup_digest'] === hash_hmac('sha256', "focusa.spec152e.registration.email.lookup.v1\0integrity.changed@example.invalid", str_repeat('v', 32)), 'hold stores only the keyed digest of the verified registration email');

// The hold persists across repeats and retries (payment cannot bypass).
$assessChangedRepeat = $integrityFixture->assessOrder($orderBase(7202, 'complete', $changedCustomer, 'integrity.stranger@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-changed-repeat-1', $regChanged['registration_uuid']));
expect_integrity($assessChangedRepeat['decision'] === 'fulfillment_held' && $assessChangedRepeat['existing'] === true, 'changed email hold persists across repeats');
expect_integrity($assessChangedRepeat['hold_key'] === $assessChanged['hold_key'], 'repeated hold returns the same hold key');
expect_integrity($integrityFixture->holdCount() === 2, 'one hold row per order/registration despite repeats');

// Idempotency conflict: same key, different request.
expect_integrity_throws(
    fn() => $integrityFixture->assessOrder($orderBase(7202, 'complete', $changedCustomer, 'integrity.other@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-changed-1', $regChanged['registration_uuid'])),
    'IDEMPOTENCY_CONFLICT',
    'idempotency-key reuse with a different request is rejected',
);

// Blank email.
$regBlank = $createRegistration('integrity.blank@example.invalid', $FACADE, $PRODUCT, 'blank', true, true);
$assessBlank = $integrityFixture->assessOrder($orderBase(7203, 'complete', $customerOf($regBlank['registration_uuid']), '', [$item($DOWNLOAD, $PRICE)], 'ci-blank-1', $regBlank['registration_uuid']));
expect_integrity($assessBlank['decision'] === 'fulfillment_held', 'blank checkout email holds fulfillment');
expect_integrity($assessBlank['mismatch_kind'] === 'blank', 'blank checkout email mismatch kind is blank');
expect_integrity($assessBlank['error_code'] === 'EDD_ORDER_UNVERIFIED', 'blank checkout email fails with EDD_ORDER_UNVERIFIED');
expect_integrity($assessBlank['entitlement_allowed'] === false, 'blank checkout email creates no deliverable entitlement');
expect_integrity($licenseCount() === 0, 'blank checkout email creates no license');
$blankHold = $integrityFixture->findByHoldKey((string) $assessBlank['hold_key']);
expect_integrity($blankHold['order_email_lookup_digest'] === null, 'blank email hold stores no order digest');

// Conflicting email: verified identity bound to a different account.
$regOther = $createRegistration('integrity.other@example.invalid', $FACADE, $PRODUCT, 'other', true, true);
$otherAccount = $accountOf($regOther['registration_uuid']);
$identities->storeVerified('integrity.conflict@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'identity_uuid' => '22222222-2222-4222-8222-222222222222',
    'account_uuid' => $otherAccount,
    'identity_state' => 'linked',
    'verified_at' => '2026-08-08T00:01:00Z',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:01:00Z',
    'source' => 'candidate_contract',
    'migration_evidence' => ['record' => 'conflict-fixture'],
]);
$regConflict = $createRegistration('integrity.conflictowner@example.invalid', $FACADE, $PRODUCT, 'conflict', true, true);
$assessConflict = $integrityFixture->assessOrder($orderBase(7204, 'complete', $customerOf($regConflict['registration_uuid']), 'integrity.conflict@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-conflict-1', $regConflict['registration_uuid']));
expect_integrity($assessConflict['decision'] === 'fulfillment_held', 'conflicting verified email holds fulfillment');
expect_integrity($assessConflict['mismatch_kind'] === 'conflicting', 'conflicting email mismatch kind is conflicting');
expect_integrity($assessConflict['error_code'] === 'ACCOUNT_MERGE_REVIEW_REQUIRED', 'conflicting email fails with ACCOUNT_MERGE_REVIEW_REQUIRED');
expect_integrity($assessConflict['entitlement_allowed'] === false, 'conflicting email creates no deliverable entitlement');
expect_integrity($licenseCount() === 0, 'conflicting email creates no license');

// Account mismatch: order under a different EDD customer/account.
$regAccount = $createRegistration('integrity.account@example.invalid', $FACADE, $PRODUCT, 'account', true, true);
$assessAccount = $integrityFixture->assessOrder($orderBase(7205, 'complete', $customerOf($regOther['registration_uuid']), 'integrity.account@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-account-1', $regAccount['registration_uuid']));
expect_integrity($assessAccount['decision'] === 'fulfillment_held', 'order customer/account mismatch holds fulfillment');
expect_integrity($assessAccount['mismatch_kind'] === 'account', 'account mismatch kind is account');
expect_integrity($assessAccount['error_code'] === 'ACCOUNT_MERGE_REVIEW_REQUIRED', 'account mismatch fails with ACCOUNT_MERGE_REVIEW_REQUIRED');
expect_integrity($assessAccount['entitlement_allowed'] === false, 'account mismatch creates no deliverable entitlement');
expect_integrity($licenseCount() === 0, 'account mismatch creates no license');

// ── Positive: already verified and safely linked email proceeds ───────

$linkIdentityUuid = '33333333-3333-4333-8333-333333333333';
$identities->storeVerified('integrity.linked@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'identity_uuid' => $linkIdentityUuid,
    'account_uuid' => $ownerAccount,
    'identity_state' => 'linked',
    'verified_at' => '2026-08-08T00:01:00Z',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:01:00Z',
    'source' => 'candidate_contract',
    'migration_evidence' => ['record' => 'linked-identity-fixture'],
]);
$assessLinked = $integrityFixture->assessOrder($orderBase(7206, 'complete', $ownerCustomer, 'integrity.linked@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-linked-1', $regOwner['registration_uuid']));
expect_integrity($assessLinked['decision'] === 'email_integrity_passed', 'verified and safely linked email proceeds');
expect_integrity($assessLinked['entitlement_allowed'] === true, 'safely linked email allows entitlement settlement');
expect_integrity($assessLinked['verified_identity_id'] === $linkIdentityUuid, 'safely linked assessment binds the verified identity');
expect_integrity($licenseCount() === 0, 'safely linked assessment creates no license');

// ── Release: only a separately verified link review releases the hold ──

// Payment evidence alone can never release.
expect_integrity_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => (string) $assessChanged['hold_key'],
        'order_email' => 'integrity.stranger@example.invalid',
        'resolved_identity_uuid' => $linkIdentityUuid,
        'release_evidence_handle' => 'ev_payment_success_0000000000000000000000',
        'evidence_kind' => 'payment_success',
        'request_id' => 'req-release-payment-1',
        'idempotency_key' => 'idem-release-payment-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'payment success alone can never release a fulfillment hold',
);
expect_integrity($integrityFixture->findByHoldKey((string) $assessChanged['hold_key'])['hold_state'] === 'held', 'hold remains held after a payment-evidence release attempt');
expect_integrity($licenseCount() === 0, 'payment-evidence release attempt creates no license');

// Releasing an unknown hold fails closed.
expect_integrity_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => 'fh_00000000000000000000000000000000',
        'order_email' => 'integrity.stranger@example.invalid',
        'resolved_identity_uuid' => $linkIdentityUuid,
        'release_evidence_handle' => 'ev_00000000000000000000000000000000',
        'request_id' => 'req-release-unknown-1',
        'idempotency_key' => 'idem-release-unknown-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'unknown hold key cannot be released',
);

// Releasing before the email is verified fails closed.
expect_integrity_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => (string) $assessChanged['hold_key'],
        'order_email' => 'integrity.stranger@example.invalid',
        'resolved_identity_uuid' => '44444444-4444-4444-8444-444444444444',
        'release_evidence_handle' => 'ev_00000000000000000000000000000000',
        'request_id' => 'req-release-unverified-1',
        'idempotency_key' => 'idem-release-unverified-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'releasing without a verified identity fails closed',
);

// Releasing with a verified identity of a different account fails closed: the
// conflicting hold's held email is verified on the other account, so the account
// binding must refuse the release.
expect_integrity_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => (string) $assessConflict['hold_key'],
        'order_email' => 'integrity.conflict@example.invalid',
        'resolved_identity_uuid' => '22222222-2222-4222-8222-222222222222',
        'release_evidence_handle' => 'ev_00000000000000000000000000000000',
        'request_id' => 'req-release-wrongacct-1',
        'idempotency_key' => 'idem-release-wrongacct-1',
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'releasing with a verified identity of another account fails closed',
);
expect_integrity($integrityFixture->findByHoldKey((string) $assessConflict['hold_key'])['hold_state'] === 'held', 'conflicting hold remains held after a wrong-account release attempt');

// Releasing a different email than the held one fails closed.
expect_integrity_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => (string) $assessChanged['hold_key'],
        'order_email' => 'integrity.changed@example.invalid',
        'resolved_identity_uuid' => $linkIdentityUuid,
        'release_evidence_handle' => 'ev_00000000000000000000000000000000',
        'request_id' => 'req-release-wrongemail-1',
        'idempotency_key' => 'idem-release-wrongemail-1',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'releasing a different email than the held one fails closed',
);

// Now verify + safely link the changed email to the same account, then release.
$resolvedIdentityUuid = '55555555-5555-4555-8555-555555555555';
$identities->storeVerified('integrity.stranger@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'identity_uuid' => $resolvedIdentityUuid,
    'account_uuid' => $accountOf($regChanged['registration_uuid']),
    'identity_state' => 'linked',
    'verified_at' => '2026-08-08T00:02:00Z',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:02:00Z',
    'source' => 'candidate_contract',
    'migration_evidence' => ['record' => 'changed-email-verified-link'],
]);
$releaseEvidence = 'ev_5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d';
$assessChangedAfter = $integrityFixture->assessOrder($orderBase(7202, 'complete', $changedCustomer, 'integrity.stranger@example.invalid', [$item($DOWNLOAD, $PRICE)], 'ci-changed-after-1', $regChanged['registration_uuid']));
expect_integrity($assessChangedAfter['decision'] === 'fulfillment_held' && $assessChangedAfter['existing'] === true, 'the journaled hold still blocks the same order even after the email is linked');
expect_integrity($licenseCount() === 0, 'no license before the release review');

$released = $integrityFixture->releaseHold([
    'hold_key' => (string) $assessChanged['hold_key'],
    'order_email' => 'integrity.stranger@example.invalid',
    'resolved_identity_uuid' => $resolvedIdentityUuid,
    'release_evidence_handle' => $releaseEvidence,
    'request_id' => 'req-release-verified-1',
    'idempotency_key' => 'idem-release-verified-1',
]);
expect_integrity($released['decision'] === 'fulfillment_released', 'verified link review releases the hold');
expect_integrity($released['hold_key'] === $assessChanged['hold_key'], 'release binds the held hold key');
expect_integrity($released['review_handle'] === $assessChanged['review_handle'], 'release preserves the opaque review handle');
expect_integrity($released['mismatch_kind'] === 'changed', 'release preserves the mismatch kind');
expect_integrity($released['resolved_identity_uuid'] === $resolvedIdentityUuid, 'release binds the separately verified identity');
expect_integrity($released['evidence_kind'] === 'verified_link_review' && $released['decided_by'] === 'verified_link_review', 'release evidence kind is the fixed verified link review');
expect_integrity($released['evidence_handle'] === $releaseEvidence, 'release preserves the opaque evidence handle');
expect_integrity($released['entitlement_allowed'] === true, 'released order is entitlement_ready');
expect_integrity($released['issuance'] === 'deferred_to_verified_issuance_service', 'release defers issuance to the verified issuance service');
expect_integrity($released['replayed'] === false && $released['existing'] === false, 'first verified release is fresh');
expect_integrity($integrityFixture->findByHoldKey((string) $assessChanged['hold_key'])['hold_state'] === 'released', 'hold is journaled as released');
expect_integrity($integrityFixture->releaseCount() === 1, 'exactly one release journaled');
expect_integrity($licenseCount() === 0, 'release review itself creates no license: issuance stays deferred');

// Release replay is idempotent; a repeated canonical release returns the existing release.
$releasedReplay = $integrityFixture->releaseHold([
    'hold_key' => (string) $assessChanged['hold_key'],
    'order_email' => 'integrity.stranger@example.invalid',
    'resolved_identity_uuid' => $resolvedIdentityUuid,
    'release_evidence_handle' => $releaseEvidence,
    'request_id' => 'req-release-verified-1',
    'idempotency_key' => 'idem-release-verified-1',
]);
expect_integrity($releasedReplay['decision'] === 'fulfillment_released' && $releasedReplay['replayed'] === true, 'verified release replay is idempotent');
$releasedRepeat = $integrityFixture->releaseHold([
    'hold_key' => (string) $assessChanged['hold_key'],
    'order_email' => 'integrity.stranger@example.invalid',
    'resolved_identity_uuid' => $resolvedIdentityUuid,
    'release_evidence_handle' => $releaseEvidence,
    'request_id' => 'req-release-verified-repeat-1',
    'idempotency_key' => 'idem-release-verified-repeat-1',
]);
expect_integrity($releasedRepeat['decision'] === 'fulfillment_released' && $releasedRepeat['existing'] === true, 'repeated release returns the existing release');
expect_integrity($integrityFixture->releaseCount() === 1, 'release replays create no second release journal');

// ── Unrelated order: no protected item, no identity requirement, no entitlement ──

$assessUnrelated = $integrityFixture->assessOrder($orderBase(7207, 'complete', 1, 'unrelated.buyer@example.invalid', [$item($NON_ENTITLEMENT_DOWNLOAD, 'price_unrelated')], 'ci-unrelated-1', ''));
expect_integrity($assessUnrelated['decision'] === 'no_entitlement', 'unrelated order is non-entitlement');
expect_integrity($assessUnrelated['protected_items'] === 0, 'unrelated order carries zero protected items');
expect_integrity($assessUnrelated['issuance'] === 'none', 'unrelated order issues nothing');
expect_integrity($licenseCount() === 0, 'unrelated order creates no license');

// ── Rollback preservation and redaction ────────────────────────────────

$preserved = $integrityMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'checkout_email_integrity_test', 'record' => 'rollback']);
expect_integrity($preserved['action'] === 'preserve', 'rollback preservation event recorded');
expect_integrity((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_checkout_email_integrity_schema_events')->fetchColumn() === 1, 'exactly one preservation event journaled');

$resultJson = json_encode([
    $assessPass, $assessPassReplay, $assessPassRepeat, $assessChanged, $assessChangedRepeat,
    $assessBlank, $assessConflict, $assessAccount, $assessLinked, $assessChangedAfter,
    $released, $releasedReplay, $releasedRepeat, $assessUnrelated,
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_integrity(strpos($resultJson, '@') === false, 'no raw email in any integrity decision');
expect_integrity(strpos($resultJson, 'fl_') === false, 'no license key in any integrity decision');
$holdRows = $db->query('SELECT * FROM wp_wpuiai_checkout_email_integrity_holds')->fetchAll(PDO::FETCH_ASSOC);
$holdJson = json_encode($holdRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_integrity(strpos($holdJson, '@') === false, 'no raw email in the hold journal');
foreach ($holdRows as $holdRow) {
    expect_integrity(preg_match('/^(fh_)[0-9a-f]{32}$/D', (string) $holdRow['hold_key']) === 1, 'hold keys are opaque bounded tokens');
    expect_integrity($holdRow['review_handle'] === null || preg_match('/^(hr_)[0-9a-f]{32}$/D', (string) $holdRow['review_handle']) === 1, 'review handles are opaque bounded tokens');
    expect_integrity($holdRow['order_email_lookup_digest'] === null || preg_match('/^[0-9a-f]{64}$/D', (string) $holdRow['order_email_lookup_digest']) === 1, 'order email digests are keyed only');
    expect_integrity($holdRow['expected_email_lookup_digest'] === null || preg_match('/^[0-9a-f]{64}$/D', (string) $holdRow['expected_email_lookup_digest']) === 1, 'expected email digests are keyed only');
}
$releaseRows = $db->query('SELECT * FROM wp_wpuiai_checkout_email_integrity_releases')->fetchAll(PDO::FETCH_ASSOC);
$releaseJson = json_encode($releaseRows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
expect_integrity(strpos($releaseJson, '@') === false, 'no raw email in the release journal');
foreach ($releaseRows as $releaseRow) {
    expect_integrity(preg_match('/^(rl_)[0-9a-f]{32}$/D', (string) $releaseRow['release_key']) === 1, 'release keys are opaque bounded tokens');
    expect_integrity(preg_match('/^(ev_)[0-9a-f]{32}$/D', (string) $releaseRow['evidence_handle']) === 1, 'release evidence handles are opaque bounded tokens');
}

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.checkout_email_integrity_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'holds_journaled' => $integrityFixture->holdCount(),
    'releases_journaled' => $integrityFixture->releaseCount(),
    'licenses_created' => $licenseCount(),
    'mismatch_fixtures' => ['changed', 'blank', 'conflicting', 'account'],
    'release_evidence' => 'verified_link_review_only',
    'entitlement_issuance' => 'deferred',
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
