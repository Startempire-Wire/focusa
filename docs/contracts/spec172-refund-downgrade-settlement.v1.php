<?php
// Spec 172 refund, chargeback, revoke, and whole-Bundle downgrade settlement
// (addendum sections 9.3, 17, and 21; atom focusa-vbcqu.20.15.16).
//
//   - The Bundle settles as a WHOLE ORDER only: a Bundle refund is a 30-day whole-order
//     refund (`FocusaSpec172BundleRefundTruthAdapter` derives `scope=whole_order` from
//     canonical EDD refund rows, never from caller input). Component-level partial
//     refunds are not supported in v1 and fail closed with
//     COMPONENT_REFUND_UNSUPPORTED; refunds outside the 30-day window fail closed with
//     REFUND_WINDOW_EXPIRED.
//   - `FocusaSpec172RefundDowngradeSettler` settles each adverse event EXACTLY ONCE
//     against the accepted composite Bundle projection (shared
//     `wpuiai_license_type_projections` journal): refund/chargeback/revoke revoke BOTH
//     underlying Operator v1 grants together (`grants_revoked=2`), increment the
//     account's monotonic authority sequence by exactly one, preserve the account,
//     customer, order, license, refund, projection, and audit history (nothing is ever
//     deleted), and return a still-mailbox-verified account to `verified_no_license`
//     limited mode (unverified accounts go to `unverified`). A duplicate redelivery of
//     an already-settled event is journaled `replayed` with zero sequence bump; a second
//     adverse event on an already-terminal Bundle (e.g. refund after revoke) is also
//     journaled `replayed` — both grants stay revoked and no state changes.
//   - Stale entitlement can never reactivate: after a terminal settlement
//     (refunded/revoked), a stale `complete`/`unsuspend`/cache event fails closed with
//     LICENSE_TERMINAL_REACTIVATION_DENIED, a genuinely new event whose authority
//     ordinal is not newer than the account's highest sequence fails closed with
//     ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED, and a stale paid credential from before the
//     settlement is rejected by `FocusaSpec172AssertionTransitionFixture`
//     (PAID_GRANT_REVOKED / STALE_CREDENTIAL_SUPERSEDED).
//   - A durable transactional outbox (`wpuiai_spec172_settlement_outbox`) records each
//     applied settlement in the SAME transaction as the sequence advance and the
//     settlement journal row; dispatch is exactly-once through the unique delivery
//     ledger (`wpuiai_spec172_settlement_deliveries`), each envelope is digest + HMAC
//     signed, and tampered envelopes dead-letter. The bounded reconciler
//     (`FocusaSpec172SettlementReconciler`) proves idempotent convergence: canonical
//     refunded/revoked/disputed Bundle orders missing a settlement are repaired
//     evidence-safe from canonical EDD truth and a second apply run repairs zero.
//   - No raw email, raw payment id, secret, license key, customer row, credential, or
//     card data is ever accepted, stored, or returned; no caller-controlled price,
//     amount, refund scope, grant, feature, limit, tier, or commercial right is
//     accepted. The lifecycle transition matrix (section 21 of the addendum) is the
//     single authority for target state, terminality, sequence effect, and refund
//     window.
//
// Requires docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php, and (for the limited assertion
// transition fixture) docs/contracts/spec172-limited-access-assertion-service.v1.php to
// be loaded first. The composite Bundle projection journal is created by
// docs/contracts/spec172-edd-license-type-projector.v1.php (shared schema migration).
declare(strict_types=1);

/**
 * Spec 172 settlement schema. Creates the settlement journal
 * (wp_wpuiai_spec172_settlements), the transactional settlement outbox
 * (wp_wpuiai_spec172_settlement_outbox), the exactly-once delivery ledger
 * (wp_wpuiai_spec172_settlement_deliveries), the reconciler run/finding/repair/
 * quarantine journals, and the append-audited schema migration journals. Settlement is
 * preservation-only: no customer, order, license, refund, projection, or audit row is
 * ever deleted, updated in place, or downgraded.
 */
final class FocusaSpec172RefundDowngradeMigration
{
    public const SCHEMA = 'focusa.spec172.refund_downgrade_settlement.v1';
    public const VERSION = 1;
    public const DISPATCH_STATES = ['pending', 'dispatched', 'failed', 'dead_letter'];
    public const DECISIONS = ['applied', 'replayed', 'denied'];
    public const TRANSITIONS = ['refund', 'chargeback', 'revoke', 'complete', 'unsuspend'];

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
        $settlements = $this->table('wpuiai_spec172_settlements');
        $outbox = $this->table('wpuiai_spec172_settlement_outbox');
        $deliveries = $this->table('wpuiai_spec172_settlement_deliveries');
        $runs = $this->table('wpuiai_spec172_settlement_runs');
        $findings = $this->table('wpuiai_spec172_settlement_findings');
        $repairs = $this->table('wpuiai_spec172_settlement_repairs');
        $quarantine = $this->table('wpuiai_spec172_settlement_quarantine');
        $migrations = $this->table('wpuiai_spec172_settlement_schema_migrations');
        $events = $this->table('wpuiai_spec172_settlement_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $decisions = "'" . implode("','", self::DECISIONS) . "'";
        $transitions = "'" . implode("','", self::TRANSITIONS) . "'";
        $states = "'" . implode("','", self::DISPATCH_STATES) . "'";

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$settlements} (
            settlement_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            projection_key VARCHAR(64) NOT NULL,
            license_type_ref VARCHAR(128) NOT NULL,
            transition VARCHAR(16) NOT NULL CHECK (transition IN ({$transitions})),
            scope VARCHAR(16) NOT NULL CHECK (scope IN ('whole_order','not_applicable')),
            from_state VARCHAR(16) NOT NULL,
            to_state VARCHAR(16) NOT NULL CHECK (to_state IN ('refunded','revoked')),
            grants_revoked INT NOT NULL DEFAULT 0 CHECK (grants_revoked >= 0),
            limited_posture VARCHAR(24) NOT NULL CHECK (limited_posture IN ('verified_no_license','unverified')),
            sequence_increment BIGINT NOT NULL DEFAULT 0 CHECK (sequence_increment >= 0),
            result_sequence BIGINT NOT NULL CHECK (result_sequence >= 0),
            decision VARCHAR(16) NOT NULL CHECK (decision IN ({$decisions})),
            error_code VARCHAR(64) NULL,
            state_reason VARCHAR(191) NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_event
            ON {$settlements} (order_id, transition)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_account
            ON {$settlements} (account_uuid, result_sequence)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_idem
            ON {$settlements} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_retention
            ON {$settlements} (retention_until)");

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$outbox} (
            event_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            event_version BIGINT NOT NULL,
            surface VARCHAR(16) NOT NULL CHECK (surface IN ('order','refund','stripe','license')),
            transition VARCHAR(24) NOT NULL,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            license_id BIGINT NULL,
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
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_outbox_due
            ON {$outbox} (dispatch_state, next_attempt_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_outbox_account
            ON {$outbox} (account_uuid, created_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_outbox_idem
            ON {$outbox} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_outbox_retention
            ON {$outbox} (retention_until)");

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$deliveries} (
            event_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
            idempotency_key {$key} NOT NULL UNIQUE,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            transition VARCHAR(24) NOT NULL,
            authority_sequence BIGINT NOT NULL,
            result_sequence BIGINT NOT NULL,
            envelope_digest VARCHAR(64) NOT NULL,
            delivered_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_deliveries_account
            ON {$deliveries} (account_uuid, delivered_at)");

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$runs} (
            run_uuid {$key} NOT NULL PRIMARY KEY,
            mode VARCHAR(8) NOT NULL CHECK (mode IN ('dry_run','apply')),
            started_at VARCHAR(32) NOT NULL,
            finished_at VARCHAR(32) NOT NULL,
            findings_total BIGINT NOT NULL DEFAULT 0,
            repairs_applied BIGINT NOT NULL DEFAULT 0,
            would_repair BIGINT NOT NULL DEFAULT 0,
            quarantined_new BIGINT NOT NULL DEFAULT 0,
            stable_quarantine BIGINT NOT NULL DEFAULT 0,
            converged INTEGER NOT NULL DEFAULT 0,
            result_handle VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$findings} (
            finding_uuid {$key} NOT NULL PRIMARY KEY,
            run_uuid {$key} NOT NULL,
            category VARCHAR(40) NOT NULL,
            classification VARCHAR(40) NOT NULL,
            severity VARCHAR(8) NOT NULL,
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            reason VARCHAR(191) NOT NULL,
            evidence_ref VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_findings_run
            ON {$findings} (run_uuid)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$repairs} (
            repair_uuid {$key} NOT NULL PRIMARY KEY,
            run_uuid {$key} NOT NULL,
            finding_uuid {$key} NOT NULL,
            category VARCHAR(40) NOT NULL,
            action VARCHAR(32) NOT NULL CHECK (action IN ('settle_bundle_refund','settle_bundle_chargeback','settle_bundle_revoke')),
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            evidence_ref VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_spec172_settlement_repairs_run
            ON {$repairs} (run_uuid)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$quarantine} (
            quarantine_uuid {$key} NOT NULL PRIMARY KEY,
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            reason VARCHAR(191) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            UNIQUE (entity_type, entity_ref, reason)
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

    /** Rollback is preservation-only: settlement journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $events = $this->table('wpuiai_spec172_settlement_schema_events');
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
 * Canonical EDD refund/chargeback/revoke truth adapter. Derives the refund scope,
 * amount, and event date EXCLUSIVELY from canonical EDD rows (order row + refund rows);
 * the caller can never supply a price, amount, scope, window, or date. Whole-order
 * scope requires exactly one complete whole-order refund row whose amount equals the
 * order total; any item-scoped (component) refund row fails closed with
 * COMPONENT_REFUND_UNSUPPORTED and any partial/unmapped truth fails closed with
 * REFUND_TRUTH_UNKNOWN.
 */
final class FocusaSpec172BundleRefundTruthAdapter
{
    public const SCHEMA = 'focusa.spec172.bundle_refund_truth.v1';
    public const REFUND_WINDOW_DAYS = 30;

    /** @return array{scope:string,amount_minor:int,event_date:string} */
    public function refundTruth(int $orderId, array $order): array
    {
        if ($orderId < 1) {
            throw new InvalidArgumentException('positive EDD order ID required');
        }
        if ((string) ($order['status'] ?? '') !== 'refunded') {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        $rows = $this->refundRows($orderId);
        if (count($rows) === 0) {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        foreach ($rows as $row) {
            if ($row['order_item_id'] !== null && (int) $row['order_item_id'] > 0) {
                throw new DomainException('COMPONENT_REFUND_UNSUPPORTED');
            }
            if ((string) $row['status'] !== 'complete') {
                throw new DomainException('REFUND_TRUTH_UNKNOWN');
            }
        }
        if (count($rows) !== 1) {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        $amountMinor = self::minorOf((string) ($rows[0]['amount'] ?? '0'));
        $orderTotalMinor = self::minorOf((string) ($order['total'] ?? '0'));
        if ($orderTotalMinor < 1 || $amountMinor !== $orderTotalMinor) {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        $eventDate = (string) ($rows[0]['date_created'] ?? '');
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($eventDate);
        return ['scope' => 'whole_order', 'amount_minor' => $amountMinor, 'event_date' => $eventDate];
    }

    /** @return array{event_date:string} */
    public function chargebackTruth(int $orderId): array
    {
        if ($orderId < 1) {
            throw new InvalidArgumentException('positive EDD order ID required');
        }
        foreach ($this->refundRows($orderId) as $row) {
            if ((string) $row['gateway'] === 'stripe' && in_array((string) $row['status'], ['disputed', 'lost'], true)) {
                $eventDate = (string) ($row['date_created'] ?? '');
                FocusaSpec172RefundDowngradeMigration::assertTimestamp($eventDate);
                return ['event_date' => $eventDate];
            }
        }
        throw new DomainException('CHARGEBACK_TRUTH_UNKNOWN');
    }

    /** @return array{event_date:string} */
    public function revokeTruth(array $order): array
    {
        if ((string) ($order['status'] ?? '') !== 'revoked') {
            throw new DomainException('REVOKE_TRUTH_UNKNOWN');
        }
        $eventDate = (string) ($order['date_updated'] ?? '');
        if ($eventDate === '') {
            $eventDate = (string) ($order['date_completed'] ?? '');
        }
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($eventDate);
        return ['event_date' => $eventDate];
    }

    /**
     * 30-day whole-order refund window: the canonical refund event date must be no later
     * than the order completion date plus REFUND_WINDOW_DAYS. Chargeback and revoke are
     * adverse authority events, never bounded by the customer refund window.
     */
    public static function assertWithinRefundWindow(string $completedAt, string $eventDate): void
    {
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($completedAt);
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($eventDate);
        $deadline = (new DateTimeImmutable($completedAt, new DateTimeZone('UTC')))
            ->modify('+' . self::REFUND_WINDOW_DAYS . ' days');
        if (new DateTimeImmutable($eventDate, new DateTimeZone('UTC')) > $deadline) {
            throw new DomainException('REFUND_WINDOW_EXPIRED');
        }
    }

    /** @return list<array{order_item_id:?int,amount:string,status:string,gateway:string,date_created:string}> */
    private function refundRows(int $orderId): array
    {
        $table = $this->prefix . 'edd_order_refunds';
        if (!$this->tableExists($table)) {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE order_id = :order ORDER BY id ASC");
        $statement->execute([':order' => $orderId]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        $result = [];
        foreach ($rows as $row) {
            $result[] = [
                'order_item_id' => isset($row['order_item_id']) && $row['order_item_id'] !== null ? (int) $row['order_item_id'] : null,
                'amount' => (string) ($row['amount'] ?? '0'),
                'status' => (string) ($row['status'] ?? ''),
                'gateway' => (string) ($row['gateway'] ?? ''),
                'date_created' => (string) ($row['date_created'] ?? ''),
            ];
        }
        return $result;
    }

    private function tableExists(string $table): bool
    {
        try {
            $this->db->query("SELECT 1 FROM {$table} LIMIT 1");
            return true;
        } catch (Throwable $error) {
            return false;
        }
    }

    /** USD money value to minor units, exactly; non-numeric or negative values fail closed. */
    private static function minorOf(string $amount): int
    {
        if (!is_numeric($amount) || (float) $amount < 0) {
            throw new DomainException('REFUND_TRUTH_UNKNOWN');
        }
        return (int) round(((float) $amount) * 100);
    }

    public function __construct(
        private PDO $db,
        private string $prefix = 'wp_',
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }
}

/** Server-owned HMAC-SHA256 envelope signer for the settlement outbox. */
final class FocusaSpec172SettlementEventSigner
{
    public const SIGNATURE_ALGORITHM = 'hmac_sha256.spec172.settlement_outbox.v1';
    public const SIGNATURE_PREFIX = 'sig_v1';
    public const KEY_ID = 'wpuiai.spec172.settlement_outbox.v1';

    public function __construct(
        private string $secret,
        private string $keyId = self::KEY_ID,
    ) {
        if (strlen($secret) < 16) {
            throw new InvalidArgumentException('server-owned signing secret required');
        }
    }

    public function keyId(): string
    {
        return $this->keyId;
    }

    /** @return array{signature:string,signing_key_id:string} */
    public function sign(string $canonicalPayload, string $digest): array
    {
        return [
            'signature' => self::SIGNATURE_PREFIX . '_' . hash_hmac('sha256', $canonicalPayload . "\n" . $digest, $this->secret),
            'signing_key_id' => $this->keyId,
        ];
    }

    public function verify(string $canonicalPayload, string $digest, string $signature, string $signingKeyId): void
    {
        if (!hash_equals($this->keyId, $signingKeyId)) {
            throw new DomainException('OUTBOX_SIGNING_KEY_UNKNOWN');
        }
        $expected = self::SIGNATURE_PREFIX . '_' . hash_hmac('sha256', $canonicalPayload . "\n" . $digest, $this->secret);
        if (!hash_equals($expected, $signature)) {
            throw new DomainException('OUTBOX_SIGNATURE_INVALID');
        }
    }
}

/**
 * Spec 172 Bundle refund/chargeback/revoke settler. Consumes one canonical adverse
 * event against one accepted composite Bundle projection and settles it EXACTLY ONCE:
 * both underlying Operator v1 grants are revoked together, the account's monotonic
 * authority sequence advances by exactly one, the transactional outbox row is appended
 * in the same transaction, and the still-verified account returns to
 * `verified_no_license` limited mode. Replays, duplicate adverse events, stale
 * reactivation attempts, sequence rollback, component refunds, and out-of-window
 * refunds never change state and never bump the sequence.
 */
final class FocusaSpec172RefundDowngradeSettler
{
    public const SCHEMA = 'focusa.spec172.refund_downgrade_settlement.v1';
    public const RESULT_SCHEMA = 'focusa.spec172.bundle_settlement.v1';
    public const VERSION = 1;
    public const BUNDLE_SKU = 'focusa_uiai_operator_bundle_lifetime_v1';
    public const BUNDLE_GRANTS = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'];
    public const BUNDLE_RESULT_SCHEMA = 'focusa.spec172.bundle_operator_lifetime_projection.v1';
    public const RETENTION_SECONDS = 7776000;
    public const REFUND_WINDOW_DAYS = 30;
    public const GRANTS_REVOKED = 2;

    /**
     * Canonical lifecycle transition matrix for the Bundle (addendum section 21): the
     * single authority for target license state, terminality, sequence effect, refresh
     * posture, and refund window. `adverse` events are settled by this settler; every
     * other transition can never run against a terminal Bundle.
     */
    public const TRANSITION_MATRIX = [
        'complete' => ['to_state' => 'active', 'sequence_increment' => 1, 'terminal' => false, 'adverse' => false, 'whole_order_only' => false, 'refund_window_days' => 0, 'refresh_posture' => 'allowed'],
        'refund' => ['to_state' => 'refunded', 'sequence_increment' => 1, 'terminal' => true, 'adverse' => true, 'whole_order_only' => true, 'refund_window_days' => 30, 'refresh_posture' => 'recovery_only'],
        'chargeback' => ['to_state' => 'refunded', 'sequence_increment' => 1, 'terminal' => true, 'adverse' => true, 'whole_order_only' => false, 'refund_window_days' => 0, 'refresh_posture' => 'recovery_only'],
        'revoke' => ['to_state' => 'revoked', 'sequence_increment' => 1, 'terminal' => true, 'adverse' => true, 'whole_order_only' => false, 'refund_window_days' => 0, 'refresh_posture' => 'recovery_only'],
        'unsuspend' => ['to_state' => 'active', 'sequence_increment' => 1, 'terminal' => false, 'adverse' => false, 'whole_order_only' => false, 'refund_window_days' => 0, 'refresh_posture' => 'allowed'],
    ];

    public const TERMINAL_STATES = ['refunded', 'revoked', 'expired', 'superseded', 'cancelled', 'denied'];

    private const FORBIDDEN_COMMERCE_FIELDS = [
        'price', 'amount', 'total', 'currency', 'scope', 'refund_scope', 'refund_amount',
        'refund_date', 'event_date', 'grants', 'features', 'limits', 'tier', 'node_limit',
        'activation_limit', 'commercial_rights', 'product_name', 'download_id', 'sku',
        'license_type', 'license_type_ref', 'price_version', 'family_digest',
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172RefundDowngradeMigration $schema,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private FocusaSpec172BundleRefundTruthAdapter $truth,
        private FocusaSpec172SettlementEventSigner $signer,
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
     * Settle one canonical adverse Bundle event. Required input:
     *   - order_id, customer_id, account_uuid: canonical references only
     *   - transition: refund | chargeback | revoke
     *   - request_id, idempotency_key
     * Optional:
     *   - authority_sequence: server ordinal used to detect out-of-order delivery
     *   - state_reason: bounded human-readable reason (no raw email or secrets)
     * Caller metadata never selects scope, amount, window, grant, price, or rights.
     */
    public function settle(array $input): array
    {
        $this->assertNoRawEmail($input);
        $this->assertNoClientCommerceFields($input);
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $orderId = (int) ($input['order_id'] ?? 0);
        $customerId = (int) ($input['customer_id'] ?? 0);
        $accountUuid = (string) ($input['account_uuid'] ?? '');
        if ($orderId < 1 || $customerId < 1) {
            throw new InvalidArgumentException('positive order and customer IDs required');
        }
        $this->assertUuid($accountUuid, 'account');
        $transition = (string) ($input['transition'] ?? '');
        $spec = self::TRANSITION_MATRIX[$transition] ?? null;
        if ($spec === null || ($spec['adverse'] ?? false) !== true) {
            throw new DomainException('EDD_TRANSITION_UNKNOWN');
        }

        try {
            $account = $this->accounts->findByUuid($accountUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        if ((int) $account['edd_customer_id'] !== $customerId) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }
        $projection = $this->loadBundleProjection($orderId, $customerId);
        if ($projection === null) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        $order = $this->edd->findOrderById($orderId);
        if ($order === null || (int) ($order['customer_id'] ?? 0) !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        $digest = $this->digest([
            'operation' => 'bundle_settlement',
            'transition' => $transition,
            'order_id' => $orderId,
            'customer_id' => $customerId,
            'account_uuid' => $accountUuid,
            'authority_sequence' => $input['authority_sequence'] ?? null,
        ]);
        $replay = $this->replayByIdempotency($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $now = $this->now();
        $reason = $this->boundedReason($input['state_reason'] ?? null);

        // Authority ordinal guard: a genuinely new event whose ordinal is not newer than
        // the account's highest sequence can never roll the sequence back.
        $authoritySequence = $input['authority_sequence'] ?? null;
        if ($authoritySequence !== null) {
            if (!is_int($authoritySequence) || $authoritySequence < 1) {
                throw new InvalidArgumentException('positive authority sequence required');
            }
            if ($authoritySequence <= (int) $account['highest_entitlement_sequence']) {
                return $this->record($input, $account, $projection, $order, $transition, 'none',
                    (string) $spec['to_state'], 0, 'denied', 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED',
                    $reason, $requestId, $idempotencyKey, $digest, $now);
            }
        }

        // Settle-once terminal guard: the first adverse event revokes both grants; every
        // later adverse event (same or different transition) is journaled replayed and
        // never changes state or bumps the sequence.
        $current = $this->currentEffectiveState($orderId);
        if (in_array($current, self::TERMINAL_STATES, true)) {
            return $this->record($input, $account, $projection, $order, $transition, $current,
                $current, 0, 'replayed', null, $reason, $requestId, $idempotencyKey, $digest, $now);
        }
        $alreadySettled = $this->findAppliedSettlement($orderId, $transition);
        if ($alreadySettled !== null) {
            return $this->record($input, $account, $projection, $order, $transition, 'active',
                (string) $alreadySettled['to_state'], 0, 'replayed', null, $reason, $requestId,
                $idempotencyKey, $digest, $now);
        }

        // Per-transition canonical EDD truth. Refunds are 30-day WHOLE-ORDER only;
        // chargeback and revoke are adverse authority events outside the refund window.
        $scope = 'not_applicable';
        $eventDate = $now;
        if ($transition === 'refund') {
            try {
                $truth = $this->truth->refundTruth($orderId, $order);
                $scope = (string) $truth['scope'];
                $eventDate = (string) $truth['event_date'];
                $this->truth->assertWithinRefundWindow((string) ($order['date_completed'] ?? ''), $eventDate);
            } catch (DomainException $error) {
                if ($error->getMessage() === 'REFUND_WINDOW_EXPIRED'
                    || $error->getMessage() === 'COMPONENT_REFUND_UNSUPPORTED') {
                    return $this->record($input, $account, $projection, $order, $transition, 'active',
                        'refunded', 0, 'denied', $error->getMessage(), $reason, $requestId,
                        $idempotencyKey, $digest, $now);
                }
                return $this->record($input, $account, $projection, $order, $transition, 'active',
                    'refunded', 0, 'denied', 'REFUND_TRUTH_UNKNOWN', $reason, $requestId,
                    $idempotencyKey, $digest, $now);
            }
        } elseif ($transition === 'chargeback') {
            try {
                $eventDate = (string) $this->truth->chargebackTruth($orderId)['event_date'];
            } catch (DomainException $error) {
                return $this->record($input, $account, $projection, $order, $transition, 'active',
                    'refunded', 0, 'denied', 'CHARGEBACK_TRUTH_UNKNOWN', $reason, $requestId,
                    $idempotencyKey, $digest, $now);
            }
        } else {
            try {
                $eventDate = (string) $this->truth->revokeTruth($order)['event_date'];
            } catch (DomainException $error) {
                return $this->record($input, $account, $projection, $order, $transition, 'active',
                    'revoked', 0, 'denied', 'REVOKE_TRUTH_UNKNOWN', $reason, $requestId,
                    $idempotencyKey, $digest, $now);
            }
        }

        $increment = (int) $spec['sequence_increment'];
        $resultSequence = (int) $account['highest_entitlement_sequence'] + $increment;
        return $this->record($input, $account, $projection, $order, $transition, 'active',
            (string) $spec['to_state'], $increment, 'applied', null, $reason, $requestId,
            $idempotencyKey, $digest, $now, $resultSequence, $scope, $eventDate);
    }

    /**
     * Stale-cache reactivation guard: a `complete`/`unsuspend`-shaped event that would
     * flip a terminal Bundle back to active fails closed with
     * LICENSE_TERMINAL_REACTIVATION_DENIED and never changes state or bumps the sequence.
     * Chargeback/revoke can therefore never be undone by a stale cache delivery.
     */
    public function guardReactivation(array $input): array
    {
        $this->assertNoRawEmail($input);
        $this->assertNoClientCommerceFields($input);
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $orderId = (int) ($input['order_id'] ?? 0);
        $customerId = (int) ($input['customer_id'] ?? 0);
        $accountUuid = (string) ($input['account_uuid'] ?? '');
        if ($orderId < 1 || $customerId < 1) {
            throw new InvalidArgumentException('positive order and customer IDs required');
        }
        $this->assertUuid($accountUuid, 'account');
        $transition = (string) ($input['transition'] ?? '');
        $spec = self::TRANSITION_MATRIX[$transition] ?? null;
        if ($spec === null || ($spec['adverse'] ?? false) === true) {
            throw new DomainException('EDD_TRANSITION_UNKNOWN');
        }
        $account = $this->accounts->findByUuid($accountUuid);
        $projection = $this->loadBundleProjection($orderId, $customerId);
        if ($projection === null) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        $digest = $this->digest([
            'operation' => 'bundle_reactivation_guard',
            'transition' => $transition,
            'order_id' => $orderId,
            'customer_id' => $customerId,
            'account_uuid' => $accountUuid,
        ]);
        $replay = $this->replayByIdempotency($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }
        $now = $this->now();
        $current = $this->currentEffectiveState($orderId);
        if (in_array($current, self::TERMINAL_STATES, true)) {
            return $this->record($input, $account, $projection, null, $transition, $current,
                $current, 0, 'denied', 'LICENSE_TERMINAL_REACTIVATION_DENIED', null, $requestId,
                $idempotencyKey, $digest, $now);
        }
        // Non-terminal: the guard performs no state change and journals nothing; an
        // active Bundle is simply not a reactivation candidate.
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'allowed',
            'transition' => $transition,
            'order_id' => $orderId,
            'customer_id' => $customerId,
            'account_id' => $accountUuid,
            'effective_state' => $current,
            'sequence_increment' => 0,
            'result_sequence' => (int) $account['highest_entitlement_sequence'],
            'created_at' => $now,
        ];
    }

    // ── Queries (bounded, public-safe) ─────────────────────────────────

    /** Effective entitlement state of the Bundle order: active | terminal state | none. */
    public function currentEffectiveState(int $orderId): string
    {
        $table = $this->schema->table('wpuiai_spec172_settlements');
        $statement = $this->db->prepare("SELECT to_state FROM {$table}
            WHERE order_id = :order AND decision = 'applied'
            ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
        $statement->execute([':order' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            $projection = $this->loadBundleProjectionById($orderId);
            return $projection === null ? 'none' : 'active';
        }
        return (string) $row['to_state'];
    }

    /** Paid grants are active only while an accepted projection has no terminal settlement. */
    public function paidGrantsActive(int $orderId): bool
    {
        $state = $this->currentEffectiveState($orderId);
        return $state === 'active';
    }

    public function settlementCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlements');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function appliedSettlementCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlements');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table} WHERE decision = 'applied'")->fetchColumn();
    }

    public function settlementForOrder(int $orderId, string $transition): ?array
    {
        if (!array_key_exists($transition, self::TRANSITION_MATRIX)) {
            return null;
        }
        $table = $this->schema->table('wpuiai_spec172_settlements');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE order_id = :order AND transition = :transition AND decision = 'applied'
            ORDER BY result_sequence DESC LIMIT 1");
        $statement->execute([':order' => $orderId, ':transition' => $transition]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── Journaling ─────────────────────────────────────────────────────

    private function record(
        array $input,
        array $account,
        array $projection,
        ?array $order,
        string $transition,
        string $fromState,
        string $toState,
        int $increment,
        string $decision,
        ?string $errorCode,
        ?string $reason,
        string $requestId,
        string $idempotencyKey,
        string $digest,
        string $now,
        ?int $resultSequence = null,
        string $scope = 'not_applicable',
        string $eventDate = '',
    ): array {
        $limitedPosture = $this->limitedPosture((string) $projection['registration_uuid']);
        $resultSequence ??= (int) $account['highest_entitlement_sequence'];
        $licenseId = isset($projection['edd_license_id']) ? (int) $projection['edd_license_id'] : null;

        $preserved = [
            'customers' => $this->countTable('edd_customers'),
            'orders' => $this->countTable('edd_orders'),
            'order_items' => $this->countTable('edd_order_items'),
            'licenses' => $this->countTable('edd_licenses'),
            'refunds' => $this->countTable('edd_order_refunds'),
            'projections' => $this->countTable('wpuiai_license_type_projections'),
            'settlement_journal' => $this->settlementCount(),
        ];

        $base = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => $decision,
            'settlement_uuid' => self::opaqueToken('stl_'),
            'transition' => $transition,
            'order_id' => (int) $projection['order_id'],
            'order_item_id' => (int) $projection['order_item_id'],
            'customer_id' => (int) $projection['customer_id'],
            'account_id' => (string) $account['account_uuid'],
            'projection_id' => (string) $projection['projection_key'],
            'edd_license_id' => $licenseId,
            'license_type_ref' => self::BUNDLE_SKU,
            'grants_revoked' => $decision === 'applied' ? self::GRANTS_REVOKED : 0,
            'grants' => self::BUNDLE_GRANTS,
            'from_state' => $fromState,
            'to_state' => $toState,
            'scope' => $scope,
            'refund_window_days' => $transition === 'refund' ? self::REFUND_WINDOW_DAYS : 0,
            'limited_posture' => $limitedPosture,
            'paid_grants_active' => $decision === 'applied' ? false : $this->paidGrantsActive((int) $projection['order_id']),
            'refresh_posture' => (string) self::TRANSITION_MATRIX[$transition]['refresh_posture'],
            'sequence' => (int) $account['highest_entitlement_sequence'],
            'result_sequence' => $resultSequence,
            'sequence_increment' => $increment,
            'error_code' => $errorCode,
            'state_reason' => $reason,
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'created_at' => $now,
            'preserved' => $preserved,
        ];

        $table = $this->schema->table('wpuiai_spec172_settlements');
        $outboxTable = $this->schema->table('wpuiai_spec172_settlement_outbox');
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';
        $stored = $base;
        unset($stored['preserved']);
        $payload = json_encode($stored, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);

        $this->db->beginTransaction();
        try {
            if ($increment > 0) {
                $statement = $this->db->prepare("UPDATE {$accountsTable}
                    SET highest_entitlement_sequence = :next, updated_at = :updated
                    WHERE account_uuid = :uuid AND highest_entitlement_sequence < :guard");
                $statement->execute([
                    ':next' => $resultSequence,
                    ':updated' => $now,
                    ':uuid' => (string) $account['account_uuid'],
                    ':guard' => $resultSequence,
                ]);
                if ($statement->rowCount() !== 1) {
                    throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
                }
            }
            $settlementStatement = $this->db->prepare("INSERT INTO {$table}
                (settlement_uuid, account_uuid, edd_customer_id, order_id, projection_key,
                 license_type_ref, transition, scope, from_state, to_state, grants_revoked,
                 limited_posture, sequence_increment, result_sequence, decision, error_code,
                 state_reason, result_payload, request_id, idempotency_key, request_digest,
                 created_at, retention_until)
                VALUES (:settlement, :account, :customer, :order, :projection,
                        :license_type, :transition, :scope, :from_state, :to_state, :grants_revoked,
                        :limited_posture, :increment, :result_sequence, :decision, :error_code,
                        :state_reason, :payload, :request_id, :idempotency_key, :request_digest,
                        :created_at, :retention_until)");
            $settlementStatement->execute([
                ':settlement' => $base['settlement_uuid'],
                ':account' => (string) $account['account_uuid'],
                ':customer' => (int) $account['edd_customer_id'],
                ':order' => (int) $projection['order_id'],
                ':projection' => (string) $projection['projection_key'],
                ':license_type' => self::BUNDLE_SKU,
                ':transition' => $transition,
                ':scope' => $scope,
                ':from_state' => $fromState,
                ':to_state' => $toState,
                ':grants_revoked' => $base['grants_revoked'],
                ':limited_posture' => $limitedPosture,
                ':increment' => $increment,
                ':result_sequence' => $resultSequence,
                ':decision' => $decision,
                ':error_code' => $errorCode,
                ':state_reason' => $reason,
                ':payload' => $payload,
                ':request_id' => $requestId,
                ':idempotency_key' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created_at' => $now,
                ':retention_until' => self::plusSeconds($now, $this->retention),
            ]);

            if ($decision === 'applied') {
                $this->appendOutbox($outboxTable, $base, $payload, $digest, $now, $order);
            }
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
        return $base;
    }

    private function appendOutbox(string $outboxTable, array $base, string $payload, string $digest, string $now, ?array $order): void
    {
        $surface = match ($base['transition']) {
            'refund' => 'refund',
            'chargeback' => 'stripe',
            'revoke' => 'order',
            default => 'order',
        };
        // The envelope digest and HMAC signature cover the canonical payload bytes; the
        // request digest is journaled separately for idempotency.
        $payloadDigest = hash('sha256', $payload);
        $signed = $this->signer->sign($payload, $payloadDigest);
        $eventUuid = self::opaqueToken('obx_');
        $statement = $this->db->prepare("INSERT INTO {$outboxTable}
            (event_uuid, event_type, event_version, surface, transition, account_uuid,
             edd_customer_id, order_id, license_id, authority_sequence, result_sequence,
             payload, envelope_digest, signature, signing_key_id, dispatch_state, attempts,
             last_attempt_at, next_attempt_at, last_error, request_id, idempotency_key,
             created_at, retention_until)
            VALUES (:event, 'bundle_settlement', 1, :surface, :transition, :account,
                    :customer, :order, :license, :authority, :result,
                    :payload, :digest, :signature, :key_id, 'pending', 0,
                    NULL, :next_attempt, NULL, :request, :idempotency,
                    :created, :retention)");
        $statement->execute([
            ':event' => $eventUuid,
            ':surface' => $surface,
            ':transition' => (string) $base['transition'],
            ':account' => (string) $base['account_id'],
            ':customer' => (int) $base['customer_id'],
            ':order' => (int) $base['order_id'],
            ':license' => $base['edd_license_id'],
            ':authority' => (int) $base['sequence'],
            ':result' => (int) $base['result_sequence'],
            ':payload' => $payload,
            ':digest' => $payloadDigest,
            ':signature' => (string) $signed['signature'],
            ':key_id' => (string) $signed['signing_key_id'],
            ':next_attempt' => $now,
            ':request' => (string) $base['request_id'],
            ':idempotency' => (string) $base['idempotency_key'],
            ':created' => $now,
            ':retention' => self::plusSeconds($now, $this->retention),
        ]);
    }

    /** The Bundle settlement outbox event uuid is exposed (bounded) for dispatch proof. */
    public function latestOutboxEvent(): ?array
    {
        $table = $this->schema->table('wpuiai_spec172_settlement_outbox');
        $statement = $this->db->query("SELECT event_uuid, transition, dispatch_state, authority_sequence, result_sequence
            FROM {$table} ORDER BY created_at DESC LIMIT 1");
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function limitedPosture(string $registrationUuid): string
    {
        try {
            $registration = $this->registrations->findByUuid($registrationUuid);
        } catch (OutOfBoundsException $error) {
            return 'unverified';
        }
        if ((string) ($registration['verification_state'] ?? '') === 'mailbox_verified'
            && ($registration['verified_at'] ?? null) !== null) {
            return 'verified_no_license';
        }
        return 'unverified';
    }

    private function loadBundleProjection(int $orderId, int $customerId): ?array
    {
        $table = $this->prefix . 'wpuiai_license_type_projections';
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE order_id = :order AND customer_id = :customer AND product_code = :sku AND status = 'active'
            ORDER BY sequence DESC, created_at DESC LIMIT 1");
        $statement->execute([':order' => $orderId, ':customer' => $customerId, ':sku' => self::BUNDLE_SKU]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function loadBundleProjectionById(int $orderId): ?array
    {
        $table = $this->prefix . 'wpuiai_license_type_projections';
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE order_id = :order AND product_code = :sku AND status = 'active' LIMIT 1");
        $statement->execute([':order' => $orderId, ':sku' => self::BUNDLE_SKU]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findAppliedSettlement(int $orderId, string $transition): ?array
    {
        $table = $this->schema->table('wpuiai_spec172_settlements');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE order_id = :order AND transition = :transition AND decision = 'applied' LIMIT 1");
        $statement->execute([':order' => $orderId, ':transition' => $transition]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function replayByIdempotency(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_spec172_settlements');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        $decision = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        // The replay itself performs no state change and never bumps the sequence again;
        // the settled result sequence stays exactly as first recorded.
        $decision['decision'] = 'replayed';
        $decision['sequence_increment'] = 0;
        $decision['settlement_uuid'] = (string) $row['settlement_uuid'];
        return $decision;
    }

    private function countTable(string $name): int
    {
        try {
            return (int) $this->db->query("SELECT COUNT(*) FROM {$this->prefix}{$name}")->fetchColumn();
        } catch (Throwable $error) {
            return 0;
        }
    }

    // ── Guards ─────────────────────────────────────────────────────────

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
        return hash('sha256', FocusaSpec172RefundDowngradeMigration::encodeCanonical($value));
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($now);
        return $now;
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
}

/**
 * Exactly-once settlement outbox dispatcher. Delivers each pending envelope at most
 * once: delivery and the dispatched mark commit in one transaction and the delivery
 * ledger is UNIQUE on the envelope idempotency key, so a crash before the dispatch
 * commit leaves the row pending (redelivery re-applies exactly once) and a crash after
 * the commit never redelivers. Tampered envelopes (digest or signature mismatch) fail
 * closed into the dead-letter state.
 */
final class FocusaSpec172SettlementDispatcher
{
    public const SCHEMA = 'focusa.spec172.refund_downgrade_settlement.v1';
    public const MAX_ATTEMPTS = 5;
    public const RETRY_BASE_SECONDS = 60;

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172RefundDowngradeMigration $schema,
        private FocusaSpec172SettlementEventSigner $signer,
        callable $clock,
        private string $prefix = 'wp_',
        private int $maxAttempts = self::MAX_ATTEMPTS,
        private int $retryBaseSeconds = self::RETRY_BASE_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /** Deliver at most one pending due envelope. Returns null when nothing is due. */
    public function dispatchOne(): ?array
    {
        $outbox = $this->schema->table('wpuiai_spec172_settlement_outbox');
        $deliveries = $this->schema->table('wpuiai_spec172_settlement_deliveries');
        $now = $this->now();
        $statement = $this->db->prepare("SELECT * FROM {$outbox}
            WHERE dispatch_state = 'pending' AND next_attempt_at <= :now
            ORDER BY created_at ASC LIMIT 1");
        $statement->execute([':now' => $now]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        $eventUuid = (string) $row['event_uuid'];
        $payload = (string) $row['payload'];
        $digest = hash('sha256', $payload);
        $reportedDigest = (string) $row['envelope_digest'];

        $this->db->beginTransaction();
        try {
            if (!hash_equals($reportedDigest, $digest)) {
                $this->markFailed($outbox, $eventUuid, $now, 'OUTBOX_DIGEST_INVALID');
                $this->db->commit();
                return ['event_uuid' => $eventUuid, 'decision' => 'dead_letter', 'error_code' => 'OUTBOX_DIGEST_INVALID'];
            }
            try {
                $this->signer->verify($payload, $digest, (string) $row['signature'], (string) $row['signing_key_id']);
            } catch (DomainException $error) {
                $this->markFailed($outbox, $eventUuid, $now, $error->getMessage());
                $this->db->commit();
                return ['event_uuid' => $eventUuid, 'decision' => 'dead_letter', 'error_code' => $error->getMessage()];
            }

            $deliveryStatement = $this->db->prepare("INSERT INTO {$deliveries}
                (event_uuid, idempotency_key, account_uuid, edd_customer_id, event_type,
                 transition, authority_sequence, result_sequence, envelope_digest, delivered_at)
                SELECT :event, :idem, :account, :customer, :type, :transition,
                       :authority, :result, :digest, :delivered
                WHERE NOT EXISTS (SELECT 1 FROM {$deliveries} WHERE idempotency_key = :idem)");
            $deliveryStatement->execute([
                ':event' => $eventUuid,
                ':idem' => (string) $row['idempotency_key'],
                ':account' => (string) $row['account_uuid'],
                ':customer' => (int) $row['edd_customer_id'],
                ':type' => (string) $row['event_type'],
                ':transition' => (string) $row['transition'],
                ':authority' => (int) $row['authority_sequence'],
                ':result' => (int) $row['result_sequence'],
                ':digest' => $digest,
                ':delivered' => $now,
            ]);
            $delivered = $deliveryStatement->rowCount() === 1;

            $mark = $this->db->prepare("UPDATE {$outbox}
                SET dispatch_state = 'dispatched', attempts = attempts + 1, last_attempt_at = :now, last_error = NULL
                WHERE event_uuid = :event");
            $mark->execute([':now' => $now, ':event' => $eventUuid]);
            $this->db->commit();
            return [
                'event_uuid' => $eventUuid,
                'decision' => 'dispatched',
                'delivered' => $delivered,
                'delivered_at' => $now,
                'transition' => (string) $row['transition'],
                'authority_sequence' => (int) $row['authority_sequence'],
                'result_sequence' => (int) $row['result_sequence'],
            ];
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    public function pendingCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlement_outbox');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table} WHERE dispatch_state = 'pending'")->fetchColumn();
    }

    public function dispatchedCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlement_outbox');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table} WHERE dispatch_state = 'dispatched'")->fetchColumn();
    }

    public function deadLetterCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlement_outbox');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table} WHERE dispatch_state = 'dead_letter'")->fetchColumn();
    }

    public function deliveryCount(): int
    {
        $table = $this->schema->table('wpuiai_spec172_settlement_deliveries');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    private function markFailed(string $outbox, string $eventUuid, string $now, string $errorCode): void
    {
        $statement = $this->db->prepare("UPDATE {$outbox}
            SET dispatch_state = 'dead_letter', attempts = attempts + 1, last_attempt_at = :now, last_error = :error
            WHERE event_uuid = :event");
        $statement->execute([':now' => $now, ':error' => $errorCode, ':event' => $eventUuid]);
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($now);
        return $now;
    }
}

/**
 * Spec 172 settlement reconciler. Compares canonical EDD truth (refunded/revoked orders
 * and Stripe dispute rows) against the accepted Bundle projection journal and the
 * settlement journal; detects missing settlements and repairs them evidence-safe from
 * canonical EDD truth (never from client input). Dry-run applies nothing; apply is
 * idempotent and converges — a second apply run repairs zero. Ambiguous or
 * fail-closed records (component refunds, unknown truth, no account link) are
 * quarantined with an exact bounded reason.
 */
final class FocusaSpec172SettlementReconciler
{
    public const SCHEMA = 'focusa.spec172.refund_downgrade_settlement.v1';
    public const MODES = ['dry_run', 'apply'];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172RefundDowngradeMigration $schema,
        private FocusaSpec172RefundDowngradeSettler $settler,
        private FocusaSpec172BundleRefundTruthAdapter $truth,
        callable $clock,
        private string $prefix = 'wp_',
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    public function run(string $mode = 'dry_run'): array
    {
        if (!in_array($mode, self::MODES, true)) {
            throw new DomainException('RECONCILIATION_MODE_UNKNOWN');
        }
        $apply = $mode === 'apply';
        $now = $this->now();
        $state = [
            'apply' => $apply,
            'findings' => [],
            'repairs' => [],
            'quarantine' => [],
            'applied' => 0,
            'would_repair' => 0,
            'quarantine_new' => 0,
            'stable' => 0,
        ];

        $this->reconcileAdverseOrders($state);
        $this->reconcileChargebacks($state);

        $finishedAt = $this->now();
        $converged = ($state['would_repair'] - ($apply ? $state['applied'] : 0)) === 0;
        $runUuid = 'run_' . bin2hex(random_bytes(16));
        $report = [
            'schema' => self::SCHEMA,
            'mode' => $mode,
            'run_uuid' => $runUuid,
            'started_at' => $now,
            'finished_at' => $finishedAt,
            'summary' => [
                'findings_total' => count($state['findings']),
                'repairable' => $state['would_repair'],
                'repairs_applied' => $state['applied'],
                'would_repair' => $state['would_repair'],
                'quarantined_new' => $state['quarantine_new'],
                'stable_quarantine' => $state['stable'],
                'converged' => $converged,
            ],
            'findings' => $state['findings'],
            'repairs' => $state['repairs'],
            'quarantine' => $state['quarantine'],
            'result_handle' => $this->reportHandle($mode, $state['findings'], $state['repairs'], $state['quarantine']),
        ];
        $this->persistRun($report);
        return $report;
    }

    /** Canonical refunded/revoked Bundle orders missing an applied settlement are repaired. */
    private function reconcileAdverseOrders(array &$state): void
    {
        $ordersTable = $this->prefix . 'edd_orders';
        $projectionsTable = $this->prefix . 'wpuiai_license_type_projections';
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';
        $settlementsTable = $this->schema->table('wpuiai_spec172_settlements');
        $statement = $this->db->prepare("SELECT o.id AS order_id, o.status AS order_status, o.customer_id
            FROM {$ordersTable} o
            JOIN {$projectionsTable} p
              ON p.order_id = o.id AND p.customer_id = o.customer_id
             AND p.product_code = :sku AND p.status = 'active'
            WHERE o.status IN ('refunded','revoked')");
        $statement->execute([':sku' => FocusaSpec172RefundDowngradeSettler::BUNDLE_SKU]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        foreach ($rows as $row) {
            $orderId = (int) $row['order_id'];
            $customerId = (int) $row['customer_id'];
            $transition = (string) $row['order_status'] === 'refunded' ? 'refund' : 'revoke';
            $existing = $this->db->prepare("SELECT 1 FROM {$settlementsTable}
                WHERE order_id = :order AND transition = :transition AND decision = 'applied' LIMIT 1");
            $existing->execute([':order' => $orderId, ':transition' => $transition]);
            if ($existing->fetchColumn() !== false) {
                continue;
            }
            $accountUuid = $this->accountUuidForCustomer($customerId, $accountsTable);
            if ($accountUuid === null) {
                $this->quarantine($state, 'missing_account_link', 'order', (string) $orderId, null,
                    'EDD_ORDER_UNVERIFIED verified account link required before settlement', 'edd_orders');
                continue;
            }
            // Evidence-safe repair requires canonical truth that would APPLY. Component
            // refunds, out-of-window refunds, and unknown/absent refund truth quarantine
            // stably (exact bounded reason) and are never counted as repairable.
            $order = $this->db->prepare("SELECT * FROM {$ordersTable} WHERE id = :order LIMIT 1");
            $order->execute([':order' => $orderId]);
            $orderRow = $order->fetch(PDO::FETCH_ASSOC);
            if ($orderRow === false) {
                $this->quarantine($state, 'settlement_denied', 'order', (string) $orderId, $accountUuid,
                    'EDD_ORDER_UNVERIFIED', 'edd_orders');
                continue;
            }
            try {
                if ($transition === 'refund') {
                    $refundTruth = $this->truth->refundTruth($orderId, $orderRow);
                    $this->truth->assertWithinRefundWindow((string) ($orderRow['date_completed'] ?? ''), (string) $refundTruth['event_date']);
                } else {
                    $this->truth->revokeTruth($orderRow);
                }
            } catch (DomainException $error) {
                $this->quarantine($state, 'settlement_denied', 'order', (string) $orderId, $accountUuid,
                    (string) $error->getMessage(), 'spec172_settlements');
                $state['stable']++;
                continue;
            }
            $this->finding($state, 'missing_settlement', 'order', (string) $orderId, $accountUuid,
                $transition === 'refund' ? 'canonical refunded Bundle order missing settlement' : 'canonical revoked Bundle order missing settlement',
                'edd_orders');
            $state['would_repair']++;
            if (!$state['apply']) {
                continue;
            }
            $result = $this->settler->settle([
                'order_id' => $orderId,
                'customer_id' => $customerId,
                'account_uuid' => $accountUuid,
                'transition' => $transition,
                'request_id' => 'reconcile.spec172.' . $orderId . '.' . $transition,
                'idempotency_key' => 'reconcile.spec172.' . $orderId . '.' . $transition . '.' . $customerId,
            ]);
            if (($result['decision'] ?? '') === 'applied') {
                $state['applied']++;
                $state['repairs'][] = [
                    'repair_uuid' => 'rpr_' . bin2hex(random_bytes(16)),
                    'action' => $transition === 'refund' ? 'settle_bundle_refund' : 'settle_bundle_revoke',
                    'entity_type' => 'order',
                    'entity_ref' => (string) $orderId,
                    'account_uuid' => $accountUuid,
                    'evidence_ref' => (string) $result['settlement_uuid'],
                ];
            } else {
                $this->quarantine($state, 'settlement_denied', 'order', (string) $orderId, $accountUuid,
                    (string) ($result['error_code'] ?? 'SETTLEMENT_DENIED'), 'spec172_settlements');
            }
        }
    }

    /** Canonical Stripe dispute rows (lost) for Bundle orders missing a chargeback settlement. */
    private function reconcileChargebacks(array &$state): void
    {
        $refundsTable = $this->prefix . 'edd_order_refunds';
        $projectionsTable = $this->prefix . 'wpuiai_license_type_projections';
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';
        $settlementsTable = $this->schema->table('wpuiai_spec172_settlements');
        try {
            $statement = $this->db->prepare("SELECT r.order_id, r.status, r.gateway, r.customer_id
                FROM {$refundsTable} r
                JOIN {$projectionsTable} p
                  ON p.order_id = r.order_id AND p.customer_id = r.customer_id
                 AND p.product_code = :sku AND p.status = 'active'
                WHERE r.status IN ('disputed','lost') AND r.gateway = 'stripe'");
        } catch (Throwable $error) {
            return; // canonical refund table absent: no chargeback truth to reconcile
        }
        $statement->execute([':sku' => FocusaSpec172RefundDowngradeSettler::BUNDLE_SKU]);
        $seen = [];
        foreach ($statement->fetchAll(PDO::FETCH_ASSOC) as $row) {
            $orderId = (int) $row['order_id'];
            if (isset($seen[$orderId])) {
                continue;
            }
            $seen[$orderId] = true;
            $customerId = (int) $row['customer_id'];
            $existing = $this->db->prepare("SELECT 1 FROM {$settlementsTable}
                WHERE order_id = :order AND transition = 'chargeback' AND decision = 'applied' LIMIT 1");
            $existing->execute([':order' => $orderId]);
            if ($existing->fetchColumn() !== false) {
                continue;
            }
            $accountUuid = $this->accountUuidForCustomer($customerId, $accountsTable);
            if ($accountUuid === null) {
                $this->quarantine($state, 'missing_account_link', 'order', (string) $orderId, null,
                    'EDD_ORDER_UNVERIFIED verified account link required before chargeback settlement', 'edd_orders');
                continue;
            }
            $this->finding($state, 'missing_settlement', 'order', (string) $orderId, $accountUuid,
                'canonical lost Stripe dispute missing chargeback settlement', 'edd_order_refunds');
            $state['would_repair']++;
            if (!$state['apply']) {
                continue;
            }
            $result = $this->settler->settle([
                'order_id' => $orderId,
                'customer_id' => $customerId,
                'account_uuid' => $accountUuid,
                'transition' => 'chargeback',
                'request_id' => 'reconcile.spec172.' . $orderId . '.chargeback',
                'idempotency_key' => 'reconcile.spec172.' . $orderId . '.chargeback.' . $customerId,
            ]);
            if (($result['decision'] ?? '') === 'applied') {
                $state['applied']++;
                $state['repairs'][] = [
                    'repair_uuid' => 'rpr_' . bin2hex(random_bytes(16)),
                    'action' => 'settle_bundle_chargeback',
                    'entity_type' => 'order',
                    'entity_ref' => (string) $orderId,
                    'account_uuid' => $accountUuid,
                    'evidence_ref' => (string) $result['settlement_uuid'],
                ];
            } else {
                $this->quarantine($state, 'settlement_denied', 'order', (string) $orderId, $accountUuid,
                    (string) ($result['error_code'] ?? 'SETTLEMENT_DENIED'), 'spec172_settlements');
            }
        }
    }

    private function accountUuidForCustomer(int $customerId, string $accountsTable): ?string
    {
        $statement = $this->db->prepare("SELECT account_uuid FROM {$accountsTable} WHERE edd_customer_id = :customer LIMIT 1");
        $statement->execute([':customer' => $customerId]);
        $value = $statement->fetchColumn();
        return $value === false ? null : (string) $value;
    }

    private function finding(array &$state, string $category, string $entityType, string $entityRef, ?string $accountUuid, string $reason, string $evidenceRef): void
    {
        $state['findings'][] = [
            'finding_uuid' => 'fnd_' . bin2hex(random_bytes(16)),
            'category' => $category,
            'classification' => 'missing',
            'severity' => 'warning',
            'entity_type' => $entityType,
            'entity_ref' => $entityRef,
            'account_uuid' => $accountUuid,
            'reason' => $reason,
            'evidence_ref' => $evidenceRef,
        ];
    }

    private function quarantine(array &$state, string $category, string $entityType, string $entityRef, ?string $accountUuid, string $reason, string $evidenceRef): void
    {
        $state['quarantine'][] = [
            'quarantine_uuid' => 'qtn_' . bin2hex(random_bytes(16)),
            'entity_type' => $entityType,
            'entity_ref' => $entityRef,
            'account_uuid' => $accountUuid,
            'reason' => $reason,
            'created_at' => $this->now(),
        ];
        $state['quarantine_new']++;
    }

    private function reportHandle(string $mode, array $findings, array $repairs, array $quarantine): string
    {
        return hash('sha256', FocusaSpec172RefundDowngradeMigration::encodeCanonical([
            'mode' => $mode,
            'findings' => $findings,
            'repairs' => $repairs,
            'quarantine' => $quarantine,
        ]));
    }

    private function persistRun(array $report): void
    {
        $runs = $this->schema->table('wpuiai_spec172_settlement_runs');
        $findings = $this->schema->table('wpuiai_spec172_settlement_findings');
        $repairs = $this->schema->table('wpuiai_spec172_settlement_repairs');
        $quarantine = $this->schema->table('wpuiai_spec172_settlement_quarantine');
        $statement = $this->db->prepare("INSERT INTO {$runs}
            (run_uuid, mode, started_at, finished_at, findings_total, repairs_applied,
             would_repair, quarantined_new, stable_quarantine, converged, result_handle,
             migration_provenance)
            VALUES (:run, :mode, :started, :finished, :findings, :repairs, :would,
                    :quarantine_new, :stable, :converged, :handle, :provenance)");
        $statement->execute([
            ':run' => (string) $report['run_uuid'],
            ':mode' => (string) $report['mode'],
            ':started' => (string) $report['started_at'],
            ':finished' => (string) $report['finished_at'],
            ':findings' => (int) $report['summary']['findings_total'],
            ':repairs' => (int) $report['summary']['repairs_applied'],
            ':would' => (int) $report['summary']['would_repair'],
            ':quarantine_new' => (int) $report['summary']['quarantined_new'],
            ':stable' => (int) $report['summary']['stable_quarantine'],
            ':converged' => (int) $report['summary']['converged'],
            ':handle' => (string) $report['result_handle'],
            ':provenance' => FocusaSpec172RefundDowngradeMigration::encodeCanonical(['source' => 'spec172_settlement_reconciler']),
        ]);
        $findingStatement = $this->db->prepare("INSERT INTO {$findings}
            (finding_uuid, run_uuid, category, classification, severity, entity_type, entity_ref,
             account_uuid, reason, evidence_ref, created_at)
            VALUES (:uuid, :run, :category, :classification, :severity, :entity, :ref,
                    :account, :reason, :evidence, :created)");
        foreach ($report['findings'] as $finding) {
            $findingStatement->execute([
                ':uuid' => (string) $finding['finding_uuid'],
                ':run' => (string) $report['run_uuid'],
                ':category' => (string) $finding['category'],
                ':classification' => (string) $finding['classification'],
                ':severity' => (string) $finding['severity'],
                ':entity' => (string) $finding['entity_type'],
                ':ref' => (string) $finding['entity_ref'],
                ':account' => $finding['account_uuid'],
                ':reason' => (string) $finding['reason'],
                ':evidence' => (string) $finding['evidence_ref'],
                ':created' => (string) $report['started_at'],
            ]);
        }
        $repairStatement = $this->db->prepare("INSERT INTO {$repairs}
            (repair_uuid, run_uuid, finding_uuid, category, action, entity_type, entity_ref,
             account_uuid, evidence_ref, created_at)
            VALUES (:uuid, :run, :finding, :category, :action, :entity, :ref,
                    :account, :evidence, :created)");
        foreach ($report['repairs'] as $repair) {
            $repairStatement->execute([
                ':uuid' => (string) $repair['repair_uuid'],
                ':run' => (string) $report['run_uuid'],
                ':finding' => (string) $repair['finding_uuid'],
                ':category' => 'missing_settlement',
                ':action' => (string) $repair['action'],
                ':entity' => (string) $repair['entity_type'],
                ':ref' => (string) $repair['entity_ref'],
                ':account' => $repair['account_uuid'],
                ':evidence' => (string) $repair['evidence_ref'],
                ':created' => (string) $report['started_at'],
            ]);
        }
        $quarantineStatement = $this->db->prepare("INSERT INTO {$quarantine}
            (quarantine_uuid, entity_type, entity_ref, account_uuid, reason, created_at)
            SELECT :uuid, :entity, :ref, :account, :reason, :created
            WHERE NOT EXISTS (SELECT 1 FROM {$quarantine}
                WHERE entity_type = :entity AND entity_ref = :ref AND reason = :reason)");
        foreach ($report['quarantine'] as $item) {
            $quarantineStatement->execute([
                ':uuid' => (string) $item['quarantine_uuid'],
                ':entity' => (string) $item['entity_type'],
                ':ref' => (string) $item['entity_ref'],
                ':account' => $item['account_uuid'],
                ':reason' => (string) $item['reason'],
                ':created' => (string) $item['created_at'],
            ]);
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($now);
        return $now;
    }
}
