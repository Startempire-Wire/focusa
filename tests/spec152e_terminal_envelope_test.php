<?php
// 152E.04.01 One-time device-encrypted terminal key envelope. The authority seals the
// canonical EDD Software Licensing key to the registration's device X25519 public key,
// binds account/license/product/registration/expiry, delivers the envelope once through
// the activation poll response (or an identical idempotent replay), and keeps plaintext
// out of facade/access/generic logs. Only the bound device decrypts the exact EDD key;
// tampered, replayed, wrong-device, and expired envelopes fail closed. The crypto is
// RFC 7748 X25519 (pure PHP via GMP) + HKDF-SHA256 + AES-256-GCM and is byte-compatible
// with the Python `cryptography` vectors in
// docs/contracts/spec152e-terminal-envelope-golden-vectors.v1.json.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-terminal-delivery-envelope.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_envelope(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_envelope_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'terminal_envelope_test']);
$envelopeMigration = new FocusaSpec152eTerminalDeliveryEnvelopeMigration($db, 'wp_');
$envelopeMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'terminal_envelope_test']);
$envelopeMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'repeat_must_preserve_first_schema_application']);
$migrations = $db->query('SELECT * FROM wp_wpuiai_terminal_delivery_envelope_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_envelope(count($migrations) === 1, 'envelope migration is version-idempotent');
expect_envelope($migrations[0]['applied_at'] === '2026-08-08T00:00:00Z', 'envelope migration preserves first application time');

$columns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_terminal_delivery_envelopes)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $columns[$column['name']] = $column;
}
foreach (['envelope_id', 'registration_uuid', 'account_uuid', 'edd_customer_id', 'edd_license_id',
    'product_code', 'license_key_digest', 'license_key_mask', 'device_public_key', 'envelope_payload',
    'delivery_status', 'consumed_at', 'issued_at', 'expires_at', 'request_id', 'idempotency_key',
    'request_digest', 'created_at', 'retention_until', 'updated_at'] as $field) {
    expect_envelope(isset($columns[$field]), "envelope journal contains {$field}");
}
expect_envelope($columns['license_key_digest']['notnull'] === 1, 'journal never stores the plaintext key');
expect_envelope($columns['envelope_payload']['notnull'] === 1, 'journal stores the sealed payload for idempotent replay');

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

$service = new FocusaSpec152eTerminalEnvelopeService($db, $envelopeMigration, $registrations, $registrationSecrets, $clock);

// A mailbox-verified, promoted, paid registration with the canonical EDD license bound.
$created = $registrations->createPending([
    'email' => 'synthetic.operator@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'safe_redirect_handle' => 'success',
    'request_id' => 'req-envelope-0001',
    'idempotency_key' => 'idem-envelope-0001',
]);
$registrationId = $created['registration']['registration_uuid'];
$pollCredential = $created['poll_credential'];
$verified = $registrations->verifyEmail($registrationId, $created['verification_secret'], 'req-envelope-0002', 'idem-envelope-0002');
$promoted = $registrations->promoteVerified($registrationId, '018f47c2-6ac0-7b16-8d1a-4e93df5a0102', 41001, 'req-envelope-0003', 'idem-envelope-0003');
$offer = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
    FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, (int) $promoted['registration']['state_version'], 'req-envelope-0004', 'idem-envelope-0004',
    ['offer_code' => 'focusa_operator', 'journey' => 'purchase']);
$checkout = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
    FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING, (int) $offer['registration']['state_version'],
    'req-envelope-0005', 'idem-envelope-0005', ['edd_cart_reference' => 'cart_opaque_0001']);
$issued = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
    FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED, (int) $checkout['registration']['state_version'],
    'req-envelope-0006', 'idem-envelope-0006', ['edd_order_id' => 501, 'edd_order_item_id' => 502, 'edd_license_id' => 503]);
$ready = $registrations->transition($registrationId, FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
    FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, (int) $issued['registration']['state_version'],
    'req-envelope-0007', 'idem-envelope-0007');
expect_envelope($ready['registration']['state'] === FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, 'registration reaches terminal_delivery_ready');
expect_envelope($ready['registration']['terminal_delivery_status'] === 'ready', 'terminal delivery is marked ready');

$canonicalKey = 'ABCD1234-EFAB5678-90CD1234-EFAB5678';
$db->exec("INSERT INTO wp_edd_licenses
    (id, license_key, customer_id, user_id, product_id, order_id, license_length, license_unit,
     expiration, activation_count, activation_limit, status, date_created)
    VALUES (503, '" . $canonicalKey . "', 41001, NULL, 1001, 501, 0, 'years', NULL, 0, 5, 'active', '2026-08-08T00:00:00Z')");

// Device X25519 keypair: only the device private key ever leaves the device.
$devicePrivate = random_bytes(32);
$devicePublicRaw = FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($devicePrivate);
$devicePublicB64 = b64url_encode_php($devicePublicRaw);
$otherDevicePrivate = random_bytes(32);
$otherDevicePublicB64 = b64url_encode_php(FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($otherDevicePrivate));

// ── RFC 7748 compliance (golden scalar vectors) ─────────────────────────

$rfcK1 = hex2bin('a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4');
$rfcU1 = hex2bin('e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c');
expect_envelope(hash_equals(hex2bin('c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552'), FocusaSpec152eTerminalEnvelopeCrypto::scalarMult($rfcK1, $rfcU1)), 'RFC 7748 X25519 vector 1');
$rfcK2 = hex2bin('4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d');
$rfcU2 = hex2bin('e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493');
expect_envelope(hash_equals(hex2bin('95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957'), FocusaSpec152eTerminalEnvelopeCrypto::scalarMult($rfcK2, $rfcU2)), 'RFC 7748 X25519 vector 2');

// ── Cross-language golden vector (byte-exact with Python cryptography) ───

$vectors = json_decode(file_get_contents($root . '/docs/contracts/spec152e-terminal-envelope-golden-vectors.v1.json'), true, 512, JSON_THROW_ON_ERROR);
expect_envelope($vectors['schema'] === 'focusa.spec152e.terminal_envelope_golden_vectors.v1', 'golden vector schema');
expect_envelope($vectors['fixture_kind'] === 'public_synthetic_nonproduction', 'golden vector is a public synthetic fixture');
$goldenDevicePrivate = hex2bin($vectors['device_private_key_hex']);
$goldenDevicePublic = FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($goldenDevicePrivate);
expect_envelope(hash_equals(hex2bin($vectors['device_public_key_hex']), $goldenDevicePublic), 'golden device public key derives identically');
$goldenEphPrivate = hex2bin($vectors['ephemeral_private_key_hex']);
$goldenEphPublic = FocusaSpec152eTerminalEnvelopeCrypto::publicKeyFromPrivate($goldenEphPrivate);
expect_envelope(hash_equals(hex2bin($vectors['ephemeral_public_key_hex']), $goldenEphPublic), 'golden ephemeral public key derives identically');
$goldenShared = FocusaSpec152eTerminalEnvelopeCrypto::deriveSharedSecret($goldenEphPrivate, $goldenDevicePublic);
expect_envelope(hash_equals(hex2bin($vectors['shared_secret_hex']), $goldenShared), 'golden shared secret derives identically');
$goldenKey = FocusaSpec152eTerminalEnvelopeCrypto::hkdf($goldenShared);
expect_envelope(hash_equals(hex2bin($vectors['derived_key_hex']), $goldenKey), 'golden HKDF key derives identically');
$goldenClaims = json_decode($vectors['canonical_claims_json'], true, 512, JSON_THROW_ON_ERROR);
$goldenPlaintext = FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($goldenClaims);
expect_envelope($goldenPlaintext === $vectors['canonical_claims_json'], 'golden claims canonical JSON is byte-exact');
$goldenEnvelope = FocusaSpec152eTerminalEnvelopeCrypto::seal(
    $goldenDevicePublic,
    $goldenPlaintext,
    $goldenEphPrivate,
    b64url_decode_php($vectors['nonce_b64url']),
);
expect_envelope($goldenEnvelope === $vectors['envelope'], 'PHP envelope is byte-identical to the Python golden vector');
expect_envelope(FocusaSpec152eTerminalEnvelopeCrypto::open($goldenDevicePrivate, $goldenEnvelope) === $goldenPlaintext, 'golden envelope opens with the device private key');

// ── Positive delivery: only the bound device decrypts the exact EDD key ──

$poll = $service->deliverPollResponse([
    'registration_id' => $registrationId,
    'poll_credential' => $pollCredential,
    'device_public_key' => $devicePublicB64,
    'request_id' => 'req-poll-envelope-0001',
    'idempotency_key' => 'idem-poll-envelope-0001',
]);
expect_envelope($poll['schema'] === FocusaSpec152eTerminalEnvelopeService::POLL_RESPONSE_SCHEMA, 'poll response uses the activation response schema');
expect_envelope($poll['registration_id'] === $registrationId, 'poll response binds the registration');
expect_envelope($poll['terminal_delivery_status'] === 'delivered', 'poll response marks terminal delivery delivered');
expect_envelope($poll['license_key_mask'] === '********-********-********-5678', 'poll response exposes only the masked key');
expect_envelope((bool) preg_match('/^env_[0-9a-f]{32}$/D', $poll['envelope_id']), 'poll response returns a bounded envelope ID');
expect_envelope(str_contains($poll['one_time_key_envelope'], '') && !str_contains($poll['one_time_key_envelope'], $canonicalKey), 'plaintext key never appears in the poll response');
expect_envelope(!isset($poll['poll_credential'], $poll['poll_credential_hash'], $poll['encrypted_normalized_email'], $poll['email']), 'poll response never returns credential or email fields');

$envelope = json_decode(b64url_decode_php($poll['one_time_key_envelope']), true, 512, JSON_THROW_ON_ERROR);
expect_envelope(is_array($envelope) && $envelope['schema'] === FocusaSpec152eTerminalEnvelopeCrypto::SCHEMA, 'one_time_key_envelope decodes to the envelope schema');

$claims = $service->openForDevice([
    'envelope' => $envelope,
    'device_private_key' => bin2hex($devicePrivate),
    'registration_id' => $registrationId,
    'now' => '2026-08-08T00:02:00Z',
]);
expect_envelope($claims['license_key'] === $canonicalKey, 'bound device decrypts the exact canonical EDD key');
expect_envelope($claims['registration_id'] === $registrationId, 'claims bind the registration');
expect_envelope($claims['account_uuid'] === '018f47c2-6ac0-7b16-8d1a-4e93df5a0102', 'claims bind the account');
expect_envelope((int) $claims['customer_id'] === 41001, 'claims bind the EDD customer');
expect_envelope((int) $claims['edd_license_id'] === 503, 'claims bind the EDD license');
expect_envelope($claims['product_code'] === 'focusa_operator', 'claims bind the product');
expect_envelope($claims['one_time'] === true, 'claims mark the envelope one-time');
expect_envelope($claims['expires_at'] === '2026-08-08T00:31:00Z', 'envelope expiry is bounded by the envelope TTL');
expect_envelope($claims['expires_at'] <= '2026-08-08T01:01:00Z', 'envelope expiry never exceeds the registration expiry');
expect_envelope($claims['envelope_id'] === $poll['envelope_id'], 'claims envelope ID matches the journal handle');

$registrationAfter = $registrations->findByUuid($registrationId);
expect_envelope(hash_equals((string) $registrationAfter['device_public_key'], $devicePublicB64), 'registration records the device public key');
expect_envelope((string) $registrationAfter['terminal_delivery_status'] === 'delivered', 'registration records one-time delivery');
expect_envelope((int) $registrationAfter['delivery_attempts'] === 1, 'registration records exactly one delivery attempt');
expect_envelope($service->envelopeCount() === 1, 'exactly one envelope journal row is created');

// ── Idempotent replay returns the identical envelope ────────────────────

$replay = $service->deliverPollResponse([
    'registration_id' => $registrationId,
    'poll_credential' => $pollCredential,
    'device_public_key' => $devicePublicB64,
    'request_id' => 'req-poll-envelope-0001',
    'idempotency_key' => 'idem-poll-envelope-0001',
]);
expect_envelope($replay['envelope_id'] === $poll['envelope_id'], 'idempotent replay returns the same envelope ID');
expect_envelope($replay['one_time_key_envelope'] === $poll['one_time_key_envelope'], 'idempotent replay returns the identical envelope');
expect_envelope($service->envelopeCount() === 1, 'replay never mints a second envelope');
expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId,
        'poll_credential' => $pollCredential,
        'device_public_key' => $devicePublicB64,
        'request_id' => 'req-poll-envelope-0009',
        'idempotency_key' => 'idem-poll-envelope-0001',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'changed poll request identity cannot reuse an idempotency key'
);

// ── One-time: a new poll after delivery fails closed, no second envelope ─

expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId,
        'poll_credential' => $pollCredential,
        'device_public_key' => $devicePublicB64,
        'request_id' => 'req-poll-envelope-0002',
        'idempotency_key' => 'idem-poll-envelope-0002',
    ]),
    'LICENSE_DELIVERY_FAILED',
    'terminal delivery is one-time; a new poll cannot reveal the key again'
);
expect_envelope($service->envelopeCount() === 1, 'one-time delivery mints exactly one envelope');

// ── Wrong-device, tamper, replay, expired, binding fail closed ───────────

expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $envelope,
        'device_private_key' => bin2hex($otherDevicePrivate),
        'registration_id' => $registrationId,
        'now' => '2026-08-08T00:02:00Z',
    ]),
    'ENVELOPE_AUTH_FAILED',
    'a non-bound device cannot decrypt the envelope'
);

$tampered = $envelope;
$tamperedCipher = b64url_decode_php($tampered['ciphertext']);
$tamperedCipher[0] = chr(ord($tamperedCipher[0]) ^ 0x01);
$tampered['ciphertext'] = b64url_encode_php($tamperedCipher);
expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $tampered,
        'device_private_key' => bin2hex($devicePrivate),
        'registration_id' => $registrationId,
        'now' => '2026-08-08T00:02:00Z',
    ]),
    'ENVELOPE_AUTH_FAILED',
    'tampered ciphertext fails closed'
);

$tamperedHeader = $envelope;
$tamperedNonce = b64url_decode_php($tamperedHeader['nonce']);
$tamperedNonce[0] = chr(ord($tamperedNonce[0]) ^ 0x01);
$tamperedHeader['nonce'] = b64url_encode_php($tamperedNonce);
expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $tamperedHeader,
        'device_private_key' => bin2hex($devicePrivate),
        'registration_id' => $registrationId,
        'now' => '2026-08-08T00:02:00Z',
    ]),
    'ENVELOPE_AUTH_FAILED',
    'tampered envelope header fails closed via AAD binding'
);

$tamperedClaims = $envelope;
$tamperedClaims['algorithm'] = 'X25519+HKDF-SHA256+AES-256-GCM-tampered';
expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $tamperedClaims,
        'device_private_key' => bin2hex($devicePrivate),
        'registration_id' => $registrationId,
        'now' => '2026-08-08T00:02:00Z',
    ]),
    'ENVELOPE_FORMAT_DENIED',
    'unknown envelope algorithm fails closed'
);

expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $envelope,
        'device_private_key' => bin2hex($devicePrivate),
        'registration_id' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0999',
        'now' => '2026-08-08T00:02:00Z',
    ]),
    'ENVELOPE_BINDING_MISMATCH',
    'an envelope replayed against another registration fails closed'
);

expect_envelope_throws(
    static fn() => $service->openForDevice([
        'envelope' => $envelope,
        'device_private_key' => bin2hex($devicePrivate),
        'registration_id' => $registrationId,
        'now' => '2026-08-09T00:02:00Z',
    ]),
    'ENVELOPE_EXPIRED',
    'an expired envelope fails closed'
);

// ── Poll validation fail-closed paths ───────────────────────────────────

expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId,
        'poll_credential' => 'wrong-credential',
        'device_public_key' => $devicePublicB64,
        'request_id' => 'req-poll-envelope-0003',
        'idempotency_key' => 'idem-poll-envelope-0003',
    ]),
    'POLL_CREDENTIAL_REQUIRED',
    'a wrong poll credential fails closed'
);
expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId,
        'poll_credential' => $pollCredential,
        'device_public_key' => 'not-a-valid-base64url-device-key',
        'request_id' => 'req-poll-envelope-0004',
        'idempotency_key' => 'idem-poll-envelope-0004',
    ]),
    'NODE_PUBLIC_KEY_REQUIRED',
    'a malformed device public key fails closed'
);

// A second registration whose delivery is not yet ready cannot receive an envelope;
// a revoked canonical license fails closed even when delivery is ready.
$created2 = $registrations->createPending([
    'email' => 'synthetic.operator2@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'request_id' => 'req-envelope-0011',
    'idempotency_key' => 'idem-envelope-0011',
]);
$registrationId2 = $created2['registration']['registration_uuid'];
$verified2 = $registrations->verifyEmail($registrationId2, $created2['verification_secret'], 'req-envelope-0012', 'idem-envelope-0012');
$promoted2 = $registrations->promoteVerified($registrationId2, '018f47c2-6ac0-7b16-8d1a-4e93df5a0103', 41002, 'req-envelope-0013', 'idem-envelope-0013');
$offer2 = $registrations->transition($registrationId2, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
    FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, (int) $promoted2['registration']['state_version'], 'req-envelope-0014', 'idem-envelope-0014',
    ['offer_code' => 'focusa_operator', 'journey' => 'purchase']);
$checkout2 = $registrations->transition($registrationId2, FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
    FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING, (int) $offer2['registration']['state_version'],
    'req-envelope-0015', 'idem-envelope-0015', ['edd_cart_reference' => 'cart_opaque_0002']);
$issued2 = $registrations->transition($registrationId2, FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
    FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED, (int) $checkout2['registration']['state_version'],
    'req-envelope-0016', 'idem-envelope-0016', ['edd_order_id' => 601, 'edd_order_item_id' => 602, 'edd_license_id' => 604]);
expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId2,
        'poll_credential' => $created2['poll_credential'],
        'device_public_key' => $otherDevicePublicB64,
        'request_id' => 'req-poll-envelope-0017',
        'idempotency_key' => 'idem-poll-envelope-0017',
    ]),
    'LICENSE_DELIVERY_PENDING',
    'a registration without terminal_delivery_ready cannot receive an envelope'
);

// The revoked license (604) fails closed at terminal delivery.
$db->exec("INSERT INTO wp_edd_licenses
    (id, license_key, customer_id, user_id, product_id, order_id, license_length, license_unit,
     expiration, activation_count, activation_limit, status, date_created)
    VALUES (604, 'ABCD5678-90CD1234-EFAB5678-90CD1234', 41002, NULL, 1001, 601, 0, 'years', NULL, 0, 5, 'revoked', '2026-08-08T00:00:00Z')");
$ready2 = $registrations->transition($registrationId2, FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
    FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, (int) $issued2['registration']['state_version'],
    'req-envelope-0018', 'idem-envelope-0018');
expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId2,
        'poll_credential' => $created2['poll_credential'],
        'device_public_key' => $otherDevicePublicB64,
        'request_id' => 'req-poll-envelope-0019',
        'idempotency_key' => 'idem-poll-envelope-0019',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'a revoked canonical license fails closed at terminal delivery'
);

// ── Expired poll credential fails closed ────────────────────────────────

$db->exec("UPDATE wp_wpuiai_activation_registrations SET poll_credential_expires_at = '2026-08-08T00:00:59Z' WHERE registration_uuid = '" . $registrationId2 . "'");
expect_envelope_throws(
    static fn() => $service->deliverPollResponse([
        'registration_id' => $registrationId2,
        'poll_credential' => $created2['poll_credential'],
        'device_public_key' => $otherDevicePublicB64,
        'request_id' => 'req-poll-envelope-0022',
        'idempotency_key' => 'idem-poll-envelope-0022',
    ]),
    'POLL_CREDENTIAL_EXPIRED',
    'an expired poll credential fails closed'
);

// ── Device key binding surface: mismatch and authority denial ───────────

expect_envelope_throws(
    static fn() => $registrations->bindDevicePublicKey($registrationId, $otherDevicePublicB64, 'req-bind-0001', 'idem-bind-0001'),
    'NODE_KEY_MISMATCH',
    're-binding to a different device key fails closed'
);
expect_envelope_throws(
    static fn() => $registrations->bindDevicePublicKey($registrationId, $devicePublicB64, 'req-bind-0002', 'idem-bind-0002', ['account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0102']),
    'PENDING_AUTHORITY_FIELD_DENIED',
    'binding context is restricted to terminal-delivery fields'
);
expect_envelope_throws(
    static fn() => $registrations->bindDevicePublicKey($registrationId, 'malformed-device-key-value', 'req-bind-0003', 'idem-bind-0003'),
    'NODE_PUBLIC_KEY_REQUIRED',
    'malformed device keys are rejected by the repository surface'
);

// ── Protected credential adapter: masked store, one-time reveal ─────────

$store = [];
$confirmation = FocusaSpec152eTerminalCredentialAdapter::storeConfirmation('handle-terminal-0001', FocusaSpec152eTerminalDeliveryEnvelope::maskKey($claims['license_key']));
expect_envelope($confirmation['mask'] === '********-********-********-5678', 'adapter store confirms only the masked key');
expect_envelope(!str_contains(json_encode($confirmation), $canonicalKey), 'adapter store confirmation never exposes the plaintext key');

$revealed = FocusaSpec152eTerminalCredentialAdapter::reveal(
    'handle-terminal-0001',
    true,
    $claims,
    '2026-08-08T00:02:00Z',
    static function (string $handle) use (&$store): bool {
        if (($store[$handle] ?? false) === true) {
            return false;
        }
        $store[$handle] = true;
        return true;
    },
);
expect_envelope($revealed['revealed'] === true && $revealed['license_key'] === $canonicalKey, 'explicit customer consent reveals the key once');
expect_envelope_throws(
    static fn() => FocusaSpec152eTerminalCredentialAdapter::reveal(
        'handle-terminal-0001',
        true,
        $claims,
        '2026-08-08T00:02:00Z',
        static function (string $handle) use (&$store): bool {
            return ($store[$handle] ?? false) !== true;
        },
    ),
    'CREDENTIAL_REVEAL_DENIED',
    'a replayed reveal fails closed after one-time consumption'
);
expect_envelope_throws(
    static fn() => FocusaSpec152eTerminalCredentialAdapter::reveal(
        'handle-terminal-0002',
        false,
        $claims,
        '2026-08-08T00:02:00Z',
        static fn(string $handle): bool => true,
    ),
    'CREDENTIAL_REVEAL_DENIED',
    'reveal without explicit customer consent fails closed'
);
expect_envelope_throws(
    static fn() => FocusaSpec152eTerminalCredentialAdapter::reveal(
        'handle-terminal-0003',
        true,
        $claims,
        '2026-08-09T00:02:00Z',
        static fn(string $handle): bool => true,
    ),
    'ENVELOPE_EXPIRED',
    'reveal after the envelope lifetime fails closed'
);

// ── Rollback is preservation-only ───────────────────────────────────────

$rollback = $envelopeMigration->preserveForRollback('2026-08-08T00:05:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_envelope_rollback',
]);
expect_envelope($rollback['action'] === 'preserve', 'envelope rollback is preservation-only');
expect_envelope($service->envelopeCount() === 1, 'rollback preserves the envelope journal');
expect_envelope((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_terminal_delivery_envelope_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback is journaled');

// ── Logging hygiene: no secrets, no unmasked real email ─────────────────

// The one-time reveal result is the explicit customer-controlled exception
// (spec §14.2 reveal mode); every other returned structure must stay clean.
$serialized = json_encode([$poll, $replay, $confirmation, $registrationAfter], JSON_THROW_ON_ERROR);
expect_envelope(!str_contains($serialized, $canonicalKey), 'no plaintext EDD key escapes into any returned structure');
expect_envelope(!preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $serialized), 'no unmasked real email appears in any returned structure');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.terminal_envelope_validation.v1',
    'rfc7748_vectors' => 2,
    'golden_vectors' => 'byte_exact_cross_language',
    'envelopes_issued' => $service->envelopeCount(),
    'delivery_attempts' => (int) $registrationAfter['delivery_attempts'],
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
