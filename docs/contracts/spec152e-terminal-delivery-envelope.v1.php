<?php
// One-time device-encrypted terminal key envelope. The authority encrypts the
// canonical EDD Software Licensing key to the registration's device public key
// (X25519, RFC 7748, pure-PHP via GMP; HKDF-SHA256; AES-256-GCM), binds
// account/license/product/registration/expiry, delivers the envelope once
// through the activation poll response or an authenticated retry, and keeps
// plaintext out of facade/access/generic logs. Spec 158 implementation is
// excluded. The crypto is deterministic and byte-compatible with libsodium and
// the Python `cryptography` X25519 exchange semantics (RFC 7748 clamping).
declare(strict_types=1);

final class FocusaSpec152eTerminalEnvelopeCrypto
{
    public const SCHEMA = 'focusa.spec152e.terminal_delivery_envelope.v1';
    public const ALGORITHM = 'X25519+HKDF-SHA256+AES-256-GCM';
    public const VERSION = 1;
    public const INFO = "focusa.spec152e.terminal_delivery_envelope.v1\0hkdf";
    public const KEY_BYTES = 32;
    public const NONCE_BYTES = 12;
    public const TAG_BYTES = 16;

    private const P = '57896044618658097711785492504343953926634992332820282019728792003956564819949';
    private const A24 = '121665';

    /** RFC 7748 scalar multiplication on Curve25519 (pure PHP, GMP). */
    public static function scalarMult(string $k, string $u): string
    {
        if (strlen($k) !== self::KEY_BYTES || strlen($u) !== self::KEY_BYTES) {
            throw new InvalidArgumentException('x25519 requires 32-byte scalar and u-coordinate');
        }
        $p = gmp_init(self::P);
        $a24 = gmp_init(self::A24);

        $kb = unpack('C*', $k);
        $kb[1] &= 248;
        $kb[32] &= 127;
        $kb[32] |= 64;
        $k = pack('C*', ...$kb);

        $ub = unpack('C*', $u);
        $ub[32] &= 127;
        $u = pack('C*', ...$ub);

        $x1 = self::decodeLittleEndian($u);
        $x2 = gmp_init(1);
        $z2 = gmp_init(0);
        $x3 = $x1;
        $z3 = gmp_init(1);
        $swap = 0;
        $scalar = self::decodeLittleEndian($k);

        for ($t = 254; $t >= 0; $t--) {
            $kt = (int) gmp_testbit($scalar, $t);
            $swap ^= $kt;
            if ($swap === 1) {
                [$x2, $x3] = [$x3, $x2];
                [$z2, $z3] = [$z3, $z2];
            }
            $swap = $kt;

            $A = gmp_mod(gmp_add($x2, $z2), $p);
            $AA = gmp_mod(gmp_mul($A, $A), $p);
            $B = gmp_mod(gmp_sub($x2, $z2), $p);
            $BB = gmp_mod(gmp_mul($B, $B), $p);
            $E = gmp_mod(gmp_sub($AA, $BB), $p);
            $C = gmp_mod(gmp_add($x3, $z3), $p);
            $D = gmp_mod(gmp_sub($x3, $z3), $p);
            $DA = gmp_mod(gmp_mul($D, $A), $p);
            $CB = gmp_mod(gmp_mul($C, $B), $p);
            $x3 = gmp_mod(gmp_pow(gmp_mod(gmp_add($DA, $CB), $p), 2), $p);
            $z3 = gmp_mod(gmp_mul($x1, gmp_pow(gmp_mod(gmp_sub($DA, $CB), $p), 2)), $p);
            $x2 = gmp_mod(gmp_mul($AA, $BB), $p);
            $z2 = gmp_mod(gmp_mul($E, gmp_mod(gmp_add($AA, gmp_mul($a24, $E)), $p)), $p);
        }
        if ($swap === 1) {
            [$x2, $x3] = [$x3, $x2];
            [$z2, $z3] = [$z3, $z2];
        }
        $z2Inv = gmp_powm($z2, gmp_sub($p, gmp_init(2)), $p);
        return self::encodeLittleEndian(gmp_mod(gmp_mul($x2, $z2Inv), $p), self::KEY_BYTES);
    }

    /** X25519 public key for a raw 32-byte private key (RFC 7748 clamped). */
    public static function publicKeyFromPrivate(string $privateKey32): string
    {
        self::assertBytes($privateKey32, self::KEY_BYTES, 'private key');
        return self::scalarMult($privateKey32, str_repeat("\x09", self::KEY_BYTES));
    }

    /** X25519 ECDH shared secret; all-zero output is rejected (RFC 7748). */
    public static function deriveSharedSecret(string $privateKey32, string $peerPublic32): string
    {
        $shared = self::scalarMult($privateKey32, $peerPublic32);
        if (gmp_cmp(self::decodeLittleEndian($shared), 0) === 0) {
            throw new DomainException('ENVELOPE_DEVICE_KEY_DENIED');
        }
        return $shared;
    }

    /**
     * Seal a plaintext string to a device public key. Returns the public
     * envelope header plus base64url ciphertext. A fresh ephemeral X25519 key
     * and nonce are generated per seal; deterministic test seams accept fixed
     * inputs so golden vectors stay byte-exact across languages. The canonical
     * header is bound as AES-GCM AAD.
     *
     * @param string      $devicePublic32    32-byte raw device public key
     * @param string      $plaintext         authenticated plaintext (claims JSON)
     * @param string|null $ephemeralPrivate32 fixed ephemeral private key (test seam)
     * @param string|null $nonce12            fixed 12-byte nonce (test seam)
     */
    public static function seal(
        string $devicePublic32,
        string $plaintext,
        ?string $ephemeralPrivate32 = null,
        ?string $nonce12 = null,
    ): array {
        self::assertBytes($devicePublic32, self::KEY_BYTES, 'device public key');
        $ephemeralPrivate = $ephemeralPrivate32 ?? random_bytes(self::KEY_BYTES);
        $nonce = $nonce12 ?? random_bytes(self::NONCE_BYTES);
        self::assertBytes($ephemeralPrivate, self::KEY_BYTES, 'ephemeral private key');
        self::assertBytes($nonce, self::NONCE_BYTES, 'nonce');

        $ephemeralPublic = self::publicKeyFromPrivate($ephemeralPrivate);
        $shared = self::deriveSharedSecret($ephemeralPrivate, $devicePublic32);
        $key = self::hkdf($shared);
        $envelope = [
            'schema' => self::SCHEMA,
            'version' => self::VERSION,
            'algorithm' => self::ALGORITHM,
            'ephemeral_public_key' => self::base64UrlEncode($ephemeralPublic),
            'nonce' => self::base64UrlEncode($nonce),
        ];
        $tag = '';
        $ciphertext = openssl_encrypt(
            $plaintext,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $nonce,
            $tag,
            self::canonicalAad($envelope),
        );
        if ($ciphertext === false || strlen($tag) !== self::TAG_BYTES) {
            throw new DomainException('ENVELOPE_ENCRYPTION_FAILED');
        }
        $envelope['ciphertext'] = self::base64UrlEncode($ciphertext . $tag);
        return $envelope;
    }

    /**
     * Open an envelope with the device private key. Fails closed on tamper,
     * wrong device, malformed structure, or any all-zero shared secret.
     * Returns the authenticated plaintext; expiry is validated by the caller
     * against the embedded claims.
     */
    public static function open(string $devicePrivate32, array $envelope): string
    {
        self::assertBytes($devicePrivate32, self::KEY_BYTES, 'device private key');
        if (!is_array($envelope)
            || (string) ($envelope['schema'] ?? '') !== self::SCHEMA
            || (int) ($envelope['version'] ?? 0) !== self::VERSION
            || (string) ($envelope['algorithm'] ?? '') !== self::ALGORITHM) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $ephemeralPublic = self::base64UrlDecode((string) ($envelope['ephemeral_public_key'] ?? ''));
        $nonce = self::base64UrlDecode((string) ($envelope['nonce'] ?? ''));
        $sealed = self::base64UrlDecode((string) ($envelope['ciphertext'] ?? ''));
        if (strlen($ephemeralPublic) !== self::KEY_BYTES
            || strlen($nonce) !== self::NONCE_BYTES
            || strlen($sealed) <= self::TAG_BYTES) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $shared = self::deriveSharedSecret($devicePrivate32, $ephemeralPublic);
        $key = self::hkdf($shared);
        $tag = substr($sealed, -self::TAG_BYTES);
        $body = substr($sealed, 0, -self::TAG_BYTES);
        $plaintext = openssl_decrypt(
            $body,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $nonce,
            $tag,
            self::canonicalAad($envelope),
        );
        if ($plaintext === false) {
            throw new DomainException('ENVELOPE_AUTH_FAILED');
        }
        return $plaintext;
    }

    public static function canonicalAad(array $envelope): string
    {
        return self::canonicalJson([
            'algorithm' => (string) ($envelope['algorithm'] ?? ''),
            'ephemeral_public_key' => (string) ($envelope['ephemeral_public_key'] ?? ''),
            'nonce' => (string) ($envelope['nonce'] ?? ''),
            'schema' => (string) ($envelope['schema'] ?? ''),
            'version' => (int) ($envelope['version'] ?? 0),
        ]);
    }

    public static function hkdf(string $sharedSecret): string
    {
        $key = hash_hkdf('sha256', $sharedSecret, 32, self::INFO, '');
        if (!is_string($key) || strlen($key) !== 32) {
            throw new DomainException('ENVELOPE_ENCRYPTION_FAILED');
        }
        return $key;
    }

    public static function base64UrlEncode(string $binary): string
    {
        return rtrim(strtr(base64_encode($binary), '+/', '-_'), '=');
    }

    public static function base64UrlDecode(string $encoded): string
    {
        if ($encoded === '' || strlen($encoded) > 4096 || preg_match('/[\r\n\x00]/', $encoded) === 1) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $padding = (4 - strlen($encoded) % 4) % 4;
        $decoded = base64_decode(strtr($encoded . str_repeat('=', $padding), '-_', '+/'), true);
        if ($decoded === false) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        return $decoded;
    }

    /** Canonical JSON with sorted keys and compact separators (Python-compatible). */
    public static function canonicalJson(array $value): string
    {
        $normalize = static function (mixed $item) use (&$normalize): mixed {
            if (!is_array($item)) {
                return $item;
            }
            if (!array_is_list($item)) {
                ksort($item, SORT_STRING);
            }
            foreach ($item as $key => $child) {
                $item[$key] = $normalize($child);
            }
            return $item;
        };
        return json_encode($normalize($value), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }

    private static function decodeLittleEndian(string $bytes): \GMP
    {
        return gmp_import(strrev($bytes), 1, GMP_MSW_FIRST | GMP_BIG_ENDIAN);
    }

    private static function encodeLittleEndian(\GMP $value, int $length): string
    {
        $encoded = gmp_export($value, $length, GMP_MSW_FIRST | GMP_BIG_ENDIAN);
        if (!is_string($encoded) || strlen($encoded) !== $length) {
            throw new DomainException('ENVELOPE_ENCRYPTION_FAILED');
        }
        return strrev($encoded);
    }

    private static function assertBytes(string $value, int $length, string $kind): void
    {
        if (strlen($value) !== $length) {
            throw new InvalidArgumentException("{$kind} must be {$length} bytes");
        }
    }
}

/**
 * Claims schema and binding validation for the one-time terminal envelope.
 * Claims are sealed inside the ciphertext; every semantic field is validated
 * fail-closed before a device may use the key.
 */
final class FocusaSpec152eTerminalDeliveryEnvelope
{
    public const SCHEMA = 'focusa.spec152e.terminal_delivery_envelope.v1';
    public const ENVELOPE_ID_PATTERN = '/^env_[0-9a-f]{32}$/D';
    public const KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
    public const PRODUCT_CODE_PATTERN = '/^[A-Za-z0-9_]{2,128}$/D';
    public const UUID_PATTERN = '/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/D';

    private const REQUIRED_CLAIMS = [
        'schema', 'envelope_id', 'registration_id', 'account_uuid', 'customer_id',
        'edd_license_id', 'product_code', 'license_key', 'issued_at', 'expires_at', 'one_time',
    ];

    public static function buildClaims(array $binding, string $licenseKey, string $envelopeId, string $issuedAt, string $expiresAt): array
    {
        self::assertTimestamp($issuedAt);
        self::assertTimestamp($expiresAt);
        if (preg_match(self::ENVELOPE_ID_PATTERN, $envelopeId) !== 1) {
            throw new InvalidArgumentException('bounded envelope ID required');
        }
        if (preg_match(self::KEY_PATTERN, $licenseKey) !== 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $claims = [
            'schema' => self::SCHEMA,
            'envelope_id' => $envelopeId,
            'registration_id' => (string) ($binding['registration_id'] ?? ''),
            'account_uuid' => (string) ($binding['account_uuid'] ?? ''),
            'customer_id' => (int) ($binding['customer_id'] ?? 0),
            'edd_license_id' => (int) ($binding['edd_license_id'] ?? 0),
            'product_code' => (string) ($binding['product_code'] ?? ''),
            'license_key' => $licenseKey,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
            'one_time' => true,
        ];
        self::assertClaims($claims, $issuedAt, null);
        return $claims;
    }

    /** Validate decoded claims; `now` is compared to expires_at when provided. */
    public static function assertClaims(array $claims, string $now, ?string $expectedRegistrationId): void
    {
        foreach (self::REQUIRED_CLAIMS as $field) {
            if (!array_key_exists($field, $claims)) {
                throw new DomainException('ENVELOPE_FORMAT_DENIED');
            }
        }
        if ((string) $claims['schema'] !== self::SCHEMA
            || preg_match(self::ENVELOPE_ID_PATTERN, (string) $claims['envelope_id']) !== 1
            || preg_match(self::UUID_PATTERN, (string) $claims['registration_id']) !== 1
            || preg_match(self::UUID_PATTERN, (string) $claims['account_uuid']) !== 1
            || (int) $claims['customer_id'] < 1
            || (int) $claims['edd_license_id'] < 1
            || preg_match(self::PRODUCT_CODE_PATTERN, (string) $claims['product_code']) !== 1
            || preg_match(self::KEY_PATTERN, (string) $claims['license_key']) !== 1
            || $claims['one_time'] !== true) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        self::assertTimestamp((string) $claims['issued_at']);
        self::assertTimestamp((string) $claims['expires_at']);
        if ((string) $claims['expires_at'] <= (string) $claims['issued_at']) {
            throw new DomainException('ENVELOPE_EXPIRED');
        }
        if ($expectedRegistrationId !== null
            && !hash_equals($expectedRegistrationId, (string) $claims['registration_id'])) {
            throw new DomainException('ENVELOPE_BINDING_MISMATCH');
        }
        if ((string) $claims['expires_at'] <= $now) {
            throw new DomainException('ENVELOPE_EXPIRED');
        }
    }

    public static function maskKey(string $licenseKey): string
    {
        $parts = explode('-', $licenseKey);
        $tail = (string) end($parts);
        return '********-********-********-' . substr($tail, -4);
    }

    public static function keyDigest(string $licenseKey): string
    {
        return hash('sha256', "focusa.spec152e.terminal_delivery_envelope.key.v1\0" . $licenseKey);
    }

    public static function canonicalJsonSafe(array $value): string
    {
        return FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($value);
    }

    public static function assertTimestamp(string $timestamp): void
    {
        $parsed = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }

    public static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }
}

/** Delivery journal: one envelope per delivery, idempotent replay, retention. */
final class FocusaSpec152eTerminalDeliveryEnvelopeMigration
{
    public const SCHEMA = 'focusa.spec152e.terminal_delivery_envelope.v1';
    public const VERSION = 1;

    public function __construct(private PDO $db, private string $prefix = 'wp_')
    {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($appliedAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $envelopes = $this->table('wpuiai_terminal_delivery_envelopes');
        $migrations = $this->table('wpuiai_terminal_delivery_envelope_schema_migrations');
        $events = $this->table('wpuiai_terminal_delivery_envelope_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$envelopes} (
            envelope_id VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            edd_customer_id BIGINT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            license_key_digest VARCHAR(64) NOT NULL,
            license_key_mask VARCHAR(32) NOT NULL,
            device_public_key TEXT NOT NULL,
            envelope_payload TEXT NOT NULL,
            delivery_status VARCHAR(16) NOT NULL CHECK (delivery_status IN ('issued', 'consumed', 'expired', 'superseded')),
            consumed_at VARCHAR(32) NULL,
            issued_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL UNIQUE,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_terminal_envelope_registration
            ON {$envelopes} (registration_uuid, delivery_status, expires_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_terminal_envelope_retention
            ON {$envelopes} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$events} (
            event_key VARCHAR(64) NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            schema_version BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");

        $statement = $this->db->prepare("INSERT INTO {$migrations}
            (schema_version, schema_name, applied_at, migration_provenance)
            SELECT :version, :schema, :applied, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)");
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied' => $appliedAt,
            ':provenance' => FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance),
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: envelope journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_terminal_delivery_envelope_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance));
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance),
            ':existing_key' => $eventKey,
        ]);
        return ['schema' => self::SCHEMA, 'action' => 'preserve', 'event_key' => $eventKey];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }
}

/**
 * Terminal delivery service. Poll responses return the one-time device-encrypted
 * envelope only while terminal delivery is ready and the exact device key is
 * bound; every failure path is public-safe and fail-closed. The plaintext EDD
 * key never leaves the sealed envelope in any response, journal, or log.
 */
final class FocusaSpec152eTerminalEnvelopeService
{
    public const SCHEMA = 'focusa.spec152e.terminal_delivery_service.v1';
    public const POLL_RESPONSE_SCHEMA = 'focusa.activation.response.v1';
    public const ENVELOPE_TTL_SECONDS = 1800;
    public const RETENTION_SECONDS = 2592000;
    public const DEVICE_KEY_PATTERN = '/^[A-Za-z0-9_-]{43}$/D';
    public const DELIVERY_READY_STATES = [
        FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
        FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY,
        FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED,
        FocusaSpec152eActivationRegistrationState::LEASE_ISSUED,
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eTerminalDeliveryEnvelopeMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        callable $clock,
        private string $eddPrefix = 'wp_',
        private int $envelopeTtl = self::ENVELOPE_TTL_SECONDS,
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $eddPrefix) !== 1) {
            throw new InvalidArgumentException('invalid EDD table prefix');
        }
        if ($this->envelopeTtl < 1 || $this->retention < 1) {
            throw new InvalidArgumentException('positive envelope TTL and retention required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Poll response for terminal delivery. Required input:
     *   - registration_id, poll_credential, device_public_key
     *   - request_id, idempotency_key
     *
     * Returns the public-safe activation response containing
     * `one_time_key_envelope` (base64url sealed envelope) exactly once. An
     * idempotent replay returns the identical stored envelope; a new poll after
     * delivery fails closed with LICENSE_DELIVERY_FAILED; wrong device keys,
     * expired credentials, tampered state, and expired registrations fail
     * closed with the stable public codes.
     */
    public function deliverPollResponse(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $pollCredential = (string) ($input['poll_credential'] ?? '');
        $devicePublicKey = (string) ($input['device_public_key'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertUuid($registrationId, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if ($pollCredential === '' || strlen($pollCredential) > 512 || preg_match('/[\r\n]/', $pollCredential)) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if (preg_match(self::DEVICE_KEY_PATTERN, $devicePublicKey) !== 1) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }

        $now = $this->now();
        $credentialHash = $this->registrationSecrets->pollHash($pollCredential);
        $digest = hash('sha256', FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe([
            'operation' => 'terminal_delivery_poll',
            'registration_id' => $registrationId,
            'poll_hash' => $credentialHash,
            'device_public_key' => $devicePublicKey,
            'request_id' => $requestId,
        ]));

        return $this->transaction(function () use ($registrationId, $credentialHash, $devicePublicKey, $requestId, $idempotencyKey, $now, $digest): array {
            $replay = $this->findEnvelopeByIdempotency($idempotencyKey, $digest);
            if ($replay !== null) {
                return $this->pollResponseFromJournal($replay, $requestId);
            }
            $registration = $this->loadDeliveryRegistration($registrationId, $credentialHash, $now);
            $license = $this->resolveCanonicalKey((int) $registration['edd_license_id'], (string) $registration['product_code']);

            // Bind the device key and mark the one-time delivery in one
            // compare-and-set update; mismatched or already-delivered state
            // fails closed before any envelope is handed out.
            $bound = $this->registrations->bindDevicePublicKey(
                $registrationId,
                $devicePublicKey,
                $requestId,
                $idempotencyKey,
                ['terminal_delivery_status' => 'delivered', 'delivered_at' => $now],
            );
            $registration = $bound['registration'];

            $envelopeId = self::opaqueToken('env_');
            $issuedAt = $now;
            $expiresAt = min(
                FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, $this->envelopeTtl),
                (string) $registration['expires_at'],
            );
            $claims = FocusaSpec152eTerminalDeliveryEnvelope::buildClaims([
                'registration_id' => $registrationId,
                'account_uuid' => (string) $registration['account_uuid'],
                'customer_id' => (int) $registration['edd_customer_id'],
                'edd_license_id' => (int) $registration['edd_license_id'],
                'product_code' => (string) $registration['product_code'],
            ], $license['key'], $envelopeId, $issuedAt, $expiresAt);

            $envelope = FocusaSpec152eTerminalEnvelopeCrypto::seal(
                self::deviceKeyToRaw($devicePublicKey),
                FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($claims),
            );
            $this->journalEnvelope($registration, $license, $devicePublicKey, $envelope, $envelopeId, $issuedAt, $expiresAt, $requestId, $idempotencyKey, $digest, $now);

            return $this->pollResponse($registration, $requestId, $envelopeId, $license['mask'], $envelope);
        });
    }

    /**
     * Device-side open: decrypt the envelope with the device private key and
     * validate every claim fail-closed (tamper, wrong device, expired,
     * binding mismatch, malformed structure). Returns the authenticated claims;
     * the credential adapter receives the key through the protected store.
     *
     * Required input: envelope (array), device_private_key (hex or raw),
     * registration_id (expected binding), request_id.
     */
    public function openForDevice(array $input): array
    {
        $envelope = $input['envelope'] ?? null;
        $devicePrivate = self::devicePrivateFromInput((string) ($input['device_private_key'] ?? ''));
        $registrationId = (string) ($input['registration_id'] ?? '');
        $this->assertUuid($registrationId, 'registration');
        $now = (string) ($input['now'] ?? $this->now());
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        if (!is_array($envelope) || $envelope === []) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $plaintext = FocusaSpec152eTerminalEnvelopeCrypto::open($devicePrivate, $envelope);
        $claims = json_decode($plaintext, true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($claims)) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, $now, $registrationId);
        return $claims;
    }

    /** Bounded: exact journal lookup by envelope ID. */
    public function findEnvelope(string $envelopeId): ?array
    {
        if (preg_match(FocusaSpec152eTerminalDeliveryEnvelope::ENVELOPE_ID_PATTERN, $envelopeId) !== 1) {
            throw new InvalidArgumentException('bounded envelope ID required');
        }
        $table = $this->schema->table('wpuiai_terminal_delivery_envelopes');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE envelope_id = :id LIMIT 1");
        $statement->execute([':id' => $envelopeId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function envelopeCount(): int
    {
        $table = $this->schema->table('wpuiai_terminal_delivery_envelopes');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    // ── private ────────────────────────────────────────────────────────────

    private function loadDeliveryRegistration(string $registrationId, string $credentialHash, string $now): array
    {
        try {
            $registration = $this->registrations->findByUuid($registrationId);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if ($registration['expires_at'] !== null && $now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if ($registration['poll_credential_hash'] === null || $registration['poll_credential_expires_at'] === null) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if ($now >= (string) $registration['poll_credential_expires_at']) {
            throw new DomainException('POLL_CREDENTIAL_EXPIRED');
        }
        if (!hash_equals((string) $registration['poll_credential_hash'], $credentialHash)) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        $state = (string) $registration['state'];
        if (!in_array($state, self::DELIVERY_READY_STATES, true)
            || (string) $registration['verification_state'] !== 'mailbox_verified') {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        $status = (string) $registration['terminal_delivery_status'];
        if ($status === 'delivered' || $status === 'failed') {
            throw new DomainException('LICENSE_DELIVERY_FAILED');
        }
        if (!in_array($status, ['ready', 'pending'], true)) {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        return $registration;
    }

    /** Canonical EDD Software Licensing storage is the only key source. */
    private function resolveCanonicalKey(int $eddLicenseId, string $productCode): array
    {
        if ($eddLicenseId < 1 || preg_match(FocusaSpec152eTerminalDeliveryEnvelope::PRODUCT_CODE_PATTERN, $productCode) !== 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT license_key, status FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $eddLicenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || (string) ($row['status'] ?? '') !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $key = (string) $row['license_key'];
        if (preg_match(FocusaSpec152eTerminalDeliveryEnvelope::KEY_PATTERN, $key) !== 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return [
            'key' => $key,
            'mask' => FocusaSpec152eTerminalDeliveryEnvelope::maskKey($key),
            'digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($key),
        ];
    }

    private function journalEnvelope(
        array $registration,
        array $license,
        string $devicePublicKey,
        array $envelope,
        string $envelopeId,
        string $issuedAt,
        string $expiresAt,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
    ): void {
        $table = $this->schema->table('wpuiai_terminal_delivery_envelopes');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (envelope_id, registration_uuid, account_uuid, edd_customer_id, edd_license_id,
             product_code, license_key_digest, license_key_mask, device_public_key,
             envelope_payload, delivery_status, consumed_at, issued_at, expires_at,
             request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
            VALUES (:envelope_id, :registration, :account, :customer, :license_id,
                    :product, :key_digest, :key_mask, :device_key,
                    :payload, 'issued', NULL, :issued, :expires,
                    :request, :idempotency, :request_digest, :created, :retention, :updated)");
        $statement->execute([
            ':envelope_id' => $envelopeId,
            ':registration' => (string) $registration['registration_uuid'],
            ':account' => (string) ($registration['account_uuid'] ?? ''),
            ':customer' => (int) ($registration['edd_customer_id'] ?? 0),
            ':license_id' => (int) $registration['edd_license_id'],
            ':product' => (string) $registration['product_code'],
            ':key_digest' => $license['digest'],
            ':key_mask' => $license['mask'],
            ':device_key' => $devicePublicKey,
            ':payload' => FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($envelope),
            ':issued' => $issuedAt,
            ':expires' => $expiresAt,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':request_digest' => $digest,
            ':created' => $now,
            ':retention' => FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, $this->retention),
            ':updated' => $now,
        ]);
    }

    private function findEnvelopeByIdempotency(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_terminal_delivery_envelopes');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function pollResponseFromJournal(array $row, string $requestId): array
    {
        $registration = $this->registrations->findByUuid((string) $row['registration_uuid']);
        $envelope = json_decode((string) $row['envelope_payload'], true, 512, JSON_THROW_ON_ERROR);
        return $this->pollResponse(
            $registration,
            $requestId,
            (string) $row['envelope_id'],
            (string) $row['license_key_mask'],
            is_array($envelope) ? $envelope : [],
        );
    }

    private function pollResponse(array $registration, string $requestId, string $envelopeId, string $mask, array $envelope): array
    {
        return [
            'schema' => self::POLL_RESPONSE_SCHEMA,
            'request_id' => $requestId,
            'registration_id' => (string) $registration['registration_uuid'],
            'state' => (string) $registration['state'],
            'terminal' => false,
            'retry' => ['posture' => 'none'],
            'next_action' => 'decrypt_and_store_key',
            'terminal_delivery_status' => 'delivered',
            'node_id' => $registration['node_uuid'] === null ? null : (string) $registration['node_uuid'],
            'license_key_mask' => $mask,
            'envelope_id' => $envelopeId,
            'one_time_key_envelope' => FocusaSpec152eTerminalEnvelopeCrypto::base64UrlEncode(FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($envelope)),
        ];
    }

    private static function deviceKeyToRaw(string $devicePublicKeyB64): string
    {
        return FocusaSpec152eTerminalEnvelopeCrypto::base64UrlDecode($devicePublicKeyB64);
    }

    private static function devicePrivateFromInput(string $input): string
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $input) === 1) {
            return hex2bin($input);
        }
        if (strlen($input) === 32) {
            return $input;
        }
        throw new DomainException('ENVELOPE_DEVICE_KEY_DENIED');
    }

    private function transaction(callable $operation): mixed
    {
        if ($this->db->inTransaction()) {
            return $operation();
        }
        $this->db->beginTransaction();
        try {
            $result = $operation();
            $this->db->commit();
            return $result;
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        return $now;
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private function assertUuid(string $value, string $kind): void
    {
        if (preg_match(FocusaSpec152eTerminalDeliveryEnvelope::UUID_PATTERN, $value) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} UUID required");
        }
    }

    private function assertRequestId(string $requestId): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $requestId) !== 1) {
            throw new InvalidArgumentException('bounded request ID required');
        }
    }

    private function assertIdempotencyKey(string $key): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
    }
}

/**
 * Protected credential adapter (client seam). Stores the decrypted terminal key
 * through an injectable protected store (OS keyring/credential manager), never
 * emits the plaintext key into logs, transcripts, or returned JSON, and reveals
 * the key once only under explicit customer-controlled consent and within the
 * envelope lifetime.
 */
final class FocusaSpec152eTerminalCredentialAdapter
{
    public const SCHEMA = 'focusa.spec152e.terminal_credential_adapter.v1';

    /**
     * Store confirmation: the decrypted key was handed to the protected store;
     * only the opaque handle and the masked key are ever observable.
     */
    public static function storeConfirmation(string $handle, string $mask): array
    {
        return [
            'schema' => self::SCHEMA,
            'operation' => 'store',
            'handle' => $handle,
            'mask' => $mask,
            'store' => 'protected_credential_store',
            'revealed' => false,
        ];
    }

    /**
     * One-time reveal under explicit customer consent and within the envelope
     * lifetime. $consume atomically consumes the stored credential; a replay
     * fails closed. Without consent, or when expired, returns the fail-closed
     * denial — never the key.
     */
    public static function reveal(
        string $handle,
        bool $customerConsent,
        array $claims,
        string $now,
        callable $consume,
        ?callable $read = null,
    ): array {
        FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, $now, null);
        if ($customerConsent !== true) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        if ($read !== null) {
            $stored = $read($handle);
            if (!is_array($stored) || ($stored['consumed'] ?? false) === true) {
                throw new DomainException('CREDENTIAL_REVEAL_DENIED');
            }
        }
        if ($consume($handle) !== true) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        return [
            'schema' => self::SCHEMA,
            'operation' => 'reveal',
            'handle' => $handle,
            'revealed' => true,
            'license_key' => (string) $claims['license_key'],
            'mask' => FocusaSpec152eTerminalDeliveryEnvelope::maskKey((string) $claims['license_key']),
        ];
    }
}
