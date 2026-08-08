<?php
// Spec 172 Focusa Operator Lifetime v1 issuance projector (addendum sections 7.1, 7.3,
// 8, 11, 16.1, 16.3, 17, and 21; atom focusa-vbcqu.20.15.13). Consumes exactly one
// issued canonical EDD Software Licensing key for one verified complete eligible Focusa
// order and projects `focusa_operator_lifetime_v1` from canonical EDD truth:
//
//   - Projection starts only from an issued issuance request: the SL issuance journal
//     (wp_wpuiai_edd_license_issuances) row for the same issuance-request handle must
//     exist with state 'issued', and the canonical EDD license row must still be active
//     and carry the exact journaled key digest. No key, no projection.
//   - Canonical EDD order truth is re-verified at projection time: the order row is
//     complete, belongs to the exact customer, and carries the exact verified-email
//     digest; refunded/revoked/pending/unverified orders and account/email mismatches
//     fail closed and never project.
//   - The server-owned dedicated Downloads contract is the only offer authority: the
//     projection resolves the offer by the exact download binding and requires the
//     canonical public code, License Type ref, price id, and amount to match the settled
//     item. Caller metadata never selects product, price, License Type, family, feature,
//     limit, node, seat, or commercial right (CLIENT_COMMERCIAL_FIELDS_FORBIDDEN).
//   - Only `focusa_operator_lifetime_v1` (product `focusa`) projects here. Any other
//     canonical License Type (for example uiai) fails closed with
//     LICENSE_TYPE_NOT_INCLUDED; offers that are not checkout-enabled fail closed with
//     EDD_CHECKOUT_REQUIRED; unknown or drifted mappings fail closed with
//     PRODUCT_MAPPING_REQUIRED.
//   - One eligible item produces exactly one projection, forever. A replay with the same
//     idempotency key returns the identical decision; a duplicate projection call for the
//     same issued request returns the same projection with existing=true and
//     projections_created=0. No second projection is ever created for the same issuance
//     request (UNIQUE on issuance_request_key).
//   - The projection freezes the family set, seat/node limits, price version, and a
//     strictly monotonic per-account sequence. The family digest is a SHA-256 over the
//     canonical frozen family record (Spec 172 section 7.1 families) and the price
//     version is derived only from the server-owned offer price and contract version.
//     The sequence advances on the authority account in the same transaction as the
//     journal row (guard update; ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED on regression).
//   - No plaintext leakage: journals and decisions carry only the 64-hex license-key
//     digest plus a masked key; no raw email, raw payment transaction id, secret, key,
//     credential, customer row, or card data is stored or returned.
//
// Failures are public-safe stable codes (EDD_ORDER_PENDING, REFUNDED, REVOKED,
// EDD_ORDER_UNVERIFIED, ACCOUNT_EMAIL_MISMATCH, EMAIL_VERIFICATION_REQUIRED,
// ACCOUNT_MERGE_REVIEW_REQUIRED, FACADE_ORIGIN_DENIED, PRODUCT_MAPPING_REQUIRED,
// PRODUCT_NOT_INCLUDED, LICENSE_TYPE_NOT_INCLUDED, EDD_CHECKOUT_REQUIRED,
// EDD_LICENSE_UNUSABLE, CLIENT_COMMERCIAL_FIELDS_FORBIDDEN, IDEMPOTENCY_CONFLICT,
// ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED). No new error code is introduced.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php,
// docs/contracts/spec152e-edd-order-binding.v1.php,
// docs/contracts/spec152e-edd-license-issuance.v1.php, and the server-owned dedicated
// downloads contract (docs/contracts/spec172-edd-operator-v1-downloads.v1.php) to be
// loaded first.
declare(strict_types=1);

final class FocusaSpec172LicenseTypeProjectionMigration
{
    public const SCHEMA = 'focusa.spec172.license_type_projection.v1';
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
        $projections = $this->table('wpuiai_license_type_projections');
        $migrations = $this->table('wpuiai_license_type_projection_schema_migrations');
        $events = $this->table('wpuiai_license_type_projection_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$projections} (
            projection_key VARCHAR(64) NOT NULL PRIMARY KEY,
            issuance_request_key VARCHAR(64) NOT NULL,
            issuance_key VARCHAR(64) NOT NULL,
            binding_key VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            order_item_id BIGINT NOT NULL,
            download_id BIGINT NOT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            license_type_ref VARCHAR(128) NOT NULL,
            price_version {$key} NOT NULL,
            family_digest VARCHAR(64) NOT NULL,
            operator_seats INT NOT NULL,
            node_limit INT NOT NULL,
            node_set VARCHAR(64) NOT NULL,
            term VARCHAR(32) NOT NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('active')),
            sequence BIGINT NOT NULL CHECK (sequence >= 1),
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (issuance_request_key)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_license_type_projection_idempotency
            ON {$projections} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_license_type_projection_retention
            ON {$projections} (retention_until)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_license_type_projection_customer
            ON {$projections} (customer_id, download_id, status)");
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

    /** Rollback is preservation-only: projection journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_license_type_projection_schema_events');
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

final class FocusaSpec172FocusaOperatorProjector
{
    public const SCHEMA = 'focusa.spec172.license_type_projection.v1';
    public const RESULT_SCHEMA = 'focusa.spec172.focusa_operator_lifetime_projection.v1';
    public const VERSION = 1;
    public const LICENSE_TYPE = 'focusa_operator_lifetime_v1';
    public const PRODUCT = 'focusa';
    public const TERM = 'lifetime';
    public const RETENTION_SECONDS = 2592000;
    public const KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';

    /**
     * Frozen Focusa Operator v1 family set (Spec 172 section 7.1): normal Focusa core,
     * automation, same-operator remote/device operation, release proof, and premium
     * update workflows, matching the Spec 152F registered family ids. New families never
     * enter silently: the digest is frozen over this exact ordered list.
     */
    public const FROZEN_FAMILIES = [
        'base_focusa',
        'automation',
        'team_remote',
        'release_proof',
        'premium_updates',
    ];

    /** Canonical authority source of the frozen family record. */
    public const FAMILY_AUTHORITY = 'docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'currency', 'tier', 'products', 'product_code',
        'license_type', 'license_type_ref', 'capability_family', 'families', 'features',
        'grants', 'limits', 'node_limit', 'activation_limit', 'operator_seats',
        'node_set', 'sale_status', 'refund_policy', 'upgrade_policy', 'commercial_rights',
        'evaluation_duration', 'edd_download_id', 'edd_price_id', 'license_key',
        'license_duration', 'expiration', 'price_version', 'family_digest',
    ];

    private const REQUEST_STATE_ISSUED = 'issued';
    private const BINDING_STATE_SETTLED = 'settled_pending_issuance';
    private const BINDING_STATE_BLOCKED = 'blocked';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172LicenseTypeProjectionMigration $schema,
        private FocusaSpec152eEddLicenseIssuanceMigration $issuanceSchema,
        private FocusaSpec152eEddOrderBindingMigration $bindingSchema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private array $dedicatedDownloads,
        callable $clock,
        private string $prefix = 'wp_',
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        if ($this->retention < 1) {
            throw new InvalidArgumentException('positive retention required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Project `focusa_operator_lifetime_v1` for exactly one issued canonical EDD license.
     * Required input:
     *   - issuance_request_handle: the opaque ir_ handle journaled by the order-binding
     *     service and issued by the canonical SL issuance service
     *   - request_id, idempotency_key
     * Caller metadata never selects any product, price, License Type, family, feature,
     * limit, node, seat, or commercial right. Returns a public-safe projection decision;
     * replays return the identical decision; duplicate projection calls for the same
     * issued request return the same projection with existing=true and zero creations.
     */
    public function project(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $issuanceRequestKey = (string) ($input['issuance_request_handle'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        if (preg_match('/^(ir_)[0-9a-f]{32}$/D', $issuanceRequestKey) !== 1) {
            throw new InvalidArgumentException('bounded issuance request handle required');
        }
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $digest = $this->requestDigest([
            'operation' => 'focusa_operator_lifetime_projection',
            'issuance_request_handle' => $issuanceRequestKey,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $request = $this->loadIssuanceRequest($issuanceRequestKey);
        if ($request['state'] !== self::REQUEST_STATE_ISSUED) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $issuanceRow = $this->loadIssuanceJournal($issuanceRequestKey);
        $binding = $this->loadBinding((string) $request['binding_key'], $issuanceRequestKey);
        if ($binding['binding_state'] === self::BINDING_STATE_BLOCKED) {
            throw new DomainException((string) ($binding['blocked_reason'] ?? 'EDD_LICENSE_UNUSABLE'));
        }
        if ($binding['binding_state'] !== self::BINDING_STATE_SETTLED) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }

        // Already projected: a duplicate projection call returns the same projection with
        // zero creations. Never a second projection for the same issuance request.
        $existing = $this->findByIssuanceRequestKey($issuanceRequestKey);
        if ($existing !== null) {
            return $this->existingProjectionDecision($existing);
        }

        $registration = $this->assertProjectionRegistration((string) $request['registration_uuid'], $binding, $request);
        $downloadId = (int) $binding['download_id'];
        $this->assertCanonicalOrder((int) $request['order_id'], (int) $request['customer_id'], $registration);
        $this->assertCanonicalOrderItem((int) $request['order_id'], (int) $request['order_item_id'], $downloadId);
        $licenseId = (int) $issuanceRow['edd_license_id'];
        $this->assertCanonicalLicense($licenseId, (string) $issuanceRow['license_key_digest']);
        $offer = $this->assertOfferMapping($request, $binding, $downloadId);

        return $this->recordProjection(
            $issuanceRow, $request, $binding, $registration, $offer, $licenseId,
            $requestId, $idempotencyKey, $digest,
        );
    }

    /** Bounded journal lookups for settlement/reconciliation. */
    public function projectionCount(): int
    {
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}");
        $statement->execute();
        return (int) $statement->fetchColumn();
    }

    /** Bounded: exact projection lookup by opaque projection handle (pr_). */
    public function findByProjectionKey(string $projectionKey): ?array
    {
        $this->assertToken($projectionKey, 64, 'projection');
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE projection_key = :key LIMIT 1");
        $statement->execute([':key' => $projectionKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Bounded: exact projection lookup by the source issuance-request handle (ir_). */
    public function findByIssuanceRequestKey(string $issuanceRequestKey): ?array
    {
        $this->assertToken($issuanceRequestKey, 64, 'issuance request');
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function recordProjection(
        array $issuanceRow,
        array $request,
        array $binding,
        array $registration,
        array $offer,
        int $licenseId,
        string $requestId,
        string $idempotencyKey,
        string $digest,
    ): array {
        $now = $this->now();
        $account = $this->accounts->findByUuid((string) $request['account_uuid']);
        $nextSequence = (int) $account['highest_entitlement_sequence'] + 1;
        $priceVersion = self::priceVersion($offer);
        $familyDigest = self::familyDigest();
        $projectionKey = self::opaqueToken('pr_');
        $projections = $this->schema->table('wpuiai_license_type_projections');
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';

        $decision = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'license_type_projected',
            'projection_id' => $projectionKey,
            'registration_id' => (string) $request['registration_uuid'],
            'account_id' => (string) $request['account_uuid'],
            'customer_id' => (int) $request['customer_id'],
            'order_id' => (int) $request['order_id'],
            'order_item_id' => (int) $request['order_item_id'],
            'download_id' => (int) $binding['download_id'],
            'edd_license_id' => $licenseId,
            'license_key_digest' => (string) $issuanceRow['license_key_digest'],
            'license_key_mask' => (string) $issuanceRow['license_key_mask'],
            'issuance' => 'canonical_edd_software_licensing',
            'product' => self::PRODUCT,
            'license_type' => self::LICENSE_TYPE,
            'grant' => self::LICENSE_TYPE,
            'price_version' => $priceVersion,
            'price_usd' => (string) $offer['price_usd'],
            'amount_minor' => (int) $offer['amount_minor'],
            'family_digest' => $familyDigest,
            'family_count' => count(self::FROZEN_FAMILIES),
            'operator_seats' => (int) $offer['operator_seats'],
            'node_limit' => (int) $offer['node_limit'],
            'node_set' => (string) $offer['node_set'],
            'term' => self::TERM,
            'status' => 'active',
            'sequence' => $nextSequence,
            'existing' => false,
            'projections_created' => 1,
        ];

        $this->db->beginTransaction();
        try {
            $statement = $this->db->prepare("UPDATE {$accountsTable}
                SET highest_entitlement_sequence = :next, updated_at = :updated
                WHERE account_uuid = :uuid AND highest_entitlement_sequence < :guard");
            $statement->execute([
                ':next' => $nextSequence,
                ':updated' => $now,
                ':uuid' => (string) $request['account_uuid'],
                ':guard' => $nextSequence,
            ]);
            if ($statement->rowCount() !== 1) {
                throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
            }
            $retention = self::plusSeconds($now, $this->retention);
            $projectionStatement = $this->db->prepare("INSERT INTO {$projections}
                (projection_key, issuance_request_key, issuance_key, binding_key, registration_uuid,
                 account_uuid, customer_id, order_id, order_item_id, download_id, edd_license_id,
                 product_code, license_type_ref, price_version, family_digest, operator_seats,
                 node_limit, node_set, term, status, sequence, result_payload,
                 request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
                VALUES (:projection, :request_key, :issuance, :binding, :registration,
                        :account, :customer, :order, :item, :download, :license_id,
                        :product, :license_type, :price_version, :family_digest, :seats,
                        :node_limit, :node_set, :term, :status, :sequence, :payload,
                        :request, :idempotency, :request_digest, :created, :retention, :updated)");
            $projectionStatement->execute([
                ':projection' => $projectionKey,
                ':request_key' => (string) $request['issuance_request_key'],
                ':issuance' => (string) $issuanceRow['issuance_key'],
                ':binding' => (string) $binding['binding_key'],
                ':registration' => (string) $request['registration_uuid'],
                ':account' => (string) ($request['account_uuid'] ?? ''),
                ':customer' => (int) $request['customer_id'],
                ':order' => (int) $request['order_id'],
                ':item' => (int) $request['order_item_id'],
                ':download' => (int) $binding['download_id'],
                ':license_id' => $licenseId,
                ':product' => self::PRODUCT,
                ':license_type' => self::LICENSE_TYPE,
                ':price_version' => $priceVersion,
                ':family_digest' => $familyDigest,
                ':seats' => (int) $offer['operator_seats'],
                ':node_limit' => (int) $offer['node_limit'],
                ':node_set' => (string) $offer['node_set'],
                ':term' => self::TERM,
                ':status' => 'active',
                ':sequence' => $nextSequence,
                ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':retention' => $retention,
                ':updated' => $now,
            ]);
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
        return $decision;
    }

    /** Duplicate projection call for the same issued request: same projection, zero new. */
    private function existingProjectionDecision(array $row): array
    {
        $decision = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $decision['decision'] = 'license_type_projected';
        $decision['existing'] = true;
        $decision['projections_created'] = 0;
        return $decision;
    }

    /**
     * Idempotent replay: same key returns the identical decision. The journaled payload
     * is the full original decision (existing=false, projections_created=1), so a replay
     * is byte-identical; a duplicate call with a NEW idempotency key routes through
     * existingProjectionDecision instead.
     */
    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    /** The issuance request must exist in the order-binding journal and be issued. */
    private function loadIssuanceRequest(string $issuanceRequestKey): array
    {
        $table = $this->bindingSchema->table('wpuiai_edd_issuance_requests');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    /** The canonical SL issuance journal must hold the issued license for this request. */
    private function loadIssuanceJournal(string $issuanceRequestKey): array
    {
        $table = $this->issuanceSchema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || $row['state'] !== 'issued') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    /** The binding must be settled for this exact issuance request (or journaled terminal). */
    private function loadBinding(string $bindingKey, string $issuanceRequestKey): array
    {
        $table = $this->bindingSchema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE binding_key = :key LIMIT 1");
        $statement->execute([':key' => $bindingKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || !hash_equals((string) ($row['issuance_request_key'] ?? ''), $issuanceRequestKey)) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    /**
     * Registration must still be mailbox-verified and bound to the exact account/customer/
     * facade of the settled request. Fulfillment states from the SL issuance are accepted;
     * registrations that never entered checkout or are terminal fail closed.
     */
    private function assertProjectionRegistration(string $registrationUuid, array $binding, array $request): array
    {
        try {
            $registration = $this->registrations->findByUuid($registrationUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ((string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $state = (string) $registration['state'];
        $fulfilled = ['entitlement_issued', 'terminal_delivery_ready', 'device_registered', 'lease_issued'];
        $nonTerminal = ['checkout_pending'];
        if (!in_array($state, array_merge($fulfilled, $nonTerminal), true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $accountId = (string) ($request['account_uuid'] ?? '');
        if ($accountId === ''
            || !hash_equals($accountId, (string) $registration['account_uuid'])
            || (int) $registration['edd_customer_id'] !== (int) $request['customer_id']) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }
        if (!hash_equals((string) ($binding['facade_id'] ?? ''), (string) $registration['facade_id'])) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        return $registration;
    }

    /** Canonical EDD order truth: complete status, exact customer, exact verified email digest. */
    private function assertCanonicalOrder(int $orderId, int $customerId, array $registration): void
    {
        $order = $this->edd->findOrderById($orderId);
        if ($order === null) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $status = (string) ($order['status'] ?? '');
        if (in_array($status, ['refunded', 'revoked'], true)) {
            throw new DomainException(strtoupper($status));
        }
        if (in_array($status, ['pending', 'processing'], true)) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        if ($status !== 'complete') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((int) $order['customer_id'] !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderEmail = (string) ($order['email'] ?? '');
        if ($orderEmail === '') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderDigest = $this->registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($orderEmail));
        if (!hash_equals((string) $registration['email_lookup_digest'], $orderDigest)) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }
    }

    /** Canonical order-item binding: the item row exists, belongs to this order, exact download. */
    private function assertCanonicalOrderItem(int $orderId, int $orderItemId, int $downloadId): void
    {
        $item = $this->edd->findOrderItemById($orderItemId);
        if ($item === null
            || (int) $item['order_id'] !== $orderId
            || (int) $item['product_id'] !== $downloadId
            || (int) $item['quantity'] < 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
    }

    /** Canonical EDD license truth: the license row exists, is active, and carries the exact digest. */
    private function assertCanonicalLicense(int $licenseId, string $expectedDigest): void
    {
        $license = $this->edd->findLicenseById($licenseId);
        if ($license === null || (string) $license['status'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $key = (string) $license['license_key'];
        if (preg_match(self::KEY_PATTERN, $key) !== 1
            || !hash_equals($expectedDigest, $this->keyDigest($key))) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
    }

    /**
     * Server-owned offer resolution at projection time: the dedicated Downloads contract
     * resolves by the settled download binding and must still carry the exact public code,
     * License Type ref, price id, amount, and an enabled mapping. Only the Focusa Operator
     * Lifetime v1 License Type projects here; any other canonical License Type (for
     * example uiai) fails closed with LICENSE_TYPE_NOT_INCLUDED.
     */
    private function assertOfferMapping(array $request, array $binding, int $downloadId): array
    {
        $offer = null;
        foreach (($this->dedicatedDownloads['records'] ?? []) as $candidate) {
            if ((int) ($candidate['edd_download_id'] ?? 0) === $downloadId) {
                $offer = $candidate;
                break;
            }
        }
        if ($offer === null
            || !hash_equals((string) ($offer['public_code'] ?? ''), (string) $request['product_code'])
            || !hash_equals((string) ($offer['license_type_ref'] ?? ''), (string) $request['license_type_ref'])
            || !hash_equals((string) ($offer['edd_price_id'] ?? ''), (string) ($binding['price_id'] ?? ''))) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (!hash_equals(self::LICENSE_TYPE, (string) $offer['license_type_ref'])) {
            throw new DomainException('LICENSE_TYPE_NOT_INCLUDED');
        }
        if (!in_array(self::PRODUCT, (array) ($offer['products'] ?? []), true)) {
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        if (($offer['checkout_enabled'] ?? false) !== true || ($offer['sale_status'] ?? '') !== 'enabled') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        if ((string) $offer['license_duration'] !== self::TERM) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if ((int) $offer['amount_minor'] !== self::canonicalAmountMinor((string) $offer['price_usd'])) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        return $offer;
    }

    /** Frozen family digest over the canonical family record; identical for every v1 license. */
    public static function familyDigest(): string
    {
        return hash('sha256', FocusaSpec172LicenseTypeProjectionMigration::encodeCanonical([
            'license_type' => self::LICENSE_TYPE,
            'product' => self::PRODUCT,
            'families' => self::FROZEN_FAMILIES,
            'authority' => self::FAMILY_AUTHORITY,
        ]));
    }

    /** Server-owned price version: license type, fixed price, and dedicated contract version. */
    public static function priceVersion(array $offer): string
    {
        return sprintf(
            '%s.%s.v%s',
            self::LICENSE_TYPE,
            (string) $offer['price_usd'],
            (int) (self::VERSION),
        );
    }

    /** Fixed USD price in minor units; only the canonical 697.00 price projects. */
    public static function canonicalAmountMinor(string $priceUsd): int
    {
        if ($priceUsd === '697.00') {
            return 69700;
        }
        return (int) round((float) $priceUsd * 100);
    }

    private function keyDigest(string $licenseKey): string
    {
        return hash('sha256', "focusa.spec152e.edd_license_issuance.key.v1\0" . $licenseKey);
    }

    private function assertNoCallerControlledGrantFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172LicenseTypeProjectionMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec172LicenseTypeProjectionMigration::encodeCanonical($value));
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private function assertToken(string $value, int $max, string $kind): void
    {
        if ($value === '' || strlen($value) > $max || preg_match('/[\r\n\x00]/', $value) === 1) {
            throw new InvalidArgumentException("bounded {$kind} token required");
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
