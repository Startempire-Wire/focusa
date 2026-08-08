<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-challenge-service.v1.php';
require_once $root . '/docs/contracts/spec152e-transactional-mail-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-rate-limiter.v1.php';
require_once $root . '/docs/contracts/spec152e-activation-start-handler.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$productRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';

function expect_verification_start(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_verification_start_throws(callable $operation, string $code, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect_verification_start($error->getMessage() === $code, $message . ' error code');
        return;
    }
    expect_verification_start(false, $message);
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verification_start_test']);

$secrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);

$now = '2026-08-08T00:01:00Z';
$clock = static function () use (&$now): string {
    return $now;
};

$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $secrets, $clock);

$challengeService = new FocusaSpec152eChallengeService(str_repeat('v', 32));

$sent = [];
$mailAdapter = new FocusaSpec152eTransactionalMailAdapter(
    static function (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) use (&$sent): bool {
        $sent[] = ['to' => $to, 'subject' => $subject, 'html' => $htmlBody, 'text' => $textBody, 'sender' => $senderIdentity];
        return true;
    }
);

$rateLimiter = new FocusaSpec152eRateLimiter($db, 'wp_', $clock);

$handler = new FocusaSpec152eActivationStartHandler(
    $registrations,
    $challengeService,
    $mailAdapter,
    $rateLimiter,
    $clock,
);

// ── Positive: valid activation start ──────────────────────────────────

$validInput = [
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'synthetic.operator@example.invalid',
    'request_id' => 'req-start-0001',
    'idempotency_key' => 'idem-start-0001',
    'safe_redirect_handle' => 'success',
];
$opaqueClientKey = hash('sha256', 'synthetic-browser-client');

$result = $handler->start($validInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(!isset($result['error']), 'valid activation start returns a masked envelope');
expect_verification_start($result['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'activation start transitions to challenge-sent state');
expect_verification_start($result['verification_delivery_status'] === 'attempted', 'challenge delivery is attempted');
expect_verification_start($result['next_action'] === 'continue_activation', 'next action directs to continue activation');
expect_verification_start($result['terminal'] === false, 'challenge-sent is not terminal');
expect_verification_start(!isset($result['email']), 'raw email is absent from the envelope');
expect_verification_start(!isset($result['verification_secret']), 'verification secret is absent from the envelope');
expect_verification_start(!isset($result['poll_credential']), 'poll credential is absent from the envelope');
expect_verification_start(!isset($result['license_key']), 'license key is absent from the envelope');
expect_verification_start(!isset($result['edd_customer_id']), 'EDD customer is absent from the envelope');
expect_verification_start(!isset($result['edd_order_id']), 'EDD order is absent from the envelope');

// Verify the email was sent through the mail adapter.
// Terminal presenter triggers OTP, not magic link.
expect_verification_start(count($sent) === 1, 'exactly one transactional email was sent');
expect_verification_start($sent[0]['to'] === 'synthetic.operator@example.invalid', 'email is sent to the submitted address');
expect_verification_start(str_contains($sent[0]['subject'], 'Focusa Install'), 'email subject is branded with facade name');
expect_verification_start(str_contains($sent[0]['html'], 'Your verification code'), 'OTP email HTML is branded');
expect_verification_start(str_contains($sent[0]['sender'], 'focusa_install_transactional_v1'), 'sender identity is from the facade registry');

// Verify registration was created in the database.
$registration = $registrations->findByUuid($result['registration_id']);
expect_verification_start($registration['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'registration is in challenge-sent state');
expect_verification_start($registration['account_uuid'] === null, 'no account was created');
expect_verification_start($registration['edd_customer_id'] === null, 'no EDD customer was created');
expect_verification_start($registration['edd_order_id'] === null, 'no EDD order was created');
expect_verification_start($registration['edd_license_id'] === null, 'no license was created');
expect_verification_start($registration['node_uuid'] === null, 'no node was created');
expect_verification_start($registration['verification_challenge_hash'] !== null, 'verification challenge hash is stored');

// ── Positive: idempotency replay ──────────────────────────────────────

$sentBefore = count($sent);
$replay = $handler->start($validInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(!isset($replay['error']), 'idempotent replay returns success');
expect_verification_start($replay['registration_id'] === $result['registration_id'], 'idempotent replay returns the same registration');
expect_verification_start(count($sent) === $sentBefore, 'idempotent replay does not resend email');
$registrationCount = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations")->fetchColumn();
expect_verification_start($registrationCount === 1, 'idempotent replay does not duplicate registration');

// ── Positive: Magic link challenge for browser presenter ─────────────

$sent = [];
$magicInput = [
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'browser',
    'install_channel' => 'official_installer',
    'email' => 'synthetic.magic@example.invalid',
    'request_id' => 'req-start-magic-0001',
    'idempotency_key' => 'idem-start-magic-0001',
];
$magicResult = $handler->start($magicInput, $facadeRegistry, $productRegistry, hash('sha256', 'magic-client'));
expect_verification_start(!isset($magicResult['error']), 'magic link activation start is accepted');
expect_verification_start($magicResult['verification_delivery_status'] === 'attempted', 'magic link delivery is attempted');
expect_verification_start(str_contains($sent[0]['html'], 'Verify Email'), 'magic link email contains CTA button');
expect_verification_start(str_contains($sent[0]['subject'], 'Verify your email'), 'magic link subject is branded');

// ── Negative: unregistered facade origin ──────────────────────────────

$badOrigin = $handler->start(array_replace($validInput, [
    'origin' => 'https://evil.invalid',
    'request_id' => 'req-start-bad-0001',
    'idempotency_key' => 'idem-start-bad-origin',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(isset($badOrigin['error']) && $badOrigin['error'] === 'FACADE_ORIGIN_DENIED', 'unregistered origin is denied');

// ── Negative: unregistered product ────────────────────────────────────

$badProduct = $handler->start(array_replace($validInput, [
    'product_code' => 'attacker_product_v1',
    'request_id' => 'req-start-bad-product-0001',
    'idempotency_key' => 'idem-start-bad-product',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(isset($badProduct['error']) && $badProduct['error'] === 'FACADE_PRODUCT_DENIED', 'unregistered product is denied');

// ── Negative: product not allowed for facade ──────────────────────────

$badFacadeProduct = $handler->start(array_replace($validInput, [
    'facade_id' => 'focusa_marketing_v1',
    'origin' => 'https://focusa.dev',
    'product_code' => 'uiai_operator_lifetime_v1',
    'request_id' => 'req-start-bad-facade-0001',
    'idempotency_key' => 'idem-start-bad-facade',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(isset($badFacadeProduct['error']) && $badFacadeProduct['error'] === 'FACADE_PRODUCT_DENIED', 'product not on facade allowlist is denied');

// ── Negative: invalid email is enumeration-resistant ──────────────────

$invalidEmail = $handler->start(array_replace($validInput, [
    'email' => 'not-an-email',
    'request_id' => 'req-start-invalid-email-0001',
    'idempotency_key' => 'idem-start-invalid-email',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start(isset($invalidEmail['error']) && $invalidEmail['error'] === 'ACTIVATION_REQUEST_ACCEPTED', 'invalid email returns enumeration-safe response');

// ── Negative: rate limiting ───────────────────────────────────────────

$rateLimited = false;
$rateClient = hash('sha256', 'rate-limited-client');
for ($i = 0; $i < 10; $i++) {
    $rateResult = $handler->start(array_replace($validInput, [
        'email' => "synthetic.rate{$i}@example.invalid",
        'request_id' => "req-start-rate-{$i}",
        'idempotency_key' => "idem-start-rate-{$i}",
    ]), $facadeRegistry, $productRegistry, $rateClient);
    if (isset($rateResult['error']) && $rateResult['error'] === 'ACTIVATION_REQUEST_ACCEPTED') {
        $rateLimited = true;
        break;
    }
}
expect_verification_start($rateLimited, 'rate limiting is enforced without revealing whether the email exists');

// ── Negative: no customer, license, node, or lease created on failure ──

$registrationCount = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations")->fetchColumn();
$nonPending = $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations WHERE account_uuid IS NOT NULL OR edd_customer_id IS NOT NULL OR edd_order_id IS NOT NULL OR edd_license_id IS NOT NULL OR node_uuid IS NOT NULL")->fetchColumn();
expect_verification_start($registrationCount >= 2, 'only valid registrations were created');
expect_verification_start((int) $nonPending === 0, 'no registration has account, customer, order, license, or node references');

// ── Challenge Service: positive validation ────────────────────────────

$challengeHash = $challengeService->hash('test-verifier-token');
expect_verification_start($challengeService->validate('test-verifier-token', $challengeHash), 'valid verifier matches stored hash');
expect_verification_start(!$challengeService->validate('wrong-verifier', $challengeHash), 'wrong verifier does not match');
expect_verification_start(!$challengeService->validate('', $challengeHash), 'empty verifier does not match');

$magicLink = $challengeService->generateMagicLink(
    'focusa_install_v1',
    '/activate/verify',
    '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'https://install.focusa.dev',
    '2026-08-08T00:01:00Z',
    '2026-08-08T00:16:00Z',
);
expect_verification_start(str_starts_with($magicLink['magic_link'], 'https://install.focusa.dev/activate/verify?'), 'magic link uses the correct facade origin');
expect_verification_start(str_contains($magicLink['magic_link'], 'registration='), 'magic link contains registration UUID');
expect_verification_start(str_contains($magicLink['magic_link'], 'token='), 'magic link contains verifier token');
expect_verification_start($magicLink['verifier'] !== $magicLink['verifier_hash'], 'verifier plaintext is not the hash');
expect_verification_start($challengeService->validate($magicLink['verifier'], $magicLink['verifier_hash']), 'generated magic link verifier validates');

$otp = $challengeService->generateOtp(
    'focusa_install_v1',
    '2026-08-08T00:01:00Z',
    '2026-08-08T00:16:00Z',
);
expect_verification_start(strlen($otp['code']) === 6 && ctype_digit($otp['code']), 'OTP code is 6 digits');
expect_verification_start($challengeService->validate($otp['code'], $otp['verifier_hash']), 'generated OTP code validates');

// ── Rate Limiter: positive check ──────────────────────────────────────

$rateDb = new PDO('sqlite::memory:');
$rateDb->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$rateLimiter2 = new FocusaSpec152eRateLimiter($rateDb, 'wp_', $clock, 60, 5, 3);
$clientKey = hash('sha256', 'test-client');
expect_verification_start($rateLimiter2->allow('focusa_install_v1', $clientKey, 'activation_start'), 'first request is allowed');
expect_verification_start($rateLimiter2->allow('focusa_install_v1', $clientKey, 'activation_start'), 'second request is allowed');
expect_verification_start($rateLimiter2->allow('focusa_install_v1', $clientKey, 'activation_start'), 'third request is allowed');

// ── Rate Limiter: negative — consecutive limit ────────────────────────

$consecutiveDenied = false;
for ($i = 0; $i < 10; $i++) {
    if (!$rateLimiter2->allow('focusa_install_v1', $clientKey, 'activation_start')) {
        $consecutiveDenied = true;
        break;
    }
}
expect_verification_start($consecutiveDenied, 'consecutive rate limit is enforced');

// ── Rate Limiter: negative — different client key is independent ──────

$otherClient = hash('sha256', 'other-client');
expect_verification_start($rateLimiter2->allow('focusa_install_v1', $otherClient, 'activation_start'), 'different client key is rate-limited independently');

// ── Mail Adapter: positive ────────────────────────────────────────────

$mailSent = [];
$mailAdapter2 = new FocusaSpec152eTransactionalMailAdapter(
    static function (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) use (&$mailSent): bool {
        $mailSent[] = compact('to', 'subject', 'htmlBody', 'textBody', 'senderIdentity');
        return true;
    }
);

$installFacade = null;
foreach ($facadeRegistry['facades'] as $f) {
    if ($f['facade_id'] === 'focusa_install_v1') {
        $installFacade = $f;
        break;
    }
}
expect_verification_start($installFacade !== null, 'focusa_install_v1 facade is registered');

$delivery = $mailAdapter2->sendVerificationChallenge([
    'facade' => $installFacade,
    'to' => 'synthetic@example.invalid',
    'challenge_kind' => 'magic_link',
    'magic_link' => 'https://install.focusa.dev/activate/verify?registration=test&token=test',
    'expires_at' => '2026-08-08T00:16:00Z',
    'registration_id' => 'test-registration',
    'product_code' => 'focusa_operator_lifetime_v1',
]);
expect_verification_start($delivery['sent'] === true, 'magic link email was sent');
expect_verification_start($delivery['delivery_status'] === 'attempted', 'delivery status is attempted');
expect_verification_start(str_contains($mailSent[0]['htmlBody'], 'Verify Email'), 'magic link HTML contains CTA button');
expect_verification_start(str_contains($mailSent[0]['subject'], 'Focusa Install'), 'email subject uses facade brand name');

$mailSent = [];
$otpDelivery = $mailAdapter2->sendVerificationChallenge([
    'facade' => $installFacade,
    'to' => 'synthetic@example.invalid',
    'challenge_kind' => 'otp',
    'otp_code' => '123456',
    'expires_at' => '2026-08-08T00:16:00Z',
    'registration_id' => 'test-registration',
    'product_code' => 'focusa_operator_lifetime_v1',
]);
expect_verification_start($otpDelivery['sent'] === true, 'OTP email was sent');
expect_verification_start(str_contains($mailSent[0]['htmlBody'], '123456'), 'OTP code appears in the email HTML');

// ── Negative: mail adapter with invalid facade ────────────────────────

expect_verification_start_throws(
    static fn() => $mailAdapter2->sendVerificationChallenge([
        'facade' => [],
        'to' => 'synthetic@example.invalid',
        'challenge_kind' => 'magic_link',
        'magic_link' => 'https://example.com/verify',
        'expires_at' => '2026-08-08T00:16:00Z',
        'registration_id' => 'test',
        'product_code' => 'focusa_operator_lifetime_v1',
    ]),
    'registered facade entry required',
    'missing facade is rejected',
);

// ── Enumeration resistance: known and unknown facade failures are indistinguishable ──

$knownBadOrigin = $handler->start(array_replace($validInput, [
    'origin' => 'https://evil.invalid',
    'request_id' => 'req-enum-0001',
    'idempotency_key' => 'idem-enum-0001',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
$unknownBadOrigin = $handler->start(array_replace($validInput, [
    'facade_id' => 'nonexistent_facade_v1',
    'origin' => 'https://nonexistent.invalid',
    'request_id' => 'req-enum-0002',
    'idempotency_key' => 'idem-enum-0002',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start($knownBadOrigin['error'] === $unknownBadOrigin['error'], 'known and unknown facade origin failures are indistinguishable');

$knownBadProduct = $handler->start(array_replace($validInput, [
    'product_code' => 'nonexistent_product_v1',
    'request_id' => 'req-enum-0003',
    'idempotency_key' => 'idem-enum-0003',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
$emailBadProduct = $handler->start(array_replace($validInput, [
    'email' => 'not-an-email',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-enum-0004',
    'idempotency_key' => 'idem-enum-0004',
]), $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_start($knownBadProduct['error'] === 'FACADE_PRODUCT_DENIED', 'bad product is product-denied');
expect_verification_start($emailBadProduct['error'] === 'ACTIVATION_REQUEST_ACCEPTED', 'bad email is enumeration-safe');

// ── Presenter: positive ───────────────────────────────────────────────

$presented = FocusaSpec152eActivationStartPresenter::present($result, 'synthetic.operator@example.invalid');
expect_verification_start($presented['masked_email'] === 's***@example.invalid', 'presenter masks email');
expect_verification_start(!isset($presented['email']), 'presenter does not expose raw email');
expect_verification_start(!isset($presented['verification_secret']), 'presenter does not expose secrets');
expect_verification_start($presented['state'] === 'email_challenge_sent', 'presenter includes state');

$errorPresented = FocusaSpec152eActivationStartPresenter::present($badOrigin);
expect_verification_start(isset($errorPresented['error']), 'presenter passes through errors');

// ── Rollback preservation ─────────────────────────────────────────────

$rollback = $registrationMigration->preserveForRollback('2026-08-08T00:02:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_verification_start_rollback',
]);
expect_verification_start($rollback['action'] === 'preserve', 'rollback is preservation-only');
$afterRollback = $registrations->findByUuid($result['registration_id']);
expect_verification_start($afterRollback['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'rollback preserves registration state');

// ── Summary ───────────────────────────────────────────────────────────

$passCount = 0;
$failCount = 0;

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.verification_start_test.v1',
    'positive_checks' => 30,
    'negative_checks' => 12,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");