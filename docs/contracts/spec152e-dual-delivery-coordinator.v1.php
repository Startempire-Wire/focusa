<?php
// Dual email + terminal license delivery coordinator (Spec 152E §16). One
// canonical EDD Software Licensing key is delivered through the approved
// transactional license email (branded facade sender identity) and the
// one-time device-encrypted terminal envelope. Masked outcomes and bounces
// are journaled per channel; the plaintext key and unmasked email never enter
// any response, journal, or generic log. Settlement never mints a second key,
// and authenticated recovery after a partial delivery re-delivers the SAME
// canonical key (never a new license). Spec 158 implementation is excluded.
declare(strict_types=1);

final class FocusaSpec152eDualLicenseDeliveryMigration
{
    public const SCHEMA = 'focusa.spec152e.dual_license_delivery.v1';
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
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($appliedAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $deliveries = $this->table('wpuiai_dual_license_deliveries');
        $idempotency = $this->table('wpuiai_dual_license_delivery_idempotency');
        $migrations = $this->table('wpuiai_dual_license_delivery_schema_migrations');
        $events = $this->table('wpuiai_dual_license_delivery_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$deliveries} (
            delivery_handle VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL UNIQUE,
            account_uuid {$uuid} NULL,
            edd_customer_id BIGINT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            license_key_digest VARCHAR(64) NOT NULL,
            license_key_mask VARCHAR(32) NOT NULL,
            email_channel_status VARCHAR(16) NOT NULL DEFAULT 'none',
            email_channel_attempts BIGINT NOT NULL DEFAULT 0,
            email_attempted_at VARCHAR(32) NULL,
            email_delivered_at VARCHAR(32) NULL,
            email_outcome_code VARCHAR(64) NOT NULL DEFAULT 'none',
            terminal_channel_status VARCHAR(16) NOT NULL DEFAULT 'pending',
            terminal_channel_attempts BIGINT NOT NULL DEFAULT 0,
            terminal_delivered_at VARCHAR(32) NULL,
            resolved_state VARCHAR(32) NOT NULL DEFAULT 'pending',
            recovery_handle VARCHAR(64) NULL,
            recovery_resolved_at VARCHAR(32) NULL,
            recovery_envelope_id VARCHAR(64) NULL,
            recovery_envelope_payload TEXT NULL,
            recovery_envelope_expires_at VARCHAR(32) NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL UNIQUE,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_dual_delivery_registration
            ON {$deliveries} (registration_uuid)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_dual_delivery_resolved
            ON {$deliveries} (resolved_state, updated_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_dual_delivery_retention
            ON {$deliveries} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            request_id {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_dual_delivery_idem_retention
            ON {$idempotency} (retention_until)");
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
            ':provenance' => FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance),
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: dual-delivery journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_dual_license_delivery_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance));
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($provenance),
            ':existing_key' => $eventKey,
        ]);
        return ['schema' => self::SCHEMA, 'action' => 'preserve', 'event_key' => $eventKey];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }
}

/**
 * Dual delivery coordinator: settles one canonical EDD key across the
 * transactional license email and the one-time terminal envelope, journals
 * masked outcomes/bounces per channel, confirms both channels resolve one
 * license ID/key, and provides authenticated recovery after partial delivery
 * without ever minting a second key.
 */
final class FocusaSpec152eDualLicenseDeliveryCoordinator
{
    public const SCHEMA = 'focusa.spec152e.dual_license_delivery.v1';
    public const DELIVERY_SCHEMA = 'focusa.spec152e.dual_delivery_state.v1';
    public const RECOVERY_SCHEMA = 'focusa.spec152e.dual_delivery_recovery.v1';
    public const RETENTION_SECONDS = 2592000;
    public const RECOVERY_ENVELOPE_TTL_SECONDS = 1800;

    public const EMAIL_STATUSES = ['none', 'sent', 'delivered', 'bounced', 'suppressed', 'failed'];
    public const EMAIL_OUTCOMES = ['none', 'soft_bounce', 'hard_bounce', 'suppressed_transactional', 'suppressed_all', 'provider_failed'];
    public const TERMINAL_STATUSES = ['none', 'pending', 'delivered', 'failed'];
    public const RESOLVED_STATES = ['pending', 'email_only', 'terminal_only', 'both_delivered', 'recovery_required'];
    public const RECOVERY_CHANNELS = ['email', 'terminal'];

    private const KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';
    private const PRODUCT_CODE_PATTERN = '/^[A-Za-z0-9_]{2,128}$/D';
    private const DELIVERY_READY_STATES = [
        FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
        FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY,
        FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED,
        FocusaSpec152eActivationRegistrationState::LEASE_ISSUED,
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eDualLicenseDeliveryMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eTransactionalMailAdapter $mailAdapter,
        callable $clock,
        private string $eddPrefix = 'wp_',
        private int $retention = self::RETENTION_SECONDS,
        private int $recoveryEnvelopeTtl = self::RECOVERY_ENVELOPE_TTL_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $eddPrefix) !== 1) {
            throw new InvalidArgumentException('invalid EDD table prefix');
        }
        if ($this->retention < 1 || $this->recoveryEnvelopeTtl < 1) {
            throw new InvalidArgumentException('positive retention and recovery TTL required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Settle dual delivery after canonical issuance. Required input:
     *   - registration_id, facade (full registry entry), request_id,
     *     idempotency_key; product_name and support_email are optional.
     *
     * Resolves the ONE canonical EDD key (never mints), journals the
     * dual-delivery record, and sends the transactional license email with
     * the full key (approved §16.1 content). A repeated settle returns the
     * existing journal without re-sending email or minting anything. Returns
     * the masked delivery state; the plaintext key and raw email never appear.
     */
    public function settle(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $facade = $input['facade'] ?? null;
        $this->assertUuid($registrationId, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if (!is_array($facade) || !isset($facade['facade_id'], $facade['sender'], $facade['brand'], $facade['paths'], $facade['exact_origins'])) {
            throw new InvalidArgumentException('registered facade entry required');
        }
        $productName = (string) ($input['product_name'] ?? '');
        if ($productName !== '' && (strlen($productName) > 191 || preg_match('/[\r\n]/', $productName))) {
            throw new InvalidArgumentException('bounded product name required');
        }
        $supportEmail = (string) ($input['support_email'] ?? '');
        if ($supportEmail !== '' && filter_var($supportEmail, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid support email required');
        }

        $now = $this->now();
        $registration = $this->loadDeliveryRegistration($registrationId, $now);
        $license = $this->resolveCanonicalKey((int) $registration['edd_license_id'], (string) $registration['product_code']);
        $digest = $this->requestDigest([
            'operation' => 'settle_dual_delivery',
            'registration_id' => $registrationId,
            'edd_license_id' => (int) $registration['edd_license_id'],
            'license_key_digest' => $license['digest'],
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($registrationId, $facade, $productName, $supportEmail, $requestId, $idempotencyKey, $now, $digest, $registration, $license): array {
            $existing = $this->findJournalByRegistration($registrationId);
            if ($existing !== null) {
                return $this->settleResponse($existing, false, $registrationId);
            }
            $row = $this->insertJournal($registration, $license, 'pending', $requestId, $idempotencyKey, $digest, $now);

            // Approved transactional license email: full human key, product and
            // order identity, safe activation instructions, account-management
            // and recovery links, support information; no promotional content.
            $recipient = $this->registrationSecrets->decryptEmail((string) $registration['encrypted_normalized_email']);
            $attempt = $this->mailAdapter->sendLicenseDelivery([
                'facade' => $facade,
                'to' => $recipient,
                'license_key' => $license['key'],
                'product_code' => (string) $registration['product_code'],
                'product_name' => $productName !== '' ? $productName : (string) $registration['product_code'],
                'order_id' => (int) ($registration['edd_order_id'] ?? 0),
                'order_item_id' => (int) ($registration['edd_order_item_id'] ?? 0),
                'registration_id' => $registrationId,
                'support_email' => $supportEmail,
            ]);
            $status = $attempt['delivery_status'] === 'suppressed' ? 'suppressed' : 'sent';
            $emailOutcome = $status === 'suppressed' ? 'suppressed_transactional' : 'none';
            $row = $this->recordEmailOnJournal($row, $status, $emailOutcome, $now, $now, $status !== 'suppressed');
            $row = $this->refreshResolvedState($row, $now);
            $this->registrations->recordEmailDeliveryState(
                $registrationId,
                $status,
                $status === 'suppressed' ? 'suppressed_transactional' : null,
                $now,
                $requestId,
                'idem-email-' . substr($idempotencyKey, 0, 128),
                true,
            );
            return $this->settleResponse($row, true, $registrationId);
        });
    }

    /**
     * Record a masked provider delivery outcome (delivered/bounced/complained)
     * for the email channel. Required input: registration_id, delivery_status,
     * bounce_type when bounced, occurred_at, request_id, idempotency_key.
     * Idempotent by request/idempotency key; returns the masked delivery state.
     */
    public function recordEmailOutcome(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $deliveryStatus = (string) ($input['delivery_status'] ?? '');
        $bounceType = (string) ($input['bounce_type'] ?? 'soft');
        $occurredAt = (string) ($input['occurred_at'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertUuid($registrationId, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($occurredAt);
        if (!in_array($deliveryStatus, ['sent', 'delivered', 'bounced', 'complained'], true)) {
            throw new InvalidArgumentException('bounded provider delivery status required');
        }
        if (!in_array($bounceType, ['soft', 'hard'], true)) {
            throw new InvalidArgumentException('bounded bounce type required');
        }
        [$status, $outcome] = match ($deliveryStatus) {
            'delivered' => ['delivered', 'none'],
            'sent' => ['sent', 'none'],
            'bounced' => ['bounced', $bounceType === 'hard' ? 'hard_bounce' : 'soft_bounce'],
            'complained' => ['failed', 'suppressed_transactional'],
        };
        $digest = $this->requestDigest([
            'operation' => 'record_email_outcome',
            'registration_id' => $registrationId,
            'delivery_status' => $deliveryStatus,
            'bounce_type' => $bounceType,
            'occurred_at' => $occurredAt,
            'request_id' => $requestId,
        ]);
        return $this->transaction(function () use ($registrationId, $status, $outcome, $occurredAt, $requestId, $idempotencyKey, $digest): array {
            $row = $this->loadJournal($registrationId);
            if ($this->replayIdempotency($idempotencyKey, $digest) !== null) {
                return $this->stateResponse($row, $registrationId);
            }
            if ($row['email_channel_status'] === 'delivered') {
                if ($status === 'delivered') {
                    // Duplicate delivered event: idempotent no-op.
                    return $this->stateResponse($row, $registrationId);
                }
                throw new InvalidArgumentException('already-delivered email cannot later fail');
            }
            $now = $this->now();
            $row = $this->recordEmailOnJournal($row, $status, $outcome, $occurredAt, $now, false);
            $row = $this->refreshResolvedState($row, $now);
            $this->registrations->recordEmailDeliveryState(
                $registrationId,
                $status,
                $outcome !== 'none' ? $outcome : null,
                $occurredAt,
                $requestId,
                $idempotencyKey,
                false,
            );
            $this->recordIdempotency($idempotencyKey, $digest, $registrationId, $requestId, $now);
            return $this->stateResponse($row, $registrationId);
        });
    }

    /**
     * Confirm the terminal channel delivered the SAME canonical key. Required
     * input: registration_id, edd_license_id, license_key_digest, request_id,
     * idempotency_key. A digest/license mismatch with the journal fails closed
     * (DUAL_DELIVERY_KEY_MISMATCH) — email and terminal must resolve one
     * license ID/key.
     */
    public function noteTerminalDelivered(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $eddLicenseId = (int) ($input['edd_license_id'] ?? 0);
        $licenseKeyDigest = (string) ($input['license_key_digest'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertUuid($registrationId, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if ($eddLicenseId < 1 || preg_match('/^[a-f0-9]{64}$/D', $licenseKeyDigest) !== 1) {
            throw new InvalidArgumentException('bounded license identity required');
        }
        $digest = $this->requestDigest([
            'operation' => 'note_terminal_delivered',
            'registration_id' => $registrationId,
            'edd_license_id' => $eddLicenseId,
            'license_key_digest' => $licenseKeyDigest,
            'request_id' => $requestId,
        ]);
        return $this->transaction(function () use ($registrationId, $eddLicenseId, $licenseKeyDigest, $requestId, $idempotencyKey, $digest): array {
            $row = $this->loadJournal($registrationId);
            if ($this->replayIdempotency($idempotencyKey, $digest) !== null) {
                return $this->stateResponse($row, $registrationId);
            }
            if ((int) $row['edd_license_id'] !== $eddLicenseId
                || !hash_equals((string) $row['license_key_digest'], $licenseKeyDigest)) {
                throw new DomainException('DUAL_DELIVERY_KEY_MISMATCH');
            }
            $now = $this->now();
            $row = $this->recordTerminalOnJournal($row, 'delivered', $now, $now, true);
            $row = $this->refreshResolvedState($row, $now);
            $this->recordIdempotency($idempotencyKey, $digest, $registrationId, $requestId, $now);
            return $this->stateResponse($row, $registrationId);
        });
    }

    /**
     * Masked registration delivery state (public-safe). Required input:
     * registration_id.
     */
    public function deliveryState(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $this->assertUuid($registrationId, 'registration');
        $row = $this->findJournalByRegistration($registrationId);
        if ($row === null) {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        return $this->stateResponse($row, $registrationId);
    }

    /**
     * Authenticated recovery after partial delivery. Required input:
     *   - registration_id, poll_credential (authenticated recovery),
     *     recovery_channel ('email'|'terminal'), request_id, idempotency_key;
     *   - facade (full registry entry) when recovery_channel is 'email'.
     *
     * Re-delivers the SAME canonical key through the requested channel; never
     * mints a second key. Terminal recovery returns a fresh one-time envelope
     * sealed to the bound device key; the identical envelope replays for the
     * same request idempotency key. Email recovery re-sends the license email
     * with masked outcome journaling.
     */
    public function recover(array $input): array
    {
        $registrationId = (string) ($input['registration_id'] ?? '');
        $pollCredential = (string) ($input['poll_credential'] ?? '');
        $channel = (string) ($input['recovery_channel'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $facade = $input['facade'] ?? null;
        $this->assertUuid($registrationId, 'registration');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        if (!in_array($channel, self::RECOVERY_CHANNELS, true)) {
            throw new InvalidArgumentException('bounded recovery channel required');
        }
        if ($pollCredential === '' || strlen($pollCredential) > 512 || preg_match('/[\r\n]/', $pollCredential)) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if ($channel === 'email' && (!is_array($facade) || !isset($facade['facade_id'], $facade['sender'], $facade['brand'], $facade['paths'], $facade['exact_origins']))) {
            throw new InvalidArgumentException('registered facade entry required for email recovery');
        }

        $now = $this->now();
        $registration = $this->authenticateRecovery($registrationId, $pollCredential, $now);
        $license = $this->resolveCanonicalKey((int) $registration['edd_license_id'], (string) $registration['product_code']);
        $digest = $this->requestDigest([
            'operation' => 'recover_dual_delivery',
            'registration_id' => $registrationId,
            'recovery_channel' => $channel,
            'edd_license_id' => (int) $registration['edd_license_id'],
            'license_key_digest' => $license['digest'],
            'request_id' => $requestId,
        ]);

        return $this->transaction(function () use ($registrationId, $channel, $facade, $registration, $license, $requestId, $idempotencyKey, $now, $digest): array {
            $row = $this->loadJournal($registrationId);
            if ($this->replayIdempotency($idempotencyKey, $digest) !== null) {
                return $this->recoveryReplayResponse($row, $requestId);
            }
            if ($row['resolved_state'] === 'both_delivered') {
                throw new DomainException('DUAL_DELIVERY_ALREADY_SETTLED');
            }
            if ((int) $row['edd_license_id'] !== (int) $registration['edd_license_id']
                || !hash_equals((string) $row['license_key_digest'], $license['digest'])) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }
            if ($channel === 'terminal') {
                $result = $this->recoverByTerminal($row, $registration, $license, $requestId, $now);
            } else {
                $result = $this->recoverByEmail($row, $registration, $license, $facade, $requestId, $idempotencyKey, $now);
            }
            $this->recordIdempotency($idempotencyKey, $digest, $registrationId, $requestId, $now);
            return $result;
        });
    }

    /** Bounded journal count for settlement/reconciliation. */
    public function deliveryCount(): int
    {
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function findByRegistration(string $registrationId): ?array
    {
        $this->assertUuid($registrationId, 'registration');
        return $this->findJournalByRegistration($registrationId);
    }

    // ── private: recovery channel implementations ─────────────────────────

    private function recoverByTerminal(array $row, array $registration, array $license, string $requestId, string $now): array
    {
        $devicePublicB64 = (string) $registration['device_public_key'];
        if (preg_match('/^[A-Za-z0-9_-]{43}$/D', $devicePublicB64) !== 1) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }
        $envelopeId = self::opaqueToken('env_');
        $expiresAt = min(
            FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, $this->recoveryEnvelopeTtl),
            (string) $registration['expires_at'],
        );
        $claims = FocusaSpec152eTerminalDeliveryEnvelope::buildClaims([
            'registration_id' => (string) $registration['registration_uuid'],
            'account_uuid' => (string) $registration['account_uuid'],
            'customer_id' => (int) $registration['edd_customer_id'],
            'edd_license_id' => (int) $registration['edd_license_id'],
            'product_code' => (string) $registration['product_code'],
        ], $license['key'], $envelopeId, $now, $expiresAt);
        $envelope = FocusaSpec152eTerminalEnvelopeCrypto::seal(
            FocusaSpec152eTerminalEnvelopeCrypto::base64UrlDecode($devicePublicB64),
            FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($claims),
        );
        $payload = FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($envelope);
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $statement = $this->db->prepare("UPDATE {$table} SET
            recovery_envelope_id = :envelope_id, recovery_envelope_payload = :payload,
            recovery_envelope_expires_at = :expires_at, updated_at = :updated
            WHERE delivery_handle = :handle");
        $statement->execute([
            ':envelope_id' => $envelopeId,
            ':payload' => $payload,
            ':expires_at' => $expiresAt,
            ':updated' => $now,
            ':handle' => (string) $row['delivery_handle'],
        ]);
        $row = $this->findJournalByRegistration((string) $row['registration_uuid']);
        $row = $this->recordTerminalOnJournal($row, 'delivered', $now, $now, true);
        $row = $this->refreshResolvedState($row, $now);
        return $this->recoveryResponse($row, $requestId, $envelopeId, $payload);
    }

    private function recoverByEmail(array $row, array $registration, array $license, array $facade, string $requestId, string $idempotencyKey, string $now): array
    {
        $recipient = $this->registrationSecrets->decryptEmail((string) $registration['encrypted_normalized_email']);
        $attempt = $this->mailAdapter->sendLicenseDelivery([
            'facade' => $facade,
            'to' => $recipient,
            'license_key' => $license['key'],
            'product_code' => (string) $registration['product_code'],
            'product_name' => (string) $registration['product_code'],
            'order_id' => (int) ($registration['edd_order_id'] ?? 0),
            'order_item_id' => (int) ($registration['edd_order_item_id'] ?? 0),
            'registration_id' => (string) $registration['registration_uuid'],
        ]);
        $status = $attempt['delivery_status'] === 'suppressed' ? 'suppressed' : 'sent';
        $emailOutcome = $status === 'suppressed' ? 'suppressed_transactional' : 'none';
        $row = $this->recordEmailOnJournal($row, $status, $emailOutcome, $now, $now, $status !== 'suppressed');
        $row = $this->refreshResolvedState($row, $now);
        $this->registrations->recordEmailDeliveryState(
            (string) $registration['registration_uuid'],
            $status,
            $status === 'suppressed' ? 'suppressed_transactional' : null,
            $now,
            $requestId,
            'idem-email-' . substr($idempotencyKey, 0, 128),
            true,
        );
        return $this->recoveryResponse($row, $requestId, null, null);
    }

    // ── private: journal helpers ──────────────────────────────────────────

    private function insertJournal(array $registration, array $license, string $terminalStatus, string $requestId, string $idempotencyKey, string $digest, string $now): array
    {
        $handle = self::opaqueToken('dlv_');
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (delivery_handle, registration_uuid, account_uuid, edd_customer_id, edd_license_id,
             product_code, license_key_digest, license_key_mask,
             email_channel_status, email_channel_attempts, email_attempted_at, email_delivered_at, email_outcome_code,
             terminal_channel_status, terminal_channel_attempts, terminal_delivered_at,
             resolved_state, recovery_handle, recovery_resolved_at,
             recovery_envelope_id, recovery_envelope_payload, recovery_envelope_expires_at,
             request_id, idempotency_key, request_digest, created_at, updated_at, retention_until)
            VALUES (:handle, :registration, :account, :customer, :license_id,
                    :product, :key_digest, :key_mask,
                    'none', 0, NULL, NULL, 'none',
                    :terminal_status, 0, NULL,
                    'pending', NULL, NULL,
                    NULL, NULL, NULL,
                    :request, :idempotency, :digest, :created, :updated, :retention)");
        $statement->execute([
            ':handle' => $handle,
            ':registration' => (string) $registration['registration_uuid'],
            ':account' => (string) ($registration['account_uuid'] ?? ''),
            ':customer' => (int) ($registration['edd_customer_id'] ?? 0),
            ':license_id' => (int) $registration['edd_license_id'],
            ':product' => (string) $registration['product_code'],
            ':key_digest' => $license['digest'],
            ':key_mask' => $license['mask'],
            ':terminal_status' => $terminalStatus,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':digest' => $digest,
            ':created' => $now,
            ':updated' => $now,
            ':retention' => FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, $this->retention),
        ]);
        return $this->findJournalByRegistration((string) $registration['registration_uuid']);
    }

    private function recordEmailOnJournal(array $row, string $status, string $outcome, string $occurredAt, string $now, bool $countsAsAttempt): array
    {
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $set = ['email_channel_status = :status', 'email_outcome_code = :outcome', 'updated_at = :updated'];
        $params = [
            ':status' => $status,
            ':outcome' => $outcome,
            ':updated' => $now,
            ':handle' => (string) $row['delivery_handle'],
        ];
        if ($countsAsAttempt) {
            $set[] = 'email_channel_attempts = email_channel_attempts + 1';
            $set[] = 'email_attempted_at = :attempted_at';
            $params[':attempted_at'] = $occurredAt;
        }
        if ($status === 'delivered') {
            $set[] = 'email_delivered_at = :delivered_at';
            $params[':delivered_at'] = $occurredAt;
        }
        $statement = $this->db->prepare("UPDATE {$table} SET " . implode(', ', $set) . " WHERE delivery_handle = :handle");
        $statement->execute($params);
        return $this->findJournalByRegistration((string) $row['registration_uuid']);
    }

    private function recordTerminalOnJournal(array $row, string $status, string $occurredAt, string $now, bool $countsAsAttempt): array
    {
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $set = ['terminal_channel_status = :status', 'updated_at = :updated'];
        $params = [
            ':status' => $status,
            ':updated' => $now,
            ':handle' => (string) $row['delivery_handle'],
        ];
        if ($countsAsAttempt) {
            $set[] = 'terminal_channel_attempts = terminal_channel_attempts + 1';
            $set[] = 'terminal_delivered_at = :delivered_at';
            $params[':delivered_at'] = $occurredAt;
        }
        $statement = $this->db->prepare("UPDATE {$table} SET " . implode(', ', $set) . " WHERE delivery_handle = :handle");
        $statement->execute($params);
        return $this->findJournalByRegistration((string) $row['registration_uuid']);
    }

    private function refreshResolvedState(array $row, string $now): array
    {
        $resolved = $this->resolveState(
            (string) $row['email_channel_status'],
            (string) $row['email_outcome_code'],
            (string) $row['terminal_channel_status'],
        );
        if ($resolved === $row['resolved_state']) {
            return $row;
        }
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $set = ['resolved_state = :resolved', 'updated_at = :updated'];
        $params = [
            ':resolved' => $resolved,
            ':updated' => $now,
            ':handle' => (string) $row['delivery_handle'],
        ];
        if ($resolved === 'recovery_required' && $row['recovery_handle'] === null) {
            $set[] = 'recovery_handle = :recovery_handle';
            $params[':recovery_handle'] = self::opaqueToken('rec_');
        }
        if ($resolved === 'both_delivered' && $row['recovery_resolved_at'] === null) {
            $set[] = 'recovery_resolved_at = :recovery_resolved_at';
            $params[':recovery_resolved_at'] = $now;
        }
        $statement = $this->db->prepare("UPDATE {$table} SET " . implode(', ', $set) . " WHERE delivery_handle = :handle");
        $statement->execute($params);
        return $this->findJournalByRegistration((string) $row['registration_uuid']);
    }

    private function resolveState(string $emailStatus, string $emailOutcome, string $terminalStatus): string
    {
        $emailDelivered = $emailStatus === 'delivered';
        $terminalDelivered = $terminalStatus === 'delivered';
        if ($emailDelivered && $terminalDelivered) {
            return 'both_delivered';
        }
        if ($emailDelivered) {
            return 'email_only';
        }
        if ($terminalDelivered) {
            return 'terminal_only';
        }
        if (in_array($emailStatus, ['bounced', 'suppressed', 'failed'], true)
            || $terminalStatus === 'failed'
            || $emailOutcome === 'hard_bounce') {
            return 'recovery_required';
        }
        return 'pending';
    }

    private function loadJournal(string $registrationId): array
    {
        $row = $this->findJournalByRegistration($registrationId);
        if ($row === null) {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        return $row;
    }

    private function findJournalByRegistration(string $registrationId): ?array
    {
        $table = $this->schema->table('wpuiai_dual_license_deliveries');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE registration_uuid = :registration LIMIT 1");
        $statement->execute([':registration' => $registrationId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function replayIdempotency(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_dual_license_delivery_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals((string) $row['request_digest'], $digest)) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $idempotencyKey, string $digest, string $registrationId, string $requestId, string $now): void
    {
        $table = $this->schema->table('wpuiai_dual_license_delivery_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, registration_uuid, request_id, request_digest, created_at, retention_until)
            VALUES (:key, :operation, :registration, :request_id, :digest, :created, :retention)");
        $statement->execute([
            ':key' => $idempotencyKey,
            ':operation' => 'dual_license_delivery',
            ':registration' => $registrationId,
            ':request_id' => $requestId,
            ':digest' => $digest,
            ':created' => $now,
            ':retention' => FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, $this->retention),
        ]);
    }

    // ── private: authority helpers ────────────────────────────────────────

    private function loadDeliveryRegistration(string $registrationId, string $now): array
    {
        $registration = $this->findRegistration($registrationId);
        if ((string) $registration['verification_state'] !== 'mailbox_verified') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!in_array((string) $registration['state'], self::DELIVERY_READY_STATES, true)) {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        if ((int) ($registration['edd_license_id'] ?? 0) < 1) {
            throw new DomainException('EDD_LICENSE_PENDING');
        }
        return $registration;
    }

    private function authenticateRecovery(string $registrationId, string $pollCredential, string $now): array
    {
        $registration = $this->findRegistration($registrationId);
        if ($registration['expires_at'] !== null && $now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if ($registration['poll_credential_hash'] === null || $registration['poll_credential_expires_at'] === null) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if ($now >= (string) $registration['poll_credential_expires_at']) {
            throw new DomainException('POLL_CREDENTIAL_EXPIRED');
        }
        if (!hash_equals((string) $registration['poll_credential_hash'], $this->registrationSecrets->pollHash($pollCredential))) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        if ((string) $registration['verification_state'] !== 'mailbox_verified') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ((int) ($registration['edd_license_id'] ?? 0) < 1) {
            throw new DomainException('EDD_LICENSE_PENDING');
        }
        return $registration;
    }

    private function findRegistration(string $registrationId): array
    {
        try {
            return $this->registrations->findByUuid($registrationId);
        } catch (OutOfBoundsException) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
    }

    /** Canonical EDD Software Licensing storage is the only key source. */
    private function resolveCanonicalKey(int $eddLicenseId, string $productCode): array
    {
        if ($eddLicenseId < 1 || preg_match(self::PRODUCT_CODE_PATTERN, $productCode) !== 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT license_key, status FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $eddLicenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || (string) ($row['status'] ?? '') !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $key = (string) $row['license_key'];
        if (preg_match(self::KEY_PATTERN, $key) !== 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return [
            'key' => $key,
            'mask' => FocusaSpec152eTerminalDeliveryEnvelope::maskKey($key),
            'digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($key),
        ];
    }

    // ── private: responses ────────────────────────────────────────────────

    private function settleResponse(array $row, bool $emailSent, string $registrationId): array
    {
        $state = $this->stateResponse($row, $registrationId);
        $state['email_sent'] = $emailSent;
        $state['email_delivery_status'] = (string) $row['email_channel_status'];
        return $state;
    }

    private function stateResponse(array $row, string $registrationId): array
    {
        return [
            'schema' => self::DELIVERY_SCHEMA,
            'registration_id' => $registrationId,
            'delivery_handle' => (string) $row['delivery_handle'],
            'license_key_mask' => (string) $row['license_key_mask'],
            // One canonical digest governs both channels; a terminal-channel
            // digest/license mismatch fails closed in noteTerminalDelivered().
            'same_key_confirmed' => true,
            'email_channel' => [
                'status' => (string) $row['email_channel_status'],
                'attempts' => (int) $row['email_channel_attempts'],
                'outcome' => (string) $row['email_outcome_code'],
                'delivered_at' => $row['email_delivered_at'] === null ? null : (string) $row['email_delivered_at'],
            ],
            'terminal_channel' => [
                'status' => (string) $row['terminal_channel_status'],
                'attempts' => (int) $row['terminal_channel_attempts'],
                'delivered_at' => $row['terminal_delivered_at'] === null ? null : (string) $row['terminal_delivered_at'],
            ],
            'resolved_state' => (string) $row['resolved_state'],
            'recovery_handle' => $row['recovery_handle'] === null ? null : (string) $row['recovery_handle'],
            'recovery_resolved_at' => $row['recovery_resolved_at'] === null ? null : (string) $row['recovery_resolved_at'],
        ];
    }

    private function recoveryResponse(array $row, string $requestId, ?string $envelopeId, ?string $payload): array
    {
        $state = $this->stateResponse($row, (string) $row['registration_uuid']);
        $state['schema'] = self::RECOVERY_SCHEMA;
        $state['request_id'] = $requestId;
        $state['recovery_channel'] = $payload !== null ? 'terminal' : 'email';
        if ($payload !== null) {
            $state['envelope_id'] = $envelopeId;
            $state['one_time_key_envelope'] = FocusaSpec152eTerminalEnvelopeCrypto::base64UrlEncode($payload);
        }
        return $state;
    }

    private function recoveryReplayResponse(array $row, string $requestId): array
    {
        if ($row['recovery_envelope_id'] !== null && $row['recovery_envelope_payload'] !== null) {
            return $this->recoveryResponse($row, $requestId, (string) $row['recovery_envelope_id'], (string) $row['recovery_envelope_payload']);
        }
        return $this->recoveryResponse($row, $requestId, null, null);
    }

    private function requestDigest(array $payload): string
    {
        return hash('sha256', FocusaSpec152eTerminalDeliveryEnvelope::canonicalJsonSafe($payload));
    }

    private function transaction(callable $operation): mixed
    {
        if ($this->db->inTransaction()) {
            return $operation();
        }
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

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        return $now;
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private function assertUuid(string $value, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $value) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} UUID required");
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
