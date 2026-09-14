<?php
// Legacy-customer migration workflow (Spec 152E §6.3 atomic promotion, §22.2 merge
// rules, §22.3 cutover step 6, §22.4 rollback). Invites or challenges legacy
// EDD/install records, attaches records only after mailbox verification plus
// evidence-backed resolution, preserves purchase/license history, and quarantines
// conflicts without entitlement loss. The migration journal is preservation-only
// on rollback; EDD customer/order/refund truth is never mutated by this surface.
// No unverified-email promotion, no local/self-issued entitlement, no independent
// facade authority, no client-controlled EDD price/grant, and no raw email or
// secret ever leaves this contract.
declare(strict_types=1);

final class FocusaSpec152eLegacyCustomerMigrationSchema
{
    public const SCHEMA = 'focusa.spec152e.legacy_customer_migration.v1';
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
        $migrations = $this->table('wpuiai_legacy_customer_migration_schema_migrations');
        $events = $this->table('wpuiai_legacy_customer_migration_schema_events');
        $challenges = $this->table('wpuiai_legacy_customer_challenges');
        $attachments = $this->table('wpuiai_legacy_customer_attachments');
        $journal = $this->table('wpuiai_legacy_customer_journal');
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
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$challenges} (
            challenge_uuid {$uuid} NOT NULL PRIMARY KEY,
            record_handle VARCHAR(191) NOT NULL,
            surface VARCHAR(64) NOT NULL,
            disposition VARCHAR(32) NOT NULL,
            mode VARCHAR(16) NOT NULL,
            state VARCHAR(16) NOT NULL DEFAULT 'open',
            email_lookup_digest VARCHAR(64) NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            quarantine_reason VARCHAR(64) NULL,
            request_id VARCHAR(191) NOT NULL,
            idempotency_key VARCHAR(191) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (record_handle)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$attachments} (
            challenge_uuid {$uuid} NOT NULL PRIMARY KEY,
            attachment_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            linked_orders TEXT NOT NULL,
            linked_licenses TEXT NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            idempotency_key VARCHAR(191) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            attached_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$journal} (
            journal_key VARCHAR(64) NOT NULL PRIMARY KEY,
            challenge_uuid {$uuid} NOT NULL,
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

    /** Rollback is preservation-only: migration challenges, attachments, and journal are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_legacy_customer_migration_schema_events');
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

final class FocusaSpec152eLegacyCustomerMigrationService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.legacy_customer_migration_result.v1';
    public const VERSION = 1;

    /** Inventory surfaces the workflow may migrate (Spec 152E §22.1). */
    public const SURFACES = [
        'edd_customer', 'edd_customer_email', 'edd_order', 'edd_order_item', 'edd_license',
        'install_site_license', 'install_site_audit_receipt', 'stripe_test_object',
        'stripe_live_expired_session', 'stripe_live_incomplete_payment_intent', 'node_record',
    ];

    /** Inventory dispositions the workflow classifies (Spec 152E §22.1/22.2). */
    public const DISPOSITIONS = [
        'canonical', 'evidence_backed_import', 'verify_first', 'duplicate',
        'synthetic_quarantine', 'refunded_revoked', 'unresolved',
    ];

    /** Invite-mode records: the verified owner is invited to claim the record. */
    public const INVITE_DISPOSITIONS = ['canonical', 'evidence_backed_import', 'verify_first'];

    /** Challenge-mode records: the claim must first overcome the recorded challenge. */
    public const CHALLENGE_DISPOSITIONS = ['duplicate', 'synthetic_quarantine', 'refunded_revoked', 'unresolved'];

    /** Only these dispositions may ever attach; everything else is quarantine-only. */
    public const ATTACHABLE_DISPOSITIONS = ['canonical', 'evidence_backed_import', 'verify_first', 'duplicate'];

    /** Disposition-level fail-closed quarantine reasons (public-safe codes, no new codes). */
    public const QUARANTINE_REASONS = [
        'synthetic_quarantine' => 'EDD_ORDER_UNVERIFIED',
        'refunded_revoked' => 'EDD_LICENSE_UNUSABLE',
        'unresolved' => 'EDD_ORDER_UNVERIFIED',
    ];

    /** Failure codes that quarantine the record without entitlement loss; everything else propagates. */
    public const FAILURE_CODES_THAT_QUARANTINE = [
        'EDD_ORDER_UNVERIFIED', 'EDD_LICENSE_UNVERIFIED', 'EDD_LICENSE_UNUSABLE',
        'LICENSE_ACCOUNT_MISMATCH', 'ACCOUNT_MERGE_REVIEW_REQUIRED',
    ];

    public const STATES = ['open', 'attached', 'quarantined'];
    private const EMAIL_DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const HANDLE_PATTERN = '/^rec_[a-z0-9_]{6,64}$/D';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eLegacyCustomerMigrationSchema $schema,
        private FocusaSpec152eAccountPromotionService $promotion,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Open an invite or challenge for one legacy EDD/install record. The record is
     * referenced by its opaque inventory handle and disposition; only the keyed email
     * lookup digest is stored (never the raw email). Evidence is pinned to a bounded
     * digest. No record is attached here. One challenge per record handle.
     *
     * Required input:
     *   - record_handle:        opaque inventory handle (rec_...)
     *   - surface:              one of SURFACES
     *   - disposition:          one of DISPOSITIONS
     *   - email_lookup_digest:  keyed digest of the legacy record email
     *   - legacy_evidence:      evidence-backed provenance (kind/source/record)
     *   - request_id / idempotency_key / migration_provenance
     */
    public function openChallenge(array $input): array
    {
        $handle = $this->assertHandle((string) ($input['record_handle'] ?? ''));
        $surface = (string) ($input['surface'] ?? '');
        if (!in_array($surface, self::SURFACES, true)) {
            throw new InvalidArgumentException('known legacy record surface required');
        }
        $disposition = (string) ($input['disposition'] ?? '');
        if (!in_array($disposition, self::DISPOSITIONS, true)) {
            throw new InvalidArgumentException('known legacy record disposition required');
        }
        $emailDigest = (string) ($input['email_lookup_digest'] ?? '');
        if (preg_match(self::EMAIL_DIGEST_PATTERN, $emailDigest) !== 1) {
            throw new InvalidArgumentException('bounded keyed email lookup digest required');
        }
        $evidence = $input['legacy_evidence'] ?? [];
        $evidenceDigest = FocusaSpec152eLegacyActivationAdapter::validateLegacyEvidence($evidence);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $mode = $this->modeFor($disposition);
        $now = $this->now();
        $digest = $this->digest([
            'operation' => 'open_challenge',
            'record_handle' => $handle,
            'surface' => $surface,
            'disposition' => $disposition,
            'email_lookup_digest' => $emailDigest,
            'legacy_evidence' => $evidence,
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($handle, $surface, $disposition, $mode, $emailDigest, $evidenceDigest, $requestId, $idempotencyKey, $provenance, $now, $digest): array {
            $table = $this->schema->table('wpuiai_legacy_customer_challenges');
            $statement = $this->db->prepare("SELECT * FROM {$table} WHERE record_handle = :handle");
            $statement->execute([':handle' => $handle]);
            $existing = $statement->fetch(PDO::FETCH_ASSOC);
            if ($existing !== false) {
                return $this->replayOrExistingChallenge($existing, $digest, $idempotencyKey, $now, 'challenge_opened');
            }

            $challengeUuid = self::uuid();
            $statement = $this->db->prepare("INSERT INTO {$table}
                (challenge_uuid, record_handle, surface, disposition, mode, state, email_lookup_digest,
                 evidence_digest, quarantine_reason, request_id, idempotency_key, request_digest,
                 created_at, updated_at, migration_provenance)
                VALUES (:challenge, :handle, :surface, :disposition, :mode, 'open', :digest,
                 :evidence, NULL, :request, :idem, :request_digest, :created, :updated, :provenance)");
            $statement->execute([
                ':challenge' => $challengeUuid,
                ':handle' => $handle,
                ':surface' => $surface,
                ':disposition' => $disposition,
                ':mode' => $mode,
                ':digest' => $emailDigest,
                ':evidence' => $evidenceDigest,
                ':request' => $requestId,
                ':idem' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':updated' => $now,
                ':provenance' => $this->encodeCanonical($provenance),
            ]);
            $this->journal($challengeUuid, $mode === 'invite' ? 'invite_opened' : 'challenge_opened', $now, [
                'record_handle' => $handle,
                'surface' => $surface,
                'disposition' => $disposition,
                'mode' => $mode,
                'state' => 'open',
            ], $provenance);
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'challenge_opened',
                'challenge_uuid' => $challengeUuid,
                'record_handle' => $handle,
                'surface' => $surface,
                'disposition' => $disposition,
                'mode' => $mode,
                'state' => 'open',
                'replayed' => false,
                'existing' => false,
            ];
        });
    }

    /**
     * Attach a legacy record after mailbox verification plus evidence-backed resolution.
     * One authority transaction delegates the EDD customer merge to the promotion
     * service (verified identity -> authority account -> EDD customer -> purchase
     * links), then journals the attachment and advances the challenge. Failures in the
     * named verification/conflict codes quarantine the record without entitlement loss;
     * the record stays recoverable through reopenQuarantined. Verified legacy customers
     * merge once: replays and repeated canonical requests return the stored attachment.
     *
     * Required input:
     *   - challenge_uuid / registration_uuid
     *   - verified_email, verification_method, transactional_consent_at
     *   - legacy_key, legacy_evidence, prior_purchases
     *   - request_id / idempotency_key / migration_provenance
     * Optional input:
     *   - promotional_consent_at, wordpress_user_id, stripe_customer_id
     */
    public function attachVerified(array $input): array
    {
        $challengeUuid = $this->assertUuid((string) ($input['challenge_uuid'] ?? ''), 'challenge');
        $verifiedEmail = (string) ($input['verified_email'] ?? '');
        if ($verifiedEmail === '') {
            throw new InvalidArgumentException('verified email is required');
        }
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $challenge = $this->findChallenge($challengeUuid);

        if ((string) $challenge['state'] === 'attached') {
            $attachment = $this->findAttachment($challengeUuid);
            if (!hash_equals((string) $attachment['idempotency_key'], $idempotencyKey)) {
                return $this->attachmentEnvelope($attachment, $challenge, replayed: false, existing: true);
            }
            $digest = $this->digest([
                'operation' => 'attach_verified',
                'challenge_uuid' => $challengeUuid,
                'record_handle' => $challenge['record_handle'],
                'verified_email' => $verifiedEmail,
                'legacy_key' => (string) ($input['legacy_key'] ?? ''),
                'migration_provenance' => $provenance,
                'request_id' => $requestId,
            ]);
            if (!hash_equals((string) $attachment['request_digest'], $digest)) {
                throw new DomainException('IDEMPOTENCY_CONFLICT');
            }
            return $this->attachmentEnvelope($attachment, $challenge, replayed: true, existing: false);
        }

        if ((string) $challenge['state'] === 'quarantined') {
            throw new DomainException((string) $challenge['quarantine_reason']);
        }

        // Quarantine-only dispositions never attach; the record is journaled quarantined.
        if (!in_array((string) $challenge['disposition'], self::ATTACHABLE_DISPOSITIONS, true)) {
            $reason = self::QUARANTINE_REASONS[(string) $challenge['disposition']];
            $this->journalQuarantine($challenge, $reason, $now, $provenance);
            throw new DomainException($reason);
        }

        // The verified registration email must equal the legacy record's keyed digest.
        $normalized = FocusaSpec152eEmailNormalizer::exact($verifiedEmail);
        if (!hash_equals((string) $challenge['email_lookup_digest'], $this->registrationSecrets->emailLookupDigest($normalized))) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }

        $digest = $this->digest([
            'operation' => 'attach_verified',
            'challenge_uuid' => $challengeUuid,
            'record_handle' => $challenge['record_handle'],
            'verified_email' => $verifiedEmail,
            'legacy_key' => (string) ($input['legacy_key'] ?? ''),
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        $mergeInput = [
            'registration_uuid' => (string) ($input['registration_uuid'] ?? ''),
            'verified_email' => $verifiedEmail,
            'verification_method' => (string) ($input['verification_method'] ?? ''),
            'transactional_consent_at' => (string) ($input['transactional_consent_at'] ?? ''),
            'promotional_consent_at' => $input['promotional_consent_at'] ?? null,
            'wordpress_user_id' => $input['wordpress_user_id'] ?? null,
            'stripe_customer_id' => $input['stripe_customer_id'] ?? null,
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'migration_provenance' => $provenance,
            'prior_purchases' => $input['prior_purchases'] ?? [],
            'legacy_key' => (string) ($input['legacy_key'] ?? ''),
            'legacy_evidence' => $input['legacy_evidence'] ?? [],
        ];

        try {
            $evidenceDigest = FocusaSpec152eLegacyActivationAdapter::validateLegacyEvidence($mergeInput['legacy_evidence']);
            $promoted = $this->promotion->mergeLegacyVerified($mergeInput);
        } catch (DomainException $error) {
            if (in_array($error->getMessage(), self::FAILURE_CODES_THAT_QUARANTINE, true)) {
                $this->journalQuarantine($challenge, $error->getMessage(), $now, $provenance);
            }
            throw $error;
        }

        return $this->transaction(function () use ($challengeUuid, $challenge, $promoted, $evidenceDigest, $idempotencyKey, $digest, $provenance, $now): array {
            $table = $this->schema->table('wpuiai_legacy_customer_attachments');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (challenge_uuid, attachment_uuid, account_uuid, edd_customer_id, linked_orders,
                 linked_licenses, evidence_digest, idempotency_key, request_digest, attached_at,
                 migration_provenance)
                SELECT :challenge, :attachment, :account, :customer, :orders, :licenses, :evidence,
                       :idem, :request_digest, :attached, :provenance
                WHERE NOT EXISTS (SELECT 1 FROM {$table} WHERE challenge_uuid = :existing_challenge)");
            $statement->execute([
                ':challenge' => $challengeUuid,
                ':attachment' => self::uuid(),
                ':account' => (string) $promoted['account_uuid'],
                ':customer' => (int) $promoted['edd_customer_id'],
                ':orders' => json_encode($promoted['linked_orders'], JSON_THROW_ON_ERROR),
                ':licenses' => json_encode($promoted['linked_licenses'], JSON_THROW_ON_ERROR),
                ':evidence' => $evidenceDigest,
                ':idem' => $idempotencyKey,
                ':request_digest' => $digest,
                ':attached' => $now,
                ':provenance' => $this->encodeCanonical($provenance),
                ':existing_challenge' => $challengeUuid,
            ]);
            $challengeTable = $this->schema->table('wpuiai_legacy_customer_challenges');
            $update = $this->db->prepare("UPDATE {$challengeTable}
                SET state = 'attached', quarantine_reason = NULL, updated_at = :updated
                WHERE challenge_uuid = :challenge AND state = 'open'");
            $update->execute([':updated' => $now, ':challenge' => $challengeUuid]);
            $this->journal($challengeUuid, 'attached', $now, [
                'record_handle' => $challenge['record_handle'],
                'account_uuid' => (string) $promoted['account_uuid'],
                'edd_customer_id' => (int) $promoted['edd_customer_id'],
            ], $provenance);
            $attachment = $this->findAttachment($challengeUuid);
            return $this->attachmentEnvelope($attachment, $challenge, replayed: false, existing: false);
        });
    }

    /**
     * Explicitly quarantine a legacy record without verification and without
     * entitlement loss. The journal keeps the record recoverable; the record can
     * never attach in quarantined state and cannot activate new nodes (the legacy
     * activation adapter still requires mailbox verification).
     */
    public function quarantineRecord(array $input): array
    {
        $handle = $this->assertHandle((string) ($input['record_handle'] ?? ''));
        $surface = (string) ($input['surface'] ?? '');
        if (!in_array($surface, self::SURFACES, true)) {
            throw new InvalidArgumentException('known legacy record surface required');
        }
        $disposition = (string) ($input['disposition'] ?? '');
        if (!in_array($disposition, self::DISPOSITIONS, true)) {
            throw new InvalidArgumentException('known legacy record disposition required');
        }
        $reason = (string) ($input['quarantine_reason'] ?? '');
        if (!in_array($reason, self::FAILURE_CODES_THAT_QUARANTINE, true)) {
            throw new InvalidArgumentException('bounded quarantine reason required');
        }
        $emailDigest = (string) ($input['email_lookup_digest'] ?? '');
        if (preg_match(self::EMAIL_DIGEST_PATTERN, $emailDigest) !== 1) {
            throw new InvalidArgumentException('bounded keyed email lookup digest required');
        }
        $evidence = $input['legacy_evidence'] ?? [];
        $evidenceDigest = FocusaSpec152eLegacyActivationAdapter::validateLegacyEvidence($evidence);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $digest = $this->digest([
            'operation' => 'quarantine_record',
            'record_handle' => $handle,
            'surface' => $surface,
            'disposition' => $disposition,
            'quarantine_reason' => $reason,
            'email_lookup_digest' => $emailDigest,
            'legacy_evidence' => $evidence,
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($handle, $surface, $disposition, $reason, $emailDigest, $evidenceDigest, $requestId, $idempotencyKey, $provenance, $now, $digest): array {
            $table = $this->schema->table('wpuiai_legacy_customer_challenges');
            $statement = $this->db->prepare("SELECT * FROM {$table} WHERE record_handle = :handle");
            $statement->execute([':handle' => $handle]);
            $existing = $statement->fetch(PDO::FETCH_ASSOC);
            if ($existing !== false) {
                if ((string) $existing['state'] === 'attached') {
                    throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
                }
                if (hash_equals((string) $existing['idempotency_key'], $idempotencyKey)
                    && hash_equals((string) $existing['request_digest'], $digest)
                    && (string) $existing['state'] === 'quarantined') {
                    return $this->quarantineEnvelope($existing, replayed: true, existing: false);
                }
                $wasQuarantined = (string) $existing['state'] === 'quarantined'
                    && hash_equals((string) $existing['quarantine_reason'], $reason);
                $challenge = $this->setQuarantined($existing, $reason, $now, $provenance);
                return $this->quarantineEnvelope($challenge, replayed: false, existing: $wasQuarantined);
            }

            $challengeUuid = self::uuid();
            $statement = $this->db->prepare("INSERT INTO {$table}
                (challenge_uuid, record_handle, surface, disposition, mode, state, email_lookup_digest,
                 evidence_digest, quarantine_reason, request_id, idempotency_key, request_digest,
                 created_at, updated_at, migration_provenance)
                VALUES (:challenge, :handle, :surface, :disposition, :mode, 'quarantined', :digest,
                 :evidence, :reason, :request, :idem, :request_digest, :created, :updated, :provenance)");
            $statement->execute([
                ':challenge' => $challengeUuid,
                ':handle' => $handle,
                ':surface' => $surface,
                ':disposition' => $disposition,
                ':mode' => $this->modeFor($disposition),
                ':digest' => $emailDigest,
                ':evidence' => $evidenceDigest,
                ':reason' => $reason,
                ':request' => $requestId,
                ':idem' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':updated' => $now,
                ':provenance' => $this->encodeCanonical($provenance),
            ]);
            $this->journal($challengeUuid, 'quarantined', $now, [
                'record_handle' => $handle,
                'surface' => $surface,
                'disposition' => $disposition,
                'quarantine_reason' => $reason,
                'state' => 'quarantined',
            ], $provenance);
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'legacy_customer_quarantined',
                'challenge_uuid' => $challengeUuid,
                'record_handle' => $handle,
                'surface' => $surface,
                'disposition' => $disposition,
                'state' => 'quarantined',
                'quarantine_reason' => $reason,
                'replayed' => false,
                'existing' => false,
            ];
        });
    }

    /**
     * Reopen a quarantined record. The record becomes recoverable: a later attach
     * attempt must again pass verification, evidence, and the disposition gate.
     * Never reopens attached records and never restores entitlement.
     */
    public function reopenQuarantined(array $input): array
    {
        $challengeUuid = $this->assertUuid((string) ($input['challenge_uuid'] ?? ''), 'challenge');
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $this->provenance($input['migration_provenance'] ?? []);
        $now = $this->now();
        $challenge = $this->findChallenge($challengeUuid);
        if ((string) $challenge['state'] !== 'quarantined') {
            throw new OutOfBoundsException('legacy migration challenge is not quarantined');
        }
        return $this->transaction(function () use ($challenge, $requestId, $idempotencyKey, $provenance, $now): array {
            $table = $this->schema->table('wpuiai_legacy_customer_challenges');
            $statement = $this->db->prepare("UPDATE {$table}
                SET state = 'open', quarantine_reason = NULL, updated_at = :updated
                WHERE challenge_uuid = :challenge AND state = 'quarantined'");
            $statement->execute([':updated' => $now, ':challenge' => $challenge['challenge_uuid']]);
            $this->journal((string) $challenge['challenge_uuid'], 'reopened', $now, [
                'record_handle' => $challenge['record_handle'],
                'surface' => $challenge['surface'],
                'disposition' => $challenge['disposition'],
                'state' => 'open',
                'request_id' => $requestId,
                'idempotency_key' => $idempotencyKey,
            ], $provenance);
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => 'legacy_customer_reopened',
                'challenge_uuid' => $challenge['challenge_uuid'],
                'record_handle' => $challenge['record_handle'],
                'surface' => $challenge['surface'],
                'disposition' => $challenge['disposition'],
                'mode' => $challenge['mode'],
                'state' => 'open',
                'reopened' => true,
            ];
        });
    }

    // ── internal helpers ────────────────────────────────────────────────

    private function journalQuarantine(array $challenge, string $reason, string $now, array $provenance): void
    {
        $this->transaction(function () use ($challenge, $reason, $now, $provenance): void {
            $this->setQuarantined($challenge, $reason, $now, $provenance);
        });
    }

    private function setQuarantined(array $challenge, string $reason, string $now, array $provenance): array
    {
        $table = $this->schema->table('wpuiai_legacy_customer_challenges');
        $statement = $this->db->prepare("UPDATE {$table}
            SET state = 'quarantined', quarantine_reason = :reason, updated_at = :updated
            WHERE challenge_uuid = :challenge");
        $statement->execute([
            ':reason' => $reason,
            ':updated' => $now,
            ':challenge' => $challenge['challenge_uuid'],
        ]);
        $this->journal((string) $challenge['challenge_uuid'], 'quarantined', $now, [
            'record_handle' => $challenge['record_handle'],
            'surface' => $challenge['surface'],
            'disposition' => $challenge['disposition'],
            'quarantine_reason' => $reason,
            'state' => 'quarantined',
        ], $provenance);
        $challenge['state'] = 'quarantined';
        $challenge['quarantine_reason'] = $reason;
        return $challenge;
    }

    private function replayOrExistingChallenge(array $existing, string $digest, string $idempotencyKey, string $now, string $action): array
    {
        if (!hash_equals((string) $existing['idempotency_key'], $idempotencyKey)) {
            return [
                'schema' => self::RESULT_SCHEMA,
                'action' => $action,
                'challenge_uuid' => $existing['challenge_uuid'],
                'record_handle' => $existing['record_handle'],
                'surface' => $existing['surface'],
                'disposition' => $existing['disposition'],
                'mode' => $existing['mode'],
                'state' => $existing['state'],
                'quarantine_reason' => $existing['quarantine_reason'],
                'replayed' => false,
                'existing' => true,
            ];
        }
        if (!hash_equals((string) $existing['request_digest'], $digest)) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => $action,
            'challenge_uuid' => $existing['challenge_uuid'],
            'record_handle' => $existing['record_handle'],
            'surface' => $existing['surface'],
            'disposition' => $existing['disposition'],
            'mode' => $existing['mode'],
            'state' => $existing['state'],
            'quarantine_reason' => $existing['quarantine_reason'],
            'replayed' => true,
            'existing' => false,
        ];
    }

    private function attachmentEnvelope(array $attachment, array $challenge, bool $replayed, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => 'legacy_customer_attached',
            'attachment_uuid' => $attachment['attachment_uuid'],
            'challenge_uuid' => $challenge['challenge_uuid'],
            'record_handle' => $challenge['record_handle'],
            'surface' => $challenge['surface'],
            'disposition' => $challenge['disposition'],
            'account_uuid' => $attachment['account_uuid'],
            'edd_customer_id' => (int) $attachment['edd_customer_id'],
            'linked_orders' => json_decode((string) $attachment['linked_orders'], true, 512, JSON_THROW_ON_ERROR),
            'linked_licenses' => json_decode((string) $attachment['linked_licenses'], true, 512, JSON_THROW_ON_ERROR),
            'merged_once' => true,
            'replayed' => $replayed,
            'existing' => $existing,
        ];
    }

    private function quarantineEnvelope(array $challenge, bool $replayed, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'action' => 'legacy_customer_quarantined',
            'challenge_uuid' => $challenge['challenge_uuid'],
            'record_handle' => $challenge['record_handle'],
            'surface' => $challenge['surface'],
            'disposition' => $challenge['disposition'],
            'state' => $challenge['state'],
            'quarantine_reason' => $challenge['quarantine_reason'],
            'replayed' => $replayed,
            'existing' => $existing,
        ];
    }

    private function findChallenge(string $challengeUuid): array
    {
        $table = $this->schema->table('wpuiai_legacy_customer_challenges');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE challenge_uuid = :challenge");
        $statement->execute([':challenge' => $challengeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('legacy migration challenge not found');
        }
        return $row;
    }

    private function findAttachment(string $challengeUuid): array
    {
        $table = $this->schema->table('wpuiai_legacy_customer_attachments');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE challenge_uuid = :challenge");
        $statement->execute([':challenge' => $challengeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('legacy migration attachment not found');
        }
        return $row;
    }

    private function journal(string $challengeUuid, string $eventType, string $occurredAt, array $detail, array $provenance): void
    {
        $table = $this->schema->table('wpuiai_legacy_customer_journal');
        $journalKey = hash('sha256', self::RESULT_SCHEMA . "\n" . $eventType . "\n" . $occurredAt . "\n" . $this->encodeCanonical($detail));
        $statement = $this->db->prepare("INSERT INTO {$table}
            (journal_key, challenge_uuid, event_type, occurred_at, detail, migration_provenance)
            SELECT :key, :challenge, :event, :occurred, :detail, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$table} WHERE journal_key = :existing_key)");
        $statement->execute([
            ':key' => $journalKey,
            ':challenge' => $challengeUuid,
            ':event' => $eventType,
            ':occurred' => $occurredAt,
            ':detail' => $this->encodeCanonical($detail),
            ':provenance' => $this->encodeCanonical($provenance),
            ':existing_key' => $journalKey,
        ]);
    }

    private function modeFor(string $disposition): string
    {
        if (in_array($disposition, self::INVITE_DISPOSITIONS, true)) {
            return 'invite';
        }
        if (in_array($disposition, self::CHALLENGE_DISPOSITIONS, true)) {
            return 'challenge';
        }
        throw new InvalidArgumentException('known legacy record disposition required');
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

    private function assertUuid(string $uuid, string $kind): string
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
        return $uuid;
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
        return FocusaSpec152eLegacyCustomerMigrationSchema::encodeCanonical($value);
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eLegacyCustomerMigrationSchema::assertTimestamp($now);
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
