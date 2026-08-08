<?php
// Spec 152E transactional authority outbox and idempotent dispatcher.
//
//   - A durable transactional outbox (wp_wpuiai_authority_outbox) records EDD order,
//     license, refund, revoke, expiry, customer, node, and lease transitions. Every
//     event is appended in the SAME transaction as the canonical EDD/account state
//     change: a committed canonical change always carries its outbox row, and a
//     rolled-back (crashed) transaction carries neither. Dispatch failure can never
//     lose canonical state and never blocks the canonical EDD commit.
//   - Each event is a bounded, signed envelope: a fixed event schema (event_type,
//     surface, transition), opaque account/EDD/node/lease references only, an
//     envelope_digest (SHA-256 of the canonical payload), and a server-side HMAC
//     signature with an explicit signing key id. The dispatcher verifies the digest
//     and signature before every dispatch; tampered envelopes fail closed into the
//     dead-letter state and are never delivered.
//   - The dispatcher delivers exactly once. Delivery, the consumer application, and
//     the dispatched mark all commit in one transaction (wp_wpuiai_outbox_deliveries
//     is the immutable, unique delivery ledger). A crash before the dispatch commit
//     leaves the row pending and the redelivery re-applies exactly once; a crash
//     after the commit leaves the row dispatched and it is never redelivered. Replay
//     is idempotent and returns the existing delivery.
//   - Durable failure state is bounded and does not block canonical state: failed
//     deliveries retry with exponential backoff and a bounded error code; rows that
//     exhaust the attempt budget (or that fail verification) move to the dead-letter
//     state with a bounded repair record (attempts, bounded error code, retention).
//     repairState() and retryDeadLetter() expose and repair exactly that bounded
//     state. No raw email, payment secret, license key, or unmasked real-email
//     evidence is ever accepted, stored, or returned; no caller-controlled price,
//     amount, grant, feature, limit, tier, or commercial field is accepted.
//
// Requires docs/contracts/spec152e-authority-account.v1.php to be loaded first.
declare(strict_types=1);

/**
 * Durable transactional outbox schema. Creates wp_wpuiai_authority_outbox (the event
 * queue with dispatch/retry/dead-letter state), wp_wpuiai_outbox_deliveries (the
 * exactly-once delivery ledger), and the append-audited schema migration journals.
 */
final class FocusaSpec152eAuthorityOutboxMigration
{
    public const SCHEMA = 'focusa.spec152e.authority_outbox.v1';
    public const VERSION = 1;
    public const DISPATCH_STATES = ['pending', 'dispatched', 'failed', 'dead_letter'];

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
        $outbox = $this->table('wpuiai_authority_outbox');
        $deliveries = $this->table('wpuiai_outbox_deliveries');
        $migrations = $this->table('wpuiai_authority_outbox_schema_migrations');
        $events = $this->table('wpuiai_authority_outbox_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $states = "'" . implode("','", self::DISPATCH_STATES) . "'";

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$outbox} (
            event_uuid {$uuid} NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            event_version BIGINT NOT NULL,
            surface VARCHAR(16) NOT NULL,
            transition VARCHAR(24) NOT NULL,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            order_id BIGINT NULL,
            order_item_id BIGINT NULL,
            license_id BIGINT NULL,
            subscription_id BIGINT NULL,
            node_uuid {$uuid} NULL,
            lease_uuid {$uuid} NULL,
            authority_sequence BIGINT NOT NULL,
            result_sequence BIGINT NOT NULL,
            payload TEXT NOT NULL,
            envelope_digest VARCHAR(64) NOT NULL,
            signature VARCHAR(191) NOT NULL,
            signing_key_id VARCHAR(64) NOT NULL,
            dispatch_state VARCHAR(16) NOT NULL CHECK (dispatch_state IN ({$states})),
            attempts BIGINT NOT NULL DEFAULT 0 CHECK (attempts >= 0),
            last_attempt_at VARCHAR(32) NULL,
            next_attempt_at VARCHAR(32) NOT NULL,
            last_error VARCHAR(64) NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_outbox_idempotency
            ON {$outbox} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_outbox_due
            ON {$outbox} (dispatch_state, next_attempt_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_outbox_account
            ON {$outbox} (account_uuid, created_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_outbox_retention
            ON {$outbox} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$deliveries} (
            event_uuid {$uuid} NOT NULL PRIMARY KEY,
            idempotency_key {$key} NOT NULL UNIQUE,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            surface VARCHAR(16) NOT NULL,
            transition VARCHAR(24) NOT NULL,
            authority_sequence BIGINT NOT NULL,
            result_sequence BIGINT NOT NULL,
            envelope_digest VARCHAR(64) NOT NULL,
            delivered_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_outbox_deliveries_account
            ON {$deliveries} (account_uuid, delivered_at)");
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

    /** Rollback is preservation-only: outbox events and delivery journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $events = $this->table('wpuiai_authority_outbox_schema_events');
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
        if ($nullable && ($timestamp === null || $timestamp === '')) {
            return;
        }
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/', $timestamp) !== 1) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        ksort($value);
        return json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }
}

/**
 * Bounded event schema registry. The single authority for outbox event types,
 * surfaces, and transitions. Every event is validated here before it can be appended
 * or dispatched: unknown types, missing required references, raw email, and any
 * caller-controlled commerce field fail closed before any durable write.
 */
final class FocusaSpec152eAuthorityEventSchema
{
    public const SCHEMA = 'focusa.spec152e.authority_outbox_event.v1';
    public const VERSION = 1;
    public const KEY_ID = 'wpuiai.spec152e.outbox.v1';
    public const SIGNATURE_ALGORITHM = 'hmac_sha256.spec152e.outbox.v1';

    /** Bounded registry: event_type => [surface, transition, required references]. */
    public const EVENT_TYPES = [
        'customer_promoted' => ['surface' => 'customer', 'transition' => 'promote', 'refs' => []],
        'email_changed' => ['surface' => 'customer', 'transition' => 'email_change', 'refs' => []],
        'checkout_created' => ['surface' => 'order', 'transition' => 'checkout', 'refs' => ['order_id']],
        'order_completed' => ['surface' => 'order', 'transition' => 'complete', 'refs' => ['order_id']],
        'order_failed' => ['surface' => 'order', 'transition' => 'cancel', 'refs' => ['order_id']],
        'license_issued' => ['surface' => 'license', 'transition' => 'issue', 'refs' => ['license_id']],
        'license_reissued' => ['surface' => 'license', 'transition' => 'reissue', 'refs' => ['license_id']],
        'license_expired' => ['surface' => 'license', 'transition' => 'expire', 'refs' => ['license_id']],
        'license_revoked' => ['surface' => 'license', 'transition' => 'revoke', 'refs' => ['license_id']],
        'license_suspended' => ['surface' => 'license', 'transition' => 'suspend', 'refs' => ['license_id']],
        'license_unsuspended' => ['surface' => 'license', 'transition' => 'unsuspend', 'refs' => ['license_id']],
        'product_upgraded' => ['surface' => 'license', 'transition' => 'upgrade', 'refs' => ['license_id']],
        'product_downgraded' => ['surface' => 'license', 'transition' => 'downgrade', 'refs' => ['license_id']],
        'refund_issued' => ['surface' => 'refund', 'transition' => 'refund', 'refs' => ['order_id']],
        'chargeback_recorded' => ['surface' => 'refund', 'transition' => 'chargeback', 'refs' => ['order_id']],
        'subscription_cancelled' => ['surface' => 'subscription', 'transition' => 'cancel', 'refs' => ['subscription_id']],
        'subscription_suspended' => ['surface' => 'subscription', 'transition' => 'suspend', 'refs' => ['subscription_id']],
        'subscription_reactivated' => ['surface' => 'subscription', 'transition' => 'unsuspend', 'refs' => ['subscription_id']],
        'node_registered' => ['surface' => 'node', 'transition' => 'register_node', 'refs' => ['node_uuid']],
        'node_deactivated' => ['surface' => 'node', 'transition' => 'deactivate_node', 'refs' => ['node_uuid']],
        'lease_issued' => ['surface' => 'lease', 'transition' => 'issue', 'refs' => ['lease_uuid', 'license_id']],
        'lease_superseded' => ['surface' => 'lease', 'transition' => 'supersede', 'refs' => ['lease_uuid', 'license_id']],
        'lease_revoked' => ['surface' => 'lease', 'transition' => 'revoke', 'refs' => ['lease_uuid', 'license_id']],
    ];

    public const SURFACES = ['customer', 'order', 'license', 'refund', 'subscription', 'node', 'lease'];

    private const FORBIDDEN_COMMERCE_FIELDS = [
        'price', 'amount', 'total', 'currency', 'grants', 'features', 'limits', 'tier',
        'node_limit', 'activation_limit', 'commercial_rights', 'product_name', 'download_id',
        'product_id',
    ];

    /** Bounded error codes the dispatcher may persist; anything else maps to a generic bounded code. */
    public const KNOWN_ERROR_CODES = [
        'DELIVERY_CONSUMER_DOWN', 'DELIVERY_TIMEOUT', 'DELIVERY_REJECTED', 'DELIVERY_RATE_LIMITED',
        'DELIVERY_TEMPORARY_FAILURE', 'DISPATCH_DELIVERY_FAILED',
    ];

    /**
     * Validate and normalize an outbox event. Required input: event_type, account_uuid,
     * edd_customer_id, the type's required references, request_id, idempotency_key.
     * Returns the canonical event array (surface/transition derived from the registry).
     * Fail-closed codes: OUTBOX_EVENT_TYPE_UNKNOWN, INPUT_RAW_EMAIL_FORBIDDEN,
     * CLIENT_COMMERCIAL_FIELDS_FORBIDDEN, OUTBOX_EVENT_FIELD_MISSING, plus bounded
     * request/idempotency/UUID/reference validation.
     */
    public function validate(array $input): array
    {
        $eventType = (string) ($input['event_type'] ?? '');
        $spec = self::EVENT_TYPES[$eventType] ?? null;
        if ($spec === null) {
            throw new DomainException('OUTBOX_EVENT_TYPE_UNKNOWN');
        }
        $this->assertNoRawEmail($input);
        $this->assertNoClientCommerceFields($input);

        // Caller-declared surface/transition must match the registry exactly (fail closed).
        if (array_key_exists('surface', $input) && (string) $input['surface'] !== $spec['surface']) {
            throw new DomainException('OUTBOX_SURFACE_MISMATCH');
        }
        if (array_key_exists('transition', $input) && (string) $input['transition'] !== $spec['transition']) {
            throw new DomainException('OUTBOX_TRANSITION_MISMATCH');
        }

        $accountUuid = (string) ($input['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $customerId = (int) ($input['edd_customer_id'] ?? 0);
        if ($customerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        $requestId = (string) ($input['request_id'] ?? '');
        $this->assertBoundedKey($requestId, 'request ID');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertBoundedKey($idempotencyKey, 'idempotency key');

        $event = [
            'schema' => self::SCHEMA,
            'event_version' => self::VERSION,
            'event_type' => $eventType,
            'surface' => $spec['surface'],
            'transition' => $spec['transition'],
            'account_uuid' => $accountUuid,
            'edd_customer_id' => $customerId,
            'order_id' => $this->nullablePositiveId($input['order_id'] ?? null, 'order'),
            'order_item_id' => $this->nullablePositiveId($input['order_item_id'] ?? null, 'order item'),
            'license_id' => $this->nullablePositiveId($input['license_id'] ?? null, 'license'),
            'subscription_id' => $this->nullablePositiveId($input['subscription_id'] ?? null, 'subscription'),
            'node_uuid' => $this->nullableUuid($input['node_uuid'] ?? null, 'node'),
            'lease_uuid' => $this->nullableUuid($input['lease_uuid'] ?? null, 'lease'),
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'state_reason' => $this->boundedReason($input['state_reason'] ?? null),
        ];
        foreach ($spec['refs'] as $reference) {
            if ($event[$reference] === null) {
                throw new DomainException('OUTBOX_EVENT_FIELD_MISSING');
            }
        }
        return $event;
    }

    public function surfaceOf(string $eventType): string
    {
        return self::EVENT_TYPES[$eventType]['surface'] ?? throw new DomainException('OUTBOX_EVENT_TYPE_UNKNOWN');
    }

    public function transitionOf(string $eventType): string
    {
        return self::EVENT_TYPES[$eventType]['transition'] ?? throw new DomainException('OUTBOX_EVENT_TYPE_UNKNOWN');
    }

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
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} UUID required");
        }
    }

    private function nullableUuid(mixed $value, string $kind): ?string
    {
        if ($value === null) {
            return null;
        }
        $this->assertUuid((string) $value, $kind);
        return (string) $value;
    }

    private function nullablePositiveId(mixed $value, string $kind): ?int
    {
        if ($value === null) {
            return null;
        }
        $id = (int) $value;
        if ($id < 1 || (string) $id !== (string) $value) {
            throw new InvalidArgumentException("bounded positive {$kind} ID required");
        }
        return $id;
    }

    private function assertBoundedKey(string $key, string $kind): void
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} required");
        }
    }

    private function boundedReason(?string $reason): ?string
    {
        if ($reason === null || $reason === '') {
            return null;
        }
        if (strlen($reason) > 191 || preg_match('/[\r\n@]/', $reason) === 1) {
            throw new DomainException('INPUT_RAW_EMAIL_FORBIDDEN');
        }
        return $reason;
    }
}

/**
 * Server-side envelope signer. Signs the canonical payload with HMAC-SHA256 under an
 * explicit key id and verifies envelopes before dispatch. The secret lives only in
 * this server-side instance; it is never stored in any outbox or delivery row.
 * Verification fails closed on malformed signatures (INVALID_SIGNATURE), unknown key
 * ids (UNKNOWN_SIGNING_KEY), and any digest/signature mismatch (OUTBOX_ENVELOPE_TAMPERED).
 */
final class FocusaSpec152eAuthorityEventSigner
{
    public const SIGNATURE_ALGORITHM = 'hmac_sha256.spec152e.outbox.v1';
    public const SIGNATURE_PREFIX = 'sig_v1';

    private string $secret;
    private string $keyId;

    public function __construct(string $secret, string $keyId = FocusaSpec152eAuthorityEventSchema::KEY_ID)
    {
        if ($secret === '' || strlen($secret) < 16) {
            throw new InvalidArgumentException('server-side signing secret required');
        }
        if (preg_match('/^[A-Za-z0-9._:-]{1,64}$/D', $keyId) !== 1) {
            throw new InvalidArgumentException('bounded signing key id required');
        }
        $this->secret = $secret;
        $this->keyId = $keyId;
    }

    public function keyId(): string
    {
        return $this->keyId;
    }

    /** @return array{signature:string, signing_key_id:string} */
    public function sign(string $canonicalPayload, string $digest): array
    {
        if (!hash_equals($digest, hash('sha256', $canonicalPayload))) {
            throw new InvalidArgumentException('envelope digest mismatch');
        }
        $mac = hash_hmac('sha256', $canonicalPayload, $this->secret);
        return [
            'signature' => self::SIGNATURE_PREFIX . ':' . $this->keyId . ':' . $mac,
            'signing_key_id' => $this->keyId,
        ];
    }

    public function verify(string $canonicalPayload, string $digest, string $signature, string $signingKeyId): void
    {
        if (preg_match('/^sig_v1:[A-Za-z0-9._:-]{1,64}:[0-9a-f]{64}$/D', $signature) !== 1) {
            throw new DomainException('INVALID_SIGNATURE');
        }
        if ($signingKeyId !== $this->keyId) {
            throw new DomainException('UNKNOWN_SIGNING_KEY');
        }
        if (!hash_equals($digest, hash('sha256', $canonicalPayload))) {
            throw new DomainException('OUTBOX_ENVELOPE_TAMPERED');
        }
        [, $embeddedKeyId, $mac] = explode(':', $signature, 3);
        if (!hash_equals($embeddedKeyId, $this->keyId)) {
            throw new DomainException('UNKNOWN_SIGNING_KEY');
        }
        if (!hash_equals($mac, hash_hmac('sha256', $canonicalPayload, $this->secret))) {
            throw new DomainException('OUTBOX_ENVELOPE_TAMPERED');
        }
    }
}

/**
 * EDD lifecycle hooks. Append a signed outbox event in the SAME transaction as the
 * canonical EDD/account state change (caller-owned transaction; append() refuses to
 * run outside one). Raw EDD/Stripe statuses map to canonical event types and fail
 * closed with EDD_STATUS_UNKNOWN; the appended event snapshots the account's canonical
 * authority sequence and never bumps it (sequence advancement belongs to lifecycle
 * projection, not dispatch).
 */
final class FocusaSpec152eEddAuthorityHook
{
    public const SCHEMA = 'focusa.spec152e.authority_outbox.v1';
    public const VERSION = 1;
    public const RETENTION_SECONDS = 7776000;

    /** Raw EDD/Stripe status -> canonical outbox event type (explicit, fail closed). */
    public const EDD_EVENT_MAP = [
        'order' => [
            'completed' => 'order_completed',
            'failed' => 'order_failed',
            'cancelled' => 'order_failed',
            'refunded' => 'refund_issued',
            'partly_refunded' => 'refund_issued',
            'revoked' => 'license_revoked',
        ],
        'license' => [
            'expired' => 'license_expired',
            'revoked' => 'license_revoked',
            'disabled' => 'license_suspended',
            'inactive' => 'license_suspended',
            'active' => 'license_unsuspended',
        ],
        'subscription' => [
            'active' => 'subscription_reactivated',
            'cancelled' => 'subscription_cancelled',
            'suspended' => 'subscription_suspended',
            'failing' => 'subscription_suspended',
        ],
        'refund' => [
            'refunded' => 'refund_issued',
            'partly_refunded' => 'refund_issued',
            'chargeback' => 'chargeback_recorded',
            'disputed' => 'chargeback_recorded',
        ],
        'stripe' => [
            'paid' => 'order_completed',
            'refunded' => 'refund_issued',
            'disputed' => 'chargeback_recorded',
            'lost' => 'chargeback_recorded',
            'won' => 'license_unsuspended',
            'past_due' => 'subscription_suspended',
            'unpaid' => 'subscription_suspended',
            'canceled' => 'order_failed',
        ],
    ];

    private PDO $db;
    private FocusaSpec152eAuthorityOutboxMigration $schema;
    private FocusaSpec152eAuthorityEventSchema $eventSchema;
    private FocusaSpec152eAuthorityEventSigner $signer;
    private FocusaSpec152eAuthorityAccountRepository $accounts;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;
    private int $retentionSeconds;

    public function __construct(
        PDO $db,
        FocusaSpec152eAuthorityOutboxMigration $schema,
        FocusaSpec152eAuthorityEventSchema $eventSchema,
        FocusaSpec152eAuthorityEventSigner $signer,
        FocusaSpec152eAuthorityAccountRepository $accounts,
        string $prefix,
        callable $clock,
        int $retentionSeconds = self::RETENTION_SECONDS,
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->schema = $schema;
        $this->eventSchema = $eventSchema;
        $this->signer = $signer;
        $this->accounts = $accounts;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
        $this->retentionSeconds = $retentionSeconds;
    }

    /** EDD hook entry: raw surface + status map to a canonical event type, then append. */
    public function appendFromEdd(array $input): array
    {
        $surface = (string) ($input['surface'] ?? '');
        $status = (string) ($input['status'] ?? '');
        $eventType = self::EDD_EVENT_MAP[$surface][$status] ?? null;
        if ($eventType === null) {
            throw new DomainException('EDD_STATUS_UNKNOWN');
        }
        $input['event_type'] = $eventType;
        return $this->append($input);
    }

    /**
     * Append one signed outbox event in the caller-owned transaction that also holds the
     * canonical EDD/account change. Requires an open transaction (OUTBOX_APPEND_REQUIRES_TRANSACTION),
     * a canonical authority account (ENTITLEMENT_REQUIRED / EDD_CUSTOMER_RESOLUTION_FAILED),
     * and existing canonical EDD rows for every provided reference (EDD_ORDER_RESOLUTION_FAILED /
     * EDD_LICENSE_RESOLUTION_FAILED / EDD_SUBSCRIPTION_RESOLUTION_FAILED). The event is
     * appended as pending; dispatch failure can never lose or block canonical state.
     */
    public function append(array $input): array
    {
        if (!$this->db->inTransaction()) {
            throw new RuntimeException('OUTBOX_APPEND_REQUIRES_TRANSACTION');
        }
        $event = $this->eventSchema->validate($input);
        $accountUuid = (string) $event['account_uuid'];
        try {
            $account = $this->accounts->findByUuid($accountUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        if ((int) $account['edd_customer_id'] !== (int) $event['edd_customer_id']) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }
        $this->assertCanonicalRefs($event);

        $now = ($this->clock)();
        FocusaSpec152eAuthorityOutboxMigration::assertTimestamp($now);
        $sequence = (int) $account['highest_entitlement_sequence'];
        $eventUuid = self::opaqueToken('evt_');
        $event['event_uuid'] = $eventUuid;
        $event['authority_sequence'] = $sequence;
        $event['result_sequence'] = $sequence;
        $event['created_at'] = $now;
        $payload = FocusaSpec152eAuthorityOutboxMigration::encodeCanonical($event);
        $digest = hash('sha256', $payload);
        $signed = $this->signer->sign($payload, $digest);

        $table = $this->schema->table('wpuiai_authority_outbox');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (event_uuid, event_type, event_version, surface, transition, account_uuid,
             edd_customer_id, order_id, order_item_id, license_id, subscription_id, node_uuid,
             lease_uuid, authority_sequence, result_sequence, payload, envelope_digest, signature,
             signing_key_id, dispatch_state, attempts, last_attempt_at, next_attempt_at, last_error,
             request_id, idempotency_key, created_at, retention_until)
            VALUES (:event_uuid, :event_type, :event_version, :surface, :transition, :account_uuid,
                    :edd_customer_id, :order_id, :order_item_id, :license_id, :subscription_id, :node_uuid,
                    :lease_uuid, :authority_sequence, :result_sequence, :payload, :envelope_digest, :signature,
                    :signing_key_id, 'pending', 0, NULL, :next_attempt_at, NULL,
                    :request_id, :idempotency_key, :created_at, :retention_until)");
        $statement->execute([
            ':event_uuid' => $eventUuid,
            ':event_type' => (string) $event['event_type'],
            ':event_version' => self::VERSION,
            ':surface' => (string) $event['surface'],
            ':transition' => (string) $event['transition'],
            ':account_uuid' => $accountUuid,
            ':edd_customer_id' => (int) $event['edd_customer_id'],
            ':order_id' => $event['order_id'],
            ':order_item_id' => $event['order_item_id'],
            ':license_id' => $event['license_id'],
            ':subscription_id' => $event['subscription_id'],
            ':node_uuid' => $event['node_uuid'],
            ':lease_uuid' => $event['lease_uuid'],
            ':authority_sequence' => $sequence,
            ':result_sequence' => $sequence,
            ':payload' => $payload,
            ':envelope_digest' => $digest,
            ':signature' => (string) $signed['signature'],
            ':signing_key_id' => (string) $signed['signing_key_id'],
            ':next_attempt_at' => $now,
            ':request_id' => (string) $event['request_id'],
            ':idempotency_key' => (string) $event['idempotency_key'],
            ':created_at' => $now,
            ':retention_until' => self::plusSeconds($now, $this->retentionSeconds),
        ]);
        return $event;
    }

    /** Canonical EDD reference truth: every provided order/license/subscription ref must exist. */
    private function assertCanonicalRefs(array $event): void
    {
        if ($event['order_id'] !== null) {
            $this->assertRow('edd_orders', (int) $event['order_id'], 'EDD_ORDER_RESOLUTION_FAILED');
        }
        if ($event['license_id'] !== null) {
            $this->assertRow('edd_licenses', (int) $event['license_id'], 'EDD_LICENSE_RESOLUTION_FAILED');
        }
        if ($event['subscription_id'] !== null) {
            $this->assertRow('edd_subscriptions', (int) $event['subscription_id'], 'EDD_SUBSCRIPTION_RESOLUTION_FAILED');
        }
    }

    private function assertRow(string $table, int $id, string $errorCode): void
    {
        $tableName = $this->prefix . $table;
        $statement = $this->db->prepare("SELECT 1 FROM {$tableName} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $id]);
        if ($statement->fetchColumn() === false) {
            throw new DomainException($errorCode);
        }
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(20));
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        return gmdate('Y-m-d\TH:i:s\Z', (int) (new DateTimeImmutable($timestamp))->format('U') + $seconds);
    }
}

/**
 * Idempotent dispatcher with bounded retry and dead-letter routines. Picks due pending
 * rows, verifies each bounded signed envelope, and delivers exactly once: the consumer
 * application, the delivery ledger row, and the dispatched mark commit in one
 * transaction. Delivery failures retry with exponential backoff and a bounded error
 * code; rows that exhaust the attempt budget or fail envelope verification move to the
 * dead-letter state. repairState() exposes bounded repair state and retryDeadLetter()
 * re-queues it; replay() is idempotent and never duplicates a delivery.
 */
final class FocusaSpec152eAuthorityOutboxDispatcher
{
    public const SCHEMA = 'focusa.spec152e.authority_outbox.v1';
    public const VERSION = 1;
    public const MAX_ATTEMPTS = 5;
    public const RETRY_BASE_SECONDS = 60;

    private PDO $db;
    private FocusaSpec152eAuthorityOutboxMigration $schema;
    private FocusaSpec152eAuthorityEventSigner $signer;
    private FocusaSpec152eAuthorityEventSchema $eventSchema;
    /** @var Closure(array): void */
    private Closure $deliver;
    /** @var Closure(): string */
    private Closure $clock;
    private string $prefix;
    private int $maxAttempts;
    private int $retryBaseSeconds;

    public function __construct(
        PDO $db,
        FocusaSpec152eAuthorityOutboxMigration $schema,
        FocusaSpec152eAuthorityEventSigner $signer,
        FocusaSpec152eAuthorityEventSchema $eventSchema,
        callable $deliver,
        callable $clock,
        string $prefix = 'wp_',
        int $maxAttempts = self::MAX_ATTEMPTS,
        int $retryBaseSeconds = self::RETRY_BASE_SECONDS,
    ) {
        if ($maxAttempts < 1) {
            throw new InvalidArgumentException('positive retry budget required');
        }
        if ($retryBaseSeconds < 0) {
            throw new InvalidArgumentException('non-negative retry backoff required');
        }
        $this->db = $db;
        $this->schema = $schema;
        $this->signer = $signer;
        $this->eventSchema = $eventSchema;
        $this->deliver = Closure::fromCallable($deliver);
        $this->clock = Closure::fromCallable($clock);
        $this->prefix = $prefix;
        $this->maxAttempts = $maxAttempts;
        $this->retryBaseSeconds = $retryBaseSeconds;
    }

    /** Dispatch every due pending row (next_attempt_at <= now), up to $limit rows. */
    public function dispatchReady(int $limit = 100): array
    {
        $now = ($this->clock)();
        FocusaSpec152eAuthorityOutboxMigration::assertTimestamp($now);
        $summary = ['dispatched' => 0, 'failed' => 0, 'dead_lettered' => 0, 'tampered' => 0];
        $table = $this->schema->table('wpuiai_authority_outbox');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE dispatch_state IN ('pending', 'failed') AND next_attempt_at <= :now
            ORDER BY created_at ASC, event_uuid ASC LIMIT :limit");
        $statement->bindValue(':now', $now);
        $statement->bindValue(':limit', $limit, PDO::PARAM_INT);
        $statement->execute();
        foreach ($statement->fetchAll(PDO::FETCH_ASSOC) as $row) {
            $outcome = $this->dispatchOne($row, $now);
            $current = $this->findByEventUuid((string) $row['event_uuid']);
            $state = (string) ($current['dispatch_state'] ?? '');
            if ($outcome === 'tampered') {
                $summary['tampered']++;
            } elseif ($state === 'dead_letter') {
                $summary['dead_lettered']++;
            } elseif ($outcome === 'dispatched') {
                $summary['dispatched']++;
            } else {
                $summary['failed']++;
            }
        }
        return $summary;
    }

    /** Idempotent replay of a single event: delivers pending rows, returns the existing delivery otherwise. */
    public function replay(string $eventUuid): array
    {
        $row = $this->findByEventUuid($eventUuid);
        if ($row === null) {
            throw new DomainException('OUTBOX_EVENT_NOT_FOUND');
        }
        $existing = $this->deliveryFor($eventUuid);
        if ($existing !== null) {
            return ['event_uuid' => $eventUuid, 'outcome' => 'replayed', 'delivery' => $existing];
        }
        $now = ($this->clock)();
        FocusaSpec152eAuthorityOutboxMigration::assertTimestamp($now);
        $this->db->beginTransaction();
        try {
            $statement = $this->db->prepare("UPDATE {$this->schema->table('wpuiai_authority_outbox')}
                SET dispatch_state = 'pending', last_error = NULL
                WHERE event_uuid = :uuid");
            $statement->execute([':uuid' => $eventUuid]);
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
        $outcome = $this->dispatchOne($this->findByEventUuid($eventUuid), $now);
        $delivery = $this->deliveryFor($eventUuid);
        if ($outcome !== 'dispatched' || $delivery === null) {
            throw new DomainException('OUTBOX_DISPATCH_FAILED');
        }
        return ['event_uuid' => $eventUuid, 'outcome' => 'delivered', 'delivery' => $delivery];
    }

    /** Re-queue bounded dead-letter rows as pending (attempt budget reset). Unknown uuids fail closed. */
    public function retryDeadLetter(array $eventUuids): int
    {
        if ($eventUuids === []) {
            return 0;
        }
        $now = ($this->clock)();
        FocusaSpec152eAuthorityOutboxMigration::assertTimestamp($now);
        $requeued = 0;
        $table = $this->schema->table('wpuiai_authority_outbox');
        foreach ($eventUuids as $eventUuid) {
            $row = $this->findByEventUuid((string) $eventUuid);
            if ($row === null) {
                throw new DomainException('OUTBOX_EVENT_NOT_FOUND');
            }
            if ((string) $row['dispatch_state'] !== 'dead_letter') {
                throw new DomainException('OUTBOX_DEAD_LETTER_REQUIRED');
            }
            $statement = $this->db->prepare("UPDATE {$table}
                SET dispatch_state = 'pending', attempts = 0, last_attempt_at = NULL,
                    next_attempt_at = :now, last_error = NULL
                WHERE event_uuid = :uuid AND dispatch_state = 'dead_letter'");
            $statement->execute([':now' => $now, ':uuid' => (string) $eventUuid]);
            if ($statement->rowCount() !== 1) {
                throw new RuntimeException('concurrent dead-letter requeue denied');
            }
            $requeued++;
        }
        return $requeued;
    }

    // ── State queries ──────────────────────────────────────────────────

    public function eventCount(): int
    {
        $table = $this->schema->table('wpuiai_authority_outbox');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function stateCount(string $state): int
    {
        $table = $this->schema->table('wpuiai_authority_outbox');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE dispatch_state = :state");
        $statement->execute([':state' => $state]);
        return (int) $statement->fetchColumn();
    }

    public function deliveryCount(): int
    {
        $table = $this->schema->table('wpuiai_outbox_deliveries');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function findByEventUuid(string $eventUuid): ?array
    {
        $table = $this->schema->table('wpuiai_authority_outbox');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE event_uuid = :uuid");
        $statement->execute([':uuid' => $eventUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function deliveryFor(string $eventUuid): ?array
    {
        $table = $this->schema->table('wpuiai_outbox_deliveries');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE event_uuid = :uuid");
        $statement->execute([':uuid' => $eventUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /**
     * Bounded repair state: state counts plus bounded dead-letter/failed/pending rows.
     * Never returns raw errors, emails, secrets, or payload internals.
     */
    public function repairState(): array
    {
        $table = $this->schema->table('wpuiai_authority_outbox');
        $counts = [];
        foreach (FocusaSpec152eAuthorityOutboxMigration::DISPATCH_STATES as $state) {
            $counts[$state] = $this->stateCount($state);
        }
        $rows = $this->db->query("SELECT event_uuid, event_type, surface, transition, dispatch_state,
                attempts, last_error, next_attempt_at, created_at, retention_until
            FROM {$table}
            WHERE dispatch_state IN ('pending', 'failed', 'dead_letter')
            ORDER BY created_at ASC")->fetchAll(PDO::FETCH_ASSOC);
        $bounded = [];
        foreach ($rows as $row) {
            $bounded[] = [
                'event_uuid' => (string) $row['event_uuid'],
                'event_type' => (string) $row['event_type'],
                'surface' => (string) $row['surface'],
                'transition' => (string) $row['transition'],
                'dispatch_state' => (string) $row['dispatch_state'],
                'attempts' => (int) $row['attempts'],
                'last_error' => $row['last_error'] === null ? null : (string) $row['last_error'],
                'next_attempt_at' => (string) $row['next_attempt_at'],
                'created_at' => (string) $row['created_at'],
                'retention_until' => (string) $row['retention_until'],
            ];
        }
        return [
            'schema' => self::SCHEMA,
            'states' => $counts,
            'total' => $this->eventCount(),
            'rows' => $bounded,
        ];
    }

    // ── Dispatch core ──────────────────────────────────────────────────

    /** @return string 'dispatched' | 'failed' | 'dead_lettered' | 'tampered' */
    private function dispatchOne(array $row, string $now): string
    {
        $eventUuid = (string) $row['event_uuid'];
        $payload = (string) $row['payload'];
        try {
            $this->signer->verify(
                $payload,
                (string) $row['envelope_digest'],
                (string) $row['signature'],
                (string) $row['signing_key_id'],
            );
            $event = json_decode($payload, true, 512, JSON_THROW_ON_ERROR);
            if (!is_array($event)) {
                throw new DomainException('OUTBOX_ENVELOPE_TAMPERED');
            }
        } catch (Throwable $error) {
            $code = 'OUTBOX_ENVELOPE_TAMPERED';
            if ($error instanceof DomainException && in_array($error->getMessage(), ['INVALID_SIGNATURE', 'UNKNOWN_SIGNING_KEY', 'OUTBOX_ENVELOPE_TAMPERED'], true)) {
                $code = $error->getMessage();
            }
            $this->settleFailure($eventUuid, $code, true, $now);
            return 'tampered';
        }

        $this->db->beginTransaction();
        try {
            $existing = $this->deliveryFor($eventUuid);
            if ($existing === null) {
                ($this->deliver)($event);
                $this->recordDelivery($event, $eventUuid, $now);
            }
            $statement = $this->db->prepare("UPDATE {$this->schema->table('wpuiai_authority_outbox')}
                SET dispatch_state = 'dispatched', attempts = attempts + 1, last_attempt_at = :now,
                    next_attempt_at = :now, last_error = NULL
                WHERE event_uuid = :uuid AND dispatch_state IN ('pending', 'failed')");
            $statement->execute([':now' => $now, ':uuid' => $eventUuid]);
            $this->db->commit();
            return 'dispatched';
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            $this->settleFailure($eventUuid, $this->boundedError($error), false, $now);
            return 'failed';
        }
    }

    private function recordDelivery(array $event, string $eventUuid, string $now): void
    {
        $table = $this->schema->table('wpuiai_outbox_deliveries');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (event_uuid, idempotency_key, account_uuid, edd_customer_id, event_type, surface,
             transition, authority_sequence, result_sequence, envelope_digest, delivered_at)
            VALUES (:event_uuid, :idempotency_key, :account_uuid, :edd_customer_id, :event_type, :surface,
                    :transition, :authority_sequence, :result_sequence, :envelope_digest, :delivered_at)");
        $statement->execute([
            ':event_uuid' => $eventUuid,
            ':idempotency_key' => (string) ($event['idempotency_key'] ?? ''),
            ':account_uuid' => (string) ($event['account_uuid'] ?? ''),
            ':edd_customer_id' => (int) ($event['edd_customer_id'] ?? 0),
            ':event_type' => (string) ($event['event_type'] ?? ''),
            ':surface' => (string) ($event['surface'] ?? ''),
            ':transition' => (string) ($event['transition'] ?? ''),
            ':authority_sequence' => (int) ($event['authority_sequence'] ?? 0),
            ':result_sequence' => (int) ($event['result_sequence'] ?? 0),
            ':envelope_digest' => hash('sha256', FocusaSpec152eAuthorityOutboxMigration::encodeCanonical($event)),
            ':delivered_at' => $now,
        ]);
    }

    /** Durable, bounded failure state: backoff retry until the budget is exhausted, then dead-letter. */
    private function settleFailure(string $eventUuid, string $code, bool $tampered, string $now): void
    {
        $table = $this->schema->table('wpuiai_authority_outbox');
        $this->db->beginTransaction();
        try {
            $statement = $this->db->prepare("SELECT attempts FROM {$table} WHERE event_uuid = :uuid");
            $statement->execute([':uuid' => $eventUuid]);
            $row = $statement->fetch(PDO::FETCH_ASSOC);
            if ($row === false) {
                $this->db->commit();
                return;
            }
            $attempts = (int) $row['attempts'] + 1;
            if ($tampered || $attempts >= $this->maxAttempts) {
                $update = $this->db->prepare("UPDATE {$table}
                    SET dispatch_state = 'dead_letter', attempts = :attempts, last_attempt_at = :now,
                        next_attempt_at = :now, last_error = :code
                    WHERE event_uuid = :uuid");
                $update->execute([':attempts' => $attempts, ':now' => $now, ':code' => $code, ':uuid' => $eventUuid]);
            } else {
                $next = self::plusSeconds($now, $this->retryBaseSeconds * $attempts);
                $update = $this->db->prepare("UPDATE {$table}
                    SET dispatch_state = 'failed', attempts = :attempts, last_attempt_at = :now,
                        next_attempt_at = :next, last_error = :code
                    WHERE event_uuid = :uuid");
                $update->execute([':attempts' => $attempts, ':now' => $now, ':next' => $next, ':code' => $code, ':uuid' => $eventUuid]);
            }
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    /** Map any delivery Throwable to a bounded, persisted error code (never raw internals). */
    private function boundedError(Throwable $error): string
    {
        $message = $error->getMessage();
        if (in_array($message, FocusaSpec152eAuthorityEventSchema::KNOWN_ERROR_CODES, true)) {
            return $message;
        }
        return 'DISPATCH_DELIVERY_FAILED';
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        return gmdate('Y-m-d\TH:i:s\Z', (int) (new DateTimeImmutable($timestamp))->format('U') + $seconds);
    }
}
