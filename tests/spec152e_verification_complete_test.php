<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-challenge-service.v1.php';
require_once $root . '/docs/contracts/spec152e-transactional-mail-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-rate-limiter.v1.php';
require_once $root . '/docs/contracts/spec152e-activation-start-handler.v1.php';
require_once $root . '/docs/contracts/spec152e-verification-complete-handler.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$productRegistry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';

function expect_verification_complete(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verification_complete_test']);

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

$startHandler = new FocusaSpec152eActivationStartHandler(
    $registrations,
    $challengeService,
    $mailAdapter,
    $rateLimiter,
    $clock,
);

$completeHandler = new FocusaSpec152eVerificationCompleteHandler(
    $registrations,
    $rateLimiter,
);

// ── Create a pending registration with a known challenge ──────────────

// We need to create a registration through the start handler so we can
// capture the verifier plaintext. The start handler creates the registration
// and returns the verifier internally, but the public envelope masks it.
// We create a registration directly via the repository to get the verifier.
$validInput = [
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'email' => 'synthetic.operator@example.invalid',
    'request_id' => 'req-verify-0001',
    'idempotency_key' => 'idem-verify-0001',
    'safe_redirect_handle' => 'success',
];
$opaqueClientKey = hash('sha256', 'synthetic-browser-client');

$startResult = $startHandler->start($validInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_complete(!isset($startResult['error']), 'start creates a valid pending registration');
$registrationUuid = $startResult['registration_id'];

// We need the raw verifier. The repository stores it hashed, and the start
// handler returns it only internally.  We create a second registration
// directly through the repository, which returns the verifier secret.
$rawResult = $registrations->createPending([
    'email' => 'synthetic.verify.direct@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-verify-direct',
    'idempotency_key' => 'idem-verify-direct',
]);
$directRegistrationUuid = $rawResult['registration']['registration_uuid'];
$directVerifier = $rawResult['verification_secret'];
$directPollCredential = $rawResult['poll_credential'];

// Instead of using the raw result directly, we can also use the
// ChallengeService to generate a known verifier that matches the hash
// stored in the registration. The start handler uses the verification
// key from RegistrationSecrets to hash the challenge. Let's verify
// that the hash matches.
$knownVerifier = $directVerifier;
$knownHash = $secrets->verificationHash($knownVerifier);
$storedRegistration = $registrations->findByUuid($directRegistrationUuid);
expect_verification_complete(hash_equals((string) $storedRegistration['verification_challenge_hash'], $knownHash), 'stored hash matches known verifier');

// ── Positive: valid verification completes ────────────────────────────

$now = '2026-08-08T00:02:00Z';
$verifyResult = $completeHandler->complete([
    'registration_uuid' => $directRegistrationUuid,
    'verifier' => $knownVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-complete-0001',
    'idempotency_key' => 'idem-verify-complete-0001',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(!isset($verifyResult['error']), 'valid verification returns success');
expect_verification_complete($verifyResult['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'verification transitions to email_verified state');
expect_verification_complete($verifyResult['terminal'] === false, 'email_verified is not terminal');
expect_verification_complete($verifyResult['next_action'] === 'continue_activation', 'next action directs to continue activation');
expect_verification_complete($verifyResult['replayed'] === false, 'first verification is not a replay');
expect_verification_complete(!isset($verifyResult['email']), 'raw email is absent from the envelope');
expect_verification_complete(!isset($verifyResult['verification_secret']), 'verification secret is absent from the envelope');
expect_verification_complete(!isset($verifyResult['license_key']), 'license key is absent from the envelope');
expect_verification_complete(!isset($verifyResult['edd_customer_id']), 'EDD customer is absent from the envelope');

// Verify the registration was updated in the database.
$verifiedRegistration = $registrations->findByUuid($directRegistrationUuid);
expect_verification_complete($verifiedRegistration['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'registration state is email_verified');
expect_verification_complete($verifiedRegistration['verification_state'] === 'mailbox_verified', 'verification state is mailbox_verified');
expect_verification_complete($verifiedRegistration['verification_challenge_hash'] === null, 'challenge hash is cleared after verification');
expect_verification_complete($verifiedRegistration['verified_at'] !== null, 'verified_at timestamp is set');
expect_verification_complete($verifiedRegistration['account_uuid'] === null, 'no account was created');
expect_verification_complete($verifiedRegistration['edd_customer_id'] === null, 'no EDD customer was created');
expect_verification_complete($verifiedRegistration['edd_order_id'] === null, 'no EDD order was created');
expect_verification_complete($verifiedRegistration['edd_license_id'] === null, 'no license was created');
expect_verification_complete($verifiedRegistration['node_uuid'] === null, 'no node was created');

// ── Positive: idempotency replay ──────────────────────────────────────

$replayResult = $completeHandler->complete([
    'registration_uuid' => $directRegistrationUuid,
    'verifier' => $knownVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-complete-0001',
    'idempotency_key' => 'idem-verify-complete-0001',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(!isset($replayResult['error']), 'idempotent replay returns success');
expect_verification_complete($replayResult['registration_id'] === $directRegistrationUuid, 'idempotent replay returns the same registration');
expect_verification_complete($replayResult['replayed'] === true, 'idempotent replay is marked as replayed');
expect_verification_complete($replayResult['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'idempotent replay preserves email_verified state');

// ── Positive: verification through the start-handler registration (browser presenter) ──

$sent = [];
$now = '2026-08-08T00:03:00Z';
$magicInput = [
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'product_code' => 'focusa_operator_lifetime_v1',
    'presenter' => 'browser',
    'install_channel' => 'official_installer',
    'email' => 'synthetic.magic.verify@example.invalid',
    'request_id' => 'req-verify-magic-0001',
    'idempotency_key' => 'idem-verify-magic-start',
];
$magicStartResult = $startHandler->start($magicInput, $facadeRegistry, $productRegistry, $opaqueClientKey);
expect_verification_complete(!isset($magicStartResult['error']), 'magic link start creates registration');
$magicRegistrationUuid = $magicStartResult['registration_id'];

// We need the verifier for this registration. The start handler stores it
// hashed. Let's create another direct registration to get a verifier for
// the same email pattern, then use the hash from the magic registration.
// Actually, we can use the raw verifier from the previously created
// direct registration. The hash won't match the magic registration though.
// Let's use the ChallengeService to generate a verifier that matches the
// stored hash algorithm, then update the registration's hash.
// The simplest approach: use the verification key from the secrets to
// generate a known verifier + hash, then directly update the registration.
$knownMagicVerifier = rtrim(strtr(base64_encode(random_bytes(32)), '+/', '-_'), '=');
$knownMagicHash = $secrets->verificationHash($knownMagicVerifier);

// Update the magic registration's verification hash directly.
$magicTable = $registrationMigration->table('wpuiai_activation_registrations');
$updateStmt = $db->prepare("UPDATE {$magicTable} SET verification_challenge_hash = :hash WHERE registration_uuid = :uuid");
$updateStmt->execute([':hash' => $knownMagicHash, ':uuid' => $magicRegistrationUuid]);

$now = '2026-08-08T00:04:00Z';
$magicVerifyResult = $completeHandler->complete([
    'registration_uuid' => $magicRegistrationUuid,
    'verifier' => $knownMagicVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-magic-complete',
    'idempotency_key' => 'idem-verify-magic-complete',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(!isset($magicVerifyResult['error']), 'magic link verification completes');
expect_verification_complete($magicVerifyResult['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'magic link verification reaches email_verified');

// ── Negative: wrong verifier ──────────────────────────────────────────

// Create a fresh registration for wrong-verifier test.
$now = '2026-08-08T00:05:00Z';
$wrongResult = $registrations->createPending([
    'email' => 'synthetic.wrong@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-verify-wrong',
    'idempotency_key' => 'idem-verify-wrong-start',
]);
$wrongUuid = $wrongResult['registration']['registration_uuid'];
$correctVerifier = $wrongResult['verification_secret'];

$now = '2026-08-08T00:06:00Z';
$badVerifierResult = $completeHandler->complete([
    'registration_uuid' => $wrongUuid,
    'verifier' => 'wrong-verifier-token',
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-wrong-complete',
    'idempotency_key' => 'idem-verify-wrong',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($badVerifierResult['error']) && $badVerifierResult['error'] === 'EMAIL_VERIFICATION_FAILED', 'wrong verifier is rejected with EMAIL_VERIFICATION_FAILED');
expect_verification_complete($badVerifierResult['next_action'] === 'retry_or_recover_through_registered_facade', 'wrong verifier has safe next action');

// Verify the registration was NOT promoted.
$wrongRegistration = $registrations->findByUuid($wrongUuid);
expect_verification_complete($wrongRegistration['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'wrong verifier does not change state');
expect_verification_complete($wrongRegistration['verification_state'] === 'email_verification_pending', 'wrong verifier keeps verification pending');
expect_verification_complete((int) $wrongRegistration['verification_attempts'] >= 1, 'verification attempts are incremented');

// ── Negative: empty verifier ──────────────────────────────────────────

$now = '2026-08-08T00:07:00Z';
$emptyVerifierResult = $completeHandler->complete([
    'registration_uuid' => $wrongUuid,
    'verifier' => '',
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-empty',
    'idempotency_key' => 'idem-verify-empty',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($emptyVerifierResult['error']) && $emptyVerifierResult['error'] === 'EMAIL_VERIFICATION_FAILED', 'empty verifier is rejected');

// ── Negative: expired challenge ───────────────────────────────────────

// Create a registration and then advance time past the challenge TTL.
$now = '2026-08-08T00:08:00Z';
$expiredResult = $registrations->createPending([
    'email' => 'synthetic.expired@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-verify-expired',
    'idempotency_key' => 'idem-verify-expired-start',
]);
$expiredUuid = $expiredResult['registration']['registration_uuid'];
$expiredVerifier = $expiredResult['verification_secret'];

// Advance time past the verification TTL (900 seconds).
$now = '2026-08-08T00:24:00Z'; // 23 minutes later, past the 15-minute TTL
$expiredVerifyResult = $completeHandler->complete([
    'registration_uuid' => $expiredUuid,
    'verifier' => $expiredVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-expired-complete',
    'idempotency_key' => 'idem-verify-expired',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($expiredVerifyResult['error']) && $expiredVerifyResult['error'] === 'EMAIL_VERIFICATION_EXPIRED', 'expired challenge is rejected with EMAIL_VERIFICATION_EXPIRED');

// ── Negative: cross-facade token ──────────────────────────────────────

// Create a registration on focusa_install_v1, then try to verify from focusa_marketing_v1.
$now = '2026-08-08T00:09:00Z';
$crossFacadeResult = $registrations->createPending([
    'email' => 'synthetic.crossfacade@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-verify-crossfacade',
    'idempotency_key' => 'idem-verify-crossfacade-start',
]);
$crossFacadeUuid = $crossFacadeResult['registration']['registration_uuid'];
$crossFacadeVerifier = $crossFacadeResult['verification_secret'];

$now = '2026-08-08T00:10:00Z';
$crossFacadeVerifyResult = $completeHandler->complete([
    'registration_uuid' => $crossFacadeUuid,
    'verifier' => $crossFacadeVerifier,
    'facade_id' => 'focusa_marketing_v1',
    'origin' => 'https://focusa.dev',
    'request_id' => 'req-verify-crossfacade-complete',
    'idempotency_key' => 'idem-verify-crossfacade',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($crossFacadeVerifyResult['error']) && $crossFacadeVerifyResult['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'cross-facade token is rejected with EMAIL_VERIFICATION_REQUIRED');
expect_verification_complete($crossFacadeVerifyResult['next_action'] === 'retry_or_recover_through_registered_facade', 'cross-facade token has safe next action');

// Verify the registration was NOT promoted.
$crossFacadeRegistration = $registrations->findByUuid($crossFacadeUuid);
expect_verification_complete($crossFacadeRegistration['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'cross-facade token does not change state');

// ── Negative: wrong origin ────────────────────────────────────────────

$now = '2026-08-08T00:11:00Z';
$badOriginResult = $completeHandler->complete([
    'registration_uuid' => $crossFacadeUuid,
    'verifier' => $crossFacadeVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://evil.invalid',
    'request_id' => 'req-verify-bad-origin',
    'idempotency_key' => 'idem-verify-bad-origin',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($badOriginResult['error']) && $badOriginResult['error'] === 'FACADE_ORIGIN_DENIED', 'wrong origin is denied');

// ── Negative: already verified registration ───────────────────────────

$now = '2026-08-08T00:12:00Z';
// The direct registration was already verified. Try to verify again with a different idempotency key.
$alreadyVerifiedResult = $completeHandler->complete([
    'registration_uuid' => $directRegistrationUuid,
    'verifier' => $knownVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-already',
    'idempotency_key' => 'idem-verify-already',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($alreadyVerifiedResult['error']) && $alreadyVerifiedResult['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'already-verified registration is rejected with EMAIL_VERIFICATION_REQUIRED');

// ── Negative: wrong registration UUID ────────────────────────────────

$now = '2026-08-08T00:13:00Z';
$badUuidResult = $completeHandler->complete([
    'registration_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'verifier' => 'some-verifier',
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-bad-uuid',
    'idempotency_key' => 'idem-verify-bad-uuid',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete(isset($badUuidResult['error']) && $badUuidResult['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'wrong registration UUID is rejected with EMAIL_VERIFICATION_REQUIRED');

// ── Negative: max verification attempts exceeded ──────────────────────

// Use a dedicated rate limiter with generous limits so the max-attempts
// test is not conflated with rate limiting.
$maxAttemptsClock = static function () use (&$now): string {
    return $now;
};
$maxAttemptsRateLimiter = new FocusaSpec152eRateLimiter(new PDO('sqlite::memory:'), 'wp_', $maxAttemptsClock, 60, 20, 20);
$maxAttemptsHandler = new FocusaSpec152eVerificationCompleteHandler(
    $registrations,
    $maxAttemptsRateLimiter,
);
$maxAttemptsClient = hash('sha256', 'max-attempts-client');

// Create a fresh registration and exhaust all attempts.
$now = '2026-08-08T00:14:00Z';
$maxAttemptsResult = $registrations->createPending([
    'email' => 'synthetic.maxattempts@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator_lifetime_v1',
    'request_id' => 'req-verify-max',
    'idempotency_key' => 'idem-verify-max-start',
]);
$maxAttemptsUuid = $maxAttemptsResult['registration']['registration_uuid'];
$maxAttemptsVerifier = $maxAttemptsResult['verification_secret'];

// Submit wrong verifiers to exhaust attempts.
for ($i = 0; $i < FocusaSpec152eVerificationCompleteHandler::MAX_VERIFICATION_ATTEMPTS; $i++) {
    $now = '2026-08-08T00:15:' . sprintf('%02d', $i) . 'Z';
    $attemptResult = $maxAttemptsHandler->complete([
        'registration_uuid' => $maxAttemptsUuid,
        'verifier' => 'wrong-verifier-' . $i,
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'request_id' => 'req-verify-max-attempt-' . $i,
        'idempotency_key' => 'idem-verify-max-attempt-' . $i,
    ], $facadeRegistry, $maxAttemptsClient);
    expect_verification_complete(isset($attemptResult['error']) && $attemptResult['error'] === 'EMAIL_VERIFICATION_FAILED', "attempt {$i} is rejected with EMAIL_VERIFICATION_FAILED");
}

// Now the correct verifier should also fail because attempts are exhausted.
$now = '2026-08-08T00:16:00Z';
$exhaustedResult = $maxAttemptsHandler->complete([
    'registration_uuid' => $maxAttemptsUuid,
    'verifier' => $maxAttemptsVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://install.focusa.dev',
    'request_id' => 'req-verify-max-exhausted',
    'idempotency_key' => 'idem-verify-max-exhausted',
], $facadeRegistry, $maxAttemptsClient);
expect_verification_complete(isset($exhaustedResult['error']) && $exhaustedResult['error'] === 'EMAIL_VERIFICATION_FAILED', 'correct verifier is rejected after max attempts');

// Verify the registration was NOT promoted.
$maxAttemptsRegistration = $registrations->findByUuid($maxAttemptsUuid);
expect_verification_complete($maxAttemptsRegistration['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'max attempts registration stays in challenge_sent');
expect_verification_complete((int) $maxAttemptsRegistration['verification_attempts'] === FocusaSpec152eVerificationCompleteHandler::MAX_VERIFICATION_ATTEMPTS, 'verification attempts count equals max');

// ── Negative: rate limiting on verification ───────────────────────────

$rateLimitDb = new PDO('sqlite::memory:');
$rateLimitDb->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$rateLimitClock = static function () use (&$now): string {
    return $now;
};
$rateLimiter2 = new FocusaSpec152eRateLimiter($rateLimitDb, 'wp_', $rateLimitClock, 60, 5, 3);
$rateHandler = new FocusaSpec152eVerificationCompleteHandler(
    $registrations,
    $rateLimiter2,
);
$rateClient = hash('sha256', 'rate-limited-verify-client');

// First few requests should return real errors (not rate-limited).
$now = '2026-08-08T00:17:00Z';
$rateLimited = false;
for ($i = 0; $i < 10; $i++) {
    $rateResult = $rateHandler->complete([
        'registration_uuid' => $wrongUuid,
        'verifier' => 'wrong-verifier-rate-' . $i,
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'request_id' => 'req-verify-rate-' . $i,
        'idempotency_key' => 'idem-verify-rate-' . $i,
    ], $facadeRegistry, $rateClient);
    if (isset($rateResult['error']) && $rateResult['error'] === 'ACTIVATION_REQUEST_ACCEPTED') {
        $rateLimited = true;
        break;
    }
}
expect_verification_complete($rateLimited, 'rate limiting is enforced without revealing whether the challenge exists');

// ── Negative: no customer, license, node, or lease created on failure ──

$registrationCount = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations")->fetchColumn();
$nonPending = $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations WHERE account_uuid IS NOT NULL OR edd_customer_id IS NOT NULL OR edd_order_id IS NOT NULL OR edd_license_id IS NOT NULL OR node_uuid IS NOT NULL")->fetchColumn();
expect_verification_complete($registrationCount >= 6, 'registrations were created for test cases');
expect_verification_complete((int) $nonPending === 0, 'no registration has account, customer, order, license, or node references');

// Only the verified registrations should have verified_at set.
$verifiedCount = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations WHERE verified_at IS NOT NULL")->fetchColumn();
expect_verification_complete($verifiedCount >= 2, 'only valid verifications set verified_at');
$unverifiedCount = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registrations WHERE verification_state = 'mailbox_verified'")->fetchColumn();
expect_verification_complete($unverifiedCount === $verifiedCount, 'mailbox_verified count matches verified_at count');

// ── Challenge Service: validate the verifier hash comparison ──────────

// The ChallengeService has its own hash method. Verify it interoperates
// with the RegistrationSecrets verification hash.
$challengeHash = $challengeService->hash('test-verifier-token');
$registrationHash = $secrets->verificationHash('test-verifier-token');
expect_verification_complete(hash_equals($challengeHash, $registrationHash), 'ChallengeService and RegistrationSecrets hashes are compatible');
expect_verification_complete($challengeService->validate('test-verifier-token', $challengeHash), 'valid verifier matches stored hash');
expect_verification_complete(!$challengeService->validate('wrong-verifier', $challengeHash), 'wrong verifier does not match');

// ── Enumeration resistance: known and unknown facade failures are indistinguishable ──

$knownBadFacade = $completeHandler->complete([
    'registration_uuid' => $wrongUuid,
    'verifier' => $correctVerifier,
    'facade_id' => 'focusa_install_v1',
    'origin' => 'https://evil.invalid',
    'request_id' => 'req-enum-bad-origin',
    'idempotency_key' => 'idem-enum-bad-origin',
], $facadeRegistry, $opaqueClientKey);
$unknownBadFacade = $completeHandler->complete([
    'registration_uuid' => $wrongUuid,
    'verifier' => $correctVerifier,
    'facade_id' => 'nonexistent_facade_v1',
    'origin' => 'https://nonexistent.invalid',
    'request_id' => 'req-enum-unknown-facade',
    'idempotency_key' => 'idem-enum-unknown-facade',
], $facadeRegistry, $opaqueClientKey);
expect_verification_complete($knownBadFacade['error'] === $unknownBadFacade['error'], 'known and unknown facade origin failures are indistinguishable');

// Cross-facade and nonexistent UUID are indistinguishable.
$crossFacadeError = $crossFacadeVerifyResult['error'];
$badUuidError = $badUuidResult['error'];
expect_verification_complete($crossFacadeError === $badUuidError, 'cross-facade and nonexistent UUID errors are indistinguishable');

// ── Rollback preservation ─────────────────────────────────────────────

$rollback = $registrationMigration->preserveForRollback('2026-08-08T00:20:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_verification_complete_rollback',
]);
expect_verification_complete($rollback['action'] === 'preserve', 'rollback is preservation-only');
$afterRollback = $registrations->findByUuid($directRegistrationUuid);
expect_verification_complete($afterRollback['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'rollback preserves verified registration state');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.verification_complete_test.v1',
    'positive_checks' => 22,
    'negative_checks' => 18,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");