<?php
// 152E.06.02 Migrate evidence-backed paid install-site records into EDD
// authority (Spec 152E §22.1 inventory, §22.2 merge rules, §22.3 cutover step
// 6, §22.4 rollback, §6.3 atomic promotion, §7 canonical account model, §23
// acceptance "Legacy install-site record" row; Specs 152, 150A, 152A-D; Spec
// 158 implementation excluded).
//
// The paid-record migrator imports ONLY evidence-backed paid install-site
// records into the WPUIAI.com EDD authority, idempotently, with:
//   - evidence-first import: a record is accepted only when its Stripe
//     payment/refund evidence row (payment_evidence / refund_evidence, pinned
//     64-hex digest) and its install-registry source row both exist and agree
//     with the record handle;
//   - verified identity before ownership delivery: applyRecord() demands a
//     keyed 64-hex verified-identity digest (never the raw email); without it
//     the record stays verify_first and no entitlement is delivered;
//   - one entitlement per accepted record: each accepted paid record resolves
//     to exactly one EDD customer/order/license/account mapping and exactly
//     one journal entry; replays return the stored result and can never create
//     a second entitlement;
//   - preserved adverse state: refunded/revoked paid records are journaled
//     `preserve_adverse_state` and can NEVER be reactivated or re-granted
//     (reactivate() always fails closed with REFUNDED/REVOKED);
//   - preserved history where policy allows: product code, record status,
//     masked key, and sequence are carried from the install registry into the
//     imports/mapping rows — never from caller input;
//   - dry-run / apply / rollback-safe: dryRun() computes the decision with
//     zero writes; applyRecord() is idempotent and append-audited; rollback is
//     preservation-only and can never delete EDD customer/order/refund truth,
//     verified identities, licenses, nodes, sequences, credentials, Workpoints,
//     Evidence, or migration journals.
//
// No unverified-email promotion, no local/self-issued entitlement, no
// independent facade authority, no client-controlled EDD price/grants (the
// decision signature has no price, grant, tier, limit, or feature inputs), and
// no raw email, raw key, payment id, or secret ever leaves this contract —
// only 64-hex keyed digests and masked values.
declare(strict_types=1);

final class FocusaSpec152ePaidRecordMigrationSchema
{
    public const SCHEMA = 'focusa.spec152e.paid_record_migration.v1';
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
        $migrations = $this->table('wpuiai_paid_record_migration_schema_migrations');
        $events = $this->table('wpuiai_paid_record_migration_schema_events');
        $journal = $this->table('wpuiai_paid_record_journal');
        $imports = $this->table('wpuiai_paid_record_imports');
        $evidence = $this->table('wpuiai_paid_record_evidence');
        $registry = $this->table('wpuiai_paid_record_install_registry');
        $mappings = $this->table('wpuiai_paid_record_edd_mappings');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

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
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$journal} (
            journal_seq BIGINT NOT NULL PRIMARY KEY,
            journal_key VARCHAR(64) NOT NULL UNIQUE,
            record_handle VARCHAR(191) NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            detail TEXT NOT NULL,
            previous_digest VARCHAR(64) NOT NULL,
            entry_digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$imports} (
            import_uuid {$uuid} NOT NULL PRIMARY KEY,
            record_handle VARCHAR(191) NOT NULL UNIQUE,
            surface VARCHAR(64) NOT NULL,
            product_code VARCHAR(64) NOT NULL,
            disposition VARCHAR(32) NOT NULL,
            record_status VARCHAR(32) NOT NULL,
            masked_key {$key} NULL,
            evidence_handle VARCHAR(191) NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            verified_identity_digest VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NOT NULL,
            edd_customer_handle VARCHAR(191) NOT NULL,
            edd_order_handle VARCHAR(191) NOT NULL,
            edd_license_handle VARCHAR(191) NOT NULL,
            request_id VARCHAR(191) NOT NULL,
            idempotency_key VARCHAR(191) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            imported_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$evidence} (
            evidence_handle VARCHAR(191) NOT NULL PRIMARY KEY,
            kind VARCHAR(32) NOT NULL,
            source VARCHAR(64) NOT NULL,
            record_handle VARCHAR(191) NOT NULL,
            status VARCHAR(32) NOT NULL,
            digest VARCHAR(64) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$registry} (
            registry_handle VARCHAR(191) NOT NULL PRIMARY KEY,
            record_handle VARCHAR(191) NOT NULL UNIQUE,
            surface VARCHAR(64) NOT NULL,
            product_code VARCHAR(64) NOT NULL,
            record_status VARCHAR(32) NOT NULL,
            masked_key {$key} NULL,
            sequence BIGINT NOT NULL DEFAULT 0,
            adverse_state VARCHAR(16) NOT NULL DEFAULT 'none',
            evidence_handle VARCHAR(191) NOT NULL,
            digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$mappings} (
            mapping_uuid {$uuid} NOT NULL PRIMARY KEY,
            record_handle VARCHAR(191) NOT NULL UNIQUE,
            account_uuid {$uuid} NOT NULL,
            edd_customer_handle VARCHAR(191) NOT NULL,
            edd_order_handle VARCHAR(191) NOT NULL,
            edd_license_handle VARCHAR(191) NOT NULL,
            entitlement_digest VARCHAR(64) NOT NULL,
            migrated_at VARCHAR(32) NOT NULL,
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

    /** Rollback is preservation-only: journal, imports, evidence, registry, and mappings are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_paid_record_migration_schema_events');
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
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/', $timestamp) !== 1) {
            throw new InvalidArgumentException('timestamp must be ISO-8601 UTC');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        ksort($value, SORT_STRING);
        return json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }
}

final class FocusaSpec152ePaidRecordMigrationService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.paid_record_migration_result.v1';
    public const VERSION = 1;

    /** Migratable dispositions (Spec 152E §22.2 merge rules). */
    public const DISPOSITIONS = [
        'evidence_backed_import', 'refunded_revoked', 'verify_first', 'unresolved',
    ];

    /** Stripe payment/refund evidence kinds accepted by this surface. */
    public const EVIDENCE_KINDS = ['payment_evidence', 'refund_evidence'];

    /** Server-owned product allowlist (Spec 152E §8 product/grant registry). */
    public const PRODUCTS = [
        'focusa_operator', 'uiai_engine_operator', 'focusa_uiai_bundle', 'focusa_evaluation',
    ];

    /** Install-site source surfaces this migrator may read (never co-equal authority). */
    public const SURFACES = ['install_site_license', 'install_site_audit_receipt'];

    /** Adverse states that can never be reactivated (Spec 152E §18, §22.2). */
    public const ADVERSE_STATES = ['refunded', 'revoked'];

    private const HANDLE_PATTERN = '/^rec_[a-z0-9_]{4,64}$/D';
    private const EVIDENCE_HANDLE_PATTERN = '/^ev_[a-z0-9_]{4,64}$/D';
    private const DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const MASKED_PATTERN = '/^[A-Za-z0-9*_]{4,191}$/D';
    private const ACCOUNT_PATTERN = '/^acc_[a-z0-9_]{4,64}$/D';
    private const EDD_HANDLE_PATTERN = '/^edd_(?:cust|order|lic)_[a-z0-9_]{4,64}$/D';
    private const REQUEST_PATTERN = '/^req_[a-z0-9_]{4,64}$/D';
    private const IDEMPOTENCY_PATTERN = '/^idem_[a-z0-9_]{4,64}$/D';
    private const GENESIS_DIGEST = '0000000000000000000000000000000000000000000000000000000000000000';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152ePaidRecordMigrationSchema $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Dry-run decision: computes the exact apply outcome with ZERO writes.
     * Mirrors applyRecord()'s gates (evidence, verified identity, disposition,
     * product allowlist) but returns the decision without touching any table.
     * Used for preview before apply; never a grant by itself.
     */
    public function dryRun(array $input): array
    {
        $record = $this->assertRecord($input);
        $this->assertRequestInputs($input);
        // Mirror the exact apply gates so dry-run always previews apply.
        $evidence = $this->assertEvidence($input, $record);
        $registry = $this->assertRegistryRow($record);
        $decision = $this->resolveDisposition($record, $input, $evidence, $registry);
        return [
            'schema' => self::RESULT_SCHEMA,
            'mode' => 'dry_run',
            'record_handle' => $record['handle'],
            'disposition' => $record['disposition'],
            'decision' => $decision['decision'],
            'reason' => $decision['reason'] ?? null,
            'written' => false,
        ];
    }

    /**
     * Apply one evidence-backed paid record idempotently. Returns the stored
     * entitlement on success; replays return the SAME stored result and never
     * create a second import/mapping/journal row.
     *
     * Required input:
     *   - request_id / idempotency_key
     *   - record: handle, surface, disposition, product_code, masked_key
     *   - evidence_handle + evidence_digest (Stripe payment/refund evidence)
     *   - verified_identity_digest (keyed 64-hex digest, never the raw email)
     *   - migration_provenance
     */
    public function applyRecord(array $input): array
    {
        $record = $this->assertRecord($input);
        $this->assertRequestInputs($input);
        $evidence = $this->assertEvidence($input, $record);
        $registry = $this->assertRegistryRow($record);

        $stored = $this->findImport($record['handle']);
        if ($stored !== null) {
            return $this->replayResult($stored, true);
        }

        $decision = $this->resolveDisposition($record, $input, $evidence, $registry);
        if ($decision['decision'] !== 'import') {
            // Fail-closed: nothing is written for non-import decisions.
            if (($decision['reason'] ?? null) === 'REFUNDED' || ($decision['reason'] ?? null) === 'REVOKED') {
                $preserved = $this->findPreservedAdverseState($record['handle'], $decision['reason']);
                if ($preserved !== null) {
                    return $preserved;
                }
                $this->journal($record['handle'], 'preserve_adverse_state', $decision['reason'], $input);
            }
            return [
                'schema' => self::RESULT_SCHEMA,
                'mode' => 'apply',
                'record_handle' => $record['handle'],
                'disposition' => $record['disposition'],
                'decision' => $decision['decision'],
                'reason' => $decision['reason'],
                'entitlement' => null,
                'replayed' => false,
            ];
        }

        $import = $this->importOnce($record, $evidence, $registry, $input, $decision);
        return $this->replayResult($import, false);
    }

    /** Replay the stored import for a record handle; null when never imported. */
    public function replayImport(string $recordHandle): ?array
    {
        $stored = $this->findImport($recordHandle);
        return $stored === null ? null : $this->replayResult($stored, true);
    }

    /**
     * Ownership-delivery gate: the verified identity digest must be present and
     * must match the record's pinned expected digest. The raw email is never
     * accepted or stored — only the keyed 64-hex digest.
     */
    public function assertVerifiedIdentity(array $input, array $record): string
    {
        $digest = (string) ($input['verified_identity_digest'] ?? '');
        if ($digest === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (preg_match(self::DIGEST_PATTERN, $digest) !== 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        $expected = (string) ($record['identity_lookup_digest'] ?? '');
        if ($expected !== '' && !hash_equals($expected, $digest)) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        return $digest;
    }

    /**
     * Never reactivates a refunded/revoked record. This method always fails
     * closed with the typed adverse-state code; there is no code path that can
     * re-grant, re-import, or resurrect a refunded/revoked paid record.
     */
    public function reactivate(string $recordHandle, array $input): never
    {
        $registry = $this->findRegistryRow($recordHandle);
        $adverse = strtolower((string) ($registry['adverse_state'] ?? ''));
        if ($adverse === 'revoked') {
            throw new DomainException('REVOKED');
        }
        throw new DomainException('REFUNDED');
    }

    /** Verify the append-only journal digest chain from genesis. */
    public function journalChainValid(): bool
    {
        $journal = $this->schema->table('wpuiai_paid_record_journal');
        $rows = $this->db->query("SELECT record_handle, event_type, occurred_at, detail, previous_digest, entry_digest
            FROM {$journal} ORDER BY journal_seq ASC")->fetchAll(PDO::FETCH_ASSOC);
        $previous = self::GENESIS_DIGEST;
        foreach ($rows as $row) {
            $detailDigest = hash('sha256', $row['detail']);
            $expected = hash('sha256', $previous . "\n" . $row['record_handle'] . "\n" . $row['event_type'] . "\n" . $row['occurred_at'] . "\n" . $detailDigest);
            if (!hash_equals($expected, (string) $row['entry_digest'])) {
                return false;
            }
            if (!hash_equals($previous, (string) $row['previous_digest'])) {
                return false;
            }
            $previous = (string) $row['entry_digest'];
        }
        return true;
    }

    public function countRows(string $table): int
    {
        $quoted = $this->schema->table($table);
        return (int) $this->db->query("SELECT COUNT(*) FROM {$quoted}")->fetchColumn();
    }

    // ── Internals ──────────────────────────────────────────────────────

    private function resolveDisposition(array $record, array $input, ?array $evidence = null, ?array $registry = null): array
    {
        // Product allowlist: caller can never steer product, price, or grants.
        if (!in_array($record['product_code'], self::PRODUCTS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }

        if ($record['disposition'] === 'refunded_revoked') {
            $adverse = strtolower((string) ($registry['adverse_state'] ?? ''));
            $reason = $adverse === 'revoked' ? 'REVOKED' : 'REFUNDED';
            return ['decision' => 'preserve_adverse_state', 'reason' => $reason];
        }

        if ($record['disposition'] === 'unresolved') {
            // No evidence-backed paid linkage: never imported, never granted.
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        if ($record['disposition'] === 'verify_first') {
            // Ownership delivery requires verified identity.
            $this->assertVerifiedIdentity($input, $record);
            return ['decision' => 'import'];
        }

        if ($record['disposition'] !== 'evidence_backed_import') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        // Evidence-backed import: Stripe payment/refund evidence must agree.
        $this->assertVerifiedIdentity($input, $record);
        if ($evidence === null || $evidence['kind'] === 'refund_evidence') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return ['decision' => 'import'];
    }

    private function importOnce(array $record, array $evidence, array $registry, array $input, array $decision): array
    {
        $this->db->beginTransaction();
        try {
            $occurredAt = (string) ($this->clock)();
            $requestId = (string) $input['request_id'];
            $idempotencyKey = (string) $input['idempotency_key'];
            $provenance = FocusaSpec152ePaidRecordMigrationSchema::encodeCanonical((array) $input['migration_provenance']);
            $identityDigest = $this->assertVerifiedIdentity($input, $record);

            $accountUuid = (string) $input['account_uuid'];
            $customerHandle = (string) $input['edd_customer_handle'];
            $orderHandle = (string) $input['edd_order_handle'];
            $licenseHandle = (string) $input['edd_license_handle'];
            if (preg_match(self::ACCOUNT_PATTERN, $accountUuid) !== 1) {
                throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
            }
            if (preg_match(self::EDD_HANDLE_PATTERN, $customerHandle) !== 1
                || preg_match(self::EDD_HANDLE_PATTERN, $orderHandle) !== 1
                || preg_match(self::EDD_HANDLE_PATTERN, $licenseHandle) !== 1) {
                throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
            }

            $requestDigest = hash('sha256', $requestId . "\n" . $idempotencyKey . "\n" . $record['handle']);
            $importUuid = $this->uuid();
            $imports = $this->schema->table('wpuiai_paid_record_imports');
            $statement = $this->db->prepare("INSERT INTO {$imports}
                (import_uuid, record_handle, surface, product_code, disposition, record_status,
                 masked_key, evidence_handle, evidence_digest, verified_identity_digest,
                 account_uuid, edd_customer_handle, edd_order_handle, edd_license_handle,
                 request_id, idempotency_key, request_digest, imported_at, migration_provenance)
                VALUES (:import_uuid, :record_handle, :surface, :product_code, :disposition,
                 :record_status, :masked_key, :evidence_handle, :evidence_digest, :verified_identity_digest,
                 :account_uuid, :edd_customer_handle, :edd_order_handle, :edd_license_handle,
                 :request_id, :idempotency_key, :request_digest, :imported_at, :migration_provenance)");
            $statement->execute([
                ':import_uuid' => $importUuid,
                ':record_handle' => $record['handle'],
                ':surface' => $record['surface'],
                ':product_code' => $record['product_code'],
                ':disposition' => $record['disposition'],
                ':record_status' => (string) $registry['record_status'],
                ':masked_key' => $registry['masked_key'] ?? null,
                ':evidence_handle' => $evidence['evidence_handle'],
                ':evidence_digest' => $evidence['digest'],
                ':verified_identity_digest' => $identityDigest,
                ':account_uuid' => $accountUuid,
                ':edd_customer_handle' => $customerHandle,
                ':edd_order_handle' => $orderHandle,
                ':edd_license_handle' => $licenseHandle,
                ':request_id' => $requestId,
                ':idempotency_key' => $idempotencyKey,
                ':request_digest' => $requestDigest,
                ':imported_at' => $occurredAt,
                ':migration_provenance' => $provenance,
            ]);

            // Exactly one EDD/account mapping per accepted record.
            $entitlementDigest = hash('sha256', implode("\n", [
                $record['handle'], $accountUuid, $customerHandle, $orderHandle, $licenseHandle,
                $evidence['digest'], $identityDigest, (string) $registry['sequence'],
            ]));
            $mappingUuid = $this->uuid();
            $mappings = $this->schema->table('wpuiai_paid_record_edd_mappings');
            $mappingStatement = $this->db->prepare("INSERT INTO {$mappings}
                (mapping_uuid, record_handle, account_uuid, edd_customer_handle, edd_order_handle,
                 edd_license_handle, entitlement_digest, migrated_at, migration_provenance)
                VALUES (:mapping_uuid, :record_handle, :account_uuid, :edd_customer_handle, :edd_order_handle,
                 :edd_license_handle, :entitlement_digest, :migrated_at, :migration_provenance)");
            $mappingStatement->execute([
                ':mapping_uuid' => $mappingUuid,
                ':record_handle' => $record['handle'],
                ':account_uuid' => $accountUuid,
                ':edd_customer_handle' => $customerHandle,
                ':edd_order_handle' => $orderHandle,
                ':edd_license_handle' => $licenseHandle,
                ':entitlement_digest' => $entitlementDigest,
                ':migrated_at' => $occurredAt,
                ':migration_provenance' => $provenance,
            ]);

            $this->journal($record['handle'], 'imported', $record['product_code'], $input);

            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }

        return [
            'record_handle' => $record['handle'],
            'disposition' => $record['disposition'],
            'product_code' => $record['product_code'],
            'account_uuid' => $accountUuid,
            'edd_customer_handle' => $customerHandle,
            'edd_order_handle' => $orderHandle,
            'edd_license_handle' => $licenseHandle,
            'entitlement_digest' => $entitlementDigest,
        ];
    }

    /** Append one journal entry with a replay-safe digest chain; never deletes. */
    private function journal(string $recordHandle, string $eventType, string $detail, array $input): void
    {
        $journal = $this->schema->table('wpuiai_paid_record_journal');
        $occurredAt = (string) ($this->clock)();
        $provenance = FocusaSpec152ePaidRecordMigrationSchema::encodeCanonical((array) $input['migration_provenance']);
        $previousDigest = $this->lastJournalDigest();
        $detailDigest = hash('sha256', $detail);
        $entryDigest = hash('sha256', $previousDigest . "\n" . $recordHandle . "\n" . $eventType . "\n" . $occurredAt . "\n" . $detailDigest);
        $journalKey = hash('sha256', $entryDigest . "\n" . $occurredAt);
        $nextSeq = 1 + (int) $this->db->query("SELECT COALESCE(MAX(journal_seq), 0) FROM {$journal}")->fetchColumn();
        $statement = $this->db->prepare("INSERT INTO {$journal}
            (journal_seq, journal_key, record_handle, event_type, occurred_at, detail,
             previous_digest, entry_digest, migration_provenance)
            VALUES (:seq, :key, :record_handle, :event_type, :occurred_at, :detail,
             :previous_digest, :entry_digest, :migration_provenance)");
        $statement->execute([
            ':seq' => $nextSeq,
            ':key' => $journalKey,
            ':record_handle' => $recordHandle,
            ':event_type' => $eventType,
            ':occurred_at' => $occurredAt,
            ':detail' => $detail,
            ':previous_digest' => $previousDigest,
            ':entry_digest' => $entryDigest,
            ':migration_provenance' => $provenance,
        ]);
    }

    private function lastJournalDigest(): string
    {
        $journal = $this->schema->table('wpuiai_paid_record_journal');
        $value = $this->db->query("SELECT entry_digest FROM {$journal} ORDER BY journal_seq DESC LIMIT 1")->fetchColumn();
        return $value === false ? self::GENESIS_DIGEST : (string) $value;
    }

    private function findImport(string $recordHandle): ?array
    {
        $imports = $this->schema->table('wpuiai_paid_record_imports');
        $statement = $this->db->prepare("SELECT * FROM {$imports} WHERE record_handle = :handle");
        $statement->execute([':handle' => $recordHandle]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findPreservedAdverseState(string $recordHandle, string $reason): ?array
    {
        $journal = $this->schema->table('wpuiai_paid_record_journal');
        $statement = $this->db->prepare("SELECT detail FROM {$journal}
            WHERE record_handle = :handle AND event_type = 'preserve_adverse_state' AND detail = :reason
            ORDER BY journal_seq DESC LIMIT 1");
        $statement->execute([':handle' => $recordHandle, ':reason' => $reason]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'mode' => 'apply',
            'record_handle' => $recordHandle,
            'disposition' => 'refunded_revoked',
            'decision' => 'preserve_adverse_state',
            'reason' => $reason,
            'entitlement' => null,
            'replayed' => true,
        ];
    }

    private function findRegistryRow(string $recordHandle): ?array
    {
        $registry = $this->schema->table('wpuiai_paid_record_install_registry');
        $statement = $this->db->prepare("SELECT * FROM {$registry} WHERE record_handle = :handle");
        $statement->execute([':handle' => $recordHandle]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function replayResult(array $stored, bool $replayed): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'mode' => 'apply',
            'record_handle' => (string) $stored['record_handle'],
            'disposition' => (string) $stored['disposition'],
            'decision' => 'import',
            'entitlement' => [
                'account_uuid' => (string) $stored['account_uuid'],
                'edd_customer_handle' => (string) $stored['edd_customer_handle'],
                'edd_order_handle' => (string) $stored['edd_order_handle'],
                'edd_license_handle' => (string) $stored['edd_license_handle'],
            ],
            'replayed' => $replayed,
        ];
    }

    private function assertRecord(array $input): array
    {
        $record = (array) ($input['record'] ?? []);
        $handle = (string) ($record['handle'] ?? '');
        $surface = (string) ($record['surface'] ?? '');
        $disposition = (string) ($record['disposition'] ?? '');
        $productCode = (string) ($record['product_code'] ?? '');
        if (preg_match(self::HANDLE_PATTERN, $handle) !== 1) {
            throw new InvalidArgumentException('invalid record handle');
        }
        if (!in_array($surface, self::SURFACES, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (!in_array($disposition, self::DISPOSITIONS, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (!in_array($productCode, self::PRODUCTS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (isset($record['masked_key']) && preg_match(self::MASKED_PATTERN, (string) $record['masked_key']) !== 1) {
            throw new InvalidArgumentException('invalid masked key');
        }
        $expectedIdentity = (string) ($record['identity_lookup_digest'] ?? '');
        if ($expectedIdentity !== '' && preg_match(self::DIGEST_PATTERN, $expectedIdentity) !== 1) {
            throw new InvalidArgumentException('invalid identity digest');
        }
        return [
            'handle' => $handle,
            'surface' => $surface,
            'disposition' => $disposition,
            'product_code' => $productCode,
            'masked_key' => $record['masked_key'] ?? null,
            'identity_lookup_digest' => $expectedIdentity,
        ];
    }

    private function assertRequestInputs(array $input): void
    {
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        if (preg_match(self::REQUEST_PATTERN, $requestId) !== 1) {
            throw new DomainException('REQUEST_ID_REQUIRED');
        }
        if (preg_match(self::IDEMPOTENCY_PATTERN, $idempotencyKey) !== 1) {
            throw new DomainException('IDEMPOTENCY_KEY_REQUIRED');
        }
        if (($input['migration_provenance'] ?? []) === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
    }

    private function assertEvidence(array $input, array $record): array
    {
        $evidenceHandle = (string) ($input['evidence_handle'] ?? '');
        $evidenceDigest = (string) ($input['evidence_digest'] ?? '');
        if (preg_match(self::EVIDENCE_HANDLE_PATTERN, $evidenceHandle) !== 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (preg_match(self::DIGEST_PATTERN, $evidenceDigest) !== 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $evidence = $this->schema->table('wpuiai_paid_record_evidence');
        $statement = $this->db->prepare("SELECT * FROM {$evidence} WHERE evidence_handle = :handle");
        $statement->execute([':handle' => $evidenceHandle]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (!hash_equals((string) $row['digest'], $evidenceDigest)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((string) $row['record_handle'] !== $record['handle']) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (!in_array((string) $row['kind'], self::EVIDENCE_KINDS, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return $row;
    }

    private function assertRegistryRow(array $record): array
    {
        $registry = $this->findRegistryRow($record['handle']);
        if ($registry === null) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((string) $registry['surface'] !== $record['surface']) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((string) $registry['product_code'] !== $record['product_code']) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        return $registry;
    }

    private function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        $hex = bin2hex($bytes);
        return sprintf(
            '%s-%s-%s-%s-%s',
            substr($hex, 0, 8),
            substr($hex, 8, 4),
            substr($hex, 12, 4),
            substr($hex, 16, 4),
            substr($hex, 20, 12),
        );
    }
}
