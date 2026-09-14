<?php
// Spec 152E terminal + agent paid activation journey (atom focusa-vbcqu.20.13.57).
// Two synthetic test-mode sessions settle the same semantic paid flow (spec 152E
// §14, §23 acceptance-matrix rows "Terminal paid Focusa" and "Agent paid Focusa"):
// verified email -> facade checkout link -> bounded poll -> EDD human key ->
// dual key delivery (transactional email + one-time device-encrypted terminal
// envelope) -> protected credential store with explicit customer-controlled key
// reveal -> node registration -> signed device-bound lease -> EDD refund cleanup
// that revokes entitlement, increments the monotonic sequence, and preserves
// account/order/delivery/node/evidence truth.
//
// Presenters (spec 152E §14):
//   - cli_terminal (install_channel=terminal): the interactive terminal. Its
//     poll response may carry the one-time device-bound envelope exactly once
//     (device channel); it refuses agent pause/resume/status steps.
//   - agent_json (install_channel=agent): the machine-readable agent presenter.
//     It returns the focusa.agent_activation_envelope.v1 transcript: typed
//     human-action states, masked email, safe checkout link, bounded poll
//     (poll_count/max_polls), and never an email, verification code, consent,
//     payment confirmation, license, credential, or one-time envelope. The
//     device receives the sealed envelope out-of-band (openEnvelope seam) and
//     the human reveals the key once under explicit opt-in + confirmation.
//
// Authority boundary (spec 152E §1, §4): WPUIAI.com EDD is the sole commerce,
// human-license, and entitlement authority. Facades (focusa_install_v1 /
// focusa_marketing_v1) are allowlisted presenters: they brand and compose
// checkout URLs from their exact allowlisted origin + path; they never resolve
// a product, price, grant, limit, order, license, node, or lease.
//
// Fail-closed invariants (spec 152E FORBIDDEN + §19 + §14.2):
//   - A submitted email creates only a pending registration attempt; no EDD
//     customer, account, checkout, order, license, node, or lease exists until
//     mailbox control is verified with the single-use, attempt-bounded
//     challenge. No unverified-email promotion.
//   - No local/self-issued entitlement: the human key is created only by the
//     EDD Software Licensing issuance step after a complete, integrity-ok
//     order bound to the verified identity. Payment success alone never
//     verifies or issues.
//   - No client-controlled EDD price/grants: caller-supplied product/price/
//     grant/limit/redirect fields fail closed (CALLER_CONTROLLED_GRANT_DENIED).
//   - Checkout email integrity: payment with a different email holds
//     fulfillment (EDD_ORDER_UNVERIFIED) until the verified identity matches.
//   - Bounded poll: max_polls (40) per session; exhaustion cancels to
//     recovery_only (fail closed, never regrants). Poll credentials are
//     registration-scoped, expiring, and stored as hashes only; they never
//     appear in agent snapshots.
//   - Pause/resume: agent_json sessions only. Terminal registrations refuse
//     resume steps. Terminal states refuse pause/resume. Resume requires the
//     re-supplied protected poll credential.
//   - Explicit key reveal: full keys are masked by default everywhere; the
//     one-time reveal requires BOTH customer opt-in (reveal_key) AND explicit
//     confirmation (reveal_confirmation), stays within the envelope lifetime,
//     is one-time (replay denied), and refuses once the registration settles.
//   - The sealed one-time envelope is bound to the registration device key and
//     never appears in an agent transcript.
//   - Spec 158 implementation is excluded. No raw email, raw human key,
//     payment reference, or secret material is stored (beyond the protected
//     credential store) or returned; receipts are redacted and carry an
//     immutable sha256 handle.
//   - Refund cleanup preserves account, order, delivery, node, and evidence
//     rows; rollback is preservation-only.
//
// The one-time terminal envelope uses the canonical X25519 + HKDF-SHA256 +
// AES-256-GCM crypto (FocusaSpec152eTerminalEnvelopeCrypto / ...DeliveryEnvelope)
// from docs/contracts/spec152e-terminal-delivery-envelope.v1.php and the signed
// lease uses the canonical pure-PHP RFC 8032 Ed25519 signer + key-set seam
// (FocusaSpec152eEd25519Signer / FocusaSpec152eAuthorityKeySetSeam) from
// docs/contracts/spec152e-edd-bound-lease-issuer.v1.php; both must be loaded
// before this contract at runtime, and the python gate re-verifies every
// signature and decrypts every envelope with `cryptography`. Envelope crypto
// test seams (fixed ephemeral private key + nonce) are public synthetic
// fixtures so harness output is byte-identical across runs.
declare(strict_types=1);

/** Presenter-aware state machine helpers (spec 152E §5, §14.2). */
final class FocusaSpec152eTerminalAgentPaidState
{
    public const PRESENTERS = ['cli_terminal', 'agent_json'];
    public const CHANNELS = ['terminal', 'agent'];

    public const ATTEMPT_CREATED = 'attempt_created';
    public const EMAIL_VERIFIED = 'email_verified';
    public const ACCOUNT_PROMOTED = 'account_promoted';
    public const CHECKOUT_PENDING = 'checkout_pending';
    public const ORDER_COMPLETE = 'order_complete';
    public const ENTITLEMENT_ISSUED = 'entitlement_issued';
    public const LICENSE_DELIVERY_READY = 'license_delivery_ready';
    public const TERMINAL_DELIVERED = 'terminal_delivered';
    public const DEVICE_REGISTERED = 'device_registered';
    public const LEASE_ISSUED = 'lease_issued';
    public const REFUNDED = 'refunded';
    public const RECOVERY_ONLY = 'recovery_only';
    public const DENIED = 'denied';

    public const TERMINAL_STATES = [
        self::REFUNDED,
        self::RECOVERY_ONLY,
        self::DENIED,
    ];

    public static function isTerminalState(string $state): bool
    {
        return in_array($state, self::TERMINAL_STATES, true);
    }

    /**
     * Map a registration state to the agent presenter state, human action,
     * and next action (spec 152E §14.2). Unknown states fail closed to denied.
     */
    public static function presenter(array $row): array
    {
        switch ((string) $row['state']) {
            case self::ATTEMPT_CREATED:
                return ['email_verification_pending', 'enter_verification_code', 'enter_code', false];
            case self::EMAIL_VERIFIED:
                return ['selection_required', 'select_offer', 'select_offer', false];
            case self::ACCOUNT_PROMOTED:
                return ['checkout_required', 'open_checkout_url', 'open_checkout_url', false];
            case self::CHECKOUT_PENDING:
                return ['payment_pending', 'complete_payment_then_poll', 'poll_until_payment_complete', false];
            case self::ORDER_COMPLETE:
            case self::ENTITLEMENT_ISSUED:
            case self::LICENSE_DELIVERY_READY:
            case self::TERMINAL_DELIVERED:
                return ['license_delivery_ready', 'reveal_or_accept_license', 'reveal_or_accept_license', false];
            case self::DEVICE_REGISTERED:
            case self::LEASE_ISSUED:
                return ['activated', '', 'none', true];
            case self::REFUNDED:
            case self::RECOVERY_ONLY:
                return ['recovery_only', '', 'none', true];
            case self::DENIED:
                return ['denied', '', 'none', true];
            default:
                return ['denied', '', 'none', true];
        }
    }
}

final class FocusaSpec152eTerminalAgentPaidMigration
{
    public const SCHEMA = 'focusa.spec152e.terminal_agent_paid_activation.v1';
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
            'wpuiai_ta_registrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_registrations (
                registration_uuid {$uuid} NOT NULL PRIMARY KEY,
                presenter VARCHAR(32) NOT NULL,
                install_channel VARCHAR(32) NOT NULL,
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
                device_public_key TEXT NOT NULL,
                device_public_key_hash VARCHAR(64) NOT NULL,
                poll_credential_hash VARCHAR(64) NOT NULL,
                poll_credential_expires_at VARCHAR(32) NOT NULL,
                poll_count BIGINT NOT NULL DEFAULT 0 CHECK (poll_count >= 0),
                max_polls BIGINT NOT NULL DEFAULT 40 CHECK (max_polls >= 1),
                is_paused BIGINT NOT NULL DEFAULT 0 CHECK (is_paused IN (0, 1)),
                state VARCHAR(32) NOT NULL,
                checkout_token VARCHAR(191) NULL,
                account_uuid {$uuid} NULL,
                customer_id BIGINT NULL,
                order_id BIGINT NULL,
                edd_license_id BIGINT NULL,
                node_uuid {$uuid} NULL,
                lease_uuid {$uuid} NULL,
                envelope_id VARCHAR(64) NULL,
                terminal_delivery_status VARCHAR(16) NOT NULL DEFAULT 'none',
                email_delivery_status VARCHAR(16) NOT NULL DEFAULT 'none',
                request_id {$key} NOT NULL,
                idempotency_key {$key} NOT NULL,
                request_digest VARCHAR(64) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                expires_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_identities' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_identities (
                identity_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                email_digest VARCHAR(64) NOT NULL UNIQUE,
                email_domain VARCHAR(191) NOT NULL,
                email_prefix_char VARCHAR(4) NOT NULL,
                verified_at VARCHAR(32) NOT NULL,
                verified_method VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL
            )",
            'wpuiai_ta_accounts' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_accounts (
                account_uuid {$uuid} NOT NULL PRIMARY KEY,
                customer_id BIGINT NOT NULL UNIQUE,
                facade_id VARCHAR(96) NOT NULL,
                presenter VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_orders' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_orders (
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
            'wpuiai_ta_order_items' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_order_items (
                order_item_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_download_id BIGINT NOT NULL,
                edd_price_id VARCHAR(191) NOT NULL,
                amount_usd VARCHAR(32) NOT NULL,
                quantity BIGINT NOT NULL DEFAULT 1
            )",
            'wpuiai_ta_licenses' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_licenses (
                edd_license_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                customer_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                license_key VARCHAR(191) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_deliveries' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_deliveries (
                delivery_id VARCHAR(64) NOT NULL PRIMARY KEY,
                edd_license_id BIGINT NOT NULL,
                channel VARCHAR(16) NOT NULL,
                recipient_mask VARCHAR(191) NOT NULL,
                key_mask VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                sent_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_envelopes' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_envelopes (
                envelope_id VARCHAR(64) NOT NULL PRIMARY KEY,
                registration_uuid {$uuid} NOT NULL,
                account_uuid {$uuid} NULL,
                edd_customer_id BIGINT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                license_key_digest VARCHAR(64) NOT NULL,
                license_key_mask VARCHAR(64) NOT NULL,
                device_public_key TEXT NOT NULL,
                envelope_payload TEXT NOT NULL,
                delivery_status VARCHAR(16) NOT NULL CHECK (delivery_status IN ('issued', 'delivered', 'consumed', 'expired', 'superseded')),
                consumed_at VARCHAR(32) NULL,
                issued_at VARCHAR(32) NOT NULL,
                expires_at VARCHAR(32) NOT NULL,
                request_id {$key} NOT NULL,
                idempotency_key {$key} NOT NULL,
                request_digest VARCHAR(64) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                retention_until VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_credential_stores' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_credential_stores (
                handle VARCHAR(64) NOT NULL PRIMARY KEY,
                registration_uuid {$uuid} NOT NULL,
                key_digest VARCHAR(64) NOT NULL,
                key_mask VARCHAR(64) NOT NULL,
                claims_json TEXT NOT NULL,
                consumed BIGINT NOT NULL DEFAULT 0 CHECK (consumed IN (0, 1)),
                consumed_at VARCHAR(32) NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_nodes' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_nodes (
                node_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                device_public_key_hash VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_leases' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_leases (
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
            'wpuiai_ta_sequences' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_sequences (
                account_uuid {$uuid} NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                current_sequence BIGINT NOT NULL DEFAULT 0 CHECK (current_sequence >= 0),
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL,
                PRIMARY KEY (account_uuid, product_code)
            )",
            'wpuiai_ta_refunds' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_refunds (
                refund_id VARCHAR(64) NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_license_id BIGINT NOT NULL,
                reason VARCHAR(191) NOT NULL,
                sequence_after BIGINT NOT NULL,
                refunded_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ta_journal' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_journal (
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
            'wpuiai_ta_schema_migrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_schema_migrations (
                schema_version BIGINT NOT NULL PRIMARY KEY,
                schema_name VARCHAR(191) NOT NULL,
                applied_at VARCHAR(32) NOT NULL,
                migration_provenance TEXT NOT NULL
            )",
            'wpuiai_ta_schema_events' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ta_schema_events (
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
            "INSERT INTO {$this->prefix}wpuiai_ta_schema_migrations (schema_version, schema_name, applied_at, migration_provenance)
             SELECT :version, :schema, :applied_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$this->prefix}wpuiai_ta_schema_migrations WHERE schema_version = :existing_version)"
        );
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied_at' => $appliedAt,
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: no Spec 152E terminal/agent row is deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare(
            "INSERT OR IGNORE INTO {$this->prefix}wpuiai_ta_schema_events (event_key, event_type, schema_version, occurred_at, migration_provenance)
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

final class FocusaSpec152eTerminalAgentPaidService
{
    public const SCHEMA = 'focusa.spec152e.terminal_agent_paid_activation.v1';
    public const AGENT_ENVELOPE_SCHEMA = 'focusa.agent_activation_envelope.v1';
    public const TERMINAL_RESPONSE_SCHEMA = 'focusa.activation.response.v1';
    public const RECEIPT_SCHEMA = 'focusa.spec152e.terminal_agent_receipt.v1';
    public const ENVELOPE_TEST_EPHEMERAL_PRIVATE_HEX = '2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e';
    public const ENVELOPE_TEST_NONCE_HEX = '000102030405060708090a0b';
    public const ENVELOPE_TTL_SECONDS = 1800;
    public const POLL_CREDENTIAL_TTL_SECONDS = 1800;
    public const MAX_POLLS = 40;
    public const VERIFICATION_TTL_MINUTES = 15;
    public const VERIFICATION_MAX_ATTEMPTS = 5;
    public const RETRY_AFTER_SECONDS = 5;

    // Server-owned synthetic test-mode product mapping (spec 152E §8). The
    // caller submits only the public product code; every EDD download, price,
    // grant, feature, limit, and commercial right is resolved here.
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

    // Registered facade allowlist (spec 152E §9). Facades are presenters: they
    // may brand and compose checkout URLs from their exact origin + path; they
    // never decide entitlement. Caller-supplied origins/redirects fail closed.
    public const FACADE_ALLOWLIST = [
        'focusa_marketing_v1' => ['origin' => 'https://focusa.dev', 'checkout_path' => '/activate/checkout/'],
        'focusa_install_v1' => ['origin' => 'https://install.focusa.dev', 'checkout_path' => '/activate/checkout/'],
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

    // ── Journey: terminal/agent registration → verification ───────────────

    public function startRegistration(array $request): array
    {
        $facadeId = (string) ($request['facade_id'] ?? '');
        $origin = (string) ($request['origin'] ?? '');
        $productCode = (string) ($request['product_code'] ?? '');
        $emailDigest = (string) ($request['email_digest'] ?? '');
        $emailDomain = strtolower((string) ($request['email_domain'] ?? ''));
        $emailPrefixChar = (string) ($request['email_prefix_char'] ?? '');
        $presenter = (string) ($request['presenter'] ?? '');
        $installChannel = (string) ($request['install_channel'] ?? '');
        $devicePublicKey = (string) ($request['device_public_key'] ?? '');
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
        if (!in_array($presenter, FocusaSpec152eTerminalAgentPaidState::PRESENTERS, true)) {
            throw new DomainException('PRESENTER_REQUIRED');
        }
        if (!in_array($installChannel, FocusaSpec152eTerminalAgentPaidState::CHANNELS, true)) {
            throw new DomainException('INSTALL_CHANNEL_REQUIRED');
        }
        $this->assertDevicePublicKey($devicePublicKey);
        $facade = self::FACADE_ALLOWLIST[$facadeId] ?? null;
        if ($facade === null || $origin !== $facade['origin']) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if (!isset(self::PRODUCT_MAPPING[$productCode])) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['facade_id', 'origin', 'product_code', 'email_digest', 'email_domain', 'email_prefix_char', 'presenter', 'install_channel', 'device_public_key', 'challenge_code', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('registration_started', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $registrationUuid = $this->opaqueId('reg_ta_' . $presenter . '_' . substr($emailDigest, 0, 8));
        $now = ($this->clock)();
        $challengeCode = (string) ($request['challenge_code'] ?? '');
        if (preg_match('/^[0-9]{6}$/', $challengeCode) !== 1) {
            throw new DomainException('CHALLENGE_FORMAT_DENIED');
        }
        $expiresAt = (new DateTimeImmutable($now))->modify('+' . self::VERIFICATION_TTL_MINUTES . ' minutes')->format('Y-m-d\TH:i:s\Z');
        $challengeHash = hash('sha256', "challenge-v1\n" . $registrationUuid . "\n" . $challengeCode . "\n" . $expiresAt);
        $pollCredential = $this->opaquePollCredential($registrationUuid, $now, 'v1');
        $pollExpiresAt = FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, self::POLL_CREDENTIAL_TTL_SECONDS);

        $statement = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ta_registrations')}
             (registration_uuid, presenter, install_channel, facade_id, origin, product_code,
              email_digest, email_domain, email_prefix_char,
              challenge_hash, challenge_expires_at, challenge_attempts, challenge_used,
              device_public_key, device_public_key_hash,
              poll_credential_hash, poll_credential_expires_at, poll_count, max_polls, is_paused,
              state, request_id, idempotency_key, request_digest, created_at, expires_at, updated_at)
             VALUES (:uuid, :presenter, :channel, :facade, :origin, :product,
              :digest, :domain, :prefix_char,
              :challenge_hash, :challenge_expires_at, 0, 0,
              :device_key, :device_key_hash,
              :poll_hash, :poll_expires, 0, :max_polls, 0,
              'attempt_created', :request_id, :idempotency, :request_digest, :now, :expires, :now)"
        );
        $statement->execute([
            ':uuid' => $registrationUuid, ':presenter' => $presenter, ':channel' => $installChannel,
            ':facade' => $facadeId, ':origin' => $origin, ':product' => $productCode,
            ':digest' => $emailDigest, ':domain' => $emailDomain, ':prefix_char' => $emailPrefixChar,
            ':challenge_hash' => $challengeHash, ':challenge_expires_at' => $expiresAt,
            ':device_key' => $this->canonicalDeviceKey($devicePublicKey),
            ':device_key_hash' => hash('sha256', 'device-key-v1\n' . $devicePublicKey),
            ':poll_hash' => hash('sha256', 'poll-credential-v1\n' . $pollCredential),
            ':poll_expires' => $pollExpiresAt, ':max_polls' => self::MAX_POLLS,
            ':request_id' => $requestId, ':idempotency' => $idempotencyKey,
            ':request_digest' => $requestDigest, ':now' => $now, ':expires' => $expiresAt,
        ]);
        $this->journal('registration_started', $registrationUuid, null, 'attempt_created', $requestId, $idempotencyKey, $requestDigest, $now);

        $maskedEmail = $emailPrefixChar . '***@' . $emailDomain;
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'presenter' => $presenter, 'install_channel' => $installChannel,
            'facade_id' => $facadeId, 'origin' => $origin, 'product_code' => $productCode,
            'state' => 'attempt_created', 'masked_email' => $maskedEmail,
            'challenge_sent_to' => $maskedEmail, 'challenge_expires_at' => $expiresAt,
            'poll_credential' => $pollCredential, 'poll_credential_expires_at' => $pollExpiresAt,
            'max_polls' => self::MAX_POLLS,
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
        if ((int) $row['challenge_used'] === 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }
        if ($now > (string) $row['challenge_expires_at']) {
            throw new DomainException('EMAIL_VERIFICATION_EXPIRED');
        }
        $attempts = (int) $row['challenge_attempts'] + 1;
        $this->updateRegistration($registrationUuid, ['challenge_attempts' => $attempts], $now);
        if ($attempts > self::VERIFICATION_MAX_ATTEMPTS) {
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
            return [
                'ok' => true, 'replayed' => true, 'registration_uuid' => $registrationUuid,
                'account_uuid' => (string) $row['account_uuid'], 'customer_id' => (int) $row['customer_id'],
                'state' => 'account_promoted', 'edd_customer_resolved_or_created' => true, 'zero_new_rows' => true,
            ];
        }
        $now = ($this->clock)();
        $accountUuid = $this->opaqueId('acct_ta_' . substr($registrationUuid, 4, 24));
        $customerId = $this->syntheticCustomerId((string) $row['email_digest']);
        $this->db->beginTransaction();
        try {
            $identity = $this->db->prepare(
                "INSERT OR IGNORE INTO {$this->table('wpuiai_ta_identities')}
                 (identity_uuid, account_uuid, email_digest, email_domain, email_prefix_char, verified_at, verified_method, state)
                 VALUES (:uuid, :account, :digest, :domain, :prefix_char, :verified_at, 'single_use_magic_code', 'verified')"
            );
            $identity->execute([
                ':uuid' => $this->opaqueId('idty_ta_' . substr($registrationUuid, 4, 24)), ':account' => $accountUuid,
                ':digest' => (string) $row['email_digest'], ':domain' => (string) $row['email_domain'],
                ':prefix_char' => (string) $row['email_prefix_char'], ':verified_at' => $now,
            ]);
            $account = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ta_accounts')}
                 (account_uuid, customer_id, facade_id, presenter, state, created_at, updated_at)
                 VALUES (:account, :customer, :facade, :presenter, 'active', :now, :now)"
            );
            $account->execute([
                ':account' => $accountUuid, ':customer' => $customerId,
                ':facade' => (string) $row['facade_id'], ':presenter' => (string) $row['presenter'], ':now' => $now,
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

    // ── Journey: checkout link → order → EDD key → dual delivery ──────────

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
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

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
        $facade = self::FACADE_ALLOWLIST[(string) $row['facade_id']];
        $brandedCheckoutUrl = $facade['origin'] . $facade['checkout_path'] . $checkoutToken;
        $this->updateRegistration($registrationUuid, ['checkout_token' => $checkoutToken, 'state' => 'checkout_pending'], $now);
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
        $orderId = $this->syntheticOrderId((string) $row['email_digest']);
        $orderItemId = $orderId * 10 + 1;
        $verifiedDigest = (string) $row['email_digest'];
        $integrityOk = hash_equals($verifiedDigest, $checkoutEmailDigest);
        $existingOrder = $this->db->prepare("SELECT state FROM {$this->table('wpuiai_ta_orders')} WHERE order_id = :order");
        $existingOrder->execute([':order' => $orderId]);
        $existingState = $existingOrder->fetchColumn();
        if ($existingState === 'complete') {
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
                // Checkout email integrity: the held order completes only now
                // that the verified identity matches the checkout email.
                $flip = $this->db->prepare(
                    "UPDATE {$this->table('wpuiai_ta_orders')}
                     SET state = 'complete', state_reason = NULL, completed_at = :now, updated_at = :now
                     WHERE order_id = :order"
                );
                $flip->execute([':now' => $now, ':order' => $orderId]);
            } elseif ($existingState === false) {
                $order = $this->db->prepare(
                    "INSERT INTO {$this->table('wpuiai_ta_orders')}
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
                    "INSERT INTO {$this->table('wpuiai_ta_order_items')}
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
        $licenseId = $this->syntheticLicenseId((string) $row['email_digest']);
        $licenseKey = $this->deriveHumanKey($licenseId, (int) $row['order_id']);
        $license = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ta_licenses')}
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
            'license_key_mask' => $this->maskHumanKey($licenseKey),
            'key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($licenseKey),
            'duplicate_license' => false,
        ];
    }

    /**
     * Dual key delivery (spec 152E §16): the transactional email channel plus
     * the one-time device-encrypted terminal envelope, both carrying the same
     * canonical EDD human key. Poll credentials rotate at delivery time.
     */
    public function prepareTerminalDelivery(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('delivery_prepared', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'entitlement_issued' || (string) $row['edd_license_id'] === '') {
            throw new DomainException('EDD_LICENSE_PENDING');
        }
        if ((string) $row['terminal_delivery_status'] !== 'none') {
            throw new DomainException('DELIVERY_ALREADY_PREPARED');
        }
        $license = $this->licenseRow((int) $row['edd_license_id']);
        if ((string) $license['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $now = ($this->clock)();
        $keyMask = $this->maskHumanKey((string) $license['license_key']);
        $maskedEmail = (string) $row['email_prefix_char'] . '***@' . (string) $row['email_domain'];
        $emailDeliveryId = 'dlv_ta_email_' . (string) $row['edd_license_id'];
        $delivery = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ta_deliveries')}
             (delivery_id, edd_license_id, channel, recipient_mask, key_mask, state, sent_at)
             VALUES (:id, :license, 'email', :recipient, :key_mask, 'sent', :now)"
        );
        $delivery->execute([
            ':id' => $emailDeliveryId, ':license' => (int) $row['edd_license_id'],
            ':recipient' => $maskedEmail, ':key_mask' => $keyMask, ':now' => $now,
        ]);

        // One-time device-encrypted terminal envelope (deterministic test seam).
        $envelopeId = 'env_' . substr(hash('sha256', 'env-v1\n' . $registrationUuid . '\n' . $now), 0, 32);
        $issuedAt = $now;
        $expiresAt = FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, self::ENVELOPE_TTL_SECONDS);
        $claims = FocusaSpec152eTerminalDeliveryEnvelope::buildClaims([
            'registration_id' => $registrationUuid,
            'account_uuid' => (string) $row['account_uuid'],
            'customer_id' => (int) $row['customer_id'],
            'edd_license_id' => (int) $row['edd_license_id'],
            'product_code' => (string) $row['product_code'],
        ], (string) $license['license_key'], $envelopeId, $issuedAt, $expiresAt);
        $envelope = FocusaSpec152eTerminalEnvelopeCrypto::seal(
            $this->deviceKeyToRaw((string) $row['device_public_key']),
            FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($claims),
            hex2bin(self::ENVELOPE_TEST_EPHEMERAL_PRIVATE_HEX),
            hex2bin(self::ENVELOPE_TEST_NONCE_HEX),
        );
        $digest = hash('sha256', FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson([
            'operation' => 'terminal_delivery_prepare',
            'registration_id' => $registrationUuid,
            'envelope_id' => $envelopeId,
            'request_id' => $requestId,
        ]));
        $envTable = $this->table('wpuiai_ta_envelopes');
        $envStatement = $this->db->prepare(
            "INSERT INTO {$envTable}
             (envelope_id, registration_uuid, account_uuid, edd_customer_id, edd_license_id,
              product_code, license_key_digest, license_key_mask, device_public_key,
              envelope_payload, delivery_status, consumed_at, issued_at, expires_at,
              request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
             VALUES (:envelope_id, :registration, :account, :customer, :license_id,
              :product, :key_digest, :key_mask, :device_key,
              :payload, 'issued', NULL, :issued, :expires,
              :request, :idempotency, :request_digest, :created, :retention, :updated)"
        );
        $envStatement->execute([
            ':envelope_id' => $envelopeId,
            ':registration' => $registrationUuid,
            ':account' => (string) $row['account_uuid'],
            ':customer' => (int) $row['customer_id'],
            ':license_id' => (int) $row['edd_license_id'],
            ':product' => (string) $row['product_code'],
            ':key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest((string) $license['license_key']),
            ':key_mask' => $keyMask,
            ':device_key' => (string) $row['device_public_key'],
            ':payload' => FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($envelope),
            ':issued' => $issuedAt,
            ':expires' => $expiresAt,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':request_digest' => $digest,
            ':created' => $now,
            ':retention' => FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, 2592000),
            ':updated' => $now,
        ]);

        // Rotate the poll credential at delivery time (fresh bounded window).
        $pollCredential = $this->opaquePollCredential($registrationUuid, $now, 'v2');
        $pollExpiresAt = FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, self::POLL_CREDENTIAL_TTL_SECONDS);
        $this->updateRegistration($registrationUuid, [
            'envelope_id' => $envelopeId,
            'terminal_delivery_status' => 'ready',
            'email_delivery_status' => 'sent',
            'poll_credential_hash' => hash('sha256', 'poll-credential-v1\n' . $pollCredential),
            'poll_credential_expires_at' => $pollExpiresAt,
            'state' => 'license_delivery_ready',
        ], $now);
        $this->journal('delivery_prepared', $registrationUuid, (string) $row['account_uuid'], 'license_delivery_ready', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'license_delivery_ready',
            'channels' => ['email' => 'sent', 'terminal' => 'ready'],
            'email_recipient_mask' => $maskedEmail, 'key_mask' => $keyMask,
            'envelope_id' => $envelopeId, 'envelope_expires_at' => $expiresAt,
            'same_canonical_key_both_channels' => true,
            'promotional_content' => false,
            'poll_credential' => $pollCredential, 'poll_credential_expires_at' => $pollExpiresAt,
        ];
    }

    // ── Bounded poll, pause/resume, agent status ──────────────────────────

    /**
     * Bounded poll (spec 152E §14.2). cli_terminal registrations may receive
     * the one-time device-bound envelope exactly once from this device channel;
     * agent_json registrations receive only the masked agent transcript, never
     * the envelope, credential, or key. Budget exhaustion cancels the session
     * fail-closed to recovery_only.
     */
    public function poll(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $pollCredential = (string) ($request['poll_credential'] ?? '');
        $devicePublicKey = (string) ($request['device_public_key'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'poll_credential', 'device_public_key', 'request_id', 'idempotency_key']);
        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['presenter'] === 'cli_terminal') {
            $this->assertDevicePublicKey($devicePublicKey);
            $this->assertDeviceBinding((string) $row['device_public_key'], $devicePublicKey);
        }
        $this->assertPollCredential($row, $pollCredential);
        if ((int) $row['is_paused'] === 1) {
            throw new DomainException('SESSION_PAUSED');
        }
        return $this->pollInternal($row, $requestId, $idempotencyKey, (string) $row['presenter'] === 'cli_terminal');
    }

    /** Read-only agent status snapshot: never consumes the poll budget and
     *  never carries an envelope, credential, or key (spec 152E §14.2). */
    public function agentStatus(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $pollCredential = (string) ($request['poll_credential'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'poll_credential', 'request_id', 'idempotency_key']);
        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['presenter'] !== 'agent_json') {
            throw new DomainException('AGENT_PRESENTER_REQUIRED');
        }
        $this->assertPollCredential($row, $pollCredential);
        return $this->agentEnvelope($row, $requestId, false);
    }

    /** Agent-only pause: rotates the protected poll credential and returns a
     *  resumable handle. Refused for terminal presenters and terminal states. */
    public function pause(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('session_paused', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['presenter'] !== 'agent_json') {
            throw new DomainException('PAUSE_STEP_DENIED');
        }
        if (FocusaSpec152eTerminalAgentPaidState::isTerminalState((string) $row['state'])) {
            throw new DomainException('PAUSE_STATE_DENIED');
        }
        if ((int) $row['is_paused'] === 1) {
            throw new DomainException('PAUSE_ALREADY_PAUSED');
        }
        $now = ($this->clock)();
        $pollCredential = $this->opaquePollCredential($registrationUuid, $now, 'v3');
        $pollExpiresAt = FocusaSpec152eTerminalDeliveryEnvelope::plusSeconds($now, self::POLL_CREDENTIAL_TTL_SECONDS);
        $this->updateRegistration($registrationUuid, [
            'is_paused' => 1,
            'poll_credential_hash' => hash('sha256', 'poll-credential-v1\n' . $pollCredential),
            'poll_credential_expires_at' => $pollExpiresAt,
        ], $now);
        $this->journal('session_paused', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => (string) $row['state'], 'paused' => true,
            'poll_credential' => $pollCredential, 'poll_credential_expires_at' => $pollExpiresAt,
            'resume_ttl_seconds' => self::POLL_CREDENTIAL_TTL_SECONDS,
        ];
    }

    /** Agent-only resume: re-supplied protected poll credential required;
     *  refuses terminal presenters, terminal states, and non-paused sessions. */
    public function resume(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $pollCredential = (string) ($request['poll_credential'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'poll_credential', 'request_id', 'idempotency_key']);
        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['presenter'] !== 'agent_json') {
            throw new DomainException('RESUME_STEP_DENIED');
        }
        if ((int) $row['is_paused'] !== 1) {
            throw new DomainException('RESUME_STATE_DENIED');
        }
        if (FocusaSpec152eTerminalAgentPaidState::isTerminalState((string) $row['state'])) {
            throw new DomainException('RESUME_STATE_DENIED');
        }
        $this->assertPollCredential($row, $pollCredential);
        $now = ($this->clock)();
        $this->updateRegistration($registrationUuid, ['is_paused' => 0], $now);
        $row['is_paused'] = 0;
        $this->journal('session_resumed', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], $requestId, $idempotencyKey, $this->requestDigest($request), $now);
        // Resume is a bounded poll with the re-supplied protected credential:
        // it consumes one poll and enforces the budget like any other poll.
        return $this->pollInternal($row, $requestId, $idempotencyKey, false);
    }

    // ── Device seam: envelope open → credential store → explicit reveal ───

    /**
     * Device-side open of the one-time terminal envelope. Accepts either the
     * envelope array (from a terminal poll response) or an envelope_id (the
     * agent device's out-of-band fetch). Fails closed on wrong device, tamper,
     * expiry, or binding mismatch; consuming a stored envelope marks it used.
     */
    public function openEnvelope(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $envelopeInput = $request['envelope'] ?? null;
        $envelopeId = (string) ($request['envelope_id'] ?? '');
        $devicePrivate = $this->devicePrivateFromInput((string) ($request['device_private_key'] ?? ''));
        $now = (string) ($request['now'] ?? $this->now());
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        $row = $this->registrationRow($registrationUuid);

        $envelope = null;
        $fromStore = false;
        if (is_array($envelopeInput) && $envelopeInput !== []) {
            $envelope = $envelopeInput;
            if (isset($envelope['envelope_id'])) {
                $envelopeId = (string) $envelope['envelope_id'];
            }
        } elseif (preg_match('/^env_[0-9a-f]{32}$/D', $envelopeId) === 1) {
            $stored = $this->envelopeRow($envelopeId);
            if (!hash_equals((string) $stored['registration_uuid'], $registrationUuid)) {
                throw new DomainException('ENVELOPE_BINDING_MISMATCH');
            }
            if (!in_array((string) $stored['delivery_status'], ['issued', 'delivered'], true)) {
                throw new DomainException('ENVELOPE_ALREADY_CONSUMED');
            }
            $envelope = json_decode((string) $stored['envelope_payload'], true, 512, JSON_THROW_ON_ERROR);
            $fromStore = true;
        } else {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        if (!is_array($envelope) || $envelope === []) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $plaintext = FocusaSpec152eTerminalEnvelopeCrypto::open($devicePrivate, $envelope);
        $claims = json_decode($plaintext, true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($claims)) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, $now, $registrationUuid);
        if ($fromStore) {
            $this->consumeEnvelope($envelopeId, $now);
        }
        return [
            'ok' => true, 'registration_uuid' => $registrationUuid,
            'envelope_id' => $envelopeId, 'claims_validated' => true,
            'binding_matches' => true, 'one_time' => true,
            'license_key_mask' => $this->maskHumanKey((string) $claims['license_key']),
            'license_key_digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest((string) $claims['license_key']),
        ];
    }

    /**
     * Protected credential adapter seam (client side): opens the device-bound
     * envelope and stores the decrypted key in the protected credential store
     * (OS keyring analog). Only the opaque handle, mask, and digest are ever
     * observable; the key is revealed later only under explicit consent.
     */
    public function credentialStore(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $envelopeInput = $request['envelope'] ?? null;
        $envelopeId = (string) ($request['envelope_id'] ?? '');
        $devicePrivate = $this->devicePrivateFromInput((string) ($request['device_private_key'] ?? ''));
        $now = (string) ($request['now'] ?? $this->now());
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'envelope', 'envelope_id', 'device_private_key', 'now', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('credential_stored', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if (FocusaSpec152eTerminalAgentPaidState::isTerminalState((string) $row['state'])) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        $envelope = null;
        $fromStore = false;
        if (is_array($envelopeInput) && $envelopeInput !== []) {
            $envelope = $envelopeInput;
            if (isset($envelope['envelope_id'])) {
                $envelopeId = (string) $envelope['envelope_id'];
            }
        } elseif (preg_match('/^env_[0-9a-f]{32}$/D', $envelopeId) === 1) {
            $stored = $this->envelopeRow($envelopeId);
            if (!hash_equals((string) $stored['registration_uuid'], $registrationUuid)) {
                throw new DomainException('ENVELOPE_BINDING_MISMATCH');
            }
            if (!in_array((string) $stored['delivery_status'], ['issued', 'delivered'], true)) {
                throw new DomainException('ENVELOPE_ALREADY_CONSUMED');
            }
            $envelope = json_decode((string) $stored['envelope_payload'], true, 512, JSON_THROW_ON_ERROR);
            $fromStore = true;
        } else {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        if (!is_array($envelope) || $envelope === []) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        $plaintext = FocusaSpec152eTerminalEnvelopeCrypto::open($devicePrivate, $envelope);
        $claims = json_decode($plaintext, true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($claims)) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, $now, $registrationUuid);
        if (preg_match('/^env_[0-9a-f]{32}$/D', $envelopeId) === 1) {
            $this->consumeEnvelope($envelopeId, $now);
        }
        $now = ($this->clock)();
        $handle = 'cred_ta_' . $this->opaqueTail((string) $row['presenter'] . '_' . substr($registrationUuid, 4, 8));
        $key = (string) $claims['license_key'];
        $store = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ta_credential_stores')}
             (handle, registration_uuid, key_digest, key_mask, claims_json, consumed, consumed_at, created_at, updated_at)
             VALUES (:handle, :registration, :digest, :mask, :claims, 0, NULL, :now, :now)"
        );
        $store->execute([
            ':handle' => $handle, ':registration' => $registrationUuid,
            ':digest' => FocusaSpec152eTerminalDeliveryEnvelope::keyDigest($key),
            ':mask' => $this->maskHumanKey($key),
            ':claims' => FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($claims), ':now' => $now,
        ]);
        $this->journal('credential_stored', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false,
            'schema' => 'focusa.spec152e.terminal_credential_adapter.v1',
            'operation' => 'store', 'handle' => $handle,
            'mask' => $this->maskHumanKey($key),
            'store' => 'protected_credential_store', 'revealed' => false,
        ];
    }

    /**
     * Explicit customer-controlled key reveal (spec 152E §14.2). Requires BOTH
     * opt-in (reveal_key) AND confirmation (reveal_confirmation), stays within
     * the envelope lifetime, is one-time (replay denied), and refuses once the
     * registration settles. Full keys are masked everywhere else.
     */
    public function revealKey(array $request): array
    {
        $handle = (string) ($request['handle'] ?? '');
        $optIn = ($request['reveal_key'] ?? false) === true;
        $confirmation = ($request['reveal_confirmation'] ?? false) === true;
        $now = (string) ($request['now'] ?? $this->now());
        FocusaSpec152eTerminalDeliveryEnvelope::assertTimestamp($now);
        if (preg_match('/^cred_[A-Za-z0-9_-]{1,96}$/D', $handle) !== 1) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        $row = $this->credentialRow($handle);
        if (!$optIn || !$confirmation) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        if ((int) $row['consumed'] === 1) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        $registration = $this->registrationRow((string) $row['registration_uuid']);
        if (FocusaSpec152eTerminalAgentPaidState::isTerminalState((string) $registration['state'])) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        $claims = json_decode((string) $row['claims_json'], true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($claims)) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        if ((string) $claims['expires_at'] <= $now) {
            throw new DomainException('CREDENTIAL_REVEAL_EXPIRED');
        }
        FocusaSpec152eTerminalDeliveryEnvelope::assertClaims($claims, $now, null);
        $update = $this->db->prepare(
            "UPDATE {$this->table('wpuiai_ta_credential_stores')}
             SET consumed = 1, consumed_at = :now, updated_at = :now
             WHERE handle = :handle AND consumed = 0"
        );
        $update->execute([':now' => $now, ':handle' => $handle]);
        if ($update->rowCount() !== 1) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        $key = (string) $claims['license_key'];
        $this->journal('credential_revealed', (string) $registration['registration_uuid'], (string) $registration['account_uuid'], (string) $registration['state'], 'req_ta_reveal_' . substr($handle, 5, 12), 'idem_ta_reveal_' . substr($handle, 5, 12), hash('sha256', 'reveal\n' . $handle), $now);
        return [
            'ok' => true,
            'schema' => 'focusa.spec152e.terminal_credential_adapter.v1',
            'operation' => 'reveal', 'handle' => $handle,
            'revealed' => true, 'license_key' => $key,
            'mask' => $this->maskHumanKey($key),
        ];
    }

    // ── Journey: node registration → signed lease → refund cleanup ────────

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
        $this->assertDevicePublicKey($devicePublicKey);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'node_id', 'device_public_key', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('node_registered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if (!in_array((string) $row['state'], ['license_delivery_ready', 'terminal_delivered', 'device_registered'], true)
            || (string) $row['edd_license_id'] === '') {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        $this->assertDeviceBinding((string) $row['device_public_key'], $devicePublicKey);
        $licenseRow = $this->licenseRow((int) $row['edd_license_id']);
        if ((string) $licenseRow['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $now = ($this->clock)();
        $nodeUuid = $this->opaqueId('node_ta_' . substr($registrationUuid, 4, 24));
        $node = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ta_nodes')}
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
        $leaseUuid = $this->opaqueId('lease_ta_' . substr($registrationUuid, 4, 24));
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
            'node_id' => (string) $nodeRow['node_uuid'],
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
            "INSERT INTO {$this->table('wpuiai_ta_leases')}
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
            $orderUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ta_orders')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE order_id = :order");
            $orderUpdate->execute([':now' => $now, ':order' => $orderId]);
            $licenseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ta_licenses')} SET state = 'refunded', updated_at = :now WHERE edd_license_id = :license");
            $licenseUpdate->execute([':now' => $now, ':license' => $licenseId]);
            $leaseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ta_leases')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE account_uuid = :account AND state = 'active'");
            $leaseUpdate->execute([':now' => $now, ':account' => (string) $row['account_uuid']]);
            $refund = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ta_refunds')}
                 (refund_id, order_id, edd_license_id, reason, sequence_after, refunded_at)
                 VALUES (:refund, :order, :license, :reason, :sequence, :now)"
            );
            $refund->execute([
                ':refund' => 'rfnd_ta_' . substr($registrationUuid, 4, 24), ':order' => $orderId, ':license' => $licenseId,
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
            "SELECT current_sequence FROM {$this->table('wpuiai_ta_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $sequenceStatement->execute([':account' => (string) $row['account_uuid'], ':product' => (string) $row['product_code']]);
        $currentSequence = $sequenceStatement->fetchColumn();
        $receipt = [
            'schema' => self::RECEIPT_SCHEMA,
            'fixture' => 'focusa-vbcqu.20.13.57',
            'presenter' => (string) $row['presenter'],
            'install_channel' => (string) $row['install_channel'],
            'facade_id' => (string) $row['facade_id'],
            'origin' => (string) $row['origin'],
            'product_code' => (string) $row['product_code'],
            'masked_email' => $maskedEmail,
            'state' => (string) $row['state'],
            'order_id' => (int) $row['order_id'],
            'edd_license_id' => $licenseRow === null ? null : (int) $row['edd_license_id'],
            'key_mask' => $licenseRow === null ? null : $this->maskHumanKey((string) $licenseRow['license_key']),
            'node_uuid' => (string) $row['node_uuid'],
            'lease_sequence' => $currentSequence === false ? null : (int) $currentSequence,
            'lease_state' => $leaseRow === null ? null : (string) $leaseRow['state'],
            'lease_envelope_digest' => $leaseRow === null ? null : (string) $leaseRow['envelope_digest'],
            'customer_id' => (int) $row['customer_id'],
            'poll_count' => (int) $row['poll_count'],
            'max_polls' => (int) $row['max_polls'],
            'install_site_authority' => 'none',
            'spec158' => 'excluded',
            'redaction' => ['raw_email' => 'absent', 'raw_key' => 'absent', 'payment_reference' => 'digest_only', 'poll_credential' => 'hash_only'],
        ];
        $canonical = $this->canonicalJson($receipt);
        $receipt['receipt_sha256'] = hash('sha256', "focusa.spec152e.terminal_agent_receipt.v1\n" . $canonical);
        $this->journal('receipt_issued', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], 'req_ta_receipt_' . substr($registrationUuid, 4, 8), 'idem_ta_receipt_' . substr($registrationUuid, 4, 8), hash('sha256', 'receipt\n' . $registrationUuid), $now);
        return $receipt;
    }

    // ── Poll internals ────────────────────────────────────────────────────

    private function pollInternal(array $row, string $requestId, string $idempotencyKey, bool $terminalDevice): array
    {
        $state = (string) $row['state'];
        if (FocusaSpec152eTerminalAgentPaidState::isTerminalState($state)) {
            return $terminalDevice
                ? $this->terminalResponse($row, $requestId, false, null)
                : $this->agentEnvelope($row, $requestId, false);
        }
        $now = $this->now();
        $pollCount = (int) $row['poll_count'];
        if ($pollCount >= (int) $row['max_polls']) {
            // Budget exhausted: cancel fail-closed to recovery_only (never regrants).
            $requestDigest = hash('sha256', 'poll-exhaust\n' . (string) $row['registration_uuid'] . '\n' . $idempotencyKey);
            $this->updateRegistration((string) $row['registration_uuid'], ['state' => 'recovery_only'], $now);
            $this->journal('poll_budget_exhausted', (string) $row['registration_uuid'], (string) $row['account_uuid'], 'recovery_only', $requestId, $idempotencyKey, $requestDigest, $now);
            $row['state'] = 'recovery_only';
            return $terminalDevice
                ? $this->terminalResponse($row, $requestId, false, null)
                : $this->agentEnvelope($row, $requestId, false);
        }
        $pollCount++;
        $this->updateRegistration((string) $row['registration_uuid'], ['poll_count' => $pollCount], $now);
        $row['poll_count'] = $pollCount;

        // The one-time envelope is delivered exactly once: any later poll for a
        // terminal device fails closed (LICENSE_DELIVERY_FAILED).
        if ($terminalDevice && (string) $row['terminal_delivery_status'] === 'delivered') {
            throw new DomainException('LICENSE_DELIVERY_FAILED');
        }

        if ($terminalDevice && $state === 'license_delivery_ready' && (string) $row['terminal_delivery_status'] === 'ready') {
            $envelope = $this->deliverEnvelopeOnce($row, $now);
            $row = $this->registrationRow((string) $row['registration_uuid']);
            return $this->terminalResponse($row, $requestId, true, $envelope);
        }
        return $terminalDevice
            ? $this->terminalResponse($row, $requestId, false, null)
            : $this->agentEnvelope($row, $requestId, false);
    }

    private function deliverEnvelopeOnce(array $row, string $now): array
    {
        $envelopeId = (string) $row['envelope_id'];
        $envelopeRow = $this->envelopeRow($envelopeId);
        if ((string) $envelopeRow['delivery_status'] !== 'issued') {
            throw new DomainException('LICENSE_DELIVERY_FAILED');
        }
        $envTable = $this->table('wpuiai_ta_envelopes');
        $update = $this->db->prepare(
            "UPDATE {$envTable} SET delivery_status = 'delivered', updated_at = :now WHERE envelope_id = :id AND delivery_status = 'issued'"
        );
        $update->execute([':now' => $now, ':id' => $envelopeId]);
        if ($update->rowCount() !== 1) {
            throw new DomainException('LICENSE_DELIVERY_FAILED');
        }
        $this->updateRegistration((string) $row['registration_uuid'], [
            'terminal_delivery_status' => 'delivered',
            'state' => 'terminal_delivered',
        ], $now);
        $row['terminal_delivery_status'] = 'delivered';
        $row['state'] = 'terminal_delivered';
        $requestId = 'req_ta_deliver_' . substr($envelopeId, 4, 8);
        $this->journal('terminal_delivered', (string) $row['registration_uuid'], (string) $row['account_uuid'], 'terminal_delivered', $requestId, 'idem_ta_deliver_' . substr($envelopeId, 4, 8), hash('sha256', 'deliver\n' . $envelopeId), $now);
        $envelope = json_decode((string) $envelopeRow['envelope_payload'], true, 512, JSON_THROW_ON_ERROR);
        return is_array($envelope) ? $envelope : [];
    }

    /** Terminal device response (focusa.activation.response.v1). */
    private function terminalResponse(array $row, string $requestId, bool $delivered, ?array $envelope): array
    {
        $response = [
            'schema' => self::TERMINAL_RESPONSE_SCHEMA,
            'request_id' => $requestId,
            'registration_id' => (string) $row['registration_uuid'],
            'state' => (string) $row['state'],
            'terminal' => true,
            'poll_count' => (int) $row['poll_count'],
            'max_polls' => (int) $row['max_polls'],
            'retry' => ['posture' => FocusaSpec152eTerminalAgentPaidState::isTerminalState((string) $row['state']) ? 'none' : 'poll_within_budget'],
            'next_action' => FocusaSpec152eTerminalAgentPaidState::presenter($row)[2],
            'terminal_delivery_status' => (string) $row['terminal_delivery_status'],
        ];
        if ((string) $row['state'] === 'checkout_pending' && (string) $row['checkout_token'] !== '') {
            $facade = self::FACADE_ALLOWLIST[(string) $row['facade_id']];
            $response['safe_url'] = $facade['origin'] . $facade['checkout_path'] . (string) $row['checkout_token'];
        }
        if ($delivered && $envelope !== null && $envelope !== []) {
            $envelopeRow = $this->envelopeRow((string) $row['envelope_id']);
            $response['envelope_id'] = (string) $row['envelope_id'];
            $response['license_key_mask'] = (string) $envelopeRow['license_key_mask'];
            $response['one_time_key_envelope'] = FocusaSpec152eTerminalEnvelopeCrypto::base64UrlEncode(FocusaSpec152eTerminalEnvelopeCrypto::canonicalJson($envelope));
            $response['delivery_status'] = 'delivered';
        }
        return $response;
    }

    /** Agent transcript (focusa.agent_activation_envelope.v1): masked only,
     *  never the raw email, key, envelope, or credential. */
    private function agentEnvelope(array $row, string $requestId, bool $error): array
    {
        [$presenterState, $humanAction, $nextAction, $terminal] = FocusaSpec152eTerminalAgentPaidState::presenter($row);
        $maskedEmail = (string) $row['email_prefix_char'] . '***@' . (string) $row['email_domain'];
        $keyPresent = (string) $row['edd_license_id'] !== '';
        $envelope = [
            'schema' => self::AGENT_ENVELOPE_SCHEMA,
            'registration_id' => (string) $row['registration_uuid'],
            'state' => $presenterState,
            'terminal' => $terminal,
            'human_action_required' => $humanAction !== '',
            'human_action' => $humanAction,
            'key_present' => $keyPresent,
            'key_visible' => false,
            'poll_count' => (int) $row['poll_count'],
            'max_polls' => (int) $row['max_polls'],
            'retry_posture' => $terminal ? 'none' : $presenterState,
            'retry_after_seconds' => self::RETRY_AFTER_SECONDS,
            'next_action' => $nextAction,
            'masked_email' => $maskedEmail,
        ];
        if (!$terminal && in_array($presenterState, ['checkout_required', 'payment_pending'], true) && (string) $row['checkout_token'] !== '') {
            $facade = self::FACADE_ALLOWLIST[(string) $row['facade_id']];
            $envelope['safe_url'] = $facade['origin'] . $facade['checkout_path'] . (string) $row['checkout_token'];
        }
        if ($error) {
            $envelope['error'] = 'STATE_TRANSITION_DENIED';
        }
        return $envelope;
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    private function table(string $name): string
    {
        return $this->prefix . $name;
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eTerminalAgentPaidMigration::assertTimestamp($now);
        return $now;
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
            "SELECT event_type, request_digest FROM {$this->table('wpuiai_ta_journal')} WHERE idempotency_key = :key AND event_type = :operation"
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
            "INSERT INTO {$this->table('wpuiai_ta_journal')}
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
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_registrations')} WHERE registration_uuid = :uuid");
        $statement->execute([':uuid' => $registrationUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('REGISTRATION_NOT_FOUND');
        }
        return $row;
    }

    private function licenseRow(int $licenseId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_licenses')} WHERE edd_license_id = :id");
        $statement->execute([':id' => $licenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    private function orderRow(int $orderId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_orders')} WHERE order_id = :id");
        $statement->execute([':id' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        return $row;
    }

    private function orderItemRow(int $orderId): int
    {
        $statement = $this->db->prepare("SELECT order_item_id FROM {$this->table('wpuiai_ta_order_items')} WHERE order_id = :id");
        $statement->execute([':id' => $orderId]);
        $itemId = $statement->fetchColumn();
        if ($itemId === false) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        return (int) $itemId;
    }

    private function nodeRow(string $nodeUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_nodes')} WHERE node_uuid = :uuid");
        $statement->execute([':uuid' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        return $row;
    }

    private function leaseRow(string $leaseUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_leases')} WHERE lease_uuid = :uuid");
        $statement->execute([':uuid' => $leaseUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('LEASE_NOT_FOUND');
        }
        return $row;
    }

    private function envelopeRow(string $envelopeId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_envelopes')} WHERE envelope_id = :id");
        $statement->execute([':id' => $envelopeId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('ENVELOPE_FORMAT_DENIED');
        }
        return $row;
    }

    private function credentialRow(string $handle): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ta_credential_stores')} WHERE handle = :handle");
        $statement->execute([':handle' => $handle]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('CREDENTIAL_REVEAL_DENIED');
        }
        return $row;
    }

    private function consumeEnvelope(string $envelopeId, string $now): void
    {
        $envTable = $this->table('wpuiai_ta_envelopes');
        $update = $this->db->prepare(
            "UPDATE {$envTable} SET delivery_status = 'consumed', consumed_at = :now, updated_at = :now
             WHERE envelope_id = :id AND delivery_status IN ('issued', 'delivered')"
        );
        $update->execute([':now' => $now, ':id' => $envelopeId]);
    }

    private function assertPollCredential(array $row, string $pollCredential): void
    {
        if ($pollCredential === '' || strlen($pollCredential) > 512 || preg_match('/[\r\n]/', $pollCredential)) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
        $now = $this->now();
        if ($now >= (string) $row['poll_credential_expires_at']) {
            throw new DomainException('POLL_CREDENTIAL_EXPIRED');
        }
        if (!hash_equals((string) $row['poll_credential_hash'], hash('sha256', 'poll-credential-v1\n' . $pollCredential))) {
            throw new DomainException('POLL_CREDENTIAL_REQUIRED');
        }
    }

    private function assertDevicePublicKey(string $key): void
    {
        $decoded = $this->deviceKeyDecode($key);
        if ($decoded === null || strlen($decoded) !== 32) {
            throw new DomainException('DEVICE_PUBLIC_KEY_REQUIRED');
        }
    }

    private function assertDeviceBinding(string $registeredKey, string $submittedKey): void
    {
        if (!hash_equals($this->canonicalDeviceKey($registeredKey), $this->canonicalDeviceKey($submittedKey))) {
            throw new DomainException('DEVICE_BINDING_MISMATCH');
        }
    }

    private function canonicalDeviceKey(string $key): string
    {
        $decoded = $this->deviceKeyDecode($key);
        return $decoded === null ? '' : rtrim(strtr(base64_encode($decoded), '+/', '-_'), '=');
    }

    private function deviceKeyDecode(string $key): ?string
    {
        if ($key === '') {
            return null;
        }
        $decoded = base64_decode($key, true);
        if ($decoded !== false) {
            return $decoded;
        }
        $padding = (4 - strlen($key) % 4) % 4;
        $decoded = base64_decode(strtr($key . str_repeat('=', $padding), '-_', '+/'), true);
        return $decoded === false ? null : $decoded;
    }

    private function deviceKeyToRaw(string $key): string
    {
        $decoded = $this->deviceKeyDecode($key);
        if ($decoded === null || strlen($decoded) !== 32) {
            throw new DomainException('ENVELOPE_DEVICE_KEY_DENIED');
        }
        return $decoded;
    }

    private function devicePrivateFromInput(string $input): string
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $input) === 1) {
            $bin = hex2bin($input);
            if ($bin !== false) {
                return $bin;
            }
        }
        if (strlen($input) === 32) {
            return $input;
        }
        throw new DomainException('ENVELOPE_DEVICE_KEY_DENIED');
    }

    private function nextSequence(string $accountUuid, string $productCode, string $now): int
    {
        $select = $this->db->prepare(
            "SELECT current_sequence FROM {$this->table('wpuiai_ta_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $select->execute([':account' => $accountUuid, ':product' => $productCode]);
        $current = $select->fetchColumn();
        if ($current === false) {
            $insert = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ta_sequences')} (account_uuid, product_code, current_sequence, created_at, updated_at)
                 VALUES (:account, :product, 0, :now, :now)"
            );
            $insert->execute([':account' => $accountUuid, ':product' => $productCode, ':now' => $now]);
            $current = 0;
        }
        $next = (int) $current + 1;
        $update = $this->db->prepare(
            "UPDATE {$this->table('wpuiai_ta_sequences')} SET current_sequence = :next, updated_at = :now
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
        $statement = $this->db->prepare("UPDATE {$this->table('wpuiai_ta_registrations')} SET " . implode(', ', $sets) . " WHERE registration_uuid = :uuid");
        $statement->execute($params);
    }

    private function opaqueId(string $seed): string
    {
        $hex = substr(hash('sha256', 'opaque-id-v1\n' . $seed), 0, 32);
        return substr($hex, 0, 8) . '-' . substr($hex, 8, 4) . '-' . substr($hex, 12, 4) . '-' . substr($hex, 16, 4) . '-' . substr($hex, 20, 12);
    }

    private function opaqueTail(string $seed): string
    {
        return substr(hash('sha256', 'opaque-tail-v1\n' . $seed), 0, 24);
    }

    private function opaquePollCredential(string $registrationUuid, string $now, string $variant): string
    {
        return 'pollcred_' . substr(hash('sha256', 'poll-v' . $variant . '\n' . $registrationUuid . '\n' . $now), 0, 32);
    }

    private function syntheticCustomerId(string $emailDigest): int
    {
        return 1001 + (int) hexdec(substr($emailDigest, 0, 8)) % 900;
    }

    private function syntheticOrderId(string $emailDigest): int
    {
        return 9001 + (int) hexdec(substr($emailDigest, 8, 8)) % 900;
    }

    private function syntheticLicenseId(string $emailDigest): int
    {
        return 7001 + (int) hexdec(substr($emailDigest, 16, 8)) % 900;
    }

    /** Canonical masked key shape (spec 152E §16.2, terminal-delivery family). */
    private function maskHumanKey(string $licenseKey): string
    {
        return FocusaSpec152eTerminalDeliveryEnvelope::maskKey($licenseKey);
    }

    private function deriveHumanKey(int $licenseId, int $orderId): string
    {
        $raw = strtoupper(substr(preg_replace('/[^A-F0-9]/', '', strtoupper(hash('sha256', "edd-sl-v1\n" . $licenseId . "\n" . $orderId))), 0, 32));
        return implode('-', str_split($raw, 8));
    }

    private function assertDigest(string $value, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{64}$/', $value) !== 1) {
            throw new DomainException(strtoupper($kind) . '_DIGEST_REQUIRED');
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
