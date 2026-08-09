<?php
// WPUIAI private authority account/identity/registration migrations for the Spec 172
// verified_no_license posture (atom focusa-vbcqu.20.15.10). The posture is persisted
// separately from paid EDD license truth: no EDD Software Licensing key, no zero-dollar
// fake license, no anonymous product capability, and no caller-controlled product,
// price, License Type, family, feature, limit, node, or commercial right. One posture
// binds the verified authority account, the verified email identity, and the verified
// registration to one product scope, one registered operator node, an explicit family
// allowlist, a monotonic authority sequence, issue/refresh times, a server-owned
// signer, and a status. Rollback is preservation-only: posture, node, and journal rows
// are never deleted.
declare(strict_types=1);

final class FocusaSpec172VerifiedAccessPostureState
{
    public const SCHEMA = 'focusa.spec172.verified_access_posture.v1';
    public const VERSION = 1;
    public const PRODUCT_SCOPES = ['focusa', 'uiai_engine'];
    public const STATUSES = ['issued', 'refreshed', 'revoked', 'superseded'];
    public const SIGNER_PATTERN = '/^wpuiai\.[a-z0-9._-]{4,63}$/D';

    // Server-owned explicit limited-mode allowlist (Spec 172 verified-limited-access.v1).
    public const FOCUSA_LIMITED_FAMILIES = [
        'manual_project',
        'manual_mission',
        'manual_focus_state',
        'manual_workpoint',
        'manual_trajectory',
        'manual_basic_evidence',
    ];
    public const UIAI_LIMITED_FAMILIES = [
        'public_search',
        'source_to_markdown',
        'public_page_read',
        'accessibility_snapshot',
        'screenshot',
        'basic_diagnostics',
    ];
    public const PERMANENT_FAMILIES = [
        'read_projection',
        'basic_customer_data_export',
        'account_control',
        'device_control',
        'license_status',
        'diagnostics',
        'repair',
        'rollback',
        'stable_security_update',
        'uninstall',
        'emergency_customer_data_recovery',
    ];

    /** Canonical explicit allowlist for a registered product scope; unknown products throw. */
    public static function allowlistFor(string $productScope): array
    {
        $families = match ($productScope) {
            'focusa' => self::FOCUSA_LIMITED_FAMILIES,
            'uiai_engine' => self::UIAI_LIMITED_FAMILIES,
            default => throw new InvalidArgumentException('unknown product scope'),
        };
        $allowlist = array_values(array_unique(array_merge($families, self::PERMANENT_FAMILIES)));
        sort($allowlist, SORT_STRING);
        return $allowlist;
    }

    public static function isRegisteredFamily(string $productScope, string $family): bool
    {
        return in_array($family, self::allowlistFor($productScope), true);
    }
}

final class FocusaSpec172VerifiedAccessPostureMigration
{
    public const SCHEMA = FocusaSpec172VerifiedAccessPostureState::SCHEMA;
    public const VERSION = FocusaSpec172VerifiedAccessPostureState::VERSION;

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
        $postures = $this->table('wpuiai_verified_access_postures');
        $nodes = $this->table('wpuiai_verified_access_nodes');
        $migrations = $this->table('wpuiai_verified_access_schema_migrations');
        $events = $this->table('wpuiai_verified_access_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$postures} (
            posture_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            identity_uuid {$uuid} NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            product_scope VARCHAR(32) NOT NULL CHECK (product_scope IN ('focusa', 'uiai_engine')),
            node_uuid VARCHAR(64) NOT NULL,
            node_digest VARCHAR(64) NOT NULL,
            family_allowlist TEXT NOT NULL,
            sequence BIGINT NOT NULL CHECK (sequence >= 1),
            issued_at VARCHAR(32) NOT NULL,
            refresh_at VARCHAR(32) NOT NULL,
            signer VARCHAR(64) NOT NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('issued', 'refreshed', 'revoked', 'superseded')),
            status_reason VARCHAR(191) NOT NULL,
            migration_provenance TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (account_uuid, product_scope, node_uuid)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$nodes} (
            node_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            node_digest VARCHAR(64) NOT NULL,
            registered_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (account_uuid, node_digest)
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

    /** Rollback is preservation-only: posture, node, and journal rows are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_verified_access_schema_events');
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

final class FocusaSpec172VerifiedAccessPostureRepository
{
    public const SCHEMA = FocusaSpec172VerifiedAccessPostureMigration::SCHEMA;
    public const VERSION = FocusaSpec172VerifiedAccessPostureMigration::VERSION;

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172VerifiedAccessPostureMigration $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Record exactly one verified_no_license posture from verified promotion input.
     * Only verified input creates a posture; unverified input fails closed with
     * EMAIL_VERIFICATION_REQUIRED and creates nothing. Replays return the existing
     * posture: one posture per (account, product scope, node). The posture binds the
     * authority account, the verified identity, the verified registration, product
     * scope, node, family allowlist, sequence, issue/refresh times, signer, and status.
     * No EDD key and no zero-dollar license is ever created.
     */
    public function recordPosture(array $input): array
    {
        if (!in_array($input['verification_state'] ?? null, ['mailbox_verified', 'account_promoted'], true)
            || !is_string($input['verified_at'] ?? null) || $input['verified_at'] === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($input['verified_at']);
        $accountUuid = $this->assertUuid((string) ($input['account_uuid'] ?? ''), 'account');
        $identityUuid = $this->assertUuid((string) ($input['identity_uuid'] ?? ''), 'identity');
        $registrationUuid = $this->assertUuid((string) ($input['registration_uuid'] ?? ''), 'registration');
        $productScope = (string) ($input['product_scope'] ?? '');
        if (!in_array($productScope, FocusaSpec172VerifiedAccessPostureState::PRODUCT_SCOPES, true)) {
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        $nodeUuid = $this->assertNodeUuid((string) ($input['node_uuid'] ?? ''));
        $nodeDigest = $this->assertNodeDigest((string) ($input['node_digest'] ?? ''));
        $families = $this->validateFamilies($productScope, $input['family_allowlist'] ?? null);
        $signer = (string) ($input['signer'] ?? '');
        if (preg_match(FocusaSpec172VerifiedAccessPostureState::SIGNER_PATTERN, $signer) !== 1) {
            throw new InvalidArgumentException('server-owned signer required');
        }
        $sequence = filter_var($input['sequence'] ?? null, FILTER_VALIDATE_INT);
        if ($sequence === false || $sequence < 1) {
            throw new InvalidArgumentException('positive authority sequence required');
        }
        $issuedAt = (string) ($input['issued_at'] ?? '');
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($issuedAt);
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($refreshAt);
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encodedProvenance = $this->schema->encodeCanonical($provenance);
        $allowlist = $this->schema->encodeCanonical($families);

        return $this->transaction(function () use ($accountUuid, $identityUuid, $registrationUuid, $productScope, $nodeUuid, $nodeDigest, $allowlist, $signer, $sequence, $issuedAt, $refreshAt, $encodedProvenance): array {
            $existing = $this->findByAccountProductNode($accountUuid, $productScope, $nodeUuid);
            if ($existing !== null) {
                return $existing;
            }
            $now = $this->now();
            $postureUuid = self::uuid();
            $table = $this->schema->table('wpuiai_verified_access_postures');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (posture_uuid, account_uuid, identity_uuid, registration_uuid, product_scope, node_uuid,
                 node_digest, family_allowlist, sequence, issued_at, refresh_at, signer, status, status_reason,
                 migration_provenance, created_at, updated_at)
                VALUES (:posture, :account, :identity, :registration, :product, :node, :node_digest,
                        :allowlist, :sequence, :issued, :refresh, :signer, 'issued', 'mailbox_verified',
                        :provenance, :created, :updated)");
            $statement->execute([
                ':posture' => $postureUuid,
                ':account' => $accountUuid,
                ':identity' => $identityUuid,
                ':registration' => $registrationUuid,
                ':product' => $productScope,
                ':node' => $nodeUuid,
                ':node_digest' => $nodeDigest,
                ':allowlist' => $allowlist,
                ':sequence' => $sequence,
                ':issued' => $issuedAt,
                ':refresh' => $refreshAt,
                ':signer' => $signer,
                ':provenance' => $encodedProvenance,
                ':created' => $now,
                ':updated' => $now,
            ]);
            $nodes = $this->schema->table('wpuiai_verified_access_nodes');
            $nodeStatement = $this->db->prepare("INSERT INTO {$nodes}
                (node_uuid, account_uuid, node_digest, registered_at, migration_provenance)
                SELECT :node, :account, :node_digest, :registered, :provenance
                WHERE NOT EXISTS (SELECT 1 FROM {$nodes}
                    WHERE account_uuid = :account_guard AND node_digest = :digest_guard)");
            $nodeStatement->execute([
                ':node' => $nodeUuid,
                ':account' => $accountUuid,
                ':node_digest' => $nodeDigest,
                ':registered' => $now,
                ':provenance' => $encodedProvenance,
                ':account_guard' => $accountUuid,
                ':digest_guard' => $nodeDigest,
            ]);
            return $this->findByUuid($postureUuid);
        });
    }

    /**
     * Advance the posture sequence and bounded refresh window. Idempotent: a same-or-older
     * request leaves the posture unchanged; a revoked or superseded posture denies with
     * VERIFIED_LIMITED_ACCESS. No access expiry is ever imposed here.
     */
    public function advanceSequence(string $postureUuid, int $nextSequence, string $refreshAt): array
    {
        if ($nextSequence < 1) {
            throw new InvalidArgumentException('positive sequence required');
        }
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($refreshAt);
        return $this->transaction(function () use ($postureUuid, $nextSequence, $refreshAt): array {
            $posture = $this->findByUuid($postureUuid);
            if ((int) $posture['sequence'] >= $nextSequence) {
                return $posture;
            }
            if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
                throw new DomainException('VERIFIED_LIMITED_ACCESS');
            }
            $now = $this->now();
            $table = $this->schema->table('wpuiai_verified_access_postures');
            $statement = $this->db->prepare("UPDATE {$table}
                SET sequence = :sequence, refresh_at = :refresh, updated_at = :updated
                WHERE posture_uuid = :posture AND sequence < :sequence_guard");
            $statement->execute([
                ':sequence' => $nextSequence,
                ':refresh' => $refreshAt,
                ':updated' => $now,
                ':posture' => $postureUuid,
                ':sequence_guard' => $nextSequence,
            ]);
            if ($statement->rowCount() !== 1) {
                throw new RuntimeException('concurrent posture advance denied');
            }
            return $this->findByUuid($postureUuid);
        });
    }

    /** Preservation-only revoke: rows are kept, status flips with an explicit reason. */
    public function revokePosture(string $postureUuid, string $reason, string $occurredAt, array $provenance): array
    {
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($occurredAt);
        if ($reason === '' || strlen($reason) > 191 || preg_match('/[\r\n\x00]/', $reason)) {
            throw new InvalidArgumentException('bounded revoke reason required');
        }
        $encoded = $this->schema->encodeCanonical($provenance);
        return $this->transaction(function () use ($postureUuid, $reason, $occurredAt, $encoded): array {
            $posture = $this->findByUuid($postureUuid);
            if (in_array((string) $posture['status'], ['revoked', 'superseded'], true)) {
                return $posture;
            }
            $table = $this->schema->table('wpuiai_verified_access_postures');
            $statement = $this->db->prepare("UPDATE {$table}
                SET status = 'revoked', status_reason = :reason, updated_at = :occurred
                WHERE posture_uuid = :posture");
            $statement->execute([':reason' => $reason, ':occurred' => $occurredAt, ':posture' => $postureUuid]);
            $this->recordEvent('posture_revoked', $occurredAt, $encoded);
            return $this->findByUuid($postureUuid);
        });
    }

    public function findByAccountProductNode(string $accountUuid, string $productScope, string $nodeUuid): ?array
    {
        $table = $this->schema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE account_uuid = :account AND product_scope = :product AND node_uuid = :node LIMIT 1");
        $statement->execute([':account' => $accountUuid, ':product' => $productScope, ':node' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function findByUuid(string $postureUuid): array
    {
        $table = $this->schema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE posture_uuid = :posture LIMIT 1");
        $statement->execute([':posture' => $postureUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('verified access posture not found');
        }
        return $row;
    }

    public function countForAccount(string $accountUuid): int
    {
        $table = $this->schema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE account_uuid = :account");
        $statement->execute([':account' => $accountUuid]);
        return (int) $statement->fetchColumn();
    }

    public function recordEvent(string $eventType, string $occurredAt, string $encodedProvenance): void
    {
        $events = $this->schema->table('wpuiai_verified_access_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\n" . $eventType . "\n" . $occurredAt . "\n" . $encodedProvenance);
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, :event_type, :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey,
            ':event_type' => $eventType,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => $encodedProvenance,
            ':existing_key' => $eventKey,
        ]);
    }

    // ── validation helpers ───────────────────────────────────────────────

    private function validateFamilies(string $productScope, mixed $value): array
    {
        if (!is_array($value) || $value === []) {
            throw new InvalidArgumentException('explicit family allowlist required');
        }
        $result = [];
        foreach ($value as $family) {
            if (!is_string($family) || preg_match('/^[a-z][a-z0-9_]{1,63}$/D', $family) !== 1) {
                throw new InvalidArgumentException('registered family code required');
            }
            if (!FocusaSpec172VerifiedAccessPostureState::isRegisteredFamily($productScope, $family)) {
                throw new DomainException('CAPABILITY_FAMILY_NOT_INCLUDED');
            }
            $result[] = $family;
        }
        sort($result, SORT_STRING);
        return $result;
    }

    private function assertUuid(string $uuid, string $kind): string
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
        return $uuid;
    }

    private function assertNodeUuid(string $nodeUuid): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,64}$/D', $nodeUuid) !== 1) {
            throw new InvalidArgumentException('bounded opaque node identifier required');
        }
        return $nodeUuid;
    }

    private function assertNodeDigest(string $nodeDigest): string
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $nodeDigest) !== 1) {
            throw new InvalidArgumentException('canonical node digest required');
        }
        return $nodeDigest;
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($now);
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

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }
}
