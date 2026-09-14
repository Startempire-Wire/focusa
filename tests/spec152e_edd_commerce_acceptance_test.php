<?php
// 152E.02.10 Run the complete EDD commerce acceptance matrix (Spec 152E §23) through
// ONE EDD-centered customer/order/license authority with no independent install-site
// issuance. The matrix exercises, end to end and fail-closed:
//   paid Focusa (website/terminal/agent: verified email -> product gate -> branded
//     checkout intent -> email-integrity pass -> order binding -> canonical EDD
//     Software Licensing key -> lifecycle projection -> transactional outbox ->
//     reconciliation convergence; one customer, one order, one license);
//   paid UIAI (explicit UIAI grants only), bundle (explicit Focusa + UIAI grants in
//     one account flow, exact_union, never an implicit single key);
//   Evaluation (verified account + eligibility -> exactly one verified_no_license
//     limited-access posture + signed assertion, zero EDD order/key; paid posture
//     preserved; duplicate/facade-switch denied);
//   wrong product (no cross-product lease, no downgrade), arbitrary amount/grants
//     (caller commerce fields forbidden on every surface), unrelated download
//     (non-entitlement, zero licenses), duplicate/replayed order (one
//     account/order/license result), changed email (fulfillment held until a
//     separately verified link review releases it), refund/revoke/expiry (strictly
//     monotonic sequence, recovery_only, terminal truth never reactivates, history
//     preserved), authority outage (no new local license; existing signed offline
//     policy only), legacy install-site records (evidence-backed migration or
//     quarantine; synthetic keys preserved, never issued), and reconciliation
//     (missing callbacks repaired, ambiguous leases quarantined, converges).
// All fixtures are synthetic; journals store only keyed digests and opaque bounded
// tokens; the full license key appears only inside the bounded delivery envelope;
// no raw email, secret, or unmasked real-email evidence is stored or returned.
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
require_once $root . '/docs/contracts/spec152e-edd-checkout-intent.v1.php';
require_once $root . '/docs/contracts/spec152e-checkout-email-integrity.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-order-binding.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-license-issuance.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';
require_once $root . '/docs/contracts/spec152e-evaluation-issuance.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-reconciliation.v1.php';

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
    expect_matrix($decision['sequence_increment'] === 0, "{$message}: denied events never bump the sequence");
}

function table_count(PDO $db, string $table): int
{
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
}

/** Hook negative: run inside a caller-owned transaction and roll it back after the fail-closed throw. */
function expect_hook_throws(callable $operation, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    $db = $GLOBALS['db'];
    $db->beginTransaction();
    try {
        $operation();
    } catch (Throwable $error) {
        $db->rollBack();
        if ($error->getMessage() !== $code) {
            fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
            exit(1);
        }
        return;
    }
    $db->rollBack();
    fwrite(STDERR, "FAIL: {$message}\n");
    exit(1);
}

// ── Setup: one SQLite authority, all nine surfaces ────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$GLOBALS['db'] = $db;

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$tokenMigration = new FocusaSpec152eEddRegistrationTokenMigration($db, 'wp_');
$tokenMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$gateMigration = new FocusaSpec152eEddGateDecisionMigration($db, 'wp_');
$gateMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$intentMigration = new FocusaSpec152eEddCheckoutIntentMigration($db, 'wp_');
$intentMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$integrityMigration = new FocusaSpec152eCheckoutEmailIntegrityMigration($db, 'wp_');
$integrityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$bindingMigration = new FocusaSpec152eEddOrderBindingMigration($db, 'wp_');
$bindingMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$issuanceMigration = new FocusaSpec152eEddLicenseIssuanceMigration($db, 'wp_');
$issuanceMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$evaluationMigration = new FocusaSpec152eEvaluationIssuanceMigration($db, 'wp_');
$evaluationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$lifecycleMigration = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
$lifecycleMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$outboxMigration = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
$outboxMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);
$reconciliationMigration = new FocusaSpec152eAuthorityReconciliationMigration($db, 'wp_');
$reconciliationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'edd_commerce_acceptance_test']);

// Canonical EDD fixture tables (single authority; the same tables feed every surface).
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
$db->exec("CREATE TABLE wp_edd_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
// Test-owned consumer side-effect table: proves exactly-once delivery.
$db->exec("CREATE TABLE wp_outbox_test_applications (
    event_uuid VARCHAR(128) NOT NULL PRIMARY KEY,
    event_type VARCHAR(32) NOT NULL,
    applied_at VARCHAR(32) NOT NULL
)");

$nowValue = '2026-08-08T00:01:00Z';
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};
$tick = static function (int $seconds) use (&$nowValue): void {
    $nowValue = gmdate('Y-m-d\TH:i:s\Z', (int) (new DateTimeImmutable($nowValue))->format('U') + $seconds);
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
$postures = new FocusaSpec172VerifiedAccessPostureRepository($db, $postureMigration, $clock);
$assertions = new FocusaSpec172SignedAccessAssertionRepository($db, $assertionMigration, $postureMigration, $clock);
$projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleMigration, 'wp_', $clock);

// The frozen registry is the operator contract and stays untouched: zero checkout-
// enabled offers, zero assigned EDD downloads. The fixture registry adds explicitly
// operator-approved test mappings (download 1001 -> focusa_operator_lifetime_v1,
// 1002 -> uiai_operator_lifetime_v1, 1003 -> focusa_uiai_operator_bundle_lifetime_v1,
// all active/checkout_enabled with server-owned price ids) so the positive paid matrix
// can run against the same single authority without mutating the frozen contract.
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
// Approved-but-blocked variant: the uiai mapping exists but checkout stays disabled
// (EDD_CHECKOUT_REQUIRED) — an approved policy that never self-enables.
$blockedUiaiRegistry = $fixtureRegistry;
foreach ($blockedUiaiRegistry['protected_offers'] as &$offer) {
    if ($offer['public_code'] === 'uiai_operator_lifetime_v1') {
        $offer['checkout_enabled'] = false;
    }
}
unset($offer);

// Surface instances: frozen (fail-closed outage view) and fixture (operator-approved).
$returnHandles = new FocusaSpec152eFacadeReturnHandleRegistry($facadeRegistry);
$cart = new FocusaSpec152eEddCartSessionAdapter($db, $intentMigration, $clock);
$gateFrozen = new FocusaSpec152eEddGateHooks($db, $gateMigration, $tokens, $registrations, $registrationSecrets, $frozenRegistry, $facadeRegistry, $clock);
$gateFixture = new FocusaSpec152eEddGateHooks($db, $gateMigration, $tokens, $registrations, $registrationSecrets, $fixtureRegistry, $facadeRegistry, $clock);
$checkoutFrozen = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $frozenRegistry, $clock);
$checkoutFixture = new FocusaSpec152eEddCheckoutIntentService($db, $intentMigration, $registrations, $cart, $returnHandles, $fixtureRegistry, $clock);
$integrityFrozen = new FocusaSpec152eCheckoutEmailIntegrityService($db, $integrityMigration, $registrations, $registrationSecrets, $identities, $accounts, $frozenRegistry, $facadeRegistry, $clock);
$integrityFixture = new FocusaSpec152eCheckoutEmailIntegrityService($db, $integrityMigration, $registrations, $registrationSecrets, $identities, $accounts, $fixtureRegistry, $facadeRegistry, $clock);
$bindingFrozen = new FocusaSpec152eEddOrderBindingService($db, $bindingMigration, $registrations, $registrationSecrets, $accounts, $frozenRegistry, $facadeRegistry, $clock);
$bindingFixture = new FocusaSpec152eEddOrderBindingService($db, $bindingMigration, $registrations, $registrationSecrets, $accounts, $fixtureRegistry, $facadeRegistry, $clock);
$bindingBlockedUiai = new FocusaSpec152eEddOrderBindingService($db, $bindingMigration, $registrations, $registrationSecrets, $accounts, $blockedUiaiRegistry, $facadeRegistry, $clock);
$issuanceFrozen = new FocusaSpec152eEddLicenseIssuanceService($db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd, $frozenRegistry, $clock);
$issuanceFixture = new FocusaSpec152eEddLicenseIssuanceService($db, $issuanceMigration, $bindingMigration, $registrations, $registrationSecrets, $edd, $fixtureRegistry, $clock);
$evaluation = new FocusaSpec152eEvaluationIssuanceService(
    $db, $evaluationMigration, $registrations, $accounts, $edd,
    $postureMigration, $postures, $assertions, $clock, 'wp_',
);
$secret = 'spec152e-commerce-acceptance-hmac-secret-v1';
$signer = new FocusaSpec152eAuthorityEventSigner($secret);
$eventSchema = new FocusaSpec152eAuthorityEventSchema();
$hook = new FocusaSpec152eEddAuthorityHook($db, $outboxMigration, $eventSchema, $signer, $accounts, 'wp_', $clock);
$consumerMode = 'normal';
$deliver = static function (array $event) use ($db, &$consumerMode, &$nowValue): void {
    $statement = $db->prepare('INSERT INTO wp_outbox_test_applications (event_uuid, event_type, applied_at) VALUES (:uuid, :type, :at)');
    $statement->execute([':uuid' => (string) $event['event_uuid'], ':type' => (string) $event['event_type'], ':at' => $nowValue]);
    if ($consumerMode === 'consumer_down') {
        throw new DomainException('DELIVERY_CONSUMER_DOWN');
    }
};
$dispatcher = new FocusaSpec152eAuthorityOutboxDispatcher($db, $outboxMigration, $signer, $eventSchema, $deliver, $clock, 'wp_', 3, 60);
$classifier = new FocusaSpec152eDiscrepancyClassifier();
$reconciler = new FocusaSpec152eAuthorityReconciler($db, $reconciliationMigration, $accounts, $projector, $hook, $classifier, 'wp_', $clock);

// ── Fixture helpers ───────────────────────────────────────────────────

$FACADE = 'focusa_install_v1';
$ORIGIN = 'https://install.focusa.dev';
$PRODUCT = 'focusa_operator_lifetime_v1';
$UIAPRODUCT = 'uiai_operator_lifetime_v1';
$BUNDLE = 'focusa_uiai_operator_bundle_lifetime_v1';
$DOWNLOAD = 1001;
$PRICE = 'price_focusa_op_v1';
$UIADOWNLOAD = 1002;
$UIAIPRICE = 'price_uiai_op_v1';
$BUNDLEDOWNLOAD = 1003;
$BUNDLEPRICE = 'price_bundle_op_v1';
$KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
$KEY_SCAN_PATTERN = '/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/D';
$EVAL_SIGNATURE = 'sig_spec152e_commerce_' . str_repeat('a', 40);

$seq = 0;
$createRegistration = static function (string $email, string $facade, string $product, string $tag, bool $verify = true, bool $promote = true) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.edd.commerce.acceptance.test',
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
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'commerce-matrix-' . $tag . '-' . $seq],
    ]);
    $result['account_uuid'] = (string) $promotionResult['account_uuid'];
    $result['identity_uuid'] = (string) $promotionResult['identity_uuid'];
    $result['edd_customer_id'] = (int) $registrations->findByUuid($uuid)['edd_customer_id'];
    return $result;
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

$rowSeq = 0;
$insertOrder = static function (int $orderId, string $status, int $customerId, string $email, array $items = []) use ($db, &$rowSeq): void {
    $statement = $db->prepare("INSERT INTO wp_edd_orders
        (id, order_number, status, type, date_created, date_completed, user_id, customer_id, email)
        VALUES (:id, :number, :status, 'sale', '2026-08-08T00:01:00Z', :completed, NULL, :customer, :email)");
    $statement->execute([
        ':id' => $orderId,
        ':number' => 'EDD-' . $orderId,
        ':status' => $status,
        ':completed' => $status === 'complete' || $status === 'completed' ? '2026-08-08T00:01:00Z' : null,
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
$insertTransaction = static function (int $orderId, string $transactionId, string $status = 'complete', string $total = '697.00') use ($db, &$txnSeq): void {
    $txnSeq++;
    $statement = $db->prepare("INSERT INTO wp_edd_order_transactions
        (id, order_id, transaction_id, gateway, status, total, currency, date_created)
        VALUES (:id, :order, :txn, 'stripe', :status, :total, 'USD', '2026-08-08T00:01:00Z')");
    $statement->execute([
        ':id' => $txnSeq,
        ':order' => $orderId,
        ':txn' => $transactionId,
        ':status' => $status,
        ':total' => $total,
    ]);
};

$bind = static function (int $orderId, string $registrationUuid, int $customerId, array $items, string $transactionId, string $tag, string $priceId = 'price_focusa_op_v1') use ($bindingFixture, $FACADE, $ORIGIN): array {
    return $bindingFixture->bindOrderComplete([
        'order_id' => $orderId,
        'order_status' => 'complete',
        'customer_id' => $customerId,
        'order_items' => array_map(static fn (array $item) => [
            'order_item_id' => (int) $item['item_id'],
            'download_id' => (int) $item['download'],
            'price_id' => (string) ($item['price'] ?? $priceId),
            'quantity' => 1,
        ], $items),
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => $transactionId, 'status' => 'complete']],
        'registration_uuid' => $registrationUuid,
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'request_id' => 'req-bind-' . $tag,
        'idempotency_key' => 'idem-bind-' . $tag,
    ]);
};

$issue = static function (string $handle, string $requestId, string $idempotencyKey) use ($issuanceFixture): array {
    return $issuanceFixture->issue([
        'issuance_request_handle' => $handle,
        'request_id' => $requestId,
        'idempotency_key' => $idempotencyKey,
    ]);
};

$appendEdd = static function (array $input) use ($hook): array {
    return $hook->appendFromEdd($input);
};

$accountSequence = static function (string $accountUuid) use ($db): int {
    $statement = $db->prepare('SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = :uuid');
    $statement->execute([':uuid' => $accountUuid]);
    return (int) $statement->fetchColumn();
};

$counts = static function () use ($db): array {
    return [
        'customers' => table_count($db, 'wp_edd_customers'),
        'orders' => table_count($db, 'wp_edd_orders'),
        'licenses' => table_count($db, 'wp_edd_licenses'),
        'bindings' => table_count($db, 'wp_wpuiai_edd_order_bindings'),
        'issuance_requests' => table_count($db, 'wp_wpuiai_edd_issuance_requests'),
        'issuances' => table_count($db, 'wp_wpuiai_edd_license_issuances'),
        'postures' => table_count($db, 'wp_wpuiai_verified_access_postures'),
        'assertions' => table_count($db, 'wp_wpuiai_signed_access_assertions'),
        'evaluations' => table_count($db, 'wp_wpuiai_evaluation_issuances'),
        'lifecycle_events' => table_count($db, 'wp_wpuiai_edd_lifecycle_events'),
        'outbox' => table_count($db, 'wp_wpuiai_authority_outbox'),
        'deliveries' => table_count($db, 'wp_wpuiai_outbox_deliveries'),
        'applications' => table_count($db, 'wp_outbox_test_applications'),
    ];
};
$baseline = $counts();

// ── 0. One authority invariants (frozen contracts stay current) ───────

expect_matrix($frozenRegistry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'frozen registry schema is canonical');
expect_matrix($frozenRegistry['counts']['checkout_enabled'] === 0, 'frozen registry has zero checkout-enabled offers');
expect_matrix($frozenRegistry['counts']['assigned_edd_downloads'] === 0, 'frozen registry has zero assigned EDD downloads');
foreach ($frozenRegistry['protected_offers'] as $offer) {
    expect_matrix($offer['mapping_status'] === 'approved_policy_blocked_edd_mapping' && $offer['checkout_enabled'] === false && $offer['edd_download_id'] === null, 'frozen offer is blocked and unassigned');
}
expect_matrix($facadeRegistry['schema'] === 'focusa.spec152e.facade_registry.v1', 'facade registry schema is canonical');
$bundleOffer = null;
foreach ($frozenRegistry['protected_offers'] as $offer) {
    if ($offer['public_code'] === $BUNDLE) {
        $bundleOffer = $offer;
    }
}
expect_matrix($bundleOffer !== null, 'bundle offer exists in the single product authority');
expect_matrix($bundleOffer['grant_composition'] === 'exact_union' && $bundleOffer['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle grants are the explicit Focusa + UIAI union');
expect_matrix($bundleOffer['products'] === ['focusa', 'uiai_engine'] && $bundleOffer['price_usd'] === '1254.60', 'bundle products and price are server-owned');
$returnSuccess = $returnHandles->resolve(['facade_id' => $FACADE, 'origin' => $ORIGIN, 'return_handle' => 'success']);
expect_matrix($returnSuccess['return_url'] === 'https://install.focusa.dev/activate/callback/success', 'branded return handle resolves server-side');

// ── 1. Paid Focusa (website/terminal/agent): one customer, one order, one license ──

$regFocusa = $createRegistration('commerce.focusa@example.invalid', $FACADE, $PRODUCT, 'focusa');
$focusaCustomer = $regFocusa['edd_customer_id'];
$focusaAccount = $regFocusa['account_uuid'];

// Product gate: verified token only, exact server-owned mapping.
$tokenFocusa = $issueToken($regFocusa['registration_uuid'], $FACADE, $PRODUCT, 'focusa');
$cartGate = $gateFixture->gateAddToCart([
    'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => $tokenFocusa['registration_token'],
    'request_id' => 'req-cart-focusa-1', 'idempotency_key' => 'idem-cart-focusa-1',
]);
expect_matrix($cartGate['decision'] === 'cart_gate_passed' && $cartGate['entitlement_allowed'] === true, 'paid Focusa passes the product gate with a verified token');
expect_matrix($cartGate['download_id'] === $DOWNLOAD && $cartGate['product_code'] === $PRODUCT, 'product gate binds the exact server-owned offer');
$checkoutGate = $gateFixture->gateCheckout([
    'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
    'verified_token' => '', 'request_id' => 'req-checkout-gate-focusa-1', 'idempotency_key' => 'idem-checkout-gate-focusa-1',
]);
expect_matrix($checkoutGate['decision'] === 'checkout_gate_passed' && $checkoutGate['price_id'] === $PRICE, 'checkout gate passes on the journaled cart binding with the exact price');

// Branded EDD checkout intent: one server-owned intent, exact price, opaque refs.
$intent = $checkoutFixture->createIntent([
    'registration_uuid' => $regFocusa['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-intent-focusa-1',
    'idempotency_key' => 'idem-intent-focusa-1',
]);
expect_matrix($intent['schema'] === 'focusa.spec152e.checkout_intent_result.v1' && $intent['replayed'] === false, 'paid Focusa creates exactly one checkout intent');
expect_matrix($intent['intent']['product_code'] === $PRODUCT && $intent['intent']['customer_id'] === $focusaCustomer, 'intent binds the exact offer and promoted customer');
expect_matrix($intent['intent']['price']['amount_usd'] === '697.00', 'intent price is the server-owned registry price (no caller amount)');
expect_matrix(str_starts_with((string) $intent['intent']['branded_checkout_url'], 'https://install.focusa.dev/activate/checkout?intent='), 'branded facade checkout URL returned');
expect_matrix($registrations->findByUuid($regFocusa['registration_uuid'])['state'] === 'checkout_pending', 'registration advances to checkout_pending');
$orderFixture = $cart->projectOrderFixture($intent['intent']['cart_reference']);
expect_matrix($orderFixture['items'][0]['download_id'] === $DOWNLOAD && $orderFixture['items'][0]['price_id'] === $PRICE, 'cart order fixture uses the exact server-owned price relationship');
expect_matrix($orderFixture['order']['email'] === null, 'order fixture never carries a raw email');

// Identity hold: matching verified checkout email proceeds.
$assessPass = $integrityFixture->assessOrder([
    'order_id' => 5001, 'order_status' => 'complete', 'customer_id' => $focusaCustomer,
    'order_email' => 'commerce.focusa@example.invalid',
    'order_items' => [['download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
    'facade_id' => $FACADE, 'origin' => $ORIGIN, 'registration_uuid' => $regFocusa['registration_uuid'],
    'request_id' => 'req-integrity-focusa-1', 'idempotency_key' => 'idem-integrity-focusa-1',
]);
expect_matrix($assessPass['decision'] === 'email_integrity_passed' && $assessPass['entitlement_allowed'] === true, 'matching checkout email passes the identity hold');
expect_matrix($assessPass['email_matches_verified_identity'] === true && $assessPass['issuance'] === 'deferred_to_verified_issuance_service', 'identity hold defers issuance to the verified service');

// Order binding: canonical complete order + linked payment -> exactly one issuance request.
$insertOrder(5001, 'complete', $focusaCustomer, 'commerce.focusa@example.invalid', [
    ['item_id' => 5001, 'download' => $DOWNLOAD],
]);
$insertTransaction(5001, 'txn_commerce_0001');
$bound = $bind(5001, $regFocusa['registration_uuid'], $focusaCustomer, [['item_id' => 5001, 'download' => $DOWNLOAD]], 'txn_commerce_0001', 'focusa-1');
expect_matrix($bound['decision'] === 'order_bound' && $bound['entitlement_allowed'] === true, 'canonical complete order binds to the registration');
expect_matrix($bound['issuance_requests_settled'] === 1 && $bound['payment_bound'] === true, 'one eligible item settles exactly one issuance request with the linked payment');
$handleFocusa = $bound['protected_items'][0]['issuance_request_handle'];
expect_matrix(preg_match('/^(ir_)[0-9a-f]{32}$/D', (string) $handleFocusa) === 1, 'issuance request handle is an opaque bounded token');

// EDD Software Licensing issuance: exactly one canonical key, one license row.
$issued = $issue($handleFocusa, 'req-issue-focusa-1', 'idem-issue-focusa-1');
expect_matrix($issued['decision'] === 'license_issued' && $issued['keys_created'] === 1, 'canonical EDD SL issuance creates exactly one key');
expect_matrix($issued['issuance'] === 'canonical_edd_software_licensing', 'issuance is canonical EDD Software Licensing');
expect_matrix($issued['customer_id'] === $focusaCustomer && $issued['order_id'] === 5001 && $issued['order_item_id'] === 5001, 'license links the one customer/order/item');
expect_matrix($issued['product_code'] === $PRODUCT && $issued['license_type_ref'] === $PRODUCT, 'license carries the server-owned product and license type');
expect_matrix(preg_match($KEY_PATTERN, (string) $issued['delivery']['license_key']) === 1, 'delivered key is a canonical EDD SL key');
expect_matrix(str_starts_with((string) $issued['delivery']['license_key'], 'focusa_live_') === false, 'the adapter never issues a synthetic install-site key');
$paidKey = (string) $issued['delivery']['license_key'];
expect_matrix(hash('sha256', "focusa.spec152e.edd_license_issuance.key.v1\0" . $paidKey) === $issued['license_key_digest'], 'journal stores only the keyed digest of the canonical key');
$licenseRowFocusa = $db->query('SELECT * FROM wp_edd_licenses WHERE order_id = 5001')->fetch(PDO::FETCH_ASSOC);
expect_matrix($licenseRowFocusa !== false && (int) $licenseRowFocusa['activation_limit'] === 3, 'node limit comes from the server-owned offer (3 shared operator nodes)');
expect_matrix($registrations->findByUuid($regFocusa['registration_uuid'])['state'] === 'entitlement_issued', 'registration fulfills to entitlement_issued');

// Lifecycle: completion projects active/allowed with a strictly monotonic sequence.
$projected = $projector->projectOrder([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'status' => 'completed', 'request_id' => 'req-lifecycle-focusa-1', 'idempotency_key' => 'idem-lifecycle-focusa-1',
]);
expect_matrix($projected['decision'] === 'applied' && $projected['license_state'] === 'active' && $projected['refresh_posture'] === 'allowed', 'order completion projects active/allowed');
expect_matrix($projected['sequence'] === 0 && $projected['result_sequence'] === 1, 'completion bumps the account sequence 0 -> 1');
expect_matrix($accountSequence($focusaAccount) === 1, 'account sequence is 1 after completion');

// Outbox: the completion event is appended in the SAME transaction as canonical truth.
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET date_completed = date_completed WHERE id = 5001')->execute();
$eComplete = $appendEdd([
    'surface' => 'order', 'status' => 'completed', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5001,
    'request_id' => 'req-outbox-focusa-1', 'idempotency_key' => 'idem-outbox-focusa-1',
]);
$db->commit();
expect_matrix(table_count($db, 'wp_wpuiai_authority_outbox') === $baseline['outbox'] + 1, 'committed completion carries exactly one signed outbox event');
expect_matrix((int) $eComplete['authority_sequence'] === 1, 'outbox event snapshots the canonical sequence at append time');
// Atomic rollback: a crashed transaction loses both the canonical change and its event.
$beforeCrash = table_count($db, 'wp_wpuiai_authority_outbox');
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = 5001')->execute([':status' => 'failed']);
$appendEdd([
    'surface' => 'order', 'status' => 'failed', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5001,
    'request_id' => 'req-outbox-crash-1', 'idempotency_key' => 'idem-outbox-crash-1',
]);
$db->rollBack();
expect_matrix(table_count($db, 'wp_wpuiai_authority_outbox') === $beforeCrash, 'injected crash loses no committed event and creates no orphan');
expect_matrix((string) $db->query('SELECT status FROM wp_edd_orders WHERE id = 5001')->fetchColumn() === 'complete', 'canonical order truth survives the rollback');
// Dispatch: exactly once.
$summary = $dispatcher->dispatchReady();
expect_matrix(($summary['dispatched'] ?? 0) === 1, 'completion event dispatches exactly once');
expect_matrix(table_count($db, 'wp_wpuiai_outbox_deliveries') === $baseline['deliveries'] + 1 && table_count($db, 'wp_outbox_test_applications') === $baseline['applications'] + 1, 'one delivery ledger row and one consumer application');
$summary = $dispatcher->dispatchReady();
expect_matrix(($summary['dispatched'] ?? 0) === 0 && ($summary['failed'] ?? 0) === 0, 'dispatch replay delivers nothing a second time');

// ── 2. Paid UIAI: explicit UIAI grants only ────────────────────────────

$regUiai = $createRegistration('commerce.uiai@example.invalid', $FACADE, $UIAPRODUCT, 'uiai');
$uiaiCustomer = $regUiai['edd_customer_id'];
$tokenUiai = $issueToken($regUiai['registration_uuid'], $FACADE, $UIAPRODUCT, 'uiai');
$uiaiCart = $gateFixture->gateAddToCart([
    'download_id' => $UIADOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $UIAPRODUCT, 'registration_uuid' => $regUiai['registration_uuid'],
    'verified_token' => $tokenUiai['registration_token'],
    'request_id' => 'req-cart-uiai-1', 'idempotency_key' => 'idem-cart-uiai-1',
]);
expect_matrix($uiaiCart['decision'] === 'cart_gate_passed' && $uiaiCart['product_code'] === $UIAPRODUCT, 'paid UIAI passes the product gate with the explicit UIAI mapping');
$uiaiIntent = $checkoutFixture->createIntent([
    'registration_uuid' => $regUiai['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-intent-uiai-1',
    'idempotency_key' => 'idem-intent-uiai-1',
]);
expect_matrix($uiaiIntent['intent']['product_code'] === $UIAPRODUCT && $uiaiIntent['intent']['price']['amount_usd'] === '697.00', 'UIAI checkout intent binds the explicit UIAI offer and server price');
$insertOrder(5010, 'complete', $uiaiCustomer, 'commerce.uiai@example.invalid', [
    ['item_id' => 5010, 'download' => $UIADOWNLOAD],
]);
$insertTransaction(5010, 'txn_commerce_0010');
$boundUiai = $bind(5010, $regUiai['registration_uuid'], $uiaiCustomer, [['item_id' => 5010, 'download' => $UIADOWNLOAD]], 'txn_commerce_0010', 'uiai-1', $UIAIPRICE);
expect_matrix($boundUiai['decision'] === 'order_bound' && $boundUiai['issuance_requests_settled'] === 1, 'UIAI order binds with the explicit UIAI offer');
$issuedUiai = $issue($boundUiai['protected_items'][0]['issuance_request_handle'], 'req-issue-uiai-1', 'idem-issue-uiai-1');
expect_matrix($issuedUiai['license_type_ref'] === $UIAPRODUCT, 'UIAI issuance grants the explicit UIAI license type only');
expect_matrix($issuedUiai['keys_created'] === 1 && preg_match($KEY_PATTERN, (string) $issuedUiai['delivery']['license_key']) === 1, 'UIAI issuance creates one canonical UIAI key');

// ── 3. Bundle: explicit Focusa + UIAI grants in one account flow ───────

$regBundle = $createRegistration('commerce.bundle@example.invalid', $FACADE, $BUNDLE, 'bundle');
$bundleCustomer = $regBundle['edd_customer_id'];
$tokenBundle = $issueToken($regBundle['registration_uuid'], $FACADE, $BUNDLE, 'bundle');
$bundleCart = $gateFixture->gateAddToCart([
    'download_id' => $BUNDLEDOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => $BUNDLE, 'registration_uuid' => $regBundle['registration_uuid'],
    'verified_token' => $tokenBundle['registration_token'],
    'request_id' => 'req-cart-bundle-1', 'idempotency_key' => 'idem-cart-bundle-1',
]);
expect_matrix($bundleCart['decision'] === 'cart_gate_passed' && $bundleCart['product_code'] === $BUNDLE, 'bundle passes the product gate in one account flow');
$bundleIntent = $checkoutFixture->createIntent([
    'registration_uuid' => $regBundle['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-intent-bundle-1',
    'idempotency_key' => 'idem-intent-bundle-1',
]);
expect_matrix($bundleIntent['intent']['product_code'] === $BUNDLE && $bundleIntent['intent']['price']['amount_usd'] === '1254.60', 'bundle checkout intent binds the server-owned bundle offer and price');
$insertOrder(5020, 'complete', $bundleCustomer, 'commerce.bundle@example.invalid', [
    ['item_id' => 5020, 'download' => $BUNDLEDOWNLOAD],
]);
$insertTransaction(5020, 'txn_commerce_0020', 'complete', '1254.60');
$boundBundle = $bind(5020, $regBundle['registration_uuid'], $bundleCustomer, [['item_id' => 5020, 'download' => $BUNDLEDOWNLOAD]], 'txn_commerce_0020', 'bundle-1', $BUNDLEPRICE);
expect_matrix($boundBundle['decision'] === 'order_bound' && $boundBundle['issuance_requests_settled'] === 1, 'bundle order settles in the one account flow');
// The composite bundle is an exact_union: it defines explicit Focusa + UIAI grants and
// NO single license_type_ref, so a composite key can never masquerade as an implicit
// component grant (no invented focusa/uiai type). Component keys issue only through
// their own explicit offers (proven in the paid Focusa and paid UIAI rows).
$bundleIssued = $issue($boundBundle['protected_items'][0]['issuance_request_handle'], 'req-issue-bundle-1', 'idem-issue-bundle-1');
expect_matrix($bundleIssued['keys_created'] === 1, 'bundle item issues exactly one canonical composite key');
expect_matrix(($bundleIssued['license_type_ref'] ?? '') === '', 'composite bundle carries no implicit single license type (exact_union)');
expect_matrix(in_array((string) ($bundleIssued['license_type_ref'] ?? ''), [$PRODUCT, $UIAPRODUCT], true) === false, 'bundle issuance never masquerades as a component product');
expect_matrix($bundleIssued['product_code'] === $BUNDLE, 'bundle key binds the explicit composite product code');
expect_matrix((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses WHERE order_id = 5020')->fetchColumn() === 1, 'one composite license row for the one bundle order');

// ── 4. Evaluation: verified account + eligibility, no EDD key ─────────

$matrix = FocusaSpec152eEvaluationEligibilityState::matrix();
$byCase = [];
foreach ($matrix as $row) {
    $byCase[(string) $row['case']] = $row;
}
expect_matrix($byCase['verified_eligible']['decision'] === 'limited_access_issued', 'eligibility matrix: verified eligible issues limited access');
expect_matrix($byCase['active_paid_customer']['decision'] === 'paid_posture_preserved', 'eligibility matrix: active paid posture is preserved');
expect_matrix($byCase['prior_evaluation_duplicate']['decision'] === 'evaluation_not_eligible', 'eligibility matrix: prior evaluation denies duplicates');
$mapping = FocusaSpec152eEvaluationProductMapping::resolve(['product_code' => 'focusa_evaluation']);
expect_matrix($mapping['resolved_posture'] === 'verified_no_license' && $mapping['creates_edd_license_key'] === false, 'evaluation maps to verified_no_license with no EDD key');
expect_matrix($mapping['edd_download_id'] === null && $mapping['edd_price_id'] === null, 'evaluation carries no dedicated EDD download or price');

$regEval = $createRegistration('commerce.eval@example.invalid', $FACADE, 'focusa_evaluation', 'eval');
$evalInput = static function (array $reg, string $node, string $tag, int $counter) use ($EVAL_SIGNATURE): array {
    return [
        'product_code' => 'focusa_evaluation',
        'registration_uuid' => $reg['registration_uuid'],
        'account_uuid' => $reg['account_uuid'],
        'identity_uuid' => $reg['identity_uuid'],
        'verification_state' => 'account_promoted',
        'verified_at' => $reg['verified_at'],
        'node_uuid' => $node,
        'node_digest' => hash('sha256', 'node-' . $node),
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.edd.commerce.acceptance.test',
        'install_channel' => 'cli',
        'request_id' => 'req-eval-' . $tag . '-' . $counter,
        'idempotency_key' => 'idem-eval-' . $tag . '-' . $counter,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $EVAL_SIGNATURE,
        'issued_at' => '2026-08-08T00:05:00Z',
        'refresh_at' => '2026-08-08T00:05:00Z',
        'migration_provenance' => ['source' => 'edd_commerce_acceptance_test', 'record' => $tag . '-' . $counter],
    ];
};
$evalResult = $evaluation->requestEvaluation($evalInput($regEval, 'node-eval-0001', 'eval', 1));
expect_matrix($evalResult['decision'] === 'limited_access_issued', 'eligible verified account receives limited access');
expect_matrix($evalResult['posture_uuid'] !== null && $evalResult['assertion_uuid'] !== null, 'evaluation issues one posture and one signed assertion');
expect_matrix($evalResult['edd_order_id'] === null && $evalResult['edd_license_id'] === null, 'evaluation creates no EDD order and no EDD license key');
expect_matrix($evalResult['duration'] === 'no_automatic_expiry', 'evaluation resolves to the no-automatic-expiry posture');
expect_matrix(table_count($db, 'wp_wpuiai_verified_access_postures') === $baseline['postures'] + 1, 'exactly one evaluation posture created');
$licensesBeforeEval = table_count($db, 'wp_edd_licenses');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeEval, 'evaluation never touches the EDD license table');
// Prior evaluation duplicate (facade-switched node) fails closed, no second posture.
expect_matrix_throws(
    fn() => $evaluation->requestEvaluation($evalInput($regEval, 'node-eval-0002', 'eval-dup', 2)),
    'EVALUATION_NOT_ELIGIBLE',
    'prior evaluation duplicate is denied with zero new posture',
);
// Paid customer requests Evaluation: paid posture preserved, never downgraded.
expect_matrix_throws(
    fn() => $evaluation->requestEvaluation($evalInput($regFocusa, 'node-paid-eval-0001', 'paid-eval', 3)),
    'PAID_POSTURE_PRESERVED',
    'active paid customer is never downgraded through the Evaluation path',
);
expect_matrix($licenseRowFocusa['status'] === 'active', 'the paid license stays active after the Evaluation request');

// ── 5. Wrong product: no cross-product lease, no downgrade ────────────

$regWrong = $createRegistration('commerce.wrong@example.invalid', $FACADE, $PRODUCT, 'wrong');
$tokenWrong = $issueToken($regWrong['registration_uuid'], $FACADE, $PRODUCT, 'claim');
expect_matrix_throws(
    fn() => $gateFixture->gateAddToCart([
        'download_id' => $UIADOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regWrong['registration_uuid'],
        'verified_token' => $tokenWrong['registration_token'],
        'request_id' => 'req-cart-cross-1', 'idempotency_key' => 'idem-cart-cross-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'a Focusa-bound registration cannot open a UIAI download (no cross-product cart)',
);
$regWrong2 = $createRegistration('commerce.wrong2@example.invalid', $FACADE, $PRODUCT, 'wrong2');
$insertOrder(5032, 'complete', $regWrong2['edd_customer_id'], 'commerce.wrong2@example.invalid', [
    ['item_id' => 5032, 'download' => $UIADOWNLOAD],
]);
$insertTransaction(5032, 'txn_commerce_0032');
expect_matrix_throws(
    fn() => $bindingFixture->bindOrderComplete([
        'order_id' => 5032, 'order_status' => 'complete', 'customer_id' => $regWrong2['edd_customer_id'],
        'order_items' => [['order_item_id' => 5032, 'download_id' => $UIADOWNLOAD, 'price_id' => $UIAIPRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_commerce_0032', 'status' => 'complete']],
        'registration_uuid' => $regWrong2['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-cross-1', 'idempotency_key' => 'idem-bind-cross-1',
    ]),
    'FACADE_PRODUCT_DENIED',
    'a Focusa registration cannot settle a UIAI order item',
);
expect_matrix_throws(
    fn() => $evaluation->requestEvaluation($evalInput($regFocusa, 'node-paid-eval-0002', 'paid-eval-2', 4)),
    'PAID_POSTURE_PRESERVED',
    'no cross-product lease or downgrade for the paid customer',
);

// ── 6. Arbitrary amount / grants: caller commerce fields are impossible ─

$forbiddenAtGate = ['price' => '1.00', 'grants' => [$PRODUCT], 'features' => ['focusa.core.mission'], 'limits' => ['nodes' => 99], 'node_limit' => 99, 'edd_price_id' => $PRICE];
foreach ($forbiddenAtGate as $field => $value) {
    expect_matrix_throws(
        fn() => $gateFixture->gateAddToCart([
            'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
            'product_code' => $PRODUCT, 'registration_uuid' => $regFocusa['registration_uuid'],
            'verified_token' => '', 'request_id' => 'req-cart-' . $field . '-1', 'idempotency_key' => 'idem-cart-' . $field . '-1',
            $field => $value,
        ]),
        'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
        "product gate rejects caller-controlled {$field}",
    );
}
$forbiddenAtIntent = ['price' => '1.00', 'amount' => '1.00', 'grants' => [$PRODUCT], 'node_limit' => 99, 'edd_download_id' => $DOWNLOAD];
foreach ($forbiddenAtIntent as $field => $value) {
    expect_matrix_throws(
        fn() => $checkoutFixture->createIntent([
            'registration_uuid' => $regFocusa['registration_uuid'],
            'facade_id' => $FACADE,
            'origin' => $ORIGIN,
            'return_handle' => 'success',
            'request_id' => 'req-intent-' . $field . '-1',
            'idempotency_key' => 'idem-intent-' . $field . '-1',
            $field => $value,
        ]),
        'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
        "checkout intent rejects caller-controlled {$field}",
    );
}
expect_matrix_throws(
    fn() => $checkoutFixture->createIntent([
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'callback_url' => 'https://evil.example/hook',
        'request_id' => 'req-intent-redirect-1',
        'idempotency_key' => 'idem-intent-redirect-1',
    ]),
    'FACADE_REDIRECT_DENIED',
    'caller-supplied redirect targets are rejected',
);
expect_matrix_throws(
    fn() => $bindingFixture->bindOrderComplete([
        'order_id' => 5001, 'order_status' => 'complete', 'customer_id' => $focusaCustomer,
        'order_items' => [['order_item_id' => 5001, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_commerce_0001', 'status' => 'complete']],
        'registration_uuid' => $regFocusa['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-price-1', 'idempotency_key' => 'idem-bind-price-1',
        'price' => '0.01',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'order binding rejects caller-controlled price',
);
expect_matrix_throws(
    fn() => $issuanceFixture->issue([
        'issuance_request_handle' => $handleFocusa,
        'request_id' => 'req-issue-forbid-1',
        'idempotency_key' => 'idem-issue-forbid-1',
        'price' => '1.00',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'issuance rejects caller-controlled price',
);
$issuedRetry = $issue($handleFocusa, 'req-issue-retry-1', 'idem-issue-retry-1');
expect_matrix($issuedRetry['existing'] === true && $issuedRetry['keys_created'] === 0 && $issuedRetry['delivery']['license_key'] === $paidKey, 'delivery retry returns the identical canonical key with zero keys created');
expect_matrix_throws(
    fn() => $projector->projectOrder([
        'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
        'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
        'status' => 'completed', 'price' => 9.99,
        'request_id' => 'req-lifecycle-price-1', 'idempotency_key' => 'idem-lifecycle-price-1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'lifecycle projector rejects caller-controlled price',
);
expect_matrix_throws(
    fn() => $projector->projectOrder([
        'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
        'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
        'status' => 'completed', 'grants' => ['release' => true],
        'request_id' => 'req-lifecycle-grants-1', 'idempotency_key' => 'idem-lifecycle-grants-1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'lifecycle projector rejects caller-controlled grants',
);
expect_hook_throws(
    fn() => $appendEdd([
        'surface' => 'order', 'status' => 'completed', 'account_uuid' => $focusaAccount,
        'edd_customer_id' => $focusaCustomer, 'order_id' => 5001,
        'price' => 9.99,
        'request_id' => 'req-outbox-price-1', 'idempotency_key' => 'idem-outbox-price-1',
    ]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'outbox append rejects caller-controlled price inside the caller transaction',
);
expect_matrix_throws(
    fn() => $evaluation->requestEvaluation($evalInput($regEval, 'node-eval-map-0001', 'eval-map', 5) + ['edd_download_id' => 453]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'evaluation rejects caller-controlled EDD mapping',
);

// ── 7. Unrelated download: proven non-entitlement, zero licenses ───────

$unrelatedGate = $gateFixture->gateAddToCart([
    'download_id' => 16, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'product_code' => '', 'registration_uuid' => '', 'verified_token' => '',
    'request_id' => 'req-cart-unrelated-1', 'idempotency_key' => 'idem-cart-unrelated-1',
]);
expect_matrix($unrelatedGate['decision'] === 'non_entitlement_allowed' && $unrelatedGate['entitlement_allowed'] === false, 'unrelated download never marks entitlement at the gate');
$unrelatedAssess = $integrityFixture->assessOrder([
    'order_id' => 5030, 'order_status' => 'complete', 'customer_id' => 9001,
    'order_email' => 'commerce.shopper@example.invalid',
    'order_items' => [['download_id' => 16, 'price_id' => 'price_unrelated', 'quantity' => 1]],
    'facade_id' => $FACADE, 'origin' => $ORIGIN, 'registration_uuid' => '',
    'request_id' => 'req-integrity-unrelated-1', 'idempotency_key' => 'idem-integrity-unrelated-1',
]);
expect_matrix($unrelatedAssess['decision'] === 'no_entitlement' && $unrelatedAssess['issuance'] === 'none', 'unrelated order is proven non-entitlement at the identity hold');
$insertOrder(5030, 'complete', 9001, 'commerce.shopper@example.invalid', [
    ['item_id' => 5030, 'download' => 16],
]);
$unrelatedBind = $bindingFixture->bindOrderComplete([
    'order_id' => 5030, 'order_status' => 'complete', 'customer_id' => 9001,
    'order_items' => [['order_item_id' => 5030, 'download_id' => 16, 'price_id' => 'price_unrelated', 'quantity' => 1]],
    'payment_transactions' => [],
    'registration_uuid' => '',
    'facade_id' => $FACADE, 'origin' => $ORIGIN,
    'request_id' => 'req-bind-unrelated-1', 'idempotency_key' => 'idem-bind-unrelated-1',
]);
expect_matrix($unrelatedBind['decision'] === 'no_entitlement' && $unrelatedBind['protected_items'] === 0, 'unrelated order settles zero protected items at the binding');
$licensesBeforeUnrelated = table_count($db, 'wp_edd_licenses');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeUnrelated, 'unrelated order creates zero licenses');
// Frozen registry scan: no catalog download can reach Focusa entitlement.
$frozenScanDenied = true;
foreach ($frozenRegistry['current_edd_catalog']['entries'] as $entry) {
    $download = (int) $entry['download_id'];
    try {
        $outcome = $gateFrozen->gateAddToCart([
            'download_id' => $download, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
            'product_code' => '', 'registration_uuid' => '', 'verified_token' => '',
            'request_id' => 'req-scan-' . $download, 'idempotency_key' => 'idem-scan-' . $download,
        ]);
        if (($outcome['decision'] ?? '') !== 'non_entitlement_allowed' || $outcome['entitlement_allowed'] !== false) {
            $frozenScanDenied = false;
        }
    } catch (DomainException $error) {
        if ($error->getMessage() !== 'PRODUCT_MAPPING_REQUIRED') {
            $frozenScanDenied = false;
        }
    }
}
expect_matrix($frozenScanDenied, 'no catalog download reaches Focusa entitlement through the single authority');

// ── 8. Duplicate / replayed order: one account/order/license result ───

$regDup = $createRegistration('commerce.dup@example.invalid', $FACADE, $PRODUCT, 'dup');
$dupCustomer = $regDup['edd_customer_id'];
$dupIntent = $checkoutFixture->createIntent([
    'registration_uuid' => $regDup['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-intent-dup-1',
    'idempotency_key' => 'idem-intent-dup-1',
]);
expect_matrix($dupIntent['replayed'] === false, 'duplicate-scenario registration creates one checkout intent');
$insertOrder(5050, 'complete', $dupCustomer, 'commerce.dup@example.invalid', [
    ['item_id' => 5050, 'download' => $DOWNLOAD],
]);
$insertTransaction(5050, 'txn_commerce_0050');
$boundDup = $bind(5050, $regDup['registration_uuid'], $dupCustomer, [['item_id' => 5050, 'download' => $DOWNLOAD]], 'txn_commerce_0050', 'dup-1');
expect_matrix($boundDup['decision'] === 'order_bound' && $boundDup['existing'] === false && $boundDup['issuance_requests_settled'] === 1, 'first canonical completion settles exactly one issuance request');
$replayedDup = $bind(5050, $regDup['registration_uuid'], $dupCustomer, [['item_id' => 5050, 'download' => $DOWNLOAD]], 'txn_commerce_0050', 'dup-1');
expect_matrix(json_encode($replayedDup, JSON_THROW_ON_ERROR) === json_encode($boundDup, JSON_THROW_ON_ERROR), 'idempotency replay returns the byte-identical decision');
$duplicateDup = $bind(5050, $regDup['registration_uuid'], $dupCustomer, [['item_id' => 5050, 'download' => $DOWNLOAD]], 'txn_commerce_0050', 'dup-2');
expect_matrix($duplicateDup['decision'] === 'order_bound' && $duplicateDup['existing'] === true, 'duplicate completion event returns the existing settlement');
expect_matrix($duplicateDup['issuance_requests_settled'] === 0, 'duplicate event settles nothing new');
expect_matrix($duplicateDup['protected_items'][0]['issuance_request_handle'] === $boundDup['protected_items'][0]['issuance_request_handle'], 'duplicate event returns the same issuance request handle');
expect_matrix((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_edd_issuance_requests WHERE order_id = 5050')->fetchColumn() === 1, 'duplicate/replay creates exactly one issuance request for the one order');
$issuedDup = $issue($boundDup['protected_items'][0]['issuance_request_handle'], 'req-issue-dup-1', 'idem-issue-dup-1');
expect_matrix($issuedDup['decision'] === 'license_issued' && $issuedDup['keys_created'] === 1, 'one canonical license issued for the one order');
$issuedDupRetry = $issue($boundDup['protected_items'][0]['issuance_request_handle'], 'req-issue-dup-2', 'idem-issue-dup-2');
expect_matrix($issuedDupRetry['existing'] === true && $issuedDupRetry['keys_created'] === 0 && $issuedDupRetry['delivery']['license_key'] === $issuedDup['delivery']['license_key'], 'issuance retry returns the identical key with zero keys created');
expect_matrix((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses WHERE order_id = 5050')->fetchColumn() === 1, 'one canonical license row for the duplicate-order fixture');
$licensesFor5001 = (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses WHERE order_id = 5001')->fetchColumn();
expect_matrix($licensesFor5001 === 1, 'one canonical license for the one paid order');
$lifecycleReplay = $projector->projectOrder([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'status' => 'completed', 'request_id' => 'req-lifecycle-focusa-1', 'idempotency_key' => 'idem-lifecycle-focusa-1',
]);
expect_matrix($lifecycleReplay['decision'] === 'replayed' && $lifecycleReplay['event_uuid'] === $projected['event_uuid'], 'lifecycle replay is idempotent (same event, no new row)');
expect_matrix($accountSequence($focusaAccount) === 1, 'lifecycle replay never bumps the sequence');
$licensesBeforeDuplicate = table_count($db, 'wp_edd_licenses');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeDuplicate, 'duplicate/replay never duplicates the license');

// ── 9. Changed email: fulfillment held until a verified link review ────

$regChanged = $createRegistration('commerce.changed@example.invalid', $FACADE, $PRODUCT, 'changed');
$changedCustomer = $regChanged['edd_customer_id'];
$assessChanged = $integrityFixture->assessOrder([
    'order_id' => 5040, 'order_status' => 'complete', 'customer_id' => $changedCustomer,
    'order_email' => 'commerce.stranger@example.invalid',
    'order_items' => [['download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
    'facade_id' => $FACADE, 'origin' => $ORIGIN, 'registration_uuid' => $regChanged['registration_uuid'],
    'request_id' => 'req-integrity-changed-1', 'idempotency_key' => 'idem-integrity-changed-1',
]);
expect_matrix($assessChanged['decision'] === 'fulfillment_held', 'changed checkout email holds fulfillment');
expect_matrix($assessChanged['mismatch_kind'] === 'changed' && $assessChanged['error_code'] === 'ACCOUNT_EMAIL_MISMATCH', 'changed email fails with ACCOUNT_EMAIL_MISMATCH');
expect_matrix($assessChanged['entitlement_allowed'] === false && $assessChanged['issuance'] === 'held_until_email_verified', 'held order is not entitlement-allowed and never issues');
expect_matrix(str_starts_with((string) $assessChanged['hold_key'], 'fh_') && str_starts_with((string) $assessChanged['review_handle'], 'hr_'), 'hold and review handles are opaque bounded tokens');
$licensesBeforeChanged = table_count($db, 'wp_edd_licenses');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeChanged, 'changed email creates no license');
// Payment evidence alone can never release; only a separately verified link review can.
expect_matrix_throws(
    fn() => $integrityFixture->releaseHold([
        'hold_key' => (string) $assessChanged['hold_key'],
        'order_email' => 'commerce.stranger@example.invalid',
        'resolved_identity_uuid' => '55555555-5555-4555-8555-555555555555',
        'release_evidence_handle' => 'ev_payment_success_0000000000000000000000',
        'evidence_kind' => 'payment_success',
        'request_id' => 'req-release-payment-1',
        'idempotency_key' => 'idem-release-payment-1',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'payment success alone can never release the fulfillment hold',
);
expect_matrix($integrityFixture->findByHoldKey((string) $assessChanged['hold_key'])['hold_state'] === 'held', 'hold stays held after the payment-evidence attempt');
$resolvedIdentityUuid = 'aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee';
$identities->storeVerified('commerce.stranger@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'identity_uuid' => $resolvedIdentityUuid,
    'account_uuid' => $regChanged['account_uuid'],
    'identity_state' => 'linked',
    'verified_at' => '2026-08-08T00:02:00Z',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:02:00Z',
    'source' => 'candidate_contract',
    'migration_evidence' => ['record' => 'changed-email-verified-link'],
]);
$released = $integrityFixture->releaseHold([
    'hold_key' => (string) $assessChanged['hold_key'],
    'order_email' => 'commerce.stranger@example.invalid',
    'resolved_identity_uuid' => $resolvedIdentityUuid,
    'release_evidence_handle' => 'ev_5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d',
    'request_id' => 'req-release-verified-1',
    'idempotency_key' => 'idem-release-verified-1',
]);
expect_matrix($released['decision'] === 'fulfillment_released' && $released['entitlement_allowed'] === true, 'separately verified link review releases the hold');
expect_matrix($released['issuance'] === 'deferred_to_verified_issuance_service', 'release defers issuance; the hold surface never creates a key');
expect_matrix($integrityFixture->findByHoldKey((string) $assessChanged['hold_key'])['hold_state'] === 'released', 'hold is journaled as released');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeChanged, 'the changed-email order never issues a license (identity hold proof)');

// ── 10. Refund / revoke / expiry: sequence, recovery_only, no reactivation ─

// Refund on the paid order: canonical truth + lifecycle + outbox in lockstep.
$db->beginTransaction();
$db->prepare("UPDATE wp_edd_orders SET status = 'refunded' WHERE id = 5001")->execute();
$db->prepare("UPDATE wp_edd_licenses SET status = 'refunded' WHERE order_id = 5001")->execute();
$refundEvent = $appendEdd([
    'surface' => 'refund', 'status' => 'refunded', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'request_id' => 'req-outbox-refund-1', 'idempotency_key' => 'idem-outbox-refund-1',
]);
$db->commit();
expect_matrix((string) $refundEvent['event_type'] === 'refund_issued', 'refunded order appends a refund_issued signed event');
$refundProjection = $projector->projectRefund([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'status' => 'refunded', 'request_id' => 'req-lifecycle-refund-1', 'idempotency_key' => 'idem-lifecycle-refund-1',
    'state_reason' => 'edd_order_refunded',
]);
expect_matrix($refundProjection['decision'] === 'applied' && $refundProjection['license_state'] === 'refunded' && $refundProjection['refresh_posture'] === 'recovery_only', 'refund projects refunded/recovery_only');
expect_matrix($refundProjection['sequence'] === 1 && $refundProjection['result_sequence'] === 2, 'refund bumps the sequence 1 -> 2');
expect_matrix($accountSequence($focusaAccount) === 2, 'account sequence is 2 after the refund');
$historyBeforeRefund = [
    'customers' => table_count($db, 'wp_edd_customers'),
    'orders' => table_count($db, 'wp_edd_orders'),
    'licenses' => table_count($db, 'wp_edd_licenses'),
];
expect_matrix(table_count($db, 'wp_edd_customers') === $historyBeforeRefund['customers'], 'refund preserves customers');
expect_matrix(table_count($db, 'wp_edd_orders') === $historyBeforeRefund['orders'], 'refund preserves orders');
expect_matrix((string) $db->query('SELECT status FROM wp_edd_licenses WHERE order_id = 5001')->fetchColumn() === 'refunded', 'refunded license row is retained (refund truth preserved)');

// Revoke on a second order/license of the same customer.
$insertOrder(5002, 'completed', $focusaCustomer, 'commerce.focusa@example.invalid', [
    ['item_id' => 5002, 'download' => $DOWNLOAD],
]);
$insertTransaction(5002, 'txn_commerce_0002');
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, user_id, product_id, order_id, status, date_created)
    VALUES ('B1C2D3E4-F5A6-7890-ABCD-EF1234567890', {$focusaCustomer}, NULL, {$DOWNLOAD}, 5002, 'active', '2026-08-08T00:01:00Z')");
$licenseRow5002 = $db->query('SELECT id FROM wp_edd_licenses WHERE order_id = 5002')->fetchColumn();
$projector->projectOrder([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5002, 'license_id' => (int) $licenseRow5002,
    'status' => 'completed', 'request_id' => 'req-lifecycle-5002-complete', 'idempotency_key' => 'idem-lifecycle-5002-complete',
]);
$db->beginTransaction();
$db->prepare("UPDATE wp_edd_orders SET status = 'revoked' WHERE id = 5002")->execute();
$db->prepare("UPDATE wp_edd_licenses SET status = 'revoked' WHERE order_id = 5002")->execute();
$revokeEvent = $appendEdd([
    'surface' => 'license', 'status' => 'revoked', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5002, 'license_id' => (int) $licenseRow5002,
    'request_id' => 'req-outbox-revoke-1', 'idempotency_key' => 'idem-outbox-revoke-1',
]);
$db->commit();
expect_matrix((string) $revokeEvent['event_type'] === 'license_revoked', 'revoked license appends a license_revoked signed event');
$revokeProjection = $projector->projectLicense([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5002, 'license_id' => (int) $licenseRow5002,
    'from_status' => 'active', 'to_status' => 'revoked', 'request_id' => 'req-lifecycle-revoke-1', 'idempotency_key' => 'idem-lifecycle-revoke-1',
]);
expect_matrix($revokeProjection['decision'] === 'applied' && $revokeProjection['license_state'] === 'revoked' && $revokeProjection['refresh_posture'] === 'recovery_only', 'revoke projects revoked/recovery_only');
expect_matrix($accountSequence($focusaAccount) === 4, 'revoke bumps the sequence to 4 (complete + revoke)');
expect_matrix((string) $db->query('SELECT status FROM wp_edd_orders WHERE id = 5002')->fetchColumn() === 'revoked', 'revoked order truth retained');

// Expiry on a third order/license of the same customer.
$insertOrder(5003, 'completed', $focusaCustomer, 'commerce.focusa@example.invalid', [
    ['item_id' => 5003, 'download' => $DOWNLOAD],
]);
$insertTransaction(5003, 'txn_commerce_0003');
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, user_id, product_id, order_id, status, date_created)
    VALUES ('D4E5F6A7-B8C9-0123-4567-89ABCDEF0123', {$focusaCustomer}, NULL, {$DOWNLOAD}, 5003, 'active', '2026-08-08T00:01:00Z')");
$licenseRow5003 = $db->query('SELECT id FROM wp_edd_licenses WHERE order_id = 5003')->fetchColumn();
$projector->projectOrder([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5003, 'license_id' => (int) $licenseRow5003,
    'status' => 'completed', 'request_id' => 'req-lifecycle-5003-complete', 'idempotency_key' => 'idem-lifecycle-5003-complete',
]);
$db->beginTransaction();
$db->prepare("UPDATE wp_edd_licenses SET status = 'expired' WHERE order_id = 5003")->execute();
$expiryEvent = $appendEdd([
    'surface' => 'license', 'status' => 'expired', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5003, 'license_id' => (int) $licenseRow5003,
    'request_id' => 'req-outbox-expiry-1', 'idempotency_key' => 'idem-outbox-expiry-1',
]);
$db->commit();
expect_matrix((string) $expiryEvent['event_type'] === 'license_expired', 'expired license appends a license_expired signed event');
$expiryProjection = $projector->projectLicense([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5003, 'license_id' => (int) $licenseRow5003,
    'from_status' => 'active', 'to_status' => 'expired', 'request_id' => 'req-lifecycle-expiry-1', 'idempotency_key' => 'idem-lifecycle-expiry-1',
]);
expect_matrix($expiryProjection['decision'] === 'applied' && $expiryProjection['license_state'] === 'expired' && $expiryProjection['refresh_posture'] === 'recovery_only', 'expiry projects expired/recovery_only');
expect_matrix($accountSequence($focusaAccount) === 6, 'expiry bumps the sequence to 6');
// The three terminal signed events dispatch exactly once on the next cycle.
$terminalDispatch = $dispatcher->dispatchReady();
expect_matrix(($terminalDispatch['dispatched'] ?? 0) === 3, 'refund/revoke/expiry signed events dispatch exactly once');
$applicationsAfterTerminal = table_count($db, 'wp_outbox_test_applications');
// Terminal truth never reactivates.
$stale = $projector->projectOrder([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'status' => 'completed', 'request_id' => 'req-lifecycle-stale-1', 'idempotency_key' => 'idem-lifecycle-stale-1',
]);
expect_matrix_denied($stale, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'stale completion after refund can never reactivate the license');
$dispute = $projector->projectStripe([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5001, 'license_id' => (int) $licenseRowFocusa['id'],
    'status' => 'won', 'request_id' => 'req-lifecycle-dispute-1', 'idempotency_key' => 'idem-lifecycle-dispute-1',
]);
expect_matrix_denied($dispute, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'dispute-won webhook cannot reactivate the refunded license');
$reactivate = $projector->projectLicense([
    'account_uuid' => $focusaAccount, 'edd_customer_id' => $focusaCustomer,
    'order_id' => 5003, 'license_id' => (int) $licenseRow5003,
    'from_status' => 'expired', 'to_status' => 'active', 'request_id' => 'req-lifecycle-reactivate-1', 'idempotency_key' => 'idem-lifecycle-reactivate-1',
]);
expect_matrix_denied($reactivate, 'LICENSE_TERMINAL_REACTIVATION_DENIED', 'expired license cannot reactivate');
expect_matrix($accountSequence($focusaAccount) === 6, 'denied reactivations never bump the sequence');

// ── 11. Authority outage: no new local license; signed offline policy only ─

$regOutage = $createRegistration('commerce.outage@example.invalid', $FACADE, $PRODUCT, 'outage');
$tokenOutage = $issueToken($regOutage['registration_uuid'], $FACADE, $PRODUCT, 'outage');
expect_matrix_throws(
    fn() => $gateFrozen->gateAddToCart([
        'download_id' => $DOWNLOAD, 'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'product_code' => $PRODUCT, 'registration_uuid' => $regOutage['registration_uuid'],
        'verified_token' => $tokenOutage['registration_token'],
        'request_id' => 'req-cart-outage-1', 'idempotency_key' => 'idem-cart-outage-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'authority outage (frozen registry) fails closed at the product gate',
);
expect_matrix_throws(
    fn() => $checkoutFrozen->createIntent([
        'registration_uuid' => $regOutage['registration_uuid'],
        'facade_id' => $FACADE,
        'origin' => $ORIGIN,
        'return_handle' => 'success',
        'request_id' => 'req-intent-outage-1',
        'idempotency_key' => 'idem-intent-outage-1',
    ]),
    'EDD_CHECKOUT_REQUIRED',
    'authority outage fails closed at checkout intent',
);
$insertOrder(5041, 'complete', $regOutage['edd_customer_id'], 'commerce.outage@example.invalid', [
    ['item_id' => 5041, 'download' => $DOWNLOAD],
]);
$insertTransaction(5041, 'txn_commerce_0041');
expect_matrix_throws(
    fn() => $bindingFrozen->bindOrderComplete([
        'order_id' => 5041, 'order_status' => 'complete', 'customer_id' => $regOutage['edd_customer_id'],
        'order_items' => [['order_item_id' => 5041, 'download_id' => $DOWNLOAD, 'price_id' => $PRICE, 'quantity' => 1]],
        'payment_transactions' => [['gateway' => 'stripe', 'transaction_id' => 'txn_commerce_0041', 'status' => 'complete']],
        'registration_uuid' => $regOutage['registration_uuid'],
        'facade_id' => $FACADE, 'origin' => $ORIGIN,
        'request_id' => 'req-bind-outage-1', 'idempotency_key' => 'idem-bind-outage-1',
    ]),
    'PRODUCT_MAPPING_REQUIRED',
    'authority outage fails closed at order binding',
);
$licensesBeforeOutage = table_count($db, 'wp_edd_licenses');
expect_matrix(table_count($db, 'wp_edd_licenses') === $licensesBeforeOutage, 'authority outage creates no new local license');
// Existing signed offline policy stays valid and replayable.
$outagePosture = $db->query("SELECT posture_uuid, status, sequence, signer FROM wp_wpuiai_verified_access_postures WHERE account_uuid = '{$regEval['account_uuid']}'")->fetch(PDO::FETCH_ASSOC);
expect_matrix($outagePosture !== false && $outagePosture['status'] === 'issued', 'existing signed offline posture survives the outage untouched');
expect_matrix($db->query("SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions WHERE posture_uuid = '{$outagePosture['posture_uuid']}'")->fetchColumn() == 1, 'the signed assertion is replayable during the outage');
expect_matrix($accountSequence($focusaAccount) === 6, 'outage leaves the authority sequence untouched');
// Consumer outage: bounded retries, dead letter, repair; canonical commit never blocked.
$consumerMode = 'normal';
$applicationsBeforeOutageDispatch = table_count($db, 'wp_outbox_test_applications');
$db->beginTransaction();
$db->prepare('UPDATE wp_edd_orders SET status = :status WHERE id = 5003')->execute([':status' => 'failed']);
$outageEvent = $appendEdd([
    'surface' => 'order', 'status' => 'failed', 'account_uuid' => $focusaAccount,
    'edd_customer_id' => $focusaCustomer, 'order_id' => 5003,
    'request_id' => 'req-outbox-outage-1', 'idempotency_key' => 'idem-outbox-outage-1',
]);
$db->commit();
$consumerMode = 'consumer_down';
$summary = $dispatcher->dispatchReady();
expect_matrix(($summary['failed'] ?? 0) === 1, 'consumer outage records a bounded dispatch failure');
$rowOutage = $dispatcher->findByEventUuid((string) $outageEvent['event_uuid']);
expect_matrix($rowOutage['dispatch_state'] === 'failed' && (int) $rowOutage['attempts'] === 1 && $rowOutage['last_error'] === 'DELIVERY_CONSUMER_DOWN', 'consumer outage exposes bounded durable failure state');
$tick(60);
$dispatcher->dispatchReady();
$tick(120);
$summary = $dispatcher->dispatchReady();
expect_matrix(($summary['dead_lettered'] ?? 0) === 1, 'attempt budget exhausted moves the event to dead letter');
expect_matrix((string) $db->query('SELECT status FROM wp_edd_orders WHERE id = 5003')->fetchColumn() === 'failed', 'dispatch failure never blocks or reverts the canonical EDD commit');
$repair = $dispatcher->retryDeadLetter([(string) $outageEvent['event_uuid']]);
expect_matrix($repair === 1, 'bounded repair re-queues the dead-lettered event');
$consumerMode = 'normal';
$summary = $dispatcher->dispatchReady();
expect_matrix(($summary['dispatched'] ?? 0) === 1, 'repaired event dispatches exactly once on the next cycle');
expect_matrix(table_count($db, 'wp_outbox_test_applications') === $applicationsBeforeOutageDispatch + 1, 'exactly-once consumer application across the consumer outage (no duplicate entitlement)');

// ── 12. Legacy install-site records: evidence-backed migration or quarantine ─

$legacyClasses = [];
foreach ($frozenRegistry['legacy_record_classes'] as $class) {
    $legacyClasses[(string) $class['id']] = (string) $class['disposition'];
}
expect_matrix($legacyClasses['install_stripe_active_focusa'] === 'migrate', 'evidence-backed install-site paid records migrate to EDD authority');
expect_matrix($legacyClasses['install_refunded_focusa'] === 'retire' && $legacyClasses['install_revoked_focusa'] === 'retire', 'terminal install-site records retire with refund/revoke history preserved');
expect_matrix($legacyClasses['install_api_active_focusa'] === 'quarantine', 'unverifiable install-site records quarantine for operator review');
// A synthetic focusa_live legacy key is preserved, never issued, and blocks new issuance.
$regSynthetic = $createRegistration('commerce.synthetic@example.invalid', $FACADE, $PRODUCT, 'synthetic');
$syntheticCustomer = $regSynthetic['edd_customer_id'];
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, user_id, product_id, order_id, status, date_created)
    VALUES ('focusa_live_1001_" . str_repeat('a', 16) . "', {$syntheticCustomer}, NULL, {$DOWNLOAD}, NULL, 'inactive', '2026-08-08T00:01:00Z')");
$insertOrder(5042, 'complete', $syntheticCustomer, 'commerce.synthetic@example.invalid', [
    ['item_id' => 5042, 'download' => $DOWNLOAD],
]);
$insertTransaction(5042, 'txn_commerce_0042');
$syntheticIntent = $checkoutFixture->createIntent([
    'registration_uuid' => $regSynthetic['registration_uuid'],
    'facade_id' => $FACADE,
    'origin' => $ORIGIN,
    'return_handle' => 'success',
    'request_id' => 'req-intent-synthetic-1',
    'idempotency_key' => 'idem-intent-synthetic-1',
]);
expect_matrix($syntheticIntent['replayed'] === false, 'synthetic-fixture registration enters checkout');
$boundSynthetic = $bind(5042, $regSynthetic['registration_uuid'], $syntheticCustomer, [['item_id' => 5042, 'download' => $DOWNLOAD]], 'txn_commerce_0042', 'synthetic-1');
expect_matrix($boundSynthetic['issuance_requests_settled'] === 1, 'inactive synthetic key does not block the binding settlement');
expect_matrix_throws(
    fn() => $issue($boundSynthetic['protected_items'][0]['issuance_request_handle'], 'req-issue-synthetic-1', 'idem-issue-synthetic-1'),
    'EDD_LICENSE_UNUSABLE',
    'canonical issuance is blocked next to a preserved synthetic install-site key',
);
expect_matrix((int) $db->query("SELECT COUNT(*) FROM wp_edd_licenses WHERE license_key LIKE 'focusa_live_%'")->fetchColumn() === 1, 'the synthetic install-site key is preserved, never deleted, never re-issued');

// ── 13. Reconciliation: missing callbacks repaired, ambiguity quarantined ─

// A canonical completed order with no projection/outbox yet (missed callback).
$insertOrder(5004, 'completed', $focusaCustomer, 'commerce.focusa@example.invalid', [
    ['item_id' => 5004, 'download' => $DOWNLOAD],
]);
$insertTransaction(5004, 'txn_commerce_0004');
$dryRun = $reconciler->run('dry_run', ['order', 'license']);
$repairable = $dryRun['summary']['repairable'];
expect_matrix($repairable >= 2, 'dry-run sees the missed-callback repair set (at least the unpiped completion projection + outbox)');
expect_matrix($dryRun['summary']['repairs_applied'] === 0 && $dryRun['summary']['would_repair'] === $repairable, 'dry-run applies nothing and reports the exact would-be repair set');
$applyRun = $reconciler->run('apply', ['order', 'license']);
expect_matrix($applyRun['summary']['repairs_applied'] === $repairable && $applyRun['summary']['converged'] === true, 'apply run repairs every missing callback and converges');
$secondApply = $reconciler->run('apply', ['order', 'license']);
expect_matrix($secondApply['summary']['repairs_applied'] === 0 && $secondApply['summary']['converged'] === true, 'repeated apply converges with zero new repairs');
$postDry = $reconciler->run('dry_run', ['order', 'license']);
expect_matrix($postDry['summary']['repairable'] === 0 && $postDry['summary']['would_repair'] === 0, 'post-apply dry-run reports zero would-be repairs (converged)');
expect_matrix($accountSequence($focusaAccount) === 8, 'reconciliation advanced the focusa sequence monotonically by exactly the two repaired focusa completions (5003 cancel + 5004 complete)');
// Full-surface run: the EDD-free evaluation posture has no signed lease event and
// quarantines for operator review (fail closed); the paid commerce surfaces converge.
$fullApply = $reconciler->run('apply');
expect_matrix($fullApply['summary']['converged'] === true, 'full-surface apply converges (commerce converged; ambiguity quarantined)');
$quarantinedLease = count(array_filter($fullApply['findings'], static fn (array $row): bool => ($row['classification'] ?? '') === 'quarantine_ambiguous' && ($row['entity_type'] ?? '') === 'lease'));
expect_matrix($quarantinedLease >= 1, 'the EDD-free posture without a signed lease event quarantines for operator review');
$fullSecond = $reconciler->run('apply');
expect_matrix($fullSecond['summary']['quarantined_new'] === 0 && $fullSecond['summary']['stable_quarantine'] >= 1, 'quarantine stays stable on repeated runs (nothing deleted, nothing duplicated)');
$handleA = $reconciler->run('dry_run', ['order'])['result_handle'];
$handleB = $reconciler->run('dry_run', ['order'])['result_handle'];
expect_matrix($handleA === $handleB, 'reconciliation result handle is deterministic across identical runs');
expect_matrix_throws(
    static fn () => $reconciler->run('apply', ['price' => 9.99]),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'reconciliation rejects caller-controlled commerce fields in scope',
);

// ── 14. Redaction, rollback preservation, bounded result handle ────────

$finalCounts = $counts();
$journalTables = [
    'wp_wpuiai_edd_gate_decisions' => ['result_payload'],
    'wp_wpuiai_edd_checkout_intents' => [],
    'wp_wpuiai_edd_checkout_cart_sessions' => [],
    'wp_wpuiai_checkout_email_integrity_holds' => [],
    'wp_wpuiai_checkout_email_integrity_releases' => [],
    'wp_wpuiai_edd_order_bindings' => [],
    'wp_wpuiai_edd_issuance_requests' => [],
    'wp_wpuiai_edd_license_issuances' => [],
    'wp_wpuiai_evaluation_issuances' => [],
    'wp_wpuiai_edd_lifecycle_events' => ['result_payload'],
    'wp_wpuiai_authority_outbox' => [],
    'wp_wpuiai_outbox_deliveries' => [],
    'wp_wpuiai_reconciliation_runs' => [],
    'wp_wpuiai_reconciliation_findings' => [],
    'wp_wpuiai_reconciliation_repairs' => [],
    'wp_wpuiai_reconciliation_quarantine' => [],
];
foreach ($journalTables as $table => $payloadColumns) {
    $rows = $db->query("SELECT * FROM {$table}")->fetchAll(PDO::FETCH_ASSOC);
    $json = json_encode($rows, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    expect_matrix(strpos($json, '@') === false, "no raw email in {$table}");
    expect_matrix(strpos($json, 'cus_') === false, "no customer/payment secret in {$table}");
    expect_matrix(strpos($json, 'txn_commerce_') === false, "no raw payment transaction id in {$table}");
    expect_matrix(preg_match($KEY_SCAN_PATTERN, $json) !== 1, "no full license key in {$table}");
    expect_matrix(strpos($json, $secret) === false, "server signing secret never leaves the signer ({$table})");
    foreach ($payloadColumns as $column) {
        foreach ($rows as $row) {
            expect_matrix(strpos((string) ($row[$column] ?? ''), '@') === false, "no raw email in {$table}.{$column}");
        }
    }
}
$summaryPayloads = $db->query('SELECT result_payload FROM wp_wpuiai_edd_gate_decisions')->fetchAll(PDO::FETCH_COLUMN);
foreach ($summaryPayloads as $payload) {
    expect_matrix(strpos((string) $payload, 'rg_') === false, 'no raw gate token in any gate decision');
}
$licenseRows = $db->query('SELECT * FROM wp_edd_licenses')->fetchAll(PDO::FETCH_ASSOC);
$canonicalKeys = 0;
$syntheticKeys = 0;
foreach ($licenseRows as $licenseRow) {
    $key = (string) $licenseRow['license_key'];
    if (str_starts_with($key, 'focusa_live_')) {
        $syntheticKeys++;
        expect_matrix(in_array($key, ['focusa_live_1001_' . str_repeat('a', 16)], true), 'the only synthetic keys are the explicit preserved legacy fixture');
    } elseif (preg_match($KEY_PATTERN, $key) === 1) {
        $canonicalKeys++;
    }
}
expect_matrix($canonicalKeys === 4, "exactly four canonical EDD keys issued by the single authority (focusa + uiai + bundle + duplicate-order) got {$canonicalKeys}");
expect_matrix($syntheticKeys === 1, 'the preserved synthetic install-site key is the only non-canonical row');

// Rollback is preservation-only across every surface schema.
$preservations = [
    $registrationMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $tokenMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $gateMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $intentMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $integrityMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $bindingMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $issuanceMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $evaluationMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $lifecycleMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $outboxMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
    $reconciliationMigration->preserveForRollback('2026-08-08T00:03:00Z', ['source' => 'edd_commerce_acceptance_test']),
];
foreach ($preservations as $preserved) {
    expect_matrix(($preserved['action'] ?? '') === 'preserve', 'every surface rollback contract is preservation-only');
}
expect_matrix($counts() === $finalCounts, 'rollback preservation never deletes customer/order/license/sequence/audit truth');

$summary = [
    'schema' => 'focusa.spec152e.edd_commerce_acceptance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'matrix_rows' => [
        'website_paid_focusa', 'terminal_paid_focusa', 'agent_paid_focusa', 'paid_uiai',
        'bundle_exact_union', 'evaluation', 'wrong_product', 'arbitrary_amount_grants',
        'unrelated_download', 'duplicate_replayed_order', 'changed_email_hold',
        'refund', 'revocation', 'expiry', 'authority_outage', 'legacy_install_site',
    ],
    'canonical_licenses_created' => $canonicalKeys,
    'synthetic_legacy_preserved' => $syntheticKeys,
    'licenses_table' => $finalCounts['licenses'],
    'bindings' => $finalCounts['bindings'],
    'issuance_requests' => $finalCounts['issuance_requests'],
    'issuances' => $finalCounts['issuances'],
    'postures' => $finalCounts['postures'],
    'assertions' => $finalCounts['assertions'],
    'evaluation_decisions' => $finalCounts['evaluations'],
    'lifecycle_events' => $finalCounts['lifecycle_events'],
    'outbox_events' => $finalCounts['outbox'],
    'deliveries' => $finalCounts['deliveries'],
    'consumer_applications' => $finalCounts['applications'],
    'final_sequence' => $accountSequence($focusaAccount),
    'reconciliation_repairs' => $applyRun['summary']['repairs_applied'],
    'reconciliation_converged' => $fullSecond['summary']['converged'],
    'reconciliation_quarantine_stable' => $fullSecond['summary']['stable_quarantine'],
    'reconciliation_handle_deterministic' => $handleA === $handleB,
    'identity_hold_release' => 'verified_link_review_only',
    'entitlement_issuance' => 'canonical_edd_software_licensing_single_authority',
    'authority' => 'one_customer_order_license_authority_no_install_site_issuance',
    'storage' => 'opaque_refs_only_no_email_no_secrets_no_keys',
    'result' => 'passed_fail_closed',
];
$summary['result_handle'] = hash('sha256', json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES));
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
