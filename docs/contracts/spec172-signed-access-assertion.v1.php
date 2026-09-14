<?php
// Spec 172 signed-access-assertion schema/repository (atom focusa-vbcqu.20.15.10).
// An assertion is the authority-signed limited-access envelope for the permanent
// verified_no_license posture: it binds the posture, account, verified identity,
// product scope, node, family allowlist, monotonic sequence, issue/refresh times,
// server-owned signer, and status. It is a signed assertion model, NOT an EDD
// Software Licensing key: no edd license/order/customer columns, no price, no
// License Type, no grant override, and no caller-controlled commercial input exist
// anywhere in the schema (assertEddFree() introspects and proves it). Real
// cryptographic issuance/refresh/revoke service work is the downstream atom
// (172.02.02); this atom freezes the idempotent schema and repository model.
declare(strict_types=1);

final class FocusaSpec172SignedAccessAssertionMigration
{
    public const SCHEMA = 'focusa.spec172.signed_access_assertion.v1';
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
        $assertions = $this->table('wpuiai_signed_access_assertions');
        $idempotency = $this->table('wpuiai_signed_access_assertion_idempotency');
        $migrations = $this->table('wpuiai_signed_access_assertion_schema_migrations');
        $events = $this->table('wpuiai_signed_access_assertion_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$assertions} (
            assertion_uuid {$uuid} NOT NULL PRIMARY KEY,
            posture_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NOT NULL,
            identity_uuid {$uuid} NOT NULL,
            product_scope VARCHAR(32) NOT NULL CHECK (product_scope IN ('focusa', 'uiai_engine')),
            node_uuid VARCHAR(64) NOT NULL,
            family_allowlist TEXT NOT NULL,
            sequence BIGINT NOT NULL CHECK (sequence >= 1),
            issued_at VARCHAR(32) NOT NULL,
            refresh_at VARCHAR(32) NOT NULL,
            signer VARCHAR(64) NOT NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('issued', 'refreshed', 'revoked', 'superseded')),
            signature_algorithm VARCHAR(32) NOT NULL,
            signature TEXT NOT NULL,
            content_digest VARCHAR(64) NOT NULL,
            previous_assertion_uuid {$uuid} NULL,
            migration_provenance TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (posture_uuid, sequence)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            posture_uuid {$uuid} NOT NULL,
            assertion_uuid {$uuid} NOT NULL,
            created_at VARCHAR(32) NOT NULL
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

    /** Rollback is preservation-only: assertion rows and journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_signed_access_assertion_schema_events');
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

final class FocusaSpec172SignedAccessAssertionRepository
{
    public const SCHEMA = FocusaSpec172SignedAccessAssertionMigration::SCHEMA;
    public const VERSION = FocusaSpec172SignedAccessAssertionMigration::VERSION;
    public const SIGNATURE_ALGORITHM = 'ed25519.spec172.v1';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172SignedAccessAssertionMigration $schema,
        private FocusaSpec172VerifiedAccessPostureMigration $postures,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Record one signed limited-access assertion from an active verified posture.
     * Fails closed when: no verified posture exists (EMAIL_VERIFICATION_REQUIRED), the
     * posture is revoked/superseded (VERIFIED_LIMITED_ACCESS), the product scope or node
     * does not match the posture (ENTITLEMENT_PRODUCT_MISMATCH / NODE_LIMIT_REACHED), a
     * family is not in the posture allowlist (CAPABILITY_FAMILY_NOT_INCLUDED), or the
     * sequence would roll back (ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED). Replays with the
     * same (posture, sequence) return the same assertion. The content digest models what
     * is signed; cryptographic issuance is the downstream 172.02.02 atom.
     */
    public function recordAssertion(array $input): array
    {
        $postureUuid = $this->assertUuid((string) ($input['posture_uuid'] ?? ''), 'posture');
        $posture = $this->loadPosture($postureUuid);
        if ($posture === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        $productScope = (string) ($input['product_scope'] ?? '');
        if ($productScope === '' || !hash_equals((string) $posture['product_scope'], $productScope)) {
            throw new DomainException('ENTITLEMENT_PRODUCT_MISMATCH');
        }
        $nodeUuid = $this->assertNodeUuid((string) ($input['node_uuid'] ?? ''));
        if (!hash_equals((string) $posture['node_uuid'], $nodeUuid)) {
            throw new DomainException('NODE_LIMIT_REACHED');
        }
        $families = $this->validateAssertedFamilies($posture, $input['family_allowlist'] ?? null);
        $sequence = filter_var($input['sequence'] ?? null, FILTER_VALIDATE_INT);
        if ($sequence === false || $sequence < 1) {
            throw new InvalidArgumentException('positive assertion sequence required');
        }
        if ((string) ($input['signature_algorithm'] ?? '') !== self::SIGNATURE_ALGORITHM) {
            throw new InvalidArgumentException('server-owned signature algorithm required');
        }
        $signature = (string) ($input['signature'] ?? '');
        if ($signature === '' || strlen($signature) > 512 || preg_match('/[\r\n\x00]/', $signature)) {
            throw new InvalidArgumentException('bounded opaque signature required');
        }
        $issuedAt = (string) ($input['issued_at'] ?? '');
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        FocusaSpec172SignedAccessAssertionMigration::assertTimestamp($issuedAt);
        FocusaSpec172SignedAccessAssertionMigration::assertTimestamp($refreshAt);
        $signer = (string) ($input['signer'] ?? '');
        if (preg_match(FocusaSpec172VerifiedAccessPostureState::SIGNER_PATTERN, $signer) !== 1) {
            throw new InvalidArgumentException('server-owned signer required');
        }
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encodedProvenance = $this->schema->encodeCanonical($provenance);
        $allowlist = $this->schema->encodeCanonical($families);

        return $this->transaction(function () use ($posture, $postureUuid, $productScope, $nodeUuid, $allowlist, $families, $sequence, $signature, $issuedAt, $refreshAt, $signer, $encodedProvenance): array {
            $replay = $this->findByPostureSequence($postureUuid, $sequence);
            if ($replay !== null) {
                return $replay;
            }
            $latest = $this->findLatestByPosture($postureUuid);
            if ($latest !== null && $sequence < (int) $latest['sequence']) {
                throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
            }
            $now = $this->now();
            $contentDigest = self::modelDigest([
                'schema' => self::SCHEMA,
                'posture_uuid' => $postureUuid,
                'account_uuid' => (string) $posture['account_uuid'],
                'identity_uuid' => (string) $posture['identity_uuid'],
                'product_scope' => $productScope,
                'node_uuid' => $nodeUuid,
                'family_allowlist' => $families,
                'sequence' => $sequence,
                'issued_at' => $issuedAt,
                'refresh_at' => $refreshAt,
                'signer' => $signer,
            ]);
            $assertionUuid = self::uuid();
            $table = $this->schema->table('wpuiai_signed_access_assertions');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (assertion_uuid, posture_uuid, account_uuid, identity_uuid, product_scope, node_uuid,
                 family_allowlist, sequence, issued_at, refresh_at, signer, status, signature_algorithm,
                 signature, content_digest, previous_assertion_uuid, migration_provenance, created_at, updated_at)
                VALUES (:assertion, :posture, :account, :identity, :product, :node, :allowlist, :sequence,
                        :issued, :refresh, :signer, 'issued', :algorithm, :signature, :digest,
                        :previous, :provenance, :created, :updated)");
            $statement->execute([
                ':assertion' => $assertionUuid,
                ':posture' => $postureUuid,
                ':account' => $posture['account_uuid'],
                ':identity' => $posture['identity_uuid'],
                ':product' => $productScope,
                ':node' => $nodeUuid,
                ':allowlist' => $allowlist,
                ':sequence' => $sequence,
                ':issued' => $issuedAt,
                ':refresh' => $refreshAt,
                ':signer' => $signer,
                ':algorithm' => self::SIGNATURE_ALGORITHM,
                ':signature' => $signature,
                ':digest' => $contentDigest,
                ':previous' => $latest === null ? null : $latest['assertion_uuid'],
                ':provenance' => $encodedProvenance,
                ':created' => $now,
                ':updated' => $now,
            ]);
            $this->advancePostureInTransaction($posture, $sequence, $refreshAt);
            return $this->findByUuid($assertionUuid);
        });
    }

    /**
     * Bounded-credential refresh: rotate the current assertion with a higher sequence and a
     * fresh refresh window, WITHOUT imposing any access expiry on the permanent posture.
     * Replays return the already-refreshed assertion (same next sequence).
     */
    public function refreshAssertion(array $input): array
    {
        $postureUuid = $this->assertUuid((string) ($input['posture_uuid'] ?? ''), 'posture');
        $posture = $this->loadPosture($postureUuid);
        if ($posture === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        if ((string) ($input['signature_algorithm'] ?? '') !== self::SIGNATURE_ALGORITHM) {
            throw new InvalidArgumentException('server-owned signature algorithm required');
        }
        $signature = (string) ($input['signature'] ?? '');
        if ($signature === '' || strlen($signature) > 512 || preg_match('/[\r\n\x00]/', $signature)) {
            throw new InvalidArgumentException('bounded opaque signature required');
        }
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        FocusaSpec172SignedAccessAssertionMigration::assertTimestamp($refreshAt);
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encodedProvenance = $this->schema->encodeCanonical($provenance);
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $digest = $this->digest([
            'operation' => 'refresh_assertion',
            'posture_uuid' => $postureUuid,
            'refresh_at' => $refreshAt,
            'migration_provenance' => $provenance,
        ]);

        return $this->transaction(function () use ($posture, $postureUuid, $signature, $refreshAt, $encodedProvenance, $idempotencyKey, $digest): array {
            $replay = $this->replayIdempotency($idempotencyKey, 'refresh_assertion', $digest);
            if ($replay !== null) {
                return $this->findByUuid((string) $replay['assertion_uuid']);
            }
            $latest = $this->findLatestByPosture($postureUuid);
            if ($latest === null) {
                throw new DomainException('VERIFIED_LIMITED_ACCESS');
            }
            $nextSequence = (int) $latest['sequence'] + 1;
            $existing = $this->findByPostureSequence($postureUuid, $nextSequence);
            if ($existing !== null) {
                return $existing;
            }
            $now = $this->now();
            $allowlist = (string) $posture['family_allowlist'];
            $families = json_decode($allowlist, true, 512, JSON_THROW_ON_ERROR);
            $contentDigest = self::modelDigest([
                'schema' => self::SCHEMA,
                'posture_uuid' => $postureUuid,
                'account_uuid' => (string) $posture['account_uuid'],
                'identity_uuid' => (string) $posture['identity_uuid'],
                'product_scope' => (string) $posture['product_scope'],
                'node_uuid' => (string) $posture['node_uuid'],
                'family_allowlist' => $families,
                'sequence' => $nextSequence,
                'issued_at' => (string) $latest['issued_at'],
                'refresh_at' => $refreshAt,
                'signer' => 'wpuiai.spec172.refresh.v1',
            ]);
            $assertionUuid = self::uuid();
            $table = $this->schema->table('wpuiai_signed_access_assertions');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (assertion_uuid, posture_uuid, account_uuid, identity_uuid, product_scope, node_uuid,
                 family_allowlist, sequence, issued_at, refresh_at, signer, status, signature_algorithm,
                 signature, content_digest, previous_assertion_uuid, migration_provenance, created_at, updated_at)
                VALUES (:assertion, :posture, :account, :identity, :product, :node, :allowlist, :sequence,
                        :issued, :refresh, :signer, 'refreshed', :algorithm, :signature, :digest,
                        :previous, :provenance, :created, :updated)");
            $statement->execute([
                ':assertion' => $assertionUuid,
                ':posture' => $postureUuid,
                ':account' => $posture['account_uuid'],
                ':identity' => $posture['identity_uuid'],
                ':product' => $posture['product_scope'],
                ':node' => $posture['node_uuid'],
                ':allowlist' => $allowlist,
                ':sequence' => $nextSequence,
                ':issued' => $latest['issued_at'],
                ':refresh' => $refreshAt,
                ':signer' => 'wpuiai.spec172.refresh.v1',
                ':algorithm' => self::SIGNATURE_ALGORITHM,
                ':signature' => $signature,
                ':digest' => $contentDigest,
                ':previous' => $latest['assertion_uuid'],
                ':provenance' => $encodedProvenance,
                ':created' => $now,
                ':updated' => $now,
            ]);
            $this->advancePostureInTransaction($posture, $nextSequence, $refreshAt);
            $this->recordIdempotency($idempotencyKey, 'refresh_assertion', $digest, $postureUuid, $assertionUuid, $now);
            return $this->findByUuid($assertionUuid);
        });
    }

    /** Preservation-only revoke: the current assertion and posture are marked, never deleted. */
    public function revokeAssertion(string $postureUuid, string $reason, string $occurredAt, array $provenance): array
    {
        $this->assertUuid($postureUuid, 'posture');
        FocusaSpec172SignedAccessAssertionMigration::assertTimestamp($occurredAt);
        if ($reason === '' || strlen($reason) > 191 || preg_match('/[\r\n\x00]/', $reason)) {
            throw new InvalidArgumentException('bounded revoke reason required');
        }
        $encoded = $this->schema->encodeCanonical($provenance);
        return $this->transaction(function () use ($postureUuid, $reason, $occurredAt, $encoded): array {
            $latest = $this->findLatestByPosture($postureUuid);
            if ($latest === null) {
                throw new DomainException('VERIFIED_LIMITED_ACCESS');
            }
            $table = $this->schema->table('wpuiai_signed_access_assertions');
            if ((string) $latest['status'] !== 'revoked') {
                $statement = $this->db->prepare("UPDATE {$table}
                    SET status = 'revoked', updated_at = :occurred
                    WHERE assertion_uuid = :assertion");
                $statement->execute([':occurred' => $occurredAt, ':assertion' => $latest['assertion_uuid']]);
            }
            $this->revokePostureInTransaction($postureUuid, $reason, $occurredAt, $encoded);
            return $this->findByUuid((string) $latest['assertion_uuid']);
        });
    }

    public function findByUuid(string $assertionUuid): array
    {
        $table = $this->schema->table('wpuiai_signed_access_assertions');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE assertion_uuid = :assertion LIMIT 1");
        $statement->execute([':assertion' => $assertionUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('signed access assertion not found');
        }
        return $row;
    }

    public function findByPostureSequence(string $postureUuid, int $sequence): ?array
    {
        $table = $this->schema->table('wpuiai_signed_access_assertions');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE posture_uuid = :posture AND sequence = :sequence LIMIT 1");
        $statement->execute([':posture' => $postureUuid, ':sequence' => $sequence]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function findLatestByPosture(string $postureUuid): ?array
    {
        $table = $this->schema->table('wpuiai_signed_access_assertions');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE posture_uuid = :posture ORDER BY sequence DESC LIMIT 1");
        $statement->execute([':posture' => $postureUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function countForPosture(string $postureUuid): int
    {
        $table = $this->schema->table('wpuiai_signed_access_assertions');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE posture_uuid = :posture");
        $statement->execute([':posture' => $postureUuid]);
        return (int) $statement->fetchColumn();
    }

    /**
     * Deterministic digest of the canonical assertion fields — the model of what the
     * authority signs. Never includes the signature envelope or caller-supplied extras.
     */
    public static function modelDigest(array $fields): string
    {
        return hash('sha256', self::encodeCanonical($fields));
    }

    public static function encodeCanonical(array $value): string
    {
        return FocusaSpec172SignedAccessAssertionMigration::encodeCanonical($value);
    }

    /**
     * Introspection guard proving no assertion or posture row can be interpreted as an
     * EDD license: no edd license/order/customer columns, no key, price, amount, total,
     * currency, License Type, download, or grant columns exist in either schema.
     */
    public function assertEddFree(): array
    {
        $forbidden = [
            'edd_license', 'edd_order', 'edd_customer', 'edd_product', 'download_id',
            'license_key', 'price', 'amount', 'total', 'currency', 'license_type', 'grant',
        ];
        $tables = ['wpuiai_signed_access_assertions', 'wpuiai_verified_access_postures'];
        foreach ($tables as $table) {
            $name = $this->schema->table($table);
            $rows = $this->db->query("PRAGMA table_info({$name})")->fetchAll(PDO::FETCH_ASSOC);
            if ($rows === []) {
                throw new DomainException('ASSERTION_SCHEMA_MISSING');
            }
            foreach ($rows as $row) {
                $column = strtolower((string) $row['name']);
                foreach ($forbidden as $token) {
                    if (str_contains($column, $token)) {
                        throw new DomainException('ASSERTION_ROW_IS_EDD_LICENSE');
                    }
                }
            }
        }
        return ['edd_free' => true, 'assertion_tables' => count($tables)];
    }

    // ── private helpers ──────────────────────────────────────────────────

    private function loadPosture(string $postureUuid): ?array
    {
        $table = $this->postures->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE posture_uuid = :posture LIMIT 1");
        $statement->execute([':posture' => $postureUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function validateAssertedFamilies(array $posture, mixed $value): array
    {
        if (!is_array($value) || $value === []) {
            throw new InvalidArgumentException('explicit family allowlist required');
        }
        $postureAllowlist = json_decode((string) $posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
        $result = [];
        foreach ($value as $family) {
            if (!is_string($family) || preg_match('/^[a-z][a-z0-9_]{1,63}$/D', $family) !== 1) {
                throw new InvalidArgumentException('registered family code required');
            }
            if (!in_array($family, $postureAllowlist, true)) {
                throw new DomainException('CAPABILITY_FAMILY_NOT_INCLUDED');
            }
            $result[] = $family;
        }
        sort($result, SORT_STRING);
        return $result;
    }

    /** Keep the posture sequence/refresh window in lockstep with the newest assertion. */
    private function advancePostureInTransaction(array $posture, int $sequence, string $refreshAt): void
    {
        if ((int) $posture['sequence'] >= $sequence) {
            return;
        }
        $table = $this->postures->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("UPDATE {$table}
            SET sequence = :sequence, refresh_at = :refresh, updated_at = :updated
            WHERE posture_uuid = :posture AND sequence < :sequence_guard");
        $statement->execute([
            ':sequence' => $sequence,
            ':refresh' => $refreshAt,
            ':updated' => $refreshAt,
            ':posture' => (string) $posture['posture_uuid'],
            ':sequence_guard' => $sequence,
        ]);
    }

    private function revokePostureInTransaction(string $postureUuid, string $reason, string $occurredAt, string $encodedProvenance): void
    {
        $table = $this->postures->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("UPDATE {$table}
            SET status = 'revoked', status_reason = :reason, updated_at = :occurred
            WHERE posture_uuid = :posture AND status <> 'revoked'");
        $statement->execute([':reason' => $reason, ':occurred' => $occurredAt, ':posture' => $postureUuid]);
        $this->recordEvent('assertion_revoked', $occurredAt, $encodedProvenance);
    }

    private function recordEvent(string $eventType, string $occurredAt, string $encodedProvenance): void
    {
        $events = $this->schema->table('wpuiai_signed_access_assertion_schema_events');
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

    private function replayIdempotency(string $key, string $operation, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_signed_access_assertion_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $key]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($operation, (string) $row['operation']) || !hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $key, string $operation, string $digest, string $postureUuid, string $assertionUuid, string $createdAt): void
    {
        $table = $this->schema->table('wpuiai_signed_access_assertion_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, posture_uuid, assertion_uuid, created_at)
            VALUES (:key, :operation, :digest, :posture, :assertion, :created)");
        $statement->execute([
            ':key' => $key,
            ':operation' => $operation,
            ':digest' => $digest,
            ':posture' => $postureUuid,
            ':assertion' => $assertionUuid,
            ':created' => $createdAt,
        ]);
    }

    private function assertIdempotencyKey(string $key): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
        return $key;
    }

    private function digest(array $value): string
    {
        return hash('sha256', self::encodeCanonical($value));
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

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172SignedAccessAssertionMigration::assertTimestamp($now);
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
