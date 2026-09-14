<?php
// Verified-registration EDD gate token. Issued only from a mailbox-verified, non-terminal
// activation registration; short-lived; single-use; stored only as a keyed digest. The raw
// token is returned exactly once and never persisted, logged, or echoed. Every token is
// bound to its registration, facade, and product, so a token minted for one facade or
// product can never open a protected EDD cart on another. Replays are idempotent and never
// re-issue a raw token. Rollback is preservation-only: tokens and journals are never
// deleted.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php (registration repository
// and state machine) to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddRegistrationTokenMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_registration_token.v1';
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
        $tokens = $this->table('wpuiai_edd_registration_tokens');
        $idempotency = $this->table('wpuiai_edd_registration_token_idempotency');
        $migrations = $this->table('wpuiai_edd_registration_token_schema_migrations');
        $events = $this->table('wpuiai_edd_registration_token_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$tokens} (
            token_hash VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            edd_customer_id BIGINT NULL,
            facade_id VARCHAR(96) NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            state VARCHAR(16) NOT NULL CHECK (state IN ('active', 'consumed', 'expired', 'revoked')),
            issued_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            consumed_at VARCHAR(32) NULL,
            revoked_at VARCHAR(32) NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_registration_token_registration
            ON {$tokens} (registration_uuid, state, expires_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_registration_token_expiry
            ON {$tokens} (expires_at, state)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            result_payload TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_registration_token_idem_retention
            ON {$idempotency} (retention_until)");
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
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: tokens and journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_registration_token_schema_events');
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

    public static function assertTimestamp(?string $timestamp, bool $nullable = false): void
    {
        if ($nullable && $timestamp === null) {
            return;
        }
        $parsed = is_string($timestamp)
            ? DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'))
            : false;
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

final class FocusaSpec152eVerifiedRegistrationTokenValidator
{
    public const SCHEMA = 'focusa.spec152e.verified_registration_token_validator.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.verified_registration_token.v1';
    public const VERSION = 1;
    public const TOKEN_TTL_SECONDS = 1800;
    public const RETENTION_SECONDS = 2592000;
    public const MAX_ACTIVE_TOKENS_PER_REGISTRATION = 16;

    /**
     * Registration states that are mailbox-verified and still before entitlement issuance;
     * these are the only states allowed to mint or accept an EDD gate token.
     */
    public const VERIFIED_NONTERMINAL_STATES = [
        FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
        FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
        FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
        FocusaSpec152eActivationRegistrationState::LIMITED_ACCESS_REVIEW,
        FocusaSpec152eActivationRegistrationState::EXISTING_KEY_REVIEW,
    ];

    private const TOKEN_PREFIX = 'rg_';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddRegistrationTokenMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        callable $clock,
        private int $tokenTtl = self::TOKEN_TTL_SECONDS,
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if ($this->tokenTtl < 1 || $this->retention < 1) {
            throw new InvalidArgumentException('positive token TTL and retention required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Issue a single-use verified-registration gate token.
     *
     * Required input:
     *   - registration_uuid:  mailbox-verified, non-terminal registration UUID
     *   - facade_id:          exact facade the registration is bound to
     *   - product_code:       exact public product code the registration is bound to
     *   - request_id / idempotency_key
     *
     * The raw token is returned exactly once; only its keyed digest is stored. Replay with
     * the same idempotency key returns a bounded envelope without re-issuing a raw token.
     */
    public function issue(array $input): array
    {
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $productCode = (string) ($input['product_code'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertToken($facadeId, 96);
        $this->assertToken($productCode, 128);
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $now = $this->now();
        $digest = $this->requestDigest([
            'operation' => 'issue_registration_token',
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'request_id' => $requestId,
        ]);

        $replay = $this->replay('issue_registration_token', $idempotencyKey, $digest);
        if ($replay !== null) {
            return [
                'schema' => self::RESULT_SCHEMA,
                'replayed' => true,
                'token_state' => (string) $replay['token_state'],
                'expires_at' => (string) $replay['expires_at'],
                'registration_uuid' => $registrationUuid,
                'facade_id' => $facadeId,
                'product_code' => $productCode,
            ];
        }

        $registration = $this->registrations->findByUuid($registrationUuid);
        $this->assertIssuableRegistration($registration, $facadeId, $productCode, $now);

        $table = $this->schema->table('wpuiai_edd_registration_tokens');
        $active = (int) $this->db->query(
            "SELECT COUNT(*) FROM {$table} WHERE registration_uuid = " . $this->db->quote($registrationUuid) . " AND state = 'active'"
        )->fetchColumn();
        if ($active >= self::MAX_ACTIVE_TOKENS_PER_REGISTRATION) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }

        $rawToken = self::opaqueToken();
        $expires = min(self::plusSeconds($now, $this->tokenTtl), (string) $registration['expires_at']);
        $statement = $this->db->prepare("INSERT INTO {$table}
            (token_hash, registration_uuid, account_uuid, edd_customer_id, facade_id, product_code,
             state, issued_at, expires_at, consumed_at, revoked_at, request_id, idempotency_key,
             created_at, retention_until)
            VALUES (:hash, :registration, :account, :customer, :facade, :product,
                    'active', :issued, :expires, NULL, NULL, :request, :idempotency,
                    :created, :retention)");
        $statement->execute([
            ':hash' => self::tokenHash($rawToken),
            ':registration' => $registrationUuid,
            ':account' => $registration['account_uuid'],
            ':customer' => $registration['edd_customer_id'] !== null ? (int) $registration['edd_customer_id'] : null,
            ':facade' => $facadeId,
            ':product' => $productCode,
            ':issued' => $now,
            ':expires' => $expires,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':created' => $now,
            ':retention' => self::plusSeconds($now, $this->retention),
        ]);
        $this->recordIdempotency($idempotencyKey, 'issue_registration_token', $digest, [
            'token_state' => 'active',
            'expires_at' => $expires,
        ], $now);

        return [
            'schema' => self::RESULT_SCHEMA,
            'registration_token' => $rawToken,
            'expires_at' => $expires,
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'replayed' => false,
        ];
    }

    /**
     * Validate a single-use gate token against its registration, facade, and product
     * binding. Re-checks that the registration is still mailbox-verified and non-terminal.
     * Optionally consumes the token (single-use). Returns a bounded decision envelope with
     * no raw token, no email, and no secret.
     *
     * Required input:
     *   - registration_token: the raw gate token
     *   - registration_uuid / facade_id / product_code: expected binding
     *   - request_id / idempotency_key
     *   - consume: bool (default true)
     */
    public function validate(array $input): array
    {
        $rawToken = (string) ($input['registration_token'] ?? '');
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $productCode = (string) ($input['product_code'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $consume = (bool) ($input['consume'] ?? true);
        if ($rawToken === '' || strlen($rawToken) > 128 || preg_match('/[\r\n\x00]/', $rawToken)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertToken($facadeId, 96);
        $this->assertToken($productCode, 128);
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $now = $this->now();
        $tokenHash = self::tokenHash($rawToken);
        $digest = $this->requestDigest([
            'operation' => 'validate_registration_token',
            'registration_token_hash' => $tokenHash,
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'consume' => $consume,
            'request_id' => $requestId,
        ]);

        $replay = $this->replay('validate_registration_token', $idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $table = $this->schema->table('wpuiai_edd_registration_tokens');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE token_hash = :hash LIMIT 1");
        $statement->execute([':hash' => $tokenHash]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || in_array((string) $row['state'], ['consumed', 'revoked'], true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ((string) $row['state'] === 'expired' || $now >= (string) $row['expires_at']) {
            throw new DomainException('EMAIL_VERIFICATION_EXPIRED');
        }
        if (!hash_equals((string) $row['registration_uuid'], $registrationUuid)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!hash_equals((string) $row['facade_id'], $facadeId)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if (!hash_equals((string) $row['product_code'], $productCode)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }

        $registration = $this->registrations->findByUuid($registrationUuid);
        if (!in_array((string) $registration['state'], self::VERIFIED_NONTERMINAL_STATES, true)
            || (string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }

        $result = [
            'schema' => self::RESULT_SCHEMA,
            'ok' => true,
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'expires_at' => (string) $row['expires_at'],
            'token_state' => 'active',
        ];
        if ($consume) {
            $update = $this->db->prepare("UPDATE {$table} SET state = 'consumed', consumed_at = :consumed
                WHERE token_hash = :hash AND state = 'active'");
            $update->execute([':consumed' => $now, ':hash' => $tokenHash]);
            $result['token_state'] = 'consumed';
        }
        $this->recordIdempotency($idempotencyKey, 'validate_registration_token', $digest, $result, $now);
        return $result;
    }

    /** Revoke every active token of a registration (refund/revoke/supersession). Bounded. */
    public function revokeForRegistration(string $registrationUuid, string $requestId, string $reason): int
    {
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertRequestId($requestId);
        if ($reason === '' || strlen($reason) > 191 || preg_match('/[\r\n\x00]/', $reason)) {
            throw new InvalidArgumentException('bounded revocation reason required');
        }
        $table = $this->schema->table('wpuiai_edd_registration_tokens');
        $statement = $this->db->prepare("UPDATE {$table} SET state = 'revoked', revoked_at = :revoked
            WHERE registration_uuid = :registration AND state = 'active'");
        $statement->execute([':revoked' => $this->now(), ':registration' => $registrationUuid]);
        return $statement->rowCount();
    }

    /** Bounded active-token count for a registration (settlement/reconciliation). */
    public function activeTokenCount(string $registrationUuid): int
    {
        $this->assertUuid($registrationUuid, 'registration');
        $table = $this->schema->table('wpuiai_edd_registration_tokens');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}
            WHERE registration_uuid = :registration AND state = 'active'");
        $statement->execute([':registration' => $registrationUuid]);
        return (int) $statement->fetchColumn();
    }

    /** The stored keyed digest of a raw token; the raw token is never persisted. */
    public static function tokenHash(string $rawToken): string
    {
        return hash('sha256', "focusa.spec152e.edd.gate.token.v1\0" . $rawToken);
    }

    private function assertIssuableRegistration(array $registration, string $facadeId, string $productCode, string $now): void
    {
        if (!in_array((string) $registration['state'], self::VERIFIED_NONTERMINAL_STATES, true)
            || (string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if (!hash_equals((string) $registration['facade_id'], $facadeId)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if (!hash_equals((string) $registration['product_code'], $productCode)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddRegistrationTokenMigration::assertTimestamp($now);
        return $now;
    }

    private function replay(string $operation, string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_registration_token_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        foreach ($rows as $row) {
            if (!hash_equals($operation, (string) $row['operation'])) {
                continue;
            }
            if (!hash_equals($digest, (string) $row['request_digest'])) {
                throw new DomainException('IDEMPOTENCY_CONFLICT');
            }
            $payload = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
            return $payload['result'];
        }
        return null;
    }

    private function recordIdempotency(string $idempotencyKey, string $operation, string $digest, array $result, string $createdAt): void
    {
        $table = $this->schema->table('wpuiai_edd_registration_token_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, result_payload, created_at, retention_until)
            VALUES (:key, :operation, :digest, :payload, :created, :retention)");
        $statement->execute([
            ':key' => $idempotencyKey,
            ':operation' => $operation,
            ':digest' => $digest,
            ':payload' => json_encode([
                'operation' => $operation,
                'result' => $result,
            ], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':created' => $createdAt,
            ':retention' => self::plusSeconds($createdAt, $this->retention),
        ]);
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddRegistrationTokenMigration::encodeCanonical($value));
    }

    private static function opaqueToken(): string
    {
        return self::TOKEN_PREFIX . rtrim(strtr(base64_encode(random_bytes(32)), '+/', '-_'), '=');
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private function assertToken(string $value, int $maxLength): void
    {
        if ($value === '' || strlen($value) > $maxLength || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException('bounded registration token required');
        }
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
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
