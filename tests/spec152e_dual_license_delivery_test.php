<?php
// 152E.04.02 Dual email + terminal delivery of one canonical key. The
// dual-delivery coordinator settles ONE canonical EDD Software Licensing key
// through the approved transactional license email (branded facade sender,
// full key, product/order identity, activation instructions, manage and
// recovery links, support info, no promotional content) and the one-time
// device-encrypted terminal envelope. Masked outcomes and bounces are
// journaled per channel; the plaintext key and unmasked email never enter any
// response, journal, or generic log. Settlement is idempotent (no duplicate
// email, no second key), and authenticated recovery after a partial delivery
// re-delivers the SAME canonical key through the failed channel — never a new
// license. Partial-failure fixtures cover: email hard-bounce with terminal
// delivery, terminal delivery loss with email delivery, and both channels
// failed, each recovered without minting a second key.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-terminal-delivery-envelope.v1.php';
require_once $root . '/docs/contracts/spec152e-transactional-mail-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-dual-delivery-coordinator.v1.php';
$facadeRegistry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_dual(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_dual_throws(callable $operation, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    try {
        $operation();
    } catch (Throwable $error) {
        if ($code === 'InvalidArgumentException') {
            if (!$error instanceof InvalidArgumentException) {
                fwrite(STDERR, "FAIL: {$message} (got " . get_class($error) . ")");
                exit(1);
            }
            return;
        }
        if ($error->getMessage() !== $code) {
            fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
            exit(1);
        }
        return;
    }
    fwrite(STDERR, "FAIL: {$message}\n");
    exit(1);
}

function b64url_encode_php(string $binary): string
{
    return rtrim(strtr(base64_encode($binary), '+/', '-_'), '=');
}

function b64url_decode_php(string $encoded): string
{
    $padding = (4 - strlen($encoded) % 4) % 4;
    $decoded = base64_decode(strtr($encoded . str_repeat('=', $padding), '-_', '+/'), true);
    if ($decoded === false) {
        throw new DomainException('ENVELOPE_FORMAT_DENIED');
    }
    return $decoded;
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'dual_delivery_test']);
$envelopeMigration = new FocusaSpec152eTerminalDeliveryEnvelopeMigration($db, 'wp_');
$envelopeMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'dual_delivery_test']);
$dualMigration = new FocusaSpec152eDualLicenseDeliveryMigration($db, 'wp_');
$dualMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'dual_delivery_test']);
$dualMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'repeat_must_preserve_first_schema_application']);
$migrations = $db->query('SELECT * FROM wp_wpuiai_dual_license_delivery_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_dual(count($migrations) === 1, 'dual-delivery migration is version-idempotent');
expect_dual($migrations[0]['applied_at'] === '2026-08-08T00:00:00Z', 'dual-delivery migration preserves first application time');

$columns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_dual_license_deliveries)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $columns[$column['name']] = $column;
}
foreach ([
    'delivery_handle', 'registration_uuid', 'account_uuid', 'edd_customer_id', 'edd_license_id',
    'product_code', 'license_key_digest', 'license_key_mask',
    'email_channel_status', 'email_channel_attempts', 'email_attempted_at', 'email_delivered_at', 'email_outcome_code',
    'terminal_channel_status', 'terminal_channel_attempts', 'terminal_delivered_at',
    'resolved_state', 'recovery_handle', 'recovery_resolved_at',
    'recovery_envelope_id', 'recovery_envelope_payload', 'recovery_envelope_expires_at',
    'request_id', 'idempotency_key', 'request_digest', 'created_at', 'updated_at', 'retention_until',
] as $field) {
    expect_dual(isset($columns[$field]), "dual-delivery journal contains {$field}");
}
expect_dual($columns['license_key_digest']['notnull'] === 1, 'journal never stores the plaintext key');
expect_dual($columns['recovery_envelope_payload']['notnull'] === 0, 'recovery envelope is sealed and optional');

$regColumns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_activation_registrations)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $regColumns[$column['name']] = $column;
}
foreach (['email_delivery_status', 'email_delivery_attempts', 'email_delivered_at', 'email_delivery_outcome'] as $field) {
    expect_dual(isset($regColumns[$field]), "registration delivery state contains {$field}");
}
expect_dual($regColumns['email_delivery_status']['notnull'] === 1, 'registration email delivery status is bounded not-null');
expect_dual($regColumns['email_delivery_status']['dflt_value'] === "'none'", 'registration email delivery status defaults to none');

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
$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $clock, attemptTtl: 86400, verificationTtl: 3600, pollTtl: 3600);

$sentEmails = [];
$suppressNextSend = false;
$mailAdapter = new FocusaSpec152eTransactionalMailAdapter(
    static function (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) use (&$sentEmails, &$suppressNextSend): bool {
        $sentEmails[] = ['to' => $to, 'subject' => $subject, 'html' => $htmlBody, 'text' => $textBody, 'sender' => $senderIdentity];
        return !$suppressNextSend;
    }
);
$coordinator = new FocusaSpec152eDualLicenseDeliveryCoordinator($db, $dualMigration, $registrations, $registrationSecrets, $mailAdapter, $clock);
$terminalService = new FocusaSpec152eTerminalEnvelopeService($db, $envelopeMigration, $registrations, $registrationSecrets, $clock);

$installFacade = null;
foreach ($facadeRegistry['facades'] as $f) {
    if ($f['facade_id'] === 'focusa_install_v1') {
        $installFacade = $f;
        break;
    }
}
expect_dual(is_array($installFacade) && isset($installFacade['sender']['identity']), 'fixture uses the registered install facade sender identity');

$eddLicenseCount = static function (): int {
    global $db;
    return (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn();
};

// Fixture: mailbox-verified, promoted, paid registration at terminal_delivery_ready.
$fixtureSequence = 0;
$makeFixture = static function () use (&$fixtureSequence, $registrations, $db, $clock): array {
    $fixtureSequence++;
    $seq = str_pad((string) $fixtureSequence, 4, '0', STR_PAD_LEFT);
    $licenseId = 1000 + $fixtureSequence;
    $orderId = 5000 + $fixtureSequence;
    $itemId = 6000 + $fixtureSequence;
    $created = $registrations->createPending([
        'email' => 'synthetic.operator' . $seq . '@example.invalid',
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'terminal',
        'install_channel' => 'source_build',
        'product_code' => 'focusa_operator',
        'request_id' => 'req-dual-' . $seq . '-0001',
        'idempotency_key' => 'idem-dual-' . $seq . '-0001',
    ]);
    $registrationId = $created['registration']['registration_uuid'];
    $pollCredential = $created['poll_credential'];
    $verified = $registrations->verifyEmail($registrationId, $created['verification_secret'], 'req-dual-' . $seq . '-0002', 'idem-dual-' . $seq . '-0002');
    $promoted = $registrations->promoteVerified($registrationId, '018f47c2-6ac0-7b16-8d1a-' . str_pad(dechex($fixtureSequence), 12, '0', STR_PAD_LEFT), 41000 + $fixtureSequence, 'req-dual-' . $seq . '-0003', 'idem-dual-' . $seq . '-0003');
    $offer = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, (int) $promoted['registration']['state_version'], 'req-dual-' . $seq . '-0004', 'idem-dual-' . $seq . '-0004',
        ['offer_code' => 'focusa_operator', 'journey' => 'purchase']);
    $checkout = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
        FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING, (int) $offer['registration']['state_version'],
        'req-dual-' . $seq . '-0005', 'idem-dual-' . $seq . '-0005', ['edd_cart_reference' => 'cart_dual_' . $seq]);
    $issued = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
        FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED, (int) $checkout['registration']['state_version'],
        'req-dual-' . $seq . '-0006', 'idem-dual-' . $seq . '-0006', ['edd_order_id' => $orderId, 'edd_order_item_id' => $itemId, 'edd_license_id' => $licenseId]);
    $ready = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
        FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, (int) $issued['registration']['state_version'],
        'req-dual-' . $seq . '-0007', 'idem-dual-' . $seq . '-0007');
    $key = strtoupper(substr(hash('sha256', 'fixture-' . $seq), 0, 8)) . '-' . strtoupper(substr(hash('sha256', 'a-' . $seq), 0, 8))
        . '-' . strtoupper(substr(hash('sha256', 'b-' . $seq), 0, 8)) . '-' . strtoupper(substr(hash('sha256', 'c-' . $seq), 0, 8));
    $db->exec("INSERT INTO wp_edd_licenses
        (id, license_key, customer_id, user_id, product_id, order_id, license_length, license_unit,
         expiration, activation_count, activation_limit, status, date_created)
        VALUES ({$licenseId}, '" . $key . "', 41000 + {$fixtureSequence}, NULL, 1001, {$orderId}, 0, 'years', NULL, 0, 5, 'active', '2026-08-08T00:00:00Z')");
    return [
        'registration_id' => $registrationId,
        'poll_credential' => $pollCredential,
        'license_id' => $licenseId,
        'license_key' => $key,
        'seq' => $seq,
    ];
};

// Device X25519 keypair for terminal fixtures.
$devicePrivate = random_bytes(32);
$devicePublicRaw = FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($devicePrivate);
$devicePublicB64 = b64url_encode_php($devicePublicRaw);

// ── Fixture A: full dual delivery — email + terminal resolve one key ─────

$fixtureA = $makeFixture();
$licensesBefore = $eddLicenseCount();
$emailsBefore = count($sentEmails);

$settle = $coordinator->settle([
    'registration_id' => $fixtureA['registration_id'],
    'facade' => $installFacade,
    'product_name' => 'Focusa Operator',
    'support_email' => 'support.synthetic@example.invalid',
    'request_id' => 'req-settle-a-0001',
    'idempotency_key' => 'idem-settle-a-0001',
]);
expect_dual($settle['schema'] === FocusaSpec152eDualLicenseDeliveryCoordinator::DELIVERY_SCHEMA, 'settle returns the dual delivery state schema');
expect_dual($settle['email_sent'] === true, 'settle sends the transactional license email');
expect_dual($settle['email_delivery_status'] === 'sent', 'settle records the email attempt');
expect_dual($settle['email_channel']['attempts'] === 1, 'one email send attempt is recorded');
expect_dual($settle['terminal_channel']['status'] === 'pending', 'terminal channel is pending after settle');
expect_dual($settle['resolved_state'] === 'pending', 'dual delivery starts pending');
expect_dual($settle['license_key_mask'] === FocusaSpec152eTerminalDeliveryEnvelope::maskKey($fixtureA['license_key']), 'settle exposes only the masked key');
expect_dual(str_contains(json_encode($settle), $fixtureA['license_key']) === false, 'no plaintext key in the settle response');
expect_dual((bool) preg_match('/^dlv_[0-9a-f]{32}$/D', $settle['delivery_handle']), 'settle returns a bounded delivery handle');

expect_dual(count($sentEmails) === $emailsBefore + 1, 'exactly one license email is sent at settle');
$licenseEmail = $sentEmails[count($sentEmails) - 1];
expect_dual($licenseEmail['sender'] === 'focusa_install_transactional_v1', 'license email uses the registered facade sender identity');
expect_dual(str_contains($licenseEmail['subject'], 'Focusa Install'), 'license email subject is branded');
expect_dual(str_contains($licenseEmail['html'], $fixtureA['license_key']), 'license email carries the full human license key');
expect_dual(str_contains($licenseEmail['text'], $fixtureA['license_key']), 'license email text carries the full human license key');
expect_dual(str_contains($licenseEmail['html'], 'https://install.focusa.dev/account'), 'license email carries the account-management link');
expect_dual(str_contains($licenseEmail['html'], 'https://install.focusa.dev/activate/recovery'), 'license email carries the recovery link');
expect_dual(str_contains($licenseEmail['html'], 'support.synthetic@example.invalid'), 'license email carries support information');
expect_dual(stripos($licenseEmail['html'], 'promotion') === false && stripos($licenseEmail['html'], 'offer') === false, 'license email has no promotional content');

// Idempotent settle: same key returns the existing journal, no second email.
$settleReplay = $coordinator->settle([
    'registration_id' => $fixtureA['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-a-0001',
    'idempotency_key' => 'idem-settle-a-0001',
]);
expect_dual($settleReplay['delivery_handle'] === $settle['delivery_handle'], 'replayed settle returns the same delivery handle');
expect_dual($settleReplay['email_sent'] === false, 'replayed settle never re-sends the email');
expect_dual(count($sentEmails) === $emailsBefore + 1, 'replayed settle mints no second email');

// A different idempotency key on the same registration is also bounded.
$settleAgain = $coordinator->settle([
    'registration_id' => $fixtureA['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-a-0009',
    'idempotency_key' => 'idem-settle-a-0009',
]);
expect_dual($settleAgain['delivery_handle'] === $settle['delivery_handle'], 'a repeated settle on the same registration is idempotent');
expect_dual(count($sentEmails) === $emailsBefore + 1, 'a repeated settle never sends a duplicate email');
expect_dual($eddLicenseCount() === $licensesBefore, 'settle never mints a second license');

// Provider confirms email delivery.
$delivered = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureA['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:02:00Z',
    'request_id' => 'req-email-a-0001',
    'idempotency_key' => 'idem-email-a-0001',
]);
expect_dual($delivered['email_channel']['status'] === 'delivered', 'provider delivery is recorded on the email channel');
expect_dual($delivered['email_channel']['delivered_at'] === '2026-08-08T00:02:00Z', 'email delivery timestamp is recorded');
expect_dual($delivered['resolved_state'] === 'email_only', 'email-only delivery resolves as partial');

// Idempotent provider callback replay.
$deliveredReplay = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureA['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:02:00Z',
    'request_id' => 'req-email-a-0001',
    'idempotency_key' => 'idem-email-a-0001',
]);
expect_dual($deliveredReplay['email_channel']['attempts'] === 1, 'replayed provider callback never double-counts');

// Terminal delivery resolves the SAME canonical key through the envelope.
$poll = $terminalService->deliverPollResponse([
    'registration_id' => $fixtureA['registration_id'],
    'poll_credential' => $fixtureA['poll_credential'],
    'device_public_key' => $devicePublicB64,
    'request_id' => 'req-poll-a-0001',
    'idempotency_key' => 'idem-poll-a-0001',
]);
$envelope = json_decode(b64url_decode_php($poll['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
$terminalClaims = $terminalService->openForDevice([
    'envelope' => $envelope,
    'device_private_key' => bin2hex($devicePrivate),
    'registration_id' => $fixtureA['registration_id'],
    'now' => '2026-08-08T00:03:00Z',
]);
expect_dual($terminalClaims['license_key'] === $fixtureA['license_key'], 'terminal envelope resolves the exact canonical EDD key');
expect_dual((int) $terminalClaims['edd_license_id'] === $fixtureA['license_id'], 'terminal envelope binds the same EDD license ID');

$terminalDigest = FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($terminalClaims['license_key']);
$confirmed = $coordinator->noteTerminalDelivered([
    'registration_id' => $fixtureA['registration_id'],
    'edd_license_id' => (int) $terminalClaims['edd_license_id'],
    'license_key_digest' => $terminalDigest,
    'request_id' => 'req-terminal-a-0001',
    'idempotency_key' => 'idem-terminal-a-0001',
]);
expect_dual($confirmed['terminal_channel']['status'] === 'delivered', 'terminal channel is recorded delivered');
expect_dual($confirmed['terminal_channel']['attempts'] === 1, 'one terminal delivery attempt is recorded');
expect_dual($confirmed['resolved_state'] === 'both_delivered', 'dual delivery settles only when both channels resolve');
expect_dual($confirmed['same_key_confirmed'] === true, 'email and terminal resolve one canonical key');
expect_dual($confirmed['recovery_resolved_at'] !== null, 'settled dual delivery records the recovery resolution');
expect_dual($eddLicenseCount() === $licensesBefore, 'dual delivery never mints a second license');

// Recovery is refused after full settlement.
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureA['registration_id'],
        'poll_credential' => $fixtureA['poll_credential'],
        'recovery_channel' => 'terminal',
        'request_id' => 'req-recover-a-0001',
        'idempotency_key' => 'idem-recover-a-0001',
    ]),
    'DUAL_DELIVERY_ALREADY_SETTLED',
    'recovery after full settlement fails closed'
);

// Registration email delivery state mirrors the masked outcome.
$regA = $registrations->findByUuid($fixtureA['registration_id']);
expect_dual((string) $regA['email_delivery_status'] === 'delivered', 'registration records the masked email delivery status');
expect_dual((int) $regA['email_delivery_attempts'] === 1, 'registration records the email send attempts');
expect_dual((string) $regA['email_delivery_outcome'] === 'none', 'registration records a clean delivery outcome');
expect_dual((string) $regA['email_delivered_at'] === '2026-08-08T00:02:00Z', 'registration records the email delivery timestamp');

// ── Fixture B: email hard-bounce, terminal delivered, email recovery ─────

$fixtureB = $makeFixture();
$licensesBeforeB = $eddLicenseCount();
$emailsBeforeB = count($sentEmails);
$coordinator->settle([
    'registration_id' => $fixtureB['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-b-0001',
    'idempotency_key' => 'idem-settle-b-0001',
]);
$bounced = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureB['registration_id'],
    'delivery_status' => 'bounced',
    'bounce_type' => 'hard',
    'occurred_at' => '2026-08-08T00:02:00Z',
    'request_id' => 'req-email-b-0001',
    'idempotency_key' => 'idem-email-b-0001',
]);
expect_dual($bounced['email_channel']['status'] === 'bounced', 'hard bounce is recorded on the email channel');
expect_dual($bounced['email_channel']['outcome'] === 'hard_bounce', 'hard bounce outcome is masked and bounded');
expect_dual($bounced['resolved_state'] === 'recovery_required', 'a hard bounce demands authenticated recovery');
expect_dual($bounced['recovery_handle'] !== null && str_starts_with((string) $bounced['recovery_handle'], 'rec_'), 'a bounded recovery handle is issued');
expect_dual((string) $registrations->findByUuid($fixtureB['registration_id'])['email_delivery_outcome'] === 'hard_bounce', 'registration records the masked bounce outcome');

// Terminal still delivers the SAME key.
$pollB = $terminalService->deliverPollResponse([
    'registration_id' => $fixtureB['registration_id'],
    'poll_credential' => $fixtureB['poll_credential'],
    'device_public_key' => $devicePublicB64,
    'request_id' => 'req-poll-b-0001',
    'idempotency_key' => 'idem-poll-b-0001',
]);
$envelopeB = json_decode(b64url_decode_php($pollB['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
$claimsB = $terminalService->openForDevice([
    'envelope' => $envelopeB,
    'device_private_key' => bin2hex($devicePrivate),
    'registration_id' => $fixtureB['registration_id'],
    'now' => '2026-08-08T00:03:00Z',
]);
expect_dual($claimsB['license_key'] === $fixtureB['license_key'], 'terminal resolves the same canonical key after email bounce');
$coordinator->noteTerminalDelivered([
    'registration_id' => $fixtureB['registration_id'],
    'edd_license_id' => (int) $claimsB['edd_license_id'],
    'license_key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($claimsB['license_key']),
    'request_id' => 'req-terminal-b-0001',
    'idempotency_key' => 'idem-terminal-b-0001',
]);
$partialB = $coordinator->deliveryState(['registration_id' => $fixtureB['registration_id']]);
expect_dual($partialB['resolved_state'] === 'terminal_only', 'terminal-only delivery after email bounce resolves partial');

// Authenticated email recovery re-delivers the SAME key; no second key.
$recoveredB = $coordinator->recover([
    'registration_id' => $fixtureB['registration_id'],
    'poll_credential' => $fixtureB['poll_credential'],
    'recovery_channel' => 'email',
    'facade' => $installFacade,
    'request_id' => 'req-recover-b-0001',
    'idempotency_key' => 'idem-recover-b-0001',
]);
expect_dual($recoveredB['schema'] === FocusaSpec152eDualLicenseDeliveryCoordinator::RECOVERY_SCHEMA, 'recovery returns the recovery schema');
expect_dual($recoveredB['recovery_channel'] === 'email', 'recovery re-delivers through the email channel');
expect_dual(count($sentEmails) === $emailsBeforeB + 2, 'recovery re-sends exactly one license email');
$recoveryEmail = $sentEmails[count($sentEmails) - 1];
expect_dual(str_contains($recoveryEmail['html'], $fixtureB['license_key']), 'recovery email carries the SAME canonical key');
expect_dual($recoveredB['email_channel']['attempts'] === 2, 'recovery increments the email attempt count');
expect_dual($eddLicenseCount() === $licensesBeforeB, 'recovery never mints a second key');

// Email recovery replay is idempotent (no third email).
$recoverReplayB = $coordinator->recover([
    'registration_id' => $fixtureB['registration_id'],
    'poll_credential' => $fixtureB['poll_credential'],
    'recovery_channel' => 'email',
    'facade' => $installFacade,
    'request_id' => 'req-recover-b-0001',
    'idempotency_key' => 'idem-recover-b-0001',
]);
expect_dual($recoverReplayB['email_channel']['attempts'] === 2, 'replayed recovery never re-sends');
expect_dual(count($sentEmails) === $emailsBeforeB + 2, 'replayed recovery mints no third email');

$settledB = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureB['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:04:00Z',
    'request_id' => 'req-email-b-0009',
    'idempotency_key' => 'idem-email-b-0009',
]);
expect_dual($settledB['resolved_state'] === 'both_delivered', 'recovered email delivery settles both channels');
expect_dual($eddLicenseCount() === $licensesBeforeB, 'partial failure and recovery never mint a second key');

// ── Fixture C: terminal delivery loss — email delivered, terminal recovery ─

$fixtureC = $makeFixture();
$licensesBeforeC = $eddLicenseCount();
$coordinator->settle([
    'registration_id' => $fixtureC['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-c-0001',
    'idempotency_key' => 'idem-settle-c-0001',
]);
$coordinator->recordEmailOutcome([
    'registration_id' => $fixtureC['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:02:00Z',
    'request_id' => 'req-email-c-0001',
    'idempotency_key' => 'idem-email-c-0001',
]);
$partialC = $coordinator->deliveryState(['registration_id' => $fixtureC['registration_id']]);
expect_dual($partialC['resolved_state'] === 'email_only', 'email-only delivery with terminal loss resolves partial');

// Bind the device key without delivering the terminal envelope (delivery loss).
$boundC = $registrations->bindDevicePublicKey($fixtureC['registration_id'], $devicePublicB64, 'req-bind-c-0001', 'idem-bind-c-0001');
expect_dual(hash_equals((string) $boundC['registration']['device_public_key'], $devicePublicB64), 'device key is bound for authenticated terminal recovery');

// Authenticated terminal recovery seals a fresh envelope for the SAME key.
$recoveredC = $coordinator->recover([
    'registration_id' => $fixtureC['registration_id'],
    'poll_credential' => $fixtureC['poll_credential'],
    'recovery_channel' => 'terminal',
    'request_id' => 'req-recover-c-0001',
    'idempotency_key' => 'idem-recover-c-0001',
]);
expect_dual($recoveredC['recovery_channel'] === 'terminal', 'recovery re-delivers through the terminal channel');
expect_dual((bool) preg_match('/^env_[0-9a-f]{32}$/D', (string) $recoveredC['envelope_id']), 'terminal recovery returns a bounded envelope ID');
expect_dual(str_contains($recoveredC['one_time_key_envelope'] ?? '', '') && str_contains($recoveredC['one_time_key_envelope'], $fixtureC['license_key']) === false, 'recovery envelope never exposes plaintext');
$recoveryEnvelope = json_decode(b64url_decode_php($recoveredC['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
$recoveryClaims = $terminalService->openForDevice([
    'envelope' => $recoveryEnvelope,
    'device_private_key' => bin2hex($devicePrivate),
    'registration_id' => $fixtureC['registration_id'],
    'now' => '2026-08-08T00:03:00Z',
]);
expect_dual($recoveryClaims['license_key'] === $fixtureC['license_key'], 'terminal recovery envelope resolves the SAME canonical key');
expect_dual((int) $recoveryClaims['edd_license_id'] === $fixtureC['license_id'], 'terminal recovery envelope binds the same license');
expect_dual($recoveredC['resolved_state'] === 'both_delivered', 'terminal recovery settles both channels');
expect_dual($eddLicenseCount() === $licensesBeforeC, 'terminal recovery never mints a second key');

// Recovery envelope replay returns the identical envelope.
$recoverReplayC = $coordinator->recover([
    'registration_id' => $fixtureC['registration_id'],
    'poll_credential' => $fixtureC['poll_credential'],
    'recovery_channel' => 'terminal',
    'request_id' => 'req-recover-c-0001',
    'idempotency_key' => 'idem-recover-c-0001',
]);
expect_dual($recoverReplayC['envelope_id'] === $recoveredC['envelope_id'], 'replayed terminal recovery returns the same envelope');
expect_dual($recoverReplayC['one_time_key_envelope'] === $recoveredC['one_time_key_envelope'], 'replayed terminal recovery returns the identical envelope');

// ── Fixture D: both channels failed — recovery re-delivers and settles ───

$fixtureD = $makeFixture();
$licensesBeforeD = $eddLicenseCount();
$emailsBeforeD = count($sentEmails);
$suppressNextSend = true;
$coordinator->settle([
    'registration_id' => $fixtureD['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-d-0001',
    'idempotency_key' => 'idem-settle-d-0001',
]);
$suppressNextSend = false;
$suppressedD = $coordinator->deliveryState(['registration_id' => $fixtureD['registration_id']]);
expect_dual($suppressedD['email_channel']['status'] === 'suppressed', 'a suppressed email send is recorded as suppressed');
expect_dual($suppressedD['email_channel']['outcome'] === 'suppressed_transactional', 'suppression is recorded with the masked outcome');
expect_dual($suppressedD['email_channel']['attempts'] === 0, 'suppression is not counted as a send attempt');
expect_dual($suppressedD['resolved_state'] === 'recovery_required', 'both-failed delivery demands authenticated recovery');

// A revoked canonical license blocks terminal delivery without a second key.
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = " . $fixtureD['license_id']);
expect_dual_throws(
    static fn() => $terminalService->deliverPollResponse([
        'registration_id' => $fixtureD['registration_id'],
        'poll_credential' => $fixtureD['poll_credential'],
        'device_public_key' => $devicePublicB64,
        'request_id' => 'req-poll-d-0001',
        'idempotency_key' => 'idem-poll-d-0001',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'a revoked canonical license fails closed at terminal delivery'
);
expect_dual($eddLicenseCount() === $licensesBeforeD, 'revoked-license terminal failure mints no second key');

// Authenticated email recovery is refused while the canonical key is unusable.
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureD['registration_id'],
        'poll_credential' => $fixtureD['poll_credential'],
        'recovery_channel' => 'email',
        'facade' => $installFacade,
        'request_id' => 'req-recover-d-0001',
        'idempotency_key' => 'idem-recover-d-0001',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'recovery fails closed while the canonical license is unusable'
);

// Restore the license; recovery re-delivers the SAME key.
$db->exec("UPDATE wp_edd_licenses SET status = 'active' WHERE id = " . $fixtureD['license_id']);
$recoveredD = $coordinator->recover([
    'registration_id' => $fixtureD['registration_id'],
    'poll_credential' => $fixtureD['poll_credential'],
    'recovery_channel' => 'email',
    'facade' => $installFacade,
    'request_id' => 'req-recover-d-0002',
    'idempotency_key' => 'idem-recover-d-0002',
]);
expect_dual($recoveredD['email_channel']['status'] === 'sent', 'authenticated recovery re-sends the email channel');
expect_dual($recoveredD['email_channel']['attempts'] === 1, 'recovery records the new send attempt');
expect_dual(count($sentEmails) === $emailsBeforeD + 2, 'suppressed settle capture plus one recovery send');
$recoveryEmailD = $sentEmails[count($sentEmails) - 1];
expect_dual(str_contains($recoveryEmailD['html'], $fixtureD['license_key']), 'recovery email carries the SAME canonical key after both-failed delivery');
$settledD = $coordinator->recordEmailOutcome([
    'registration_id' => $fixtureD['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:04:00Z',
    'request_id' => 'req-email-d-0001',
    'idempotency_key' => 'idem-email-d-0001',
]);
expect_dual($settledD['resolved_state'] === 'email_only', 'email recovery settles the email channel first');

// The restored canonical license now delivers through the terminal channel too.
$pollD = $terminalService->deliverPollResponse([
    'registration_id' => $fixtureD['registration_id'],
    'poll_credential' => $fixtureD['poll_credential'],
    'device_public_key' => $devicePublicB64,
    'request_id' => 'req-poll-d-0002',
    'idempotency_key' => 'idem-poll-d-0002',
]);
$envelopeD = json_decode(b64url_decode_php($pollD['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
$claimsD = $terminalService->openForDevice([
    'envelope' => $envelopeD,
    'device_private_key' => bin2hex($devicePrivate),
    'registration_id' => $fixtureD['registration_id'],
    'now' => '2026-08-08T00:05:00Z',
]);
expect_dual($claimsD['license_key'] === $fixtureD['license_key'], 'restored license terminal envelope resolves the SAME canonical key');
$finalD = $coordinator->noteTerminalDelivered([
    'registration_id' => $fixtureD['registration_id'],
    'edd_license_id' => (int) $claimsD['edd_license_id'],
    'license_key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($claimsD['license_key']),
    'request_id' => 'req-terminal-d-0001',
    'idempotency_key' => 'idem-terminal-d-0001',
]);
expect_dual($finalD['resolved_state'] === 'both_delivered', 'both-failed fixture recovers to full settlement');
expect_dual($eddLicenseCount() === $licensesBeforeD, 'both-failed recovery never mints a second key');

// ── Negative checks: authority and hygiene fail closed ──────────────────

// Wrong poll credential cannot recover.
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureC['registration_id'],
        'poll_credential' => 'wrong-poll-credential',
        'recovery_channel' => 'terminal',
        'request_id' => 'req-recover-c-0099',
        'idempotency_key' => 'idem-recover-c-0099',
    ]),
    'POLL_CREDENTIAL_REQUIRED',
    'recovery with a wrong poll credential fails closed'
);

// Expired poll credential fails closed.
$db->exec("UPDATE wp_wpuiai_activation_registrations SET poll_credential_expires_at = '2026-08-08T00:00:59Z' WHERE registration_uuid = '" . $fixtureC['registration_id'] . "'");
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureC['registration_id'],
        'poll_credential' => $fixtureC['poll_credential'],
        'recovery_channel' => 'terminal',
        'request_id' => 'req-recover-c-0098',
        'idempotency_key' => 'idem-recover-c-0098',
    ]),
    'POLL_CREDENTIAL_EXPIRED',
    'recovery with an expired poll credential fails closed'
);

// Terminal recovery without a bound device key fails closed.
$fixtureE = $makeFixture();
$coordinator->settle([
    'registration_id' => $fixtureE['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-e-0001',
    'idempotency_key' => 'idem-settle-e-0001',
]);
$coordinator->recordEmailOutcome([
    'registration_id' => $fixtureE['registration_id'],
    'delivery_status' => 'delivered',
    'occurred_at' => '2026-08-08T00:02:00Z',
    'request_id' => 'req-email-e-0001',
    'idempotency_key' => 'idem-email-e-0001',
]);
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixtureE['registration_id'],
        'poll_credential' => $fixtureE['poll_credential'],
        'recovery_channel' => 'terminal',
        'request_id' => 'req-recover-e-0001',
        'idempotency_key' => 'idem-recover-e-0001',
    ]),
    'NODE_PUBLIC_KEY_REQUIRED',
    'terminal recovery without a bound device key fails closed'
);

// Recovery before settlement fails closed.
$fixturePending = $makeFixture();
expect_dual_throws(
    static fn() => $coordinator->recover([
        'registration_id' => $fixturePending['registration_id'],
        'poll_credential' => $fixturePending['poll_credential'],
        'recovery_channel' => 'email',
        'facade' => $installFacade,
        'request_id' => 'req-recover-p-0001',
        'idempotency_key' => 'idem-recover-p-0001',
    ]),
    'LICENSE_DELIVERY_PENDING',
    'recovery before dual settlement fails closed'
);

// Settle on an unverified registration fails closed.
$fixtureUnverified = $makeFixture();
$unverifiedRow = $registrations->findByUuid($fixtureUnverified['registration_id']);
$db->exec("UPDATE wp_wpuiai_activation_registrations SET verification_state = 'email_verification_pending'
    WHERE registration_uuid = '" . $fixtureUnverified['registration_id'] . "'");
expect_dual_throws(
    static fn() => $coordinator->settle([
        'registration_id' => $fixtureUnverified['registration_id'],
        'facade' => $installFacade,
        'request_id' => 'req-settle-u-0001',
        'idempotency_key' => 'idem-settle-u-0001',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'settle on an unverified registration fails closed'
);

// Settle on a revoked canonical license fails closed.
$fixtureRevoked = $makeFixture();
$db->exec("UPDATE wp_edd_licenses SET status = 'revoked' WHERE id = " . $fixtureRevoked['license_id']);
expect_dual_throws(
    static fn() => $coordinator->settle([
        'registration_id' => $fixtureRevoked['registration_id'],
        'facade' => $installFacade,
        'request_id' => 'req-settle-r-0001',
        'idempotency_key' => 'idem-settle-r-0001',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'settle with a revoked canonical license fails closed'
);

// Terminal confirmation with a mismatched key fails closed (one-key rule).
$fixtureMismatch = $makeFixture();
$coordinator->settle([
    'registration_id' => $fixtureMismatch['registration_id'],
    'facade' => $installFacade,
    'request_id' => 'req-settle-m-0001',
    'idempotency_key' => 'idem-settle-m-0001',
]);
expect_dual_throws(
    static fn() => $coordinator->noteTerminalDelivered([
        'registration_id' => $fixtureMismatch['registration_id'],
        'edd_license_id' => $fixtureMismatch['license_id'],
        'license_key_digest' => hash('sha256', 'not-the-canonical-key'),
        'request_id' => 'req-terminal-m-0001',
        'idempotency_key' => 'idem-terminal-m-0001',
    ]),
    'DUAL_DELIVERY_KEY_MISMATCH',
    'a terminal channel with a different key fails closed'
);
expect_dual_throws(
    static fn() => $coordinator->noteTerminalDelivered([
        'registration_id' => $fixtureMismatch['registration_id'],
        'edd_license_id' => $fixtureMismatch['license_id'] + 1,
        'license_key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($fixtureMismatch['license_key']),
        'request_id' => 'req-terminal-m-0002',
        'idempotency_key' => 'idem-terminal-m-0002',
    ]),
    'DUAL_DELIVERY_KEY_MISMATCH',
    'a terminal channel with a different license ID fails closed'
);

// Reused idempotency key with changed identity fails closed.
expect_dual_throws(
    static fn() => $coordinator->recordEmailOutcome([
        'registration_id' => $fixtureA['registration_id'],
        'delivery_status' => 'bounced',
        'bounce_type' => 'soft',
        'occurred_at' => '2026-08-08T00:05:00Z',
        'request_id' => 'req-email-a-0001',
        'idempotency_key' => 'idem-email-a-0001',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'a changed provider outcome cannot reuse an idempotency key'
);

// Bounded validation on provider outcomes.
expect_dual_throws(
    static fn() => $coordinator->recordEmailOutcome([
        'registration_id' => $fixtureA['registration_id'],
        'delivery_status' => 'bounced',
        'bounce_type' => 'soft',
        'occurred_at' => '2026-08-08T00:05:00Z',
        'request_id' => 'req-email-a-0002',
        'idempotency_key' => 'idem-email-a-0002',
    ]),
    'InvalidArgumentException',
    'an already-delivered email channel rejects later failure outcomes'
);
expect_dual_throws(
    static fn() => $coordinator->recordEmailOutcome([
        'registration_id' => $fixtureA['registration_id'],
        'delivery_status' => 'spam_flag',
        'occurred_at' => '2026-08-08T00:05:00Z',
        'request_id' => 'req-email-a-0003',
        'idempotency_key' => 'idem-email-a-0003',
    ]),
    'InvalidArgumentException',
    'an unknown provider status is rejected'
);

// ── Logging hygiene: no plaintext keys, no unmasked email ────────────────

// The approved email channel receives the full key by design (§16.1); every
// other returned structure and the journals must stay masked.
$mailPayloads = implode("\n", array_map(static fn(array $mail): string => json_encode($mail), $sentEmails));
expect_dual(substr_count($mailPayloads, $fixtureA['license_key']) === 2, 'the email channel carries the full key exactly once per send');
$allResponses = json_encode([
    $settle, $settleReplay, $settleAgain, $delivered, $deliveredReplay, $confirmed,
    $bounced, $partialB, $recoveredB, $recoverReplayB, $settledB,
    $partialC, $boundC, $recoveredC, $recoverReplayC,
    $suppressedD, $recoveredD, $settledD,
], JSON_THROW_ON_ERROR);
expect_dual(!str_contains($allResponses, $fixtureA['license_key']), 'no plaintext EDD key escapes into any returned structure');
expect_dual(!str_contains($allResponses, $fixtureB['license_key']), 'no second-key plaintext escapes into any returned structure');
expect_dual(!str_contains($allResponses, $fixtureC['license_key']), 'no terminal-recovery plaintext escapes into any returned structure');
expect_dual(!preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $allResponses), 'no unmasked email appears in any returned structure');

$journal = $db->query('SELECT * FROM wp_wpuiai_dual_license_deliveries')->fetchAll(PDO::FETCH_ASSOC);
$journalJson = json_encode($journal, JSON_THROW_ON_ERROR);
expect_dual(!str_contains($journalJson, $fixtureA['license_key']) && !str_contains($journalJson, $fixtureB['license_key']), 'dual-delivery journals never store the plaintext key');
expect_dual(!preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $journalJson), 'dual-delivery journals never store unmasked email');
expect_dual(!str_contains($journalJson, $fixtureC['license_key']), 'recovery envelopes in the journal are sealed only');

// ── Rollback is preservation-only ───────────────────────────────────────

$rollback = $dualMigration->preserveForRollback('2026-08-08T00:05:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_dual_delivery_rollback',
]);
expect_dual($rollback['action'] === 'preserve', 'dual-delivery rollback is preservation-only');
expect_dual($coordinator->deliveryCount() === (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_dual_license_deliveries')->fetchColumn(), 'rollback preserves the dual-delivery journal');
expect_dual((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_dual_license_delivery_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback is journaled');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.dual_license_delivery_validation.v1',
    'fixtures' => 'partial_failure_email_hard_bounce_terminal_ok_terminal_loss_email_ok_both_failed',
    'emails_sent' => count($sentEmails),
    'delivery_journals' => $coordinator->deliveryCount(),
    'edd_licenses_created' => $eddLicenseCount(),
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
