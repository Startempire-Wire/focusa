<?php
// Spec 152E lease refresh service: refresh, sequence, refund, revoke, and expiry settlement.
//
//   - POST /v1/lease/refresh (spec 152E §10, §17, §18, §19, §20, §23) re-reads canonical
//     EDD/account/node state from the databases — never from the presented client claims —
//     and settles the current active lease for (account, node, product) against that truth.
//   - Every refresh is idempotent under the caller's idempotency key; the caller may submit
//     only node_id, refresh_credential, and the advisory current_sequence (the bounded
//     facade input allowlist). No EDD id, price, grant, feature, limit, tier, commercial
//     field, or sequence is ever caller-controlled (CALLER_CONTROLLED_GRANT_DENIED).
//   - Rotation: when canonical truth holds and the presented lease is current and inside its
//     signed offline bounds, the service issues the next strictly-monotonic signed lease
//     through the EDD-bound issuer (which re-validates account/license/order/product/node),
//     supersedes the presented lease, rotates the bounded refresh credential (hash-at-rest
//     only, plaintext returned exactly once), chains previous_lease_digest, and appends
//     lease_superseded/lease_issued outbox events in the same transaction as the settlement.
//   - Denial: refund, revoke, expiry, suspension, node removal, stale sequence, or unusable
//     EDD truth produce a SIGNED refusal envelope (focusa.spec152e.refresh_refusal.v1) with
//     the recovery-only posture, the refusal reason, and the current authority sequence. The
//     presented lease is settled to the matching terminal status (refunded/revoked/
//     superseded), a lease_superseded/lease_revoked outbox event is appended, and the signed
//     refusal is journaled. Stale software/state cannot restore access: the old lease fails
//     sequence enforcement at the refusal's authority sequence and is never extended locally.
//   - The EDD lifecycle projector (docs/contracts/spec152e-edd-lifecycle-projection.v1.php)
//     is consulted per license for the canonical refresh posture; the transactional outbox
//     (docs/contracts/spec152e-authority-outbox.v1.php) receives every lease settlement.
//   - No raw email, secret, license key, or unmasked real-email evidence is accepted or
//     stored; refund/revoke/expiry never delete the EDD customer, order, license, node,
//     lease, credential, or audit history (preservation-only settlement).
//
// Requires docs/contracts/spec152e-edd-bound-lease-issuer.v1.php,
// docs/contracts/spec152e-edd-lifecycle-projection.v1.php, and
// docs/contracts/spec152e-authority-outbox.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eLeaseRefreshMigration
{
    public const SCHEMA = 'focusa.spec152e.lease_refresh.v1';
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
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $log = $this->table('wpuiai_lease_refresh_log');
        $credentials = $this->table('wpuiai_lease_refresh_credentials');
        $idempotency = $this->table('wpuiai_lease_refresh_idempotency');
        $migrations = $this->table('wpuiai_lease_refresh_schema_migrations');
        $events = $this->table('wpuiai_lease_refresh_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$log} (
            refresh_uuid {$uuid} NOT NULL PRIMARY KEY,
            lease_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NOT NULL,
            node_id VARCHAR(191) NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            presented_sequence BIGINT NOT NULL,
            authority_sequence BIGINT NOT NULL,
            decision VARCHAR(16) NOT NULL CHECK (decision IN ('rotated','denied','replayed')),
            posture VARCHAR(16) NOT NULL CHECK (posture IN ('activated','recovery_only')),
            error_code VARCHAR(64) NULL,
            rotated_lease_uuid {$uuid} NULL,
            refusal_payload_b64 TEXT NULL,
            refusal_signature_b64 TEXT NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_lease_refresh_log_idempotency_idx
            ON {$log} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_lease_refresh_log_account_idx
            ON {$log} (account_uuid, created_at)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$credentials} (
            lease_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            node_id VARCHAR(191) NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            credential_digest VARCHAR(64) NOT NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('current','superseded','revoked')),
            issued_at VARCHAR(32) NOT NULL,
            rotated_at VARCHAR(32) NULL,
            revoked_at VARCHAR(32) NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(32) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            refresh_uuid {$uuid} NOT NULL,
            result_decision VARCHAR(16) NOT NULL,
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

    /** Rollback is preservation-only: refresh logs, credentials, and journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $events = $this->table('wpuiai_lease_refresh_schema_events');
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
        if (preg_match('/^[A-Za-z0-9_]*$/D', $name) !== 1) {
            throw new InvalidArgumentException('invalid table name');
        }
        return $this->prefix . $name;
    }

    public static function assertTimestamp(?string $timestamp, bool $nullable = false): void
    {
        if ($nullable && ($timestamp === null || $timestamp === '')) {
            return;
        }
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/', $timestamp) !== 1) {
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
            foreach ($item as $index => $child) {
                $item[$index] = $normalize($child);
            }
            return $item;
        };
        return json_encode($normalize($value), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }
}

/**
 * Lease refresh service. Re-reads canonical EDD/account/node state, enforces the monotonic
 * sequence and signed offline bounds, rotates the refresh credential and the signed lease
 * safely, and denies refund/revoke/expiry/node removal with a signed recovery-only refusal.
 */
final class FocusaSpec152eLeaseRefreshService
{
    public const SCHEMA = 'focusa.spec152e.lease_refresh.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.lease_refresh.v1';
    public const REFUSAL_SCHEMA = 'focusa.spec152e.refresh_refusal.v1';
    public const VERSION = 1;
    public const REFUSAL_VALIDITY_DAYS = 30;
    public const REFUSAL_SIGNER = 'focusa.spec152e.lease_refresh.v1';

    /** Bounded refusal reason codes; anything else is never emitted or accepted. */
    public const REFUSAL_REASONS = [
        'REFUNDED', 'REVOKED', 'EXPIRED', 'STALE_SEQUENCE', 'SUSPENDED', 'CANCELLED',
        'SUPERSEDED', 'NODE_NOT_FOUND', 'NODE_NOT_ACTIVE', 'NODE_NOT_BOUND',
        'EDD_LICENSE_UNUSABLE', 'LICENSE_ACCOUNT_MISMATCH', 'EDD_ORDER_PENDING',
        'EDD_ORDER_UNVERIFIED', 'REFRESH_CREDENTIAL_INVALID', 'NOT_YET_VALID',
        'ENTITLEMENT_REQUIRED',
    ];

    /** Refusal reason -> client posture (spec §18: refund/revoke/expiry => recovery-only). */
    public const REFUSAL_POSTURES = [
        'REFUNDED' => 'recovery_only', 'REVOKED' => 'recovery_only', 'EXPIRED' => 'recovery_only',
        'STALE_SEQUENCE' => 'recovery_only', 'SUSPENDED' => 'denied', 'CANCELLED' => 'recovery_only',
        'SUPERSEDED' => 'recovery_only', 'NODE_NOT_FOUND' => 'recovery_only',
        'NODE_NOT_ACTIVE' => 'recovery_only', 'NODE_NOT_BOUND' => 'recovery_only',
        'EDD_LICENSE_UNUSABLE' => 'recovery_only', 'LICENSE_ACCOUNT_MISMATCH' => 'recovery_only',
        'EDD_ORDER_PENDING' => 'recovery_only', 'EDD_ORDER_UNVERIFIED' => 'recovery_only',
        'REFRESH_CREDENTIAL_INVALID' => 'recovery_only', 'NOT_YET_VALID' => 'recovery_only',
        'ENTITLEMENT_REQUIRED' => 'denied',
    ];

    /**
     * Refusal reason -> lease settlement (status, status_reason). Entitlement-truth refusals
     * settle the presented lease to the matching terminal status; authentication-only
     * refusals (REFRESH_CREDENTIAL_INVALID, NOT_YET_VALID) are not in the map and never
     * mutate the lease row. Settlement is preservation-only.
     */
    public const LEASE_STATUS_BY_REASON = [
        'REFUNDED' => ['refunded', 'edd_refunded'],
        'REVOKED' => ['revoked', 'edd_revoked'],
        'EXPIRED' => ['superseded', 'lease_expired'],
        'STALE_SEQUENCE' => ['superseded', 'stale_sequence'],
        'SUSPENDED' => ['superseded', 'license_suspended'],
        'CANCELLED' => ['superseded', 'license_cancelled'],
        'SUPERSEDED' => ['superseded', 'license_superseded'],
        'NODE_NOT_FOUND' => ['superseded', 'node_removed'],
        'NODE_NOT_ACTIVE' => ['superseded', 'node_deactivated'],
        'NODE_NOT_BOUND' => ['superseded', 'node_unbound'],
        'EDD_LICENSE_UNUSABLE' => ['superseded', 'edd_license_unusable'],
        'LICENSE_ACCOUNT_MISMATCH' => ['superseded', 'license_account_mismatch'],
        'EDD_ORDER_PENDING' => ['superseded', 'edd_order_pending'],
        'EDD_ORDER_UNVERIFIED' => ['superseded', 'edd_order_unverified'],
    ];

    /** Refusal reason -> outbox lease event type (spec §17: lease transitions journaled). */
    public const OUTBOX_EVENT_BY_REASON = [
        'REVOKED' => 'lease_revoked',
        'REFUNDED' => 'lease_superseded', 'EXPIRED' => 'lease_superseded',
        'STALE_SEQUENCE' => 'lease_superseded', 'SUSPENDED' => 'lease_superseded',
        'CANCELLED' => 'lease_superseded', 'SUPERSEDED' => 'lease_superseded',
        'NODE_NOT_FOUND' => 'lease_superseded', 'NODE_NOT_ACTIVE' => 'lease_superseded',
        'NODE_NOT_BOUND' => 'lease_superseded', 'EDD_LICENSE_UNUSABLE' => 'lease_superseded',
        'LICENSE_ACCOUNT_MISMATCH' => 'lease_superseded', 'EDD_ORDER_PENDING' => 'lease_superseded',
        'EDD_ORDER_UNVERIFIED' => 'lease_superseded',
    ];

    /** Settled lease (status:status_reason) -> refusal reason for stable re-denial. */
    private const SETTLED_REASON_BY_STATUS = [
        'refunded:edd_refunded' => 'REFUNDED',
        'revoked:edd_revoked' => 'REVOKED',
        'superseded:lease_expired' => 'EXPIRED',
        'superseded:stale_sequence' => 'STALE_SEQUENCE',
        'superseded:node_removed' => 'NODE_NOT_FOUND',
        'superseded:node_deactivated' => 'NODE_NOT_ACTIVE',
        'superseded:node_unbound' => 'NODE_NOT_BOUND',
        'superseded:edd_license_unusable' => 'EDD_LICENSE_UNUSABLE',
        'superseded:license_account_mismatch' => 'LICENSE_ACCOUNT_MISMATCH',
        'superseded:edd_order_pending' => 'EDD_ORDER_PENDING',
        'superseded:edd_order_unverified' => 'EDD_ORDER_UNVERIFIED',
        'superseded:license_suspended' => 'SUSPENDED',
        'superseded:license_cancelled' => 'CANCELLED',
        'superseded:license_superseded' => 'SUPERSEDED',
        'superseded:refresh_rotated' => 'STALE_SEQUENCE',
    ];

    private const FORBIDDEN_COMMERCE_FIELDS = [
        'price', 'amount', 'total', 'currency', 'grants', 'features', 'limits', 'tier',
        'node_limit', 'activation_limit', 'commercial_rights', 'product_name', 'download_id',
        'edd_customer_id', 'edd_order_id', 'edd_license_id', 'lease_uuid', 'sequence',
        'expires_at', 'offline_grace_until',
    ];

    /** Server-owned EDD download mapping (spec 152E §8); mirrors the issuer's frozen registry. */
    private const SERVER_OWNED_DOWNLOAD_BY_PRODUCT = [
        'focusa_operator_lifetime_v1' => 1001,
        'uiai_operator_lifetime_v1' => 1002,
        'focusa_uiai_operator_bundle_lifetime_v1' => 1003,
        'focusa_evaluation' => 1004,
    ];

    private PDO $db;
    private FocusaSpec152eEddBoundLeaseIssuer $issuer;
    private FocusaSpec152eAuthorityKeySetSeam $keySet;
    private FocusaSpec152eEddLifecycleProjector $projector;
    private FocusaSpec152eEddAuthorityHook $outboxHook;
    private FocusaSpec152eLeaseRefreshMigration $schema;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;
    private int $retentionSeconds;

    public function __construct(
        PDO $db,
        FocusaSpec152eEddBoundLeaseIssuer $issuer,
        FocusaSpec152eAuthorityKeySetSeam $keySet,
        FocusaSpec152eEddLifecycleProjector $projector,
        FocusaSpec152eEddAuthorityHook $outboxHook,
        FocusaSpec152eLeaseRefreshMigration $schema,
        string $prefix,
        callable $clock,
        int $retentionSeconds = 7776000,
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        $this->issuer = $issuer;
        $this->keySet = $keySet;
        $this->projector = $projector;
        $this->outboxHook = $outboxHook;
        $this->schema = $schema;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
        $this->retentionSeconds = $retentionSeconds;
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        $this->schema->migrate($appliedAt, $provenance);
    }

    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        return $this->schema->preserveForRollback($occurredAt, $provenance);
    }

    /**
     * Issue the initial bounded refresh credential for one issued lease. Returns the
     * plaintext credential exactly once; only the SHA-256 digest is stored at rest.
     * A second issuance for a lease that already holds a current credential fails closed
     * (REFRESH_CREDENTIAL_ALREADY_ISSUED) so an existing client credential is never
     * silently invalidated. Used by the activation/poll authority seam after issuance.
     */
    public function issueRefreshCredential(array $input): array
    {
        $leaseUuid = (string) ($input['lease_uuid'] ?? '');
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($leaseUuid, 'lease');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $requestId = (string) ($input['request_id'] ?? '');
        $this->assertRequestId($requestId);
        $lease = $this->issuer->findLease($leaseUuid);
        if ($lease === null) {
            throw new DomainException('LEASE_NOT_FOUND');
        }
        $digest = $this->digest(['operation' => 'issue_refresh_credential', 'lease_uuid' => $leaseUuid]);

        return $this->transaction(function () use ($lease, $leaseUuid, $idempotencyKey, $digest, $requestId): array {
            $replay = $this->replayIdempotency($idempotencyKey, 'issue_refresh_credential', $digest);
            if ($replay !== null) {
                return ['schema' => self::RESULT_SCHEMA, 'lease_uuid' => $leaseUuid, 'replayed' => true];
            }
            $existing = $this->credentialRow($leaseUuid);
            if ($existing !== null && ($existing['status'] ?? '') === 'current') {
                throw new DomainException('REFRESH_CREDENTIAL_ALREADY_ISSUED');
            }
            $now = ($this->clock)();
            self::assertTimestamp($now);
            $credential = self::opaqueToken('rc_');
            $this->upsertCredential($leaseUuid, (string) $lease['account_uuid'], (string) $lease['node_id'], (string) $lease['product_code'], $credential, $now);
            $this->recordIdempotency($idempotencyKey, 'issue_refresh_credential', $digest, $leaseUuid, 'issued', $now);
            return [
                'schema' => self::RESULT_SCHEMA,
                'lease_uuid' => $leaseUuid,
                'refresh_credential' => $credential,
                'credential_digest' => hash('sha256', $credential),
                'issued_at' => $now,
            ];
        });
    }

    /**
     * Refresh one lease. Request fields (caller-bounded facade allowlist):
     *   - account_uuid (resolved server-side by IdentityService; never client-supplied in
     *     production), product_code (registry binding), node_id, refresh_credential,
     *     optional current_sequence (advisory; the authority sequence is server-derived),
     *     idempotency_key, request_id.
     * Returns a rotated result (state activated + fresh signed lease + one-time refresh
     * credential) or a signed refusal result (state recovery_only). Hard identity/input
     * failures throw DomainException codes; entitlement-truth failures return the signed
     * refusal so the client can record recovery posture.
     */
    public function refresh(array $request): array
    {
        $accountUuid = (string) ($request['account_uuid'] ?? '');
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($accountUuid, 'account');
        $productCode = (string) ($request['product_code'] ?? '');
        if ($productCode === '' || strlen($productCode) > 191) {
            throw new InvalidArgumentException('bounded product code required');
        }
        $nodeId = (string) ($request['node_id'] ?? '');
        if (preg_match(FocusaSpec152eEddBoundLeaseIssuer::NODE_ID_PATTERN, $nodeId) !== 1) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        $refreshCredential = (string) ($request['refresh_credential'] ?? '');
        if ($refreshCredential === '' || strlen($refreshCredential) > 191
            || preg_match('/[\r\n@\x00]/', $refreshCredential) === 1) {
            throw new InvalidArgumentException('bounded refresh credential required');
        }
        $currentSequence = $request['current_sequence'] ?? null;
        if ($currentSequence !== null) {
            if (!is_int($currentSequence) && !ctype_digit((string) $currentSequence)) {
                throw new InvalidArgumentException('positive current sequence required');
            }
            $currentSequence = (int) $currentSequence;
            if ($currentSequence < 1) {
                throw new InvalidArgumentException('positive current sequence required');
            }
        }
        $idempotencyKey = (string) ($request['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $requestId = (string) ($request['request_id'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertNoRawEmail($request);
        $this->assertNoClientCommerceFields($request);

        $digest = $this->digest([
            'operation' => 'lease_refresh',
            'account_uuid' => $accountUuid,
            'product_code' => $productCode,
            'node_id' => $nodeId,
            'credential_digest' => hash('sha256', $refreshCredential),
            'current_sequence' => $currentSequence,
        ]);

        // Idempotent replay first: a retry after a settlement crash converges on the same
        // rotation result (the issuer's derived idempotency key replays the same lease) and
        // re-seeds the one-time refresh credential for the rotated lease.
        $replay = $this->replayIdempotency($idempotencyKey, 'lease_refresh', $digest);
        if ($replay !== null) {
            return $this->replayResult($replay);
        }

        // ── Re-read canonical account truth (hard failures) ──
        $account = (new FocusaSpec152eEddAccountAdapter($this->db, $this->prefix))->resolve($accountUuid);
        $grant = FocusaSpec152eEddProductAdapter::resolve($productCode);
        $now = ($this->clock)();
        self::assertTimestamp($now);

        // ── Resolve the current lease for (account, node, product) ──
        $lease = $this->latestLease($accountUuid, $nodeId, $productCode);
        if ($lease === null) {
            throw new DomainException('LEASE_NOT_FOUND');
        }
        // A settled lease is a stable, already-authoritative denial: return the signed
        // refusal without re-settling or re-journaling the outbox (preservation-only).
        if (($lease['status'] ?? '') !== 'active') {
            $settledReason = self::SETTLED_REASON_BY_STATUS[($lease['status'] ?? '') . ':' . ($lease['status_reason'] ?? '')] ?? null;
            if ($settledReason === null) {
                throw new DomainException('LEASE_NOT_FOUND');
            }
            return $this->recordRefusal($account, $lease, $nodeId, $productCode, $settledReason, $requestId, $idempotencyKey, $digest, $now, false);
        }
        $customerId = (int) $account['customer_id'];
        $licenseId = (int) $lease['edd_license_id'];

        // ── Entitlement-truth checks; each produces a signed refusal ──
        $reason = $this->checkNodeTruth($nodeId, $accountUuid, $productCode, $lease);
        $reason ??= $this->checkCredential($lease, $refreshCredential);
        $reason ??= $this->checkSequence($lease, $account, $currentSequence);
        $reason ??= $this->checkLifecyclePosture($accountUuid, $licenseId);
        $reason ??= $this->checkOfflineBounds($lease, $now);
        $reason ??= $this->checkEddTruth($licenseId, $customerId, $productCode, $grant, $now);
        if ($reason !== null) {
            return $this->recordRefusal($account, $lease, $nodeId, $productCode, $reason, $requestId, $idempotencyKey, $digest, $now);
        }

        // ── Rotation: re-issue through the EDD-bound issuer (own transaction) ──
        $node = $this->nodeRow($nodeId);
        $rotated = $this->issuer->issueLease([
            'account_uuid' => $accountUuid,
            'product_code' => $productCode,
            'node_id' => $nodeId,
            'device_public_key' => (string) $node['device_public_key'],
            'idempotency_key' => self::derivedInternalKey('refresh-internal', $idempotencyKey),
            'request_id' => $requestId,
        ]);
        return $this->recordRotation($account, $lease, $rotated, $nodeId, $productCode, $requestId, $idempotencyKey, $digest, $now);
    }

    /** Highest monotonic authority sequence for an account/product (sequence surface). */
    public function highestSequence(string $accountUuid, string $productCode): array
    {
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($accountUuid, 'account');
        $statement = $this->db->prepare(
            "SELECT highest_entitlement_sequence FROM {$this->prefix}wpuiai_authority_accounts WHERE account_uuid = :uuid"
        );
        $statement->execute([':uuid' => $accountUuid]);
        $accountSequence = (int) ($statement->fetchColumn() ?: 0);
        $ledger = $this->issuer->sequenceLedger($accountUuid, $productCode);
        $ledgerSequence = $ledger === null ? 0 : (int) $ledger['current_sequence'];
        return [
            'schema' => self::RESULT_SCHEMA,
            'account_uuid' => $accountUuid,
            'product_code' => $productCode,
            'highest_entitlement_sequence' => $accountSequence,
            'lease_sequence_ledger' => $ledgerSequence,
            'highest_sequence' => max($accountSequence, $ledgerSequence),
        ];
    }

    /**
     * Verify a signed refresh refusal like the runtime verifier: key-set trust, signature,
     * schema, reason bound, key-id match, refusal window, and sequence authority. Returns
     * the normalized refusal snapshot; throws DomainException codes on every rejection.
     */
    public function verifyRefusal(array $envelope, array $context): array
    {
        if (($envelope['schema'] ?? '') !== FocusaSpec152eEddBoundLeaseIssuer::ENVELOPE_SCHEMA) {
            throw new DomainException('UNSUPPORTED_ENVELOPE_SCHEMA');
        }
        $signerKeyId = (string) ($envelope['signer_key_id'] ?? '');
        $payloadBytes = FocusaSpec152eAuthorityKeySetSeam::decodePayload((string) ($envelope['payload_b64'] ?? ''));
        $signature = base64_decode((string) ($envelope['signature_b64'] ?? ''), true);
        if ($signature === false) {
            throw new DomainException('INVALID_BASE64');
        }
        $key = $this->keySet()['keys'][0] ?? null;
        if ($key === null || ($key['key_id'] ?? '') !== $signerKeyId) {
            throw new DomainException('UNKNOWN_KEY');
        }
        if (($key['status'] ?? '') === 'revoked') {
            throw new DomainException('REVOKED_KEY');
        }
        $now = (string) ($context['now'] ?? (string) ($this->clock)());
        if ($now < $key['not_before'] || $now > $key['not_after']) {
            throw new DomainException('KEY_OUTSIDE_VALIDITY');
        }
        $publicKey = base64_decode((string) $key['public_key_b64'], true);
        if ($publicKey === false || strlen($publicKey) !== 32) {
            throw new DomainException('INVALID_PUBLIC_KEY');
        }
        if (!FocusaSpec152eEd25519Signer::verify($publicKey, $signature, FocusaSpec152eEd25519Signer::LEASE_DOMAIN, $payloadBytes)) {
            throw new DomainException('INVALID_SIGNATURE');
        }
        $payload = FocusaSpec152eAuthorityKeySetSeam::decodeJson((string) ($envelope['payload_b64'] ?? ''));
        if (($payload['schema'] ?? '') !== self::REFUSAL_SCHEMA) {
            throw new DomainException('UNSUPPORTED_PAYLOAD_SCHEMA');
        }
        if (($payload['authority_key_id'] ?? '') !== $signerKeyId) {
            throw new DomainException('AUTHORITY_KEY_MISMATCH');
        }
        if (!in_array($payload['reason_code'] ?? '', self::REFUSAL_REASONS, true)) {
            throw new DomainException('REFUSAL_REASON_UNKNOWN');
        }
        if (isset($context['expected_account_uuid']) && ($payload['account_uuid'] ?? '') !== (string) $context['expected_account_uuid']) {
            throw new DomainException('WRONG_ACCOUNT');
        }
        if (isset($context['expected_node_id']) && ($payload['node_id'] ?? '') !== (string) $context['expected_node_id']) {
            throw new DomainException('WRONG_NODE');
        }
        if (isset($context['expected_product_code']) && ($payload['product_code'] ?? '') !== (string) $context['expected_product_code']) {
            throw new DomainException('WRONG_PRODUCT');
        }
        $presented = (int) ($payload['presented_sequence'] ?? 0);
        $authority = (int) ($payload['authority_sequence'] ?? 0);
        if ($authority < $presented) {
            throw new DomainException('REFUSAL_STALE');
        }
        if ($now < (string) ($payload['not_before'] ?? '')) {
            throw new DomainException('NOT_YET_VALID');
        }
        if ($now > (string) ($payload['expires_at'] ?? '')) {
            throw new DomainException('EXPIRED');
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'refusal_id' => (string) $payload['refusal_id'],
            'account_uuid' => (string) $payload['account_uuid'],
            'node_id' => (string) $payload['node_id'],
            'product_code' => (string) $payload['product_code'],
            'lease_uuid' => (string) $payload['lease_uuid'],
            'presented_sequence' => $presented,
            'authority_sequence' => $authority,
            'posture' => (string) $payload['posture'],
            'reason_code' => (string) $payload['reason_code'],
            'expires_at' => (string) $payload['expires_at'],
            'refusal_digest' => 'sha256:' . hash('sha256', $payloadBytes),
        ];
    }

    public function refreshCount(): int
    {
        return (int) $this->db->query("SELECT COUNT(*) FROM {$this->schema->table('wpuiai_lease_refresh_log')}")->fetchColumn();
    }

    public function findByRefreshUuid(string $refreshUuid): ?array
    {
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($refreshUuid, 'refresh');
        $statement = $this->db->prepare(
            "SELECT * FROM {$this->schema->table('wpuiai_lease_refresh_log')} WHERE refresh_uuid = :uuid"
        );
        $statement->execute([':uuid' => $refreshUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function credentialRow(string $leaseUuid): ?array
    {
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($leaseUuid, 'lease');
        $statement = $this->db->prepare(
            "SELECT * FROM {$this->schema->table('wpuiai_lease_refresh_credentials')} WHERE lease_uuid = :uuid"
        );
        $statement->execute([':uuid' => $leaseUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function table(string $name): string
    {
        return $this->schema->table($name);
    }

    // ── Checks (each returns a bounded refusal reason or null) ─────────

    /** Node removal/deactivation/binding truth (spec §18 node removal denies refresh). */
    private function checkNodeTruth(string $nodeId, string $accountUuid, string $productCode, array $lease): ?string
    {
        $node = $this->nodeRow($nodeId);
        if ($node === null) {
            return 'NODE_NOT_FOUND';
        }
        if (($node['status'] ?? '') !== 'active') {
            return 'NODE_NOT_ACTIVE';
        }
        if ((string) $node['account_uuid'] !== $accountUuid || (string) $node['product_code'] !== $productCode) {
            return 'NODE_NOT_BOUND';
        }
        if ((int) $node['edd_license_id'] !== (int) $lease['edd_license_id']) {
            return 'STALE_SEQUENCE';
        }
        return null;
    }

    /** The presented refresh credential must hash-match the lease's stored current credential. */
    private function checkCredential(array $lease, string $refreshCredential): ?string
    {
        $row = $this->credentialRow((string) $lease['lease_uuid']);
        if ($row === null) {
            throw new DomainException('REFRESH_CREDENTIAL_REQUIRED');
        }
        if (($row['status'] ?? '') !== 'current'
            || !hash_equals((string) $row['credential_digest'], hash('sha256', $refreshCredential))) {
            return 'REFRESH_CREDENTIAL_INVALID';
        }
        return null;
    }

    /** Monotonic sequence enforcement: the lease must be current at the account's highest sequence. */
    private function checkSequence(array $lease, array $account, ?int $currentSequence): ?string
    {
        if ($currentSequence !== null && $currentSequence !== (int) $lease['sequence']) {
            return 'STALE_SEQUENCE';
        }
        if ((int) $lease['sequence'] < (int) $account['highest_entitlement_sequence']) {
            return 'STALE_SEQUENCE';
        }
        $lifecycle = $this->latestLifecycleForLicense((string) $account['account_uuid'], (int) $lease['edd_license_id']);
        if ($lifecycle !== null && (int) $lifecycle['result_sequence'] > (int) $lease['sequence']) {
            return 'STALE_SEQUENCE';
        }
        // Account-level projector posture: an authority-relevant transition past the lease's
        // own sequence makes the lease stale even when the per-license journal lags.
        $accountLatest = $this->projector->latestProjectionForAccount((string) $account['account_uuid']);
        if ($accountLatest !== null && (int) $accountLatest['result_sequence'] > (int) $lease['sequence']) {
            return 'STALE_SEQUENCE';
        }
        return null;
    }

    /** EDD lifecycle projector posture for the exact license (refund/revoke/expiry/suspend). */
    private function checkLifecyclePosture(string $accountUuid, int $licenseId): ?string
    {
        $lifecycle = $this->latestLifecycleForLicense($accountUuid, $licenseId);
        if ($lifecycle === null) {
            return null;
        }
        $posture = (string) $lifecycle['refresh_posture'];
        $state = (string) $lifecycle['license_state'];
        if ($posture === 'allowed') {
            return null;
        }
        return match ($state) {
            'refunded' => 'REFUNDED',
            'revoked' => 'REVOKED',
            'expired' => 'EXPIRED',
            'suspended' => 'SUSPENDED',
            'cancelled' => 'CANCELLED',
            'superseded' => 'SUPERSEDED',
            default => 'ENTITLEMENT_REQUIRED',
        };
    }

    /** Signed offline bounds: refresh is denied past expiry+grace; inside grace it is allowed. */
    private function checkOfflineBounds(array $lease, string $now): ?string
    {
        if ($now < (string) $lease['not_before']) {
            return 'NOT_YET_VALID';
        }
        $bound = (string) ($lease['offline_grace_until'] ?? '');
        if ($bound === '') {
            $bound = (string) $lease['expires_at'];
        }
        if ($now > $bound) {
            return 'EXPIRED';
        }
        return null;
    }

    /** Re-read canonical EDD license/order truth; unusable truth denies without local extension. */
    private function checkEddTruth(int $licenseId, int $customerId, string $productCode, array $grant, string $now): ?string
    {
        try {
            $license = (new FocusaSpec152eEddLicenseAdapter($this->db, $this->prefix))->resolve($licenseId, $customerId, $now);
            if ((int) $license['download_id'] !== self::downloadIdFor($productCode)) {
                return 'EDD_ORDER_UNVERIFIED';
            }
            (new FocusaSpec152eEddOrderAdapter($this->db, $this->prefix))->resolve(
                $license,
                $customerId,
                (string) $grant['commercial']['price_usd'],
            );
        } catch (DomainException $error) {
            $code = $error->getMessage();
            if (in_array($code, ['EDD_LICENSE_UNUSABLE', 'LICENSE_ACCOUNT_MISMATCH', 'EDD_ORDER_PENDING', 'EDD_ORDER_UNVERIFIED'], true)) {
                return $code;
            }
            throw $error;
        }
        return null;
    }

    // ── Settlement recorders ───────────────────────────────────────────

    /**
     * Record one signed refusal settlement atomically (log + lease status + outbox).
     * When $settle is false the lease is already settled: only the refresh log and the
     * idempotency record are written (stable re-denial, no duplicate journaling).
     */
    private function recordRefusal(
        array $account,
        array $lease,
        string $nodeId,
        string $productCode,
        string $reason,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
        bool $settle = true,
    ): array {
        $refusalPayload = [
            'schema' => self::REFUSAL_SCHEMA,
            'refusal_id' => FocusaSpec152eEddBoundLeaseIssuer::opaqueUuid(),
            'account_uuid' => (string) $account['account_uuid'],
            'node_id' => $nodeId,
            'product_code' => $productCode,
            'lease_uuid' => (string) $lease['lease_uuid'],
            'presented_sequence' => (int) $lease['sequence'],
            // Authority sequence is the account's highest entitlement sequence, never below
            // the refused lease's own sequence (a refusal must always be authoritative).
            'authority_sequence' => max((int) $account['highest_entitlement_sequence'], (int) $lease['sequence']),
            'posture' => self::REFUSAL_POSTURES[$reason],
            'reason_code' => $reason,
            'issued_at' => $now,
            'not_before' => $now,
            'expires_at' => FocusaSpec152eEddBoundLeaseIssuer::plusDays($now, self::REFUSAL_VALIDITY_DAYS),
            'authority_key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
            'signer' => self::REFUSAL_SIGNER,
        ];
        $refusal = $this->keySet->seal(
            $refusalPayload,
            FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
            $this->keySet->leaseSeed(),
            FocusaSpec152eEd25519Signer::LEASE_DOMAIN,
        );
        $result = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'denied',
            'state' => 'recovery_only',
            'request_id' => $requestId,
            'presented_sequence' => (int) $lease['sequence'],
            'authority_sequence' => max((int) $account['highest_entitlement_sequence'], (int) $lease['sequence']),
            'error' => $reason,
            'refusal' => $refusal,
            'created_at' => $now,
        ];

        $this->transaction(function () use ($account, $lease, $nodeId, $productCode, $reason, $result, $refusal, $requestId, $idempotencyKey, $digest, $now, $settle): void {
            $settlement = $settle ? (self::LEASE_STATUS_BY_REASON[$reason] ?? null) : null;
            if ($settlement !== null) {
                $statement = $this->db->prepare(
                    "UPDATE {$this->prefix}wpuiai_authority_leases
                     SET status = :status, status_reason = :status_reason, updated_at = :updated
                     WHERE lease_uuid = :uuid"
                );
                $statement->execute([
                    ':status' => $settlement[0],
                    ':status_reason' => $settlement[1],
                    ':updated' => $now,
                    ':uuid' => (string) $lease['lease_uuid'],
                ]);
            }
            $eventType = self::OUTBOX_EVENT_BY_REASON[$reason] ?? null;
            if ($eventType !== null) {
                // edd_customer_id is the account's EDD customer id (customer_id view and the
                // authority-account repository's edd_customer_id are the same EDD customer).
                $this->outboxHook->append([
                    'event_type' => $eventType,
                    'account_uuid' => (string) $account['account_uuid'],
                    'edd_customer_id' => (int) $account['customer_id'],
                    'lease_uuid' => (string) $lease['lease_uuid'],
                    'license_id' => (int) $lease['edd_license_id'],
                    'request_id' => $requestId,
                    'idempotency_key' => self::derivedInternalKey('refresh-' . $eventType, $idempotencyKey),
                    'state_reason' => $settlement[1] ?? $reason,
                ]);
            }
            $this->insertLog($account, $lease, $nodeId, $productCode, 'denied', 'recovery_only', $reason, null, $refusal, $result, $requestId, $idempotencyKey, $digest, $now);
            $this->recordIdempotency($idempotencyKey, 'lease_refresh', $digest, (string) $lease['lease_uuid'], 'denied', $now);
        });
        return $result;
    }

    /** Record one rotation settlement atomically (supersede old lease/credential, seed new, outbox). */
    private function recordRotation(
        array $account,
        array $lease,
        array $rotated,
        string $nodeId,
        string $productCode,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
    ): array {
        $rotatedLeaseUuid = (string) $rotated['lease_uuid'];
        $credential = null;
        $storedResult = null;

        $this->transaction(function () use ($account, $lease, $rotated, $rotatedLeaseUuid, $nodeId, $productCode, $requestId, $idempotencyKey, $digest, $now, &$credential, &$storedResult): void {
            $statement = $this->db->prepare(
                "UPDATE {$this->prefix}wpuiai_authority_leases
                 SET status = 'superseded', status_reason = 'refresh_rotated', updated_at = :updated
                 WHERE lease_uuid = :uuid"
            );
            $statement->execute([':updated' => $now, ':uuid' => (string) $lease['lease_uuid']]);
            $statement = $this->db->prepare(
                "UPDATE {$this->schema->table('wpuiai_lease_refresh_credentials')}
                 SET status = 'superseded', rotated_at = :rotated_at
                 WHERE lease_uuid = :uuid"
            );
            $statement->execute([':rotated_at' => $now, ':uuid' => (string) $lease['lease_uuid']]);

            $credential = self::opaqueToken('rc_');
            $this->upsertCredential($rotatedLeaseUuid, (string) $account['account_uuid'], $nodeId, $productCode, $credential, $now);

            $this->outboxHook->append([
                'event_type' => 'lease_superseded',
                'account_uuid' => (string) $account['account_uuid'],
                'edd_customer_id' => (int) $account['customer_id'],
                'lease_uuid' => (string) $lease['lease_uuid'],
                'license_id' => (int) $lease['edd_license_id'],
                'request_id' => $requestId,
                'idempotency_key' => self::derivedInternalKey('refresh-lease_superseded', $idempotencyKey),
                'state_reason' => 'refresh_rotated',
            ]);
            $this->outboxHook->append([
                'event_type' => 'lease_issued',
                'account_uuid' => (string) $account['account_uuid'],
                'edd_customer_id' => (int) $account['customer_id'],
                'lease_uuid' => $rotatedLeaseUuid,
                'license_id' => (int) $lease['edd_license_id'],
                'request_id' => $requestId,
                'idempotency_key' => self::derivedInternalKey('refresh-lease_issued', $idempotencyKey),
                'state_reason' => 'refresh_rotated',
            ]);

            $result = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'rotated',
                'state' => 'activated',
                'request_id' => $requestId,
                'presented_sequence' => (int) $lease['sequence'],
                'authority_sequence' => (int) $account['highest_entitlement_sequence'],
                'previous_lease_uuid' => (string) $lease['lease_uuid'],
                'lease' => $rotated,
                'created_at' => $now,
            ];
            // The plaintext refresh credential is returned exactly once and never stored.
            $storedResult = $result;
            $this->insertLog($account, $lease, $nodeId, $productCode, 'rotated', 'activated', null, $rotatedLeaseUuid, null, $storedResult, $requestId, $idempotencyKey, $digest, $now);
            $this->recordIdempotency($idempotencyKey, 'lease_refresh', $digest, $rotatedLeaseUuid, 'rotated', $now);
        });

        $result = $storedResult ?? [];
        $result['refresh_credential'] = $credential ?? '';
        return $result;
    }

    // ── Persistence helpers ────────────────────────────────────────────

    private function insertLog(
        array $account,
        array $lease,
        string $nodeId,
        string $productCode,
        string $decision,
        string $posture,
        ?string $errorCode,
        ?string $rotatedLeaseUuid,
        ?array $refusal,
        array $result,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
    ): void {
        $table = $this->schema->table('wpuiai_lease_refresh_log');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (refresh_uuid, lease_uuid, account_uuid, node_id, product_code, presented_sequence,
             authority_sequence, decision, posture, error_code, rotated_lease_uuid,
             refusal_payload_b64, refusal_signature_b64, result_payload, request_id,
             idempotency_key, request_digest, created_at, retention_until)
            VALUES (:refresh_uuid, :lease_uuid, :account_uuid, :node_id, :product_code, :presented_sequence,
                    :authority_sequence, :decision, :posture, :error_code, :rotated_lease_uuid,
                    :refusal_payload_b64, :refusal_signature_b64, :result_payload, :request_id,
                    :idempotency_key, :request_digest, :created_at, :retention_until)");
        $statement->execute([
            ':refresh_uuid' => FocusaSpec152eEddBoundLeaseIssuer::opaqueUuid(),
            ':lease_uuid' => (string) $lease['lease_uuid'],
            ':account_uuid' => (string) $account['account_uuid'],
            ':node_id' => $nodeId,
            ':product_code' => $productCode,
            ':presented_sequence' => (int) $lease['sequence'],
            ':authority_sequence' => (int) $account['highest_entitlement_sequence'],
            ':decision' => $decision,
            ':posture' => $posture,
            ':error_code' => $errorCode,
            ':rotated_lease_uuid' => $rotatedLeaseUuid,
            ':refusal_payload_b64' => $refusal !== null ? (string) $refusal['payload_b64'] : null,
            ':refusal_signature_b64' => $refusal !== null ? (string) $refusal['signature_b64'] : null,
            ':result_payload' => self::canonicalJson($result),
            ':request_id' => $requestId,
            ':idempotency_key' => $idempotencyKey,
            ':request_digest' => $digest,
            ':created_at' => $now,
            ':retention_until' => FocusaSpec152eEddBoundLeaseIssuer::plusDays($now, 90),
        ]);
    }

    private function upsertCredential(string $leaseUuid, string $accountUuid, string $nodeId, string $productCode, string $credential, string $now): void
    {
        $table = $this->schema->table('wpuiai_lease_refresh_credentials');
        $digest = hash('sha256', $credential);
        if ($this->credentialRow($leaseUuid) === null) {
            $statement = $this->db->prepare("INSERT INTO {$table}
                (lease_uuid, account_uuid, node_id, product_code, credential_digest, status, issued_at)
                VALUES (:lease_uuid, :account_uuid, :node_id, :product_code, :digest, 'current', :issued_at)");
            $statement->execute([
                ':lease_uuid' => $leaseUuid,
                ':account_uuid' => $accountUuid,
                ':node_id' => $nodeId,
                ':product_code' => $productCode,
                ':digest' => $digest,
                ':issued_at' => $now,
            ]);
            return;
        }
        $statement = $this->db->prepare("UPDATE {$table}
            SET account_uuid = :account_uuid, node_id = :node_id, product_code = :product_code,
                credential_digest = :digest, status = 'current', issued_at = :issued_at,
                rotated_at = NULL, revoked_at = NULL
            WHERE lease_uuid = :lease_uuid");
        $statement->execute([
            ':account_uuid' => $accountUuid,
            ':node_id' => $nodeId,
            ':product_code' => $productCode,
            ':digest' => $digest,
            ':issued_at' => $now,
            ':lease_uuid' => $leaseUuid,
        ]);
    }

    private function latestLease(string $accountUuid, string $nodeId, string $productCode): ?array
    {
        $statement = $this->db->prepare(
            "SELECT * FROM {$this->prefix}wpuiai_authority_leases
             WHERE account_uuid = :account AND node_id = :node AND product_code = :product
             ORDER BY sequence DESC, created_at DESC LIMIT 1"
        );
        $statement->execute([':account' => $accountUuid, ':node' => $nodeId, ':product' => $productCode]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function nodeRow(string $nodeId): ?array
    {
        $statement = $this->db->prepare(
            "SELECT node_uuid, account_uuid, edd_license_id, product_code, device_public_key, status
             FROM {$this->prefix}wpuiai_authority_nodes WHERE node_uuid = :node"
        );
        $statement->execute([':node' => $nodeId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function latestLifecycleForLicense(string $accountUuid, int $licenseId): ?array
    {
        $table = $this->prefix . 'wpuiai_edd_lifecycle_events';
        $statement = $this->db->prepare("SELECT license_state, refresh_posture, result_sequence FROM {$table}
            WHERE account_uuid = :account AND license_id = :license AND decision IN ('applied','replayed')
            ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
        $statement->execute([':account' => $accountUuid, ':license' => $licenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function keySet(): array
    {
        // Mirrors FocusaSpec152eEddBoundLeaseIssuer::keySet(): the seam's canonical lease key.
        return [
            'schema' => FocusaSpec152eAuthorityKeySetSeam::KEY_SET_SCHEMA,
            'sequence' => FocusaSpec152eAuthorityKeySetSeam::KEY_SET_SEQUENCE,
            'issued_at' => '2026-08-01T00:00:00Z',
            'expires_at' => '2030-01-01T00:00:00Z',
            'keys' => [[
                'key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
                'public_key_b64' => $this->keySet->leasePublicKeyB64(),
                'status' => 'active',
                'not_before' => '2026-08-01T00:00:00Z',
                'not_after' => '2029-01-01T00:00:00Z',
            ]],
        ];
    }

    private static function downloadIdFor(string $productCode): int
    {
        return self::SERVER_OWNED_DOWNLOAD_BY_PRODUCT[$productCode] ?? 0;
    }

    private static function derivedInternalKey(string $prefix, string $idempotencyKey): string
    {
        return $prefix . '-' . substr(hash('sha256', $idempotencyKey), 0, 24);
    }

    // ── Input guards ───────────────────────────────────────────────────

    private function assertNoRawEmail(array $input): void
    {
        $scan = static function (mixed $value) use (&$scan): bool {
            if (is_array($value)) {
                foreach ($value as $item) {
                    if ($scan($item)) {
                        return true;
                    }
                }
                return false;
            }
            return is_string($value) && str_contains($value, '@');
        };
        if ($scan($input)) {
            throw new DomainException('INPUT_RAW_EMAIL_FORBIDDEN');
        }
    }

    private function assertNoClientCommerceFields(array $input): void
    {
        foreach (self::FORBIDDEN_COMMERCE_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
            }
        }
    }

    public function assertIdempotencyKey(string $key): void
    {
        if ($key === '' || strlen($key) > 191 || preg_match('/[\r\n@\x00]/', $key) === 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
    }

    public function assertRequestId(string $requestId): void
    {
        if ($requestId === '' || strlen($requestId) > 191 || preg_match('/[\r\n@\x00]/', $requestId) === 1) {
            throw new InvalidArgumentException('bounded request id required');
        }
    }

    private function digest(array $parts): string
    {
        ksort($parts, SORT_STRING);
        return hash('sha256', json_encode($parts, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES));
    }

    private function transaction(callable $operation): mixed
    {
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

    /** Replay a previously recorded refresh: return the stored decision; re-seed the
     *  one-time credential for a rotated result (the previous credential was returned
     *  exactly once and is never stored at rest). */
    private function replayResult(array $replay): array
    {
        $table = $this->schema->table('wpuiai_lease_refresh_log');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => (string) $replay['idempotency_key']]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('REFRESH_REPLAY_NOT_FOUND');
        }
        $result = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $result['replayed'] = true;
        if (($row['decision'] ?? '') === 'rotated' && isset($row['rotated_lease_uuid'])) {
            $rotatedLease = $this->issuer->findLease((string) $row['rotated_lease_uuid']);
            if ($rotatedLease !== null) {
                $now = ($this->clock)();
                self::assertTimestamp($now);
                $credential = self::opaqueToken('rc_');
                $this->transaction(function () use ($rotatedLease, $row, $credential, $now): void {
                    $this->upsertCredential((string) $row['rotated_lease_uuid'], (string) $rotatedLease['account_uuid'], (string) $rotatedLease['node_id'], (string) $rotatedLease['product_code'], $credential, $now);
                });
                $result['refresh_credential'] = $credential;
            }
        }
        return $result;
    }

    private function replayIdempotency(string $idempotencyKey, string $operation, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_lease_refresh_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if ($row['operation'] !== $operation || $row['request_digest'] !== $digest) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $idempotencyKey, string $operation, string $digest, string $leaseUuid, string $decision, string $now): void
    {
        $table = $this->schema->table('wpuiai_lease_refresh_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, refresh_uuid, result_decision, created_at)
            VALUES (:key, :operation, :digest, :lease, :decision, :created_at)");
        $statement->execute([
            ':key' => $idempotencyKey,
            ':operation' => $operation,
            ':digest' => $digest,
            ':lease' => $leaseUuid,
            ':decision' => $decision,
            ':created_at' => $now,
        ]);
    }

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

    private static function assertTimestamp(string $timestamp): void
    {
        FocusaSpec152eLeaseRefreshMigration::assertTimestamp($timestamp);
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(24));
    }
}

/**
 * Bounded authority route seam for POST /v1/lease/refresh (spec 152E §10, §19.9: idempotent).
 * Resolves the registered route fail-closed, invokes the refresh service, and masks the
 * response to the public facade allowlist. The one-time refresh credential is never part of
 * a masked envelope — the HTTP adapter delivers it directly to the protected credential store.
 * For HTTP rejections the adapter maps REFUNDED -> LICENSE_REFUNDED and REVOKED -> LEASE_REVOKED
 * so the existing Rust authority client disposes into RecoveryOnly.
 */
final class FocusaSpec152eLeaseRefreshRoutes
{
    public const SCHEMA = 'focusa.spec152e.lease_refresh_routes.v1';
    public const PATH = '/v1/lease/refresh';
    public const METHOD = 'POST';

    private const CLIENT_ERROR_MAP = [
        'REFUNDED' => 'LICENSE_REFUNDED',
        'REVOKED' => 'LEASE_REVOKED',
    ];

    private const PUBLIC_RESULT_FIELDS = [
        'schema', 'decision', 'state', 'request_id', 'presented_sequence',
        'authority_sequence', 'error', 'created_at',
    ];

    public static function resolveRoute(string $method, string $path): array
    {
        if (strtoupper($method) !== self::METHOD || $path !== self::PATH) {
            return self::failure('FACADE_ROUTE_DENIED', 'use_registered_authority_proxy_route');
        }
        return [
            'ok' => true,
            'schema' => self::SCHEMA,
            'authority_route' => self::PATH,
            'method' => self::METHOD,
            'handler' => 'LeaseRefreshHandler',
        ];
    }

    /** Invoke the refresh service; DomainException codes become bounded error envelopes. */
    public static function handle(FocusaSpec152eLeaseRefreshService $service, array $input): array
    {
        try {
            $result = $service->refresh($input);
        } catch (DomainException $error) {
            $code = $error->getMessage();
            $clientCode = self::CLIENT_ERROR_MAP[$code] ?? $code;
            return [
                'ok' => false,
                'status' => 400,
                'error' => $clientCode,
                'envelope' => [
                    'schema' => 'focusa.spec152e.masked_error.v1',
                    'error' => $clientCode,
                    'next_action' => $code === 'REFRESH_CREDENTIAL_INVALID' ? 'use_recovery' : 'retry_or_use_recovery',
                ],
            ];
        }
        return ['ok' => true, 'status' => 200, 'result' => $result, 'envelope' => self::maskedResponse($result)];
    }

    /** Mask a refresh result: only bounded public fields and the signed lease envelope pass. */
    public static function maskedResponse(array $result): array
    {
        $envelope = ['schema' => 'focusa.spec152e.masked_activation_envelope.v1'];
        foreach (self::PUBLIC_RESULT_FIELDS as $field) {
            if (array_key_exists($field, $result)) {
                $envelope[$field] = $result[$field];
            }
        }
        $envelope['state'] = (string) ($result['state'] ?? 'recovery_only');
        if (($result['decision'] ?? '') === 'rotated') {
            $lease = $result['lease'] ?? [];
            if (isset($lease['envelope'])) {
                $envelope['lease_envelope'] = $lease['envelope'];
            }
            $envelope['next_action'] = 'activated';
        } else {
            $envelope['next_action'] = 'use_recovery';
        }
        if (isset($result['node_id'])) {
            $envelope['node_id'] = $result['node_id'];
        }
        return $envelope;
    }

    private static function failure(string $code, string $nextAction): array
    {
        return ['ok' => false, 'status' => 400, 'error' => $code, 'envelope' => [
            'schema' => 'focusa.spec152e.masked_error.v1',
            'error' => $code,
            'next_action' => $nextAction,
        ]];
    }
}
