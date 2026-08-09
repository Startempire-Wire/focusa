<?php
// Candidate-owned canonical authority-node schema/repository seam (spec 152E §7.4).
// Registers/deactivates nodes only for a verified authority account and a usable
// canonical EDD license, reserving the server-owned product node limit atomically
// and idempotently. It does not bootstrap WordPress.
declare(strict_types=1);

final class FocusaSpec152eAuthorityNodeMigration
{
    public const SCHEMA = 'focusa.spec152e.authority_node.v1';
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
        $nodes = $this->table('wpuiai_authority_nodes');
        $limits = $this->table('wpuiai_authority_node_limits');
        $reservations = $this->table('wpuiai_authority_node_reservations');
        $idempotency = $this->table('wpuiai_authority_node_idempotency');
        $migrations = $this->table('wpuiai_authority_node_schema_migrations');
        $events = $this->table('wpuiai_authority_node_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$nodes} (
            node_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            device_public_key TEXT NOT NULL,
            assurance_class VARCHAR(32) NOT NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('active', 'deactivated', 'revoked')),
            status_reason VARCHAR(191) NOT NULL,
            activated_at VARCHAR(32) NOT NULL,
            last_seen_at VARCHAR(32) NULL,
            deactivated_at VARCHAR(32) NULL,
            reservation_id {$key} NOT NULL,
            settlement_id {$key} NULL,
            migration_provenance TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$limits} (
            account_uuid {$uuid} NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            node_limit BIGINT NOT NULL CHECK (node_limit >= 0),
            reserved_count BIGINT NOT NULL DEFAULT 0 CHECK (reserved_count >= 0 AND reserved_count <= node_limit),
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            PRIMARY KEY (account_uuid, product_code)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$reservations} (
            reservation_id {$key} NOT NULL PRIMARY KEY,
            node_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NOT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            device_public_key TEXT NOT NULL,
            assurance_class VARCHAR(32) NOT NULL,
            node_limit BIGINT NOT NULL,
            state VARCHAR(16) NOT NULL CHECK (state IN ('reserved', 'settled', 'released')),
            release_reason VARCHAR(191) NULL,
            idempotency_key {$key} NOT NULL UNIQUE,
            request_digest VARCHAR(64) NOT NULL,
            reserved_at VARCHAR(32) NOT NULL,
            settled_at VARCHAR(32) NULL,
            released_at VARCHAR(32) NULL,
            settlement_id {$key} NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(32) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            node_uuid {$uuid} NOT NULL,
            result_state VARCHAR(16) NOT NULL,
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
     * Rollback is intentionally data-preserving: nodes, reservations, counters,
     * idempotency journals, provenance, and timestamps are never undone.
     */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeProvenance($provenance);
        $events = $this->table('wpuiai_authority_node_schema_events');
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

final class FocusaSpec152eAuthorityNodeRepository
{
    /**
     * Server-owned product codes from the frozen spec 152E/172 registry. Callers never
     * submit EDD IDs, prices, tiers, limits, or features; the node limit is read from
     * the canonical EDD license row (activation_limit), which the issuance adapter set
     * from the server-owned offer.
     */
    public const SERVER_OWNED_PRODUCTS = [
        'focusa_operator_lifetime_v1',
        'uiai_operator_lifetime_v1',
        'focusa_uiai_operator_bundle_lifetime_v1',
    ];

    public const DEVICE_KEY_PATTERN = '/^[A-Za-z0-9_-]{43}$/D';
    public const ASSURANCE_CLASSES = ['device_key_v1'];
    private const VERIFIED_ACCOUNT_REASONS = ['mailbox_verified', 'account_promoted'];

    private PDO $db;
    private FocusaSpec152eAuthorityNodeMigration $schema;
    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(PDO $db, FocusaSpec152eAuthorityNodeMigration $schema, callable $clock)
    {
        $this->db = $db;
        $this->schema = $schema;
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Register one node for a verified account and a usable canonical EDD license.
     * Reservation and settlement happen in one transaction: the atomic counter
     * increment can never exceed the license node limit, and any failure rolls the
     * whole attempt back (reservation released exactly — no dangling reservation,
     * no counter drift, no partial node). A retry with the same idempotency key
     * re-attempts cleanly; a replay of a successful attempt returns the same node.
     */
    public function registerNode(array $attempt): array
    {
        $nodeUuid = (string) ($attempt['node_uuid'] ?? '');
        $this->assertUuid($nodeUuid, 'node');
        $accountUuid = (string) ($attempt['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $idempotencyKey = (string) ($attempt['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest([
            'operation' => 'register_node',
            'node_uuid' => $nodeUuid,
            'account_uuid' => $accountUuid,
            'edd_license_id' => $attempt['edd_license_id'] ?? null,
            'product_code' => $attempt['product_code'] ?? null,
            'device_public_key' => $attempt['device_public_key'] ?? null,
            'assurance_class' => $attempt['assurance_class'] ?? null,
        ]);

        return $this->transaction(function () use ($attempt, $nodeUuid, $accountUuid, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'register_node', $digest);
            if ($replay !== null) {
                return $this->findNodeByUuid((string) $replay['node_uuid']);
            }
            $reservation = $this->reserveInner($attempt);
            $settlementId = self::opaqueToken('ns_');
            $node = $this->settleInner((string) $reservation['reservation_id'], $settlementId, (string) $reservation['node_uuid']);
            $this->recordIdempotency($idempotencyKey, 'register_node', $digest, $nodeUuid, 'active', $node['updated_at']);
            return $node;
        });
    }

    /**
     * Reserve a node slot atomically (two-phase flow). Validates the account, the
     * usable EDD license, the server-owned product, and the device key before the
     * atomic counter increment; returns the committed reservation row.
     */
    public function reserve(array $attempt): array
    {
        $nodeUuid = (string) ($attempt['node_uuid'] ?? '');
        $this->assertUuid($nodeUuid, 'node');
        $accountUuid = (string) ($attempt['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $licenseId = filter_var($attempt['edd_license_id'] ?? null, FILTER_VALIDATE_INT);
        if ($licenseId === false || $licenseId < 1) {
            throw new InvalidArgumentException('positive EDD license ID required');
        }
        $productCode = (string) ($attempt['product_code'] ?? '');
        if (!in_array($productCode, self::SERVER_OWNED_PRODUCTS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $devicePublicKey = (string) ($attempt['device_public_key'] ?? '');
        if (preg_match(self::DEVICE_KEY_PATTERN, $devicePublicKey) !== 1) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }
        $assuranceClass = (string) ($attempt['assurance_class'] ?? '');
        if (!in_array($assuranceClass, self::ASSURANCE_CLASSES, true)) {
            throw new InvalidArgumentException('bounded device assurance class required');
        }
        $idempotencyKey = (string) ($attempt['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $provenance = FocusaSpec152eAuthorityNodeMigration::encodeProvenance($attempt['migration_provenance'] ?? []);
        $digest = $this->digest([
            'operation' => 'reserve_node',
            'node_uuid' => $nodeUuid,
            'account_uuid' => $accountUuid,
            'edd_license_id' => $licenseId,
            'product_code' => $productCode,
            'device_public_key' => $devicePublicKey,
            'assurance_class' => $assuranceClass,
        ]);

        return $this->transaction(function () use (
            $nodeUuid, $accountUuid, $licenseId, $productCode, $devicePublicKey,
            $assuranceClass, $idempotencyKey, $provenance, $digest
        ): array {
            $replay = $this->replay($idempotencyKey, 'reserve_node', $digest);
            if ($replay !== null) {
                return $this->findReservationByNode($nodeUuid);
            }
            $reservation = $this->reserveInner([
                'node_uuid' => $nodeUuid,
                'account_uuid' => $accountUuid,
                'edd_license_id' => $licenseId,
                'product_code' => $productCode,
                'device_public_key' => $devicePublicKey,
                'assurance_class' => $assuranceClass,
                'reservation_idempotency_key' => $idempotencyKey,
                'migration_provenance' => json_decode($provenance, true, 512, JSON_THROW_ON_ERROR),
            ]);
            $this->recordIdempotency($idempotencyKey, 'reserve_node', $digest, $nodeUuid, 'reserved', (string) $reservation['reserved_at']);
            return $this->findReservation((string) $reservation['reservation_id']);
        });
    }

    /**
     * Shared validation + atomic counter increment + reservation row creation.
     * Must run inside the caller's transaction (never starts its own).
     */
    private function reserveInner(array $attempt): array
    {
        $nodeUuid = (string) ($attempt['node_uuid'] ?? '');
        $this->assertUuid($nodeUuid, 'node');
        $accountUuid = (string) ($attempt['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $licenseId = filter_var($attempt['edd_license_id'] ?? null, FILTER_VALIDATE_INT);
        if ($licenseId === false || $licenseId < 1) {
            throw new InvalidArgumentException('positive EDD license ID required');
        }
        $productCode = (string) ($attempt['product_code'] ?? '');
        if (!in_array($productCode, self::SERVER_OWNED_PRODUCTS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $devicePublicKey = (string) ($attempt['device_public_key'] ?? '');
        if (preg_match(self::DEVICE_KEY_PATTERN, $devicePublicKey) !== 1) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }
        $assuranceClass = (string) ($attempt['assurance_class'] ?? '');
        if (!in_array($assuranceClass, self::ASSURANCE_CLASSES, true)) {
            throw new InvalidArgumentException('bounded device assurance class required');
        }
        $provenance = FocusaSpec152eAuthorityNodeMigration::encodeProvenance($attempt['migration_provenance'] ?? []);
        $now = ($this->clock)();
        FocusaSpec152eAuthorityNodeMigration::assertTimestamp($now);
        $account = $this->verifiedAccount($accountUuid);
        $license = $this->usableLicense((int) $licenseId, (int) $account['edd_customer_id']);
        $nodeLimit = (int) $license['activation_limit'];
        if ($nodeLimit < 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $this->assertDeviceFree($accountUuid, $devicePublicKey);
        $this->reserveCounter($accountUuid, $productCode, $nodeLimit, $now);
        $reservationId = self::opaqueToken('nr_');
        $reservationKey = (string) ($attempt['reservation_idempotency_key'] ?? self::opaqueToken('idem-reserve-'));
        $reservationDigest = $this->digest([
            'reserve_node' => $nodeUuid,
            'account_uuid' => $accountUuid,
            'edd_license_id' => $licenseId,
            'product_code' => $productCode,
            'device_public_key' => $devicePublicKey,
            'assurance_class' => $assuranceClass,
        ]);
        $table = $this->schema->table('wpuiai_authority_node_reservations');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (reservation_id, node_uuid, account_uuid, edd_license_id, product_code,
             device_public_key, assurance_class, node_limit,
             state, idempotency_key, request_digest, reserved_at, migration_provenance)
            VALUES (:reservation, :node, :account, :license, :product,
                    :device, :assurance, :limit,
                    'reserved', :idem, :digest, :reserved_at, :provenance)");
        $statement->execute([
            ':reservation' => $reservationId,
            ':node' => $nodeUuid,
            ':account' => $accountUuid,
            ':license' => $licenseId,
            ':product' => $productCode,
            ':device' => $devicePublicKey,
            ':assurance' => $assuranceClass,
            ':limit' => $nodeLimit,
            ':idem' => $reservationKey,
            ':digest' => $reservationDigest,
            ':reserved_at' => $now,
            ':provenance' => $provenance,
        ]);
        return $this->findReservation($reservationId);
    }

    /**
     * Settle a committed reservation: bind the node row (account/license/product/
     * public-key references, active status, activation timestamp) and mark the
     * reservation settled. The counter stays incremented — the slot is live.
     */
    public function settleReservation(string $reservationId, string $settlementId, string $idempotencyKey): array
    {
        $this->assertBoundedToken($reservationId, 'reservation id');
        $this->assertBoundedToken($settlementId, 'settlement id');
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest([
            'operation' => 'settle_node',
            'reservation_id' => $reservationId,
            'settlement_id' => $settlementId,
        ]);

        return $this->transaction(function () use ($reservationId, $settlementId, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'settle_node', $digest);
            if ($replay !== null) {
                return $this->findNodeByUuid((string) $replay['node_uuid']);
            }
            $node = $this->settleInner($reservationId, $settlementId, '');
            $this->recordIdempotency($idempotencyKey, 'settle_node', $digest, (string) $node['node_uuid'], 'active', (string) $node['updated_at']);
            return $node;
        });
    }

    /**
     * Shared settlement: bind the node row (account/license/product/public-key
     * references, active status, activation timestamp) and mark the reservation
     * settled. The counter stays incremented — the slot is live. Must run inside
     * the caller's transaction (never starts its own).
     */
    private function settleInner(string $reservationId, string $settlementId, string $expectedNodeUuid): array
    {
        $this->assertBoundedToken($reservationId, 'reservation id');
        $this->assertBoundedToken($settlementId, 'settlement id');
        $reservation = $this->findReservation($reservationId);
        if ($reservation['state'] !== 'reserved') {
            throw new DomainException('RESERVATION_NOT_PENDING');
        }
        if ($expectedNodeUuid !== '' && !hash_equals($expectedNodeUuid, (string) $reservation['node_uuid'])) {
            throw new DomainException('RESERVATION_NOT_PENDING');
        }
        $now = ($this->clock)();
        FocusaSpec152eAuthorityNodeMigration::assertTimestamp($now);
        $nodes = $this->schema->table('wpuiai_authority_nodes');
        $statement = $this->db->prepare("INSERT INTO {$nodes}
            (node_uuid, account_uuid, edd_license_id, product_code, device_public_key,
             assurance_class, status, status_reason, activated_at, reservation_id,
             settlement_id, migration_provenance, created_at, updated_at)
            VALUES (:node, :account, :license, :product, :device, :assurance,
                    'active', 'device_registered', :activated, :reservation, :settlement,
                    :provenance, :created, :updated)");
        $statement->execute([
            ':node' => $reservation['node_uuid'],
            ':account' => $reservation['account_uuid'],
            ':license' => $reservation['edd_license_id'],
            ':product' => $reservation['product_code'],
            ':device' => $reservation['device_public_key'],
            ':assurance' => $reservation['assurance_class'],
            ':activated' => $now,
            ':reservation' => $reservationId,
            ':settlement' => $settlementId,
            ':provenance' => $reservation['migration_provenance'],
            ':created' => $now,
            ':updated' => $now,
        ]);
        $reservations = $this->schema->table('wpuiai_authority_node_reservations');
        $update = $this->db->prepare("UPDATE {$reservations}
            SET state = 'settled', settled_at = :settled_at, settlement_id = :settlement
            WHERE reservation_id = :reservation AND state = 'reserved'");
        $update->execute([
            ':settled_at' => $now,
            ':settlement' => $settlementId,
            ':reservation' => $reservationId,
        ]);
        if ($update->rowCount() !== 1) {
            throw new RuntimeException('reservation settlement lost its lock');
        }
        return $this->findNodeByUuid((string) $reservation['node_uuid']);
    }

    /**
     * Release a committed reservation exactly once: the counter is decremented
     * (slot freed) and the reservation is terminal in state released.
     */
    public function releaseReservation(string $reservationId, string $reason, string $idempotencyKey): array
    {
        $this->assertBoundedToken($reservationId, 'reservation id');
        $this->assertReleaseReason($reason);
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest([
            'operation' => 'release_reservation',
            'reservation_id' => $reservationId,
            'reason' => $reason,
        ]);

        return $this->transaction(function () use ($reservationId, $reason, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'release_reservation', $digest);
            if ($replay !== null) {
                return $this->findReservation($reservationId);
            }
            $reservation = $this->findReservation($reservationId);
            if ($reservation['state'] !== 'reserved') {
                throw new DomainException('RESERVATION_NOT_PENDING');
            }
            $now = ($this->clock)();
            FocusaSpec152eAuthorityNodeMigration::assertTimestamp($now);
            $this->releaseCounter(
                (string) $reservation['account_uuid'],
                (string) $reservation['product_code'],
                $now,
            );
            $table = $this->schema->table('wpuiai_authority_node_reservations');
            $update = $this->db->prepare("UPDATE {$table}
                SET state = 'released', released_at = :released_at, release_reason = :reason
                WHERE reservation_id = :reservation AND state = 'reserved'");
            $update->execute([
                ':released_at' => $now,
                ':reason' => $reason,
                ':reservation' => $reservationId,
            ]);
            if ($update->rowCount() !== 1) {
                throw new RuntimeException('reservation release lost its lock');
            }
            $this->recordIdempotency($idempotencyKey, 'release_reservation', $digest, (string) $reservation['node_uuid'], 'released', $now);
            return $this->findReservation($reservationId);
        });
    }

    /**
     * Explicit node management: deactivate an active node, preserving its history.
     * The node row is never deleted; its reservation is released (slot freed) and
     * the device may re-register under a new node afterwards.
     */
    public function deactivateNode(array $attempt): array
    {
        $nodeUuid = (string) ($attempt['node_uuid'] ?? '');
        $this->assertUuid($nodeUuid, 'node');
        $accountUuid = (string) ($attempt['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $reason = (string) ($attempt['status_reason'] ?? '');
        $this->assertReleaseReason($reason);
        $idempotencyKey = (string) ($attempt['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest([
            'operation' => 'deactivate_node',
            'node_uuid' => $nodeUuid,
            'account_uuid' => $accountUuid,
            'status_reason' => $reason,
        ]);

        return $this->transaction(function () use ($nodeUuid, $accountUuid, $reason, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'deactivate_node', $digest);
            if ($replay !== null) {
                return $this->findNodeByUuid($nodeUuid);
            }
            $node = $this->findNodeByUuid($nodeUuid);
            if (!hash_equals((string) $node['account_uuid'], $accountUuid)) {
                throw new DomainException('NODE_NOT_FOUND');
            }
            if ($node['status'] !== 'active') {
                throw new DomainException('NODE_NOT_ACTIVE');
            }
            $now = ($this->clock)();
            FocusaSpec152eAuthorityNodeMigration::assertTimestamp($now);
            $nodes = $this->schema->table('wpuiai_authority_nodes');
            $update = $this->db->prepare("UPDATE {$nodes}
                SET status = 'deactivated', status_reason = :reason, deactivated_at = :deactivated, updated_at = :updated
                WHERE node_uuid = :node AND status = 'active'");
            $update->execute([
                ':reason' => $reason,
                ':deactivated' => $now,
                ':updated' => $now,
                ':node' => $nodeUuid,
            ]);
            if ($update->rowCount() !== 1) {
                throw new RuntimeException('node deactivation lost its lock');
            }
            $this->releaseCounter((string) $node['account_uuid'], (string) $node['product_code'], $now);
            $reservations = $this->schema->table('wpuiai_authority_node_reservations');
            $this->db->prepare("UPDATE {$reservations}
                SET state = 'released', released_at = :released_at, release_reason = 'deactivated'
                WHERE reservation_id = :reservation AND state = 'settled'")
                ->execute([
                    ':released_at' => $now,
                    ':reservation' => (string) $node['reservation_id'],
                ]);
            $this->recordIdempotency($idempotencyKey, 'deactivate_node', $digest, $nodeUuid, 'deactivated', $now);
            return $this->findNodeByUuid($nodeUuid);
        });
    }

    /** Record a last-seen heartbeat for an active node (idempotent). */
    public function recordLastSeen(string $nodeUuid, string $idempotencyKey): array
    {
        $this->assertUuid($nodeUuid, 'node');
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->digest(['operation' => 'touch_last_seen', 'node_uuid' => $nodeUuid]);

        return $this->transaction(function () use ($nodeUuid, $idempotencyKey, $digest): array {
            $replay = $this->replay($idempotencyKey, 'touch_last_seen', $digest);
            if ($replay !== null) {
                return $this->findNodeByUuid($nodeUuid);
            }
            $node = $this->findNodeByUuid($nodeUuid);
            if ($node['status'] !== 'active') {
                throw new DomainException('NODE_NOT_ACTIVE');
            }
            $now = ($this->clock)();
            FocusaSpec152eAuthorityNodeMigration::assertTimestamp($now);
            $nodes = $this->schema->table('wpuiai_authority_nodes');
            $this->db->prepare("UPDATE {$nodes} SET last_seen_at = :last_seen, updated_at = :updated WHERE node_uuid = :node")
                ->execute([':last_seen' => $now, ':updated' => $now, ':node' => $nodeUuid]);
            $this->recordIdempotency($idempotencyKey, 'touch_last_seen', $digest, $nodeUuid, 'active', $now);
            return $this->findNodeByUuid($nodeUuid);
        });
    }

    /** Device history for an account: active, deactivated, and revoked nodes. */
    public function listNodes(string $accountUuid): array
    {
        $this->assertUuid($accountUuid, 'account');
        $table = $this->schema->table('wpuiai_authority_nodes');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE account_uuid = :account ORDER BY activated_at");
        $statement->execute([':account' => $accountUuid]);
        return $statement->fetchAll(PDO::FETCH_ASSOC);
    }

    public function findNodeByUuid(string $nodeUuid): array
    {
        $this->assertUuid($nodeUuid, 'node');
        $table = $this->schema->table('wpuiai_authority_nodes');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE node_uuid = :node");
        $statement->execute([':node' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('authority node not found');
        }
        return $row;
    }

    /** Server-owned limit ledger: the atomic reservation counter for one account/product. */
    public function limitLedger(string $accountUuid, string $productCode): ?array
    {
        $table = $this->schema->table('wpuiai_authority_node_limits');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE account_uuid = :account AND product_code = :product");
        $statement->execute([':account' => $accountUuid, ':product' => $productCode]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function verifiedAccount(string $accountUuid): array
    {
        $table = $this->schema->table('wpuiai_authority_accounts');
        try {
            $statement = $this->db->prepare("SELECT account_uuid, edd_customer_id, status, status_reason FROM {$table} WHERE account_uuid = :uuid");
            $statement->execute([':uuid' => $accountUuid]);
            $row = $statement->fetch(PDO::FETCH_ASSOC);
        } catch (PDOException) {
            $row = false;
        }
        if ($row === false) {
            throw new DomainException('ACCOUNT_NOT_FOUND');
        }
        if ((string) $row['status'] !== 'active'
            || !in_array((string) $row['status_reason'], self::VERIFIED_ACCOUNT_REASONS, true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        return $row;
    }

    private function usableLicense(int $licenseId, int $accountCustomerId): array
    {
        $table = $this->schema->table('edd_licenses');
        try {
            $statement = $this->db->prepare("SELECT id, customer_id, activation_limit, status, expiration FROM {$table} WHERE id = :id");
            $statement->execute([':id' => $licenseId]);
            $row = $statement->fetch(PDO::FETCH_ASSOC);
        } catch (PDOException) {
            $row = false;
        }
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        if ((string) $row['status'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        if ((int) $row['customer_id'] !== $accountCustomerId) {
            throw new DomainException('LICENSE_ACCOUNT_MISMATCH');
        }
        $expiration = $row['expiration'] ?? null;
        if (is_string($expiration) && $expiration !== '') {
            $expiresAt = strtotime($expiration);
            if ($expiresAt === false || $expiresAt <= time()) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }
        }
        return $row;
    }

    private function assertDeviceFree(string $accountUuid, string $devicePublicKey): void
    {
        $table = $this->schema->table('wpuiai_authority_nodes');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}
            WHERE account_uuid = :account AND device_public_key = :device AND status = 'active'");
        $statement->execute([':account' => $accountUuid, ':device' => $devicePublicKey]);
        if ((int) $statement->fetchColumn() !== 0) {
            throw new DomainException('DEVICE_PUBLIC_KEY_IN_USE');
        }
    }

    /** Atomic increment: the write lock makes concurrent activations serialize. */
    private function reserveCounter(string $accountUuid, string $productCode, int $nodeLimit, string $now): void
    {
        $table = $this->schema->table('wpuiai_authority_node_limits');
        $this->db->prepare("INSERT INTO {$table}
            (account_uuid, product_code, node_limit, reserved_count, created_at, updated_at)
            SELECT :account, :product, :limit, 0, :created, :updated
            WHERE NOT EXISTS (SELECT 1 FROM {$table}
                WHERE account_uuid = :existing_account AND product_code = :existing_product)")
            ->execute([
                ':account' => $accountUuid,
                ':product' => $productCode,
                ':limit' => $nodeLimit,
                ':created' => $now,
                ':updated' => $now,
                ':existing_account' => $accountUuid,
                ':existing_product' => $productCode,
            ]);
        $update = $this->db->prepare("UPDATE {$table}
            SET reserved_count = reserved_count + 1, updated_at = :updated
            WHERE account_uuid = :account AND product_code = :product
              AND node_limit = :limit AND reserved_count < node_limit");
        $update->execute([
            ':updated' => $now,
            ':account' => $accountUuid,
            ':product' => $productCode,
            ':limit' => $nodeLimit,
        ]);
        if ($update->rowCount() !== 1) {
            throw new DomainException('NODE_LIMIT_EXHAUSTED');
        }
    }

    private function releaseCounter(string $accountUuid, string $productCode, string $now): void
    {
        $table = $this->schema->table('wpuiai_authority_node_limits');
        $update = $this->db->prepare("UPDATE {$table}
            SET reserved_count = reserved_count - 1, updated_at = :updated
            WHERE account_uuid = :account AND product_code = :product AND reserved_count > 0");
        $update->execute([
            ':updated' => $now,
            ':account' => $accountUuid,
            ':product' => $productCode,
        ]);
        if ($update->rowCount() !== 1) {
            throw new RuntimeException('counter release lost its lock');
        }
    }

    private function findReservation(string $reservationId): array
    {
        $table = $this->schema->table('wpuiai_authority_node_reservations');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE reservation_id = :id");
        $statement->execute([':id' => $reservationId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('node reservation not found');
        }
        return $row;
    }

    private function findReservationByNode(string $nodeUuid): array
    {
        $table = $this->schema->table('wpuiai_authority_node_reservations');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE node_uuid = :node LIMIT 1");
        $statement->execute([':node' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('node reservation not found');
        }
        return $row;
    }

    private function replay(string $key, string $operation, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_authority_node_idempotency');
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

    private function recordIdempotency(string $key, string $operation, string $digest, string $nodeUuid, string $resultState, string $createdAt): void
    {
        $table = $this->schema->table('wpuiai_authority_node_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, node_uuid, result_state, created_at)
            VALUES (:key, :operation, :digest, :node, :state, :created)");
        $statement->execute([
            ':key' => $key, ':operation' => $operation, ':digest' => $digest,
            ':node' => $nodeUuid, ':state' => $resultState, ':created' => $createdAt,
        ]);
    }

    private function digest(array $value): string
    {
        return hash('sha256', FocusaSpec152eAuthorityNodeMigration::encodeProvenance($value));
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

    private static function assertUuid(string $value, string $label): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $value) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$label} UUID required");
        }
    }

    private function assertIdempotencyKey(string $key): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
    }

    private function assertReleaseReason(string $reason): void
    {
        if ($reason === '' || strlen($reason) > 191 || preg_match('/[\r\n@\x00]/', $reason) === 1) {
            throw new InvalidArgumentException('bounded status reason required');
        }
    }

    private function assertBoundedToken(string $value, string $label): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $value) !== 1) {
            throw new InvalidArgumentException("bounded {$label} required");
        }
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }
}
