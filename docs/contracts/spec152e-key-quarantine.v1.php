<?php
// Key quarantine ledger and settlement workflow (Spec 152E §6.4 quarantine of
// synthetic, duplicate, and ambiguous keys, §22.2 merge rules, §22.3 cutover
// step 6, §22.4 rollback). Quarantines synthetic focusa_live rows, duplicate
// EDD/custom keys, orphan payment IDs, and unresolved products/accounts in a
// durable audit ledger; prevents new activation/lease from quarantined records;
// selects the canonical key only with proof (keyed identity digest plus
// evidence-backed provenance); and exposes an explicit operator review/settlement
// path. The ledger retains audit/evidence forever and is preservation-only on
// rollback; EDD customer/order/refund truth, verified identities, and licenses
// are never mutated here. No raw key, raw email, payment id, or secret ever
// leaves this contract — only 64-hex keyed digests and masked keys.
declare(strict_types=1);

final class FocusaSpec152eKeyQuarantineSchema
{
    public const SCHEMA = 'focusa.spec152e.key_quarantine.v1';
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
        $migrations = $this->table('wpuiai_key_quarantine_schema_migrations');
        $events = $this->table('wpuiai_key_quarantine_schema_events');
        $ledger = $this->table('wpuiai_key_quarantine_ledger');
        $settlements = $this->table('wpuiai_key_quarantine_settlements');
        $journal = $this->table('wpuiai_key_quarantine_journal');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';

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
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$ledger} (
            quarantine_uuid {$uuid} NOT NULL PRIMARY KEY,
            record_handle VARCHAR(191) NOT NULL,
            surface VARCHAR(64) NOT NULL,
            key_group VARCHAR(64) NULL,
            state VARCHAR(24) NOT NULL DEFAULT 'quarantined',
            quarantine_reason VARCHAR(64) NOT NULL,
            key_digest VARCHAR(64) NOT NULL,
            masked_key VARCHAR(191) NULL,
            email_lookup_digest VARCHAR(64) NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            request_id VARCHAR(191) NOT NULL,
            idempotency_key VARCHAR(191) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            settled_at VARCHAR(32) NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (record_handle)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$settlements} (
            settlement_uuid {$uuid} NOT NULL PRIMARY KEY,
            quarantine_uuid {$uuid} NOT NULL,
            decision VARCHAR(32) NOT NULL,
            operator_id VARCHAR(64) NOT NULL,
            reason VARCHAR(64) NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            request_id VARCHAR(191) NOT NULL,
            idempotency_key VARCHAR(191) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            settled_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$journal} (
            journal_key VARCHAR(64) NOT NULL PRIMARY KEY,
            quarantine_uuid {$uuid} NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            detail TEXT NOT NULL,
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

    /** Rollback is preservation-only: ledger, settlements, and journal are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_key_quarantine_schema_events');
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

    /** Bounded evidence validation: provenance-backed kinds only; digest is never reversible. */
    public static function validateEvidence(array $evidence): string
    {
        if ($evidence === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $kind = (string) ($evidence['kind'] ?? '');
        if (!in_array($kind, FocusaSpec152eKeyQuarantineService::EVIDENCE_KINDS, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $source = (string) ($evidence['source'] ?? '');
        $record = (string) ($evidence['record'] ?? '');
        if ($source === '' || $record === ''
            || strlen($source) > 191 || strlen($record) > 191
            || preg_match('/[\r\n\x00]/', $source) === 1
            || preg_match('/[\r\n\x00]/', $record) === 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return hash('sha256', self::encodeCanonical($evidence));
    }
}

final class FocusaSpec152eKeyQuarantineService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.key_quarantine_result.v1';
    public const VERSION = 1;

    /** Surfaces the quarantine ledger governs (Spec 152E §6.4 / §22.1). */
    public const SURFACES = [
        'focusa_live_synthetic', 'duplicate_edd_key', 'duplicate_custom_key',
        'orphan_payment_id', 'unresolved_product', 'unresolved_account',
    ];

    /** Ledger states; only settled_approved may ever pass an activation/lease gate. */
    public const STATES = ['quarantined', 'settled_approved', 'settled_denied'];

    /** Bounded evidence provenance kinds. */
    public const EVIDENCE_KINDS = [
        'purchase_evidence', 'stripe_reconciliation', 'install_site_migration', 'operator_review',
    ];

    /** Surface-level fail-closed quarantine reasons (all registered activation error codes). */
    public const QUARANTINE_REASONS = [
        'focusa_live_synthetic' => 'EDD_ORDER_UNVERIFIED',
        'duplicate_edd_key' => 'LICENSE_ACCOUNT_MISMATCH',
        'duplicate_custom_key' => 'ACCOUNT_MERGE_REVIEW_REQUIRED',
        'orphan_payment_id' => 'EDD_ORDER_UNVERIFIED',
        'unresolved_product' => 'PRODUCT_MAPPING_REQUIRED',
        'unresolved_account' => 'EDD_CUSTOMER_RESOLUTION_FAILED',
    ];

    /** Denial reasons an operator may settle with (registered activation error codes). */
    public const DENY_REASONS = [
        'EDD_ORDER_UNVERIFIED', 'EDD_LICENSE_UNUSABLE', 'LICENSE_ACCOUNT_MISMATCH',
        'ACCOUNT_MERGE_REVIEW_REQUIRED', 'REFUNDED', 'REVOKED',
    ];

    /** Approval settlement reason: canonical selection only ever with proof. */
    public const APPROVE_REASON = 'PROOF_BACKED_SELECTION';

    /** Gate purposes that must fail closed against quarantined records. */
    public const PURPOSES = ['activation', 'lease'];

    /** Synthetic key prefixes that can never become authority, ledgered or not. */
    public const SYNTHETIC_PREFIXES = ['focusa_live_', 'synthetic_', 'local_', 'eval_'];

    private const EMAIL_DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const HANDLE_PATTERN = '/^rec_[a-z0-9_]{6,64}$/D';
    private const GROUP_PATTERN = '/^kg_[a-z0-9_]{4,64}$/D';
    private const OPERATOR_PATTERN = '/^op_[a-z0-9_]{4,64}$/D';
    private const MASKED_PATTERN = '/^[A-Za-z0-9*_]{4,191}$/D';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eKeyQuarantineSchema $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Quarantine one record: a synthetic focusa_live row, a duplicate EDD/custom
     * key, an orphan payment ID, or an unresolved product/account. Only the
     * 64-hex key digest, masked key, keyed email digest, and evidence digest are
     * stored — never the raw key, raw email, payment id, or secret. One ledger
     * row per record handle; replays return the stored row. Quarantined records
     * cannot pass the activation/lease gate and cannot be canonical until proof.
     *
     * Required input:
     *   - record_handle / surface / quarantine_reason (must equal the surface map)
     *   - key_material OR key_digest (the key, payment id, product code, or
     *     account handle presented for activation/lease)
     *   - legacy_evidence / request_id / idempotency_key / migration_provenance
     * Optional input:
     *   - masked_key, key_group, email_lookup_digest
     */
    public function quarantineRecord(array $input): array
    {
        $handle = $this->assertHandle((string) ($input['record_handle'] ?? ''));
        $surface = (string) ($input['surface'] ?? '');
        if (!in_array($surface, self::SURFACES, true)) {
            throw new InvalidArgumentException('known quarantine surface required');
        }
        $reason = (string) ($input['quarantine_reason'] ?? '');
        if (!hash_equals(self::QUARANTINE_REASONS[$surface], $reason)) {
            throw new InvalidArgumentException('surface-mapped quarantine reason required');
        }
        $keyDigest = $this->resolveKeyDigest($input);
        $maskedKey = $this->optionalMaskedKey($input['masked_key'] ?? null);
        $group = $this->optionalGroup($input['key_group'] ?? null);
        $emailDigest = $this->optionalEmailDigest($input['email_lookup_digest'] ?? null);
        $evidenceDigest = FocusaSpec152eKeyQuarantineSchema::validateEvidence($input['legacy_evidence'] ?? []);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $digest = $this->digest([
            'operation' => 'quarantine_record',
            'record_handle' => $handle,
            'surface' => $surface,
            'quarantine_reason' => $reason,
            'key_digest' => $keyDigest,
            'key_group' => $group,
            'email_lookup_digest' => $emailDigest,
            'legacy_evidence' => $input['legacy_evidence'] ?? [],
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($handle, $surface, $reason, $keyDigest, $maskedKey, $group, $emailDigest, $evidenceDigest, $requestId, $idempotencyKey, $provenance, $now, $digest): array {
            $table = $this->schema->table('wpuiai_key_quarantine_ledger');
            $statement = $this->db->prepare("SELECT * FROM {$table} WHERE record_handle = :handle");
            $statement->execute([':handle' => $handle]);
            $existing = $statement->fetch(PDO::FETCH_ASSOC);
            if ($existing !== false) {
                if ((string) $existing['state'] === 'settled_approved') {
                    throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
                }
                return $this->ledgerEnvelope($existing, 'legacy_record_quarantined', replayed: false, existing: true);
            }

            $quarantineUuid = self::uuid();
            $statement = $this->db->prepare("INSERT INTO {$table}
                (quarantine_uuid, record_handle, surface, key_group, state, quarantine_reason,
                 key_digest, masked_key, email_lookup_digest, evidence_digest, request_id,
                 idempotency_key, request_digest, created_at, updated_at, settled_at,
                 migration_provenance)
                VALUES (:quarantine, :handle, :surface, :group, 'quarantined', :reason,
                 :key_digest, :masked, :email, :evidence, :request, :idem, :request_digest,
                 :created, :updated, NULL, :provenance)");
            $statement->execute([
                ':quarantine' => $quarantineUuid,
                ':handle' => $handle,
                ':surface' => $surface,
                ':group' => $group,
                ':reason' => $reason,
                ':key_digest' => $keyDigest,
                ':masked' => $maskedKey,
                ':email' => $emailDigest,
                ':evidence' => $evidenceDigest,
                ':request' => $requestId,
                ':idem' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':updated' => $now,
                ':provenance' => $this->encodeCanonical($provenance),
            ]);
            $this->journal($quarantineUuid, 'quarantined', $now, [
                'record_handle' => $handle,
                'surface' => $surface,
                'quarantine_reason' => $reason,
                'key_group' => $group,
                'state' => 'quarantined',
            ], $provenance);
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'legacy_record_quarantined',
                'quarantine_uuid' => $quarantineUuid,
                'record_handle' => $handle,
                'surface' => $surface,
                'state' => 'quarantined',
                'quarantine_reason' => $reason,
                'key_digest' => $keyDigest,
                'replayed' => false,
                'existing' => false,
            ];
        });
    }

    /**
     * Select the canonical key within a duplicate key group only with proof: the
     * keyed email identity digest must equal the candidate's stored digest and
     * evidence-backed provenance must be presented. If two candidates of the
     * group match the same proof, the selection is ambiguous and fails closed
     * (ACCOUNT_MERGE_REVIEW_REQUIRED) with both records remaining quarantined.
     * The winner is settled_approved with an audited settlement; the other
     * duplicates stay quarantined until an explicit operator settlement.
     *
     * Required input:
     *   - key_group / record_handle / operator_id / email_lookup_digest
     *   - legacy_evidence / request_id / idempotency_key / migration_provenance
     */
    public function selectCanonicalKey(array $input): array
    {
        $group = $this->assertGroup((string) ($input['key_group'] ?? ''));
        $handle = $this->assertHandle((string) ($input['record_handle'] ?? ''));
        $operatorId = $this->assertOperator((string) ($input['operator_id'] ?? ''));
        $emailDigest = $this->assertEmailDigest((string) ($input['email_lookup_digest'] ?? ''));
        $evidenceDigest = FocusaSpec152eKeyQuarantineSchema::validateEvidence($input['legacy_evidence'] ?? []);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $digest = $this->digest([
            'operation' => 'select_canonical_key',
            'key_group' => $group,
            'record_handle' => $handle,
            'operator_id' => $operatorId,
            'email_lookup_digest' => $emailDigest,
            'legacy_evidence' => $input['legacy_evidence'] ?? [],
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        // Read-path proof checks run before any write; the ambiguity journal commits
        // independently so the operator review trail survives the fail-closed denial.
        $table = $this->schema->table('wpuiai_key_quarantine_ledger');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE record_handle = :handle");
        $statement->execute([':handle' => $handle]);
        $candidate = $statement->fetch(PDO::FETCH_ASSOC);
        if ($candidate === false || (string) $candidate['key_group'] !== $group) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((string) $candidate['state'] !== 'quarantined') {
            $existing = $this->findSettlementByQuarantine((string) $candidate['quarantine_uuid']);
            if ((string) $candidate['state'] === 'settled_approved'
                && $existing !== null
                && hash_equals((string) $existing['idempotency_key'], $idempotencyKey)
                && hash_equals((string) $existing['request_digest'], $digest)) {
                return [
                    'schema' => self::RESULT_SCHEMA,
                    'action' => 'canonical_key_selected',
                    'settlement_uuid' => $existing['settlement_uuid'],
                    'quarantine_uuid' => $candidate['quarantine_uuid'],
                    'record_handle' => $handle,
                    'key_group' => $group,
                    'state' => 'settled_approved',
                    'canonical' => true,
                    'replayed' => true,
                ];
            }
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }
        $storedEmail = (string) $candidate['email_lookup_digest'];
        if ($storedEmail === '' || !hash_equals($storedEmail, $emailDigest)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        // Only one canonical record per key group: a settled group cannot be re-opened.
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}
            WHERE key_group = :group AND record_handle <> :handle AND state = 'settled_approved'");
        $statement->execute([':group' => $group, ':handle' => $handle]);
        if ((int) $statement->fetchColumn() > 0) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }

        // The same proof matching more than one duplicate is ambiguous.
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}
            WHERE key_group = :group AND email_lookup_digest = :email
              AND state IN ('quarantined', 'settled_denied')");
        $statement->execute([':group' => $group, ':email' => $emailDigest]);
        if ((int) $statement->fetchColumn() > 1) {
            $this->journal((string) $candidate['quarantine_uuid'], 'selection_ambiguous', $now, [
                'record_handle' => $handle,
                'key_group' => $group,
                'state' => 'quarantined',
            ], $provenance);
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }

        return $this->transaction(function () use ($candidate, $group, $handle, $operatorId, $evidenceDigest, $requestId, $idempotencyKey, $provenance, $now, $digest): array {
            $table = $this->schema->table('wpuiai_key_quarantine_ledger');
            $settlementUuid = $this->insertSettlement(
                (string) $candidate['quarantine_uuid'],
                'approve_canonical',
                $operatorId,
                self::APPROVE_REASON,
                $evidenceDigest,
                $requestId,
                $idempotencyKey,
                $digest,
                $now,
                $provenance,
            );
            $statement = $this->db->prepare("UPDATE {$table}
                SET state = 'settled_approved', updated_at = :updated, settled_at = :settled
                WHERE quarantine_uuid = :quarantine AND state = 'quarantined'");
            $statement->execute([
                ':updated' => $now,
                ':settled' => $now,
                ':quarantine' => $candidate['quarantine_uuid'],
            ]);
            $this->journal((string) $candidate['quarantine_uuid'], 'canonical_selected', $now, [
                'record_handle' => $handle,
                'key_group' => $group,
                'settlement_uuid' => $settlementUuid,
                'state' => 'settled_approved',
            ], $provenance);
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'canonical_key_selected',
                'settlement_uuid' => $settlementUuid,
                'quarantine_uuid' => $candidate['quarantine_uuid'],
                'record_handle' => $handle,
                'key_group' => $group,
                'state' => 'settled_approved',
                'canonical' => true,
                'replayed' => false,
            ];
        });
    }

    /**
     * Explicit operator review/settlement path. An audited operator settles a
     * quarantined record: approve_canonical makes it usable again (only with the
     * proof-backed reason), deny keeps it fail-closed with a bounded denial code.
     * One settlement per record: replays return the stored decision; a different
     * settlement on the same record is an IDEMPOTENCY_CONFLICT.
     *
     * Required input:
     *   - record_handle / operator_id / decision / reason
     *   - legacy_evidence / request_id / idempotency_key / migration_provenance
     */
    public function settleRecord(array $input): array
    {
        $handle = $this->assertHandle((string) ($input['record_handle'] ?? ''));
        $operatorId = $this->assertOperator((string) ($input['operator_id'] ?? ''));
        $decision = (string) ($input['decision'] ?? '');
        if (!in_array($decision, ['approve_canonical', 'deny'], true)) {
            throw new InvalidArgumentException('bounded settlement decision required');
        }
        $reason = (string) ($input['reason'] ?? '');
        if ($decision === 'approve_canonical') {
            if (!hash_equals(self::APPROVE_REASON, $reason)) {
                throw new InvalidArgumentException('proof-backed approval reason required');
            }
        } elseif (!in_array($reason, self::DENY_REASONS, true)) {
            throw new InvalidArgumentException('bounded denial reason required');
        }
        $evidenceDigest = FocusaSpec152eKeyQuarantineSchema::validateEvidence($input['legacy_evidence'] ?? []);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $digest = $this->digest([
            'operation' => 'settle_record',
            'record_handle' => $handle,
            'operator_id' => $operatorId,
            'decision' => $decision,
            'reason' => $reason,
            'legacy_evidence' => $input['legacy_evidence'] ?? [],
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($handle, $operatorId, $decision, $reason, $evidenceDigest, $requestId, $idempotencyKey, $provenance, $now, $digest): array {
            $table = $this->schema->table('wpuiai_key_quarantine_ledger');
            $statement = $this->db->prepare("SELECT * FROM {$table} WHERE record_handle = :handle");
            $statement->execute([':handle' => $handle]);
            $record = $statement->fetch(PDO::FETCH_ASSOC);
            if ($record === false) {
                throw new OutOfBoundsException('quarantine ledger record not found');
            }
            if ((string) $record['state'] !== 'quarantined') {
                $existing = $this->findSettlementByQuarantine((string) $record['quarantine_uuid']);
                if ($existing !== null && hash_equals((string) $existing['idempotency_key'], $idempotencyKey)) {
                    if (!hash_equals((string) $existing['request_digest'], $digest)) {
                        throw new DomainException('IDEMPOTENCY_CONFLICT');
                    }
                    return $this->settlementEnvelope($existing, $record, replayed: true);
                }
                throw new DomainException('IDEMPOTENCY_CONFLICT');
            }

            $settlementUuid = $this->insertSettlement(
                (string) $record['quarantine_uuid'],
                $decision,
                $operatorId,
                $reason,
                $evidenceDigest,
                $requestId,
                $idempotencyKey,
                $digest,
                $now,
                $provenance,
            );
            $targetState = $decision === 'approve_canonical' ? 'settled_approved' : 'settled_denied';
            $statement = $this->db->prepare("UPDATE {$table}
                SET state = :state, updated_at = :updated, settled_at = :settled
                WHERE quarantine_uuid = :quarantine AND state = 'quarantined'");
            $statement->execute([
                ':state' => $targetState,
                ':updated' => $now,
                ':settled' => $now,
                ':quarantine' => $record['quarantine_uuid'],
            ]);
            $this->journal((string) $record['quarantine_uuid'], 'settled', $now, [
                'record_handle' => $handle,
                'settlement_uuid' => $settlementUuid,
                'decision' => $decision,
                'operator_id' => $operatorId,
                'reason' => $reason,
                'state' => $targetState,
            ], $provenance);
            $record['state'] = $targetState;
            $record['settled_at'] = $now;
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'settlement_recorded',
                'settlement_uuid' => $settlementUuid,
                'quarantine_uuid' => $record['quarantine_uuid'],
                'record_handle' => $handle,
                'decision' => $decision,
                'state' => $targetState,
                'replayed' => false,
            ];
        });
    }

    /**
     * Fail-closed activation/lease gate. An unresolved quarantined record always
     * denies new activation/lease with its recorded reason; a record denied by
     * explicit operator settlement denies with the recorded reason; an explicitly
     * approved canonical record (settled_approved) passes — synthetic records
     * require that separate operator approval and never pass silently. A key
     * absent from the ledger that still carries a synthetic prefix (focusa_live_*
     * or any synthetic prefix) is denied fail-closed; other unledgered keys pass
     * (canonical issuance truth is enforced by the authority surfaces). Never
     * returns the raw key or any secret.
     *
     * Required input:
     *   - key_material OR key_digest
     *   - purpose ('activation' | 'lease') / request_id
     */
    public function activationLeaseGate(array $input): array
    {
        $purpose = (string) ($input['purpose'] ?? '');
        if (!in_array($purpose, self::PURPOSES, true)) {
            throw new InvalidArgumentException('gate purpose required');
        }
        $keyDigest = $this->resolveKeyDigest($input);
        $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $material = (string) ($input['key_material'] ?? '');

        $table = $this->schema->table('wpuiai_key_quarantine_ledger');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE key_digest = :digest ORDER BY created_at");
        $statement->execute([':digest' => $keyDigest]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        if ($rows === []) {
            foreach (self::SYNTHETIC_PREFIXES as $prefix) {
                if (str_starts_with($material, $prefix)) {
                    return $this->gateDenied('EDD_ORDER_UNVERIFIED', $purpose, 'synthetic_key_denied');
                }
            }
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'gate_allowed',
                'purpose' => $purpose,
                'allowed' => true,
                'ledgered' => false,
                'canonical' => false,
            ];
        }
        $quarantinedReason = null;
        $approvedHandle = null;
        $deniedReason = null;
        foreach ($rows as $row) {
            $state = (string) $row['state'];
            if ($state === 'quarantined') {
                $quarantinedReason = $quarantinedReason ?? (string) $row['quarantine_reason'];
            } elseif ($state === 'settled_approved') {
                $approvedHandle = $approvedHandle ?? (string) $row['record_handle'];
            } elseif ($state === 'settled_denied') {
                $deniedReason = $deniedReason ?? (string) $row['quarantine_reason'];
            }
        }
        if ($quarantinedReason !== null) {
            return $this->gateDenied($quarantinedReason, $purpose, 'quarantined_key_denied');
        }
        if ($approvedHandle !== null) {
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'gate_allowed',
                'purpose' => $purpose,
                'allowed' => true,
                'ledgered' => true,
                'canonical' => true,
                'canonical_record_handle' => $approvedHandle,
            ];
        }
        if ($deniedReason !== null) {
            return $this->gateDenied($deniedReason, $purpose, 'denied_key_denied');
        }
        return $this->gateDenied('ACCOUNT_MERGE_REVIEW_REQUIRED', $purpose, 'ambiguous_key_denied');
    }

    /**
     * Bounded operator read path over the quarantine ledger. Returns only
     * digests and masked keys — never raw keys, raw emails, payment ids, or
     * secrets — plus state counts for the operator review queue.
     *
     * Required input: request_id. Optional: surface, limit (1..100).
     */
    public function listQuarantined(array $input): array
    {
        $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $limit = (int) ($input['limit'] ?? 100);
        if ($limit < 1 || $limit > 100) {
            throw new InvalidArgumentException('bounded ledger page limit required');
        }
        $surface = (string) ($input['surface'] ?? '');
        if ($surface !== '' && !in_array($surface, self::SURFACES, true)) {
            throw new InvalidArgumentException('known quarantine surface required');
        }
        $table = $this->schema->table('wpuiai_key_quarantine_ledger');
        $params = [];
        $where = '';
        if ($surface !== '') {
            $where = 'WHERE surface = :surface';
            $params[':surface'] = $surface;
        }
        $statement = $this->db->prepare("SELECT quarantine_uuid, record_handle, surface, key_group, state,
            quarantine_reason, key_digest, masked_key, email_lookup_digest, evidence_digest,
            created_at, updated_at, settled_at
            FROM {$table} {$where} ORDER BY created_at LIMIT :limit");
        $statement->bindValue(':limit', $limit, PDO::PARAM_INT);
        foreach ($params as $key => $value) {
            $statement->bindValue($key, $value);
        }
        $statement->execute();
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        $stateCounts = ['quarantined' => 0, 'settled_approved' => 0, 'settled_denied' => 0];
        foreach ($rows as $row) {
            $stateCounts[(string) $row['state']]++;
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => 'ledger_listed',
            'count' => count($rows),
            'state_counts' => $stateCounts,
            'records' => $rows,
        ];
    }

    // ── internal helpers ────────────────────────────────────────────────

    private function insertSettlement(string $quarantineUuid, string $decision, string $operatorId, string $reason, string $evidenceDigest, string $requestId, string $idempotencyKey, string $digest, string $now, array $provenance): string
    {
        $table = $this->schema->table('wpuiai_key_quarantine_settlements');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE quarantine_uuid = :quarantine");
        $statement->execute([':quarantine' => $quarantineUuid]);
        $existing = $statement->fetch(PDO::FETCH_ASSOC);
        if ($existing !== false) {
            if (!hash_equals((string) $existing['idempotency_key'], $idempotencyKey)
                || !hash_equals((string) $existing['request_digest'], $digest)) {
                throw new DomainException('IDEMPOTENCY_CONFLICT');
            }
            return (string) $existing['settlement_uuid'];
        }
        $settlementUuid = self::uuid();
        $statement = $this->db->prepare("INSERT INTO {$table}
            (settlement_uuid, quarantine_uuid, decision, operator_id, reason, evidence_digest,
             request_id, idempotency_key, request_digest, settled_at, migration_provenance)
            VALUES (:settlement, :quarantine, :decision, :operator, :reason, :evidence,
             :request, :idem, :request_digest, :settled, :provenance)");
        $statement->execute([
            ':settlement' => $settlementUuid,
            ':quarantine' => $quarantineUuid,
            ':decision' => $decision,
            ':operator' => $operatorId,
            ':reason' => $reason,
            ':evidence' => $evidenceDigest,
            ':request' => $requestId,
            ':idem' => $idempotencyKey,
            ':request_digest' => $digest,
            ':settled' => $now,
            ':provenance' => $this->encodeCanonical($provenance),
        ]);
        return $settlementUuid;
    }

    private function findSettlementByQuarantine(string $quarantineUuid): ?array
    {
        $table = $this->schema->table('wpuiai_key_quarantine_settlements');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE quarantine_uuid = :quarantine");
        $statement->execute([':quarantine' => $quarantineUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function settlementEnvelope(array $settlement, array $record, bool $replayed): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => 'settlement_recorded',
            'settlement_uuid' => $settlement['settlement_uuid'],
            'quarantine_uuid' => $record['quarantine_uuid'],
            'record_handle' => $record['record_handle'],
            'decision' => $settlement['decision'],
            'state' => $record['state'],
            'replayed' => $replayed,
        ];
    }

    private function ledgerEnvelope(array $record, string $action, bool $replayed, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => $action,
            'quarantine_uuid' => $record['quarantine_uuid'],
            'record_handle' => $record['record_handle'],
            'surface' => $record['surface'],
            'state' => $record['state'],
            'quarantine_reason' => $record['quarantine_reason'],
            'key_digest' => $record['key_digest'],
            'replayed' => $replayed,
            'existing' => $existing,
        ];
    }

    private function gateDenied(string $reason, string $purpose, string $kind): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => 'gate_denied',
            'purpose' => $purpose,
            'allowed' => false,
            'kind' => $kind,
            'reason' => $reason,
        ];
    }

    private function resolveKeyDigest(array $input): string
    {
        $material = (string) ($input['key_material'] ?? '');
        if ($material !== '') {
            if (strlen($material) > 191 || preg_match('/[\r\n\x00]/', $material) === 1) {
                throw new InvalidArgumentException('bounded key material required');
            }
            return hash('sha256', $material);
        }
        $digest = (string) ($input['key_digest'] ?? '');
        if (preg_match('/^[0-9a-f]{64}$/D', $digest) !== 1) {
            throw new InvalidArgumentException('bounded key digest required');
        }
        return $digest;
    }

    private function optionalMaskedKey(?string $masked): ?string
    {
        if ($masked === null || $masked === '') {
            return null;
        }
        if (preg_match(self::MASKED_PATTERN, $masked) !== 1) {
            throw new InvalidArgumentException('bounded masked key required');
        }
        return $masked;
    }

    private function optionalGroup(?string $group): ?string
    {
        if ($group === null || $group === '') {
            return null;
        }
        return $this->assertGroup($group);
    }

    private function optionalEmailDigest(?string $digest): ?string
    {
        if ($digest === null || $digest === '') {
            return null;
        }
        return $this->assertEmailDigest($digest);
    }

    private function journal(string $quarantineUuid, string $eventType, string $occurredAt, array $detail, array $provenance): void
    {
        $table = $this->schema->table('wpuiai_key_quarantine_journal');
        $journalKey = hash('sha256', self::RESULT_SCHEMA . "\n" . $eventType . "\n" . $occurredAt . "\n" . $this->encodeCanonical($detail));
        $statement = $this->db->prepare("INSERT INTO {$table}
            (journal_key, quarantine_uuid, event_type, occurred_at, detail, migration_provenance)
            SELECT :key, :quarantine, :event, :occurred, :detail, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$table} WHERE journal_key = :existing_key)");
        $statement->execute([
            ':key' => $journalKey,
            ':quarantine' => $quarantineUuid,
            ':event' => $eventType,
            ':occurred' => $occurredAt,
            ':detail' => $this->encodeCanonical($detail),
            ':provenance' => $this->encodeCanonical($provenance),
            ':existing_key' => $journalKey,
        ]);
    }

    private function provenance(mixed $value): array
    {
        if (!is_array($value) || $value === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        return $value;
    }

    private function assertHandle(string $handle): string
    {
        if (preg_match(self::HANDLE_PATTERN, $handle) !== 1) {
            throw new InvalidArgumentException('bounded opaque record handle required');
        }
        return $handle;
    }

    private function assertGroup(string $group): string
    {
        if (preg_match(self::GROUP_PATTERN, $group) !== 1) {
            throw new InvalidArgumentException('bounded opaque key group required');
        }
        return $group;
    }

    private function assertOperator(string $operatorId): string
    {
        if (preg_match(self::OPERATOR_PATTERN, $operatorId) !== 1) {
            throw new InvalidArgumentException('bounded operator audit identity required');
        }
        return $operatorId;
    }

    private function assertEmailDigest(string $digest): string
    {
        if (preg_match(self::EMAIL_DIGEST_PATTERN, $digest) !== 1) {
            throw new InvalidArgumentException('bounded keyed email lookup digest required');
        }
        return $digest;
    }

    private function assertRequestId(string $requestId): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $requestId) !== 1) {
            throw new InvalidArgumentException('bounded request ID required');
        }
        return $requestId;
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
        return hash('sha256', $this->encodeCanonical($value));
    }

    private function encodeCanonical(array $value): string
    {
        return FocusaSpec152eKeyQuarantineSchema::encodeCanonical($value);
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eKeyQuarantineSchema::assertTimestamp($now);
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
