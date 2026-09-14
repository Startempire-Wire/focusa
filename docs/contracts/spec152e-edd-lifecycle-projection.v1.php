<?php
// Spec 152E EDD lifecycle projection: refund, revoke, expiry, and subscription truth.
//
//   - Every authority-relevant EDD transition (order completion, refund, chargeback,
//     manual revoke, suspend, expiry, subscription cancellation, upgrade/downgrade,
//     reissue) is projected to the exact EDD license state and a strictly monotonic
//     authority sequence. The projection is deterministic, idempotent, append-audited,
//     and preservation-only: it never deletes the EDD customer, order, license,
//     subscription, refund, device, evidence, or audit history.
//   - A durable transactional outbox (wp_wpuiai_edd_lifecycle_events) records each
//     order/license/subscription/refund hook in the same transaction as the canonical
//     sequence advance. Replay is idempotent; dispatch failure cannot lose state.
//   - Stripe/EDD status adapters map raw hook statuses to canonical transitions and
//     fail closed on any unmapped or caller-invented status (EDD_STATUS_UNKNOWN).
//   - Stale entitlement cannot reactivate: an event that would flip a terminal license
//     state (refunded | revoked | expired | superseded | cancelled | denied) back to
//     active fails closed with LICENSE_TERMINAL_REACTIVATION_DENIED; a genuinely new
//     event carrying an authority ordinal not newer than the account's highest sequence
//     fails closed with ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED; duplicate redeliveries of
//     an already-applied state are journaled as 'replayed' without bumping the sequence.
//   - No raw email, raw payment id, secret, license key, or unmasked real-email evidence
//     is ever accepted, stored, or returned. No caller-controlled price, amount, grant,
//     feature, limit, tier, or commercial field is accepted.
//
// Requires docs/contracts/spec152e-authority-account.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddLifecycleProjectionMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_lifecycle_projection.v1';
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
        $events = $this->table('wpuiai_edd_lifecycle_events');
        $migrations = $this->table('wpuiai_edd_lifecycle_schema_migrations');
        $schemaEvents = $this->table('wpuiai_edd_lifecycle_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$events} (
            event_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            order_id BIGINT NULL,
            order_item_id BIGINT NULL,
            license_id BIGINT NULL,
            subscription_id BIGINT NULL,
            surface VARCHAR(16) NOT NULL CHECK (surface IN ('order','license','subscription','refund','stripe')),
            transition VARCHAR(24) NOT NULL CHECK (transition IN ('complete','refund','chargeback','revoke','suspend','unsuspend','expire','cancel','upgrade','downgrade','reissue','email_change','deactivate_node')),
            from_state VARCHAR(16) NOT NULL,
            to_state VARCHAR(16) NOT NULL,
            license_state VARCHAR(16) NOT NULL,
            refresh_posture VARCHAR(16) NOT NULL CHECK (refresh_posture IN ('allowed','denied','recovery_only')),
            sequence_increment BIGINT NOT NULL DEFAULT 0,
            result_sequence BIGINT NOT NULL,
            decision VARCHAR(16) NOT NULL CHECK (decision IN ('applied','replayed','denied')),
            error_code VARCHAR(64) NULL,
            state_reason VARCHAR(191) NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_lifecycle_idempotency
            ON {$events} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_lifecycle_account
            ON {$events} (account_uuid, result_sequence)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_lifecycle_license
            ON {$events} (license_id, result_sequence)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_lifecycle_retention
            ON {$events} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$schemaEvents} (
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

    /** Rollback is preservation-only: lifecycle journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_lifecycle_schema_events');
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
 * Stripe/EDD status adapters. Maps raw hook statuses from the EDD order, EDD Software
 * Licensing, EDD Recurring (subscription), and Stripe surfaces to the canonical lifecycle
 * transition. Every mapping is explicit; any unmapped status fails closed with
 * EDD_STATUS_UNKNOWN and can never produce or reactivate entitlement.
 */
final class FocusaSpec152eEddStatusAdapter
{
    /** @return array{transition:string,license_state:string,refresh_posture:string,sequence_increment:int} */
    public static function adaptOrder(string $status): array
    {
        return match ($status) {
            'completed' => ['transition' => 'complete', 'license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1],
            'refunded', 'partly_refunded' => ['transition' => 'refund', 'license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'revoked' => ['transition' => 'revoke', 'license_state' => 'revoked', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'cancelled', 'failed' => ['transition' => 'cancel', 'license_state' => 'cancelled', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            default => throw new DomainException('EDD_STATUS_UNKNOWN'),
        };
    }

    /** EDD Software Licensing license-status change (from_status -> to_status). */
    public static function adaptLicenseChange(string $from, string $to): array
    {
        $to = strtolower($to);
        return match ($to) {
            'expired' => ['transition' => 'expire', 'license_state' => 'expired', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'revoked' => ['transition' => 'revoke', 'license_state' => 'revoked', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'disabled', 'inactive' => ['transition' => 'suspend', 'license_state' => 'suspended', 'refresh_posture' => 'denied', 'sequence_increment' => 1],
            'active' => ['transition' => 'unsuspend', 'license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1],
            default => throw new DomainException('EDD_STATUS_UNKNOWN'),
        };
    }

    /** EDD Recurring subscription status. */
    public static function adaptSubscription(string $status): array
    {
        return match ($status) {
            'active' => ['transition' => 'complete', 'license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1],
            'cancelled' => ['transition' => 'cancel', 'license_state' => 'cancelled', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'expired' => ['transition' => 'expire', 'license_state' => 'expired', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'suspended', 'failing' => ['transition' => 'suspend', 'license_state' => 'suspended', 'refresh_posture' => 'denied', 'sequence_increment' => 1],
            default => throw new DomainException('EDD_STATUS_UNKNOWN'),
        };
    }

    /** Stripe payment/subscription status (chargeback and dunning included). */
    public static function adaptStripe(string $status): array
    {
        return match ($status) {
            'paid' => ['transition' => 'complete', 'license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1],
            'refunded' => ['transition' => 'refund', 'license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'disputed', 'lost' => ['transition' => 'chargeback', 'license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'canceled', 'cancelled', 'void' => ['transition' => 'cancel', 'license_state' => 'cancelled', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'past_due', 'unpaid' => ['transition' => 'suspend', 'license_state' => 'suspended', 'refresh_posture' => 'denied', 'sequence_increment' => 1],
            'won' => ['transition' => 'unsuspend', 'license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1],
            default => throw new DomainException('EDD_STATUS_UNKNOWN'),
        };
    }

    /** EDD refund hook status (edd_order_refunded / chargeback adapters). */
    public static function adaptRefund(string $status): array
    {
        return match ($status) {
            'refunded', 'partly_refunded' => ['transition' => 'refund', 'license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            'chargeback', 'disputed', 'lost' => ['transition' => 'chargeback', 'license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1],
            default => throw new DomainException('EDD_STATUS_UNKNOWN'),
        };
    }
}

/**
 * Authority lifecycle projector. Consumes EDD order/license/subscription/refund hooks and
 * Stripe status adapters, projects the exact EDD license state, and advances the
 * account's monotonic authority sequence in the same transaction as the outbox event.
 * Replay is idempotent; out-of-order events and stale reactivation fail closed.
 */
final class FocusaSpec152eEddLifecycleProjector
{
    public const SCHEMA = 'focusa.spec152e.edd_lifecycle_projection.v1';
    public const VERSION = 1;

    /** Canonical transition table: the single authority for target state and sequence effect. */
    public const TRANSITIONS = [
        'complete' => ['license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1, 'audit_only' => false],
        'refund' => ['license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'chargeback' => ['license_state' => 'refunded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'revoke' => ['license_state' => 'revoked', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'suspend' => ['license_state' => 'suspended', 'refresh_posture' => 'denied', 'sequence_increment' => 1, 'audit_only' => false],
        'unsuspend' => ['license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1, 'audit_only' => false],
        'expire' => ['license_state' => 'expired', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'cancel' => ['license_state' => 'cancelled', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'upgrade' => ['license_state' => 'superseded', 'refresh_posture' => 'recovery_only', 'sequence_increment' => 1, 'audit_only' => false],
        'downgrade' => ['license_state' => 'superseded', 'refresh_posture' => 'allowed', 'sequence_increment' => 1, 'audit_only' => false],
        'reissue' => ['license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 1, 'audit_only' => false],
        'email_change' => ['license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 0, 'audit_only' => true],
        'deactivate_node' => ['license_state' => 'active', 'refresh_posture' => 'allowed', 'sequence_increment' => 0, 'audit_only' => true],
    ];

    /** Terminal license states: stale events can never flip these back to active. */
    public const TERMINAL_STATES = ['refunded', 'revoked', 'expired', 'superseded', 'cancelled', 'denied'];

    public const SURFACES = ['order', 'license', 'subscription', 'refund', 'stripe'];

    private const FORBIDDEN_COMMERCE_FIELDS = [
        'price', 'amount', 'total', 'currency', 'grants', 'features', 'limits', 'tier',
        'node_limit', 'activation_limit', 'commercial_rights', 'product_name', 'download_id',
    ];

    private PDO $db;
    private FocusaSpec152eAuthorityAccountRepository $accounts;
    private FocusaSpec152eEddLifecycleProjectionMigration $schema;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;
    private int $retentionSeconds;

    public function __construct(
        PDO $db,
        FocusaSpec152eAuthorityAccountRepository $accounts,
        FocusaSpec152eEddLifecycleProjectionMigration $schema,
        string $prefix,
        callable $clock,
        int $retentionSeconds = 7776000,
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->accounts = $accounts;
        $this->schema = $schema;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
        $this->retentionSeconds = $retentionSeconds;
    }

    // ── Surface entry points ──────────────────────────────────────────

    /** EDD order hook: raw edd_orders.status -> canonical projection. */
    public function projectOrder(array $input): array
    {
        $input['surface'] = 'order';
        $input['transition'] = FocusaSpec152eEddStatusAdapter::adaptOrder((string) ($input['status'] ?? ''))['transition'];
        return $this->project($input);
    }

    /** EDD Software Licensing license-status hook: from_status -> to_status. */
    public function projectLicense(array $input): array
    {
        $input['surface'] = 'license';
        $input['transition'] = FocusaSpec152eEddStatusAdapter::adaptLicenseChange(
            (string) ($input['from_status'] ?? ''),
            (string) ($input['to_status'] ?? ''),
        )['transition'];
        return $this->project($input);
    }

    /** EDD Recurring subscription hook: raw subscription status -> canonical projection. */
    public function projectSubscription(array $input): array
    {
        $input['surface'] = 'subscription';
        $input['transition'] = FocusaSpec152eEddStatusAdapter::adaptSubscription((string) ($input['status'] ?? ''))['transition'];
        return $this->project($input);
    }

    /** EDD refund hook: refunded / partly_refunded / chargeback statuses. */
    public function projectRefund(array $input): array
    {
        $input['surface'] = 'refund';
        $input['transition'] = FocusaSpec152eEddStatusAdapter::adaptRefund((string) ($input['status'] ?? ''))['transition'];
        return $this->project($input);
    }

    /** Stripe adapter hook: raw Stripe status -> canonical projection. */
    public function projectStripe(array $input): array
    {
        $input['surface'] = 'stripe';
        $input['transition'] = FocusaSpec152eEddStatusAdapter::adaptStripe((string) ($input['status'] ?? ''))['transition'];
        return $this->project($input);
    }

    /** Internal/management surface for explicit transitions (upgrade, reissue, audit hooks). */
    public function projectTransition(array $input): array
    {
        if (!is_string($input['transition'] ?? null) || $input['transition'] === '') {
            throw new DomainException('EDD_TRANSITION_UNKNOWN');
        }
        if (!is_string($input['surface'] ?? null)) {
            $input['surface'] = 'order';
        }
        return $this->project($input);
    }

    // ── Projection ─────────────────────────────────────────────────────

    /**
     * Project one EDD lifecycle event. Required input:
     *   - surface: order|license|subscription|refund|stripe
     *   - account_uuid, edd_customer_id
     *   - order_id / order_item_id / license_id / subscription_id (at least one ref)
     *   - transition (explicit) or status/from_status/to_status (adapter surfaces)
     *   - request_id, idempotency_key
     * Optional:
     *   - authority_sequence: server ordinal used to detect out-of-order delivery
     *   - state_reason: bounded human-readable reason (no raw email or secrets)
     * Never accepted: price/amount/grants/features/limits/tier/commercial fields or any
     * raw email anywhere in the payload.
     */
    public function project(array $input): array
    {
        $this->assertSurface($input);
        $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $this->assertNoRawEmail($input);
        $this->assertNoClientCommerceFields($input);

        $accountUuid = (string) ($input['account_uuid'] ?? '');
        $this->assertUuid($accountUuid, 'account');
        $customerId = (int) ($input['edd_customer_id'] ?? 0);
        if ($customerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        try {
            $account = $this->accounts->findByUuid($accountUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        if ((int) $account['edd_customer_id'] !== $customerId) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }

        $transition = (string) ($input['transition'] ?? '');
        $spec = self::TRANSITIONS[$transition] ?? null;
        if ($spec === null) {
            throw new DomainException('EDD_TRANSITION_UNKNOWN');
        }

        $digest = $this->digest([
            'surface' => (string) ($input['surface'] ?? ''),
            'transition' => $transition,
            'account_uuid' => $accountUuid,
            'edd_customer_id' => $customerId,
            'order_id' => $input['order_id'] ?? null,
            'order_item_id' => $input['order_item_id'] ?? null,
            'license_id' => $input['license_id'] ?? null,
            'subscription_id' => $input['subscription_id'] ?? null,
            'authority_sequence' => $input['authority_sequence'] ?? null,
        ]);

        $replayed = $this->replayDecision($idempotencyKey, $digest);
        if ($replayed !== null) {
            return $replayed;
        }

        $current = $this->currentProjection($accountUuid, $input);
        $now = ($this->clock)();
        FocusaSpec152eEddLifecycleProjectionMigration::assertTimestamp($now);
        $requestId = (string) $input['request_id'];
        $reason = $this->boundedReason($input['state_reason'] ?? null);

        // Audit-only hooks (email_change, deactivate_node) never alter entitlement truth.
        if ($spec['audit_only']) {
            $targetState = $current['license_state'] ?? 'none';
            $targetPosture = $current['refresh_posture'] ?? 'allowed';
            return $this->record(
                $input, $account, $transition, $current, $targetState, $targetPosture, 0,
                'applied', null, $reason, $requestId, $idempotencyKey, $digest, $now,
            );
        }

        // Duplicate redelivery of an already-applied state: journaled replay, no bump.
        $targetState = $spec['license_state'];
        $targetPosture = $spec['refresh_posture'];
        if (($current['license_state'] ?? 'none') === $targetState) {
            return $this->record(
                $input, $account, $transition, $current, $targetState, $targetPosture, 0,
                'replayed', null, $reason, $requestId, $idempotencyKey, $digest, $now,
            );
        }

        // Out-of-order delivery: a genuinely new event whose authority ordinal is not
        // newer than the account's highest sequence can never roll the sequence back.
        $authoritySequence = $input['authority_sequence'] ?? null;
        if ($authoritySequence !== null) {
            $this->assertAuthoritySequence($authoritySequence);
            if ($authoritySequence <= (int) $account['highest_entitlement_sequence']) {
                return $this->record(
                    $input, $account, $transition, $current, $targetState, $targetPosture, 0,
                    'denied', 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', $reason, $requestId, $idempotencyKey, $digest, $now,
                );
            }
        }

        // Stale reactivation: terminal states (refunded/revoked/expired/superseded/
        // cancelled/denied) can never flip back to active from a hook event.
        if ($targetState === 'active' && in_array($current['license_state'] ?? 'none', self::TERMINAL_STATES, true)) {
            return $this->record(
                $input, $account, $transition, $current, $targetState, $targetPosture, 0,
                'denied', 'LICENSE_TERMINAL_REACTIVATION_DENIED', $reason, $requestId, $idempotencyKey, $digest, $now,
            );
        }

        // Entitlement-requiring transitions cannot fire against a scope with no truth.
        if (($current['license_state'] ?? 'none') === 'none' && $transition !== 'complete') {
            return $this->record(
                $input, $account, $transition, $current, $targetState, $targetPosture, 0,
                'denied', 'ENTITLEMENT_REQUIRED', $reason, $requestId, $idempotencyKey, $digest, $now,
            );
        }

        $increment = (int) $spec['sequence_increment'];
        $resultSequence = (int) $account['highest_entitlement_sequence'] + $increment;
        return $this->record(
            $input, $account, $transition, $current, $targetState, $targetPosture, $increment,
            'applied', null, $reason, $requestId, $idempotencyKey, $digest, $now,
            $resultSequence,
        );
    }

    // ── Journaling and queries ─────────────────────────────────────────

    public function eventCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function findByEventUuid(string $eventUuid): ?array
    {
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE event_uuid = :uuid");
        $statement->execute([':uuid' => $eventUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Account-level posture: the latest applied/replayed event for the account. */
    public function latestProjectionForAccount(string $accountUuid): ?array
    {
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE account_uuid = :uuid AND decision IN ('applied','replayed')
            ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
        $statement->execute([':uuid' => $accountUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function currentProjection(string $accountUuid, array $input): array
    {
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        $column = null;
        $value = null;
        foreach (['license_id', 'order_item_id', 'order_id', 'subscription_id'] as $candidate) {
            if (isset($input[$candidate]) && $input[$candidate] !== null) {
                $column = $candidate;
                $value = (int) $input[$candidate];
                break;
            }
        }
        $params = [':uuid' => $accountUuid];
        if ($column === null) {
            $where = 'account_uuid = :uuid';
        } else {
            $where = "account_uuid = :uuid AND {$column} = :scope";
            $params[':scope'] = $value;
        }
        $statement = $this->db->prepare("SELECT license_state, refresh_posture FROM {$table}
            WHERE {$where} AND decision IN ('applied','replayed')
            ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
        $statement->execute($params);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? [] : ['license_state' => (string) $row['license_state'], 'refresh_posture' => (string) $row['refresh_posture']];
    }

    private function record(
        array $input,
        array $account,
        string $transition,
        array $current,
        string $targetState,
        string $targetPosture,
        int $increment,
        string $decision,
        ?string $errorCode,
        ?string $reason,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
        ?int $resultSequence = null,
    ): array {
        $fromState = (string) ($current['license_state'] ?? 'none');
        $base = [
            'schema' => 'focusa.spec152e.edd_lifecycle_event.v1',
            'event_uuid' => self::opaqueToken('evt_'),
            'account_uuid' => (string) $account['account_uuid'],
            'edd_customer_id' => (int) $account['edd_customer_id'],
            'order_id' => isset($input['order_id']) ? (int) $input['order_id'] : null,
            'order_item_id' => isset($input['order_item_id']) ? (int) $input['order_item_id'] : null,
            'license_id' => isset($input['license_id']) ? (int) $input['license_id'] : null,
            'subscription_id' => isset($input['subscription_id']) ? (int) $input['subscription_id'] : null,
            'surface' => (string) $input['surface'],
            'transition' => $transition,
            'from_state' => $fromState,
            'to_state' => $targetState,
            'license_state' => $targetState,
            'refresh_posture' => $targetPosture,
            'sequence_increment' => $increment,
            'sequence' => (int) $account['highest_entitlement_sequence'],
            'result_sequence' => $resultSequence ?? (int) $account['highest_entitlement_sequence'],
            'decision' => $decision,
            'error_code' => $errorCode,
            'state_reason' => $reason,
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'created_at' => $now,
        ];

        $next = $resultSequence ?? (int) $account['highest_entitlement_sequence'];
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        $this->db->beginTransaction();
        try {
            if ($increment > 0) {
                $statement = $this->db->prepare("UPDATE {$this->prefix}wpuiai_authority_accounts
                    SET highest_entitlement_sequence = :next, updated_at = :updated
                    WHERE account_uuid = :uuid AND highest_entitlement_sequence < :guard");
                $statement->execute([
                    ':next' => $next,
                    ':updated' => $now,
                    ':uuid' => (string) $account['account_uuid'],
                    ':guard' => $next,
                ]);
                if ($statement->rowCount() !== 1) {
                    throw new RuntimeException('concurrent sequence advance denied');
                }
            }
            $statement = $this->db->prepare("INSERT INTO {$table}
                (event_uuid, account_uuid, edd_customer_id, order_id, order_item_id, license_id,
                 subscription_id, surface, transition, from_state, to_state, license_state,
                 refresh_posture, sequence_increment, result_sequence, decision, error_code,
                 state_reason, result_payload, request_id, idempotency_key, request_digest,
                 created_at, retention_until)
                VALUES (:event_uuid, :account_uuid, :customer, :order_id, :order_item_id, :license_id,
                        :subscription_id, :surface, :transition, :from_state, :to_state, :license_state,
                        :refresh_posture, :increment, :result_sequence, :decision, :error_code,
                        :state_reason, :payload, :request_id, :idempotency_key, :request_digest,
                        :created_at, :retention_until)");
            $stored = $base;
            unset($stored['sequence']);
            $statement->execute([
                ':event_uuid' => $base['event_uuid'],
                ':account_uuid' => $base['account_uuid'],
                ':customer' => $base['edd_customer_id'],
                ':order_id' => $base['order_id'],
                ':order_item_id' => $base['order_item_id'],
                ':license_id' => $base['license_id'],
                ':subscription_id' => $base['subscription_id'],
                ':surface' => $base['surface'],
                ':transition' => $base['transition'],
                ':from_state' => $base['from_state'],
                ':to_state' => $base['to_state'],
                ':license_state' => $base['license_state'],
                ':refresh_posture' => $base['refresh_posture'],
                ':increment' => $base['sequence_increment'],
                ':result_sequence' => $base['result_sequence'],
                ':decision' => $base['decision'],
                ':error_code' => $base['error_code'],
                ':state_reason' => $base['state_reason'],
                ':payload' => json_encode($stored, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request_id' => $base['request_id'],
                ':idempotency_key' => $base['idempotency_key'],
                ':request_digest' => $digest,
                ':created_at' => $base['created_at'],
                ':retention_until' => self::plusSeconds($now, $this->retentionSeconds),
            ]);
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
        return $base;
    }

    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_lifecycle_events');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        $decision = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $decision['decision'] = 'replayed';
        return $decision;
    }

    // ── Input guards ───────────────────────────────────────────────────

    private function assertSurface(array $input): void
    {
        $surface = (string) ($input['surface'] ?? '');
        if (!in_array($surface, self::SURFACES, true)) {
            throw new DomainException('EDD_TRANSITION_UNKNOWN');
        }
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

    private function assertAuthoritySequence(mixed $value): void
    {
        if (!is_int($value) || $value < 1) {
            throw new InvalidArgumentException('positive authority sequence required');
        }
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} UUID required");
        }
    }

    private function boundedReason(?string $reason): ?string
    {
        if ($reason === null || $reason === '') {
            return null;
        }
        if (strlen($reason) > 191 || preg_match('/[\r\n@]/', $reason) === 1) {
            throw new InvalidArgumentException('bounded state reason required');
        }
        return $reason;
    }

    private function digest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddLifecycleProjectionMigration::encodeCanonical($value));
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
