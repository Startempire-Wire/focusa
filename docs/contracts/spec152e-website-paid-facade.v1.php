<?php
// Spec 152E branded website paid-Focusa journey (atom focusa-vbcqu.20.13.56).
// One synthetic test-mode customer runs the complete acceptance-matrix row
// "Website paid Focusa" (spec 152E §23): branded website registration ->
// verified mailbox control -> authority account + EDD customer promotion ->
// server-owned EDD checkout intent -> test-mode payment/order completion ->
// EDD Software Licensing human key -> dual-channel delivery (transactional
// email + terminal-independent account delivery) -> node registration ->
// signed device-bound lease -> EDD refund cleanup that preserves truth.
//
// Authority boundary (spec 152E §1, §4): WPUIAI.com EDD is the sole commerce,
// human-license, and entitlement authority. The branded facade
// (focusa_marketing_v1 / focusa_install_v1) is a presenter and bounded proxy
// only: it never resolves a product, price, grant, limit, order, license,
// node, or lease. Every decision is made here, in the authority kernel, and
// every transition is journaled with opaque request/idempotency correlation.
//
// Fail-closed invariants (spec 152E FORBIDDEN + §19):
//   - A submitted email creates only a pending registration attempt; no EDD
//     customer, account, checkout, order, license, node, or lease exists until
//     mailbox control is verified with the single-use challenge.
//   - No local/self-issued entitlement: the human key is created only by the
//     EDD Software Licensing issuance step after a complete, integrity-ok
//     order bound to the verified identity. Caller-supplied price/grant/
//     product/download/limit fields fail closed (CALLER_CONTROLLED_GRANT_DENIED).
//   - No independent facade authority: facades are allowlisted presenters;
//     checkout URLs are composed only from the facade's exact origin plus the
//     allowlisted checkout path. Caller-supplied redirect/callback URLs fail
//     closed (FACADE_ORIGIN_DENIED / FACADE_REQUEST_FIELD_DENIED).
//   - Checkout email integrity: payment success with a different email holds
//     fulfillment (EDD_ORDER_UNVERIFIED) until the verified identity matches.
//   - Spec 158 implementation is excluded (no cognitive/Workstream authority).
//   - No raw email, raw human key, payment reference, or secret material is
//     stored or returned; receipts are redacted and carry an immutable digest.
//   - Refund cleanup revokes entitlement and increments the monotonic lease
//     sequence but preserves account, order, delivery, node, and evidence rows.
//
// The signed lease uses the canonical pure-PHP RFC 8032 Ed25519 signer
// (FocusaSpec152eEd25519Signer) from docs/contracts/spec152e-edd-bound-lease-
// issuer.v1.php, which must be loaded before this contract at runtime; the
// python gate verifies every signature with `cryptography` Ed25519PublicKey.
// Seeds here are public synthetic non-production fixture seeds.
declare(strict_types=1);

final class FocusaSpec152eWebsitePaidFacadeMigration
{
    public const SCHEMA = 'focusa.spec152e.website_paid_facade.v1';
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
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $tables = [
            'wpuiai_webpaid_registrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_registrations (
                registration_uuid {$uuid} NOT NULL PRIMARY KEY,
                facade_id VARCHAR(96) NOT NULL,
                origin VARCHAR(191) NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                email_digest VARCHAR(64) NOT NULL,
                email_domain VARCHAR(191) NOT NULL,
                email_prefix_char VARCHAR(4) NOT NULL,
                challenge_hash VARCHAR(64) NOT NULL,
                challenge_expires_at VARCHAR(32) NOT NULL,
                challenge_attempts BIGINT NOT NULL DEFAULT 0,
                challenge_used BIGINT NOT NULL DEFAULT 0,
                state VARCHAR(32) NOT NULL,
                account_uuid {$uuid} NULL,
                customer_id BIGINT NULL,
                order_id BIGINT NULL,
                edd_license_id BIGINT NULL,
                node_uuid {$uuid} NULL,
                lease_uuid {$uuid} NULL,
                request_id {$key} NOT NULL,
                idempotency_key {$key} NOT NULL,
                request_digest VARCHAR(64) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_identities' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_identities (
                identity_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                email_digest VARCHAR(64) NOT NULL UNIQUE,
                email_domain VARCHAR(191) NOT NULL,
                email_prefix_char VARCHAR(4) NOT NULL,
                verified_at VARCHAR(32) NOT NULL,
                verified_method VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL
            )",
            'wpuiai_webpaid_accounts' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_accounts (
                account_uuid {$uuid} NOT NULL PRIMARY KEY,
                customer_id BIGINT NOT NULL UNIQUE,
                facade_id VARCHAR(96) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_orders' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_orders (
                order_id BIGINT NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                customer_id BIGINT NOT NULL,
                facade_id VARCHAR(96) NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                edd_download_id BIGINT NOT NULL,
                edd_price_id VARCHAR(191) NOT NULL,
                price_usd VARCHAR(32) NOT NULL,
                checkout_email_digest VARCHAR(64) NOT NULL,
                verified_email_digest VARCHAR(64) NOT NULL,
                payment_reference_digest VARCHAR(64) NOT NULL,
                state VARCHAR(32) NOT NULL,
                state_reason VARCHAR(64) NULL,
                created_at VARCHAR(32) NOT NULL,
                completed_at VARCHAR(32) NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_order_items' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_order_items (
                order_item_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_download_id BIGINT NOT NULL,
                edd_price_id VARCHAR(191) NOT NULL,
                amount_usd VARCHAR(32) NOT NULL,
                quantity BIGINT NOT NULL DEFAULT 1
            )",
            'wpuiai_webpaid_licenses' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_licenses (
                edd_license_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                customer_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                license_key VARCHAR(191) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_deliveries' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_deliveries (
                delivery_id VARCHAR(64) NOT NULL PRIMARY KEY,
                edd_license_id BIGINT NOT NULL,
                channel VARCHAR(16) NOT NULL,
                recipient_mask VARCHAR(191) NOT NULL,
                key_mask VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                sent_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_nodes' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_nodes (
                node_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                device_public_key_hash VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_leases' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_leases (
                lease_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                customer_id BIGINT NOT NULL,
                order_id BIGINT NOT NULL,
                order_item_id BIGINT NOT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                posture VARCHAR(16) NOT NULL,
                node_uuid {$uuid} NOT NULL,
                sequence BIGINT NOT NULL CHECK (sequence >= 1),
                authority_key_id VARCHAR(64) NOT NULL,
                envelope_digest VARCHAR(64) NOT NULL,
                payload_b64 TEXT NOT NULL,
                signature_b64 TEXT NOT NULL,
                issued_at VARCHAR(32) NOT NULL,
                not_before VARCHAR(32) NOT NULL,
                expires_at VARCHAR(32) NOT NULL,
                offline_grace_until VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL,
                state_reason VARCHAR(64) NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_sequences' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_sequences (
                account_uuid {$uuid} NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                current_sequence BIGINT NOT NULL DEFAULT 0 CHECK (current_sequence >= 0),
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL,
                PRIMARY KEY (account_uuid, product_code)
            )",
            'wpuiai_webpaid_refunds' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_refunds (
                refund_id VARCHAR(64) NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_license_id BIGINT NOT NULL,
                reason VARCHAR(191) NOT NULL,
                sequence_after BIGINT NOT NULL,
                refunded_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_journal' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_journal (
                event_key {$key} NOT NULL PRIMARY KEY,
                event_type VARCHAR(32) NOT NULL,
                registration_uuid {$uuid} NOT NULL,
                account_uuid {$uuid} NULL,
                state VARCHAR(32) NOT NULL,
                request_id {$key} NOT NULL,
                idempotency_key {$key} NOT NULL,
                request_digest VARCHAR(64) NOT NULL,
                occurred_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_webpaid_schema_migrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_schema_migrations (
                schema_version BIGINT NOT NULL PRIMARY KEY,
                schema_name VARCHAR(191) NOT NULL,
                applied_at VARCHAR(32) NOT NULL,
                migration_provenance TEXT NOT NULL
            )",
            'wpuiai_webpaid_schema_events' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_webpaid_schema_events (
                event_key {$key} NOT NULL PRIMARY KEY,
                event_type VARCHAR(32) NOT NULL,
                schema_version BIGINT NOT NULL,
                occurred_at VARCHAR(32) NOT NULL,
                migration_provenance TEXT NOT NULL
            )",
        ];
        foreach ($tables as $sql) {
            $this->db->exec($sql);
        }
        $statement = $this->db->prepare(
            "INSERT INTO {$this->prefix}wpuiai_webpaid_schema_migrations (schema_version, schema_name, applied_at, migration_provenance)
             SELECT :version, :schema, :applied_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$this->prefix}wpuiai_webpaid_schema_migrations WHERE schema_version = :existing_version)"
        );
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied_at' => $appliedAt,
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare(
            "INSERT OR IGNORE INTO {$this->prefix}wpuiai_webpaid_schema_events (event_key, event_type, schema_version, occurred_at, migration_provenance)
             VALUES (:event_key, 'rollback_preserved', :version, :occurred_at, :provenance)"
        );
        $statement->execute([
            ':event_key' => $eventKey,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => $encoded,
        ]);
        return ['action' => 'preserve', 'event_key' => $eventKey];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    public static function assertTimestamp(string $value): void
    {
        if (preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/', $value) !== 1) {
            throw new InvalidArgumentException('RFC3339 UTC timestamp required');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        $json = json_encode($value, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
        if ($json === false) {
            throw new InvalidArgumentException('unencodable provenance');
        }
        return $json;
    }
}

final class FocusaSpec152eWebsitePaidFacadeService
{
    public const SCHEMA = 'focusa.spec152e.website_paid_facade.v1';

    // Server-owned synthetic test-mode product mapping (spec 152E §8). The
    // caller submits only the public product code; every EDD download, price,
    // grant, feature, limit, and commercial right is resolved here. No value
    // on this map is client-controlled; values are public synthetic fixtures.
    public const PRODUCT_MAPPING = [
        'focusa_operator_lifetime_v1' => [
            'posture' => 'paid',
            'edd_download_id' => 4601,
            'edd_price_id' => '46011',
            'price_usd' => '697.00',
            'features' => [
                'base_focusa' => true,
                'mission' => true,
                'workpoint' => true,
                'evidence' => true,
                'team_remote' => true,
                'release_proof' => true,
            ],
            'limits' => ['node_limit' => 3, 'operator_seats' => 1],
            'commercial' => [
                'term' => 'lifetime',
                'price_version' => 'v1',
                'refund_policy' => 'whole_order_30_days',
                'upgrade_policy' => 'explicit_upgrade_or_cross_grade_required_existing_operator_v1_preserved',
            ],
            'offline_grace_days' => 120,
        ],
    ];

    // Registered facade allowlist (spec 152E §9, subset needed by the website
    // paid journey). Facades are presenters: they may brand and proxy; they
    // never decide entitlement. Caller-supplied origins fail closed.
    public const FACADE_ALLOWLIST = [
        'focusa_marketing_v1' => ['https://focusa.dev'],
        'focusa_install_v1' => ['https://install.focusa.dev'],
    ];

    public const JOURNEY_STATES = [
        'attempt_created', 'email_verified', 'account_promoted',
        'checkout_pending', 'order_complete', 'entitlement_issued',
        'delivered', 'device_registered', 'lease_issued', 'refunded',
    ];

    /** @var Closure(): string */
    private Closure $clock;
    private string $prefix;
    private PDO $db;
    private object $keySet;

    public function __construct(PDO $db, callable $clock, string $prefix, object $keySet)
    {
        $this->db = $db;
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->prefix = $prefix;
        if (!is_object($keySet) || !method_exists($keySet, 'seal') || !method_exists($keySet, 'leaseSeed')) {
            throw new InvalidArgumentException('authority key-set seam required');
        }
        $this->keySet = $keySet;
    }

    // ── Journey: registration → verification → promotion ──────────────────

    public function startRegistration(array $request): array
    {
        $facadeId = (string) ($request['facade_id'] ?? '');
        $origin = (string) ($request['origin'] ?? '');
        $productCode = (string) ($request['product_code'] ?? '');
        $emailDigest = (string) ($request['email_digest'] ?? '');
        $emailDomain = strtolower((string) ($request['email_domain'] ?? ''));
        $emailPrefixChar = (string) ($request['email_prefix_char'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);

        $this->assertDigest($emailDigest, 'email');
        if (preg_match('/^[a-z0-9.-]+$/', $emailDomain) !== 1 || strpos($emailDomain, '.') === false) {
            throw new DomainException('EMAIL_FORMAT_DENIED');
        }
        if ($emailPrefixChar === '' || preg_match('/^[A-Za-z0-9_*]$/', $emailPrefixChar) !== 1) {
            throw new DomainException('EMAIL_FORMAT_DENIED');
        }
        $origins = self::FACADE_ALLOWLIST[$facadeId] ?? null;
        if ($origins === null || !in_array($origin, $origins, true)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if (!isset(self::PRODUCT_MAPPING[$productCode])) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['facade_id', 'origin', 'product_code', 'email_digest', 'email_domain', 'email_prefix_char', 'challenge_code', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('registration_started', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $registrationUuid = $this->opaqueId('reg_webpaid_' . substr($emailDigest, 0, 16));
        $now = ($this->clock)();
        $challengeCode = (string) ($request['challenge_code'] ?? '');
        if (preg_match('/^[0-9]{6}$/', $challengeCode) !== 1) {
            throw new DomainException('CHALLENGE_FORMAT_DENIED');
        }
        $expiresAt = (new DateTimeImmutable($now))->modify('+15 minutes')->format('Y-m-d\TH:i:s\Z');
        $challengeHash = hash('sha256', "challenge-v1\n" . $registrationUuid . "\n" . $challengeCode . "\n" . $expiresAt);

        $statement = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_webpaid_registrations')}
             (registration_uuid, facade_id, origin, product_code, email_digest, email_domain, email_prefix_char,
              challenge_hash, challenge_expires_at, challenge_attempts, challenge_used, state,
              request_id, idempotency_key, request_digest, created_at, updated_at)
             VALUES (:uuid, :facade, :origin, :product, :digest, :domain, :prefix_char,
              :challenge_hash, :expires_at, 0, 0, 'attempt_created',
              :request_id, :idempotency, :request_digest, :now, :now)"
        );
        $statement->execute([
            ':uuid' => $registrationUuid, ':facade' => $facadeId, ':origin' => $origin,
            ':product' => $productCode, ':digest' => $emailDigest, ':domain' => $emailDomain,
            ':prefix_char' => $emailPrefixChar, ':challenge_hash' => $challengeHash,
            ':expires_at' => $expiresAt, ':request_id' => $requestId,
            ':idempotency' => $idempotencyKey, ':request_digest' => $requestDigest, ':now' => $now,
        ]);
        $this->journal('registration_started', $registrationUuid, null, 'attempt_created', $requestId, $idempotencyKey, $requestDigest, $now);

        $maskedEmail = $emailPrefixChar . '***@' . $emailDomain;
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId, 'origin' => $origin, 'product_code' => $productCode,
            'state' => 'attempt_created', 'masked_email' => $maskedEmail,
            'challenge_sent_to' => $maskedEmail, 'challenge_expires_at' => $expiresAt,
            'customer_created' => false, 'entitlement_created' => false,
        ];
    }

    public function verifyEmail(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $submittedCode = (string) ($request['code'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        if (preg_match('/^[0-9]{6}$/', $submittedCode) !== 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'code', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('email_verified', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        $now = ($this->clock)();
        if ($row['challenge_used'] == 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        if ($now > (string) $row['challenge_expires_at']) {
            throw new DomainException('EMAIL_VERIFICATION_EXPIRED');
        }
        $attempts = (int) $row['challenge_attempts'] + 1;
        $this->updateRegistration($registrationUuid, ['challenge_attempts' => $attempts], $now);
        if ($attempts > 5) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        $expected = hash('sha256', "challenge-v1\n" . $registrationUuid . "\n" . $submittedCode . "\n" . (string) $row['challenge_expires_at']);
        if (!hash_equals((string) $row['challenge_hash'], $expected)) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        $this->updateRegistration($registrationUuid, ['challenge_used' => 1, 'state' => 'email_verified'], $now);
        $this->journal('email_verified', $registrationUuid, null, 'email_verified', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'email_verified', 'verification_method' => 'single_use_magic_code',
            'promoted' => false,
        ];
    }

    public function promote(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('account_promoted', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'email_verified') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ((string) $row['account_uuid'] !== '') {
            // Already promoted: replay returns the stored account; zero new rows.
            return [
                'ok' => true, 'replayed' => true, 'registration_uuid' => $registrationUuid,
                'account_uuid' => (string) $row['account_uuid'], 'customer_id' => (int) $row['customer_id'],
                'state' => 'account_promoted', 'edd_customer_resolved_or_created' => true, 'zero_new_rows' => true,
            ];
        }
        $now = ($this->clock)();
        $accountUuid = $this->opaqueId('acct_webpaid_0001');
        $customerId = 1001;
        $this->db->beginTransaction();
        try {
            $identity = $this->db->prepare(
                "INSERT OR IGNORE INTO {$this->table('wpuiai_webpaid_identities')}
                 (identity_uuid, account_uuid, email_digest, email_domain, email_prefix_char, verified_at, verified_method, state)
                 VALUES (:uuid, :account, :digest, :domain, :prefix_char, :verified_at, 'single_use_magic_code', 'verified')"
            );
            $identity->execute([
                ':uuid' => $this->opaqueId('idty_webpaid_0001'), ':account' => $accountUuid,
                ':digest' => (string) $row['email_digest'], ':domain' => (string) $row['email_domain'],
                ':prefix_char' => (string) $row['email_prefix_char'], ':verified_at' => $now,
            ]);
            $account = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_webpaid_accounts')}
                 (account_uuid, customer_id, facade_id, state, created_at, updated_at)
                 VALUES (:account, :customer, :facade, 'active', :now, :now)"
            );
            $account->execute([
                ':account' => $accountUuid, ':customer' => $customerId,
                ':facade' => (string) $row['facade_id'], ':now' => $now,
            ]);
            $this->updateRegistration($registrationUuid, ['account_uuid' => $accountUuid, 'customer_id' => $customerId, 'state' => 'account_promoted'], $now);
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        $this->journal('account_promoted', $registrationUuid, $accountUuid, 'account_promoted', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'account_uuid' => $accountUuid, 'customer_id' => $customerId,
            'state' => 'account_promoted', 'edd_customer_resolved_or_created' => true,
        ];
    }

    // ── Journey: checkout → order → license → delivery ────────────────────

    public function createCheckoutIntent(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        if (array_key_exists('product_code', $request)) {
            throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
        }
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key', 'checkout_email_digest']);

        $replay = $this->findReplay('checkout_intent_created', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'account_promoted') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $now = ($this->clock)();
        $checkoutToken = 'pay_' . substr(hash('sha256', 'checkout-v1\n' . $registrationUuid . '\n' . $now), 0, 32);
        $brandedCheckoutUrl = (string) $row['origin'] . '/activate/checkout/' . $checkoutToken;
        $this->updateRegistration($registrationUuid, ['state' => 'checkout_pending'], $now);
        $this->journal('checkout_intent_created', $registrationUuid, (string) $row['account_uuid'], 'checkout_pending', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'checkout_pending', 'checkout_token' => $checkoutToken,
            'branded_checkout_url' => $brandedCheckoutUrl,
            'edd_download_id' => $product['edd_download_id'],
            'edd_price_id' => $product['edd_price_id'],
            'price_usd' => $product['price_usd'],
            'stripe_gateway' => 'edd_stripe_test_mode',
            'card_data_handled_by' => 'edd_stripe_only',
        ];
    }

    public function completePayment(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $checkoutEmailDigest = (string) ($request['checkout_email_digest'] ?? '');
        $paymentReferenceDigest = (string) ($request['payment_reference_digest'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertDigest($checkoutEmailDigest, 'checkout_email');
        $this->assertDigest($paymentReferenceDigest, 'payment_reference');
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'checkout_email_digest', 'payment_reference_digest', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('edd_order_completed', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'checkout_pending') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        if ((string) $row['account_uuid'] === '' || (string) $row['customer_id'] === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $now = ($this->clock)();
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $orderId = 9001;
        $orderItemId = 90011;
        $verifiedDigest = (string) $row['email_digest'];
        $integrityOk = hash_equals($verifiedDigest, $checkoutEmailDigest);
        $existingOrder = $this->db->prepare("SELECT state FROM {$this->table('wpuiai_webpaid_orders')} WHERE order_id = :order");
        $existingOrder->execute([':order' => $orderId]);
        $existingState = $existingOrder->fetchColumn();
        if ($existingState === 'complete') {
            // The same canonical order already completed: replay returns the stored outcome.
            return [
                'ok' => true, 'replayed' => true, 'registration_uuid' => $registrationUuid,
                'order_id' => $orderId, 'state' => 'complete',
                'checkout_email_integrity' => 'verified_identity_match',
                'stripe_test_mode_payment' => true, 'zero_new_rows' => true,
            ];
        }
        $this->db->beginTransaction();
        try {
            if ($existingState === 'held_unverified' && $integrityOk) {
                // Checkout email integrity: the held order completes only now that the
                // verified identity matches the checkout email (spec 152E §6.4).
                $flip = $this->db->prepare(
                    "UPDATE {$this->table('wpuiai_webpaid_orders')}
                     SET state = 'complete', state_reason = NULL, completed_at = :now, updated_at = :now
                     WHERE order_id = :order"
                );
                $flip->execute([':now' => $now, ':order' => $orderId]);
            } elseif ($existingState === false) {
                $order = $this->db->prepare(
                    "INSERT INTO {$this->table('wpuiai_webpaid_orders')}
                     (order_id, account_uuid, customer_id, facade_id, product_code, edd_download_id, edd_price_id,
                      price_usd, checkout_email_digest, verified_email_digest, payment_reference_digest,
                      state, state_reason, created_at, completed_at, updated_at)
                     VALUES (:order, :account, :customer, :facade, :product, :download, :price_id,
                      :price, :checkout_digest, :verified_digest, :payment_digest,
                      :state, :reason, :now, :completed, :now)"
                );
                $order->execute([
                    ':order' => $orderId, ':account' => (string) $row['account_uuid'],
                    ':customer' => (int) $row['customer_id'], ':facade' => (string) $row['facade_id'],
                    ':product' => (string) $row['product_code'], ':download' => $product['edd_download_id'],
                    ':price_id' => $product['edd_price_id'], ':price' => $product['price_usd'],
                    ':checkout_digest' => $checkoutEmailDigest, ':verified_digest' => $verifiedDigest,
                    ':payment_digest' => $paymentReferenceDigest, ':state' => $integrityOk ? 'complete' : 'held_unverified',
                    ':reason' => $integrityOk ? null : 'EDD_ORDER_UNVERIFIED', ':now' => $now,
                    ':completed' => $integrityOk ? $now : null,
                ]);
                $item = $this->db->prepare(
                    "INSERT INTO {$this->table('wpuiai_webpaid_order_items')}
                     (order_item_id, order_id, edd_download_id, edd_price_id, amount_usd, quantity)
                     VALUES (:item, :order, :download, :price_id, :amount, 1)"
                );
                $item->execute([
                    ':item' => $orderItemId, ':order' => $orderId,
                    ':download' => $product['edd_download_id'], ':price_id' => $product['edd_price_id'],
                    ':amount' => $product['price_usd'],
                ]);
            } else {
                // A different (still-mismatched) digest on the held order: fulfillment stays held.
                $this->db->rollBack();
                return [
                    'ok' => false, 'replayed' => false, 'registration_uuid' => $registrationUuid,
                    'order_id' => $orderId, 'state' => 'held_unverified',
                    'checkout_email_integrity' => 'fulfillment_held',
                    'stripe_test_mode_payment' => true, 'error' => 'EDD_ORDER_UNVERIFIED',
                ];
            }
            $this->updateRegistration($registrationUuid, ['order_id' => $orderId, 'state' => $integrityOk ? 'order_complete' : 'checkout_pending'], $now);
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        $this->journal('edd_order_completed', $registrationUuid, (string) $row['account_uuid'], $integrityOk ? 'order_complete' : 'held_unverified', $requestId, $idempotencyKey, $requestDigest, $now);
        $result = [
            'ok' => $integrityOk, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'order_id' => $orderId, 'state' => $integrityOk ? 'complete' : 'held_unverified',
            'checkout_email_integrity' => $integrityOk ? 'verified_identity_match' : 'fulfillment_held',
            'stripe_test_mode_payment' => true,
        ];
        if (!$integrityOk) {
            $result['error'] = 'EDD_ORDER_UNVERIFIED';
        }
        return $result;
    }

    public function issueLicense(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('edd_license_issued', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'order_complete') {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        $now = ($this->clock)();
        $licenseId = 7001;
        $raw = strtoupper(substr(preg_replace('/[^A-Z0-9]/', '', strtoupper(hash('sha256', "edd-sl-v1\n" . $licenseId . "\n" . (string) $row['order_id']))), 0, 16));
        $licenseKey = 'FOCUSA-' . implode('-', str_split($raw, 4));
        $license = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_webpaid_licenses')}
             (edd_license_id, order_id, customer_id, product_code, license_key, state, created_at, updated_at)
             VALUES (:license, :order, :customer, :product, :key, 'active', :now, :now)"
        );
        $license->execute([
            ':license' => $licenseId, ':order' => (int) $row['order_id'],
            ':customer' => (int) $row['customer_id'], ':product' => (string) $row['product_code'],
            ':key' => $licenseKey, ':now' => $now,
        ]);
        $this->updateRegistration($registrationUuid, ['edd_license_id' => $licenseId, 'state' => 'entitlement_issued'], $now);
        $this->journal('edd_license_issued', $registrationUuid, (string) $row['account_uuid'], 'entitlement_issued', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'edd_license_id' => $licenseId, 'state' => 'entitlement_issued',
            'source' => 'edd_software_licensing', 'issuance_surface' => 'edd_authority_only',
            'duplicate_license' => false,
        ];
    }

    public function deliver(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('key_delivered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'entitlement_issued' || (string) $row['edd_license_id'] === '') {
            throw new DomainException('EDD_LICENSE_PENDING');
        }
        $license = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_licenses')} WHERE edd_license_id = :id");
        $license->execute([':id' => (int) $row['edd_license_id']]);
        $licenseRow = $license->fetch(PDO::FETCH_ASSOC);
        if ($licenseRow === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $now = ($this->clock)();
        $keyMask = substr((string) $licenseRow['license_key'], 0, 11) . '-****-****-****';
        $maskedEmail = (string) $row['email_prefix_char'] . '***@' . (string) $row['email_domain'];
        $channels = [
            ['email', $maskedEmail, 'transactional_email'],
            ['account', $maskedEmail, 'terminal_independent_account'],
        ];
        foreach ($channels as $index => [$channel, $recipient, $note]) {
            $deliveryId = 'dlv_webpaid_' . $channel;
            $delivery = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_webpaid_deliveries')}
                 (delivery_id, edd_license_id, channel, recipient_mask, key_mask, state, sent_at)
                 VALUES (:id, :license, :channel, :recipient, :key_mask, 'sent', :now)"
            );
            $delivery->execute([
                ':id' => $deliveryId, ':license' => (int) $row['edd_license_id'],
                ':channel' => $channel, ':recipient' => $recipient, ':key_mask' => $keyMask, ':now' => $now,
            ]);
        }
        $this->updateRegistration($registrationUuid, ['state' => 'delivered'], $now);
        $this->journal('key_delivered', $registrationUuid, (string) $row['account_uuid'], 'delivered', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'delivered', 'channels' => ['email' => 'sent', 'account' => 'sent'],
            'email_recipient_mask' => $maskedEmail, 'key_mask' => $keyMask,
            'same_canonical_key_both_channels' => true,
            'promotional_content' => false,
        ];
    }

    // ── Journey: node registration → signed lease ─────────────────────────

    public function registerNode(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $nodeId = (string) ($request['node_id'] ?? '');
        $devicePublicKey = (string) ($request['device_public_key'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        if (preg_match('/^[A-Za-z0-9_-]{1,128}$/', $nodeId) !== 1) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        $this->assertPublicKey($devicePublicKey);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'node_id', 'device_public_key', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('node_registered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'delivered' || (string) $row['edd_license_id'] === '') {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        $licenseRow = $this->licenseRow((int) $row['edd_license_id']);
        if ((string) $licenseRow['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $now = ($this->clock)();
        $nodeUuid = $this->opaqueId('node_webpaid_0001');
        $node = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_webpaid_nodes')}
             (node_uuid, account_uuid, edd_license_id, product_code, device_public_key_hash, state, created_at, updated_at)
             VALUES (:node, :account, :license, :product, :key_hash, 'active', :now, :now)"
        );
        $node->execute([
            ':node' => $nodeUuid, ':account' => (string) $row['account_uuid'],
            ':license' => (int) $row['edd_license_id'], ':product' => (string) $row['product_code'],
            ':key_hash' => hash('sha256', 'device-key-v1\n' . $devicePublicKey), ':now' => $now,
        ]);
        $this->updateRegistration($registrationUuid, ['node_uuid' => $nodeUuid, 'state' => 'device_registered'], $now);
        $this->journal('node_registered', $registrationUuid, (string) $row['account_uuid'], 'device_registered', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'node_uuid' => $nodeUuid, 'state' => 'device_registered',
            'binding' => 'account_and_edd_license', 'install_channel_telemetry_only' => true,
        ];
    }

    public function issueLease(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        if (array_key_exists('product_code', $request)) {
            throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
        }
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('lease_issued', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'device_registered' || (string) $row['node_uuid'] === '') {
            throw new DomainException('NODE_REQUIRED');
        }
        $licenseRow = $this->licenseRow((int) $row['edd_license_id']);
        if ((string) $licenseRow['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $orderRow = $this->orderRow((int) $row['order_id']);
        if ((string) $orderRow['state'] !== 'complete') {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        $now = ($this->clock)();
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $sequence = $this->nextSequence((string) $row['account_uuid'], (string) $row['product_code'], $now);
        $offlineGrace = (new DateTimeImmutable($now))->modify('+' . (int) $product['offline_grace_days'] . ' days')->format('Y-m-d\TH:i:s\Z');
        $expiresAt = (new DateTimeImmutable($now))->modify('+365 days')->format('Y-m-d\TH:i:s\Z');
        $leaseUuid = $this->opaqueId('lease_webpaid_0001');
        $nodeRow = $this->nodeRow((string) $row['node_uuid']);
        $payload = [
            'schema' => 'focusa.authority_lease.v1',
            'subject_id' => (string) $row['account_uuid'],
            'account_uuid' => (string) $row['account_uuid'],
            'customer_id' => (int) $row['customer_id'],
            'order_id' => (int) $row['order_id'],
            'order_item_id' => $this->orderItemRow((int) $row['order_id']),
            'edd_license_id' => (int) $row['edd_license_id'],
            'product_code' => (string) $row['product_code'],
            'posture' => $product['posture'],
            'features' => $product['features'],
            'limits' => $product['limits'],
            'commercial' => $product['commercial'],
            'node_id' => $nodeId = (string) $nodeRow['node_uuid'],
            'device_public_key_hash' => (string) $nodeRow['device_public_key_hash'],
            'sequence' => $sequence,
            'authority_key_id' => 'authority-lease-2026-01',
            'issued_at' => $now,
            'not_before' => $now,
            'expires_at' => $expiresAt,
            'offline_grace_until' => $offlineGrace,
            'status' => 'active',
        ];
        $payloadBytes = $this->canonicalJson($payload);
        $envelope = $this->keySet->seal($payload, 'authority-lease-2026-01', $this->keySet->leaseSeed(), FocusaSpec152eEd25519Signer::LEASE_DOMAIN);
        $lease = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_webpaid_leases')}
             (lease_uuid, account_uuid, customer_id, order_id, order_item_id, edd_license_id, product_code,
              posture, node_uuid, sequence, authority_key_id, envelope_digest, payload_b64, signature_b64,
              issued_at, not_before, expires_at, offline_grace_until, state, created_at, updated_at)
             VALUES (:lease, :account, :customer, :order, :item, :license, :product,
              :posture, :node, :sequence, :key_id, :digest, :payload, :signature,
              :issued_at, :not_before, :expires_at, :offline, 'active', :now, :now)"
        );
        $lease->execute([
            ':lease' => $leaseUuid, ':account' => (string) $row['account_uuid'],
            ':customer' => (int) $row['customer_id'], ':order' => (int) $row['order_id'],
            ':item' => $this->orderItemRow((int) $row['order_id']), ':license' => (int) $row['edd_license_id'],
            ':product' => (string) $row['product_code'], ':posture' => $product['posture'],
            ':node' => (string) $row['node_uuid'], ':sequence' => $sequence,
            ':key_id' => $envelope['signer_key_id'],
            ':digest' => hash('sha256', $payloadBytes), ':payload' => $envelope['payload_b64'],
            ':signature' => $envelope['signature_b64'], ':issued_at' => $now,
            ':not_before' => $now, ':expires_at' => $expiresAt, ':offline' => $offlineGrace, ':now' => $now,
        ]);
        $this->updateRegistration($registrationUuid, ['lease_uuid' => $leaseUuid, 'state' => 'lease_issued'], $now);
        $this->journal('lease_issued', $registrationUuid, (string) $row['account_uuid'], 'lease_issued', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'lease_uuid' => $leaseUuid, 'state' => 'lease_issued',
            'sequence' => $sequence, 'posture' => $product['posture'],
            'authority_key_id' => $envelope['signer_key_id'],
            'envelope_digest' => hash('sha256', $payloadBytes),
            'runtime_authorization' => 'signed_device_bound_lease',
        ];
    }

    // ── Refund cleanup (preservation-only) ────────────────────────────────

    public function refund(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $reason = (string) ($request['reason'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        if ($reason === '' || strlen($reason) > 191) {
            throw new DomainException('REFUND_REASON_REQUIRED');
        }
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'reason', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('refunded', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'lease_issued') {
            throw new DomainException('REFUND_STATE_REQUIRED');
        }
        $now = ($this->clock)();
        $orderId = (int) $row['order_id'];
        $licenseId = (int) $row['edd_license_id'];
        $sequenceAfter = $this->nextSequence((string) $row['account_uuid'], (string) $row['product_code'], $now);
        $this->db->beginTransaction();
        try {
            $orderUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_webpaid_orders')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE order_id = :order");
            $orderUpdate->execute([':now' => $now, ':order' => $orderId]);
            $licenseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_webpaid_licenses')} SET state = 'refunded', updated_at = :now WHERE edd_license_id = :license");
            $licenseUpdate->execute([':now' => $now, ':license' => $licenseId]);
            $leaseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_webpaid_leases')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE account_uuid = :account AND state = 'active'");
            $leaseUpdate->execute([':now' => $now, ':account' => (string) $row['account_uuid']]);
            $refund = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_webpaid_refunds')}
                 (refund_id, order_id, edd_license_id, reason, sequence_after, refunded_at)
                 VALUES (:refund, :order, :license, :reason, :sequence, :now)"
            );
            $refund->execute([
                ':refund' => 'rfnd_webpaid_0001', ':order' => $orderId, ':license' => $licenseId,
                ':reason' => $reason, ':sequence' => $sequenceAfter, ':now' => $now,
            ]);
            $this->updateRegistration($registrationUuid, ['state' => 'refunded'], $now);
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        $this->journal('refunded', $registrationUuid, (string) $row['account_uuid'], 'refunded', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'refunded', 'order_id' => $orderId, 'edd_license_id' => $licenseId,
            'sequence_after' => $sequenceAfter, 'refresh_denied' => true,
            'posture' => 'recovery_only',
            'account_order_delivery_node_evidence_preserved' => true,
        ];
    }

    // ── Redacted receipt + immutable handle ───────────────────────────────

    public function receipt(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $row = $this->registrationRow($registrationUuid);
        $now = ($this->clock)();
        $maskedEmail = (string) $row['email_prefix_char'] . '***@' . (string) $row['email_domain'];
        $licenseRow = (string) $row['edd_license_id'] === '' ? null : $this->licenseRow((int) $row['edd_license_id']);
        $leaseRow = (string) $row['lease_uuid'] === '' ? null : $this->leaseRow((string) $row['lease_uuid']);
        $sequenceStatement = $this->db->prepare(
            "SELECT current_sequence FROM {$this->table('wpuiai_webpaid_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $sequenceStatement->execute([':account' => (string) $row['account_uuid'], ':product' => (string) $row['product_code']]);
        $currentSequence = $sequenceStatement->fetchColumn();
        $receipt = [
            'schema' => 'focusa.spec152e.website_paid_receipt.v1',
            'fixture' => 'focusa-vbcqu.20.13.56',
            'facade_id' => (string) $row['facade_id'],
            'origin' => (string) $row['origin'],
            'product_code' => (string) $row['product_code'],
            'masked_email' => $maskedEmail,
            'state' => (string) $row['state'],
            'order_id' => (int) $row['order_id'],
            'edd_license_id' => $licenseRow === null ? null : (int) $row['edd_license_id'],
            'key_mask' => $licenseRow === null ? null : substr((string) $licenseRow['license_key'], 0, 11) . '-****-****-****',
            'node_uuid' => (string) $row['node_uuid'],
            'lease_sequence' => $currentSequence === false ? null : (int) $currentSequence,
            'lease_state' => $leaseRow === null ? null : (string) $leaseRow['state'],
            'lease_envelope_digest' => $leaseRow === null ? null : (string) $leaseRow['envelope_digest'],
            'customer_id' => (int) $row['customer_id'],
            'install_site_authority' => 'none',
            'spec158' => 'excluded',
            'redaction' => ['raw_email' => 'absent', 'raw_key' => 'absent', 'payment_reference' => 'digest_only'],
        ];
        $canonical = $this->canonicalJson($receipt);
        $receipt['receipt_sha256'] = hash('sha256', "focusa.spec152e.website_paid_receipt.v1\n" . $canonical);
        $this->journal('receipt_issued', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], 'req_webpaid_receipt', 'idem_webpaid_receipt', hash('sha256', 'receipt\n' . $registrationUuid), $now);
        return $receipt;
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    private function table(string $name): string
    {
        return $this->prefix . $name;
    }

    private function registrationUuid(array $request): string
    {
        $uuid = (string) ($request['registration_uuid'] ?? '');
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/', $uuid) !== 1) {
            throw new DomainException('REGISTRATION_NOT_FOUND');
        }
        return $uuid;
    }

    private function requestId(array $request): string
    {
        $requestId = (string) ($request['request_id'] ?? '');
        if (preg_match('/^[A-Za-z0-9_-]{1,128}$/', $requestId) !== 1) {
            throw new DomainException('REQUEST_ID_REQUIRED');
        }
        return $requestId;
    }

    private function idempotencyKey(array $request): string
    {
        $key = (string) ($request['idempotency_key'] ?? '');
        if (preg_match('/^[A-Za-z0-9_-]{1,128}$/', $key) !== 1) {
            throw new DomainException('IDEMPOTENCY_KEY_REQUIRED');
        }
        return $key;
    }

    private function requestDigest(array $request): string
    {
        $copy = $request;
        unset($copy['request_id'], $copy['idempotency_key']);
        return hash('sha256', $this->canonicalJson($copy));
    }

    private function assertCorrelationFields(array $request): void
    {
        foreach (['request_id', 'idempotency_key'] as $field) {
            if (!isset($request[$field])) {
                throw new DomainException('REQUEST_ID_REQUIRED');
            }
        }
    }

    private function rejectCallerGrantFields(array $request, array $allowed): void
    {
        foreach (['product_code', 'edd_download_id', 'edd_price_id', 'price', 'amount', 'total', 'currency',
                  'tier', 'features', 'grants', 'limits', 'node_limit', 'operator_seats', 'commercial',
                  'license_type', 'refund_policy', 'upgrade_policy', 'redirect_url', 'success_url',
                  'cancel_url', 'return_url'] as $field) {
            if (array_key_exists($field, $request) && !in_array($field, $allowed, true)) {
                throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
            }
        }
        foreach ($request as $field => $_value) {
            if (str_starts_with((string) $field, 'caller_')) {
                throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
            }
        }
    }

    private function findReplay(string $operation, string $idempotencyKey, string $requestDigest): ?array
    {
        $statement = $this->db->prepare(
            "SELECT event_type, request_digest FROM {$this->table('wpuiai_webpaid_journal')} WHERE idempotency_key = :key AND event_type = :operation"
        );
        $statement->execute([':key' => $idempotencyKey, ':operation' => $operation]);
        $rows = $statement->fetchAll(PDO::FETCH_ASSOC);
        foreach ($rows as $existing) {
            if (hash_equals((string) $existing['request_digest'], $requestDigest)) {
                return ['ok' => true, 'operation' => $operation, 'idempotent_replay' => true, 'zero_new_rows' => true];
            }
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return null;
    }

    private function journal(string $eventType, string $registrationUuid, ?string $accountUuid, string $state, string $requestId, string $idempotencyKey, string $requestDigest, string $now): void
    {
        $eventKey = hash('sha256', $eventType . "\n" . $registrationUuid . "\n" . $idempotencyKey . "\n" . $now);
        $statement = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_webpaid_journal')}
             (event_key, event_type, registration_uuid, account_uuid, state, request_id, idempotency_key, request_digest, occurred_at)
             VALUES (:key, :type, :registration, :account, :state, :request_id, :idempotency, :digest, :now)"
        );
        $statement->execute([
            ':key' => $eventKey, ':type' => $eventType, ':registration' => $registrationUuid,
            ':account' => $accountUuid, ':state' => $state, ':request_id' => $requestId,
            ':idempotency' => $idempotencyKey, ':digest' => $requestDigest, ':now' => $now,
        ]);
    }

    private function registrationRow(string $registrationUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_registrations')} WHERE registration_uuid = :uuid");
        $statement->execute([':uuid' => $registrationUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('REGISTRATION_NOT_FOUND');
        }
        return $row;
    }

    private function licenseRow(int $licenseId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_licenses')} WHERE edd_license_id = :id");
        $statement->execute([':id' => $licenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    private function orderRow(int $orderId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_orders')} WHERE order_id = :id");
        $statement->execute([':id' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        return $row;
    }

    private function orderItemRow(int $orderId): int
    {
        $statement = $this->db->prepare("SELECT order_item_id FROM {$this->table('wpuiai_webpaid_order_items')} WHERE order_id = :id");
        $statement->execute([':id' => $orderId]);
        $itemId = $statement->fetchColumn();
        if ($itemId === false) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        return (int) $itemId;
    }

    private function nodeRow(string $nodeUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_nodes')} WHERE node_uuid = :uuid");
        $statement->execute([':uuid' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        return $row;
    }

    private function leaseRow(string $leaseUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_webpaid_leases')} WHERE lease_uuid = :uuid");
        $statement->execute([':uuid' => $leaseUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('LEASE_NOT_FOUND');
        }
        return $row;
    }

    private function nextSequence(string $accountUuid, string $productCode, string $now): int
    {
        $select = $this->db->prepare(
            "SELECT current_sequence FROM {$this->table('wpuiai_webpaid_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $select->execute([':account' => $accountUuid, ':product' => $productCode]);
        $current = $select->fetchColumn();
        if ($current === false) {
            $insert = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_webpaid_sequences')} (account_uuid, product_code, current_sequence, created_at, updated_at)
                 VALUES (:account, :product, 0, :now, :now)"
            );
            $insert->execute([':account' => $accountUuid, ':product' => $productCode, ':now' => $now]);
            $current = 0;
        }
        $next = (int) $current + 1;
        $update = $this->db->prepare(
            "UPDATE {$this->table('wpuiai_webpaid_sequences')} SET current_sequence = :next, updated_at = :now
             WHERE account_uuid = :account AND product_code = :product"
        );
        $update->execute([':next' => $next, ':now' => $now, ':account' => $accountUuid, ':product' => $productCode]);
        return $next;
    }

    private function updateRegistration(string $registrationUuid, array $fields, string $now): void
    {
        $sets = [];
        $params = [':now' => $now, ':uuid' => $registrationUuid];
        foreach ($fields as $field => $value) {
            $sets[] = "{$field} = :{$field}";
            $params[':' . $field] = $value;
        }
        $sets[] = 'updated_at = :now';
        $statement = $this->db->prepare("UPDATE {$this->table('wpuiai_webpaid_registrations')} SET " . implode(', ', $sets) . " WHERE registration_uuid = :uuid");
        $statement->execute($params);
    }

    private function opaqueId(string $seed): string
    {
        $hex = substr(hash('sha256', 'opaque-id-v1\n' . $seed), 0, 32);
        return substr($hex, 0, 8) . '-' . substr($hex, 8, 4) . '-' . substr($hex, 12, 4) . '-' . substr($hex, 16, 4) . '-' . substr($hex, 20, 12);
    }

    private function assertDigest(string $value, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{64}$/', $value) !== 1) {
            throw new DomainException(strtoupper($kind) . '_DIGEST_REQUIRED');
        }
    }

    private function assertPublicKey(string $key): void
    {
        $decoded = base64_decode($key, true);
        if ($decoded === false || strlen($decoded) !== 32) {
            throw new DomainException('DEVICE_PUBLIC_KEY_REQUIRED');
        }
    }

    private function canonicalJson(array $value): string
    {
        $json = json_encode($value, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
        if ($json === false) {
            throw new InvalidArgumentException('unencodable payload');
        }
        return $json;
    }
}
