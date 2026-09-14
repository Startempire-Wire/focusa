<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_promotion(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_promotion_throws_code(callable $operation, string $code, string $message): void
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

function expect_promotion_throws_type(callable $operation, string $exception, string $message): void
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

// Authority schemas.
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'account_promotion_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'account_promotion_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'account_promotion_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'account_promotion_test']);

// EDD tables (simulated EDD 3.x schema).
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
$db->exec("CREATE TABLE wp_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_email VARCHAR(100) NOT NULL
)");
$db->exec("INSERT INTO wp_users (id, user_email) VALUES (501, 'wp-501@example.invalid')");
$db->exec("INSERT INTO wp_users (id, user_email) VALUES (777, 'wp-777@example.invalid')");

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

$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $clock);
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

$counts = static function () use ($db): array {
    return [
        'accounts' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn(),
        'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
        'identities' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities')->fetchColumn(),
        'links' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_account_promotion_purchase_links')->fetchColumn(),
        'promotions' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_account_promotion_idempotency')->fetchColumn(),
    ];
};

$registrationSeq = 0;
$createVerified = static function (string $email, string $facade, string $tag) use ($registrations, &$registrationSeq): array {
    $registrationSeq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.promotion.test',
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
        'state_version' => (int) $verified['registration']['state_version'],
    ];
};

$uuidPattern = '/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D';
$isUuid = static fn(string $value): bool => preg_match($uuidPattern, $value) === 1;

// ── Positive 1: fresh verified promotion is atomic and complete ─────────

$regAlpha = $createVerified('synthetic.alpha@example.invalid', 'focusa_install_v1', 'alpha');
$alpha = $promotion->promoteVerified([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'synthetic.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'promotional_consent_at' => '2026-08-08T00:31:00Z',
    'wordpress_user_id' => 501,
    'stripe_customer_id' => 'cus_promotion_alpha',
    'request_id' => 'req-promote-alpha-0001',
    'idempotency_key' => 'idem-promote-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-alpha'],
]);
expect_promotion($alpha['schema'] === 'focusa.spec152e.account_promotion_result.v1', 'promotion returns typed result envelope');
expect_promotion($alpha['account_resolution'] === 'new', 'fresh promotion creates a new authority account');
expect_promotion($alpha['customer_resolution'] === 'new', 'fresh promotion creates a new EDD customer');
expect_promotion($isUuid($alpha['account_uuid']), 'promotion returns opaque account UUID');
expect_promotion($isUuid($alpha['identity_uuid']), 'promotion returns opaque identity UUID');
expect_promotion((int) $alpha['edd_customer_id'] > 0, 'promotion links a positive EDD customer ID');
expect_promotion($alpha['identity_state'] === 'primary', 'first verified identity is marked primary');
expect_promotion($alpha['transactional_consent_at'] === '2026-08-08T00:30:00Z', 'transactional consent is persisted');
expect_promotion($alpha['promotional_consent_at'] === '2026-08-08T00:31:00Z', 'promotional consent is persisted separately');
expect_promotion($alpha['linked_orders'] === [], 'no prior orders linked when none provided');
expect_promotion($alpha['replayed'] === false, 'first promotion is not a replay');
expect_promotion(!isset($alpha['email']), 'raw email is absent from promotion envelope');
expect_promotion(!str_contains(json_encode($alpha, JSON_THROW_ON_ERROR), 'synthetic.alpha@example.invalid'), 'masked envelope never leaks the verified email');

$alphaReg = $registrations->findByUuid($regAlpha['registration_uuid']);
expect_promotion($alphaReg['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'registration advances to account_promoted');
expect_promotion((string) $alphaReg['account_uuid'] === $alpha['account_uuid'], 'registration stores the promoted account UUID');
expect_promotion((int) $alphaReg['edd_customer_id'] === (int) $alpha['edd_customer_id'], 'registration stores the promoted EDD customer ID');
$c1 = $counts();
expect_promotion($c1['accounts'] === 1 && $c1['customers'] === 1 && $c1['identities'] === 1, 'one promotion yields exactly one account, customer, and identity');

$alphaIdentity = $identities->findByUuid($alpha['identity_uuid']);
expect_promotion($alphaIdentity['transactional_consent_at'] === '2026-08-08T00:30:00Z', 'identity row records transactional consent');
expect_promotion($alphaIdentity['promotional_consent_at'] === '2026-08-08T00:31:00Z', 'identity row records promotional consent');
expect_promotion($alphaIdentity['bounce_state'] === 'none' && $alphaIdentity['suppression_state'] === 'none', 'promoted identity starts clean of bounce and suppression');
$alphaAccount = $accounts->findByUuid($alpha['account_uuid']);
expect_promotion((int) $alphaAccount['wordpress_user_id'] === 501, 'optional WordPress user is linked to the authority account');
expect_promotion((int) $alphaAccount['highest_entitlement_sequence'] === 0, 'promoted account starts at entitlement sequence zero');
$alphaCustomer = $edd->findCustomerById((int) $alpha['edd_customer_id']);
expect_promotion((int) $alphaCustomer['user_id'] === 501, 'optional WordPress user is linked to the EDD customer');
expect_promotion($alphaCustomer['stripe_customer_id'] === 'cus_promotion_alpha', 'optional Stripe reference is linked to the EDD customer');

// ── Positive 2: identical replay returns the same bounded result ────────

$alphaReplay = $promotion->promoteVerified([
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'synthetic.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'promotional_consent_at' => '2026-08-08T00:31:00Z',
    'wordpress_user_id' => 501,
    'stripe_customer_id' => 'cus_promotion_alpha',
    'request_id' => 'req-promote-alpha-0001',
    'idempotency_key' => 'idem-promote-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-alpha'],
]);
expect_promotion($alphaReplay['replayed'] === true, 'identical promotion request replays');
expect_promotion($alphaReplay['account_uuid'] === $alpha['account_uuid'], 'replay returns the same authority account');
expect_promotion($alphaReplay['identity_uuid'] === $alpha['identity_uuid'], 'replay returns the same identity');
$c2 = $counts();
expect_promotion($c2['accounts'] === 1 && $c2['customers'] === 1 && $c2['identities'] === 1 && $c2['promotions'] === 1, 'replay creates no duplicate account, customer, identity, or promotion');

// ── Positive 3: evidence-backed prior purchases are linked ──────────────

$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.bravo@example.invalid', '', 0, 0, '', '2026-08-01T00:00:00Z', NULL)");
$bravoCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-1001', 'complete', 'sale', '2026-08-01T00:01:00Z', '2026-08-01T00:02:00Z', {$bravoCustomer}, 'synthetic.bravo@example.invalid')");
$bravoOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_order_items (order_id, product_id, product_name, quantity)
    VALUES ({$bravoOrder}, 453, 'WPUIAI Pro Lifetime', 1)");
$bravoItem = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_bravo_0001', {$bravoCustomer}, {$bravoOrder}, 453, 'active')");
$bravoLicense = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-1002', 'complete', 'sale', '2026-08-02T00:00:00Z', '2026-08-02T00:01:00Z', {$bravoCustomer}, 'synthetic.bravo@example.invalid')");
$bravoOrder2 = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_bravo_0002', {$bravoCustomer}, {$bravoOrder2}, 453, 'active')");
$bravoLicense2 = (int) $db->lastInsertId();

$regBravo = $createVerified('synthetic.bravo@example.invalid', 'focusa_install_v1', 'bravo');
$bravo = $promotion->promoteVerified([
    'registration_uuid' => $regBravo['registration_uuid'],
    'verified_email' => 'synthetic.bravo@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:40:00Z',
    'promotional_consent_at' => null,
    'request_id' => 'req-promote-bravo-0001',
    'idempotency_key' => 'idem-promote-bravo-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-bravo'],
    'prior_purchases' => [
        ['order_id' => $bravoOrder, 'item_id' => $bravoItem, 'license_id' => $bravoLicense],
        ['order_id' => $bravoOrder2, 'license_id' => $bravoLicense2],
    ],
]);
expect_promotion($bravo['customer_resolution'] === 'existing', 'existing EDD customer is resolved, not duplicated');
expect_promotion((int) $bravo['edd_customer_id'] === $bravoCustomer, 'promotion resolves the pre-existing EDD customer');
expect_promotion($bravo['promotional_consent_at'] === null, 'promotional consent stays absent when not provided');
expect_promotion($bravo['linked_orders'] === [$bravoOrder, $bravoOrder2], 'evidence-backed prior orders are linked in order');
expect_promotion($bravo['linked_licenses'] === [$bravoLicense, $bravoLicense2], 'evidence-backed prior licenses are linked');
$c3 = $counts();
expect_promotion($c3['customers'] === 2 && $c3['links'] === 2, 'prior purchase linking creates no duplicate customer or link rows');
$linkRow = $db->query("SELECT * FROM wp_wpuiai_account_promotion_purchase_links WHERE edd_order_id = {$bravoOrder}")->fetch(PDO::FETCH_ASSOC);
expect_promotion((string) $linkRow['account_uuid'] === $bravo['account_uuid'], 'purchase link is bound to the promoted authority account');

// ── Positive 4: same-email merge resolves one account/customer/identity ─

$regAlpha2 = $createVerified('synthetic.alpha@example.invalid', 'focusa_install_v1', 'alpha2');
$alphaMerge = $promotion->promoteVerified([
    'registration_uuid' => $regAlpha2['registration_uuid'],
    'verified_email' => 'synthetic.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:45:00Z',
    'request_id' => 'req-promote-alpha2-0001',
    'idempotency_key' => 'idem-promote-alpha2-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-alpha-merge'],
]);
expect_promotion($alphaMerge['account_resolution'] === 'existing', 'same-email merge resolves the existing authority account');
expect_promotion($alphaMerge['customer_resolution'] === 'existing', 'same-email merge resolves the existing EDD customer');
expect_promotion($alphaMerge['identity_uuid'] === $alpha['identity_uuid'], 'same-email merge resolves the existing verified identity');
expect_promotion($alphaMerge['account_uuid'] === $alpha['account_uuid'], 'merge attaches the new registration to the same account');
$c4 = $counts();
expect_promotion($c4['accounts'] === 2 && $c4['customers'] === 2 && $c4['identities'] === 2, 'merge creates no duplicate account, customer, or identity');
$alpha2Reg = $registrations->findByUuid($regAlpha2['registration_uuid']);
expect_promotion($alpha2Reg['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'merged registration also advances to account_promoted');

// ── Positive 5: secondary email becomes a linked identity on the account ─

$db->prepare("INSERT INTO wp_edd_customer_email_addresses (customer_id, email, type, date_created)
    VALUES (:cid, 'synthetic.alpha.second@example.invalid', 'secondary', '2026-08-08T00:46:00Z')")
    ->execute([':cid' => (int) $alpha['edd_customer_id']]);
$regAlphaSec = $createVerified('synthetic.alpha.second@example.invalid', 'focusa_install_v1', 'alpha-sec');
$alphaSec = $promotion->promoteVerified([
    'registration_uuid' => $regAlphaSec['registration_uuid'],
    'verified_email' => 'synthetic.alpha.second@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:47:00Z',
    'request_id' => 'req-promote-alpha-sec-0001',
    'idempotency_key' => 'idem-promote-alpha-sec-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-alpha-secondary'],
]);
expect_promotion($alphaSec['account_uuid'] === $alpha['account_uuid'], 'secondary verified email attaches to the same authority account');
expect_promotion($alphaSec['identity_state'] === 'linked', 'second verified identity is marked linked');
expect_promotion($alphaSec['identity_uuid'] !== $alpha['identity_uuid'], 'second verified email creates a distinct identity row');
$alphaPrimaryStill = $identities->findByUuid($alpha['identity_uuid']);
expect_promotion($alphaPrimaryStill['identity_state'] === 'primary', 'existing primary identity is preserved');

// ── Positive 6: crash/retry yields exactly one account/customer/identity ─

$regCrash = $createVerified('synthetic.crash@example.invalid', 'focusa_install_v1', 'crash');
$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.crash@example.invalid', '', 0, 0, '', '2026-08-03T00:00:00Z', NULL)");
$crashCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-2001', 'complete', 'sale', '2026-08-03T00:01:00Z', '2026-08-03T00:02:00Z', {$crashCustomer}, 'synthetic.crash@example.invalid')");
$crashOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_crash_0001', {$crashCustomer}, {$crashOrder}, 453, 'revoked')");
$crashLicense = (int) $db->lastInsertId();

$beforeCrash = $counts();
$crashInput = [
    'registration_uuid' => $regCrash['registration_uuid'],
    'verified_email' => 'synthetic.crash@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:50:00Z',
    'request_id' => 'req-promote-crash-0001',
    'idempotency_key' => 'idem-promote-crash-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-crash'],
    'prior_purchases' => [['order_id' => $crashOrder, 'license_id' => $crashLicense]],
];
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified($crashInput),
    'EDD_LICENSE_UNUSABLE',
    'revoked license fails the promotion transaction'
);
$afterCrash = $counts();
expect_promotion($afterCrash === $beforeCrash, 'failed promotion leaves zero partial writes');
$crashRegAfter = $registrations->findByUuid($regCrash['registration_uuid']);
expect_promotion($crashRegAfter['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'failed promotion leaves the registration un-promoted');

$retryInput = $crashInput;
unset($retryInput['prior_purchases']);
$crashRetry = $promotion->promoteVerified($retryInput);
expect_promotion($crashRetry['replayed'] === false, 'retry after rollback executes a fresh promotion');
$c5 = $counts();
expect_promotion(
    $c5['accounts'] === $beforeCrash['accounts'] + 1
    && $c5['identities'] === $beforeCrash['identities'] + 1
    && $c5['customers'] === $beforeCrash['customers'],
    'crash/retry yields exactly one new account and identity with one resolved customer'
);

// ── Negative 1: unverified registration never promotes ─────────────────

$pending = $registrations->createPending([
    'email' => 'synthetic.unverified@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'candidate.promotion.test',
    'install_channel' => 'cli',
    'product_code' => 'focusa_operator',
    'request_id' => 'req-unverified-0001',
    'idempotency_key' => 'idem-unverified-0001',
]);
$beforeUnverified = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $pending['registration']['registration_uuid'],
        'verified_email' => 'synthetic.unverified@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:00:00Z',
        'request_id' => 'req-promote-unverified-0001',
        'idempotency_key' => 'idem-promote-unverified-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-unverified'],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration is denied'
);
expect_promotion($counts() === $beforeUnverified, 'unverified denial writes nothing');

// ── Negative 2: registration/email mismatch never promotes ─────────────

$regMismatch = $createVerified('synthetic.mismatch@example.invalid', 'focusa_install_v1', 'mismatch');
$beforeMismatch = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regMismatch['registration_uuid'],
        'verified_email' => 'synthetic.other-email@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:05:00Z',
        'request_id' => 'req-promote-mismatch-0001',
        'idempotency_key' => 'idem-promote-mismatch-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-mismatch'],
    ]),
    'ACCOUNT_EMAIL_MISMATCH',
    'promoting a different email than the verified registration is denied'
);
expect_promotion($counts() === $beforeMismatch, 'email mismatch denial writes nothing');

// ── Negative 3: expired registration never promotes ────────────────────

$regExpired = $createVerified('synthetic.expired@example.invalid', 'focusa_install_v1', 'expired');
$clockTick += 60; // advance past the 30-minute attempt TTL
$beforeExpired = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regExpired['registration_uuid'],
        'verified_email' => 'synthetic.expired@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T02:00:00Z',
        'request_id' => 'req-promote-expired-0001',
        'idempotency_key' => 'idem-promote-expired-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-expired'],
    ]),
    'REGISTRATION_EXPIRED',
    'expired registration is denied'
);
expect_promotion($counts() === $beforeExpired, 'expired denial writes nothing');

// ── Negative 4: identity bound to another account enters merge review ───

$identities->storeVerified('synthetic.merge-conflict@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-08T01:10:00Z',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0c01',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0d01',
    'identity_state' => 'primary',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T01:10:00Z',
    'promotional_consent_at' => null,
    'promotional_consent_revoked_at' => null,
    'source' => 'merge.conflict.fixture',
    'migration_evidence' => ['record' => 'identity-conflict-fixture'],
]);
$regMergeConflict = $createVerified('synthetic.merge-conflict@example.invalid', 'focusa_install_v1', 'merge-conflict');
$beforeMergeConflict = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regMergeConflict['registration_uuid'],
        'verified_email' => 'synthetic.merge-conflict@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:12:00Z',
        'request_id' => 'req-promote-merge-conflict-0001',
        'idempotency_key' => 'idem-promote-merge-conflict-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-merge-conflict'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'identity bound to another account enters merge review'
);
expect_promotion($counts() === $beforeMergeConflict, 'merge review conflict writes nothing');

// ── Negative 5: WordPress user already linked elsewhere enters review ───

$regWp1 = $createVerified('synthetic.wp-one@example.invalid', 'focusa_install_v1', 'wp1');
$promotion->promoteVerified([
    'registration_uuid' => $regWp1['registration_uuid'],
    'verified_email' => 'synthetic.wp-one@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T01:15:00Z',
    'wordpress_user_id' => 777,
    'request_id' => 'req-promote-wp1-0001',
    'idempotency_key' => 'idem-promote-wp1-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-wp1'],
]);
$regWp2 = $createVerified('synthetic.wp-two@example.invalid', 'focusa_install_v1', 'wp2');
$beforeWp2 = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regWp2['registration_uuid'],
        'verified_email' => 'synthetic.wp-two@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:17:00Z',
        'wordpress_user_id' => 777,
        'request_id' => 'req-promote-wp2-0001',
        'idempotency_key' => 'idem-promote-wp2-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-wp2'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'WordPress user already linked to another account enters merge review'
);
expect_promotion($counts() === $beforeWp2, 'WordPress duplicate denial writes nothing');

// ── Negative 6: Stripe customer already linked elsewhere enters review ──

$regStripe1 = $createVerified('synthetic.stripe-one@example.invalid', 'focusa_install_v1', 'stripe1');
$promotion->promoteVerified([
    'registration_uuid' => $regStripe1['registration_uuid'],
    'verified_email' => 'synthetic.stripe-one@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T01:20:00Z',
    'stripe_customer_id' => 'cus_merge_review_777',
    'request_id' => 'req-promote-stripe1-0001',
    'idempotency_key' => 'idem-promote-stripe1-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-stripe1'],
]);
$regStripe2 = $createVerified('synthetic.stripe-two@example.invalid', 'focusa_install_v1', 'stripe2');
$beforeStripe2 = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regStripe2['registration_uuid'],
        'verified_email' => 'synthetic.stripe-two@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:22:00Z',
        'stripe_customer_id' => 'cus_merge_review_777',
        'request_id' => 'req-promote-stripe2-0001',
        'idempotency_key' => 'idem-promote-stripe2-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-stripe2'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'Stripe customer already linked to another account enters merge review'
);
expect_promotion($counts() === $beforeStripe2, 'Stripe duplicate denial writes nothing');

// ── Negative 7: order not owned by the promoted customer is unverified ─

$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.other-owner@example.invalid', '', 0, 0, '', '2026-08-04T00:00:00Z', NULL)");
$otherCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-3001', 'complete', 'sale', '2026-08-04T00:01:00Z', '2026-08-04T00:02:00Z', {$otherCustomer}, 'synthetic.other-owner@example.invalid')");
$otherOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_other_0001', {$otherCustomer}, {$otherOrder}, 453, 'active')");
$otherLicense = (int) $db->lastInsertId();

$regGamma = $createVerified('synthetic.gamma@example.invalid', 'focusa_install_v1', 'gamma');
$beforeGamma = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regGamma['registration_uuid'],
        'verified_email' => 'synthetic.gamma@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:25:00Z',
        'request_id' => 'req-promote-gamma-0001',
        'idempotency_key' => 'idem-promote-gamma-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-gamma'],
        'prior_purchases' => [['order_id' => $otherOrder, 'license_id' => $otherLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'order owned by another customer is not evidence for this customer'
);
expect_promotion($counts() === $beforeGamma, 'foreign-order denial writes nothing');

// ── Negative 8: refunded order never links ─────────────────────────────

$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.delta@example.invalid', '', 0, 0, '', '2026-08-05T00:00:00Z', NULL)");
$deltaCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-4001', 'refunded', 'sale', '2026-08-05T00:01:00Z', '2026-08-05T00:02:00Z', {$deltaCustomer}, 'synthetic.delta@example.invalid')");
$deltaOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_delta_0001', {$deltaCustomer}, {$deltaOrder}, 453, 'active')");
$deltaLicense = (int) $db->lastInsertId();

$regDelta = $createVerified('synthetic.delta@example.invalid', 'focusa_install_v1', 'delta');
$beforeDelta = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regDelta['registration_uuid'],
        'verified_email' => 'synthetic.delta@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:27:00Z',
        'request_id' => 'req-promote-delta-0001',
        'idempotency_key' => 'idem-promote-delta-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-delta'],
        'prior_purchases' => [['order_id' => $deltaOrder, 'license_id' => $deltaLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'refunded order is not evidence-backed'
);
expect_promotion($counts() === $beforeDelta, 'refunded-order denial writes nothing');

// ── Negative 9: revoked license never links ────────────────────────────

$regEpsilon = $createVerified('synthetic.epsilon@example.invalid', 'focusa_install_v1', 'epsilon');
$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.epsilon@example.invalid', '', 0, 0, '', '2026-08-06T00:00:00Z', NULL)");
$epsilonCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-5001', 'complete', 'sale', '2026-08-06T00:01:00Z', '2026-08-06T00:02:00Z', {$epsilonCustomer}, 'synthetic.epsilon@example.invalid')");
$epsilonOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_epsilon_0001', {$epsilonCustomer}, {$epsilonOrder}, 453, 'revoked')");
$epsilonLicense = (int) $db->lastInsertId();

$beforeEpsilon = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regEpsilon['registration_uuid'],
        'verified_email' => 'synthetic.epsilon@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:30:00Z',
        'request_id' => 'req-promote-epsilon-0001',
        'idempotency_key' => 'idem-promote-epsilon-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-epsilon'],
        'prior_purchases' => [['order_id' => $epsilonOrder, 'license_id' => $epsilonLicense]],
    ]),
    'EDD_LICENSE_UNUSABLE',
    'revoked license is not evidence-backed'
);
expect_promotion($counts() === $beforeEpsilon, 'revoked-license denial writes nothing');

// ── Negative 10: license already linked to another account enters review ─

$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'synthetic.eta@example.invalid', '', 0, 0, '', '2026-08-07T00:00:00Z', NULL)");
$etaCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-6001', 'complete', 'sale', '2026-08-07T00:01:00Z', '2026-08-07T00:02:00Z', {$etaCustomer}, 'synthetic.eta@example.invalid')");
$etaOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_synthetic_eta_0001', {$etaCustomer}, {$etaOrder}, 453, 'active')");
$etaLicense = (int) $db->lastInsertId();

// Pre-seed a legacy claim: the eta order/license is already linked to a different account.
$db->prepare("INSERT INTO wp_wpuiai_account_promotion_purchase_links
    (link_uuid, account_uuid, edd_customer_id, edd_order_id, edd_order_item_id, edd_license_id,
     evidence_digest, linked_at, migration_provenance)
    VALUES (:link, :account, :customer, :order, NULL, :license, :evidence, :linked, :provenance)")
    ->execute([
        ':link' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0e01',
        ':account' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0f01',
        ':customer' => $etaCustomer,
        ':order' => $etaOrder,
        ':license' => $etaLicense,
        ':evidence' => hash('sha256', 'legacy-claim'),
        ':linked' => '2026-08-07T00:03:00Z',
        ':provenance' => '{"source":"legacy_claim_fixture"}',
    ]);
$regEta = $createVerified('synthetic.eta@example.invalid', 'focusa_install_v1', 'eta');
$beforeEta = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regEta['registration_uuid'],
        'verified_email' => 'synthetic.eta@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:35:00Z',
        'request_id' => 'req-promote-eta-0001',
        'idempotency_key' => 'idem-promote-eta-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-eta'],
        'prior_purchases' => [['order_id' => $etaOrder, 'license_id' => $etaLicense]],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'license claimed by another account enters merge review'
);
expect_promotion($counts() === $beforeEta, 'license-conflict denial writes nothing');

// ── Negative 11: missing transactional consent is rejected ─────────────

$regNoConsent = $createVerified('synthetic.noconsent@example.invalid', 'focusa_install_v1', 'noconsent');
$beforeNoConsent = $counts();
expect_promotion_throws_type(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regNoConsent['registration_uuid'],
        'verified_email' => 'synthetic.noconsent@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => null,
        'request_id' => 'req-promote-noconsent-0001',
        'idempotency_key' => 'idem-promote-noconsent-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-noconsent'],
    ]),
    InvalidArgumentException::class,
    'missing transactional consent is rejected'
);
expect_promotion($counts() === $beforeNoConsent, 'missing-consent rejection writes nothing');

// ── Negative 12: changed request cannot reuse an idempotency key ───────

$regIdem = $createVerified('synthetic.idem@example.invalid', 'focusa_install_v1', 'idem');
$promotion->promoteVerified([
    'registration_uuid' => $regIdem['registration_uuid'],
    'verified_email' => 'synthetic.idem@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T01:40:00Z',
    'request_id' => 'req-promote-idem-0001',
    'idempotency_key' => 'idem-promote-idem-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-idem'],
]);
$beforeIdemConflict = $counts();
expect_promotion_throws_code(
    static fn() => $promotion->promoteVerified([
        'registration_uuid' => $regIdem['registration_uuid'],
        'verified_email' => 'synthetic.idem-changed@example.invalid',
        'verification_method' => 'magic_link',
        'transactional_consent_at' => '2026-08-08T01:40:00Z',
        'request_id' => 'req-promote-idem-0001',
        'idempotency_key' => 'idem-promote-idem-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'promotion-idem'],
    ]),
    'IDEMPOTENCY_CONFLICT',
    'changed request cannot reuse a promotion idempotency key'
);
expect_promotion($counts() === $beforeIdemConflict, 'idempotency conflict writes nothing');

// ── Rollback preservation ──────────────────────────────────────────────

$beforeRollback = $counts();
$promotionRollback = $promotionMigration->preserveForRollback('2026-08-08T02:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_account_promotion_rollback',
]);
expect_promotion($promotionRollback['action'] === 'preserve', 'promotion rollback is preservation-only');
expect_promotion($counts() === $beforeRollback, 'rollback preserves promotion truth');
expect_promotion((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_account_promotion_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.account_promotion_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'accounts' => $counts()['accounts'],
    'customers' => $counts()['customers'],
    'identities' => $counts()['identities'],
    'purchase_links' => $counts()['links'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
