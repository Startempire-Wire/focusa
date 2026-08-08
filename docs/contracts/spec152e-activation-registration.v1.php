<?php
// Candidate-owned pending-registration schema/repository seam. It does not bootstrap WordPress.
declare(strict_types=1);

final class FocusaSpec152eActivationRegistrationState
{
    public const ATTEMPT_CREATED = 'attempt_created';
    public const EMAIL_CHALLENGE_SENT = 'email_challenge_sent';
    public const EMAIL_VERIFIED = 'email_verified';
    public const ACCOUNT_PROMOTED = 'account_promoted';
    public const OFFER_SELECTED = 'offer_selected';
    public const CHECKOUT_PENDING = 'checkout_pending';
    public const LIMITED_ACCESS_REVIEW = 'limited_access_review';
    public const EXISTING_KEY_REVIEW = 'existing_key_review';
    public const ENTITLEMENT_ISSUED = 'entitlement_issued';
    public const TERMINAL_DELIVERY_READY = 'terminal_delivery_ready';
    public const DEVICE_REGISTERED = 'device_registered';
    public const LEASE_ISSUED = 'lease_issued';
    public const DELIVERED = 'delivered';
    public const EXPIRED = 'expired';
    public const DENIED = 'denied';
    public const REFUNDED = 'refunded';
    public const REVOKED = 'revoked';
    public const SUPERSEDED = 'superseded';
    public const RECOVERY_ONLY = 'recovery_only';

    public const NONTERMINAL = [
        self::ATTEMPT_CREATED,
        self::EMAIL_CHALLENGE_SENT,
        self::EMAIL_VERIFIED,
        self::ACCOUNT_PROMOTED,
        self::OFFER_SELECTED,
        self::CHECKOUT_PENDING,
        self::LIMITED_ACCESS_REVIEW,
        self::EXISTING_KEY_REVIEW,
        self::ENTITLEMENT_ISSUED,
        self::TERMINAL_DELIVERY_READY,
        self::DEVICE_REGISTERED,
        self::LEASE_ISSUED,
    ];

    public const TERMINAL = [
        self::DELIVERED,
        self::EXPIRED,
        self::DENIED,
        self::REFUNDED,
        self::REVOKED,
        self::SUPERSEDED,
        self::RECOVERY_ONLY,
    ];

    private const TRANSITIONS = [
        self::ATTEMPT_CREATED => [self::EMAIL_CHALLENGE_SENT, self::EXPIRED, self::DENIED],
        self::EMAIL_CHALLENGE_SENT => [self::EMAIL_VERIFIED, self::EXPIRED, self::DENIED],
        self::EMAIL_VERIFIED => [self::ACCOUNT_PROMOTED, self::DENIED],
        self::ACCOUNT_PROMOTED => [self::OFFER_SELECTED, self::LIMITED_ACCESS_REVIEW, self::EXISTING_KEY_REVIEW, self::DENIED],
        self::OFFER_SELECTED => [self::CHECKOUT_PENDING, self::LIMITED_ACCESS_REVIEW, self::EXISTING_KEY_REVIEW, self::DENIED],
        self::CHECKOUT_PENDING => [self::ENTITLEMENT_ISSUED, self::DENIED, self::EXPIRED],
        self::LIMITED_ACCESS_REVIEW => [self::DEVICE_REGISTERED, self::DENIED],
        self::EXISTING_KEY_REVIEW => [self::ENTITLEMENT_ISSUED, self::DENIED],
        self::ENTITLEMENT_ISSUED => [self::TERMINAL_DELIVERY_READY, self::DEVICE_REGISTERED, self::REFUNDED, self::REVOKED],
        self::TERMINAL_DELIVERY_READY => [self::DEVICE_REGISTERED, self::REFUNDED, self::REVOKED],
        self::DEVICE_REGISTERED => [self::LEASE_ISSUED, self::DENIED, self::REFUNDED, self::REVOKED],
        self::LEASE_ISSUED => [self::DELIVERED, self::SUPERSEDED, self::REFUNDED, self::REVOKED, self::RECOVERY_ONLY],
        self::DELIVERED => [self::SUPERSEDED, self::REFUNDED, self::REVOKED, self::RECOVERY_ONLY],
        self::EXPIRED => [self::RECOVERY_ONLY],
        self::DENIED => [self::RECOVERY_ONLY],
        self::REFUNDED => [self::RECOVERY_ONLY],
        self::REVOKED => [self::RECOVERY_ONLY],
        self::SUPERSEDED => [self::RECOVERY_ONLY],
        self::RECOVERY_ONLY => [],
    ];

    public static function all(): array
    {
        return array_merge(self::NONTERMINAL, self::TERMINAL);
    }

    public static function canTransition(string $from, string $to): bool
    {
        return in_array($to, self::TRANSITIONS[$from] ?? [], true);
    }

    public static function isTerminal(string $state): bool
    {
        return in_array($state, self::TERMINAL, true);
    }

    public static function isPending(string $state): bool
    {
        return in_array($state, [
            self::ATTEMPT_CREATED,
            self::EMAIL_CHALLENGE_SENT,
            self::EMAIL_VERIFIED,
            self::ACCOUNT_PROMOTED,
            self::OFFER_SELECTED,
            self::CHECKOUT_PENDING,
            self::LIMITED_ACCESS_REVIEW,
            self::EXISTING_KEY_REVIEW,
            self::ENTITLEMENT_ISSUED,
            self::TERMINAL_DELIVERY_READY,
            self::DEVICE_REGISTERED,
            self::LEASE_ISSUED,
        ], true);
    }
}

final class FocusaSpec152eActivationRegistrationMigration
{
    public const SCHEMA = 'focusa.spec152e.activation_registration.v1';
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
        self::assertTimestamp($appliedAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $registration = $this->table('wpuiai_activation_registrations');
        $transitions = $this->table('wpuiai_activation_registration_transitions');
        $idempotency = $this->table('wpuiai_activation_registration_idempotency');
        $migrations = $this->table('wpuiai_activation_registration_schema_migrations');
        $events = $this->table('wpuiai_activation_registration_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $stateList = "'" . implode("','", FocusaSpec152eActivationRegistrationState::all()) . "'";

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$registration} (
            registration_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NULL,
            edd_customer_id BIGINT NULL,
            facade_id VARCHAR(96) NOT NULL,
            presenter VARCHAR(96) NOT NULL,
            install_channel VARCHAR(96) NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            safe_redirect_handle VARCHAR(128) NULL,
            state VARCHAR(32) NOT NULL CHECK (state IN ({$stateList})),
            state_reason VARCHAR(191) NOT NULL,
            state_version BIGINT NOT NULL DEFAULT 0 CHECK (state_version >= 0),
            encrypted_normalized_email TEXT NOT NULL,
            email_lookup_digest VARCHAR(64) NOT NULL,
            verification_state VARCHAR(32) NOT NULL CHECK (verification_state IN ('email_verification_pending', 'mailbox_verified', 'expired', 'failed')),
            verification_challenge_hash VARCHAR(64) NULL,
            verification_challenge_issued_at VARCHAR(32) NULL,
            verification_challenge_expires_at VARCHAR(32) NULL,
            verification_attempts BIGINT NOT NULL DEFAULT 0 CHECK (verification_attempts >= 0),
            verified_at VARCHAR(32) NULL,
            offer_code VARCHAR(128) NULL,
            journey VARCHAR(32) NULL,
            edd_cart_reference VARCHAR(191) NULL,
            edd_order_id BIGINT NULL,
            edd_order_item_id BIGINT NULL,
            edd_license_id BIGINT NULL,
            node_uuid {$uuid} NULL,
            device_public_key TEXT NULL,
            poll_credential_hash VARCHAR(64) NULL,
            poll_credential_issued_at VARCHAR(32) NULL,
            poll_credential_expires_at VARCHAR(32) NULL,
            terminal_delivery_status VARCHAR(16) NOT NULL DEFAULT 'none' CHECK (terminal_delivery_status IN ('none', 'pending', 'ready', 'delivered', 'failed')),
            delivery_attempts BIGINT NOT NULL DEFAULT 0 CHECK (delivery_attempts >= 0),
            delivery_ready_at VARCHAR(32) NULL,
            delivered_at VARCHAR(32) NULL,
            delivery_failure_reason VARCHAR(191) NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            settled_at VARCHAR(32) NULL,
            updated_at VARCHAR(32) NOT NULL,
            CHECK (state NOT IN ('attempt_created', 'email_challenge_sent', 'email_verified')
                OR (account_uuid IS NULL AND edd_customer_id IS NULL
                    AND edd_cart_reference IS NULL AND edd_order_id IS NULL AND edd_order_item_id IS NULL
                    AND edd_license_id IS NULL AND node_uuid IS NULL
                    AND terminal_delivery_status = 'none' AND delivery_attempts = 0
                    AND delivery_ready_at IS NULL AND delivered_at IS NULL)),
            CHECK (state NOT IN ('account_promoted', 'offer_selected', 'checkout_pending', 'limited_access_review', 'existing_key_review', 'entitlement_issued', 'terminal_delivery_ready', 'device_registered', 'lease_issued', 'delivered')
                OR (account_uuid IS NOT NULL AND edd_customer_id IS NOT NULL))
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$transitions} (
            transition_uuid {$uuid} NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            from_state VARCHAR(32) NOT NULL,
            to_state VARCHAR(32) NOT NULL,
            expected_version BIGINT NOT NULL,
            result_version BIGINT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            transition_digest VARCHAR(64) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            UNIQUE (registration_uuid, result_version)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            request_id {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            result_state VARCHAR(32) NOT NULL,
            result_version BIGINT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
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

        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_state_expiry
            ON {$registration} (state, expires_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_facade_state
            ON {$registration} (facade_id, state, created_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_email_lookup
            ON {$registration} (email_lookup_digest)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_poll_hash
            ON {$registration} (poll_credential_hash)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_request
            ON {$registration} (request_id)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_transition_retention
            ON {$transitions} (retention_until)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_activation_registration_idempotency_retention
            ON {$idempotency} (retention_until)");

        $statement = $this->db->prepare("INSERT INTO {$migrations}
            (schema_version, schema_name, applied_at, migration_provenance)
            SELECT :version, :schema, :applied, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)");
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied' => $appliedAt,
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: registrations, transitions, and journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_activation_registration_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => $encoded,
            ':existing_key' => $eventKey,
        ]);
        return ['schema' => self::SCHEMA, 'action' => 'preserve', 'event_key' => $eventKey];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    public static function assertTimestamp(string $timestamp): void
    {
        $parsed = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }

    public static function encodeCanonical(array $value): string
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
}

final class FocusaSpec152eActivationRegistrationSecrets
{
    public function __construct(
        private string $encryptionKey,
        private string $verificationKey,
        private string $pollKey,
    ) {
        if (strlen($this->encryptionKey) !== 32 || strlen($this->verificationKey) < 32 || strlen($this->pollKey) < 32) {
            throw new InvalidArgumentException('independent bounded registration keys required');
        }
        if ($this->sameKey($this->encryptionKey, $this->verificationKey)
            || $this->sameKey($this->encryptionKey, $this->pollKey)
            || $this->sameKey($this->verificationKey, $this->pollKey)) {
            throw new InvalidArgumentException('registration encryption and hash keys must be independent');
        }
    }

    public function emailLookupDigest(string $normalizedEmail): string
    {
        return hash_hmac('sha256', "focusa.spec152e.registration.email.lookup.v1\0" . $normalizedEmail, $this->verificationKey);
    }

    public function verificationHash(string $verifier): string
    {
        return hash_hmac('sha256', "focusa.spec152e.registration.verification.v1\0" . $verifier, $this->verificationKey);
    }

    public function pollHash(string $credential): string
    {
        return hash_hmac('sha256', "focusa.spec152e.registration.poll.v1\0" . $credential, $this->pollKey);
    }

    public function encryptEmail(string $normalizedEmail): string
    {
        if (function_exists('sodium_crypto_secretbox')) {
            $nonce = random_bytes(SODIUM_CRYPTO_SECRETBOX_NONCEBYTES);
            $ciphertext = sodium_crypto_secretbox($normalizedEmail, $nonce, $this->encryptionKey);
            $envelope = "s1\0" . $nonce . $ciphertext;
        } else {
            $nonce = random_bytes(12);
            $tag = '';
            $ciphertext = openssl_encrypt($normalizedEmail, 'aes-256-gcm', $this->encryptionKey, OPENSSL_RAW_DATA, $nonce, $tag);
            if ($ciphertext === false) {
                throw new RuntimeException('EMAIL_IDENTITY_ENCRYPTION_FAILED');
            }
            $envelope = "g1\0" . $nonce . $tag . $ciphertext;
        }
        return rtrim(strtr(base64_encode($envelope), '+/', '-_'), '=');
    }

    public function decryptEmail(string $envelope): string
    {
        $padding = (4 - strlen($envelope) % 4) % 4;
        $decoded = base64_decode(strtr($envelope . str_repeat('=', $padding), '-_', '+/'), true);
        if ($decoded === false || strlen($decoded) < 3 || substr($decoded, 2, 1) !== "\0") {
            throw new DomainException('EMAIL_IDENTITY_DECRYPTION_FAILED');
        }
        $version = substr($decoded, 0, 2);
        if ($version === 's1' && function_exists('sodium_crypto_secretbox_open')) {
            $nonceLength = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
            $plaintext = sodium_crypto_secretbox_open(
                substr($decoded, 3 + $nonceLength),
                substr($decoded, 3, $nonceLength),
                $this->encryptionKey
            );
        } elseif ($version === 'g1') {
            $nonce = substr($decoded, 3, 12);
            $tag = substr($decoded, 15, 16);
            $plaintext = openssl_decrypt(substr($decoded, 31), 'aes-256-gcm', $this->encryptionKey, OPENSSL_RAW_DATA, $nonce, $tag);
        } else {
            $plaintext = false;
        }
        if ($plaintext === false) {
            throw new DomainException('EMAIL_IDENTITY_DECRYPTION_FAILED');
        }
        return $plaintext;
    }

    private function sameKey(string $left, string $right): bool
    {
        return hash_equals(hash('sha256', $left, true), hash('sha256', substr($right, 0, strlen($left)), true));
    }
}

final class FocusaSpec152eActivationRegistrationPresenter
{
    public const SCHEMA = 'focusa.spec152e.registration.snapshot.v1';

    public static function snapshot(array $row): array
    {
        $state = (string) ($row['state'] ?? '');
        if (!in_array($state, FocusaSpec152eActivationRegistrationState::all(), true)) {
            throw new DomainException('UNKNOWN_REGISTRATION_STATE');
        }
        $terminal = FocusaSpec152eActivationRegistrationState::isTerminal($state);
        return [
            'schema' => self::SCHEMA,
            'registration_id' => (string) $row['registration_uuid'],
            'request_id' => (string) $row['request_id'],
            'facade_id' => (string) $row['facade_id'],
            'presenter' => (string) $row['presenter'],
            'install_channel' => (string) $row['install_channel'],
            'product_code' => (string) $row['product_code'],
            'state' => $state,
            'state_version' => (int) $row['state_version'],
            'terminal' => $terminal,
            'retry' => ['posture' => $terminal ? 'none' : 'safe_retry'],
            'next_action' => $terminal ? 'recover_or_manage_activation' : 'continue_activation',
            'terminal_delivery_status' => (string) $row['terminal_delivery_status'],
            'node_id' => $row['node_uuid'] === null ? null : (string) $row['node_uuid'],
        ];
    }
}

final class FocusaSpec152eActivationRegistrationRepository
{
    public const ATTEMPT_TTL_SECONDS = 1800;
    public const VERIFICATION_TTL_SECONDS = 900;
    public const POLL_TTL_SECONDS = 1800;
    public const RETENTION_SECONDS = 2592000;

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eActivationRegistrationMigration $schema,
        private FocusaSpec152eActivationRegistrationSecrets $secrets,
        callable $clock,
        private int $attemptTtl = self::ATTEMPT_TTL_SECONDS,
        private int $verificationTtl = self::VERIFICATION_TTL_SECONDS,
        private int $pollTtl = self::POLL_TTL_SECONDS,
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        foreach ([$this->attemptTtl, $this->verificationTtl, $this->pollTtl, $this->retention] as $ttl) {
            if ($ttl < 1) {
                throw new InvalidArgumentException('positive registration TTL required');
            }
        }
    }

    public function createPending(array $input): array
    {
        $this->requireFields($input, ['email', 'facade_id', 'presenter', 'install_channel', 'product_code', 'request_id', 'idempotency_key']);
        $this->rejectPendingAuthorityFields($input);
        $normalized = self::normalizeEmail((string) $input['email']);
        $this->assertToken((string) $input['facade_id'], 96);
        $this->assertToken((string) $input['presenter'], 96);
        $this->assertToken((string) $input['install_channel'], 96);
        $this->assertToken((string) $input['product_code'], 128);
        $this->assertRequestId((string) $input['request_id']);
        $this->assertIdempotencyKey((string) $input['idempotency_key']);
        $registrationUuid = (string) ($input['registration_uuid'] ?? self::uuid());
        $this->assertUuid($registrationUuid, 'registration');
        $now = $this->now();
        $expires = self::plusSeconds($now, $this->attemptTtl);
        $challengeExpires = min(self::plusSeconds($now, $this->verificationTtl), $expires);
        $pollExpires = min(self::plusSeconds($now, $this->pollTtl), $expires);
        $challenge = self::opaqueSecret();
        $pollCredential = self::opaqueSecret();
        $emailDigest = $this->secrets->emailLookupDigest($normalized);
        $digest = $this->requestDigest([
            'operation' => 'create_pending',
            'registration_uuid' => $input['registration_uuid'] ?? null,
            'email_lookup_digest' => $emailDigest,
            'facade_id' => $input['facade_id'],
            'presenter' => $input['presenter'],
            'install_channel' => $input['install_channel'],
            'product_code' => $input['product_code'],
            'safe_redirect_handle' => $input['safe_redirect_handle'] ?? null,
            'device_public_key' => $input['device_public_key'] ?? null,
            'request_id' => $input['request_id'],
        ]);

        return $this->transaction(function () use ($input, $normalized, $emailDigest, $registrationUuid, $now, $expires, $challengeExpires, $pollExpires, $challenge, $pollCredential, $digest): array {
            $replay = $this->replay('create_pending', (string) $input['idempotency_key'], $digest);
            if ($replay !== null) {
                return [
                    'registration' => $this->findByUuid((string) $replay['registration_uuid']),
                    'replayed' => true,
                ];
            }
            $table = $this->schema->table('wpuiai_activation_registrations');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (registration_uuid, account_uuid, edd_customer_id, facade_id, presenter, install_channel, product_code,
                 safe_redirect_handle, state, state_reason, state_version, encrypted_normalized_email, email_lookup_digest,
                 verification_state, verification_challenge_hash, verification_challenge_issued_at,
                 verification_challenge_expires_at, verification_attempts, verified_at, offer_code, journey,
                 edd_cart_reference, edd_order_id, edd_order_item_id, edd_license_id, node_uuid, device_public_key,
                 poll_credential_hash, poll_credential_issued_at, poll_credential_expires_at, terminal_delivery_status,
                 delivery_attempts, delivery_ready_at, delivered_at, delivery_failure_reason, request_id, idempotency_key,
                 request_digest, created_at, expires_at, settled_at, updated_at)
                VALUES (:registration, NULL, NULL, :facade, :presenter, :channel, :product, :redirect,
                 'attempt_created', 'attempt_created', 0, :encrypted_email, :email_digest,
                 'email_verification_pending', :verification_hash, :verification_issued, :verification_expires,
                 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, :device_key,
                 :poll_hash, :poll_issued, :poll_expires, 'none', 0, NULL, NULL, NULL,
                 :request_id, :idempotency_key, :request_digest, :created, :expires, NULL, :updated)");
            $statement->execute([
                ':registration' => $registrationUuid,
                ':facade' => $input['facade_id'],
                ':presenter' => $input['presenter'],
                ':channel' => $input['install_channel'],
                ':product' => $input['product_code'],
                ':redirect' => $input['safe_redirect_handle'] ?? null,
                ':encrypted_email' => $this->secrets->encryptEmail($normalized),
                ':email_digest' => $emailDigest,
                ':verification_hash' => $this->secrets->verificationHash($challenge),
                ':verification_issued' => $now,
                ':verification_expires' => $challengeExpires,
                ':device_key' => $input['device_public_key'] ?? null,
                ':poll_hash' => $this->secrets->pollHash($pollCredential),
                ':poll_issued' => $now,
                ':poll_expires' => $pollExpires,
                ':request_id' => $input['request_id'],
                ':idempotency_key' => $input['idempotency_key'],
                ':request_digest' => $digest,
                ':created' => $now,
                ':expires' => $expires,
                ':updated' => $now,
            ]);
            $row = $this->findByUuid($registrationUuid);
            $row = $this->transitionWithinTransaction($row, FocusaSpec152eActivationRegistrationState::ATTEMPT_CREATED,
                FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 0, (string) $input['request_id'],
                (string) $input['idempotency_key'], ['state_reason' => 'challenge_sent'], false);
            $this->recordIdempotency('create_pending', (string) $input['idempotency_key'], $digest, $registrationUuid, (string) $input['request_id'], $row, $now);
            return [
                'registration' => $row,
                'verification_secret' => $challenge,
                'poll_credential' => $pollCredential,
                'replayed' => false,
            ];
        });
    }

    public function verifyEmail(string $registrationUuid, string $verifier, string $requestId, string $idempotencyKey): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if ($verifier === '' || strlen($verifier) > 256 || preg_match('/[\r\n]/', $verifier)) {
            throw new InvalidArgumentException('bounded verification verifier required');
        }
        $digest = $this->requestDigest([
            'operation' => 'verify_email',
            'registration_uuid' => $registrationUuid,
            'verification_hash' => $this->secrets->verificationHash($verifier),
            'request_id' => $requestId,
        ]);
        try {
            return $this->transaction(function () use ($registrationUuid, $verifier, $requestId, $idempotencyKey, $digest): array {
            $replay = $this->replay('verify_email', $idempotencyKey, $digest);
            if ($replay !== null) {
                $row = $this->findByUuid($registrationUuid);
                $this->assertNotExpired($row, true);
                if ((string) $replay['result_state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT) {
                    throw new DomainException('EMAIL_VERIFICATION_FAILED');
                }
                return ['registration' => $row, 'replayed' => true];
            }
            $row = $this->findByUuid($registrationUuid);
            $this->assertNotExpired($row, false);
            if ($row['state'] !== FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT
                || $row['verification_state'] !== 'email_verification_pending'
                || $row['verification_challenge_hash'] === null) {
                throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
            }
            $now = $this->now();
            if ($row['verification_challenge_expires_at'] === null || $now >= $row['verification_challenge_expires_at']) {
                throw new DomainException('EMAIL_VERIFICATION_EXPIRED');
            }
            $expectedHash = $this->secrets->verificationHash($verifier);
            if (!hash_equals((string) $row['verification_challenge_hash'], $expectedHash)) {
                throw new DomainException('EMAIL_VERIFICATION_FAILED');
            }
            $updated = $this->transitionWithinTransaction($row, FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
                FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, (int) $row['state_version'], $requestId,
                $idempotencyKey, ['state_reason' => 'mailbox_verified'], true);
            $table = $this->schema->table('wpuiai_activation_registrations');
            $statement = $this->db->prepare("UPDATE {$table}
                SET verification_state = 'mailbox_verified', verification_challenge_hash = NULL,
                    verified_at = :verified, updated_at = :updated
                WHERE registration_uuid = :registration AND state = :state AND state_version = :version");
            $statement->execute([
                ':verified' => $now,
                ':updated' => $now,
                ':registration' => $registrationUuid,
                ':state' => FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
                ':version' => $updated['state_version'],
            ]);
            if ($statement->rowCount() !== 1) {
                throw new RuntimeException('verification state CAS failed');
            }
            $updated = $this->findByUuid($registrationUuid);
            $this->recordIdempotency('verify_email', $idempotencyKey, $digest, $registrationUuid, $requestId, $updated, $now);
            return ['registration' => $updated, 'replayed' => false];
            });
        } catch (DomainException $error) {
            if ($error->getMessage() === 'EMAIL_VERIFICATION_FAILED') {
                $this->incrementVerificationAttempts(
                    $this->findByUuid($registrationUuid),
                    $this->now(),
                    $idempotencyKey,
                    $digest,
                    $requestId,
                );
            }
            throw $error;
        }
    }

    /** Attach only already-resolved canonical references, after mailbox verification. */
    public function promoteVerified(string $registrationUuid, string $accountUuid, int $eddCustomerId, string $requestId, string $idempotencyKey): array
    {
        return $this->transaction(function () use ($registrationUuid, $accountUuid, $eddCustomerId, $requestId, $idempotencyKey): array {
            return $this->promoteVerifiedInTransaction($registrationUuid, $accountUuid, $eddCustomerId, $requestId, $idempotencyKey);
        });
    }

    /**
     * Caller-owned transaction primitive: advance a mailbox-verified registration to
     * account_promoted with already-resolved canonical references. Used by the atomic
     * verified-account promotion service; never starts its own transaction.
     */
    public function promoteVerifiedInTransaction(string $registrationUuid, string $accountUuid, int $eddCustomerId, string $requestId, string $idempotencyKey): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertUuid($accountUuid, 'account');
        if ($eddCustomerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->requestDigest([
            'operation' => 'promote_verified',
            'registration_uuid' => $registrationUuid,
            'account_uuid' => $accountUuid,
            'edd_customer_id' => $eddCustomerId,
            'request_id' => $requestId,
        ]);
        $replay = $this->replay('promote_verified', $idempotencyKey, $digest);
        if ($replay !== null) {
            return ['registration' => $this->findByUuid($registrationUuid), 'replayed' => true];
        }
        $row = $this->findByUuid($registrationUuid);
        $this->assertNotExpired($row, false);
        if ($row['state'] !== FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED
            || $row['verification_state'] !== 'mailbox_verified'
            || $row['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $updated = $this->transitionWithinTransaction($row, FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
            FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, (int) $row['state_version'], $requestId,
            $idempotencyKey, [
                'state_reason' => 'account_promoted',
                'account_uuid' => $accountUuid,
                'edd_customer_id' => $eddCustomerId,
            ], false);
        $this->recordIdempotency('promote_verified', $idempotencyKey, $digest, $registrationUuid, $requestId, $updated, $this->now());
        return ['registration' => $updated, 'replayed' => false];
    }

    /** Every state change requires the caller's observed state and version. */
    public function transition(string $registrationUuid, string $fromState, string $toState, int $expectedVersion, string $requestId, string $idempotencyKey, array $context = []): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if (!in_array($fromState, FocusaSpec152eActivationRegistrationState::all(), true)
            || !in_array($toState, FocusaSpec152eActivationRegistrationState::all(), true)) {
            throw new DomainException('UNKNOWN_REGISTRATION_STATE');
        }
        if ($expectedVersion < 0) {
            throw new InvalidArgumentException('non-negative expected state version required');
        }
        $digest = $this->requestDigest([
            'operation' => 'transition',
            'registration_uuid' => $registrationUuid,
            'from_state' => $fromState,
            'to_state' => $toState,
            'expected_version' => $expectedVersion,
            'context' => $this->safeContext($context),
            'request_id' => $requestId,
        ]);
        return $this->transaction(function () use ($registrationUuid, $fromState, $toState, $expectedVersion, $requestId, $idempotencyKey, $context, $digest): array {
            $replay = $this->replay('transition', $idempotencyKey, $digest);
            if ($replay !== null) {
                return ['registration' => $this->findByUuid($registrationUuid), 'replayed' => true];
            }
            $row = $this->findByUuid($registrationUuid);
            $updated = $this->transitionWithinTransaction($row, $fromState, $toState, $expectedVersion, $requestId, $idempotencyKey, $context, false);
            $this->recordIdempotency('transition', $idempotencyKey, $digest, $registrationUuid, $requestId, $updated, $this->now());
            return ['registration' => $updated, 'replayed' => false];
        });
    }

    /** Rotate a poll credential without ever persisting the plaintext credential. */
    public function issuePollCredential(string $registrationUuid, string $requestId, string $idempotencyKey): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->requestDigest([
            'operation' => 'issue_poll_credential',
            'registration_uuid' => $registrationUuid,
            'request_id' => $requestId,
        ]);
        return $this->transaction(function () use ($registrationUuid, $requestId, $idempotencyKey, $digest): array {
            $replay = $this->replay('issue_poll_credential', $idempotencyKey, $digest);
            if ($replay !== null) {
                return ['registration' => $this->findByUuid($registrationUuid), 'replayed' => true];
            }
            $row = $this->findByUuid($registrationUuid);
            $this->assertNotExpired($row, false);
            if (FocusaSpec152eActivationRegistrationState::isTerminal((string) $row['state'])) {
                throw new DomainException('POLL_CREDENTIAL_EXPIRED');
            }
            $now = $this->now();
            $expires = min(self::plusSeconds($now, $this->pollTtl), (string) $row['expires_at']);
            $credential = self::opaqueSecret();
            $table = $this->schema->table('wpuiai_activation_registrations');
            $statement = $this->db->prepare("UPDATE {$table}
                SET poll_credential_hash = :hash, poll_credential_issued_at = :issued,
                    poll_credential_expires_at = :expires, state_version = state_version + 1, updated_at = :updated
                WHERE registration_uuid = :registration AND state = :state AND state_version = :version");
            $statement->execute([
                ':hash' => $this->secrets->pollHash($credential),
                ':issued' => $now,
                ':expires' => $expires,
                ':updated' => $now,
                ':registration' => $registrationUuid,
                ':state' => $row['state'],
                ':version' => $row['state_version'],
            ]);
            if ($statement->rowCount() !== 1) {
                throw new DomainException('REGISTRATION_STATE_CONFLICT');
            }
            $updated = $this->findByUuid($registrationUuid);
            $this->recordIdempotency('issue_poll_credential', $idempotencyKey, $digest, $registrationUuid, $requestId, $updated, $now);
            return ['registration' => $updated, 'poll_credential' => $credential, 'replayed' => false];
        });
    }

    /** Poll returns only a redacted state snapshot; hashes and encrypted identity stay internal. */
    public function poll(string $registrationUuid, string $pollCredential, string $requestId, string $idempotencyKey, ?string $devicePublicKey = null): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if ($pollCredential === '' || strlen($pollCredential) > 512 || preg_match('/[\r\n]/', $pollCredential)) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        $credentialHash = $this->secrets->pollHash($pollCredential);
        $digest = $this->requestDigest([
            'operation' => 'poll',
            'registration_uuid' => $registrationUuid,
            'poll_hash' => $credentialHash,
            'device_public_key' => $devicePublicKey,
            'request_id' => $requestId,
        ]);
        return $this->transaction(function () use ($registrationUuid, $credentialHash, $requestId, $idempotencyKey, $devicePublicKey, $digest): array {
            $row = $this->findByUuid($registrationUuid);
            $this->assertNotExpired($row, false);
            $replay = $this->replay('poll', $idempotencyKey, $digest);
            if ($replay !== null) {
                return ['snapshot' => FocusaSpec152eActivationRegistrationPresenter::snapshot($row), 'replayed' => true];
            }
            if ($row['poll_credential_hash'] === null || $row['poll_credential_expires_at'] === null) {
                throw new DomainException('POLL_CREDENTIAL_REQUIRED');
            }
            $now = $this->now();
            if ($now >= $row['poll_credential_expires_at']) {
                throw new DomainException('POLL_CREDENTIAL_EXPIRED');
            }
            if (!hash_equals((string) $row['poll_credential_hash'], $credentialHash)) {
                throw new DomainException('POLL_CREDENTIAL_REQUIRED');
            }
            if ($devicePublicKey !== null && $row['device_public_key'] !== null
                && !hash_equals((string) $row['device_public_key'], $devicePublicKey)) {
                throw new DomainException('NODE_KEY_MISMATCH');
            }
            $this->recordIdempotency('poll', $idempotencyKey, $digest, $registrationUuid, $requestId, $row, $now);
            return ['snapshot' => FocusaSpec152eActivationRegistrationPresenter::snapshot($row), 'replayed' => false];
        });
    }

    /** Expire due pending attempts, then remove only unpromoted records past the retention horizon. */
    public function cleanup(string $now, ?int $retentionSeconds = null): array
    {
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($now);
        $retention = $retentionSeconds ?? $this->retention;
        if ($retention < 1) {
            throw new InvalidArgumentException('positive retention is required');
        }
        $expired = $this->expireDue($now);
        $cutoff = self::minusSeconds($now, $retention);
        $table = $this->schema->table('wpuiai_activation_registrations');
        $transitionTable = $this->schema->table('wpuiai_activation_registration_transitions');
        $idempotencyTable = $this->schema->table('wpuiai_activation_registration_idempotency');
        $statement = $this->db->prepare("SELECT registration_uuid FROM {$table}
            WHERE settled_at IS NOT NULL AND settled_at <= :cutoff
              AND account_uuid IS NULL AND edd_customer_id IS NULL
              AND edd_order_id IS NULL AND edd_order_item_id IS NULL AND edd_license_id IS NULL
              AND node_uuid IS NULL
              AND state IN ('expired', 'denied', 'recovery_only')");
        $statement->execute([':cutoff' => $cutoff]);
        $registrations = array_map('strval', $statement->fetchAll(PDO::FETCH_COLUMN));
        $deleted = 0;
        if ($registrations !== []) {
            $this->transaction(function () use ($registrations, $table, $transitionTable, $idempotencyTable): void {
                $placeholders = implode(',', array_fill(0, count($registrations), '?'));
                $deleteTransitions = $this->db->prepare("DELETE FROM {$transitionTable} WHERE registration_uuid IN ({$placeholders})");
                $deleteTransitions->execute($registrations);
                $deleteIdempotency = $this->db->prepare("DELETE FROM {$idempotencyTable} WHERE registration_uuid IN ({$placeholders})");
                $deleteIdempotency->execute($registrations);
                $deleteRegistrations = $this->db->prepare("DELETE FROM {$table} WHERE registration_uuid IN ({$placeholders})");
                $deleteRegistrations->execute($registrations);
            });
            $deleted = count($registrations);
        }
        $this->db->prepare("DELETE FROM {$transitionTable} WHERE retention_until <= :now")->execute([':now' => $now]);
        $this->db->prepare("DELETE FROM {$idempotencyTable} WHERE retention_until <= :now")->execute([':now' => $now]);
        return ['expired' => $expired, 'deleted' => $deleted, 'retention_cutoff' => $cutoff];
    }

    public function findByUuid(string $registrationUuid): array
    {
        $this->assertUuid($registrationUuid, 'registration');
        $table = $this->schema->table('wpuiai_activation_registrations');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE registration_uuid = :registration");
        $statement->execute([':registration' => $registrationUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('activation registration not found');
        }
        return $row;
    }

    public function findByEmailDigest(string $emailDigest): ?array
    {
        if (!preg_match('/^[a-f0-9]{64}$/D', $emailDigest)) {
            throw new InvalidArgumentException('email lookup digest required');
        }
        $table = $this->schema->table('wpuiai_activation_registrations');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE email_lookup_digest = :digest ORDER BY created_at DESC LIMIT 1");
        $statement->execute([':digest' => $emailDigest]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function expireDue(string $now): int
    {
        $table = $this->schema->table('wpuiai_activation_registrations');
        $statement = $this->db->prepare("SELECT registration_uuid, state, state_version FROM {$table}
            WHERE expires_at <= :now AND state IN ('attempt_created', 'email_challenge_sent', 'checkout_pending')");
        $statement->execute([':now' => $now]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        $count = 0;
        foreach ($rows as $row) {
            $id = (string) $row['registration_uuid'];
            $key = 'expiry:' . substr(hash('sha256', $id . ':' . $row['state_version']), 0, 48);
            $request = 'expiry-' . substr(hash('sha256', $id . ':' . $row['state_version']), 0, 32);
            try {
                $this->transition($id, (string) $row['state'], FocusaSpec152eActivationRegistrationState::EXPIRED,
                    (int) $row['state_version'], $request, $key, ['state_reason' => 'ttl_expired']);
                $count++;
            } catch (DomainException $error) {
                if (!in_array($error->getMessage(), ['REGISTRATION_STATE_CONFLICT', 'REGISTRATION_EXPIRED'], true)) {
                    throw $error;
                }
            }
        }
        return $count;
    }

    private function transitionWithinTransaction(array $row, string $fromState, string $toState, int $expectedVersion, string $requestId, string $idempotencyKey, array $context, bool $verified): array
    {
        if ((string) $row['state'] !== $fromState || (int) $row['state_version'] !== $expectedVersion) {
            throw new DomainException('REGISTRATION_STATE_CONFLICT');
        }
        if (!FocusaSpec152eActivationRegistrationState::canTransition($fromState, $toState)) {
            throw new DomainException('INVALID_REGISTRATION_TRANSITION');
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED && !$verified) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $now = $this->now();
        $allowTerminalExpiry = $toState === FocusaSpec152eActivationRegistrationState::EXPIRED
            || $toState === FocusaSpec152eActivationRegistrationState::RECOVERY_ONLY;
        if ($toState === FocusaSpec152eActivationRegistrationState::EXPIRED
            && !$this->isExpiredPending($row, $now)) {
            throw new DomainException('REGISTRATION_NOT_DUE');
        }
        $this->assertNotExpired($row, $allowTerminalExpiry);
        $context = $this->safeContext($context);
        if (in_array($fromState, [
            FocusaSpec152eActivationRegistrationState::ATTEMPT_CREATED,
            FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
            FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
        ], true) && $toState !== FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED) {
            $authorityFields = ['account_uuid', 'edd_customer_id', 'edd_cart_reference', 'edd_order_id', 'edd_order_item_id', 'edd_license_id', 'node_uuid', 'device_public_key'];
            if (array_intersect($authorityFields, array_keys($context)) !== []) {
                throw new DomainException('PENDING_AUTHORITY_FIELD_DENIED');
            }
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED) {
            $promotionFields = ['state_reason', 'account_uuid', 'edd_customer_id'];
            if (array_diff(array_keys($context), $promotionFields) !== []) {
                throw new DomainException('PENDING_AUTHORITY_FIELD_DENIED');
            }
            if ($row['verification_state'] !== 'mailbox_verified' || $row['verified_at'] === null
                || !isset($context['account_uuid'], $context['edd_customer_id'])) {
                throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
            }
            $this->assertUuid((string) $context['account_uuid'], 'account');
            if ((int) $context['edd_customer_id'] < 1) {
                throw new InvalidArgumentException('positive EDD customer ID required');
            }
        }
        if ($row['account_uuid'] !== null && array_key_exists('account_uuid', $context)
            && !hash_equals((string) $row['account_uuid'], (string) $context['account_uuid'])) {
            throw new DomainException('ACCOUNT_BINDING_CONFLICT');
        }
        if ($row['edd_customer_id'] !== null && array_key_exists('edd_customer_id', $context)
            && (int) $row['edd_customer_id'] !== (int) $context['edd_customer_id']) {
            throw new DomainException('ACCOUNT_BINDING_CONFLICT');
        }
        if (in_array($toState, [
            FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
            FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
            FocusaSpec152eActivationRegistrationState::LIMITED_ACCESS_REVIEW,
            FocusaSpec152eActivationRegistrationState::EXISTING_KEY_REVIEW,
            FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
            FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY,
            FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED,
            FocusaSpec152eActivationRegistrationState::LEASE_ISSUED,
            FocusaSpec152eActivationRegistrationState::DELIVERED,
        ], true) && ($row['account_uuid'] === null && !isset($context['account_uuid']))) {
            throw new DomainException('ACCOUNT_PROMOTION_REQUIRED');
        }
        $updates = [
            'state' => $toState,
            'state_reason' => (string) ($context['state_reason'] ?? $toState),
            'updated_at' => $now,
        ];
        foreach (['account_uuid', 'edd_customer_id', 'offer_code', 'journey', 'edd_cart_reference', 'edd_order_id', 'edd_order_item_id', 'edd_license_id', 'node_uuid', 'device_public_key', 'terminal_delivery_status', 'delivery_attempts', 'delivery_ready_at', 'delivered_at', 'delivery_failure_reason'] as $field) {
            if (array_key_exists($field, $context)) {
                $updates[$field] = $context[$field];
            }
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED) {
            foreach (['edd_order_id', 'edd_order_item_id', 'edd_license_id'] as $field) {
                if (!isset($updates[$field]) && $row[$field] === null) {
                    throw new DomainException('EDD_LICENSE_PENDING');
                }
            }
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED) {
            if (($updates['node_uuid'] ?? $row['node_uuid']) === null || ($updates['device_public_key'] ?? $row['device_public_key']) === null) {
                throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
            }
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY) {
            $updates['terminal_delivery_status'] = $updates['terminal_delivery_status'] ?? 'ready';
            $updates['delivery_ready_at'] = $updates['delivery_ready_at'] ?? $now;
        }
        if ($toState === FocusaSpec152eActivationRegistrationState::DELIVERED) {
            $updates['terminal_delivery_status'] = 'delivered';
            $updates['delivered_at'] = $updates['delivered_at'] ?? $now;
        }
        if ($verified) {
            $updates['verification_state'] = 'mailbox_verified';
            $updates['verified_at'] = $now;
            $updates['verification_challenge_hash'] = null;
        }
        if (FocusaSpec152eActivationRegistrationState::isTerminal($toState)) {
            $updates['settled_at'] = $row['settled_at'] ?? $now;
        }
        $set = [];
        $params = [':registration' => $row['registration_uuid'], ':from_state' => $fromState, ':expected_version' => $expectedVersion];
        foreach ($updates as $field => $value) {
            $set[] = $field . ' = :' . $field;
            $params[':' . $field] = $value;
        }
        $set[] = 'state_version = state_version + 1';
        $table = $this->schema->table('wpuiai_activation_registrations');
        $sql = "UPDATE {$table} SET " . implode(', ', $set) . "
            WHERE registration_uuid = :registration AND state = :from_state AND state_version = :expected_version
              AND (state NOT IN ('attempt_created', 'email_challenge_sent', 'email_verified', 'account_promoted', 'offer_selected', 'checkout_pending', 'limited_access_review', 'existing_key_review')
                   OR expires_at > :guard_now OR :to_state = 'expired')";
        $params[':guard_now'] = $now;
        $params[':to_state'] = $toState;
        $statement = $this->db->prepare($sql);
        $statement->execute($params);
        if ($statement->rowCount() !== 1) {
            $latest = $this->findByUuid((string) $row['registration_uuid']);
            if ($this->isExpiredPending($latest, $now) && $toState !== FocusaSpec152eActivationRegistrationState::EXPIRED) {
                throw new DomainException('REGISTRATION_EXPIRED');
            }
            throw new DomainException('REGISTRATION_STATE_CONFLICT');
        }
        $resultVersion = $expectedVersion + 1;
        $transitionTable = $this->schema->table('wpuiai_activation_registration_transitions');
        $transitionUuid = self::uuid();
        $transitionDigest = $this->requestDigest([
            'registration_uuid' => $row['registration_uuid'],
            'from_state' => $fromState,
            'to_state' => $toState,
            'expected_version' => $expectedVersion,
            'result_version' => $resultVersion,
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
        ]);
        $retentionUntil = self::plusSeconds($now, $this->retention);
        $statement = $this->db->prepare("INSERT INTO {$transitionTable}
            (transition_uuid, registration_uuid, from_state, to_state, expected_version, result_version,
             request_id, idempotency_key, transition_digest, occurred_at, retention_until)
            VALUES (:transition, :registration, :from_state, :to_state, :expected, :result,
                    :request_id, :idempotency, :digest, :occurred, :retention)");
        $statement->execute([
            ':transition' => $transitionUuid,
            ':registration' => $row['registration_uuid'],
            ':from_state' => $fromState,
            ':to_state' => $toState,
            ':expected' => $expectedVersion,
            ':result' => $resultVersion,
            ':request_id' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':digest' => $transitionDigest,
            ':occurred' => $now,
            ':retention' => $retentionUntil,
        ]);
        return $this->findByUuid((string) $row['registration_uuid']);
    }

    private function incrementVerificationAttempts(array $row, string $now, string $idempotencyKey, string $digest, string $requestId): void
    {
        $this->transaction(function () use ($row, $now, $idempotencyKey, $digest, $requestId): void {
            if ($this->replay('verify_email', $idempotencyKey, $digest) !== null) {
                return;
            }
            $table = $this->schema->table('wpuiai_activation_registrations');
            $statement = $this->db->prepare("UPDATE {$table}
                SET verification_attempts = verification_attempts + 1, state_version = state_version + 1, updated_at = :updated
                WHERE registration_uuid = :registration AND state = :state AND state_version = :version
                  AND expires_at > :now");
            $statement->execute([
                ':updated' => $now,
                ':registration' => $row['registration_uuid'],
                ':state' => $row['state'],
                ':version' => $row['state_version'],
                ':now' => $now,
            ]);
            if ($statement->rowCount() !== 1) {
                throw new DomainException('REGISTRATION_STATE_CONFLICT');
            }
            $updated = $this->findByUuid((string) $row['registration_uuid']);
            $this->recordIdempotency('verify_email', $idempotencyKey, $digest, (string) $row['registration_uuid'], $requestId, $updated, $now);
        });
    }

    private function assertNotExpired(array $row, bool $allowTerminalExpiry): void
    {
        $now = $this->now();
        if (($allowTerminalExpiry === false && (string) $row['state'] === FocusaSpec152eActivationRegistrationState::EXPIRED)
            || ($this->isExpiredPending($row, $now) && !$allowTerminalExpiry)) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if ($row['expires_at'] !== null && $now >= (string) $row['expires_at']
            && !$allowTerminalExpiry && FocusaSpec152eActivationRegistrationState::isPending((string) $row['state'])) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if ($row['expires_at'] !== null && $now >= (string) $row['expires_at']
            && (string) $row['state'] === FocusaSpec152eActivationRegistrationState::EXPIRED) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
    }

    private function isExpiredPending(array $row, string $now): bool
    {
        return FocusaSpec152eActivationRegistrationState::isPending((string) $row['state'])
            && $row['expires_at'] !== null && $now >= (string) $row['expires_at'];
    }

    private function replay(string $operation, string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_activation_registration_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals((string) $row['operation'], $operation) || !hash_equals((string) $row['request_digest'], $digest)) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $operation, string $idempotencyKey, string $digest, string $registrationUuid, string $requestId, array $row, string $now): void
    {
        $table = $this->schema->table('wpuiai_activation_registration_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, registration_uuid, request_id, request_digest, result_state, result_version, created_at, retention_until)
            VALUES (:key, :operation, :registration, :request_id, :digest, :state, :version, :created, :retention)");
        $statement->execute([
            ':key' => $idempotencyKey,
            ':operation' => $operation,
            ':registration' => $registrationUuid,
            ':request_id' => $requestId,
            ':digest' => $digest,
            ':state' => $row['state'],
            ':version' => $row['state_version'],
            ':created' => $now,
            ':retention' => self::plusSeconds($now, $this->retention),
        ]);
    }

    private function safeContext(array $context): array
    {
        $allowed = [
            'state_reason', 'account_uuid', 'edd_customer_id', 'offer_code', 'journey', 'edd_cart_reference',
            'edd_order_id', 'edd_order_item_id', 'edd_license_id', 'node_uuid', 'device_public_key',
            'terminal_delivery_status', 'delivery_attempts', 'delivery_ready_at', 'delivered_at', 'delivery_failure_reason',
        ];
        if (array_diff(array_keys($context), $allowed) !== []) {
            throw new DomainException('REGISTRATION_CONTEXT_FIELD_DENIED');
        }
        foreach ($context as $key => $value) {
            if (is_string($value)) {
                $this->assertToken($value, 191);
            } elseif (!is_int($value) && $value !== null) {
                throw new InvalidArgumentException('bounded registration context required');
            }
        }
        if (isset($context['state_reason']) && !preg_match('/^[A-Za-z0-9_.:-]{1,191}$/D', (string) $context['state_reason'])) {
            throw new InvalidArgumentException('safe registration reason required');
        }
        return $context;
    }

    private function requestDigest(array $payload): string
    {
        return hash('sha256', FocusaSpec152eActivationRegistrationMigration::encodeCanonical($payload));
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($now);
        return $now;
    }

    private function transaction(callable $callback): mixed
    {
        $this->db->beginTransaction();
        try {
            $result = $callback();
            $this->db->commit();
            return $result;
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    private function requireFields(array $input, array $fields): void
    {
        foreach ($fields as $field) {
            if (!array_key_exists($field, $input) || !is_string($input[$field]) || $input[$field] === '') {
                throw new InvalidArgumentException($field . ' is required');
            }
        }
    }

    private function rejectPendingAuthorityFields(array $input): void
    {
        $forbidden = [
            'account_uuid', 'edd_customer_id', 'edd_order_id', 'edd_order_item_id', 'edd_license_id',
            'entitlement', 'entitlement_sequence', 'lease', 'license_key', 'price', 'grants', 'features', 'limits',
        ];
        if (array_intersect($forbidden, array_keys($input)) !== []) {
            throw new DomainException('PENDING_AUTHORITY_FIELD_DENIED');
        }
    }

    private function assertToken(string $value, int $maxLength): void
    {
        if ($value === '' || strlen($value) > $maxLength || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException('bounded registration token required');
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

    private function assertUuid(string $value, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $value) !== 1) {
            throw new InvalidArgumentException('canonical opaque ' . $kind . ' UUID required');
        }
    }

    private static function normalizeEmail(string $submitted): string
    {
        $email = trim($submitted, " \t\r\n\0\x0B");
        if ($email === '' || strlen($email) > 254 || preg_match('/[\x00-\x1F\x7F]/', $email) === 1) {
            throw new InvalidArgumentException('valid email identity required');
        }
        $at = strrpos($email, '@');
        if ($at === false || $at < 1 || $at === strlen($email) - 1) {
            throw new InvalidArgumentException('valid email identity required');
        }
        $local = substr($email, 0, $at);
        $domain = strtolower(substr($email, $at + 1));
        $normalized = $local . '@' . $domain;
        if (filter_var($normalized, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid email identity required');
        }
        return $normalized;
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private static function minusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('-' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private static function opaqueSecret(): string
    {
        return rtrim(strtr(base64_encode(random_bytes(32)), '+/', '-_'), '=');
    }

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }
}
