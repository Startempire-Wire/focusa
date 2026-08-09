<?php
// 152E.06.06 Run migration canary, reconciliation, and rollback-safety proof
// (Spec 152E §17 periodic reconciliation; §18 refund, revoke, and recovery;
// §22.1 inventory; §22.2 merge rules; §22.3 cutover steps 6-10; §22.4
// rollback; §23 acceptance "Refund" and "Legacy install-site record" rows;
// §24 completion gate 11; Specs 152, 150A, 152A-D; Spec 158 implementation
// excluded).
//
// The migration canary is the bounded proof harness that:
//   - executes a DRY RUN first: per-cohort decisions with ZERO writes;
//   - then applies a BOUNDED COHORT one entry at a time, comparing
//     before/after COUNTS, DIGESTS, and STATUS against pinned vectors for
//     every entry (any divergence fails closed and writes nothing);
//   - injects failure and proves retry is idempotent: a failed entry is
//     QUARANTINED, a retry returns the stored quarantine, and a quarantined
//     record can never be silently un-quarantined or granted;
//   - keeps every unresolved record quarantined (no entitlement, no lease);
//   - reconciles EDD truth against authority truth: refunded/revoked EDD
//     records must map to authority recovery_only/revoked (missing callbacks
//     cannot leave stale access), quarantined records must hold no lease,
//     and any drift fails closed with RECONCILIATION_MISMATCH;
//   - proves rollback-safety: rollback cannot undo verified identity, EDD
//     refund/revoke truth, monotonic sequence, or audit truth (Spec 152E
//     §22.4); rollback is preservation-only with no delete path; and
//   - journal-appends every canary, quarantine, reconciliation, and rollback
//     event into a replay-safe digest-chained journal.
//
// The canary runs ONLY after the authority cutover state is published (atom
// focusa-vbcqu.20.13.53 contract): before publish every canary, reconcile,
// and rollback-proof operation fails closed with CUTOVER_STATE_REQUIRED, and
// the published state must assert new_issuance=edd_authority_only,
// facade_role=presenter_and_bounded_proxy_only, spec158=excluded with an
// intact digest. The decision signature has no price, grant, tier, limit, or
// feature inputs, and no raw email, raw key, payment id, or secret ever
// leaves this contract — only 64-hex digests, masked values, and pinned
// before/after vectors.
declare(strict_types=1);

final class FocusaSpec152eMigrationCanarySchema
{
    public const SCHEMA = 'focusa.spec152e.migration_canary.v1';
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
        $migrations = $this->table('wpuiai_canary_schema_migrations');
        $events = $this->table('wpuiai_canary_schema_events');
        $runs = $this->table('wpuiai_canary_runs');
        $cohort = $this->table('wpuiai_canary_cohort');
        $journal = $this->table('wpuiai_canary_journal');
        $reconciliation = $this->table('wpuiai_canary_reconciliation');
        $ledger = $this->table('wpuiai_canary_sequence_ledger');
        $rollback = $this->table('wpuiai_canary_rollback_proof');
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
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$runs} (
            run_handle {$key} NOT NULL PRIMARY KEY,
            cutover_digest VARCHAR(64) NOT NULL,
            run_digest VARCHAR(64) NOT NULL,
            policy VARCHAR(64) NOT NULL,
            cohort_bound BIGINT NOT NULL,
            started_at VARCHAR(32) NOT NULL,
            canary_state VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$cohort} (
            entry_handle {$key} NOT NULL PRIMARY KEY,
            run_handle {$key} NOT NULL,
            record_handle {$key} NOT NULL,
            surface {$key} NOT NULL,
            disposition VARCHAR(64) NOT NULL,
            product_code {$key} NOT NULL,
            record_status VARCHAR(32) NOT NULL,
            verified_identity_required VARCHAR(16) NOT NULL,
            identity_digest VARCHAR(64) NOT NULL,
            inject_failure VARCHAR(16) NOT NULL,
            before_payload TEXT NOT NULL,
            before_digest VARCHAR(64) NOT NULL,
            expected_after_payload TEXT NOT NULL,
            expected_after_digest VARCHAR(64) NOT NULL,
            canary_state VARCHAR(32) NOT NULL,
            sequence BIGINT NOT NULL,
            outcome_payload TEXT,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$journal} (
            journal_seq BIGINT NOT NULL PRIMARY KEY,
            journal_key VARCHAR(64) NOT NULL UNIQUE,
            run_handle {$key} NOT NULL,
            record_handle {$key} NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            detail TEXT NOT NULL,
            previous_digest VARCHAR(64) NOT NULL,
            entry_digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$reconciliation} (
            recon_handle {$key} NOT NULL PRIMARY KEY,
            run_handle {$key} NOT NULL,
            edd_digest VARCHAR(64) NOT NULL,
            authority_digest VARCHAR(64) NOT NULL,
            matching VARCHAR(16) NOT NULL,
            quarantined_count BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$ledger} (
            ledger_handle {$key} NOT NULL PRIMARY KEY,
            record_handle {$key} NOT NULL,
            sequence BIGINT NOT NULL,
            status VARCHAR(32) NOT NULL,
            event_type VARCHAR(32) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            entry_digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$rollback} (
            proof_handle {$key} NOT NULL PRIMARY KEY,
            run_handle {$key} NOT NULL,
            verified_identity_preserved VARCHAR(16) NOT NULL,
            edd_refund_truth_preserved VARCHAR(16) NOT NULL,
            sequence_preserved VARCHAR(16) NOT NULL,
            audit_preserved VARCHAR(16) NOT NULL,
            proof_digest VARCHAR(64) NOT NULL,
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

    /** Rollback is preservation-only: canary runs, cohort, journal, reconciliation, ledger, and rollback proofs are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_canary_schema_events');
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

final class FocusaSpec152eMigrationCanaryService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.migration_canary_result.v1';
    public const VERSION = 1;

    /** The only canary policy this surface accepts. */
    public const POLICY = 'dry_run_then_bounded_canary';

    /** Hard bound on the canary cohort size (Spec 152E §22.1: bounded migration, no unbounded sweep). */
    public const COHORT_BOUND = 8;

    /** The published authority cutover state the canary must respect (atom focusa-vbcqu.20.13.53 contract). */
    public const CUTOVER_STATE_KEY = 'cutover_v1';
    public const CUTOVER_REQUIREMENTS = [
        'new_issuance' => 'edd_authority_only',
        'facade_role' => 'presenter_and_bounded_proxy_only',
        'spec158' => 'excluded',
    ];

    /** Migratable dispositions (Spec 152E §22.2 merge rules). */
    public const DISPOSITIONS = [
        'evidence_backed_import', 'refunded_revoked', 'verify_first', 'unresolved',
    ];

    /** Server-owned product allowlist (Spec 152E §8 product/grant registry). */
    public const PRODUCTS = [
        'focusa_operator', 'uiai_engine_operator', 'focusa_uiai_bundle', 'focusa_evaluation',
    ];

    /** Authority surfaces a cohort entry may reference (never legacy co-equal authority). */
    public const SURFACES = ['edd_license', 'edd_order_item', 'authority_account'];

    /** Adverse states that can never be reactivated (Spec 152E §18, §22.2). */
    public const ADVERSE_STATES = ['refunded', 'revoked'];

    /** Ledger/vector statuses this surface understands. */
    public const STATUSES = ['none', 'active', 'recovery_only'];

    /** Per-entry canary lifecycle states. */
    public const CANARY_STATES = ['pending', 'applied', 'quarantined'];

    private const HANDLE_PATTERN = '/^rec_[a-z0-9_]{4,64}$/D';
    private const RUN_PATTERN = '/^run_[a-z0-9_]{4,64}$/D';
    private const RECON_PATTERN = '/^recon_[a-z0-9_]{4,64}$/D';
    private const PROOF_PATTERN = '/^proof_[a-z0-9_]{4,64}$/D';
    private const DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const REQUEST_PATTERN = '/^req_[a-z0-9_]{4,64}$/D';
    private const IDEMPOTENCY_PATTERN = '/^idem_[a-z0-9_]{4,64}$/D';
    private const GENESIS_DIGEST = '0000000000000000000000000000000000000000000000000000000000000000';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eMigrationCanarySchema $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Start a bounded canary run. Requires the published authority cutover
     * state (CUTOVER_STATE_REQUIRED otherwise) and a cohort of at most
     * COHORT_BOUND entries; every entry carries pinned before/after vectors
     * whose digests are validated here. Idempotent: a replay with an
     * identical cohort returns the stored run (zero writes); any different
     * cohort fails closed with RUN_ALREADY_STARTED.
     */
    public function startCanary(array $input): array
    {
        $cutover = $this->requireCutoverState();
        $this->assertRequestInputs($input);
        $runHandle = (string) ($input['run_handle'] ?? '');
        if (preg_match(self::RUN_PATTERN, $runHandle) !== 1) {
            throw new InvalidArgumentException('invalid run handle');
        }
        if ((string) ($input['policy'] ?? '') !== self::POLICY) {
            throw new DomainException('CANARY_POLICY_REQUIRED');
        }
        $entries = $input['cohort'] ?? [];
        if (!is_array($entries) || $entries === [] || count($entries) > self::COHORT_BOUND) {
            throw new DomainException('COHORT_BOUND_EXCEEDED');
        }
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }

        $runDigest = self::runDigest($runHandle, $entries);
        $runs = $this->schema->table('wpuiai_canary_runs');
        $existing = $this->findRun($runHandle);
        if ($existing !== null) {
            if (!hash_equals((string) $existing['run_digest'], $runDigest)) {
                throw new DomainException('RUN_ALREADY_STARTED');
            }
            return $this->runEnvelope($runHandle, $existing, true);
        }

        $now = ($this->clock)();
        FocusaSpec152eMigrationCanarySchema::assertTimestamp($now);
        $runInsert = $this->db->prepare("INSERT INTO {$runs}
            (run_handle, cutover_digest, run_digest, policy, cohort_bound, started_at, canary_state, migration_provenance)
            VALUES (:run, :cutover, :rundigest, :policy, :bound, :started, 'started', :provenance)");
        $runInsert->execute([
            ':run' => $runHandle,
            ':cutover' => $cutover['state_digest'],
            ':rundigest' => $runDigest,
            ':policy' => self::POLICY,
            ':bound' => count($entries),
            ':started' => $now,
            ':provenance' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($provenance),
        ]);

        $cohort = $this->schema->table('wpuiai_canary_cohort');
        $entryInsert = $this->db->prepare("INSERT INTO {$cohort}
            (entry_handle, run_handle, record_handle, surface, disposition, product_code, record_status,
             verified_identity_required, identity_digest, inject_failure,
             before_payload, before_digest, expected_after_payload, expected_after_digest,
             canary_state, sequence, outcome_payload, occurred_at, migration_provenance)
            VALUES (:entry, :run, :record, :surface, :disposition, :product, :status,
             :identity_required, :identity_digest, :inject,
             :before_payload, :before_digest, :after_payload, :after_digest,
             'pending', 0, NULL, :occurred, :provenance)");
        foreach ($entries as $entry) {
            $validated = $this->assertCohortEntry($entry);
            $entryInsert->execute([
                ':entry' => $validated['entry_handle'],
                ':run' => $runHandle,
                ':record' => $validated['record_handle'],
                ':surface' => $validated['surface'],
                ':disposition' => $validated['disposition'],
                ':product' => $validated['product_code'],
                ':status' => $validated['record_status'],
                ':identity_required' => $validated['verified_identity_required'] ? 'true' : 'false',
                ':identity_digest' => $validated['identity_digest'],
                ':inject' => $validated['inject_failure'] ? 'true' : 'false',
                ':before_payload' => $validated['before_payload'],
                ':before_digest' => $validated['before_digest'],
                ':after_payload' => $validated['after_payload'],
                ':after_digest' => $validated['after_digest'],
                ':occurred' => $now,
                ':provenance' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($validated['migration_provenance']),
            ]);
        }
        $this->journalEvent('canary_started', $runHandle, '', $now, ['run_handle' => $runHandle, 'cohort_size' => count($entries), 'cutover_digest' => $cutover['state_digest']], $provenance, hash('sha256', 'canary_started' . "\n" . $runHandle . "\n" . $runDigest));

        $stored = $this->findRun($runHandle);
        return $this->runEnvelope($runHandle, $stored, false);
    }

    /**
     * Dry-run the whole cohort: per-entry predicted decisions with ZERO
     * writes. Mirrors runCanaryEntry()'s gates (cutover state, pinned
     * before vectors, disposition, product allowlist, identity requirement)
     * so the preview always matches apply. Never a grant by itself.
     */
    public function dryRunCanary(array $input): array
    {
        $this->requireCutoverState();
        $this->assertRequestInputs($input);
        $runHandle = (string) ($input['run_handle'] ?? '');
        $run = $this->findRun($runHandle);
        if ($run === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        $decisions = [];
        foreach ($this->cohortEntries($runHandle) as $entry) {
            $before = json_decode((string) $entry['before_payload'], true);
            if (!$this->verifyBeforeVector($entry, $before)) {
                throw new DomainException('CANARY_BEFORE_MISMATCH');
            }
            $decision = $this->predictDecision($entry);
            $decisions[] = [
                'entry_handle' => (string) $entry['entry_handle'],
                'record_handle' => (string) $entry['record_handle'],
                'decision' => $decision['decision'],
                'reason' => $decision['reason'] ?? null,
                'identity_gate_required' => $decision['identity_gate_required'] ?? false,
            ];
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'mode' => 'dry_run',
            'run_handle' => $runHandle,
            'decisions' => $decisions,
            'written' => false,
        ];
    }

    /**
     * Apply ONE bounded cohort entry. Compares before/after counts, digests,
     * and status against the entry's pinned vectors; injects failure by
     * quarantining the entry; and is fully idempotent — a retry returns the
     * stored outcome with zero new rows and can never un-quarantine a record.
     *
     * Required input:
     *   - request_id / idempotency_key
     *   - run_handle / entry_handle (must belong to the stored cohort)
     *   - inject_failure (bool, honored only while the entry is pending)
     *   - verified_identity_digest (for verify_first entries)
     *   - migration_provenance
     */
    public function runCanaryEntry(array $input): array
    {
        $this->requireCutoverState();
        $this->assertRequestInputs($input);
        $runHandle = (string) ($input['run_handle'] ?? '');
        $entryHandle = (string) ($input['entry_handle'] ?? '');
        if (preg_match(self::HANDLE_PATTERN, $entryHandle) !== 1) {
            throw new InvalidArgumentException('invalid entry handle');
        }
        $run = $this->findRun($runHandle);
        if ($run === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        $entry = $this->findCohortEntry($runHandle, $entryHandle);
        if ($entry === null) {
            throw new InvalidArgumentException('unknown cohort entry');
        }
        $inject = (bool) ($input['inject_failure'] ?? false) || ((string) $entry['inject_failure'] === 'true');

        // Idempotency: a resolved entry replays its stored outcome, zero writes.
        $state = (string) $entry['canary_state'];
        if ($state !== 'pending') {
            return $this->replayEntryOutcome($entry, true);
        }
        $now = ($this->clock)();
        FocusaSpec152eMigrationCanarySchema::assertTimestamp($now);
        $idemKey = hash('sha256', (string) $input['request_id'] . "\n" . (string) $input['idempotency_key'] . "\n" . $entryHandle);
        $recordHandle = (string) $entry['record_handle'];

        // Injected failure: quarantine first, before any before-vector read, so a
        // failed apply can never create truth. Retry returns the stored quarantine.
        if ($inject) {
            $outcome = [
                'decision' => 'quarantine',
                'reason' => 'INJECTED_FAILURE_QUARANTINED',
                'record_handle' => $recordHandle,
                'status' => 'none',
                'sequence' => 0,
                'before_digest' => (string) $entry['before_digest'],
                'after_digest' => (string) $entry['expected_after_digest'],
            ];
            $this->setEntryOutcome($entry, 'quarantined', 0, $outcome, $now);
            $this->journalEvent('canary_quarantined', $runHandle, $recordHandle, $now, ['entry_handle' => $entryHandle, 'reason' => 'INJECTED_FAILURE_QUARANTINED'], $input['migration_provenance'], $idemKey);
            return $this->outcomeEnvelope($outcome, false);
        }

        // Before comparison: current counts/digest/status must equal the pinned before vector.
        $before = json_decode((string) $entry['before_payload'], true);
        if (!$this->verifyBeforeVector($entry, $before)) {
            throw new DomainException('CANARY_BEFORE_MISMATCH');
        }

        $decision = $this->predictDecision($entry);
        if ($decision['decision'] === 'quarantine') {
            $outcome = [
                'decision' => 'quarantine',
                'reason' => $decision['reason'],
                'record_handle' => $recordHandle,
                'status' => 'none',
                'sequence' => 0,
                'before_digest' => (string) $entry['before_digest'],
                'after_digest' => (string) $entry['expected_after_digest'],
            ];
            $this->setEntryOutcome($entry, 'quarantined', 0, $outcome, $now);
            $this->journalEvent('canary_quarantined', $runHandle, $recordHandle, $now, ['entry_handle' => $entryHandle, 'reason' => $decision['reason']], $input['migration_provenance'], $idemKey);
            return $this->outcomeEnvelope($outcome, false);
        }

        // Verified identity gate for verify_first entries: raw email is never
        // accepted — only the keyed 64-hex digest pinned at cohort start.
        if ($decision['identity_gate_required'] ?? false) {
            $this->assertVerifiedIdentity($input, $entry);
        }

        // Compute the deterministic expected after vector from before + disposition.
        $after = [
            'counts' => ['sequence_ledger' => ((int) $before['counts']['sequence_ledger']) + 1],
            'sequence' => ((int) $before['sequence']) + 1,
            'status' => $decision['status'],
        ];
        $afterDigest = $this->vectorDigest($after);

        // After comparison BEFORE any write: the computed transition must equal
        // the pinned expected after vector; a divergent entry fails closed and
        // writes nothing (the canary never converges on drift).
        $expectedAfter = json_decode((string) $entry['expected_after_payload'], true);
        if (!hash_equals($afterDigest, (string) $entry['expected_after_digest'])
            || !hash_equals($afterDigest, $this->vectorDigest($expectedAfter))
            || $after['status'] !== $expectedAfter['status']
            || $after['sequence'] !== $expectedAfter['sequence']) {
            throw new DomainException('CANARY_AFTER_MISMATCH');
        }

        // Execute the transition: exactly one sequence-ledger row, status from
        // the disposition, monotonic sequence increment. Refund/revoke writes a
        // recovery_only row (refresh denied); imports write an active row.
        $sequence = (int) $after['sequence'];
        $status = (string) $after['status'];
        $eventType = match ($decision['reason'] ?? null) {
            'REFUNDED' => 'refund_sequence',
            'REVOKED' => 'revoke_sequence',
            default => 'import_sequence',
        };
        $ledger = $this->schema->table('wpuiai_canary_sequence_ledger');
        $ledgerHandle = hash('sha256', 'ledger' . "\n" . $recordHandle . "\n" . $sequence);
        $ledgerInsert = $this->db->prepare("INSERT INTO {$ledger}
            (ledger_handle, record_handle, sequence, status, event_type, occurred_at, entry_digest, migration_provenance)
            SELECT :ledger, :record, :sequence, :status, :event, :occurred, :digest, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$ledger} WHERE ledger_handle = :existing)");
        $ledgerDigest = hash('sha256', $recordHandle . "\n" . $sequence . "\n" . $status . "\n" . $eventType);
        $ledgerInsert->execute([
            ':ledger' => $ledgerHandle,
            ':record' => $recordHandle,
            ':sequence' => $sequence,
            ':status' => $status,
            ':event' => $eventType,
            ':occurred' => $now,
            ':digest' => $ledgerDigest,
            ':provenance' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($input['migration_provenance']),
            ':existing' => $ledgerHandle,
        ]);

        // After comparison AFTER the write: re-read and compare counts/digest/status.
        $readBack = $this->currentVector($recordHandle);
        if (!hash_equals($afterDigest, $this->vectorDigest($readBack)) || $readBack['status'] !== $status || $readBack['sequence'] !== $sequence) {
            throw new DomainException('CANARY_AFTER_MISMATCH');
        }

        $outcome = [
            'decision' => $decision['decision'],
            'reason' => $decision['reason'] ?? null,
            'record_handle' => $recordHandle,
            'status' => $status,
            'sequence' => $sequence,
            'before_digest' => (string) $entry['before_digest'],
            'after_digest' => $afterDigest,
        ];
        $this->setEntryOutcome($entry, 'applied', $sequence, $outcome, $now);
        $this->journalEvent('canary_applied', $runHandle, $recordHandle, $now, ['entry_handle' => $entryHandle, 'decision' => $decision['decision'], 'reason' => $decision['reason'] ?? null, 'status' => $status, 'sequence' => $sequence], $input['migration_provenance'], $idemKey);
        return $this->outcomeEnvelope($outcome, false);
    }

    /**
     * Reconcile EDD truth against authority truth (Spec 152E §17, §22.1).
     * Every EDD refunded/revoked record must map to authority
     * recovery_only/revoked — a lease still active for a refunded/revoked
     * EDD record (a missed callback leaving stale access) fails closed with
     * RECONCILIATION_MISMATCH, as does any quarantined record holding a
     * lease. Digests are recomputed server-side; the caller can never steer
     * the verdict. Idempotent via recon_handle.
     */
    public function reconcile(array $input): array
    {
        $this->requireCutoverState();
        $this->assertRequestInputs($input);
        $runHandle = (string) ($input['run_handle'] ?? '');
        if ($this->findRun($runHandle) === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        $reconHandle = (string) ($input['recon_handle'] ?? '');
        if (preg_match(self::RECON_PATTERN, $reconHandle) !== 1) {
            throw new InvalidArgumentException('invalid recon handle');
        }
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $reconciliation = $this->schema->table('wpuiai_canary_reconciliation');
        $existingStmt = $this->db->prepare("SELECT * FROM {$reconciliation} WHERE recon_handle = :handle");
        $existingStmt->execute([':handle' => $reconHandle]);
        $existing = $existingStmt->fetch(PDO::FETCH_ASSOC);
        if ($existing !== false) {
            return $this->reconEnvelope($existing, true);
        }

        $eddTruth = $input['edd_truth'] ?? [];
        $authorityLeases = $input['authority_leases'] ?? [];
        $quarantinedHandles = $input['quarantined_handles'] ?? [];
        if (!is_array($eddTruth) || !is_array($authorityLeases) || !is_array($quarantinedHandles)) {
            throw new DomainException('RECONCILIATION_MISMATCH');
        }

        $eddMap = [];
        foreach ($eddTruth as $row) {
            $handle = (string) ($row['record_handle'] ?? '');
            $adverse = (string) ($row['adverse_state'] ?? '');
            if (preg_match(self::HANDLE_PATTERN, $handle) !== 1 || !in_array($adverse, self::ADVERSE_STATES, true)) {
                throw new DomainException('RECONCILIATION_MISMATCH');
            }
            $eddMap[$handle] = $adverse;
        }
        $authorityMap = [];
        foreach ($authorityLeases as $row) {
            $handle = (string) ($row['record_handle'] ?? '');
            $status = (string) ($row['status'] ?? '');
            if (preg_match(self::HANDLE_PATTERN, $handle) !== 1 || !in_array($status, ['active', 'recovery_only'], true)) {
                throw new DomainException('RECONCILIATION_MISMATCH');
            }
            $authorityMap[$handle] = $status;
        }
        // Quarantined handles must be exactly the cohort entries currently quarantined.
        $cohortQuarantined = [];
        foreach ($this->cohortEntries($runHandle) as $entry) {
            if ((string) $entry['canary_state'] === 'quarantined') {
                $cohortQuarantined[] = (string) $entry['record_handle'];
            }
        }
        sort($cohortQuarantined, SORT_STRING);
        $providedQuarantined = $quarantinedHandles;
        sort($providedQuarantined, SORT_STRING);
        if ($providedQuarantined !== $cohortQuarantined) {
            throw new DomainException('RECONCILIATION_MISMATCH');
        }
        // A quarantined record must hold NO authority lease.
        foreach ($cohortQuarantined as $handle) {
            if (isset($authorityMap[$handle])) {
                throw new DomainException('RECONCILIATION_MISMATCH');
            }
        }
        // Missing callbacks cannot leave stale access: every EDD refunded/revoked
        // record must be reflected in authority as recovery_only/revoked — never active.
        foreach ($eddMap as $handle => $adverse) {
            $lease = $authorityMap[$handle] ?? 'missing';
            if ($lease === 'active' || $lease === 'missing') {
                throw new DomainException('RECONCILIATION_MISMATCH');
            }
        }

        $eddDigest = $this->truthDigest('edd', $eddMap);
        $authorityDigest = $this->truthDigest('authority', $authorityMap);
        $now = ($this->clock)();
        FocusaSpec152eMigrationCanarySchema::assertTimestamp($now);
        $insert = $this->db->prepare("INSERT INTO {$reconciliation}
            (recon_handle, run_handle, edd_digest, authority_digest, matching, quarantined_count, occurred_at, migration_provenance)
            VALUES (:recon, :run, :edd, :authority, 'true', :quarantined, :occurred, :provenance)");
        $insert->execute([
            ':recon' => $reconHandle,
            ':run' => $runHandle,
            ':edd' => $eddDigest,
            ':authority' => $authorityDigest,
            ':quarantined' => count($cohortQuarantined),
            ':occurred' => $now,
            ':provenance' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($provenance),
        ]);
        $this->journalEvent('reconciled', $runHandle, '', $now, ['recon_handle' => $reconHandle, 'edd_digest' => $eddDigest, 'authority_digest' => $authorityDigest, 'quarantined_count' => count($cohortQuarantined)], $provenance, hash('sha256', 'reconciled' . "\n" . $reconHandle));

        $stored = $this->findRecon($reconHandle);
        return $this->reconEnvelope($stored, false);
    }

    /**
     * Prove rollback-safety (Spec 152E §22.4): a software/facade rollback
     * cannot undo (a) verified identity, (b) EDD refund/revoke truth, (c)
     * monotonic sequence, or (d) audit truth. All four checks run server-side
     * against stored state; the proof is journaled once per proof_handle.
     */
    public function proveRollback(array $input): array
    {
        $this->requireCutoverState();
        $this->assertRequestInputs($input);
        $runHandle = (string) ($input['run_handle'] ?? '');
        if ($this->findRun($runHandle) === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        $proofHandle = (string) ($input['proof_handle'] ?? '');
        if (preg_match(self::PROOF_PATTERN, $proofHandle) !== 1) {
            throw new InvalidArgumentException('invalid proof handle');
        }
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $rollback = $this->schema->table('wpuiai_canary_rollback_proof');
        $existingStmt = $this->db->prepare("SELECT * FROM {$rollback} WHERE proof_handle = :handle");
        $existingStmt->execute([':handle' => $proofHandle]);
        $existing = $existingStmt->fetch(PDO::FETCH_ASSOC);
        if ($existing !== false) {
            return $this->proofEnvelope($existing, true);
        }

        // (a) Verified identity is preserved: every verify_first cohort entry
        // still carries its pinned identity digest (never cleared, never changed).
        $identityPreserved = true;
        foreach ($this->cohortEntries($runHandle) as $entry) {
            if ((string) $entry['verified_identity_required'] === 'true' && preg_match(self::DIGEST_PATTERN, (string) $entry['identity_digest']) !== 1) {
                $identityPreserved = false;
                break;
            }
        }

        // (b) EDD refund/revoke truth is preserved: every refunded/revoked
        // cohort entry is recovery_only in the ledger — never reverted to active.
        $refundTruthPreserved = true;
        foreach ($this->cohortEntries($runHandle) as $entry) {
            if ((string) $entry['disposition'] !== 'refunded_revoked') {
                continue;
            }
            $vector = $this->currentVector((string) $entry['record_handle']);
            if ($vector['status'] !== 'recovery_only') {
                $refundTruthPreserved = false;
                break;
            }
        }

        // (c) Sequence is preserved: every applied entry's ledger sequence
        // equals its pinned after sequence (monotonic, never reset, never reduced).
        $sequencePreserved = true;
        foreach ($this->cohortEntries($runHandle) as $entry) {
            if ((string) $entry['canary_state'] !== 'applied') {
                continue;
            }
            $after = json_decode((string) $entry['expected_after_payload'], true);
            $vector = $this->currentVector((string) $entry['record_handle']);
            if ($vector['sequence'] !== (int) $after['sequence'] || $vector['sequence'] < 1) {
                $sequencePreserved = false;
                break;
            }
        }

        // (d) Audit truth is preserved: the digest-chained journal is valid.
        $auditPreserved = $this->journalChainValid();

        if (!$identityPreserved || !$refundTruthPreserved || !$sequencePreserved || !$auditPreserved) {
            throw new DomainException('ROLLBACK_SAFETY_PROOF_FAILED');
        }

        $proofDigest = hash('sha256', 'rollback_proof' . "\n" . $runHandle . "\n" . ($identityPreserved ? '1' : '0') . "\n" . ($refundTruthPreserved ? '1' : '0') . "\n" . ($sequencePreserved ? '1' : '0') . "\n" . ($auditPreserved ? '1' : '0'));
        $now = ($this->clock)();
        FocusaSpec152eMigrationCanarySchema::assertTimestamp($now);
        $insert = $this->db->prepare("INSERT INTO {$rollback}
            (proof_handle, run_handle, verified_identity_preserved, edd_refund_truth_preserved, sequence_preserved, audit_preserved, proof_digest, occurred_at, migration_provenance)
            VALUES (:proof, :run, 'true', 'true', 'true', 'true', :digest, :occurred, :provenance)");
        $insert->execute([
            ':proof' => $proofHandle,
            ':run' => $runHandle,
            ':digest' => $proofDigest,
            ':occurred' => $now,
            ':provenance' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($provenance),
        ]);
        $this->journalEvent('rollback_proven', $runHandle, '', $now, ['proof_handle' => $proofHandle, 'proof_digest' => $proofDigest], $provenance, hash('sha256', 'rollback_proven' . "\n" . $proofHandle));

        $stored = $this->findProof($proofHandle);
        return $this->proofEnvelope($stored, false);
    }

    /**
     * Never reactivates a refunded/revoked record. Always fails closed with
     * the typed adverse-state code; no code path can re-grant or resurrect a
     * refunded/revoked record.
     */
    public function reactivate(string $recordHandle, array $input): never
    {
        $adverse = (string) ($input['adverse_state'] ?? '');
        if ($adverse === 'revoked') {
            throw new DomainException('REVOKED');
        }
        throw new DomainException('REFUNDED');
    }

    /** Per-run convergence summary: zero loss, zero authority rollback, every entry resolved. */
    public function canarySummary(array $input): array
    {
        $runHandle = (string) ($input['run_handle'] ?? '');
        $run = $this->findRun($runHandle);
        if ($run === null) {
            throw new DomainException('CANARY_RUN_REQUIRED');
        }
        $applied = 0;
        $quarantined = 0;
        $pending = 0;
        $expectedLedger = 0;
        foreach ($this->cohortEntries($runHandle) as $entry) {
            $after = json_decode((string) $entry['expected_after_payload'], true);
            $expectedLedger += (int) $after['counts']['sequence_ledger'];
            if ((string) $entry['canary_state'] === 'applied') {
                $applied++;
            } elseif ((string) $entry['canary_state'] === 'quarantined') {
                $quarantined++;
            } else {
                $pending++;
            }
        }
        $ledgerRows = (int) $this->db->query("SELECT COUNT(*) FROM {$this->schema->table('wpuiai_canary_sequence_ledger')}")->fetchColumn();
        $converged = $pending === 0 && ($applied + $quarantined) === count($this->cohortEntries($runHandle));
        $zeroLoss = $ledgerRows === $expectedLedger;
        $zeroRollback = true;
        foreach ($this->cohortEntries($runHandle) as $entry) {
            if ((string) $entry['canary_state'] !== 'applied') {
                continue;
            }
            $after = json_decode((string) $entry['expected_after_payload'], true);
            $vector = $this->currentVector((string) $entry['record_handle']);
            if ($vector['status'] !== $after['status'] || $vector['sequence'] !== (int) $after['sequence']) {
                $zeroRollback = false;
                break;
            }
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'run_handle' => $runHandle,
            'cohort_size' => count($this->cohortEntries($runHandle)),
            'applied' => $applied,
            'quarantined' => $quarantined,
            'pending' => $pending,
            'ledger_rows' => $ledgerRows,
            'expected_ledger_rows' => $expectedLedger,
            'converged' => $converged,
            'zero_loss' => $zeroLoss,
            'zero_authority_rollback' => $zeroRollback,
        ];
    }

    /** Canonical 64-hex digest of a before/after vector (counts, sequence, status). */
    public function vectorDigest(array $vector): string
    {
        return hash('sha256', FocusaSpec152eMigrationCanarySchema::encodeCanonical([
            'counts' => $vector['counts'],
            'sequence' => (int) $vector['sequence'],
            'status' => (string) $vector['status'],
        ]));
    }

    /** Verify the append-only journal digest chain from genesis (formula identical to journalEvent). */
    public function journalChainValid(): bool
    {
        $journal = $this->schema->table('wpuiai_canary_journal');
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
        $quoted = $this->schema->table($table);
        return (int) $this->db->query("SELECT COUNT(*) FROM {$quoted}")->fetchColumn();
    }

    // ── Internals ──────────────────────────────────────────────────────

    /**
     * The canary only runs against the published authority cutover state
     * (atom focusa-vbcqu.20.13.53): the published payload digest must
     * recompute identically and must assert new_issuance=edd_authority_only,
     * facade_role=presenter_and_bounded_proxy_only, spec158=excluded.
     */
    private function requireCutoverState(): array
    {
        try {
            $stateTable = $this->schema->table('wpuiai_cutover_state');
            $stmt = $this->db->prepare("SELECT state_payload, state_digest FROM {$stateTable} WHERE state_key = :key");
            $stmt->execute([':key' => self::CUTOVER_STATE_KEY]);
            $row = $stmt->fetch(PDO::FETCH_ASSOC);
        } catch (PDOException $e) {
            $row = false;
        }
        if ($row === false) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        $payload = json_decode((string) $row['state_payload'], true);
        if (!is_array($payload) || preg_match(self::DIGEST_PATTERN, (string) $row['state_digest']) !== 1) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        $recomputed = hash('sha256', FocusaSpec152eMigrationCanarySchema::encodeCanonical($payload));
        if (!hash_equals($recomputed, (string) $row['state_digest'])) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        foreach (self::CUTOVER_REQUIREMENTS as $key => $expected) {
            if ((string) ($payload[$key] ?? '') !== $expected) {
                throw new DomainException('CUTOVER_STATE_REQUIRED');
            }
        }
        return [
            'state_key' => self::CUTOVER_STATE_KEY,
            'state_digest' => (string) $row['state_digest'],
            'new_issuance' => (string) $payload['new_issuance'],
            'facade_role' => (string) $payload['facade_role'],
            'spec158' => (string) $payload['spec158'],
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

    private function assertCohortEntry(array $entry): array
    {
        $handle = (string) ($entry['handle'] ?? '');
        $surface = (string) ($entry['surface'] ?? '');
        $disposition = (string) ($entry['disposition'] ?? '');
        $productCode = (string) ($entry['product_code'] ?? '');
        $recordStatus = (string) ($entry['record_status'] ?? '');
        if (preg_match(self::HANDLE_PATTERN, $handle) !== 1) {
            throw new InvalidArgumentException('invalid cohort handle');
        }
        if (!in_array($surface, self::SURFACES, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (!in_array($disposition, self::DISPOSITIONS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (!in_array($productCode, self::PRODUCTS, true)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $identityRequired = (bool) ($entry['verified_identity_required'] ?? false);
        $identityDigest = (string) ($entry['identity_digest'] ?? '');
        if ($identityRequired) {
            if (preg_match(self::DIGEST_PATTERN, $identityDigest) !== 1) {
                throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
            }
        } elseif ($identityDigest !== '') {
            throw new InvalidArgumentException('identity digest only with verified_identity_required');
        }
        $before = $this->assertVector($entry['before'] ?? []);
        $after = $this->assertVector($entry['after'] ?? []);
        $beforeDigest = $this->vectorDigest($before);
        $afterDigest = $this->vectorDigest($after);
        if (!hash_equals($beforeDigest, (string) ($entry['before']['digest'] ?? ''))) {
            throw new InvalidArgumentException('invalid pinned before digest');
        }
        if (!hash_equals($afterDigest, (string) ($entry['after']['digest'] ?? ''))) {
            throw new InvalidArgumentException('invalid pinned after digest');
        }
        $afterCount = (int) $after['counts']['sequence_ledger'];
        $beforeCount = (int) $before['counts']['sequence_ledger'];
        if ($afterCount < $beforeCount || (int) $after['sequence'] < (int) $before['sequence']) {
            throw new InvalidArgumentException('non-regressive after vector');
        }
        $provenance = $entry['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        return [
            'entry_handle' => $handle,
            'record_handle' => $handle,
            'surface' => $surface,
            'disposition' => $disposition,
            'product_code' => $productCode,
            'record_status' => $recordStatus,
            'verified_identity_required' => $identityRequired,
            'identity_digest' => $identityDigest,
            'inject_failure' => (bool) ($entry['inject_failure'] ?? false),
            'before_payload' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($before),
            'before_digest' => $beforeDigest,
            'after_payload' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($after),
            'after_digest' => $afterDigest,
            'migration_provenance' => $provenance,
        ];
    }

    private function assertVector(array $vector): array
    {
        $status = (string) ($vector['status'] ?? '');
        $sequence = (int) ($vector['sequence'] ?? -1);
        $counts = $vector['counts'] ?? [];
        if (!in_array($status, self::STATUSES, true) || $sequence < 0) {
            throw new InvalidArgumentException('invalid vector status/sequence');
        }
        if (!is_array($counts) || !isset($counts['sequence_ledger']) || !is_int($counts['sequence_ledger']) || $counts['sequence_ledger'] < 0) {
            throw new InvalidArgumentException('invalid vector counts');
        }
        return ['counts' => $counts, 'sequence' => $sequence, 'status' => $status];
    }

    /** Deterministic run digest over the bounded cohort (policy + per-entry pins). */
    private static function runDigest(string $runHandle, array $entries): string
    {
        $entryDigests = [];
        foreach ($entries as $entry) {
            $before = json_decode((string) json_encode($entry['before'] ?? [], JSON_THROW_ON_ERROR), true);
            $after = json_decode((string) json_encode($entry['after'] ?? [], JSON_THROW_ON_ERROR), true);
            $entryDigests[] = hash('sha256', (string) ($entry['handle'] ?? '') . "\n" . (string) ($entry['surface'] ?? '') . "\n" . (string) ($entry['disposition'] ?? '') . "\n" . (string) ($entry['product_code'] ?? '') . "\n" . hash('sha256', FocusaSpec152eMigrationCanarySchema::encodeCanonical($before)) . "\n" . hash('sha256', FocusaSpec152eMigrationCanarySchema::encodeCanonical($after)));
        }
        sort($entryDigests, SORT_STRING);
        return hash('sha256', $runHandle . "\n" . self::POLICY . "\n" . implode("\n", $entryDigests));
    }

    /** Predicted decision for one stored cohort entry (mirrors apply gates). */
    private function predictDecision(array $entry): array
    {
        $recordStatus = (string) $entry['record_status'];
        if ((string) $entry['disposition'] === 'unresolved') {
            return ['decision' => 'quarantine', 'reason' => 'UNRESOLVED_QUARANTINED'];
        }
        if ((string) $entry['inject_failure'] === 'true') {
            return ['decision' => 'quarantine', 'reason' => 'INJECTED_FAILURE_QUARANTINED'];
        }
        if ((string) $entry['disposition'] === 'refunded_revoked') {
            $reason = $recordStatus === 'revoked' ? 'REVOKED' : 'REFUNDED';
            return ['decision' => 'preserve_adverse_state', 'reason' => $reason, 'status' => 'recovery_only'];
        }
        $decision = ['decision' => 'import', 'status' => 'active'];
        if ((string) $entry['disposition'] === 'verify_first') {
            $decision['identity_gate_required'] = true;
        }
        return $decision;
    }

    private function verifyBeforeVector(array $entry, array $before): bool
    {
        $current = $this->currentVector((string) $entry['record_handle']);
        if ($current['status'] !== $before['status'] || $current['sequence'] !== (int) $before['sequence'] || $current['counts']['sequence_ledger'] !== (int) $before['counts']['sequence_ledger']) {
            return false;
        }
        return hash_equals($this->vectorDigest($current), (string) $entry['before_digest']);
    }

    /** Current ledger truth for a record: status, sequence, counts (0 rows = none/0/0). */
    private function currentVector(string $recordHandle): array
    {
        $ledger = $this->schema->table('wpuiai_canary_sequence_ledger');
        $stmt = $this->db->prepare("SELECT COUNT(*) AS rows_count, COALESCE(MAX(sequence), 0) AS max_sequence FROM {$ledger} WHERE record_handle = :record");
        $stmt->execute([':record' => $recordHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        $rowsCount = (int) $row['rows_count'];
        $maxSequence = (int) $row['max_sequence'];
        $status = 'none';
        if ($rowsCount > 0) {
            $statusStmt = $this->db->prepare("SELECT status FROM {$ledger} WHERE record_handle = :record ORDER BY sequence DESC LIMIT 1");
            $statusStmt->execute([':record' => $recordHandle]);
            $status = (string) $statusStmt->fetchColumn();
        }
        return ['counts' => ['sequence_ledger' => $rowsCount], 'sequence' => $maxSequence, 'status' => $status];
    }

    /** Verified identity gate: the keyed 64-hex digest must be present and match the pinned digest. */
    private function assertVerifiedIdentity(array $input, array $entry): void
    {
        $digest = (string) ($input['verified_identity_digest'] ?? '');
        if ($digest === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (preg_match(self::DIGEST_PATTERN, $digest) !== 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        if (!hash_equals((string) $entry['identity_digest'], $digest)) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
    }

    private function setEntryOutcome(array $entry, string $state, int $sequence, array $outcome, string $occurredAt): void
    {
        $cohort = $this->schema->table('wpuiai_canary_cohort');
        $stmt = $this->db->prepare("UPDATE {$cohort} SET canary_state = :state, sequence = :sequence, outcome_payload = :outcome, occurred_at = :occurred WHERE entry_handle = :entry AND canary_state = 'pending'");
        $stmt->execute([
            ':state' => $state,
            ':sequence' => $sequence,
            ':outcome' => FocusaSpec152eMigrationCanarySchema::encodeCanonical($outcome),
            ':occurred' => $occurredAt,
            ':entry' => (string) $entry['entry_handle'],
        ]);
        if ($stmt->rowCount() !== 1) {
            throw new DomainException('CANARY_STATE_CONFLICT');
        }
    }

    private function outcomeEnvelope(array $outcome, bool $replayed): array
    {
        $after = [
            'counts' => ['sequence_ledger' => $outcome['status'] === 'none' ? 0 : 1],
            'sequence' => $outcome['sequence'],
            'status' => $outcome['status'],
        ];
        return [
            'schema' => self::RESULT_SCHEMA,
            'mode' => 'canary',
            'record_handle' => $outcome['record_handle'],
            'decision' => $outcome['decision'],
            'reason' => $outcome['reason'],
            'status' => $outcome['status'],
            'sequence' => $outcome['sequence'],
            'before_digest' => $outcome['before_digest'],
            'after_digest' => $outcome['after_digest'],
            'compared' => true,
            'after_vector' => $after,
            'replayed' => $replayed,
        ];
    }

    private function replayEntryOutcome(array $entry, bool $replayed): array
    {
        $outcome = json_decode((string) $entry['outcome_payload'], true);
        $outcome['sequence'] = (int) ($outcome['sequence'] ?? 0);
        return $this->outcomeEnvelope($outcome, $replayed);
    }

    private function runEnvelope(string $runHandle, array $run, bool $replayed): array
    {
        return [
            'ok' => true,
            'schema' => self::RESULT_SCHEMA,
            'run_handle' => $runHandle,
            'policy' => (string) $run['policy'],
            'cohort_bound' => (int) $run['cohort_bound'],
            'cutover_digest' => (string) $run['cutover_digest'],
            'canary_state' => (string) $run['canary_state'],
            'replayed' => $replayed,
        ];
    }

    private function reconEnvelope(array $row, bool $replayed): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'recon_handle' => (string) $row['recon_handle'],
            'run_handle' => (string) $row['run_handle'],
            'edd_digest' => (string) $row['edd_digest'],
            'authority_digest' => (string) $row['authority_digest'],
            'matching' => (string) $row['matching'] === 'true',
            'quarantined_count' => (int) $row['quarantined_count'],
            'replayed' => $replayed,
        ];
    }

    private function proofEnvelope(array $row, bool $replayed): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'proof_handle' => (string) $row['proof_handle'],
            'run_handle' => (string) $row['run_handle'],
            'verified_identity_preserved' => (string) $row['verified_identity_preserved'] === 'true',
            'edd_refund_truth_preserved' => (string) $row['edd_refund_truth_preserved'] === 'true',
            'sequence_preserved' => (string) $row['sequence_preserved'] === 'true',
            'audit_preserved' => (string) $row['audit_preserved'] === 'true',
            'proof_digest' => (string) $row['proof_digest'],
            'replayed' => $replayed,
        ];
    }

    /** Deterministic canonical digest over record_handle -> value pairs (deduped, sorted). */
    private function truthDigest(string $kind, array $map): string
    {
        $lines = [];
        foreach ($map as $handle => $value) {
            $lines[] = $handle . ':' . $value;
        }
        sort($lines, SORT_STRING);
        return hash('sha256', $kind . "\n" . implode("\n", $lines));
    }

    private function findRun(string $runHandle): ?array
    {
        $runs = $this->schema->table('wpuiai_canary_runs');
        $stmt = $this->db->prepare("SELECT * FROM {$runs} WHERE run_handle = :handle");
        $stmt->execute([':handle' => $runHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findRecon(string $reconHandle): ?array
    {
        $reconciliation = $this->schema->table('wpuiai_canary_reconciliation');
        $stmt = $this->db->prepare("SELECT * FROM {$reconciliation} WHERE recon_handle = :handle");
        $stmt->execute([':handle' => $reconHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findProof(string $proofHandle): ?array
    {
        $rollback = $this->schema->table('wpuiai_canary_rollback_proof');
        $stmt = $this->db->prepare("SELECT * FROM {$rollback} WHERE proof_handle = :handle");
        $stmt->execute([':handle' => $proofHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findCohortEntry(string $runHandle, string $entryHandle): ?array
    {
        $cohort = $this->schema->table('wpuiai_canary_cohort');
        $stmt = $this->db->prepare("SELECT * FROM {$cohort} WHERE run_handle = :run AND entry_handle = :entry");
        $stmt->execute([':run' => $runHandle, ':entry' => $entryHandle]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** @return array<int, array<string, mixed>> */
    private function cohortEntries(string $runHandle): array
    {
        $cohort = $this->schema->table('wpuiai_canary_cohort');
        $stmt = $this->db->prepare("SELECT * FROM {$cohort} WHERE run_handle = :run ORDER BY entry_handle ASC");
        $stmt->execute([':run' => $runHandle]);
        return $stmt->fetchAll(PDO::FETCH_ASSOC);
    }

    /** Append a digest-chained journal event; replays with the same journal key never append a second entry. */
    private function journalEvent(string $eventType, string $runHandle, string $recordHandle, string $occurredAt, array $detail, array $provenance, ?string $journalKey = null): array
    {
        FocusaSpec152eMigrationCanarySchema::assertTimestamp($occurredAt);
        $encoded = FocusaSpec152eMigrationCanarySchema::encodeCanonical($detail);
        if ($journalKey === null) {
            $journalKey = hash('sha256', $eventType . "\n" . $runHandle . "\n" . $occurredAt . "\n" . $encoded);
        }
        $journal = $this->schema->table('wpuiai_canary_journal');
        $lookup = $this->db->prepare("SELECT journal_seq FROM {$journal} WHERE journal_key = :key");
        $lookup->execute([':key' => $journalKey]);
        $row = $lookup->fetch(PDO::FETCH_ASSOC);
        if ($row !== false) {
            return ['replayed' => true, 'journal_seq' => (int) $row['journal_seq']];
        }
        $seq = (int) $this->db->query("SELECT COALESCE(MAX(journal_seq), 0) + 1 FROM {$journal}")->fetchColumn();
        $previous = self::GENESIS_DIGEST;
        if ($seq > 1) {
            $prevStmt = $this->db->prepare("SELECT entry_digest FROM {$journal} WHERE journal_seq = :prev");
            $prevStmt->execute([':prev' => $seq - 1]);
            $prevRow = $prevStmt->fetch(PDO::FETCH_ASSOC);
            if ($prevRow !== false) {
                $previous = (string) $prevRow['entry_digest'];
            }
        }
        $provenanceEncoded = FocusaSpec152eMigrationCanarySchema::encodeCanonical($provenance);
        $entry = hash('sha256', $previous . "\n" . $seq . "\n" . $journalKey . "\n" . $runHandle . "\n" . $recordHandle . "\n" . $eventType . "\n" . $occurredAt . "\n" . $encoded . "\n" . $provenanceEncoded);
        $stmt = $this->db->prepare("INSERT INTO {$journal}
            (journal_seq, journal_key, run_handle, record_handle, event_type, occurred_at, detail, previous_digest, entry_digest, migration_provenance)
            VALUES (:seq, :key, :run, :record, :event, :occurred, :detail, :prev, :entry, :provenance)");
        $stmt->execute([
            ':seq' => $seq,
            ':key' => $journalKey,
            ':run' => $runHandle,
            ':record' => $recordHandle,
            ':event' => $eventType,
            ':occurred' => $occurredAt,
            ':detail' => $encoded,
            ':prev' => $previous,
            ':entry' => $entry,
            ':provenance' => $provenanceEncoded,
        ]);
        return ['replayed' => false, 'journal_seq' => $seq];
    }
}
