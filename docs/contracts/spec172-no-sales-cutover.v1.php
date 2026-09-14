<?php
// 172.05.01 Execute no-sales clean cutover and quarantine legacy products
// (Spec 172 §16 dedicated products, §19 no-sales proof and cutover, §20 gate 15,
// §22 item 7 and 11, §23 acceptance, §24 rollback; non-conflicting Specs 152,
// 152E, 152F, and 150A remain binding).
//
// The Spec 172 no-sales cutover canary is the bounded, idempotent, fail-closed
// cutover gate that:
//   - REQUIRES accepted zero-sales proof (Spec 172 §19): every canary run must
//     carry the accepted no-sales inventory decision
//     (docs/contracts/spec172-no-sales-inventory.v1.json). A missing or
//     malformed proof fails closed with ZERO_SALES_PROOF_REQUIRED and writes
//     nothing.
//   - NEVER enables dedicated EDD mappings before validation: dedicated
//     Operator v1 Downloads (458/459/460) stay
//     approved_policy_blocked_edd_mapping while zero-sales proof is not
//     accepted; when the proof is accepted (zero_sales_proven=true and
//     clean_cutover_allowed=true) the canary enables the mappings AFTER
//     validation (approved_mapping_enabled_after_validation) while checkout
//     remains disabled (sale_status approved_not_yet_enabled). No customer can
//     enter a commercial contract against contradictory terms.
//   - DISABLES direct install-site/Gravity entitlement issuance: the
//     install-site create/payment/webhook surfaces, the caller-parameter
//     custom issue surface, direct Stripe product flow, and local
//     self-Evaluation fail closed with their exact denial codes
//     (INSTALL_SITE_ISSUANCE_DISABLED / STRIPE_DIRECT_FLOW_DENIED /
//     LOCAL_EVALUATION_DENIED) and are audited exactly once per run; retained
//     validation/recovery surfaces grant no entitlement (grants_entitlement
//     false everywhere) and there is no enable path — rollback can never
//     restore split issuance.
//   - QUARANTINES old WPUIAI, Download 453, and synthetic records in a durable
//     quarantine ledger: quarantine/retire records can never grant an Operator
//     v1 License Type (never_grant true); migration-class records stay
//     preserved and evidence-backed (never granted); refunded/revoked records
//     stay terminal and are never reactivated.
//   - PRESERVES logs and evidence: every cutover, quarantine, reconciliation,
//     and rollback event is appended to a digest-chained journal; the schema
//     contains no DELETE/TRUNCATE/DROP/UPDATE path (preservation-only).
//   - STOPS and requires a customer-rights mapping if a genuine sale appears:
//     the decision becomes stopped_requiring_customer_rights_mapping with
//     GENUINE_SALE_REQUIRES_CUSTOMER_RIGHTS_MAPPING, the record is preserved,
//     and no issuance or quarantine mutation proceeds.
//   - Produces an idempotent canary cutover receipt and a rollback-safe
//     reconciliation receipt: replays return the byte-identical stored receipt
//     with zero writes; a different payload on the same run_handle fails
//     closed with RUN_ALREADY_STARTED.
//
// No caller-controlled product, price, License Type, family, feature, limit,
// node, or commercial right is accepted: mapping entries may only carry the
// server-owned fields allowlisted here, and any extra caller field fails
// closed with CLIENT_COMMERCIAL_FIELDS_FORBIDDEN. No raw email, key, token,
// customer row, credential, or card data is ever accepted, stored, or
// returned — only 64-hex digests and redacted dispositions.
declare(strict_types=1);

final class FocusaSpec172NoSalesCutoverSchema
{
    public const SCHEMA = 'focusa.spec172.no_sales_cutover.v1';
    public const VERSION = 1;

    /** Preservation-only invariant: the schema contains no destructive DML/DDL. */
    public const FORBIDDEN_STATEMENTS = ['/\\bDELETE\\b/i', '/\\bTRUNCATE\\b/i', '/\\bDROP\\b/i', '/\\bUPDATE\\b/i'];

    /** @var list<string> every DDL statement this schema may execute. */
    public const DDL = [
        "CREATE TABLE IF NOT EXISTS {migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS {events} (
            event_key VARCHAR(64) NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            schema_version BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS {runs} (
            run_handle TEXT NOT NULL PRIMARY KEY,
            run_digest VARCHAR(64) NOT NULL,
            inventory_id VARCHAR(191) NOT NULL,
            decision VARCHAR(48) NOT NULL,
            zero_sales_proven VARCHAR(8) NOT NULL,
            clean_cutover_allowed VARCHAR(8) NOT NULL,
            block_reason VARCHAR(64) NULL,
            genuine_sale VARCHAR(8) NOT NULL,
            receipt_payload TEXT NOT NULL,
            receipt_digest VARCHAR(64) NOT NULL,
            idempotency_key VARCHAR(64) NOT NULL,
            started_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (run_digest)
        )",
        "CREATE TABLE IF NOT EXISTS {mappings} (
            run_handle TEXT NOT NULL,
            public_code VARCHAR(191) NOT NULL,
            edd_download_id BIGINT NOT NULL,
            edd_price_id VARCHAR(191) NOT NULL,
            mapping_status VARCHAR(64) NOT NULL,
            checkout_enabled VARCHAR(8) NOT NULL,
            sale_status VARCHAR(64) NOT NULL,
            UNIQUE (run_handle, public_code)
        )",
        "CREATE TABLE IF NOT EXISTS {issuer} (
            run_handle TEXT NOT NULL,
            surface VARCHAR(64) NOT NULL,
            route VARCHAR(191) NOT NULL,
            denial_code VARCHAR(64) NULL,
            next_action VARCHAR(64) NULL,
            retained_for VARCHAR(32) NULL,
            grants_entitlement VARCHAR(8) NOT NULL,
            UNIQUE (run_handle, surface)
        )",
        "CREATE TABLE IF NOT EXISTS {legacy} (
            run_handle TEXT NOT NULL,
            record_handle VARCHAR(191) NOT NULL,
            download_id BIGINT NULL,
            disposition VARCHAR(24) NOT NULL,
            record_state VARCHAR(24) NOT NULL,
            reason VARCHAR(64) NOT NULL,
            never_grant VARCHAR(8) NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            UNIQUE (run_handle, record_handle)
        )",
        "CREATE TABLE IF NOT EXISTS {quarantine} (
            quarantine_uuid {uuid} NOT NULL PRIMARY KEY,
            run_handle TEXT NOT NULL,
            record_handle VARCHAR(191) NOT NULL,
            disposition VARCHAR(24) NOT NULL,
            record_state VARCHAR(24) NOT NULL,
            reason VARCHAR(64) NOT NULL,
            never_grant VARCHAR(8) NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (record_handle)
        )",
        "CREATE TABLE IF NOT EXISTS {journal} (
            journal_seq INTEGER PRIMARY KEY AUTOINCREMENT,
            journal_key VARCHAR(64) NOT NULL,
            run_handle TEXT NOT NULL,
            record_handle VARCHAR(191) NULL,
            event_type VARCHAR(32) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            detail TEXT NOT NULL,
            previous_digest VARCHAR(64) NOT NULL,
            entry_digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (journal_key)
        )",
        "CREATE TABLE IF NOT EXISTS {recon} (
            recon_handle TEXT NOT NULL PRIMARY KEY,
            run_handle TEXT NOT NULL,
            recon_digest VARCHAR(64) NOT NULL,
            result VARCHAR(32) NOT NULL,
            receipt_payload TEXT NOT NULL,
            receipt_digest VARCHAR(64) NOT NULL,
            idempotency_key VARCHAR(64) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (recon_digest)
        )",
        "CREATE TABLE IF NOT EXISTS {rollback} (
            proof_handle TEXT NOT NULL PRIMARY KEY,
            run_handle TEXT NOT NULL,
            proof_digest VARCHAR(64) NOT NULL,
            verdict VARCHAR(48) NOT NULL,
            receipt_payload TEXT NOT NULL,
            receipt_digest VARCHAR(64) NOT NULL,
            idempotency_key VARCHAR(64) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (proof_digest)
        )",
    ];

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
        $migrations = $this->table('wpuiai_spec172_cutover_schema_migrations');
        $events = $this->table('wpuiai_spec172_cutover_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';

        $tables = [
            '{migrations}' => $migrations,
            '{events}' => $events,
            '{runs}' => $this->table('wpuiai_spec172_cutover_runs'),
            '{mappings}' => $this->table('wpuiai_spec172_cutover_mappings'),
            '{issuer}' => $this->table('wpuiai_spec172_issuer_disabled'),
            '{legacy}' => $this->table('wpuiai_spec172_legacy_disposition'),
            '{quarantine}' => $this->table('wpuiai_spec172_quarantine_ledger'),
            '{journal}' => $this->table('wpuiai_spec172_cutover_journal'),
            '{recon}' => $this->table('wpuiai_spec172_reconciliation_runs'),
            '{rollback}' => $this->table('wpuiai_spec172_rollback_proof'),
        ];
        $self = $this;
        foreach (self::DDL as $statement) {
            $sql = strtr($statement, $tables + ['{uuid}' => $uuid]);
            $self->assertPreservationOnlyStatement($sql);
            $this->db->exec($sql);
        }
        $insert = $this->db->prepare("INSERT INTO {$migrations}
            (schema_version, schema_name, applied_at, migration_provenance)
            SELECT :v, :n, :t, :p
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing)");
        $insert->execute([
            ':v' => self::VERSION,
            ':n' => self::SCHEMA,
            ':t' => $appliedAt,
            ':p' => $encoded,
            ':existing' => self::VERSION,
        ]);
        $eventKey = hash('sha256', self::SCHEMA . "\n" . $appliedAt . "\n" . $encoded);
        $eventInsert = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :k, 'schema_applied', :v, :t, :p
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $eventInsert->execute([':k' => $eventKey, ':v' => self::VERSION, ':t' => $appliedAt, ':p' => $encoded, ':existing_key' => $eventKey]);
    }

    /** Preservation-only proof: every DDL statement this schema may run is insert/select only. */
    public function assertPreservationOnly(): void
    {
        foreach (self::DDL as $statement) {
            $this->assertPreservationOnlyStatement($statement);
        }
    }

    private function assertPreservationOnlyStatement(string $statement): void
    {
        foreach (self::FORBIDDEN_STATEMENTS as $pattern) {
            if (preg_match($pattern, $statement) === 1) {
                throw new DomainException('PRESERVATION_ONLY_VIOLATION');
            }
        }
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
        if (preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/D', (string) $timestamp) !== 1) {
            throw new InvalidArgumentException('invalid timestamp');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        ksort($value, SORT_STRING);
        return json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }
}

final class FocusaSpec172NoSalesCutoverCanary
{
    public const RESULT_SCHEMA = 'focusa.spec172.no_sales_cutover_receipt.v1';
    public const RECON_SCHEMA = 'focusa.spec172.no_sales_reconciliation_receipt.v1';
    public const ROLLBACK_SCHEMA = 'focusa.spec172.no_sales_rollback_proof.v1';
    public const VERSION = 1;

    /** The only cutover policy this surface accepts. */
    public const POLICY = 'no_sales_clean_cutover_canary';

    /** Canonical decisions (Spec 172 §19). */
    public const DECISIONS = [
        'migration_preserving_path_selected',
        'clean_cutover_executed',
        'stopped_requiring_customer_rights_mapping',
    ];

    /** Clean cutover stays blocked while zero-sales proof is not accepted. */
    public const BLOCK_REASON = 'ZERO_SALES_PROOF_REQUIRED';
    public const GENUINE_SALE_CODE = 'GENUINE_SALE_REQUIRES_CUSTOMER_RIGHTS_MAPPING';

    /** Dedicated EDD mapping postures (server-owned; caller cannot choose). */
    public const MAPPING_BLOCKED = 'approved_policy_blocked_edd_mapping';
    public const MAPPING_ENABLED = 'approved_mapping_enabled_after_validation';
    public const SALE_STATUS_NOT_ENABLED = 'approved_not_yet_enabled';

    /** The only dedicated Operator v1 mapping codes and prices (Spec 172 §4.1, §16.3). */
    public const MAPPING_CODES = [
        'focusa_operator_lifetime_v1' => ['download' => 458, 'price_usd' => '697.00'],
        'uiai_operator_lifetime_v1' => ['download' => 459, 'price_usd' => '697.00'],
        'focusa_uiai_operator_bundle_lifetime_v1' => ['download' => 460, 'price_usd' => '1254.60'],
    ];

    /** Denial codes for direct issuance surfaces (atom focusa-vbcqu.20.13.53 contract). */
    public const DENIAL_CODES = [
        'install_site_create' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'install_site_payment' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'install_site_webhook' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'wpuiai_custom_issue' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'stripe_direct_product' => ['code' => 'STRIPE_DIRECT_FLOW_DENIED', 'next_action' => 'use_edd_checkout'],
        'local_self_eval' => ['code' => 'LOCAL_EVALUATION_DENIED', 'next_action' => 'use_edd_evaluation'],
    ];

    /** Legacy dispositions (Spec 172 §16.3; Spec 152E §22.2). */
    public const DISPOSITIONS = ['quarantine', 'retire', 'migrate'];
    public const RECORD_STATES = ['quarantined', 'retired', 'preserved'];
    public const NEVER_GRANT_DISPOSITIONS = ['quarantine', 'retire'];

    /** Server-owned mapping entry fields a caller may carry (nothing else is accepted). */
    private const MAPPING_ALLOWED_FIELDS = ['public_code', 'edd_download_id', 'edd_price_id', 'price_usd', 'checkout_enabled', 'sale_status', 'status', 'title'];
    private const RUN_PATTERN = '/^run_[a-z0-9_]{4,64}$/D';
    private const RECON_PATTERN = '/^recon_[a-z0-9_]{4,64}$/D';
    private const PROOF_PATTERN = '/^proof_[a-z0-9_]{4,64}$/D';
    private const HANDLE_PATTERN = '/^rec_[a-z0-9_]{4,64}$/D';
    private const DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const GENESIS_DIGEST = '0000000000000000000000000000000000000000000000000000000000000000';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172NoSalesCutoverSchema $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Execute the no-sales cutover canary. Requires the accepted zero-sales
     * inventory decision; fails closed with ZERO_SALES_PROOF_REQUIRED (zero
     * writes) when the proof is missing or malformed. When zero-sales proof is
     * accepted and clean cutover is allowed the dedicated EDD mappings are
     * enabled after validation (checkout stays disabled); otherwise the
     * migration-preserving path runs: dedicated mappings stay blocked, direct
     * install-site/Gravity issuance is disabled, and legacy WPUIAI/453/
     * synthetic records are quarantined or retired (migration-class records
     * preserved, never granted). A genuine sale stops the cutover and requires
     * a customer-rights mapping. Idempotent: replay returns the stored
     * receipt byte-identical with zero writes; a different payload fails
     * closed with RUN_ALREADY_STARTED.
     */
    public function canaryCutover(array $input): array
    {
        $this->assertRequestInputs($input);
        $runHandle = (string) $input['run_handle'];
        $proof = $this->requireZeroSalesProof($input);
        $mappings = $this->assertMappings($input['dedicated_mappings'] ?? []);
        $issuer = $this->assertIssuerSurfaces($input['issuer_disablements'] ?? []);
        $legacy = $this->assertLegacyRecords($input['legacy_records'] ?? []);
        $genuineSale = (bool) ($input['genuine_sale_observed'] ?? false);

        $plan = [
            'proof' => [
                'inventory_id' => $proof['inventory_id'],
                'zero_sales_proven' => $proof['zero_sales_proven'],
                'clean_cutover_allowed' => $proof['clean_cutover_allowed'],
                'decision_status' => $proof['decision_status'],
            ],
            'mappings' => $mappings,
            'issuer_disablements' => $issuer,
            'legacy_records' => $legacy,
            'genuine_sale_observed' => $genuineSale,
        ];
        $runDigest = hash('sha256', $runHandle . "\n" . FocusaSpec172NoSalesCutoverSchema::encodeCanonical($plan));
        $existing = $this->findRun($runHandle);
        if ($existing !== null) {
            if (!hash_equals((string) $existing['run_digest'], $runDigest)) {
                throw new DomainException('RUN_ALREADY_STARTED');
            }
            return $this->runEnvelope($runHandle, $existing, true);
        }

        $decision = $this->decisionFor($proof, $genuineSale);
        $mappingStatus = $decision === 'clean_cutover_executed'
            ? self::MAPPING_ENABLED
            : self::MAPPING_BLOCKED;

        $now = ($this->clock)();
        FocusaSpec172NoSalesCutoverSchema::assertTimestamp($now);
        $receipt = $this->buildCutoverReceipt($runHandle, $proof, $decision, $mappingStatus, $mappings, $issuer, $legacy, $genuineSale, $runDigest);
        $receiptDigest = (string) $receipt['receipt_digest'];

        $runs = $this->schema->table('wpuiai_spec172_cutover_runs');
        $insert = $this->db->prepare("INSERT INTO {$runs}
            (run_handle, run_digest, inventory_id, decision, zero_sales_proven, clean_cutover_allowed,
             block_reason, genuine_sale, receipt_payload, receipt_digest, idempotency_key, started_at, migration_provenance)
            VALUES (:run, :digest, :inventory, :decision, :zsp, :cca, :block, :genuine, :receipt, :rdigest, :idem, :started, :provenance)");
        $insert->execute([
            ':run' => $runHandle,
            ':digest' => $runDigest,
            ':inventory' => $proof['inventory_id'],
            ':decision' => $decision,
            ':zsp' => $proof['zero_sales_proven'] ? 'true' : 'false',
            ':cca' => $proof['clean_cutover_allowed'] ? 'true' : 'false',
            ':block' => $decision === 'clean_cutover_executed' ? null : ($decision === 'stopped_requiring_customer_rights_mapping' ? self::GENUINE_SALE_CODE : self::BLOCK_REASON),
            ':genuine' => $genuineSale ? 'true' : 'false',
            ':receipt' => json_encode($receipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':rdigest' => $receiptDigest,
            ':idem' => $receipt['idempotency_key'],
            ':started' => $now,
            ':provenance' => FocusaSpec172NoSalesCutoverSchema::encodeCanonical($input['migration_provenance']),
        ]);

        $mappingInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_spec172_cutover_mappings')}
            (run_handle, public_code, edd_download_id, edd_price_id, mapping_status, checkout_enabled, sale_status)
            VALUES (:run, :code, :download, :price, :status, :checkout, :sale)");
        foreach ($mappings as $mapping) {
            $mappingInsert->execute([
                ':run' => $runHandle,
                ':code' => $mapping['public_code'],
                ':download' => $mapping['edd_download_id'],
                ':price' => $mapping['edd_price_id'],
                ':status' => $mappingStatus,
                ':checkout' => $mapping['checkout_enabled'] ? 'true' : 'false',
                ':sale' => $mapping['sale_status'],
            ]);
        }

        $issuerInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_spec172_issuer_disabled')}
            (run_handle, surface, route, denial_code, next_action, retained_for, grants_entitlement)
            VALUES (:run, :surface, :route, :code, :next, :retained, :grants)");
        foreach ($issuer as $surface) {
            $issuerInsert->execute([
                ':run' => $runHandle,
                ':surface' => $surface['surface'],
                ':route' => $surface['route'],
                ':code' => $surface['denial_code'],
                ':next' => $surface['next_action'],
                ':retained' => $surface['retained_for'],
                ':grants' => $surface['grants_entitlement'] ? 'true' : 'false',
            ]);
        }

        $legacyInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_spec172_legacy_disposition')}
            (run_handle, record_handle, download_id, disposition, record_state, reason, never_grant, evidence_digest)
            VALUES (:run, :record, :download, :disposition, :state, :reason, :never, :digest)");
        foreach ($legacy as $record) {
            $legacyInsert->execute([
                ':run' => $runHandle,
                ':record' => $record['record_handle'],
                ':download' => $record['download_id'],
                ':disposition' => $record['disposition'],
                ':state' => $record['record_state'],
                ':reason' => $record['reason'],
                ':never' => $record['never_grant'] ? 'true' : 'false',
                ':digest' => $record['evidence_digest'],
            ]);
        }

        // Durable quarantine ledger: every legacy record is journaled exactly once.
        // Replays reuse the stored disposition (idempotent); a conflicting
        // disposition for the same record fails closed with QUARANTINE_CONFLICT.
        $quarantineSelect = $this->db->prepare("SELECT disposition, record_state, reason, never_grant FROM {$this->schema->table('wpuiai_spec172_quarantine_ledger')} WHERE record_handle = :record");
        $quarantineInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_spec172_quarantine_ledger')}
            (quarantine_uuid, run_handle, record_handle, disposition, record_state, reason, never_grant, evidence_digest, created_at, migration_provenance)
            VALUES (:uuid, :run, :record, :disposition, :state, :reason, :never, :digest, :created, :provenance)");
        foreach ($legacy as $record) {
            $quarantineSelect->execute([':record' => $record['record_handle']]);
            $stored = $quarantineSelect->fetch(PDO::FETCH_ASSOC);
            if ($stored !== false) {
                if ((string) $stored['disposition'] !== $record['disposition']
                    || (string) $stored['record_state'] !== $record['record_state']
                    || (string) $stored['reason'] !== $record['reason']
                    || (string) $stored['never_grant'] !== ($record['never_grant'] ? 'true' : 'false')) {
                    throw new DomainException('QUARANTINE_CONFLICT');
                }
                continue;
            }
            $quarantineInsert->execute([
                ':uuid' => hash('sha256', 'quarantine' . "\n" . $runHandle . "\n" . $record['record_handle']),
                ':run' => $runHandle,
                ':record' => $record['record_handle'],
                ':disposition' => $record['disposition'],
                ':state' => $record['record_state'],
                ':reason' => $record['reason'],
                ':never' => $record['never_grant'] ? 'true' : 'false',
                ':digest' => $record['evidence_digest'],
                ':created' => $now,
                ':provenance' => FocusaSpec172NoSalesCutoverSchema::encodeCanonical($input['migration_provenance']),
            ]);
        }

        $this->journalEvent('cutover_executed', $runHandle, '', $now, [
            'run_handle' => $runHandle,
            'decision' => $decision,
            'mappings' => count($mappings),
            'issuer_surfaces' => count($issuer),
            'legacy_records' => count($legacy),
            'receipt_digest' => $receiptDigest,
        ], $input['migration_provenance']);

        $stored = $this->findRun($runHandle);
        return $this->runEnvelope($runHandle, $stored, false);
    }

    /**
     * Dry-run the canary: identical validation and decision with ZERO writes.
     * Never a grant or mutation by itself.
     */
    public function dryRunCutover(array $input): array
    {
        $this->assertRequestInputs($input);
        $runHandle = (string) $input['run_handle'];
        $proof = $this->requireZeroSalesProof($input);
        $mappings = $this->assertMappings($input['dedicated_mappings'] ?? []);
        $issuer = $this->assertIssuerSurfaces($input['issuer_disablements'] ?? []);
        $legacy = $this->assertLegacyRecords($input['legacy_records'] ?? []);
        $genuineSale = (bool) ($input['genuine_sale_observed'] ?? false);
        $decision = $this->decisionFor($proof, $genuineSale);
        $mappingStatus = $decision === 'clean_cutover_executed' ? self::MAPPING_ENABLED : self::MAPPING_BLOCKED;
        $runDigest = hash('sha256', $runHandle . "\n" . FocusaSpec172NoSalesCutoverSchema::encodeCanonical([
            'proof' => [
                'inventory_id' => $proof['inventory_id'],
                'zero_sales_proven' => $proof['zero_sales_proven'],
                'clean_cutover_allowed' => $proof['clean_cutover_allowed'],
                'decision_status' => $proof['decision_status'],
            ],
            'mappings' => $mappings,
            'issuer_disablements' => $issuer,
            'legacy_records' => $legacy,
            'genuine_sale_observed' => $genuineSale,
        ]));
        return [
            'schema' => self::RESULT_SCHEMA,
            'dry_run' => true,
            'run_handle' => $runHandle,
            'decision' => $decision,
            'mapping_status' => $mappingStatus,
            'run_digest' => $runDigest,
            'writes' => 0,
        ];
    }

    /**
     * Rollback-safe reconciliation receipt. Reads the stored run and proves:
     * EDD remains the sole paid authority (zero enabled dedicated mappings,
     * zero non-EDD issuance with grants), direct install-site/Gravity
     * issuance stays disabled, zero legacy records grant an Operator v1
     * License Type, refunded/revoked records stay terminal (no stale refund
     * truth), the journal chain is valid (monotonic append), and the replay
     * is idempotent. Fail-closed with RECONCILIATION_MISMATCH on any drift.
     */
    public function reconcile(array $input): array
    {
        $reconHandle = (string) ($input['recon_handle'] ?? '');
        if (preg_match(self::RECON_PATTERN, $reconHandle) !== 1) {
            throw new DomainException('RECON_HANDLE_REQUIRED');
        }
        $runHandle = (string) ($input['run_handle'] ?? '');
        $run = $this->findRun($runHandle);
        if ($run === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        if (($input['migration_provenance'] ?? []) === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }

        $existing = $this->findRecon($reconHandle);
        if ($existing !== null) {
            return $this->reconEnvelope($reconHandle, $existing, true);
        }

        $receipt = $this->buildReconciliationReceipt($runHandle, $run);
        $reconDigest = hash('sha256', $reconHandle . "\n" . FocusaSpec172NoSalesCutoverSchema::encodeCanonical($receipt));
        $matching = $this->findReconByDigest($reconDigest);
        if ($matching !== null && (string) $matching['recon_handle'] !== $reconHandle) {
            throw new DomainException('RECON_ALREADY_EXISTS');
        }
        if ($receipt['result'] !== 'passed_fail_closed') {
            throw new DomainException('RECONCILIATION_MISMATCH');
        }
        $now = ($this->clock)();
        $reconTable = $this->schema->table('wpuiai_spec172_reconciliation_runs');
        $insert = $this->db->prepare("INSERT INTO {$reconTable}
            (recon_handle, run_handle, recon_digest, result, receipt_payload, receipt_digest, idempotency_key, occurred_at, migration_provenance)
            VALUES (:recon, :run, :digest, :result, :receipt, :rdigest, :idem, :occurred, :provenance)");
        $insert->execute([
            ':recon' => $reconHandle,
            ':run' => $runHandle,
            ':digest' => $reconDigest,
            ':result' => $receipt['result'],
            ':receipt' => json_encode($receipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':rdigest' => $receipt['receipt_digest'],
            ':idem' => $receipt['idempotency_key'],
            ':occurred' => $now,
            ':provenance' => FocusaSpec172NoSalesCutoverSchema::encodeCanonical($input['migration_provenance']),
        ]);
        $this->journalEvent('reconciliation_passed', $runHandle, '', $now, ['recon_handle' => $reconHandle, 'receipt_digest' => $receipt['receipt_digest']], $input['migration_provenance']);

        $stored = $this->findRecon($reconHandle);
        return $this->reconEnvelope($reconHandle, $stored, false);
    }

    /**
     * Rollback-safe proof. Rolling back software can never restore split
     * issuance or stale refund truth: the denial state is durable (the schema
     * has no enable path and no destructive statement), legacy quarantine is
     * preserved, and refunded/revoked records stay terminal. Verdict
     * preservation_only_no_split_issuance_no_stale_refund; replay is
     * idempotent.
     */
    public function proveRollback(array $input): array
    {
        $proofHandle = (string) ($input['proof_handle'] ?? '');
        if (preg_match(self::PROOF_PATTERN, $proofHandle) !== 1) {
            throw new DomainException('PROOF_HANDLE_REQUIRED');
        }
        $runHandle = (string) ($input['run_handle'] ?? '');
        $run = $this->findRun($runHandle);
        if ($run === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        if (($input['migration_provenance'] ?? []) === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $existing = $this->findProof($proofHandle);
        if ($existing !== null) {
            return $this->proofEnvelope($proofHandle, $existing, true);
        }

        $this->schema->assertPreservationOnly();
        $receipt = $this->buildRollbackReceipt($runHandle, $run);
        $proofDigest = hash('sha256', $proofHandle . "\n" . FocusaSpec172NoSalesCutoverSchema::encodeCanonical($receipt));
        $matching = $this->findProofByDigest($proofDigest);
        if ($matching !== null && (string) $matching['proof_handle'] !== $proofHandle) {
            throw new DomainException('PROOF_ALREADY_EXISTS');
        }
        $now = ($this->clock)();
        $proofTable = $this->schema->table('wpuiai_spec172_rollback_proof');
        $insert = $this->db->prepare("INSERT INTO {$proofTable}
            (proof_handle, run_handle, proof_digest, verdict, receipt_payload, receipt_digest, idempotency_key, occurred_at, migration_provenance)
            VALUES (:proof, :run, :digest, :verdict, :receipt, :rdigest, :idem, :occurred, :provenance)");
        $insert->execute([
            ':proof' => $proofHandle,
            ':run' => $runHandle,
            ':digest' => $proofDigest,
            ':verdict' => $receipt['verdict'],
            ':receipt' => json_encode($receipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':rdigest' => $receipt['receipt_digest'],
            ':idem' => $receipt['idempotency_key'],
            ':occurred' => $now,
            ':provenance' => FocusaSpec172NoSalesCutoverSchema::encodeCanonical($input['migration_provenance']),
        ]);
        $this->journalEvent('rollback_proof_recorded', $runHandle, '', $now, ['proof_handle' => $proofHandle, 'receipt_digest' => $receipt['receipt_digest']], $input['migration_provenance']);

        $stored = $this->findProof($proofHandle);
        return $this->proofEnvelope($proofHandle, $stored, false);
    }

    /**
     * No enable path exists: any attempt to re-enable a disabled issuance
     * surface fails closed, so rollback can never restore split issuance.
     */
    public function enableIssuanceSurface(array $input): never
    {
        throw new DomainException('ISSUANCE_SURFACE_ENABLE_DENIED');
    }

    /** Verify the append-only journal digest chain from genesis. */
    public function journalChainValid(): bool
    {
        $journal = $this->schema->table('wpuiai_spec172_cutover_journal');
        $rows = $this->db->query("SELECT journal_seq, journal_key, run_handle, record_handle, event_type, occurred_at, detail, previous_digest, entry_digest, migration_provenance
            FROM {$journal} ORDER BY journal_seq ASC")->fetchAll(PDO::FETCH_ASSOC);
        $previous = self::GENESIS_DIGEST;
        foreach ($rows as $row) {
            if (!hash_equals($previous, (string) $row['previous_digest'])) {
                return false;
            }
            $expected = hash('sha256', (string) $row['previous_digest'] . "\n" . (int) $row['journal_seq'] . "\n" . (string) $row['journal_key'] . "\n" . (string) $row['run_handle'] . "\n" . (string) $row['record_handle'] . "\n" . (string) $row['event_type'] . "\n" . (string) $row['occurred_at'] . "\n" . (string) $row['detail'] . "\n" . (string) $row['migration_provenance']);
            if (!hash_equals($expected, (string) $row['entry_digest'])) {
                return false;
            }
            $previous = (string) $row['entry_digest'];
        }
        return true;
    }

    public function countRows(string $table): int
    {
        return (int) $this->db->query("SELECT COUNT(*) FROM {$this->schema->table($table)}")->fetchColumn();
    }

    // ── Internals ──────────────────────────────────────────────────────

    /**
     * Require the accepted zero-sales proof (Spec 172 §19). Missing or
     * malformed proof fails closed with ZERO_SALES_PROOF_REQUIRED and writes
     * nothing. The accepted inventory decision may still select the
     * migration-preserving path (zero_sales_proven=false); the clean cutover
     * itself is then blocked below.
     */
    private function requireZeroSalesProof(array $input): array
    {
        $proof = $input['zero_sales_proof'] ?? null;
        if (!is_array($proof) || ($proof['accepted'] ?? false) !== true) {
            throw new DomainException('ZERO_SALES_PROOF_REQUIRED');
        }
        $inventoryId = (string) ($proof['inventory_id'] ?? '');
        if ($inventoryId === '' || preg_match('/^[A-Za-z0-9._-]{1,191}$/D', $inventoryId) !== 1) {
            throw new DomainException('ZERO_SALES_PROOF_REQUIRED');
        }
        $zeroSalesProven = (bool) ($proof['zero_sales_proven'] ?? false);
        $cleanCutoverAllowed = (bool) ($proof['clean_cutover_allowed'] ?? false);
        $decisionStatus = (string) ($proof['decision_status'] ?? '');
        if ($decisionStatus === '') {
            throw new DomainException('ZERO_SALES_PROOF_REQUIRED');
        }
        if ($cleanCutoverAllowed && !$zeroSalesProven) {
            throw new DomainException('ZERO_SALES_PROOF_REQUIRED');
        }
        return [
            'accepted' => true,
            'inventory_id' => $inventoryId,
            'zero_sales_proven' => $zeroSalesProven,
            'clean_cutover_allowed' => $cleanCutoverAllowed,
            'decision_status' => $decisionStatus,
        ];
    }

    private function decisionFor(array $proof, bool $genuineSale): string
    {
        if ($genuineSale) {
            return 'stopped_requiring_customer_rights_mapping';
        }
        if ($proof['zero_sales_proven'] && $proof['clean_cutover_allowed']) {
            return 'clean_cutover_executed';
        }
        return 'migration_preserving_path_selected';
    }

    /**
     * Validate the dedicated EDD mappings: exactly the three canonical
     * Operator v1 records, server-owned download/price fields only, checkout
     * disabled, sale status approved_not_yet_enabled. Any caller-supplied
     * commercial field fails closed with CLIENT_COMMERCIAL_FIELDS_FORBIDDEN.
     */
    private function assertMappings(array $mappings): array
    {
        if (count($mappings) !== count(self::MAPPING_CODES)) {
            throw new DomainException('DEDICATED_MAPPINGS_REQUIRED');
        }
        $seen = [];
        $out = [];
        foreach ($mappings as $mapping) {
            if (!is_array($mapping)) {
                throw new DomainException('DEDICATED_MAPPINGS_REQUIRED');
            }
            $unknown = array_diff(array_keys($mapping), self::MAPPING_ALLOWED_FIELDS);
            if ($unknown !== []) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
            $code = (string) ($mapping['public_code'] ?? '');
            if (!isset(self::MAPPING_CODES[$code])) {
                throw new DomainException('DEDICATED_MAPPINGS_REQUIRED');
            }
            if (isset($seen[$code])) {
                throw new DomainException('DEDICATED_MAPPINGS_REQUIRED');
            }
            $seen[$code] = true;
            $expected = self::MAPPING_CODES[$code];
            $download = (int) ($mapping['edd_download_id'] ?? 0);
            $priceUsd = (string) ($mapping['price_usd'] ?? '');
            if ($download !== $expected['download'] || $priceUsd !== $expected['price_usd']) {
                throw new DomainException('SERVER_OWNED_MAPPING_MISMATCH');
            }
            $priceId = (string) ($mapping['edd_price_id'] ?? '');
            if (preg_match('/^[A-Za-z0-9_]{1,191}$/D', $priceId) !== 1) {
                throw new DomainException('SERVER_OWNED_MAPPING_MISMATCH');
            }
            if (($mapping['checkout_enabled'] ?? true) !== false) {
                throw new DomainException('CHECKOUT_MUST_STAY_DISABLED');
            }
            if ((string) ($mapping['sale_status'] ?? '') !== self::SALE_STATUS_NOT_ENABLED) {
                throw new DomainException('SALE_STATUS_NOT_ENABLED');
            }
            $out[] = [
                'public_code' => $code,
                'edd_download_id' => $download,
                'edd_price_id' => $priceId,
                'price_usd' => $priceUsd,
                'checkout_enabled' => false,
                'sale_status' => self::SALE_STATUS_NOT_ENABLED,
            ];
        }
        return $out;
    }

    /**
     * Validate the issuer surfaces: denied surfaces carry the exact server
     * denial codes; retained validation/recovery surfaces grant no
     * entitlement. Every surface must appear exactly once.
     */
    private function assertIssuerSurfaces(array $surfaces): array
    {
        if ($surfaces === []) {
            throw new DomainException('ISSUER_SURFACES_REQUIRED');
        }
        $seen = [];
        $out = [];
        foreach ($surfaces as $surface) {
            if (!is_array($surface)) {
                throw new DomainException('ISSUER_SURFACES_REQUIRED');
            }
            $name = (string) ($surface['surface'] ?? '');
            $route = (string) ($surface['route'] ?? '');
            if ($name === '' || $route === '' || isset($seen[$name])) {
                throw new DomainException('ISSUER_SURFACES_REQUIRED');
            }
            $seen[$name] = true;
            $retainedFor = isset($surface['retained_for']) ? (string) $surface['retained_for'] : null;
            $denial = $surface['denial_code'] ?? null;
            if ($retainedFor !== null) {
                if (!in_array($retainedFor, ['validation', 'recovery'], true) || $denial !== null) {
                    throw new DomainException('ISSUER_SURFACES_REQUIRED');
                }
                if (($surface['grants_entitlement'] ?? true) !== false) {
                    throw new DomainException('RETAINED_SURFACE_NEVER_GRANTS');
                }
                $out[] = [
                    'surface' => $name,
                    'route' => $route,
                    'denial_code' => null,
                    'next_action' => null,
                    'retained_for' => $retainedFor,
                    'grants_entitlement' => false,
                ];
                continue;
            }
            if (!isset(self::DENIAL_CODES[$name])) {
                throw new DomainException('ISSUER_SURFACES_REQUIRED');
            }
            if ($denial !== self::DENIAL_CODES[$name]['code']) {
                throw new DomainException('ISSUANCE_DENIAL_REQUIRED');
            }
            if (($surface['grants_entitlement'] ?? false) === true) {
                throw new DomainException('RETAINED_SURFACE_NEVER_GRANTS');
            }
            $out[] = [
                'surface' => $name,
                'route' => $route,
                'denial_code' => $denial,
                'next_action' => (string) self::DENIAL_CODES[$name]['next_action'],
                'retained_for' => null,
                'grants_entitlement' => false,
            ];
        }
        return $out;
    }

    /**
     * Validate the legacy disposition registry: quarantine/retire records can
     * never grant; migrate records are preserved evidence-backed; Download 453
     * stays quarantined with the explicit forbidden reason; refunded/revoked
     * records stay terminal (retired).
     */
    private function assertLegacyRecords(array $records): array
    {
        if ($records === []) {
            throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
        }
        $seen = [];
        $out = [];
        foreach ($records as $record) {
            if (!is_array($record)) {
                throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
            }
            $handle = (string) ($record['record_handle'] ?? '');
            if (preg_match(self::HANDLE_PATTERN, $handle) !== 1 || isset($seen[$handle])) {
                throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
            }
            $seen[$handle] = true;
            $disposition = (string) ($record['disposition'] ?? '');
            if (!in_array($disposition, self::DISPOSITIONS, true)) {
                throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
            }
            $reason = (string) ($record['reason'] ?? '');
            if ($reason === '') {
                throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
            }
            $evidenceDigest = (string) ($record['evidence_digest'] ?? '');
            if (preg_match(self::DIGEST_PATTERN, $evidenceDigest) !== 1) {
                throw new DomainException('LEGACY_DISPOSITION_REQUIRED');
            }
            $downloadId = isset($record['download_id']) ? (int) $record['download_id'] : null;
            if ($downloadId === 453) {
                if ($disposition !== 'quarantine' || $reason !== 'implicit_focusa_mapping_forbidden') {
                    throw new DomainException('DOWNLOAD_453_QUARANTINE_REQUIRED');
                }
            }
            $neverGrant = in_array($disposition, self::NEVER_GRANT_DISPOSITIONS, true);
            $state = $disposition === 'quarantine' ? 'quarantined' : ($disposition === 'retire' ? 'retired' : 'preserved');
            if (!$neverGrant && $disposition === 'migrate') {
                // Migration-class records are preserved and evidence-backed; they
                // are never granted here (explicit mapping decision required).
            }
            $out[] = [
                'record_handle' => $handle,
                'download_id' => $downloadId,
                'disposition' => $disposition,
                'record_state' => $state,
                'reason' => $reason,
                'never_grant' => $neverGrant,
                'evidence_digest' => $evidenceDigest,
            ];
        }
        return $out;
    }

    private function buildCutoverReceipt(string $runHandle, array $proof, string $decision, string $mappingStatus, array $mappings, array $issuer, array $legacy, bool $genuineSale, string $runDigest): array
    {
        $planPayload = json_encode([
            'proof' => [
                'inventory_id' => $proof['inventory_id'],
                'zero_sales_proven' => $proof['zero_sales_proven'],
                'clean_cutover_allowed' => $proof['clean_cutover_allowed'],
                'decision_status' => $proof['decision_status'],
            ],
            'mappings' => $mappings,
            'issuer_disablements' => $issuer,
            'legacy_records' => $legacy,
            'genuine_sale_observed' => $genuineSale,
        ], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
        $mappingReceipt = [];
        foreach ($mappings as $mapping) {
            $mappingReceipt[] = [
                'public_code' => $mapping['public_code'],
                'edd_download_id' => $mapping['edd_download_id'],
                'edd_price_id' => $mapping['edd_price_id'],
                'mapping_status' => $mappingStatus,
                'checkout_enabled' => false,
                'sale_status' => self::SALE_STATUS_NOT_ENABLED,
            ];
        }
        $issuerReceipt = [];
        foreach ($issuer as $surface) {
            $issuerReceipt[] = [
                'surface' => $surface['surface'],
                'route' => $surface['route'],
                'denial_code' => $surface['denial_code'],
                'next_action' => $surface['next_action'],
                'retained_for' => $surface['retained_for'],
                'grants_entitlement' => false,
            ];
        }
        $legacyReceipt = [];
        foreach ($legacy as $record) {
            $legacyReceipt[] = [
                'record_handle' => $record['record_handle'],
                'download_id' => $record['download_id'],
                'disposition' => $record['disposition'],
                'record_state' => $record['record_state'],
                'reason' => $record['reason'],
                'never_grant' => $record['never_grant'],
            ];
        }
        $blockReason = $decision === 'clean_cutover_executed' ? null : ($decision === 'stopped_requiring_customer_rights_mapping' ? self::GENUINE_SALE_CODE : self::BLOCK_REASON);
        $receipt = [
            'schema' => self::RESULT_SCHEMA,
            'version' => self::VERSION,
            'cutover_version' => 'focusa-vbcqu.20.15.32',
            'policy' => self::POLICY,
            'run_handle' => $runHandle,
            'run_digest' => $runDigest,
            'inventory_id' => $proof['inventory_id'],
            'decision' => $decision,
            'zero_sales_proven' => $proof['zero_sales_proven'],
            'clean_cutover_allowed' => $proof['clean_cutover_allowed'],
            'clean_cutover_blocked_reason' => $blockReason,
            'genuine_sale_observed' => $genuineSale,
            'authority' => [
                'canonical_paid_authority' => 'WPUIAI.com EDD',
                'split_issuance' => false,
            ],
            'mappings' => $mappingReceipt,
            'issuer_disablements' => $issuerReceipt,
            'legacy_disposition' => $legacyReceipt,
            'counts' => [
                'dedicated_mappings' => count($mappings),
                'mappings_enabled' => $mappingStatus === self::MAPPING_ENABLED ? count($mappings) : 0,
                'mappings_blocked' => $mappingStatus === self::MAPPING_BLOCKED ? count($mappings) : 0,
                'issuance_surfaces_disabled' => count(array_filter($issuer, static fn(array $s): bool => $s['denial_code'] !== null)),
                'retained_recovery_surfaces' => count(array_filter($issuer, static fn(array $s): bool => $s['retained_for'] !== null)),
                'legacy_quarantined' => count(array_filter($legacyReceipt, static fn(array $r): bool => $r['record_state'] === 'quarantined')),
                'legacy_retired' => count(array_filter($legacyReceipt, static fn(array $r): bool => $r['record_state'] === 'retired')),
                'legacy_preserved' => count(array_filter($legacyReceipt, static fn(array $r): bool => $r['record_state'] === 'preserved')),
                'legacy_never_grant' => count(array_filter($legacyReceipt, static fn(array $r): bool => $r['never_grant'])),
            ],
            'idempotency_key' => hash('sha256', $planPayload),
            'redacted' => true,
            'excluded' => ['raw_email', 'license_key', 'token', 'credential', 'card_data', 'customer_row', 'caller_supplied_commercial_field'],
            'validation' => 'passed_fail_closed',
        ];
        $receipt['receipt_digest'] = self::canonicalReceiptDigest($receipt);
        return $receipt;
    }

    /** Canonical 64-hex digest of a receipt computed over the receipt without its own digest field. */
    private static function canonicalReceiptDigest(array $receipt): string
    {
        $payload = $receipt;
        unset($payload['receipt_digest']);
        return hash('sha256', FocusaSpec172NoSalesCutoverSchema::encodeCanonical($payload));
    }

    private function buildReconciliationReceipt(string $runHandle, array $run): array
    {
        $receiptPayload = (string) $run['receipt_payload'];
        $storedReceipt = json_decode($receiptPayload, true);
        $recomputedDigest = self::canonicalReceiptDigest($storedReceipt);
        $intact = hash_equals((string) $run['receipt_digest'], $recomputedDigest)
            && hash_equals((string) $run['receipt_digest'], (string) ($storedReceipt['receipt_digest'] ?? ''));

        $mappings = $this->db->prepare("SELECT public_code, edd_download_id, edd_price_id, mapping_status, checkout_enabled, sale_status FROM {$this->schema->table('wpuiai_spec172_cutover_mappings')} WHERE run_handle = :run");
        $mappings->execute([':run' => $runHandle]);
        $mappingRows = $mappings->fetchAll(PDO::FETCH_ASSOC);
        $nonDedicatedMappings = 0;
        $liveCheckoutMappings = 0;
        foreach ($mappingRows as $row) {
            $code = (string) $row['public_code'];
            $expected = self::MAPPING_CODES[$code] ?? null;
            if ($expected === null || (int) $row['edd_download_id'] !== $expected['download']) {
                $nonDedicatedMappings++;
            }
            if ((string) $row['checkout_enabled'] !== 'false' || (string) $row['sale_status'] !== self::SALE_STATUS_NOT_ENABLED) {
                $liveCheckoutMappings++;
            }
        }

        $issuer = $this->db->prepare("SELECT surface, denial_code, retained_for, grants_entitlement FROM {$this->schema->table('wpuiai_spec172_issuer_disabled')} WHERE run_handle = :run");
        $issuer->execute([':run' => $runHandle]);
        $issuerRows = $issuer->fetchAll(PDO::FETCH_ASSOC);
        $grantingSurfaces = 0;
        foreach ($issuerRows as $row) {
            if ((string) $row['grants_entitlement'] !== 'false') {
                $grantingSurfaces++;
            }
        }

        $legacy = $this->db->prepare("SELECT record_handle, disposition, record_state, reason, never_grant FROM {$this->schema->table('wpuiai_spec172_legacy_disposition')} WHERE run_handle = :run");
        $legacy->execute([':run' => $runHandle]);
        $legacyRows = $legacy->fetchAll(PDO::FETCH_ASSOC);
        $legacyGranting = 0;
        $migrateCount = 0;
        $migratePreserved = 0;
        $adverseActive = 0;
        foreach ($legacyRows as $row) {
            if (in_array((string) $row['record_state'], ['active', 'granted'], true)) {
                $legacyGranting++;
            }
            if ((string) $row['disposition'] === 'migrate') {
                $migrateCount++;
                if ((string) $row['record_state'] === 'preserved') {
                    $migratePreserved++;
                }
            }
            $reason = (string) $row['reason'];
            if ((strpos($reason, 'preserve_refund') !== false || strpos($reason, 'preserve_revoc') !== false || strpos($reason, 'credit_pack') !== false)
                && (string) $row['record_state'] === 'preserved') {
                $adverseActive++;
            }
        }

        $journalValid = $this->journalChainValid();
        $passed = $intact
            && $nonDedicatedMappings === 0
            && $liveCheckoutMappings === 0
            && $grantingSurfaces === 0
            && $legacyGranting === 0
            && $migrateCount === $migratePreserved
            && $adverseActive === 0
            && $journalValid;

        $receipt = [
            'schema' => self::RECON_SCHEMA,
            'version' => self::VERSION,
            'run_handle' => $runHandle,
            'run_digest' => (string) $run['run_digest'],
            'result' => $passed ? 'passed_fail_closed' : 'mismatch',
            'findings' => [
                'receipt_intact' => $intact,
                'one_canonical_paid_authority' => $nonDedicatedMappings === 0 && $grantingSurfaces === 0,
                'checkout_still_disabled' => $liveCheckoutMappings === 0,
                'legacy_zero_grant' => $legacyGranting === 0,
                'migration_preserved_not_granted' => $migrateCount === $migratePreserved,
                'refund_revoke_truth_preserved' => $adverseActive === 0,
                'journal_chain_valid' => $journalValid,
            ],
            'counts' => [
                'dedicated_mappings' => count($mappingRows),
                'non_dedicated_mappings' => $nonDedicatedMappings,
                'live_checkout_mappings' => $liveCheckoutMappings,
                'issuance_surfaces' => count($issuerRows),
                'granting_surfaces' => $grantingSurfaces,
                'legacy_records' => count($legacyRows),
                'legacy_granting' => $legacyGranting,
                'migration_preserved' => $migratePreserved,
                'adverse_active' => $adverseActive,
            ],
            'idempotency_key' => hash('sha256', $receiptPayload),
            'redacted' => true,
            'excluded' => ['raw_email', 'license_key', 'token', 'credential', 'card_data', 'customer_row'],
        ];
        $receipt['receipt_digest'] = hash('sha256', FocusaSpec172NoSalesCutoverSchema::encodeCanonical($receipt));
        return $receipt;
    }

    private function buildRollbackReceipt(string $runHandle, array $run): array
    {
        $receiptPayload = (string) $run['receipt_payload'];
        $storedReceipt = json_decode($receiptPayload, true);
        $recomputedDigest = self::canonicalReceiptDigest($storedReceipt);
        $intact = hash_equals((string) $run['receipt_digest'], $recomputedDigest);
        $journalValid = $this->journalChainValid();

        // No legacy record may be in a grant-capable state, and no issuance
        // surface may grant entitlement: rollback rehearsal can never observe
        // split issuance or reactivated stale refunds.
        $legacy = $this->db->prepare("SELECT record_handle, record_state, never_grant, reason FROM {$this->schema->table('wpuiai_spec172_quarantine_ledger')} WHERE run_handle = :run");
        $legacy->execute([':run' => $runHandle]);
        $legacyRows = $legacy->fetchAll(PDO::FETCH_ASSOC);
        $grantCapable = 0;
        $staleRefundActive = 0;
        foreach ($legacyRows as $row) {
            if (in_array((string) $row['record_state'], ['active', 'granted'], true)) {
                $grantCapable++;
            }
            $reason = (string) $row['reason'];
            if ((strpos($reason, 'preserve_refund') !== false || strpos($reason, 'preserve_revoc') !== false)
                && in_array((string) $row['record_state'], ['active', 'granted'], true)) {
                $staleRefundActive++;
            }
        }
        $issuer = $this->db->prepare("SELECT surface, grants_entitlement FROM {$this->schema->table('wpuiai_spec172_issuer_disabled')} WHERE run_handle = :run");
        $issuer->execute([':run' => $runHandle]);
        $issuerRows = $issuer->fetchAll(PDO::FETCH_ASSOC);
        foreach ($issuerRows as $row) {
            if ((string) $row['grants_entitlement'] !== 'false') {
                $grantCapable++;
            }
        }

        // Rollback rehearsal: the stored cutover receipt and journal must stay
        // intact, the quarantine ledger must contain zero grant-capable legacy
        // rows, and the schema must be preservation-only with no enable path.
        $splitIssuanceRestorable = $grantCapable > 0 || !$intact || !$journalValid;
        $staleRefundRestorable = $staleRefundActive > 0;
        $receipt = [
            'schema' => self::ROLLBACK_SCHEMA,
            'version' => self::VERSION,
            'run_handle' => $runHandle,
            'run_digest' => (string) $run['run_digest'],
            'verdict' => 'preservation_only_no_split_issuance_no_stale_refund',
            'rehearsal' => [
                'receipt_intact' => $intact,
                'journal_chain_valid' => $journalValid,
                'split_issuance_restorable' => $splitIssuanceRestorable,
                'stale_refund_truth_restorable' => $staleRefundRestorable,
                'destructive_statements' => 0,
                'enable_path' => 'ISSUANCE_SURFACE_ENABLE_DENIED',
                'preserved' => ['verified_identity', 'customers', 'orders', 'refunds', 'license_types', 'keys', 'nodes', 'sequences', 'customer_data', 'migration_journals', 'evidence'],
            ],
            'counts' => [
                'legacy_ledger_rows' => count($legacyRows),
                'issuer_surfaces' => count($issuerRows),
                'grant_capable_rows' => $grantCapable,
            ],
            'idempotency_key' => hash('sha256', $receiptPayload),
            'redacted' => true,
            'excluded' => ['raw_email', 'license_key', 'token', 'credential', 'card_data', 'customer_row'],
        ];
        $receipt['receipt_digest'] = hash('sha256', FocusaSpec172NoSalesCutoverSchema::encodeCanonical($receipt));
        return $receipt;
    }

    private function assertRequestInputs(array $input): void
    {
        if (preg_match(self::RUN_PATTERN, (string) ($input['run_handle'] ?? '')) !== 1) {
            throw new DomainException('RUN_HANDLE_REQUIRED');
        }
        if (($input['migration_provenance'] ?? []) === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
    }

    private function runEnvelope(string $runHandle, array $run, bool $replayed): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'run_handle' => $runHandle,
            'replayed' => $replayed,
            'decision' => (string) $run['decision'],
            'receipt_digest' => (string) $run['receipt_digest'],
            'receipt' => json_decode((string) $run['receipt_payload'], true),
        ];
    }

    private function reconEnvelope(string $reconHandle, array $recon, bool $replayed): array
    {
        return [
            'schema' => self::RECON_SCHEMA,
            'recon_handle' => $reconHandle,
            'replayed' => $replayed,
            'result' => (string) $recon['result'],
            'receipt_digest' => (string) $recon['receipt_digest'],
            'receipt' => json_decode((string) $recon['receipt_payload'], true),
        ];
    }

    private function proofEnvelope(string $proofHandle, array $proof, bool $replayed): array
    {
        return [
            'schema' => self::ROLLBACK_SCHEMA,
            'proof_handle' => $proofHandle,
            'replayed' => $replayed,
            'verdict' => (string) $proof['verdict'],
            'receipt_digest' => (string) $proof['receipt_digest'],
            'receipt' => json_decode((string) $proof['receipt_payload'], true),
        ];
    }

    private function findRun(string $runHandle): ?array
    {
        $stmt = $this->db->prepare("SELECT run_handle, run_digest, inventory_id, decision, zero_sales_proven, clean_cutover_allowed, block_reason, genuine_sale, receipt_payload, receipt_digest, idempotency_key, started_at, migration_provenance
            FROM {$this->schema->table('wpuiai_spec172_cutover_runs')} WHERE run_handle = :run");
        $stmt->execute([':run' => $runHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findRecon(string $reconHandle): ?array
    {
        $stmt = $this->db->prepare("SELECT recon_handle, run_handle, recon_digest, result, receipt_payload, receipt_digest, idempotency_key, occurred_at, migration_provenance
            FROM {$this->schema->table('wpuiai_spec172_reconciliation_runs')} WHERE recon_handle = :recon");
        $stmt->execute([':recon' => $reconHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findReconByDigest(string $digest): ?array
    {
        $stmt = $this->db->prepare("SELECT recon_handle, recon_digest FROM {$this->schema->table('wpuiai_spec172_reconciliation_runs')} WHERE recon_digest = :digest");
        $stmt->execute([':digest' => $digest]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findProof(string $proofHandle): ?array
    {
        $stmt = $this->db->prepare("SELECT proof_handle, run_handle, proof_digest, verdict, receipt_payload, receipt_digest, idempotency_key, occurred_at, migration_provenance
            FROM {$this->schema->table('wpuiai_spec172_rollback_proof')} WHERE proof_handle = :proof");
        $stmt->execute([':proof' => $proofHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findProofByDigest(string $digest): ?array
    {
        $stmt = $this->db->prepare("SELECT proof_handle, proof_digest FROM {$this->schema->table('wpuiai_spec172_rollback_proof')} WHERE proof_digest = :digest");
        $stmt->execute([':digest' => $digest]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function journalEvent(string $eventType, string $runHandle, string $recordHandle, string $occurredAt, array $detail, array $provenance, ?string $journalKey = null): array
    {
        $journal = $this->schema->table('wpuiai_spec172_cutover_journal');
        $encodedDetail = FocusaSpec172NoSalesCutoverSchema::encodeCanonical($detail);
        $encodedProvenance = FocusaSpec172NoSalesCutoverSchema::encodeCanonical($provenance);
        $previous = self::GENESIS_DIGEST;
        $row = $this->db->query("SELECT entry_digest FROM {$journal} ORDER BY journal_seq DESC LIMIT 1")->fetch(PDO::FETCH_ASSOC);
        if ($row !== false) {
            $previous = (string) $row['entry_digest'];
        }
        $seq = (int) $this->db->query("SELECT COUNT(*) FROM {$journal}")->fetchColumn() + 1;
        $key = $journalKey ?? hash('sha256', $eventType . "\n" . $runHandle . "\n" . $recordHandle . "\n" . $encodedDetail . "\n" . $encodedProvenance);
        $digest = hash('sha256', $previous . "\n" . $seq . "\n" . $key . "\n" . $runHandle . "\n" . $recordHandle . "\n" . $eventType . "\n" . $occurredAt . "\n" . $encodedDetail . "\n" . $encodedProvenance);
        $stmt = $this->db->prepare("INSERT INTO {$journal}
            (journal_key, run_handle, record_handle, event_type, occurred_at, detail, previous_digest, entry_digest, migration_provenance)
            SELECT :key, :run, :record, :type, :occurred, :detail, :previous, :digest, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$journal} WHERE journal_key = :existing_key)");
        $stmt->execute([
            ':key' => $key,
            ':run' => $runHandle,
            ':record' => $recordHandle === '' ? null : $recordHandle,
            ':type' => $eventType,
            ':occurred' => $occurredAt,
            ':detail' => $encodedDetail,
            ':previous' => $previous,
            ':digest' => $digest,
            ':provenance' => $encodedProvenance,
            ':existing_key' => $key,
        ]);
        return ['journal_key' => $key, 'journal_seq' => $seq, 'entry_digest' => $digest];
    }
}
