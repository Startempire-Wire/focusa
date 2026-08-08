<?php
// Candidate-owned verified email-identity schema/repository seam. It does not bootstrap WordPress.
declare(strict_types=1);

final class FocusaSpec152eEmailIdentityMigration
{
    public const SCHEMA = 'focusa.spec152e.email_identity.v1';
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
        $identities = $this->table('wpuiai_email_identities');
        $migrations = $this->table('wpuiai_email_identity_migrations');
        $events = $this->table('wpuiai_email_identity_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$identities} (
            identity_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            encrypted_normalized_email TEXT NOT NULL,
            email_lookup_digest VARCHAR(64) NOT NULL UNIQUE,
            verified_at VARCHAR(32) NOT NULL,
            verification_method VARCHAR(32) NOT NULL,
            identity_state VARCHAR(16) NOT NULL CHECK (identity_state IN ('primary', 'linked', 'revoked')),
            transactional_consent_at VARCHAR(32) NULL,
            promotional_consent_at VARCHAR(32) NULL,
            promotional_consent_revoked_at VARCHAR(32) NULL,
            bounce_state VARCHAR(16) NOT NULL DEFAULT 'none' CHECK (bounce_state IN ('none', 'soft', 'hard')),
            bounced_at VARCHAR(32) NULL,
            suppression_state VARCHAR(16) NOT NULL DEFAULT 'none' CHECK (suppression_state IN ('none', 'transactional', 'promotional', 'all')),
            suppressed_at VARCHAR(32) NULL,
            revoked_at VARCHAR(32) NULL,
            source VARCHAR(64) NOT NULL,
            migration_evidence TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        if ($this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql') {
            $this->createMysqlPrimaryConstraint($identities);
        } else {
            $this->db->exec("CREATE UNIQUE INDEX IF NOT EXISTS {$this->prefix}wpuiai_email_one_primary_per_account
                ON {$identities} (account_uuid) WHERE identity_state = 'primary'");
        }
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
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing)");
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied' => $appliedAt,
            ':provenance' => $encoded,
            ':existing' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: identity rows and migration journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_email_identity_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey, ':version' => self::VERSION, ':occurred_at' => $occurredAt,
            ':provenance' => $encoded, ':existing_key' => $eventKey,
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

    private function createMysqlPrimaryConstraint(string $table): void
    {
        // A generated NULL for non-primary rows permits many linked identities while the unique
        // key permits exactly one primary row per account.
        try {
            $this->db->exec("ALTER TABLE {$table} ADD COLUMN primary_account_uuid VARCHAR(36)
                GENERATED ALWAYS AS (CASE WHEN identity_state = 'primary' THEN account_uuid ELSE NULL END) STORED");
        } catch (PDOException $error) {
            if (!str_contains(strtolower($error->getMessage()), 'duplicate column')) {
                throw $error;
            }
        }
        try {
            $this->db->exec("CREATE UNIQUE INDEX {$this->prefix}wpuiai_email_one_primary_per_account
                ON {$table} (primary_account_uuid)");
        } catch (PDOException $error) {
            if (!str_contains(strtolower($error->getMessage()), 'duplicate')) {
                throw $error;
            }
        }
    }
}

final class FocusaSpec152eEmailNormalizer
{
    /**
     * Canonicalization is deliberately provider-neutral: surrounding presentation whitespace
     * and domain case/IDN spelling are canonicalized; local-part dots, plus tags, and case are kept.
     */
    public static function exact(string $submitted): string
    {
        $email = trim($submitted, " \t\r\n\0\x0B");
        if ($email === '' || strlen($email) > 254 || preg_match('/[\x00-\x1F\x7F]/', $email) === 1) {
            throw new InvalidArgumentException('valid email identity required');
        }
        $at = strrpos($email, '@');
        if ($at === false) {
            throw new InvalidArgumentException('valid email identity required');
        }
        $local = substr($email, 0, $at);
        $domain = substr($email, $at + 1);
        if ($local === '' || strlen($local) > 64 || $domain === '') {
            throw new InvalidArgumentException('valid email identity required');
        }
        if (class_exists(Normalizer::class)) {
            $local = Normalizer::normalize($local, Normalizer::FORM_C) ?: $local;
            $domain = Normalizer::normalize($domain, Normalizer::FORM_C) ?: $domain;
        }
        if (function_exists('idn_to_ascii')) {
            $ascii = idn_to_ascii($domain, IDNA_DEFAULT, INTL_IDNA_VARIANT_UTS46);
            if ($ascii === false) {
                throw new InvalidArgumentException('valid email identity required');
            }
            $domain = $ascii;
        }
        $normalized = $local . '@' . strtolower($domain);
        if (filter_var($normalized, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid email identity required');
        }
        return $normalized;
    }
}

final class FocusaSpec152eEmailIdentitySecrets
{
    private string $encryptionKey;
    private string $lookupKey;

    public function __construct(string $encryptionKey, string $lookupKey)
    {
        if (strlen($encryptionKey) !== 32 || strlen($lookupKey) < 32) {
            throw new InvalidArgumentException('independent bounded identity keys required');
        }
        if (hash_equals($encryptionKey, substr($lookupKey, 0, 32))) {
            throw new InvalidArgumentException('encryption and lookup keys must be independent');
        }
        $this->encryptionKey = $encryptionKey;
        $this->lookupKey = $lookupKey;
    }

    public function digest(string $normalized): string
    {
        return hash_hmac('sha256', "focusa.spec152e.email.lookup.v1\0" . $normalized, $this->lookupKey);
    }

    public function encrypt(string $normalized): string
    {
        if (function_exists('sodium_crypto_secretbox')) {
            $nonce = random_bytes(SODIUM_CRYPTO_SECRETBOX_NONCEBYTES);
            $ciphertext = sodium_crypto_secretbox($normalized, $nonce, $this->encryptionKey);
            $envelope = "s1\0" . $nonce . $ciphertext;
        } else {
            // PHP builds without ext-sodium retain authenticated encryption via OpenSSL.
            $nonce = random_bytes(12);
            $tag = '';
            $ciphertext = openssl_encrypt($normalized, 'aes-256-gcm', $this->encryptionKey, OPENSSL_RAW_DATA, $nonce, $tag);
            if ($ciphertext === false) {
                throw new RuntimeException('EMAIL_IDENTITY_ENCRYPTION_FAILED');
            }
            $envelope = "g1\0" . $nonce . $tag . $ciphertext;
        }
        return rtrim(strtr(base64_encode($envelope), '+/', '-_'), '=');
    }

    public function decrypt(string $envelope): string
    {
        $decoded = base64_decode(strtr($envelope, '-_', '+/'), true);
        if ($decoded === false || strlen($decoded) < 3 || substr($decoded, 2, 1) !== "\0") {
            throw new DomainException('EMAIL_IDENTITY_DECRYPTION_FAILED');
        }
        $version = substr($decoded, 0, 2);
        if ($version === 's1' && function_exists('sodium_crypto_secretbox_open')) {
            $nonceLength = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
            $plaintext = sodium_crypto_secretbox_open(substr($decoded, 3 + $nonceLength), substr($decoded, 3, $nonceLength), $this->encryptionKey);
        } elseif ($version === 'g1') {
            $nonce = substr($decoded, 3, 12);
            $tag = substr($decoded, 15, 16);
            $plaintext = openssl_decrypt(substr($decoded, 31), 'aes-256-gcm', $this->encryptionKey, OPENSSL_RAW_DATA, $nonce, $tag);
        } else {
            $plaintext = false;
        }
        if ($plaintext === false) {
            throw new DomainException('EMAIL_IDENTITY_DECRYPTION_FAILED');
        }
        return $plaintext;
    }
}

final class FocusaSpec152eEmailIdentityRepository
{
    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEmailIdentityMigration $schema,
        private FocusaSpec152eEmailIdentitySecrets $secrets,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
    }

    public function storeVerified(string $submittedEmail, array $identity): array
    {
        if (($identity['verification_state'] ?? null) !== 'mailbox_verified') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        foreach (['verified_at', 'transactional_consent_at', 'promotional_consent_at', 'promotional_consent_revoked_at'] as $field) {
            FocusaSpec152eEmailIdentityMigration::assertTimestamp($identity[$field] ?? null, $field !== 'verified_at');
        }
        $this->assertUuid((string) ($identity['identity_uuid'] ?? ''), 'identity');
        $this->assertUuid((string) ($identity['account_uuid'] ?? ''), 'account');
        $state = (string) ($identity['identity_state'] ?? '');
        if (!in_array($state, ['primary', 'linked'], true)) {
            throw new InvalidArgumentException('verified identity state required');
        }
        $method = (string) ($identity['verification_method'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_]{1,31}$/D', $method) !== 1) {
            throw new InvalidArgumentException('verification method required');
        }
        $source = (string) ($identity['source'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_.:-]{1,63}$/D', $source) !== 1) {
            throw new InvalidArgumentException('bounded identity source required');
        }
        $evidence = $identity['migration_evidence'] ?? [];
        if (!is_array($evidence) || $evidence === []) {
            throw new InvalidArgumentException('migration evidence is required');
        }

        $normalized = FocusaSpec152eEmailNormalizer::exact($submittedEmail);
        $digest = $this->secrets->digest($normalized);
        $table = $this->schema->table('wpuiai_email_identities');
        $this->db->beginTransaction();
        try {
            $existing = $this->findStoredByDigest($digest);
            if ($existing !== null) {
                if (!hash_equals($existing['identity_uuid'], $identity['identity_uuid'])
                    || !hash_equals($existing['account_uuid'], $identity['account_uuid'])) {
                    throw new DomainException('EMAIL_IDENTITY_CONFLICT');
                }
                $this->db->commit();
                return $this->safe($existing);
            }
            $now = ($this->clock)();
            FocusaSpec152eEmailIdentityMigration::assertTimestamp($now);
            $statement = $this->db->prepare("INSERT INTO {$table} (
                identity_uuid, account_uuid, encrypted_normalized_email, email_lookup_digest,
                verified_at, verification_method, identity_state, transactional_consent_at,
                promotional_consent_at, promotional_consent_revoked_at, bounce_state, bounced_at,
                suppression_state, suppressed_at, revoked_at, source, migration_evidence, created_at, updated_at
            ) VALUES (
                :identity, :account, :encrypted, :digest, :verified, :method, :state, :transactional,
                :promotional, :promotional_revoked, 'none', NULL, 'none', NULL, NULL, :source, :evidence, :created, :updated
            )");
            $statement->execute([
                ':identity' => $identity['identity_uuid'], ':account' => $identity['account_uuid'],
                ':encrypted' => $this->secrets->encrypt($normalized), ':digest' => $digest,
                ':verified' => $identity['verified_at'], ':method' => $method, ':state' => $state,
                ':transactional' => $identity['transactional_consent_at'] ?? null,
                ':promotional' => $identity['promotional_consent_at'] ?? null,
                ':promotional_revoked' => $identity['promotional_consent_revoked_at'] ?? null,
                ':source' => $source,
                ':evidence' => FocusaSpec152eEmailIdentityMigration::encodeCanonical($evidence),
                ':created' => $now, ':updated' => $now,
            ]);
            $stored = $this->findStoredByDigest($digest);
            $this->db->commit();
            return $this->safe($stored ?? throw new RuntimeException('identity insert missing'));
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    public function findExact(string $submittedEmail): ?array
    {
        $digest = $this->secrets->digest(FocusaSpec152eEmailNormalizer::exact($submittedEmail));
        $row = $this->findStoredByDigest($digest);
        return $row === null ? null : $this->safe($row);
    }

    /**
     * Caller-owned transaction primitive: insert a verified identity for an already-resolved
     * authority account. Used by the atomic verified-account promotion service; never starts
     * its own transaction. The caller has already proven mailbox control for this email.
     */
    public function storeVerifiedInTransaction(string $submittedEmail, array $identity): array
    {
        if (($identity['verification_state'] ?? null) !== 'mailbox_verified') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        foreach (['verified_at', 'transactional_consent_at', 'promotional_consent_at', 'promotional_consent_revoked_at'] as $field) {
            FocusaSpec152eEmailIdentityMigration::assertTimestamp($identity[$field] ?? null, $field !== 'verified_at');
        }
        $this->assertUuid((string) ($identity['identity_uuid'] ?? ''), 'identity');
        $this->assertUuid((string) ($identity['account_uuid'] ?? ''), 'account');
        $state = (string) ($identity['identity_state'] ?? '');
        if (!in_array($state, ['primary', 'linked'], true)) {
            throw new InvalidArgumentException('verified identity state required');
        }
        $method = (string) ($identity['verification_method'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_]{1,31}$/D', $method) !== 1) {
            throw new InvalidArgumentException('verification method required');
        }
        $source = (string) ($identity['source'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_.:-]{1,63}$/D', $source) !== 1) {
            throw new InvalidArgumentException('bounded identity source required');
        }
        $evidence = $identity['migration_evidence'] ?? [];
        if (!is_array($evidence) || $evidence === []) {
            throw new InvalidArgumentException('migration evidence is required');
        }

        $normalized = FocusaSpec152eEmailNormalizer::exact($submittedEmail);
        $digest = $this->secrets->digest($normalized);
        $existing = $this->findStoredByDigest($digest);
        if ($existing !== null) {
            if (!hash_equals($existing['identity_uuid'], $identity['identity_uuid'])
                || !hash_equals($existing['account_uuid'], $identity['account_uuid'])) {
                throw new DomainException('EMAIL_IDENTITY_CONFLICT');
            }
            return $this->safe($existing);
        }
        $now = ($this->clock)();
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($now);
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("INSERT INTO {$table} (
            identity_uuid, account_uuid, encrypted_normalized_email, email_lookup_digest,
            verified_at, verification_method, identity_state, transactional_consent_at,
            promotional_consent_at, promotional_consent_revoked_at, bounce_state, bounced_at,
            suppression_state, suppressed_at, revoked_at, source, migration_evidence, created_at, updated_at
        ) VALUES (
            :identity, :account, :encrypted, :digest, :verified, :method, :state, :transactional,
            :promotional, :promotional_revoked, 'none', NULL, 'none', NULL, NULL, :source, :evidence, :created, :updated
        )");
        $statement->execute([
            ':identity' => $identity['identity_uuid'], ':account' => $identity['account_uuid'],
            ':encrypted' => $this->secrets->encrypt($normalized), ':digest' => $digest,
            ':verified' => $identity['verified_at'], ':method' => $method, ':state' => $state,
            ':transactional' => $identity['transactional_consent_at'] ?? null,
            ':promotional' => $identity['promotional_consent_at'] ?? null,
            ':promotional_revoked' => $identity['promotional_consent_revoked_at'] ?? null,
            ':source' => $source,
            ':evidence' => FocusaSpec152eEmailIdentityMigration::encodeCanonical($evidence),
            ':created' => $now, ':updated' => $now,
        ]);
        $stored = $this->findStoredByDigest($digest);
        return $this->safe($stored ?? throw new RuntimeException('identity insert missing'));
    }

    /**
     * Caller-owned transaction primitive: settle promotion consent without ever overwriting
     * consent that was settled earlier. Transactional consent is required at promotion and is
     * recorded separately from optional promotional consent.
     */
    public function settleConsentAtPromotionInTransaction(string $identityUuid, string $transactional, ?string $promotional, string $occurredAt): array
    {
        $this->assertUuid($identityUuid, 'identity');
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($transactional);
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($promotional, true);
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($occurredAt);
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("UPDATE {$table} SET
            transactional_consent_at = COALESCE(transactional_consent_at, :transactional),
            promotional_consent_at = COALESCE(promotional_consent_at, :promotional),
            updated_at = :occurred WHERE identity_uuid = :identity");
        $statement->execute([
            ':transactional' => $transactional,
            ':promotional' => $promotional,
            ':occurred' => $occurredAt,
            ':identity' => $identityUuid,
        ]);
        if ($statement->rowCount() !== 1) {
            throw new OutOfBoundsException('email identity not found');
        }
        return $this->findSafeByUuid($identityUuid);
    }

    /** Bounded primary check used by promotion to mark the first identity primary. */
    public function hasPrimaryForAccount(string $accountUuid): bool
    {
        $this->assertUuid($accountUuid, 'account');
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE account_uuid = :account AND identity_state = 'primary'");
        $statement->execute([':account' => $accountUuid]);
        return (int) $statement->fetchColumn() > 0;
    }

    /** Explicit authenticated-workflow seam; callers must never write its result to generic logs. */
    public function revealForAuthenticatedWorkflow(string $identityUuid): string
    {
        $this->assertUuid($identityUuid, 'identity');
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("SELECT encrypted_normalized_email FROM {$table} WHERE identity_uuid = :identity");
        $statement->execute([':identity' => $identityUuid]);
        $envelope = $statement->fetchColumn();
        if (!is_string($envelope)) {
            throw new OutOfBoundsException('email identity not found');
        }
        return $this->secrets->decrypt($envelope);
    }

    public function findByUuid(string $identityUuid): array
    {
        $this->assertUuid($identityUuid, 'identity');
        return $this->findSafeByUuid($identityUuid);
    }

    public function settleConsent(string $identityUuid, string $consentField, string $occurredAt): array
    {
        $this->assertUuid($identityUuid, 'identity');
        if (!in_array($consentField, ['transactional_consent_at', 'promotional_consent_at'], true)) {
            throw new InvalidArgumentException('bounded consent field required');
        }
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($occurredAt);
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("UPDATE {$table} SET {$consentField} = :occurred, updated_at = :occurred
            WHERE identity_uuid = :identity AND {$consentField} IS NULL");
        $statement->execute([
            ':occurred' => $occurredAt,
            ':identity' => $identityUuid,
        ]);
        if ($statement->rowCount() !== 1) {
            throw new DomainException('CONSENT_ALREADY_SETTLED');
        }
        return $this->findSafeByUuid($identityUuid);
    }

    public function revokePromotionalConsent(string $identityUuid, string $occurredAt): array
    {
        $this->assertUuid($identityUuid, 'identity');
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($occurredAt);
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("UPDATE {$table} SET
            promotional_consent_revoked_at = :occurred,
            suppression_state = CASE WHEN suppression_state = 'none' THEN 'promotional' ELSE suppression_state END,
            updated_at = :occurred
            WHERE identity_uuid = :identity");
        $statement->execute([
            ':occurred' => $occurredAt,
            ':identity' => $identityUuid,
        ]);
        if ($statement->rowCount() !== 1) {
            throw new OutOfBoundsException('email identity not found');
        }
        return $this->findSafeByUuid($identityUuid);
    }

    public function recordDeliveryState(string $identityUuid, string $bounceState, string $suppressionState, string $occurredAt): array
    {
        $this->assertUuid($identityUuid, 'identity');
        if (!in_array($bounceState, ['none', 'soft', 'hard'], true)
            || !in_array($suppressionState, ['none', 'transactional', 'promotional', 'all'], true)) {
            throw new InvalidArgumentException('bounded delivery state required');
        }
        FocusaSpec152eEmailIdentityMigration::assertTimestamp($occurredAt);
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("UPDATE {$table} SET bounce_state = :bounce,
            bounced_at = CASE WHEN :bounce = 'none' THEN NULL ELSE :occurred END,
            suppression_state = :suppression,
            suppressed_at = CASE WHEN :suppression = 'none' THEN NULL ELSE :occurred END,
            updated_at = :occurred WHERE identity_uuid = :identity");
        $statement->execute([
            ':bounce' => $bounceState, ':suppression' => $suppressionState,
            ':occurred' => $occurredAt, ':identity' => $identityUuid,
        ]);
        if ($statement->rowCount() !== 1) {
            throw new OutOfBoundsException('email identity not found');
        }
        return $this->findSafeByUuid($identityUuid);
    }

    private function findStoredByDigest(string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE email_lookup_digest = :digest");
        $statement->execute([':digest' => $digest]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findSafeByUuid(string $identityUuid): array
    {
        $table = $this->schema->table('wpuiai_email_identities');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE identity_uuid = :identity");
        $statement->execute([':identity' => $identityUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('email identity not found');
        }
        return $this->safe($row);
    }

    private function safe(array $row): array
    {
        unset($row['encrypted_normalized_email'], $row['email_lookup_digest']);
        return $row;
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
    }
}
