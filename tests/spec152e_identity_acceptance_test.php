<?php
// 152E.01.10 Run the complete verified-identity acceptance matrix.
// Exercises new, existing, alias, wrong code, expired, replay, bounce, suppressed,
// resend, duplicate, conflict, legacy paid, prior Evaluation, and facade-switch cases
// across every Spec 152E identity repository/service/route with synthetic fixtures.
// Fail-closed invariant: zero unverified customer, checkout, license, node, or lease
// creation. All result envelopes are masked; no secrets or unmasked real email.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-legacy-activation-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-challenge-service.v1.php';
require_once $root . '/docs/contracts/spec152e-transactional-mail-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-rate-limiter.v1.php';
require_once $root . '/docs/contracts/spec152e-activation-start-handler.v1.php';
require_once $root . '/docs/contracts/spec152e-verification-complete-handler.v1.php';
require_once $root . '/docs/contracts/spec152e-email-delivery-consent.v1.php';
require_once $root . '/docs/contracts/spec152e-facade-protocol.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$productRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_identity(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_identity_throws_code(callable $operation, string $code, string $message): void
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

function expect_identity_throws_type(callable $operation, string $exception, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'identity_acceptance_matrix']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'identity_acceptance_matrix']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'identity_acceptance_matrix']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'identity_acceptance_matrix']);

// EDD 3.x synthetic fixture tables (never the authority surface; fixtures only).
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

// Handler wiring for the route surfaces.
$sent = [];
$mailAdapter = new FocusaSpec152eTransactionalMailAdapter(
    static function (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) use (&$sent): bool {
        $sent[] = ['to' => $to, 'sender' => $senderIdentity];
        return true;
    }
);
$rateLimiter = new FocusaSpec152eRateLimiter($db, 'wp_', $clock, windowSeconds: 60, maxPerWindow: 5, consecutiveMax: 3);
$challengeService = new FocusaSpec152eChallengeService(str_repeat('v', 32));
$startHandler = new FocusaSpec152eActivationStartHandler($registrations, $challengeService, $mailAdapter, $rateLimiter, $clock);
$completeHandler = new FocusaSpec152eVerificationCompleteHandler($registrations, $rateLimiter);
$consentHandler = new FocusaSpec152eEmailDeliveryConsentHandler($identities, $registrations);

$counts = static function () use ($db): array {
    return [
        'accounts' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn(),
        'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
        'identities' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities')->fetchColumn(),
        'registrations' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_activation_registrations')->fetchColumn(),
        'orders' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_orders')->fetchColumn(),
        'licenses' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn(),
        'links' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_account_promotion_purchase_links')->fetchColumn(),
    ];
};

$uuidPattern = '/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D';
$isUuid = static fn(string $value): bool => preg_match($uuidPattern, $value) === 1;

$registrationSeq = 0;
$createVerified = static function (string $email, string $facade, string $tag) use ($registrations, &$registrationSeq): array {
    $registrationSeq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'terminal',
        'install_channel' => 'source_build',
        'product_code' => 'focusa_operator_lifetime_v1',
        'safe_redirect_handle' => 'success-' . $tag . '-' . $registrationSeq,
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

$promoteFixed = static function (array $reg, string $email, string $tag, string $requestId, string $idempotencyKey) use ($promotion): array {
    return $promotion->promoteVerified([
        'registration_uuid' => $reg['registration_uuid'],
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:30:00Z',
        'promotional_consent_at' => null,
        'request_id' => $requestId,
        'idempotency_key' => $idempotencyKey,
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-' . $tag],
    ]);
};

// ── 1. NEW: fresh verified email promotes to one new customer/account/identity ──

$baseBefore = $counts();
$regNew = $createVerified('matrix.new@example.invalid', 'focusa_install_v1', 'new');
$newEnvelope = $promoteFixed($regNew, 'matrix.new@example.invalid', 'new', 'req-promote-new-0001', 'idem-promote-new-0001');
expect_identity($newEnvelope['schema'] === 'focusa.spec152e.account_promotion_result.v1', 'NEW promotion returns the typed authority envelope');
expect_identity($newEnvelope['customer_resolution'] === 'new', 'NEW creates a fresh EDD customer');
expect_identity($newEnvelope['account_resolution'] === 'new', 'NEW creates a fresh authority account');
expect_identity($newEnvelope['identity_state'] === 'primary', 'NEW identity is primary for the account');
expect_identity($isUuid($newEnvelope['account_uuid']) && $isUuid($newEnvelope['identity_uuid']), 'NEW promotion returns opaque UUID references');
expect_identity(!str_contains(json_encode($newEnvelope, JSON_THROW_ON_ERROR), 'matrix.new@example.invalid'), 'NEW envelope is masked (no raw email)');
$cNew = $counts();
expect_identity($cNew['customers'] === $baseBefore['customers'] + 1
    && $cNew['accounts'] === $baseBefore['accounts'] + 1
    && $cNew['identities'] === $baseBefore['identities'] + 1, 'NEW creates exactly one customer, account, and identity');
expect_identity($cNew['orders'] === $baseBefore['orders'] && $cNew['licenses'] === $baseBefore['licenses'], 'NEW creates zero orders and zero licenses');
$newReg = $registrations->findByUuid($regNew['registration_uuid']);
expect_identity($newReg['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'NEW advances the registration to account_promoted');
expect_identity($newReg['edd_order_id'] === null && $newReg['edd_license_id'] === null && $newReg['node_uuid'] === null, 'NEW identity promotion never attaches order/license/node');

// ── 2. EXISTING: same verified email resolves the same account/identity ──

$regExisting = $createVerified('matrix.new@example.invalid', 'focusa_install_v1', 'existing');
$existingEnvelope = $promoteFixed($regExisting, 'matrix.new@example.invalid', 'existing', 'req-promote-existing-0001', 'idem-promote-existing-0001');
expect_identity($existingEnvelope['customer_resolution'] === 'existing', 'EXISTING resolves the prior EDD customer');
expect_identity($existingEnvelope['account_uuid'] === $newEnvelope['account_uuid'], 'EXISTING resolves the same authority account');
expect_identity($existingEnvelope['identity_uuid'] === $newEnvelope['identity_uuid'], 'EXISTING resolves the same verified identity');
$cExisting = $counts();
expect_identity($cExisting['customers'] === $cNew['customers']
    && $cExisting['accounts'] === $cNew['accounts']
    && $cExisting['identities'] === $cNew['identities'], 'EXISTING creates zero duplicates');

// ── 3. ALIAS: provider dot/plus/case aliases are never merged ──

expect_identity(FocusaSpec152eEmailNormalizer::exact('matrix.new@EXAMPLE.INVALID') === 'matrix.new@example.invalid', 'domain case canonicalizes to the same identity');
expect_identity($identities->findExact('matrix.new@EXAMPLE.INVALID')['identity_uuid'] === $newEnvelope['identity_uuid'], 'domain-case variant resolves the same verified identity');

$regAliasPlus = $createVerified('matrix.new+tag@example.invalid', 'focusa_install_v1', 'alias-plus');
$aliasPlus = $promoteFixed($regAliasPlus, 'matrix.new+tag@example.invalid', 'alias-plus', 'req-promote-aliasplus-0001', 'idem-promote-aliasplus-0001');
expect_identity($aliasPlus['identity_uuid'] !== $newEnvelope['identity_uuid'], 'plus alias is a distinct verified identity (never merged)');
expect_identity($aliasPlus['account_uuid'] !== $newEnvelope['account_uuid'], 'plus alias is a distinct account (never merged)');

$regAliasDot = $createVerified('m.atrix.new@example.invalid', 'focusa_install_v1', 'alias-dot');
$aliasDot = $promoteFixed($regAliasDot, 'm.atrix.new@example.invalid', 'alias-dot', 'req-promote-aliasdot-0001', 'idem-promote-aliasdot-0001');
expect_identity($aliasDot['identity_uuid'] !== $newEnvelope['identity_uuid'], 'dot alias is a distinct verified identity (never merged)');

$regAliasCase = $createVerified('Matrix.New@example.invalid', 'focusa_install_v1', 'alias-case');
$aliasCase = $promoteFixed($regAliasCase, 'Matrix.New@example.invalid', 'alias-case', 'req-promote-aliascase-0001', 'idem-promote-aliascase-0001');
expect_identity($aliasCase['identity_uuid'] !== $newEnvelope['identity_uuid'], 'local-part case variation is a distinct identity (provider-neutral)');
$cAlias = $counts();
expect_identity($cAlias['customers'] === $cExisting['customers'] + 3
    && $cAlias['accounts'] === $cExisting['accounts'] + 3
    && $cAlias['identities'] === $cExisting['identities'] + 3, 'three alias variants create exactly three distinct identities');

// ── 4. WRONG CODE: bounded attempts, no promotion, zero state ──

$regWrong = $registrations->createPending([
    'email' => 'matrix.wrongcode@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'safe_redirect_handle' => 'success-wrong-1',
    'request_id' => 'req-wrongcode-0001',
    'idempotency_key' => 'idem-wrongcode-0001',
]);
$wrongUuid = $regWrong['registration']['registration_uuid'];
$cBeforeWrong = $counts();
expect_identity_throws_code(
    fn() => $registrations->verifyEmail($wrongUuid, '000000', 'req-wrong-verify-0001', 'idem-wrong-verify-0001'),
    'EMAIL_VERIFICATION_FAILED',
    'WRONG CODE is rejected without revealing anything'
);
$wrongRow = $registrations->findByUuid($wrongUuid);
expect_identity((int) $wrongRow['verification_attempts'] === 1, 'wrong code increments the bounded attempt counter');
expect_identity($counts()['customers'] === $cBeforeWrong['customers'] && $counts()['registrations'] === $cBeforeWrong['registrations'], 'WRONG CODE writes zero customer and registration state');

$opaqueClientKey = hash('sha256', 'synthetic-matrix-browser-client');
$maskedWrong = $completeHandler->complete([
    'registration_uuid' => $wrongUuid,
    'verifier' => '999999',
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-masked-wrong-0001',
    'idempotency_key' => 'idem-masked-wrong-0001',
], $facadeRegistry, $opaqueClientKey);
expect_identity(($maskedWrong['error'] ?? '') === 'EMAIL_VERIFICATION_FAILED', 'WRONG CODE through the route returns the enumeration-resistant masked error');
expect_identity_throws_code(
    fn() => $promoteFixed(['registration_uuid' => $wrongUuid], 'matrix.wrongcode@example.invalid', 'wrong-promote', 'req-wrong-promote-0001', 'idem-wrong-promote-0001'),
    'EMAIL_VERIFICATION_REQUIRED',
    'an unverified registration can never be promoted'
);

// ── 5. EXPIRED: challenge and registration TTLs fail closed ──

$expiryNow = '2026-08-08T12:00:00Z';
$expiryClock = static function () use (&$expiryNow): string {
    return $expiryNow;
};
$expiryRegistrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $expiryClock);
$regExpired = $expiryRegistrations->createPending([
    'email' => 'matrix.expired@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'safe_redirect_handle' => 'success-expired-1',
    'request_id' => 'req-expired-0001',
    'idempotency_key' => 'idem-expired-0001',
]);
$expiredUuid = $regExpired['registration']['registration_uuid'];
$cBeforeExpired = $counts();
$expiryNow = '2026-08-08T12:15:01Z'; // challenge TTL (900s) passed; attempt TTL (1800s) not yet.
expect_identity_throws_code(
    fn() => $expiryRegistrations->verifyEmail($expiredUuid, $regExpired['verification_secret'], 'req-expired-verify-0001', 'idem-expired-verify-0001'),
    'EMAIL_VERIFICATION_EXPIRED',
    'an EXPIRED challenge is rejected'
);
$expiryNow = '2026-08-08T12:31:00Z'; // attempt TTL (1800s) passed.
expect_identity_throws_code(
    fn() => $expiryRegistrations->verifyEmail($expiredUuid, $regExpired['verification_secret'], 'req-expired-verify-0002', 'idem-expired-verify-0002'),
    'REGISTRATION_EXPIRED',
    'an EXPIRED registration attempt is rejected'
);
$cleanup = $expiryRegistrations->cleanup('2026-08-08T12:31:00Z');
expect_identity($cleanup['expired'] >= 1, 'expiry cleanup transitions the due pending attempt');
expect_identity($counts()['customers'] === $cBeforeExpired['customers'], 'EXPIRED attempts create zero customer state');

// ── 6. REPLAY: idempotent verification and promotion, single-use verifier ──

$replayCreated = $registrations->createPending([
    'email' => 'matrix.replay@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'safe_redirect_handle' => 'success-replay-1',
    'request_id' => 'req-replay-create-0001',
    'idempotency_key' => 'idem-replay-create-0001',
]);
$replayUuid = $replayCreated['registration']['registration_uuid'];
$replayVerifier = $replayCreated['verification_secret'];
$firstVerify = $registrations->verifyEmail($replayUuid, $replayVerifier, 'req-replay-verify-0001', 'idem-replay-verify-0001');
expect_identity($firstVerify['replayed'] === false, 'first verification is not a replay');
expect_identity($firstVerify['registration']['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'first verification reaches email_verified');
$replayVerify = $registrations->verifyEmail($replayUuid, $replayVerifier, 'req-replay-verify-0001', 'idem-replay-verify-0001');
expect_identity($replayVerify['replayed'] === true, 'identical verification REPLAY returns the bounded result');
expect_identity($replayVerify['registration']['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'verification replay does not re-transition state');
expect_identity_throws_code(
    fn() => $registrations->verifyEmail($replayUuid, $replayVerifier, 'req-replay-verify-0002', 'idem-replay-verify-0002'),
    'EMAIL_VERIFICATION_REQUIRED',
    'a consumed verifier is single-use and cannot be replayed with a new key'
);

$cBeforePromoReplay = $counts();
$promoteReplay = $promoteFixed(['registration_uuid' => $replayUuid], 'matrix.replay@example.invalid', 'replay', 'req-promote-replay-0001', 'idem-promote-replay-0001');
expect_identity($promoteReplay['replayed'] === false && $promoteReplay['customer_resolution'] === 'new', 'first promotion commits the account');
$promoteReplayAgain = $promoteFixed(['registration_uuid' => $replayUuid], 'matrix.replay@example.invalid', 'replay', 'req-promote-replay-0001', 'idem-promote-replay-0001');
expect_identity($promoteReplayAgain['replayed'] === true, 'identical promotion REPLAY returns the stored bounded result');
expect_identity($promoteReplayAgain['account_uuid'] === $promoteReplay['account_uuid'], 'promotion replay returns the same authority account');
$cPromoReplay = $counts();
expect_identity($cPromoReplay['customers'] === $cBeforePromoReplay['customers'] + 1
    && $cPromoReplay['accounts'] === $cBeforePromoReplay['accounts'] + 1
    && $cPromoReplay['identities'] === $cBeforePromoReplay['identities'] + 1, 'REPLAY promotion creates no duplicates');

// ── 7. DUPLICATE: duplicate start with the same idempotency key ──

$cBeforeDup = $counts();
$dupStartInput = [
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'browser',
    'install_channel' => 'source_build',
    'email' => 'matrix.duplicate@example.invalid',
    'request_id' => 'req-dup-start-0001',
    'idempotency_key' => 'idem-dup-start-0001',
    'safe_redirect_handle' => 'success',
];
$dupFirst = $startHandler->start($dupStartInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
$dupSecond = $startHandler->start($dupStartInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_identity(!isset($dupFirst['error']) && !isset($dupSecond['error']), 'DUPLICATE start requests both return the masked envelope');
expect_identity($dupSecond['registration_id'] === $dupFirst['registration_id'], 'DUPLICATE start with the same idempotency key returns the same registration');
expect_identity($dupFirst['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'duplicate start stays at the bounded pending attempt');
$cDup = $counts();
expect_identity($cDup['registrations'] === $cBeforeDup['registrations'] + 1
    && $cDup['customers'] === $cBeforeDup['customers'], 'DUPLICATE creates exactly one bounded registration and zero customers');

// ── 8. RESEND: new bounded attempt, branded delivery, rate-bounded ──

$burstNow = '2026-08-08T20:00:00Z';
$burstClock = static function () use (&$burstNow): string {
    return $burstNow;
};
$burstRegistrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $burstClock, attemptTtl: 86400, verificationTtl: 3600, pollTtl: 3600);
$burstLimiter = new FocusaSpec152eRateLimiter($db, 'wp_', $burstClock, windowSeconds: 60, maxPerWindow: 3, consecutiveMax: 3);
$burstStart = new FocusaSpec152eActivationStartHandler($burstRegistrations, $challengeService, $mailAdapter, $burstLimiter, $burstClock);
$burstKey = hash('sha256', 'synthetic-resend-burst-client');
$cBeforeResend = $counts();
$mailBefore = count($sent);
$resendFirst = $burstStart->start([
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'matrix.resend@example.invalid',
    'request_id' => 'req-resend-0001',
    'idempotency_key' => 'idem-resend-0001',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $burstKey);
expect_identity(!isset($resendFirst['error']) && ($resendFirst['verification_delivery_status'] ?? '') === 'attempted', 'RESEND first attempt delivers the branded challenge');
expect_identity(count($sent) === $mailBefore + 1, 'RESEND sends exactly one branded challenge email');
$resendSecond = $burstStart->start([
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'matrix.resend@example.invalid',
    'request_id' => 'req-resend-0002',
    'idempotency_key' => 'idem-resend-0002',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $burstKey);
expect_identity($resendSecond['registration_id'] !== $resendFirst['registration_id'], 'a RESEND is a new bounded pending attempt, never a new customer');
expect_identity($counts()['customers'] === $cBeforeResend['customers'], 'RESEND writes zero customer state');
$resendThird = $burstStart->start([
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'matrix.resend@example.invalid',
    'request_id' => 'req-resend-0003',
    'idempotency_key' => 'idem-resend-0003',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $burstKey);
expect_identity(!isset($resendThird['error']), 'third resend within the window is still bounded');
$resendFourth = $burstStart->start([
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'matrix.resend@example.invalid',
    'request_id' => 'req-resend-0004',
    'idempotency_key' => 'idem-resend-0004',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $burstKey);
expect_identity(($resendFourth['error'] ?? '') === 'ACTIVATION_REQUEST_ACCEPTED', 'rate-limited RESEND returns the enumeration-resistant masked response');
$cResend = $counts();
expect_identity($cResend['registrations'] === $cBeforeResend['registrations'] + 3
    && $cResend['customers'] === $cBeforeResend['customers'], 'RESEND burst creates exactly three bounded attempts and zero customers');

// ── 9. BOUNCE: hard bounce blocks delivery and never becomes identity ──

$regBounce = $createVerified('matrix.bounce@example.invalid', 'focusa_install_v1', 'bounce');
$bouncePromote = $promoteFixed($regBounce, 'matrix.bounce@example.invalid', 'bounce', 'req-promote-bounce-0001', 'idem-promote-bounce-0001');
$bounceIdentityUuid = $bouncePromote['identity_uuid'];
$bounceOutcome = $consentHandler->recordDeliveryOutcome([
    'identity_uuid' => $bounceIdentityUuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'bounced',
    'bounce_type' => 'hard',
    'occurred_at' => '2026-08-08T01:00:00Z',
    'request_id' => 'req-bounce-outcome-0001',
    'idempotency_key' => 'idem-bounce-outcome-0001',
]);
expect_identity(($bounceOutcome['bounce_state'] ?? '') === 'hard', 'hard BOUNCE is recorded on the verified identity');
expect_identity($consentHandler->canSendTransactional($bounceIdentityUuid) === false, 'hard bounce blocks transactional delivery');
expect_identity($consentHandler->canSendPromotional($bounceIdentityUuid) === false, 'hard bounce blocks promotional delivery');
expect_identity($consentHandler->canVerifyIdentity($bounceIdentityUuid) === false, 'bounce never becomes verified identity capability');
$bouncePromoConsent = $consentHandler->settlePromotionalConsent([
    'identity_uuid' => $bounceIdentityUuid,
    'occurred_at' => '2026-08-08T01:05:00Z',
    'request_id' => 'req-bounce-consent-0001',
    'idempotency_key' => 'idem-bounce-consent-0001',
]);
expect_identity(($bouncePromoConsent['error'] ?? '') === 'EMAIL_DELIVERY_FAILED', 'promotional consent settlement is denied after a hard bounce');

// ── 10. SUPPRESSED: complaint suppresses delivery; consent never gates transactional ──

$regSuppressed = $createVerified('matrix.suppressed@example.invalid', 'focusa_install_v1', 'suppressed');
$suppressedPromote = $promoteFixed($regSuppressed, 'matrix.suppressed@example.invalid', 'suppressed', 'req-promote-suppressed-0001', 'idem-promote-suppressed-0001');
$suppressedIdentityUuid = $suppressedPromote['identity_uuid'];
$complaint = $consentHandler->recordDeliveryOutcome([
    'identity_uuid' => $suppressedIdentityUuid,
    'message_kind' => 'promotional',
    'delivery_status' => 'complained',
    'occurred_at' => '2026-08-08T01:10:00Z',
    'request_id' => 'req-complaint-0001',
    'idempotency_key' => 'idem-complaint-0001',
]);
expect_identity(($complaint['suppression_state'] ?? '') === 'promotional', 'complaint SUPPRESSES promotional delivery');
expect_identity($consentHandler->canSendPromotional($suppressedIdentityUuid) === false, 'suppression blocks promotional delivery');
expect_identity($consentHandler->canSendTransactional($suppressedIdentityUuid) === true, 'transactional consent is never gated by promotional suppression');
expect_identity($consentHandler->canVerifyIdentity($suppressedIdentityUuid) === false, 'suppression blocks verification capability');
$suppressMail = new FocusaSpec152eTransactionalMailAdapter(static fn(string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity): bool => false);
$suppressedDelivery = $suppressMail->sendVerificationChallenge([
    'facade' => $facadeRegistry['facades'][0],
    'to' => 'matrix.suppressed@example.invalid',
    'challenge_kind' => 'otp',
    'otp_code' => '123456',
    'expires_at' => '2026-08-08T02:00:00Z',
    'registration_id' => $regSuppressed['registration_uuid'],
    'product_code' => 'focusa_operator_lifetime_v1',
]);
expect_identity(($suppressedDelivery['delivery_status'] ?? '') === 'suppressed', 'a refused delivery is recorded as suppressed');

// ── 11. CONFLICT: conflicting bindings and paid records enter review ──

expect_identity_throws_code(
    fn() => $identities->storeVerified('matrix.new@example.invalid', [
        'verification_state' => 'mailbox_verified',
        'verified_at' => '2026-08-08T00:40:00Z',
        'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5affff',
        'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5aff01',
        'identity_state' => 'primary',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:40:00Z',
        'promotional_consent_at' => null,
        'promotional_consent_revoked_at' => null,
        'source' => 'matrix.acceptance',
        'migration_evidence' => ['source' => 'spec152e_candidate', 'record' => 'matrix-conflict'],
    ]),
    'EMAIL_IDENTITY_CONFLICT',
    'an existing verified identity never accepts a different account binding'
);

$regWp1 = $createVerified('matrix.wp1@example.invalid', 'focusa_install_v1', 'wp1');
$wp1 = $promotion->promoteVerified([
    'registration_uuid' => $regWp1['registration_uuid'],
    'verified_email' => 'matrix.wp1@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:45:00Z',
    'promotional_consent_at' => null,
    'wordpress_user_id' => 1001,
    'request_id' => 'req-promote-wp1-0001',
    'idempotency_key' => 'idem-promote-wp1-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-wp1'],
]);
expect_identity($isUuid($wp1['account_uuid']), 'WordPress-bound promotion commits an account');
$regWp2 = $createVerified('matrix.wp2@example.invalid', 'focusa_install_v1', 'wp2');
expect_identity_throws_code(
    fn() => $promotion->promoteVerified([
        'registration_uuid' => $regWp2['registration_uuid'],
        'verified_email' => 'matrix.wp2@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:45:00Z',
        'promotional_consent_at' => null,
        'wordpress_user_id' => 1001,
        'request_id' => 'req-promote-wp2-0001',
        'idempotency_key' => 'idem-promote-wp2-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-wp2'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'a WordPress user already bound to another verified account enters merge review'
);

$regStripe1 = $createVerified('matrix.stripe1@example.invalid', 'focusa_install_v1', 'stripe1');
$stripe1 = $promotion->promoteVerified([
    'registration_uuid' => $regStripe1['registration_uuid'],
    'verified_email' => 'matrix.stripe1@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:46:00Z',
    'promotional_consent_at' => null,
    'stripe_customer_id' => 'cus_matrix_0001',
    'request_id' => 'req-promote-stripe1-0001',
    'idempotency_key' => 'idem-promote-stripe1-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-stripe1'],
]);
expect_identity($isUuid($stripe1['account_uuid']), 'Stripe-bound promotion commits an account');
$regStripe2 = $createVerified('matrix.stripe2@example.invalid', 'focusa_install_v1', 'stripe2');
expect_identity_throws_code(
    fn() => $promotion->promoteVerified([
        'registration_uuid' => $regStripe2['registration_uuid'],
        'verified_email' => 'matrix.stripe2@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:46:00Z',
        'promotional_consent_at' => null,
        'stripe_customer_id' => 'cus_matrix_0001',
        'request_id' => 'req-promote-stripe2-0001',
        'idempotency_key' => 'idem-promote-stripe2-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-stripe2'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'a Stripe customer already bound to another verified account enters merge review'
);

// ── 12. LEGACY PAID: verified legacy owner merges with evidence; truth preserved ──

$db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
    VALUES (NULL, 'matrix.legacy@example.invalid', 'Legacy Matrix Customer', 149.00, 1, '', '2026-07-01T00:00:00Z', NULL)");
$legacyCustomer = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
    VALUES ('ORD-M-3001', 'complete', 'sale', '2026-07-01T00:01:00Z', '2026-07-01T00:02:00Z', {$legacyCustomer}, 'matrix.legacy@example.invalid')");
$legacyOrder = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_order_items (order_id, product_id, product_name, quantity)
    VALUES ({$legacyOrder}, 453, 'WPUIAI Pro Lifetime', 1)");
$legacyItem = (int) $db->lastInsertId();
$db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
    VALUES ('fl_matrix_legacy_0001', {$legacyCustomer}, {$legacyOrder}, 453, 'active')");
$legacyLicense = (int) $db->lastInsertId();
$legacyEvidence = ['kind' => 'purchase_evidence', 'source' => 'edd_software_licensing', 'record' => 'matrix-legacy-3001'];

$regLegacy = $createVerified('matrix.legacy@example.invalid', 'focusa_install_v1', 'legacy');
$legacyResolution = $legacy->resolveForActivation([
    'registration_uuid' => $regLegacy['registration_uuid'],
    'verified_email' => 'matrix.legacy@example.invalid',
    'license_key' => 'fl_matrix_legacy_0001',
    'purpose' => 'node_activation',
    'legacy_evidence' => $legacyEvidence,
    'request_id' => 'req-legacy-resolve-0001',
]);
expect_identity($legacyResolution['owner_match'] === true && $legacyResolution['node_activation_allowed'] === true, 'LEGACY PAID verified owner may activate a node');
expect_identity($legacyResolution['evidence_digest'] !== '' && preg_match('/^[a-f0-9]{64}$/D', $legacyResolution['evidence_digest']) === 1, 'legacy resolution is pinned by a bounded evidence digest');
$legacyLicenseBefore = $db->query("SELECT * FROM wp_edd_licenses WHERE id = {$legacyLicense}")->fetch(PDO::FETCH_ASSOC);
$legacyMerge = $promotion->mergeLegacyVerified([
    'registration_uuid' => $regLegacy['registration_uuid'],
    'verified_email' => 'matrix.legacy@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:50:00Z',
    'promotional_consent_at' => null,
    'request_id' => 'req-legacy-merge-0001',
    'idempotency_key' => 'idem-legacy-merge-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'matrix-legacy-merge'],
    'legacy_key' => 'fl_matrix_legacy_0001',
    'legacy_evidence' => $legacyEvidence,
    'prior_purchases' => [['order_id' => $legacyOrder, 'item_id' => $legacyItem, 'license_id' => $legacyLicense]],
]);
expect_identity($legacyMerge['legacy_merge'] === true, 'LEGACY PAID merge is marked');
expect_identity($legacyMerge['customer_resolution'] === 'existing', 'LEGACY PAID merge resolves the existing EDD customer');
expect_identity($legacyMerge['linked_orders'] === [$legacyOrder] && $legacyMerge['linked_licenses'] === [$legacyLicense], 'LEGACY PAID merge links the evidence-backed order and license');
$legacyLicenseAfter = $db->query("SELECT * FROM wp_edd_licenses WHERE id = {$legacyLicense}")->fetch(PDO::FETCH_ASSOC);
expect_identity($legacyLicenseAfter === $legacyLicenseBefore && (string) $legacyLicenseAfter['status'] === 'active', 'LEGACY PAID posture is preserved; the paid license is never downgraded');

// ── 13. PRIOR EVALUATION: no local bypass, no unverified entitlement ──

$evalPending = $registrations->createPending([
    'email' => 'matrix.eval@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'safe_redirect_handle' => 'success-eval-1',
    'request_id' => 'req-eval-0001',
    'idempotency_key' => 'idem-eval-0001',
]);
$cBeforeEval = $counts();
expect_identity_throws_code(
    fn() => $promoteFixed(['registration_uuid' => $evalPending['registration']['registration_uuid']], 'matrix.eval@example.invalid', 'eval', 'req-eval-promote-0001', 'idem-eval-promote-0001'),
    'EMAIL_VERIFICATION_REQUIRED',
    'a PRIOR Evaluation from an unverified email can never promote or issue entitlement'
);
expect_identity($counts()['customers'] === $cBeforeEval['customers'] && $counts()['registrations'] === $cBeforeEval['registrations'], 'a PRIOR Evaluation attempt writes zero customer/registration state');

$regEvalBound = $createVerified('matrix.eval.bound@example.invalid', 'focusa_install_v1', 'eval-bound');
expect_identity_throws_code(
    fn() => $promoteFixed(['registration_uuid' => $regEvalBound['registration_uuid']], 'matrix.eval.other@example.invalid', 'eval-bound', 'req-eval-bound-0001', 'idem-eval-bound-0001'),
    'ACCOUNT_EMAIL_MISMATCH',
    'a verified registration is bound to exactly the verified email (no cross-email bypass)'
);
$cEval = $counts();
expect_identity($cEval['orders'] === $cBeforeEval['orders'] && $cEval['licenses'] === $cBeforeEval['licenses'], 'PRIOR Evaluation surfaces never create EDD order or license state');
expect_identity($cEval['customers'] === $cBeforeEval['customers'], 'the failed PRIOR Evaluation bindings write zero customer state');

// ── 14. FACADE-SWITCH: cross-facade verification and spoofs fail closed ──

$regSwitch = $registrations->createPending([
    'email' => 'matrix.switch@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'safe_redirect_handle' => 'success-switch-1',
    'request_id' => 'req-switch-0001',
    'idempotency_key' => 'idem-switch-0001',
]);
$switchUuid = $regSwitch['registration']['registration_uuid'];
$cBeforeSwitch = $counts();
$crossFacade = $completeHandler->complete([
    'registration_uuid' => $switchUuid,
    'verifier' => $regSwitch['verification_secret'],
    'facade_id' => 'uiai_engine_v1',
    'origin' => 'https://engine.focusa.dev',
    'request_id' => 'req-switch-verify-0001',
    'idempotency_key' => 'idem-switch-verify-0001',
], $facadeRegistry, $opaqueClientKey);
expect_identity(($crossFacade['error'] ?? '') === 'EMAIL_VERIFICATION_REQUIRED', 'FACADE-SWITCH verification is denied (facade-bound registration)');
$switchRow = $registrations->findByUuid($switchUuid);
expect_identity($switchRow['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'a cross-facade attempt leaves the registration unchanged');
expect_identity($counts()['customers'] === $cBeforeSwitch['customers'], 'FACADE-SWITCH writes zero customer state');

$spoofStart = $startHandler->start([
    'facade_id' => 'focusa_spoof_v1',
    'origin' => 'https://spoof.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'browser',
    'install_channel' => 'source_build',
    'email' => 'matrix.spoof@example.invalid',
    'request_id' => 'req-spoof-0001',
    'idempotency_key' => 'idem-spoof-0001',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_identity(($spoofStart['error'] ?? '') === 'FACADE_ORIGIN_DENIED', 'a FACADE spoof origin is denied at the route boundary');

$wrongProductStart = $startHandler->start([
    'facade_id' => 'uiai_engine_v1',
    'origin' => 'https://engine.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'browser',
    'install_channel' => 'source_build',
    'email' => 'matrix.wrongproduct@example.invalid',
    'request_id' => 'req-wrongproduct-0001',
    'idempotency_key' => 'idem-wrongproduct-0001',
    'safe_redirect_handle' => 'success',
], $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_identity(($wrongProductStart['error'] ?? '') === 'FACADE_PRODUCT_DENIED', 'a WRONG PRODUCT is denied at the facade boundary (no cross-product lease)');
expect_identity($counts()['customers'] === $cBeforeSwitch['customers'], 'facade spoof and wrong-product attempts write zero customer state');

// Signed facade protocol: proper request verifies; origin/product/replay spoofs fail closed.
$nowEpoch = time();
$protocolRequest = FocusaSpec152eFacadeProtocol::signRequest([
    'credential_id' => 'facade-cred-1',
    'timestamp' => (string) $nowEpoch,
    'nonce' => 'nonce-matrix-0001',
    'request_id' => 'req-protocol-0001',
    'idempotency_key' => 'idem-protocol-0001',
    'registration_id' => $switchUuid,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'action' => 'activation_verify',
    'redirect_handle' => 'success',
    'continuation_token' => '',
    'body_sha256' => str_repeat('a', 64),
], 'facade-cred-1', 'matrix-credential-secret-1');
$credentialResolver = static fn(string $id): ?array => $id === 'facade-cred-1'
    ? ['facade_id' => 'focusa_install_v1', 'credential' => 'matrix-credential-secret-1', 'active' => true]
    : null;
$nonceStore = [];
$consumeNonce = static function (string $credentialId, string $facadeId, string $nonce, int $timestamp) use (&$nonceStore): bool {
    $key = $credentialId . ':' . $facadeId . ':' . $nonce;
    if (isset($nonceStore[$key])) {
        return false;
    }
    $nonceStore[$key] = true;
    return true;
};
$protocolOk = FocusaSpec152eFacadeProtocol::verifyRequest($protocolRequest, $facadeRegistry, $credentialResolver, $consumeNonce, $nowEpoch);
expect_identity(($protocolOk['ok'] ?? false) === true, 'a properly signed facade request verifies end to end');
$protocolReplay = FocusaSpec152eFacadeProtocol::verifyRequest($protocolRequest, $facadeRegistry, $credentialResolver, $consumeNonce, $nowEpoch);
expect_identity(($protocolReplay['ok'] ?? true) === false && ($protocolReplay['error'] ?? '') === 'FACADE_REPLAY_DENIED', 'a replayed signed facade request is denied');
$originSpoof = $protocolRequest;
$originSpoof['origin'] = 'https://spoof.focusa.dev';
$originSpoof['nonce'] = 'nonce-matrix-0002';
$originSpoof['signature'] = FocusaSpec152eFacadeProtocol::signRequest($originSpoof + ['credential_id' => 'facade-cred-1'], 'facade-cred-1', 'matrix-credential-secret-1')['signature'];
$protocolOriginSpoof = FocusaSpec152eFacadeProtocol::verifyRequest($originSpoof, $facadeRegistry, $credentialResolver, $consumeNonce, $nowEpoch);
expect_identity(($protocolOriginSpoof['ok'] ?? true) === false && ($protocolOriginSpoof['error'] ?? '') === 'FACADE_ORIGIN_DENIED', 'a signed FACADE origin spoof is denied');
$productSpoof = $protocolRequest;
$productSpoof['product_code'] = 'future_operator_v1';
$productSpoof['nonce'] = 'nonce-matrix-0003';
$productSpoof['signature'] = FocusaSpec152eFacadeProtocol::signRequest($productSpoof + ['credential_id' => 'facade-cred-1'], 'facade-cred-1', 'matrix-credential-secret-1')['signature'];
$protocolProductSpoof = FocusaSpec152eFacadeProtocol::verifyRequest($productSpoof, $facadeRegistry, $credentialResolver, $consumeNonce, $nowEpoch);
expect_identity(($protocolProductSpoof['ok'] ?? true) === false && ($protocolProductSpoof['error'] ?? '') === 'FACADE_PRODUCT_DENIED', 'a signed FACADE product spoof is denied');

// ── 15. Zero unverified creation + redaction gate ─────────────────────

$cFinal = $counts();
// Verified promotions creating a customer+account+identity: new, alias-plus,
// alias-dot, alias-case, replay, bounce, suppressed, wp1, stripe1 (9). The legacy
// merge resolves the existing EDD fixture customer but creates one more
// account+identity, so accounts/identities total 10 and customers 9 + 1 fixture = 10.
$expectedAccounts = 10;
$expectedCustomers = 10;
$expectedIdentities = $expectedAccounts;
expect_identity($cFinal['accounts'] === $expectedAccounts, 'exactly the verified promotions created authority accounts');
expect_identity($cFinal['identities'] === $expectedIdentities, 'every authority account has exactly one verified email identity');
expect_identity($cFinal['customers'] === $expectedCustomers, 'customers equal verified promotions plus the synthetic legacy fixture');
expect_identity($cFinal['orders'] === 1 && $cFinal['licenses'] === 1, 'identity surfaces created zero orders and zero licenses (only the synthetic fixture exists)');
expect_identity($cFinal['links'] === 1, 'only the evidence-backed legacy merge created a purchase link');

foreach ($db->query('SELECT verified_at, identity_state FROM wp_wpuiai_email_identities')->fetchAll(PDO::FETCH_ASSOC) as $row) {
    expect_identity(($row['verified_at'] ?? '') !== '' && in_array($row['identity_state'] ?? '', ['primary', 'linked'], true), 'every stored email identity is mailbox-verified (no unverified email identity)');
}
$unverifiedWithRefs = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations
    WHERE state <> 'account_promoted' AND (account_uuid IS NOT NULL OR edd_customer_id IS NOT NULL)")->fetchColumn();
expect_identity($unverifiedWithRefs === 0, 'no unverified registration carries customer or account references');
foreach ($db->query("SELECT edd_order_id, edd_license_id, node_uuid FROM wp_wpuiai_activation_registrations WHERE state = 'account_promoted'")->fetchAll(PDO::FETCH_ASSOC) as $row) {
    expect_identity($row['edd_order_id'] === null && $row['edd_license_id'] === null && $row['node_uuid'] === null, 'identity promotion never attaches order/license/node references');
}
foreach ($productRegistry['protected_offers'] as $offer) {
    expect_identity(($offer['checkout_enabled'] ?? true) === false, 'protected offers are never checkout-enabled client-side');
}
expect_identity(($productRegistry['verified_no_license']['checkout_enabled'] ?? true) === false, 'the verified-no-license posture has no client checkout');
expect_identity(($productRegistry['verified_no_license']['edd_software_license_key'] ?? true) === false, 'verified_no_license never carries an EDD key');
expect_identity(($facadeRegistry['authority']['entitlement_issuance'] ?? '') === 'forbidden', 'facades never issue entitlement');
expect_identity(($facadeRegistry['authority']['wildcard_authority'] ?? '') === 'forbidden', 'no wildcard facade authority');

$redactedEmails = [
    'matrix.new@example.invalid',
    'matrix.new+tag@example.invalid',
    'm.atrix.new@example.invalid',
    'Matrix.New@example.invalid',
    'matrix.replay@example.invalid',
    'matrix.bounce@example.invalid',
    'matrix.suppressed@example.invalid',
    'matrix.wp1@example.invalid',
    'matrix.stripe1@example.invalid',
    'matrix.legacy@example.invalid',
    'matrix.resend@example.invalid',
    'matrix.duplicate@example.invalid',
    'matrix.eval@example.invalid',
];
$summary = json_encode([
    'schema' => 'focusa.spec152e.identity_acceptance_matrix.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'accounts' => $cFinal['accounts'],
    'customers' => $cFinal['customers'],
    'identities' => $cFinal['identities'],
    'orders' => $cFinal['orders'],
    'licenses' => $cFinal['licenses'],
    'purchase_links' => $cFinal['links'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
foreach ($redactedEmails as $email) {
    expect_identity(!str_contains($summary, $email), 'redacted summary never contains a raw email');
}
expect_identity(!str_contains($summary, 'fl_matrix_legacy_0001'), 'redacted summary never contains a raw license key');
expect_identity(!str_contains($summary, 'matrix-credential-secret-1') && !str_contains($summary, 'verification_secret'), 'redacted summary never contains secrets or verifiers');

fwrite(STDOUT, $summary . "\n");
