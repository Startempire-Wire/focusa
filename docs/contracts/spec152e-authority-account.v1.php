<?php
// Candidate-owned authority-account schema/repository seam. It does not bootstrap WordPress.
declare(strict_types=1);

final class FocusaSpec152eAuthorityAccountMigration
{
    public const SCHEMA = 'focusa.spec152e.authority_account.v1';
    public const VERSION = 1;

    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->prefix = $prefix;
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        self::assertTimestamp($appliedAt);
        $encodedProvenance = self::encodeProvenance($provenance);
        $account = $this->table('wpuiai_authority_accounts');
        $idempotency = $this->table('wpuiai_authority_account_idempotency');
        $migrations = $this->table('wpuiai_authority_schema_migrations');
        $events = $this->table('wpuiai_authority_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$account} (
            account_uuid {$uuid} NOT NULL PRIMARY KEY,
            edd_customer_id BIGINT NOT NULL UNIQUE,
            wordpress_user_id BIGINT NULL,
            stripe_customer_id VARCHAR(191) NULL,
            status VARCHAR(32) NOT NULL,
            status_reason VARCHAR(191) NOT NULL,
            highest_entitlement_sequence BIGINT NOT NULL DEFAULT 0 CHECK (highest_entitlement_sequence >= 0),
            migration_provenance TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NOT NULL,
            result_sequence BIGINT NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$events} (
            event_key {$key} NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            schema_version BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");

        $statement = $this->db->prepare(
            "INSERT INTO {$migrations} (schema_version, schema_name, applied_at, migration_provenance)
             SELECT :version, :schema, :applied_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)"
        );
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied_at' => $appliedAt,
            ':provenance' => $encodedProvenance,
            ':existing_version' => self::VERSION,
        ]);
    }

    /**
     * Rollback is intentionally data-preserving. Older code may be restored, but authority
     * identity, commerce links, sequence, provenance, timestamps, and journals are not undone.
     */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeProvenance($provenance);
        $events = $this->table('wpuiai_authority_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare(
            "INSERT INTO {$events} (event_key, event_type, schema_version, occurred_at, migration_provenance)
             SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)"
        );
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

    public static function encodeProvenance(array $provenance): string
    {
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $normalize = static function (mixed $value) use (&$normalize): mixed {
            if (!is_array($value)) {
                return $value;
            }
            if (!array_is_list($value)) {
                ksort($value, SORT_STRING);
            }
            foreach ($value as $index => $item) {
                $value[$index] = $normalize($item);
            }
            return $value;
        };
        return json_encode($normalize($provenance), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }

    public static function assertTimestamp(string $timestamp): void
    {
        $parsed = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }
}

final class FocusaSpec152eAuthorityAccountRepository
{
    private PDO $db;
    private FocusaSpec152eAuthorityAccountMigration $schema;
    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(PDO $db, FocusaSpec152eAuthorityAccountMigration $schema, callable $clock)
    {
        $this->db = $db;
        $this->schema = $schema;
        $this->clock = Closure::fromCallable($clock);
    }

    public function promoteVerified(array $attempt): array
    {
        if (!in_array($attempt['verification_state'] ?? null, ['mailbox_verified', 'account_promoted'], true)
            || !is_string($attempt['verified_at'] ?? null) || $attempt['verified_at'] === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        FocusaSpec152eAuthorityAccountMigration::assertTimestamp($attempt['verified_at']);
        $accountUuid = (string) ($attempt['account_uuid'] ?? '');
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $accountUuid) !== 1) {
            throw new InvalidArgumentException('canonical opaque account UUID required');
        }
        $customerId = filter_var($attempt['edd_customer_id'] ?? null, FILTER_VALIDATE_INT);
        if ($customerId === false || $customerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        $idempotencyKey = $this->idempotencyKey($attempt);
        $provenance = FocusaSpec152eAuthorityAccountMigration::encodeProvenance($attempt['migration_provenance'] ?? []);
        $digest = $this->digest([
            'account_uuid' => $accountUuid,
            'edd_customer_id' => $customerId,
            'wordpress_user_id' => $attempt['wordpress_user_id'] ?? null,
            'stripe_customer_id' => $attempt['stripe_customer_id'] ?? null,
            'verified_at' => $attempt['verified_at'],
            'migration_provenance' => json_decode($provenance, true, 512, JSON_THROW_ON_ERROR),
        ]);
        return $this->transaction(function () use ($attempt, $accountUuid, $customerId, $idempotencyKey, $provenance, $digest): array {
            $replay = $this->replay($idempotencyKey, 'promote_verified', $digest);
            if ($replay !== null) {
                return $this->findByUuid($replay['account_uuid']);
            }
            $now = ($this->clock)();
            FocusaSpec152eAuthorityAccountMigration::assertTimestamp($now);
            $existing = $this->findByCustomerId($customerId);
            if ($existing === null) {
                $table = $this->schema->table('wpuiai_authority_accounts');
                $statement = $this->db->prepare("INSERT INTO {$table}
                    (account_uuid, edd_customer_id, wordpress_user_id, stripe_customer_id, status, status_reason,
                     highest_entitlement_sequence, migration_provenance, created_at, updated_at)
                    VALUES (:uuid, :customer, :wp_user, :stripe, 'active', 'mailbox_verified', 0, :provenance, :created, :updated)");
                $statement->execute([
                    ':uuid' => $accountUuid,
                    ':customer' => $customerId,
                    ':wp_user' => $attempt['wordpress_user_id'] ?? null,
                    ':stripe' => $attempt['stripe_customer_id'] ?? null,
                    ':provenance' => $provenance,
                    ':created' => $now,
                    ':updated' => $now,
                ]);
                $existing = $this->findByUuid($accountUuid);
            }
            $this->recordIdempotency($idempotencyKey, 'promote_verified', $digest, $existing['account_uuid'], (int) $existing['highest_entitlement_sequence'], $now);
            return $existing;
        });
    }

    public function advanceSequence(string $accountUuid, int $nextSequence, string $idempotencyKey): array
    {
        if ($nextSequence < 1) {
            throw new InvalidArgumentException('positive sequence required');
        }
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest(['account_uuid' => $accountUuid, 'next_sequence' => $nextSequence]);

        return $this->transaction(function () use ($accountUuid, $nextSequence, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'advance_sequence', $digest);
            if ($replay !== null) {
                return $this->findByUuid($accountUuid);
            }
            $account = $this->findByUuid($accountUuid);
            if ($nextSequence <= (int) $account['highest_entitlement_sequence']) {
                throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
            }
            $now = ($this->clock)();
            FocusaSpec152eAuthorityAccountMigration::assertTimestamp($now);
            $table = $this->schema->table('wpuiai_authority_accounts');
            $statement = $this->db->prepare("UPDATE {$table}
                SET highest_entitlement_sequence = :sequence, updated_at = :updated
                WHERE account_uuid = :uuid AND highest_entitlement_sequence < :sequence_guard");
            $statement->execute([':sequence' => $nextSequence, ':updated' => $now, ':uuid' => $accountUuid, ':sequence_guard' => $nextSequence]);
            if ($statement->rowCount() !== 1) {
                throw new RuntimeException('concurrent sequence advance denied');
            }
            $this->recordIdempotency($idempotencyKey, 'advance_sequence', $digest, $accountUuid, $nextSequence, $now);
            return $this->findByUuid($accountUuid);
        });
    }

    public function findByCustomerId(int $customerId): ?array
    {
        $table = $this->schema->table('wpuiai_authority_accounts');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE edd_customer_id = :customer");
        $statement->execute([':customer' => $customerId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function findByUuid(string $accountUuid): array
    {
        $table = $this->schema->table('wpuiai_authority_accounts');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE account_uuid = :uuid");
        $statement->execute([':uuid' => $accountUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('authority account not found');
        }
        return $row;
    }

    /**
     * Caller-owned transaction primitive: resolve or create the authority account for an
     * already-resolved EDD customer, linking optional WordPress and Stripe references without
     * creating duplicates. Used by the atomic verified-account promotion service; never
     * starts its own transaction. Returns the account row plus the resolution outcome.
     */
    public function resolveForPromotionInTransaction(int $customerId, ?int $wordpressUserId, ?string $stripeCustomerId, string $provenance, string $verifiedAt): array
    {
        if ($customerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        FocusaSpec152eAuthorityAccountMigration::assertTimestamp($verifiedAt);
        if ($provenance === '') {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $now = ($this->clock)();
        FocusaSpec152eAuthorityAccountMigration::assertTimestamp($now);
        $existing = $this->findByCustomerId($customerId);
        if ($existing !== null) {
            $this->linkOptionalReferencesInTransaction((int) $existing['edd_customer_id'], $wordpressUserId, $stripeCustomerId, $now);
            return ['account' => $this->findByUuid((string) $existing['account_uuid']), 'resolution' => 'existing'];
        }
        $accountUuid = self::uuid();
        $table = $this->schema->table('wpuiai_authority_accounts');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (account_uuid, edd_customer_id, wordpress_user_id, stripe_customer_id, status, status_reason,
             highest_entitlement_sequence, migration_provenance, created_at, updated_at)
            VALUES (:uuid, :customer, :wp_user, :stripe, 'active', 'mailbox_verified', 0, :provenance, :created, :updated)");
        $statement->execute([
            ':uuid' => $accountUuid,
            ':customer' => $customerId,
            ':wp_user' => $wordpressUserId,
            ':stripe' => $stripeCustomerId,
            ':provenance' => $provenance,
            ':created' => $now,
            ':updated' => $now,
        ]);
        return ['account' => $this->findByUuid($accountUuid), 'resolution' => 'new'];
    }

    /** Point lookup used to prove a WordPress user is not already linked to another account. */
    public function findByWordpressUserId(int $wordpressUserId): ?array
    {
        if ($wordpressUserId < 1) {
            throw new InvalidArgumentException('positive WordPress user ID required');
        }
        $table = $this->schema->table('wpuiai_authority_accounts');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE wordpress_user_id = :user LIMIT 1");
        $statement->execute([':user' => $wordpressUserId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Point lookup used to prove a Stripe customer is not already linked to another account. */
    public function findByStripeCustomerId(string $stripeCustomerId): ?array
    {
        if ($stripeCustomerId === '' || strlen($stripeCustomerId) > 191 || preg_match('/[\r\n]/', $stripeCustomerId)) {
            throw new InvalidArgumentException('bounded Stripe customer ID required');
        }
        $table = $this->schema->table('wpuiai_authority_accounts');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE stripe_customer_id = :stripe LIMIT 1");
        $statement->execute([':stripe' => $stripeCustomerId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function linkOptionalReferencesInTransaction(int $customerId, ?int $wordpressUserId, ?string $stripeCustomerId, string $now): void
    {
        $table = $this->schema->table('wpuiai_authority_accounts');
        $fields = [];
        $params = [':customer' => $customerId];
        if ($wordpressUserId !== null) {
            $row = $this->findByCustomerId($customerId);
            if ($row !== null && $row['wordpress_user_id'] !== null && (int) $row['wordpress_user_id'] !== $wordpressUserId) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
            $fields[] = 'wordpress_user_id = :wp_user';
            $params[':wp_user'] = $wordpressUserId;
        }
        if ($stripeCustomerId !== null) {
            $row = $this->findByCustomerId($customerId);
            if ($row !== null && $row['stripe_customer_id'] !== null
                && !hash_equals((string) $row['stripe_customer_id'], $stripeCustomerId)) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
            $fields[] = 'stripe_customer_id = :stripe';
            $params[':stripe'] = $stripeCustomerId;
        }
        if ($fields === []) {
            return;
        }
        $fields[] = 'updated_at = :updated';
        $params[':updated'] = $now;
        $setClause = implode(', ', $fields);
        $this->db->prepare("UPDATE {$table} SET {$setClause} WHERE edd_customer_id = :customer")->execute($params);
    }

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }

    private function replay(string $key, string $operation, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_authority_account_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $key]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($operation, $row['operation']) || !hash_equals($digest, $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $key, string $operation, string $digest, string $accountUuid, int $sequence, string $createdAt): void
    {
        $table = $this->schema->table('wpuiai_authority_account_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, account_uuid, result_sequence, created_at)
            VALUES (:key, :operation, :digest, :uuid, :sequence, :created)");
        $statement->execute([
            ':key' => $key, ':operation' => $operation, ':digest' => $digest,
            ':uuid' => $accountUuid, ':sequence' => $sequence, ':created' => $createdAt,
        ]);
    }

    private function idempotencyKey(array $attempt): string
    {
        $key = (string) ($attempt['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($key);
        return $key;
    }

    private function assertIdempotencyKey(string $key): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
    }

    private function digest(array $value): string
    {
        return hash('sha256', FocusaSpec152eAuthorityAccountMigration::encodeProvenance($value));
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
}
