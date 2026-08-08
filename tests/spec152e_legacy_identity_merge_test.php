<?php
// 152E.01.09 Require verification for legacy EDD customers and key owners.
// Forces mailbox verification before new-node activation, reissue, account merge, or
// terminal delivery for legacy customers; preserves existing order/license state; and
// requires stronger evidence for conflicting paid records. Raw matching email alone
// never transfers ownership; synthetic records remain quarantined.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-legacy-activation-adapter.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_legacy(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_legacy_throws_code(callable $operation, string $code, string $message): void
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

function expect_legacy_throws_type(callable $operation, string $exception, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    try {
        $operation();
    } catch (Throwable $error) {
        if (!($error instanceof $exception)) {
            fwrite(STDERR, "FAIL: {$message} (got " . get_class($error) . ": {$error->getMessage()})\n");
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_identity_merge_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_identity_merge_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_identity_merge_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_identity_merge_test']);

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

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-08T00:01:00Z'))
        ->modify('+' . $clockTick . ' minutes')
        ->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
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
$legacy = new FocusaSpec152eLegacyActivationAdapter($db, $registrations, $registrationSecrets, $edd, $clock);

$counts = static function () use ($db): array {
    return [
        'accounts' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn(),
        'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
        'identities' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities')->fetchColumn(),
        'links' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_account_promotion_purchase_links')->fetchColumn(),
    ];
};

$registrationSeq = 0;
$createVerified = static function (string $email, string $facade, string $tag) use ($registrations, &$registrationSeq): array {
    $registrationSeq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.legacy.test',
        'install_channel' => 'cli',
        'product_code' => 'focusa_operator',
        'safe_redirect_handle' => 'safe-' . $tag . '-' . $registrationSeq,
        'request_id' => 'req-' . $tag . '-' . $registrationSeq,
        'idempotency_key' => 'idem-' . $tag . '-' . $registrationSeq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $registrationSeq,
        'idem-verify-' . $tag . '-' . $registrationSeq,
    );
    return [
        'registration_uuid' => $uuid,
        'verified_at' => $verified['registration']['verified_at'],
    ];
};

$uuidPattern = '/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D';
$isUuid = static fn(string $value): bool => preg_match($uuidPattern, $value) === 1;

$insertLegacyCustomer = static function (string $email) use ($db): int {
    $db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
        VALUES (NULL, '{$email}', 'Legacy Customer', 99.50, 1, '', '2026-07-01T00:00:00Z', NULL)");
    return (int) $db->lastInsertId();
};
$insertLegacyOrder = static function (int $customerId, string $email, string $status, string $orderNumber) use ($db): int {
    $db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
        VALUES ('{$orderNumber}', '{$status}', 'sale', '2026-07-01T00:01:00Z', '2026-07-01T00:02:00Z', {$customerId}, '{$email}')");
    return (int) $db->lastInsertId();
};
$insertLegacyLicense = static function (int $customerId, int $orderId, string $key, string $status) use ($db): int {
    $db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
        VALUES ('{$key}', {$customerId}, {$orderId}, 453, '{$status}')");
    return (int) $db->lastInsertId();
};

// Legacy fixture alpha: paid owner with a complete order and an active key.
$alphaCustomer = $insertLegacyCustomer('legacy.alpha@example.invalid');
$alphaOrder = $insertLegacyOrder($alphaCustomer, 'legacy.alpha@example.invalid', 'complete', 'ORD-2001');
$db->exec("INSERT INTO wp_edd_order_items (order_id, product_id, product_name, quantity)
    VALUES ({$alphaOrder}, 453, 'WPUIAI Pro Lifetime', 1)");
$alphaItem = (int) $db->lastInsertId();
$alphaLicense = $insertLegacyLicense($alphaCustomer, $alphaOrder, 'fl_legacy_alpha_0001', 'active');

// Legacy fixture bravo: key is revoked (unusable).
$bravoCustomer = $insertLegacyCustomer('legacy.bravo@example.invalid');
$bravoOrder = $insertLegacyOrder($bravoCustomer, 'legacy.bravo@example.invalid', 'complete', 'ORD-2002');
$bravoLicense = $insertLegacyLicense($bravoCustomer, $bravoOrder, 'fl_legacy_bravo_0001', 'revoked');

// Legacy fixture charlie: order is refunded (not complete).
$charlieCustomer = $insertLegacyCustomer('legacy.charlie@example.invalid');
$charlieOrder = $insertLegacyOrder($charlieCustomer, 'legacy.charlie@example.invalid', 'refunded', 'ORD-2003');
$charlieLicense = $insertLegacyLicense($charlieCustomer, $charlieOrder, 'fl_legacy_charlie_0001', 'active');

// Legacy fixture delta: valid key owned by delta; unrelated email must not activate it.
$deltaCustomer = $insertLegacyCustomer('legacy.delta@example.invalid');
$deltaOrder = $insertLegacyOrder($deltaCustomer, 'legacy.delta@example.invalid', 'complete', 'ORD-2004');
$deltaLicense = $insertLegacyLicense($deltaCustomer, $deltaOrder, 'fl_legacy_delta_0001', 'active');

// Legacy fixture echo: key owned by echo; a linked secondary email is an owner email too.
$echoCustomer = $insertLegacyCustomer('legacy.echo@example.invalid');
$db->exec("INSERT INTO wp_edd_customer_email_addresses (customer_id, email, type, date_created)
    VALUES ({$echoCustomer}, 'legacy.echo.linked@example.invalid', 'secondary', '2026-07-02T00:00:00Z')");
$echoOrder = $insertLegacyOrder($echoCustomer, 'legacy.echo@example.invalid', 'complete', 'ORD-2005');
$echoLicense = $insertLegacyLicense($echoCustomer, $echoOrder, 'fl_legacy_echo_0001', 'active');

// Legacy fixture synthetic: a synthetic key must remain quarantined.
$syntheticCustomer = $insertLegacyCustomer('legacy.synthetic@example.invalid');
$syntheticOrder = $insertLegacyOrder($syntheticCustomer, 'legacy.synthetic@example.invalid', 'complete', 'ORD-2006');
$syntheticLicense = $insertLegacyLicense($syntheticCustomer, $syntheticOrder, 'fl_legacy_synthetic_0001', 'active');

// Legacy fixture foreign: order.email drifts to another address while the license
// belongs to a different customer; raw email match alone must not transfer ownership.
$foreignCustomer = $insertLegacyCustomer('foreign.holder@example.invalid');
$foreignOrder = $insertLegacyOrder($foreignCustomer, 'legacy.alpha@example.invalid', 'complete', 'ORD-2099');
$foreignLicense = $insertLegacyLicense($foreignCustomer, $foreignOrder, 'fl_legacy_foreign_0001', 'active');

$purchaseEvidence = ['kind' => 'purchase_evidence', 'source' => 'edd_software_licensing', 'record' => 'legacy-alpha-2001'];
$linkedEvidence = ['kind' => 'install_site_migration', 'source' => 'install_site_reconciliation', 'record' => 'legacy-echo-2005'];
$stripeEvidence = ['kind' => 'stripe_reconciliation', 'source' => 'stripe_reconciliation', 'record' => 'legacy-delta-2004'];
$syntheticEvidence = ['kind' => 'synthetic', 'source' => 'custom_key_generator', 'record' => 'legacy-synthetic-2006'];

$regAlpha = $createVerified('legacy.alpha@example.invalid', 'focusa_install_v1', 'alpha');
$regBravo = $createVerified('legacy.bravo@example.invalid', 'focusa_install_v1', 'bravo');
$regCharlie = $createVerified('legacy.charlie@example.invalid', 'focusa_install_v1', 'charlie');
$regDelta = $createVerified('legacy.delta@example.invalid', 'focusa_install_v1', 'delta');
$regUnrelated = $createVerified('unrelated@example.invalid', 'focusa_install_v1', 'unrelated');
$regEchoLinked = $createVerified('legacy.echo.linked@example.invalid', 'focusa_install_v1', 'echolinked');
$regSynthetic = $createVerified('legacy.synthetic@example.invalid', 'focusa_install_v1', 'synthetic');

// ── Positive: legacy activation adapter gates each purpose ─────────────

$alphaNode = $legacy->resolveForActivation([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'license_key' => 'fl_legacy_alpha_0001',
    'purpose' => 'node_activation',
    'legacy_evidence' => $purchaseEvidence,
    'request_id' => 'req-resolve-alpha-node-0001',
]);
expect_legacy($alphaNode['schema'] === 'focusa.spec152e.legacy_activation_resolution.v1', 'legacy resolution returns the typed envelope');
expect_legacy($alphaNode['verification_required'] === false, 'verified legacy owner is not sent back to verification');
expect_legacy($alphaNode['owner_match'] === true, 'legacy key owner matches the verified email');
expect_legacy($alphaNode['node_activation_allowed'] === true, 'verified legacy owner may activate a new node');
expect_legacy($alphaNode['reissue_allowed'] === false, 'node-activation resolution does not grant reissue');
expect_legacy($alphaNode['terminal_delivery_allowed'] === false, 'node-activation resolution does not grant terminal delivery');
expect_legacy($alphaNode['account_merge_allowed'] === false, 'node-activation resolution does not grant account merge');
expect_legacy((int) $alphaNode['license_id'] === $alphaLicense, 'resolution returns the masked license reference');
expect_legacy((int) $alphaNode['customer_id'] === $alphaCustomer, 'resolution returns the masked customer reference');
expect_legacy((int) $alphaNode['order_id'] === $alphaOrder, 'resolution returns the masked order reference');
expect_legacy((int) $alphaNode['product_id'] === 453, 'resolution returns the product reference');
expect_legacy($alphaNode['status'] === 'active', 'resolution reports the preserved license status');
expect_legacy(preg_match('/^[a-f0-9]{64}$/D', $alphaNode['evidence_digest']) === 1, 'resolution pins a bounded evidence digest');
$alphaNodeJson = json_encode($alphaNode, JSON_THROW_ON_ERROR);
expect_legacy(!str_contains($alphaNodeJson, 'legacy.alpha@example.invalid'), 'resolution never leaks the verified email');
expect_legacy(!str_contains($alphaNodeJson, 'fl_legacy_alpha_0001'), 'resolution never leaks the license key');

$alphaReissue = $legacy->resolveForActivation([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'license_key' => 'fl_legacy_alpha_0001',
    'purpose' => 'reissue',
    'legacy_evidence' => $purchaseEvidence,
    'request_id' => 'req-resolve-alpha-reissue-0001',
]);
expect_legacy($alphaReissue['reissue_allowed'] === true, 'verified legacy owner may request a reissue');
expect_legacy($alphaReissue['node_activation_allowed'] === false, 'reissue resolution does not grant new-node activation');

$alphaTerminal = $legacy->resolveForActivation([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'license_key' => 'fl_legacy_alpha_0001',
    'purpose' => 'terminal_delivery',
    'legacy_evidence' => $purchaseEvidence,
    'request_id' => 'req-resolve-alpha-terminal-0001',
]);
expect_legacy($alphaTerminal['terminal_delivery_allowed'] === true, 'verified legacy owner may receive terminal delivery');

$alphaMergePre = $legacy->resolveForActivation([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'license_key' => 'fl_legacy_alpha_0001',
    'purpose' => 'account_merge',
    'legacy_evidence' => $purchaseEvidence,
    'request_id' => 'req-resolve-alpha-merge-0001',
]);
expect_legacy($alphaMergePre['account_merge_allowed'] === true, 'verified legacy owner passes the account-merge pre-check');

// Deterministic resolution: same inputs, same digest, no writes.
$alphaNodeAgain = $legacy->resolveForActivation([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'license_key' => 'fl_legacy_alpha_0001',
    'purpose' => 'node_activation',
    'legacy_evidence' => $purchaseEvidence,
    'request_id' => 'req-resolve-alpha-node-0002',
]);
expect_legacy($alphaNodeAgain['evidence_digest'] === $alphaNode['evidence_digest'], 'legacy resolution is deterministic and evidence-pinned');
expect_legacy($counts()['customers'] === 7, 'legacy resolution is read-only over EDD truth');

// Linked (secondary) owner email resolves the same legacy key.
$echoNode = $legacy->resolveForActivation([
    'registration_uuid' => $regEchoLinked['registration_uuid'],
    'verified_email' => 'legacy.echo.linked@example.invalid',
    'license_key' => 'fl_legacy_echo_0001',
    'purpose' => 'node_activation',
    'legacy_evidence' => $linkedEvidence,
    'request_id' => 'req-resolve-echo-node-0001',
]);
expect_legacy($echoNode['owner_match'] === true, 'a verified linked owner email matches the legacy key owner');
expect_legacy((int) $echoNode['customer_id'] === $echoCustomer, 'linked-owner resolution returns the legacy customer');

// ── Positive: atomic legacy merge preserves order/license truth ────────

$alphaOrderBefore = $db->query("SELECT * FROM wp_edd_orders WHERE id = {$alphaOrder}")->fetch(PDO::FETCH_ASSOC);
$alphaLicenseBefore = $db->query("SELECT * FROM wp_edd_licenses WHERE id = {$alphaLicense}")->fetch(PDO::FETCH_ASSOC);
$alphaCustomerBefore = $db->query("SELECT * FROM wp_edd_customers WHERE id = {$alphaCustomer}")->fetch(PDO::FETCH_ASSOC);

$merge = $promotion->mergeLegacyVerified([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'promotional_consent_at' => null,
    'request_id' => 'req-merge-alpha-0001',
    'idempotency_key' => 'idem-merge-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'legacy-merge-alpha'],
    'legacy_key' => 'fl_legacy_alpha_0001',
    'legacy_evidence' => $purchaseEvidence,
    'prior_purchases' => [
        ['order_id' => $alphaOrder, 'item_id' => $alphaItem, 'license_id' => $alphaLicense],
    ],
]);
expect_legacy($merge['schema'] === 'focusa.spec152e.account_promotion_result.v1', 'legacy merge returns the typed promotion envelope');
expect_legacy($merge['legacy_merge'] === true, 'merge result marks the legacy merge');
expect_legacy((int) $merge['legacy_license_id'] === $alphaLicense, 'merge pins the resolved legacy license');
expect_legacy((int) $merge['legacy_order_id'] === $alphaOrder, 'merge pins the resolved legacy order');
expect_legacy($merge['customer_resolution'] === 'existing', 'legacy merge resolves the existing EDD customer');
expect_legacy($isUuid($merge['account_uuid']) && $isUuid($merge['identity_uuid']), 'merge creates the authority account and identity');
expect_legacy($merge['linked_orders'] === [$alphaOrder], 'merge links the evidence-backed legacy order');
expect_legacy($merge['linked_licenses'] === [$alphaLicense], 'merge links the evidence-backed legacy license');
expect_legacy(!str_contains(json_encode($merge, JSON_THROW_ON_ERROR), 'legacy.alpha@example.invalid'), 'merge envelope never leaks the verified email');
expect_legacy(!str_contains(json_encode($merge, JSON_THROW_ON_ERROR), 'fl_legacy_alpha_0001'), 'merge envelope never leaks the license key');

$alphaOrderAfter = $db->query("SELECT * FROM wp_edd_orders WHERE id = {$alphaOrder}")->fetch(PDO::FETCH_ASSOC);
$alphaLicenseAfter = $db->query("SELECT * FROM wp_edd_licenses WHERE id = {$alphaLicense}")->fetch(PDO::FETCH_ASSOC);
$alphaCustomerAfter = $db->query("SELECT * FROM wp_edd_customers WHERE id = {$alphaCustomer}")->fetch(PDO::FETCH_ASSOC);
expect_legacy($alphaOrderAfter === $alphaOrderBefore, 'legacy merge preserves the EDD order row untouched');
expect_legacy($alphaLicenseAfter === $alphaLicenseBefore, 'legacy merge preserves the EDD license row untouched');
expect_legacy($alphaCustomerAfter === $alphaCustomerBefore, 'legacy merge preserves EDD customer purchase state');
expect_legacy((string) $alphaLicenseAfter['status'] === 'active', 'paid entitlement is not downgraded by the merge');

$linkRow = $db->query("SELECT * FROM wp_wpuiai_account_promotion_purchase_links WHERE edd_order_id = {$alphaOrder}")->fetch(PDO::FETCH_ASSOC);
$linkProvenance = json_decode($linkRow['migration_provenance'], true, 512, JSON_THROW_ON_ERROR);
$expectedEvidence = $purchaseEvidence;
ksort($expectedEvidence, SORT_STRING);
expect_legacy(($linkProvenance['legacy_evidence'] ?? null) === $expectedEvidence, 'purchase link journals the legacy evidence');
expect_legacy(isset($linkProvenance['legacy_evidence_digest']) && strlen($linkProvenance['legacy_evidence_digest']) === 64, 'purchase link journals the bounded legacy evidence digest');

$mergeReg = $registrations->findByUuid($regAlpha['registration_uuid']);
expect_legacy($mergeReg['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'legacy merge advances the registration to account_promoted');
$c1 = $counts();
expect_legacy($c1['accounts'] === 1 && $c1['customers'] === 7 && $c1['identities'] === 1 && $c1['links'] === 1, 'one legacy merge yields one account, identity, and purchase link');

// Identical replay returns the same bounded result with no duplicates.
$mergeReplay = $promotion->mergeLegacyVerified([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'promotional_consent_at' => null,
    'request_id' => 'req-merge-alpha-0001',
    'idempotency_key' => 'idem-merge-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'legacy-merge-alpha'],
    'legacy_key' => 'fl_legacy_alpha_0001',
    'legacy_evidence' => $purchaseEvidence,
    'prior_purchases' => [
        ['order_id' => $alphaOrder, 'item_id' => $alphaItem, 'license_id' => $alphaLicense],
    ],
]);
expect_legacy($mergeReplay['replayed'] === true, 'identical legacy merge replays');
expect_legacy($mergeReplay['account_uuid'] === $merge['account_uuid'], 'merge replay returns the same authority account');
expect_legacy($mergeReplay['legacy_license_id'] === $alphaLicense, 'merge replay returns the same pinned license');
$c2 = $counts();
expect_legacy($c2['accounts'] === 1 && $c2['customers'] === 7 && $c2['identities'] === 1 && $c2['links'] === 1, 'merge replay creates no duplicates');

// Legacy merge through a linked owner email attaches to the same customer.
$echoMerge = $promotion->mergeLegacyVerified([
    'registration_uuid' => $regEchoLinked['registration_uuid'],
    'verified_email' => 'legacy.echo.linked@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:40:00Z',
    'promotional_consent_at' => null,
    'request_id' => 'req-merge-echo-0001',
    'idempotency_key' => 'idem-merge-echo-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'legacy-merge-echo'],
    'legacy_key' => 'fl_legacy_echo_0001',
    'legacy_evidence' => $linkedEvidence,
    'prior_purchases' => [
        ['order_id' => $echoOrder, 'license_id' => $echoLicense],
    ],
]);
expect_legacy((int) $echoMerge['edd_customer_id'] === $echoCustomer, 'linked-owner legacy merge resolves the owning customer');
expect_legacy($echoMerge['legacy_merge'] === true && (int) $echoMerge['legacy_license_id'] === $echoLicense, 'linked-owner merge pins the legacy license');
$c3 = $counts();
expect_legacy($c3['accounts'] === 2 && $c3['identities'] === 2 && $c3['links'] === 2, 'linked-owner merge creates exactly one more account and identity');

// ── Negative: legacy activation adapter fails closed ───────────────────

$regUnverified = $registrations->createPending([
    'email' => 'legacy.unverified@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'candidate.legacy.test',
    'install_channel' => 'cli',
    'product_code' => 'focusa_operator',
    'safe_redirect_handle' => 'safe-unverified-1',
    'request_id' => 'req-unverified-0001',
    'idempotency_key' => 'idem-unverified-0001',
]);
$unverifiedUuid = $regUnverified['registration']['registration_uuid'];

expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $unverifiedUuid,
        'verified_email' => 'legacy.unverified@example.invalid',
        'license_key' => 'fl_legacy_alpha_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-unverified-0001',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'legacy activation is impossible without mailbox verification'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha.mismatch@example.invalid',
        'license_key' => 'fl_legacy_alpha_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-mismatch-0001',
    ]),
    'ACCOUNT_EMAIL_MISMATCH',
    'registration email digest must bind the submitted email'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'license_key' => 'fl_legacy_unknown_0000',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-unknownkey-0001',
    ]),
    'EDD_LICENSE_UNVERIFIED',
    'an unknown legacy key fails closed without revealing existence'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regBravo['registration_uuid'],
        'verified_email' => 'legacy.bravo@example.invalid',
        'license_key' => 'fl_legacy_bravo_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-revoked-0001',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'a revoked legacy key cannot authorize activation'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regCharlie['registration_uuid'],
        'verified_email' => 'legacy.charlie@example.invalid',
        'license_key' => 'fl_legacy_charlie_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-refunded-0001',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'a refunded legacy order cannot authorize activation'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regUnrelated['registration_uuid'],
        'verified_email' => 'unrelated@example.invalid',
        'license_key' => 'fl_legacy_delta_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $stripeEvidence,
        'request_id' => 'req-neg-unrelated-0001',
    ]),
    'LICENSE_ACCOUNT_MISMATCH',
    'a key and an unrelated verified email cannot activate a node'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'license_key' => 'fl_legacy_alpha_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $syntheticEvidence,
        'request_id' => 'req-neg-synthetic-0001',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'synthetic legacy records remain quarantined'
);
expect_legacy_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'license_key' => 'fl_legacy_alpha_0001',
        'purpose' => 'node_activation',
        'request_id' => 'req-neg-noevidence-0001',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'missing legacy evidence is quarantined'
);
expect_legacy_throws_type(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'license_key' => 'fl_legacy_alpha_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $purchaseEvidence,
        'request_id' => 'req-neg-badpurpose2-0001',
        'purpose' => 'self_issue',
    ]),
    InvalidArgumentException::class,
    'an unknown purpose is rejected as malformed input'
);

// ── Negative: legacy merge fails closed and requires stronger evidence ─

// Fresh verified registrations for merge negatives (regAlpha is promoted by then).
$regAlphaNegUnknown = $createVerified('legacy.alpha@example.invalid', 'focusa_install_v1', 'alpha-neg-unknown');
$regAlphaNegForeign = $createVerified('legacy.alpha@example.invalid', 'focusa_install_v1', 'alpha-neg-foreign');

expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $unverifiedUuid,
        'verified_email' => 'legacy.unverified@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-unverified-0001',
        'idempotency_key' => 'idem-neg-merge-unverified-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-unverified'],
        'legacy_key' => 'fl_legacy_alpha_0001',
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $alphaOrder, 'license_id' => $alphaLicense]],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'legacy merge is impossible without mailbox verification'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regAlphaNegUnknown['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-unknownkey-0001',
        'idempotency_key' => 'idem-neg-merge-unknownkey-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-unknownkey'],
        'legacy_key' => 'fl_legacy_unknown_0000',
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $alphaOrder, 'license_id' => $alphaLicense]],
    ]),
    'EDD_LICENSE_UNVERIFIED',
    'legacy merge with an unknown key fails closed'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regBravo['registration_uuid'],
        'verified_email' => 'legacy.bravo@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-revoked-0001',
        'idempotency_key' => 'idem-neg-merge-revoked-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-revoked'],
        'legacy_key' => 'fl_legacy_bravo_0001',
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $bravoOrder, 'license_id' => $bravoLicense]],
    ]),
    'EDD_LICENSE_UNUSABLE',
    'legacy merge of a revoked license is denied'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regCharlie['registration_uuid'],
        'verified_email' => 'legacy.charlie@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-refunded-0001',
        'idempotency_key' => 'idem-neg-merge-refunded-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-refunded'],
        'legacy_key' => 'fl_legacy_charlie_0001',
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $charlieOrder, 'license_id' => $charlieLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'legacy merge of a refunded order is denied'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regUnrelated['registration_uuid'],
        'verified_email' => 'unrelated@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-unrelated-0001',
        'idempotency_key' => 'idem-neg-merge-unrelated-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-unrelated'],
        'legacy_key' => 'fl_legacy_delta_0001',
        'legacy_evidence' => $stripeEvidence,
        'prior_purchases' => [['order_id' => $deltaOrder, 'license_id' => $deltaLicense]],
    ]),
    'LICENSE_ACCOUNT_MISMATCH',
    'legacy merge never transfers ownership to an unrelated verified email'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-synthetic-0001',
        'idempotency_key' => 'idem-neg-merge-synthetic-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-synthetic'],
        'legacy_key' => 'fl_legacy_synthetic_0001',
        'legacy_evidence' => $syntheticEvidence,
        'prior_purchases' => [['order_id' => $syntheticOrder, 'license_id' => $syntheticLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'synthetic legacy evidence remains quarantined in the merge'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-noevidence-0001',
        'idempotency_key' => 'idem-neg-merge-noevidence-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-noevidence'],
        'legacy_key' => 'fl_legacy_alpha_0001',
        'prior_purchases' => [['order_id' => $alphaOrder, 'license_id' => $alphaLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'a legacy merge without evidence is quarantined'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regDelta['registration_uuid'],
        'verified_email' => 'legacy.delta@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-conflict-0001',
        'idempotency_key' => 'idem-neg-merge-conflict-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-conflict'],
        'legacy_key' => 'fl_legacy_delta_0001',
        'legacy_evidence' => $stripeEvidence,
        'prior_purchases' => [['order_id' => $echoOrder, 'license_id' => $echoLicense]],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'conflicting paid records require stronger evidence and enter review'
);
expect_legacy_throws_code(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regAlphaNegForeign['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-foreign-0001',
        'idempotency_key' => 'idem-neg-merge-foreign-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-foreign'],
        'legacy_key' => 'fl_legacy_foreign_0001',
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $foreignOrder, 'license_id' => $foreignLicense]],
    ]),
    'LICENSE_ACCOUNT_MISMATCH',
    'raw order-email match alone never transfers ownership to a different customer'
);
expect_legacy_throws_type(
    fn() => $promotion->mergeLegacyVerified([
        'registration_uuid' => $regAlpha['registration_uuid'],
        'verified_email' => 'legacy.alpha@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:50:00Z',
        'request_id' => 'req-neg-merge-nokey-0001',
        'idempotency_key' => 'idem-neg-merge-nokey-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'neg-nokey'],
        'legacy_evidence' => $purchaseEvidence,
        'prior_purchases' => [['order_id' => $alphaOrder, 'license_id' => $alphaLicense]],
    ]),
    InvalidArgumentException::class,
    'a legacy merge without a key is malformed input'
);

// Failed merges write nothing.
$cNeg = $counts();
expect_legacy($cNeg['accounts'] === 2 && $cNeg['identities'] === 2 && $cNeg['links'] === 2, 'all failed legacy merges write zero state');

// ── Rollback preservation ──────────────────────────────────────────────

$beforeRollback = $counts();
$legacyRollback = $promotionMigration->preserveForRollback('2026-08-08T02:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_legacy_identity_merge_rollback',
]);
expect_legacy($legacyRollback['action'] === 'preserve', 'legacy rollback is preservation-only');
expect_legacy($counts() === $beforeRollback, 'rollback preserves legacy merge truth');
expect_legacy((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_account_promotion_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.legacy_identity_merge_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'accounts' => $counts()['accounts'],
    'customers' => $counts()['customers'],
    'identities' => $counts()['identities'],
    'purchase_links' => $counts()['links'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
