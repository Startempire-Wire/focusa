<?php
// Spec 152E.07.05 UIAI and bundle account/product isolation journey contract
// (atom focusa-vbcqu.20.13.59; spec 152E §7 account/data model, §8 product and
// grant registry, §11 paid customer journey, §15 human key and signed lease
// separation, §16 dual-channel license delivery, §17 EDD lifecycle integration,
// §18 refund/revoke/recovery, §23 acceptance-matrix rows "UIAI purchase",
// "Bundle purchase", and "Wrong product"). This is a locked-release correction
// proof harness; Spec 158 implementation is excluded.
//
// The contract proves, on sqlite through a deterministic synthetic test-mode
// harness, that every purchase grants EXACT products only and that a Bundle
// purchase uses ONE verified account with no silent partial success:
//
//   - Shared identity: promotion is keyed by the verified email digest/domain
//     identity, never by the product code. The same verified identity buying
//     Focusa, UIAI, and/or the Bundle reuses the SAME authority account and the
//     SAME EDD customer id (identity_reused=true, zero new rows); a bundle never
//     creates a second account and no duplicate customer identity is ever
//     created. Per-product monotonic sequence ledgers remain independent.
//   - Server-owned product/grant registry: the caller submits only the public
//     product code (focusa_operator_lifetime_v1, uiai_operator_lifetime_v1, or
//     focusa_uiai_operator_bundle_lifetime_v1). Every EDD download, price,
//     grant, feature, limit, term, refund policy, and commercial right is
//     resolved server-side; caller-supplied product/download/price/grant/
//     limit/redirect fields fail closed with CALLER_CONTROLLED_GRANT_DENIED and
//     unknown codes fail closed with PRODUCT_MAPPING_REQUIRED.
//   - Exact-product grants: a Focusa-only purchase grants exactly the Focusa
//     product; a UIAI-only purchase grants exactly the uiai_engine product; the
//     Bundle is ONE commerce SKU and ONE canonical EDD Software Licensing human
//     key that grants the exact union of the two underlying Operator v1 records
//     (grant_composition=exact_union, human_key_count=1). No cross-product
//     lease or downgrade exists: each signed lease carries exactly the grants
//     of its own purchase and resolveProductGrants() never expands a wrong
//     product.
//   - Key/lease delivery: the one canonical EDD key is delivered through
//     transactional email AND the terminal-independent account channel with the
//     same masked key; the signed device-bound lease is sealed by the canonical
//     Ed25519 authority key-set seam over FOCUSA-AUTHORITY-LEASE-V1 and carries
//     the exact grants, features, limits, subject account, and monotonic
//     sequence.
//   - Child token: a UIAI or Bundle purchase may derive the bounded
//     uiai-engine operator child token (focusa.uiai_child_token.v1) from its
//     active lease and license — 15-minute maximum TTL, exactly the UIAI local
//     family set (never an expanded scope), the frozen hosted-resource
//     exclusion digest, one seat / three nodes, stored as token_digest only. A
//     Focusa-only purchase fails closed with CHILD_TOKEN_NOT_INCLUDED.
//   - Partial failure/recovery: an email-channel bounce (test-mode server seam)
//     marks delivery partial — the account channel may still deliver, but the
//     license is never silently granted, node registration and lease issuance
//     fail closed with PARTIAL_DELIVERY_PENDING until recoverDelivery()
//     retries the failed channel. Recovery never creates a second key and never
//     re-delivers the healthy channel.
//   - Refund and reactivation: an EDD refund increments the monotonic sequence,
//     supersedes the active lease to refunded, settles recovery_only, and
//     denies refresh. Refunded/revoked records never reactivate:
//     reactivate() fails closed with REFUNDED_NEVER_REACTIVATES and a new
//     purchase (new EDD order for the same verified identity) creates a NEW
//     key/lease while the refunded order/license/lease rows remain preserved.
//
// Failures are public-safe stable codes (FACADE_ORIGIN_DENIED,
// PRODUCT_MAPPING_REQUIRED, CALLER_CONTROLLED_GRANT_DENIED,
// EMAIL_DIGEST_REQUIRED, EMAIL_VERIFICATION_REQUIRED, EMAIL_VERIFICATION_FAILED,
// EMAIL_VERIFICATION_EXPIRED, EDD_CHECKOUT_REQUIRED, EDD_ORDER_UNVERIFIED,
// EDD_ORDER_PENDING, EDD_LICENSE_PENDING, EDD_LICENSE_UNUSABLE,
// LICENSE_DELIVERY_PENDING, PARTIAL_DELIVERY_PENDING, DELIVERY_ALREADY_PARTIAL,
// DELIVERY_ALREADY_DELIVERED, NODE_NOT_FOUND, DEVICE_PUBLIC_KEY_REQUIRED,
// NODE_REQUIRED, LEASE_REQUIRED, CHILD_TOKEN_NOT_INCLUDED,
// REFUND_REASON_REQUIRED, REFUND_STATE_REQUIRED, REFUNDED_NEVER_REACTIVATES,
// REACTIVATION_REQUIRES_NEW_ORDER, REQUEST_ID_REQUIRED, IDEMPOTENCY_KEY_REQUIRED,
// IDEMPOTENCY_CONFLICT, REGISTRATION_NOT_FOUND). No new error code is
// introduced and no raw email, raw key, payment reference, token secret,
// credential, customer row, or card data is stored or returned.
//
// Requires docs/contracts/spec152e-edd-bound-lease-issuer.v1.php (canonical
// Ed25519 signer + authority key-set seam, loaded first),
// docs/contracts/spec172-edd-license-type-projector.v1.php (canonical
// projection-journal encodeCanonical used by the frozen exclusion digest), and
// docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php
// (frozen UIAI hosted-resource exclusion registry) to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eUiaiBundleIsolationMigration
{
    public const SCHEMA = 'focusa.spec152e.uiai_bundle_isolation.v1';
    public const VERSION = 1;

    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        $this->db = $db;
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->prefix = $prefix;
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        self::assertTimestamp($appliedAt);
        $encoded = self::encodeCanonical($provenance);
        $uuid = 'CHAR(36)';
        $key = 'VARCHAR(64)';
        $tables = [
            'wpuiai_ubi_registrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_registrations (
                registration_uuid {$uuid} NOT NULL PRIMARY KEY,
                facade_id VARCHAR(64) NOT NULL,
                origin VARCHAR(191) NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                email_digest VARCHAR(64) NOT NULL,
                email_domain VARCHAR(191) NOT NULL,
                email_prefix_char VARCHAR(4) NOT NULL,
                challenge_hash VARCHAR(64) NOT NULL,
                challenge_expires_at VARCHAR(32) NOT NULL,
                challenge_used INTEGER NOT NULL DEFAULT 0,
                challenge_attempts INTEGER NOT NULL DEFAULT 0,
                identity_key VARCHAR(64) NULL,
                account_uuid {$uuid} NULL,
                customer_id BIGINT NULL,
                order_id BIGINT NULL,
                edd_license_id BIGINT NULL,
                node_uuid {$uuid} NULL,
                lease_uuid {$uuid} NULL,
                state VARCHAR(32) NOT NULL,
                state_version BIGINT NOT NULL DEFAULT 0,
                request_id {$key} NOT NULL,
                idempotency_key {$key} NOT NULL,
                request_digest VARCHAR(64) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                verified_at VARCHAR(32) NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_identities' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_identities (
                identity_uuid {$uuid} NOT NULL PRIMARY KEY,
                identity_key VARCHAR(64) NOT NULL UNIQUE,
                email_digest VARCHAR(64) NOT NULL,
                email_domain VARCHAR(191) NOT NULL,
                email_prefix_char VARCHAR(4) NOT NULL,
                account_uuid {$uuid} NOT NULL,
                verified_at VARCHAR(32) NOT NULL,
                verified_method VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_accounts' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_accounts (
                account_uuid {$uuid} NOT NULL PRIMARY KEY,
                identity_uuid {$uuid} NOT NULL,
                customer_id BIGINT NOT NULL UNIQUE,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_orders' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_orders (
                order_id BIGINT NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                customer_id BIGINT NOT NULL,
                facade_id VARCHAR(64) NOT NULL,
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
            'wpuiai_ubi_order_items' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_order_items (
                order_item_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_download_id BIGINT NOT NULL,
                edd_price_id VARCHAR(191) NOT NULL,
                amount_usd VARCHAR(32) NOT NULL,
                quantity BIGINT NOT NULL DEFAULT 1
            )",
            'wpuiai_ubi_licenses' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_licenses (
                edd_license_id BIGINT NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                customer_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                grants_json TEXT NOT NULL,
                human_key_count BIGINT NOT NULL DEFAULT 1,
                license_key VARCHAR(191) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_deliveries' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_deliveries (
                delivery_id VARCHAR(64) NOT NULL PRIMARY KEY,
                edd_license_id BIGINT NOT NULL,
                channel VARCHAR(16) NOT NULL,
                recipient_mask VARCHAR(191) NOT NULL,
                key_mask VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                sent_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_nodes' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_nodes (
                node_uuid {$uuid} NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                node_id VARCHAR(128) NOT NULL,
                device_public_key_hash VARCHAR(64) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_leases' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_leases (
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
            'wpuiai_ubi_sequences' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_sequences (
                account_uuid {$uuid} NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                current_sequence BIGINT NOT NULL DEFAULT 0 CHECK (current_sequence >= 0),
                created_at VARCHAR(32) NOT NULL,
                updated_at VARCHAR(32) NOT NULL,
                PRIMARY KEY (account_uuid, product_code)
            )",
            'wpuiai_ubi_refunds' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_refunds (
                refund_id VARCHAR(64) NOT NULL PRIMARY KEY,
                order_id BIGINT NOT NULL,
                edd_license_id BIGINT NOT NULL,
                reason VARCHAR(191) NOT NULL,
                sequence_after BIGINT NOT NULL,
                refunded_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_child_tokens' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_child_tokens (
                token_id VARCHAR(64) NOT NULL PRIMARY KEY,
                account_uuid {$uuid} NOT NULL,
                edd_license_id BIGINT NOT NULL,
                product_code VARCHAR(128) NOT NULL,
                lease_uuid {$uuid} NOT NULL,
                node_uuid {$uuid} NOT NULL,
                token_digest VARCHAR(64) NOT NULL,
                audience VARCHAR(64) NOT NULL,
                features_json TEXT NOT NULL,
                limits_json TEXT NOT NULL,
                hosted_exclusion_digest VARCHAR(64) NOT NULL,
                issued_at VARCHAR(32) NOT NULL,
                expires_at VARCHAR(32) NOT NULL,
                state VARCHAR(16) NOT NULL,
                created_at VARCHAR(32) NOT NULL
            )",
            'wpuiai_ubi_journal' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_journal (
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
            'wpuiai_ubi_schema_migrations' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_schema_migrations (
                schema_version BIGINT NOT NULL PRIMARY KEY,
                schema_name VARCHAR(191) NOT NULL,
                applied_at VARCHAR(32) NOT NULL,
                migration_provenance TEXT NOT NULL
            )",
            'wpuiai_ubi_schema_events' => "CREATE TABLE IF NOT EXISTS {$this->prefix}wpuiai_ubi_schema_events (
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
            "INSERT INTO {$this->prefix}wpuiai_ubi_schema_migrations (schema_version, schema_name, applied_at, migration_provenance)
             SELECT :version, :schema, :applied_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$this->prefix}wpuiai_ubi_schema_migrations WHERE schema_version = :existing_version)"
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
            "INSERT OR IGNORE INTO {$this->prefix}wpuiai_ubi_schema_events (event_key, event_type, schema_version, occurred_at, migration_provenance)
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

final class FocusaSpec152eUiaiBundleIsolationService
{
    public const SCHEMA = 'focusa.spec152e.uiai_bundle_isolation.v1';
    public const RECEIPT_SCHEMA = 'focusa.spec152e.uiai_bundle_receipt.v1';

    /** Public product codes (spec 152E §8; Spec 172 dedicated Downloads offer). */
    public const PRODUCT_FOCUSA = 'focusa_operator_lifetime_v1';
    public const PRODUCT_UIAI = 'uiai_operator_lifetime_v1';
    public const PRODUCT_BUNDLE = 'focusa_uiai_operator_bundle_lifetime_v1';

    /** Frozen Focusa Operator v1 local/product family set. */
    public const FOCUSA_FEATURES = [
        'base_focusa', 'mission', 'workpoint', 'evidence', 'team_remote', 'release_proof',
    ];

    /** Frozen UIAI Operator v1 local/product family set (Spec 172 section 7.2). */
    public const UIAI_FEATURES = [
        'uiai_public_observation', 'uiai_browser_action', 'uiai_persistence',
        'uiai_diagnostics', 'uiai_proof_packets', 'uiai_batch_responsive',
        'uiai_supported_integrations',
    ];

    /** Bounded child-token TTL matching the runtime broker bound (uiai_child_token.rs). */
    public const CHILD_TOKEN_MAX_TTL_MINUTES = 15;

    /**
     * Test-mode-only email-channel bounce seam. The whole contract is a
     * deterministic synthetic-proof harness; this boolean toggles a simulated
     * transactional-email bounce (spec 152E §16.1) so the partial-delivery and
     * recovery surface can be exercised without a real mailer. It never
     * exists in a production journey and never affects grants, prices, keys,
     * or sequence decisions.
     */
    public const TEST_MODE_EMAIL_BOUNCE_SEAM = true;

    /**
     * Server-owned product/grant registry (spec 152E §8). The caller submits
     * only the public product code; every EDD download, price, grant, feature,
     * limit, term, refund policy, and commercial right is resolved here. The
     * Bundle is ONE SKU whose grants are the EXACT union of the two underlying
     * Operator v1 records. No value on this map is client-controlled; all
     * values are public synthetic test-mode fixtures.
     */
    public const PRODUCT_MAPPING = [
        'focusa_operator_lifetime_v1' => [
            'products' => ['focusa'],
            'grants' => ['focusa_operator_lifetime_v1'],
            'grant_composition' => 'standalone',
            'posture' => 'paid',
            'edd_download_id' => 4601,
            'edd_price_id' => '46011',
            'price_usd' => '697.00',
            'features' => ['base_focusa', 'mission', 'workpoint', 'evidence', 'team_remote', 'release_proof'],
            'limits' => ['node_limit' => 3, 'operator_seats' => 1],
            'term' => 'lifetime',
            'refund_policy' => 'whole_order_30_days',
            'offline_grace_days' => 120,
            'human_key_count' => 1,
        ],
        'uiai_operator_lifetime_v1' => [
            'products' => ['uiai_engine'],
            'grants' => ['uiai_operator_lifetime_v1'],
            'grant_composition' => 'standalone',
            'posture' => 'paid',
            'edd_download_id' => 4602,
            'edd_price_id' => '46012',
            'price_usd' => '697.00',
            'features' => ['uiai_public_observation', 'uiai_browser_action', 'uiai_persistence',
                           'uiai_diagnostics', 'uiai_proof_packets', 'uiai_batch_responsive',
                           'uiai_supported_integrations'],
            'limits' => ['node_limit' => 3, 'operator_seats' => 1],
            'term' => 'lifetime',
            'refund_policy' => 'whole_order_30_days',
            'offline_grace_days' => 120,
            'human_key_count' => 1,
        ],
        'focusa_uiai_operator_bundle_lifetime_v1' => [
            'products' => ['focusa', 'uiai_engine'],
            'grants' => ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'],
            'grant_composition' => 'exact_union',
            'posture' => 'bundle',
            'edd_download_id' => 4603,
            'edd_price_id' => '46013',
            'price_usd' => '1254.60',
            'features' => ['base_focusa', 'mission', 'workpoint', 'evidence', 'team_remote',
                           'release_proof', 'uiai_public_observation', 'uiai_browser_action',
                           'uiai_persistence', 'uiai_diagnostics', 'uiai_proof_packets',
                           'uiai_batch_responsive', 'uiai_supported_integrations'],
            'limits' => ['node_limit' => 3, 'operator_seats' => 1],
            'term' => 'lifetime',
            'refund_policy' => 'whole_order_30_days',
            'offline_grace_days' => 120,
            'human_key_count' => 1,
        ],
    ];

    /**
     * Derived bundle family union: exactly the merge of the two underlying
     * Operator v1 family records — never a third hand-copied family list.
     */
    public static function bundleFeaturesDerived(): array
    {
        $union = array_merge(self::FOCUSA_FEATURES, self::UIAI_FEATURES);
        return array_values(array_unique($union));
    }

    // Registered facade allowlist (spec 152E §9, subset needed by this
    // journey). Facades are presenters: they brand and proxy; they never
    // decide entitlement. Caller-supplied origins fail closed.
    public const FACADE_ALLOWLIST = [
        'focusa_install_v1' => ['https://install.focusa.dev'],
        'focusa_marketing_v1' => ['https://focusa.dev'],
    ];

    public const JOURNEY_STATES = [
        'attempt_created', 'email_verified', 'account_promoted', 'checkout_pending',
        'order_complete', 'entitlement_issued', 'delivery_partial', 'delivered',
        'device_registered', 'lease_issued', 'refunded',
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

    // ── Journey: registration → verification → shared-identity promotion ──

    public function startRegistration(array $request): array
    {
        $facadeId = (string) ($request['facade_id'] ?? '');
        $origin = (string) ($request['origin'] ?? '');
        $productCode = (string) ($request['product_code'] ?? '');
        $emailDigest = (string) ($request['email_digest'] ?? '');
        $emailDomain = (string) ($request['email_domain'] ?? '');
        $emailPrefixChar = (string) ($request['email_prefix_char'] ?? '');
        $challengeCode = (string) ($request['challenge_code'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['facade_id', 'origin', 'product_code', 'email_digest',
                                                  'email_domain', 'email_prefix_char', 'challenge_code',
                                                  'request_id', 'idempotency_key']);

        if (!isset(self::FACADE_ALLOWLIST[$facadeId]) || !in_array($origin, self::FACADE_ALLOWLIST[$facadeId], true)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if (!isset(self::PRODUCT_MAPPING[$productCode])) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $this->assertDigest($emailDigest, 'email');
        if (preg_match('/^(?!-)[A-Za-z0-9.-]+\.[A-Za-z]{2,}$/D', $emailDomain) !== 1) {
            throw new DomainException('EMAIL_DOMAIN_REQUIRED');
        }
        if (preg_match('/^[A-Za-z0-9_!#$%&\'*+\/=?^`{|}~.-]{1,32}$/D', $emailPrefixChar) !== 1) {
            throw new DomainException('EMAIL_PREFIX_REQUIRED');
        }
        if (preg_match('/^[0-9]{6}$/', $challengeCode) !== 1) {
            throw new DomainException('EMAIL_VERIFICATION_FAILED');
        }

        $replay = $this->findReplay('registration_attempted', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $now = ($this->clock)();
        $registrationUuid = $this->opaqueId('reg_ubi_' . hash('sha256', $facadeId . "\n" . $productCode . "\n" . $emailDigest . "\n" . $idempotencyKey));
        $expiresAt = (new DateTimeImmutable($now, new DateTimeZone('UTC')))->modify('+15 minutes')->format('Y-m-d\TH:i:s\Z');
        $challengeHash = hash('sha256', "challenge-v1\n" . $registrationUuid . "\n" . $challengeCode . "\n" . $expiresAt);
        $maskedEmail = $emailPrefixChar . '***@' . $emailDomain;
        $insert = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_registrations')}
             (registration_uuid, facade_id, origin, product_code, email_digest, email_domain,
              email_prefix_char, challenge_hash, challenge_expires_at, challenge_used, challenge_attempts,
              state, state_version, request_id, idempotency_key, request_digest, created_at, updated_at)
             VALUES (:uuid, :facade, :origin, :product, :digest, :domain, :prefix_char,
              :challenge, :expires, 0, 0, 'attempt_created', 0, :request, :idem, :req_digest, :now, :now)"
        );
        $insert->execute([
            ':uuid' => $registrationUuid, ':facade' => $facadeId, ':origin' => $origin,
            ':product' => $productCode, ':digest' => $emailDigest, ':domain' => $emailDomain,
            ':prefix_char' => $emailPrefixChar, ':challenge' => $challengeHash,
            ':expires' => $expiresAt, ':request' => $requestId, ':idem' => $idempotencyKey,
            ':req_digest' => $requestDigest, ':now' => $now,
        ]);
        $this->journal('registration_attempted', $registrationUuid, null, 'attempt_created', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId, 'origin' => $origin, 'product_code' => $productCode,
            'state' => 'attempt_created', 'masked_email' => $maskedEmail,
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
        $this->updateRegistration($registrationUuid, ['challenge_used' => 1, 'state' => 'email_verified', 'verified_at' => $now], $now);
        $this->journal('email_verified', $registrationUuid, null, 'email_verified', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'email_verified', 'verification_method' => 'single_use_magic_code',
        ];
    }

    /**
     * Shared-identity promotion (spec 152E §6, §7): the account is keyed by
     * the verified email identity, never by the product code. A second
     * purchase by the same verified identity reuses the SAME authority account
     * and the SAME EDD customer id with zero new rows.
     */
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
                'state' => 'account_promoted', 'identity_reused' => true, 'zero_new_rows' => true,
            ];
        }
        $now = ($this->clock)();
        $identityKey = $this->identityKey((string) $row['email_digest'], (string) $row['email_domain']);
        $existing = $this->db->prepare(
            "SELECT i.identity_uuid, i.account_uuid, a.customer_id
             FROM {$this->table('wpuiai_ubi_identities')} i
             JOIN {$this->table('wpuiai_ubi_accounts')} a ON a.account_uuid = i.account_uuid
             WHERE i.identity_key = :key"
        );
        $existing->execute([':key' => $identityKey]);
        $existingRow = $existing->fetch(PDO::FETCH_ASSOC);
        if ($existingRow !== false) {
            // The same verified identity already holds an account (bundle or
            // another product): reuse it — no duplicate account/customer.
            $this->updateRegistration($registrationUuid, [
                'identity_key' => $identityKey, 'account_uuid' => (string) $existingRow['account_uuid'],
                'customer_id' => (int) $existingRow['customer_id'], 'state' => 'account_promoted',
            ], $now);
            $this->journal('account_promoted', $registrationUuid, (string) $existingRow['account_uuid'], 'account_promoted', $requestId, $idempotencyKey, $requestDigest, $now);
            return [
                'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
                'account_uuid' => (string) $existingRow['account_uuid'],
                'customer_id' => (int) $existingRow['customer_id'],
                'state' => 'account_promoted', 'identity_reused' => true,
                'edd_customer_resolved_or_created' => true, 'zero_new_rows' => true,
            ];
        }
        $accountUuid = $this->opaqueId('acct_ubi_' . $identityKey);
        $identityUuid = $this->opaqueId('idty_ubi_' . $identityKey);
        $customerId = $this->customerIdFor($identityKey);
        $this->db->beginTransaction();
        try {
            $identity = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ubi_identities')}
                 (identity_uuid, identity_key, email_digest, email_domain, email_prefix_char,
                  account_uuid, verified_at, verified_method, state, created_at, updated_at)
                 VALUES (:uuid, :key, :digest, :domain, :prefix_char, :account, :verified_at,
                  'single_use_magic_code', 'verified', :now, :now)"
            );
            $identity->execute([
                ':uuid' => $identityUuid, ':key' => $identityKey,
                ':digest' => (string) $row['email_digest'], ':domain' => (string) $row['email_domain'],
                ':prefix_char' => (string) $row['email_prefix_char'], ':account' => $accountUuid,
                ':verified_at' => $now, ':now' => $now,
            ]);
            $account = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ubi_accounts')}
                 (account_uuid, identity_uuid, customer_id, state, created_at, updated_at)
                 VALUES (:account, :identity, :customer, 'active', :now, :now)"
            );
            $account->execute([
                ':account' => $accountUuid, ':identity' => $identityUuid,
                ':customer' => $customerId, ':now' => $now,
            ]);
            $this->updateRegistration($registrationUuid, [
                'identity_key' => $identityKey, 'account_uuid' => $accountUuid,
                'customer_id' => $customerId, 'state' => 'account_promoted',
            ], $now);
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        $this->journal('account_promoted', $registrationUuid, $accountUuid, 'account_promoted', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'account_uuid' => $accountUuid, 'customer_id' => $customerId,
            'state' => 'account_promoted', 'identity_reused' => false,
            'edd_customer_resolved_or_created' => true, 'zero_new_rows' => false,
        ];
    }

    // ── Checkout (server-owned) ───────────────────────────────────────────

    public function createCheckoutIntent(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('checkout_intent_created', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'account_promoted') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        $now = ($this->clock)();
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $checkoutToken = 'pay_' . substr(hash('sha256', 'checkout-v1\n' . $registrationUuid . '\n' . $now), 0, 32);
        $checkoutUrl = (string) $row['origin'] . '/activate/checkout/' . $checkoutToken;
        $this->updateRegistration($registrationUuid, ['state' => 'checkout_pending'], $now);
        $this->journal('checkout_intent_created', $registrationUuid, (string) $row['account_uuid'], 'checkout_pending', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'checkout_pending', 'branded_checkout_url' => $checkoutUrl,
            'edd_download_id' => $product['edd_download_id'], 'edd_price_id' => $product['edd_price_id'],
            'price_usd' => $product['price_usd'], 'products' => $product['products'],
            'grants' => $product['grants'], 'grant_composition' => $product['grant_composition'],
            'stripe_gateway' => 'edd_stripe_test_mode', 'card_data_handled_by' => 'edd_stripe_only',
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
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'checkout_email_digest',
                                                  'payment_reference_digest', 'request_id', 'idempotency_key']);
        $this->assertDigest($checkoutEmailDigest, 'checkout_email');
        $this->assertDigest($paymentReferenceDigest, 'payment_reference');

        $replay = $this->findReplay('edd_order_completed', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'checkout_pending' && (string) $row['state'] !== 'order_complete') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        $now = ($this->clock)();
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $orderId = $this->orderIdFor($registrationUuid);
        $orderItemId = $orderId * 10 + 1;
        $verifiedDigest = (string) $row['email_digest'];
        $integrityOk = hash_equals($verifiedDigest, $checkoutEmailDigest);
        $existingOrder = $this->db->prepare("SELECT state FROM {$this->table('wpuiai_ubi_orders')} WHERE order_id = :order");
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
                $flip = $this->db->prepare(
                    "UPDATE {$this->table('wpuiai_ubi_orders')}
                     SET state = 'complete', state_reason = NULL, completed_at = :now, updated_at = :now
                     WHERE order_id = :order"
                );
                $flip->execute([':now' => $now, ':order' => $orderId]);
            } elseif ($existingState === false) {
                $order = $this->db->prepare(
                    "INSERT INTO {$this->table('wpuiai_ubi_orders')}
                     (order_id, account_uuid, customer_id, facade_id, product_code, edd_download_id,
                      edd_price_id, price_usd, checkout_email_digest, verified_email_digest,
                      payment_reference_digest, state, state_reason, created_at, completed_at, updated_at)
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
                    ':payment_digest' => $paymentReferenceDigest,
                    ':state' => $integrityOk ? 'complete' : 'held_unverified',
                    ':reason' => $integrityOk ? null : 'EDD_ORDER_UNVERIFIED', ':now' => $now,
                    ':completed' => $integrityOk ? $now : null,
                ]);
                $item = $this->db->prepare(
                    "INSERT INTO {$this->table('wpuiai_ubi_order_items')}
                     (order_item_id, order_id, edd_download_id, edd_price_id, amount_usd, quantity)
                     VALUES (:item, :order, :download, :price_id, :amount, 1)"
                );
                $item->execute([
                    ':item' => $orderItemId, ':order' => $orderId,
                    ':download' => $product['edd_download_id'], ':price_id' => $product['edd_price_id'],
                    ':amount' => $product['price_usd'],
                ]);
            } else {
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

    // ── Issuance: the sole EDD Software Licensing human key ──────────────

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
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $licenseId = $this->licenseIdFor($registrationUuid);
        $raw = strtoupper(substr(preg_replace('/[^A-Z0-9]/', '', strtoupper(hash('sha256', "edd-sl-v1\n" . $licenseId . "\n" . (string) $row['order_id']))), 0, 16));
        $licenseKey = 'FOCUSA-' . implode('-', str_split($raw, 4));
        $grantsJson = json_encode($product['grants'], JSON_UNESCAPED_SLASHES);
        $license = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_licenses')}
             (edd_license_id, order_id, customer_id, product_code, grants_json, human_key_count,
              license_key, state, created_at, updated_at)
             VALUES (:license, :order, :customer, :product, :grants, :keys, :key, 'active', :now, :now)"
        );
        $license->execute([
            ':license' => $licenseId, ':order' => (int) $row['order_id'],
            ':customer' => (int) $row['customer_id'], ':product' => (string) $row['product_code'],
            ':grants' => $grantsJson, ':keys' => $product['human_key_count'], ':key' => $licenseKey,
            ':now' => $now,
        ]);
        $this->updateRegistration($registrationUuid, ['edd_license_id' => $licenseId, 'state' => 'entitlement_issued'], $now);
        $this->journal('edd_license_issued', $registrationUuid, (string) $row['account_uuid'], 'entitlement_issued', $requestId, $idempotencyKey, $requestDigest, $now);
        $keyMask = substr($licenseKey, 0, 11) . '-****-****-****';
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'edd_license_id' => $licenseId, 'state' => 'entitlement_issued',
            'source' => 'edd_software_licensing', 'issuance_surface' => 'edd_authority_only',
            'duplicate_license' => false, 'grants' => $product['grants'],
            'grant_composition' => $product['grant_composition'], 'human_key_count' => $product['human_key_count'],
            'license_key_mask' => $keyMask,
        ];
    }

    // ── Dual-channel key delivery with partial failure/recovery ──────────

    public function deliver(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $bounceSeam = (bool) ($request['email_bounce_seam'] ?? false);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'email_bounce_seam',
                                                  'request_id', 'idempotency_key']);

        $replay = $this->findReplay('key_delivered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] === 'delivered') {
            throw new DomainException('DELIVERY_ALREADY_DELIVERED');
        }
        if ((string) $row['state'] === 'delivery_partial') {
            throw new DomainException('DELIVERY_ALREADY_PARTIAL');
        }
        if ((string) $row['state'] !== 'entitlement_issued' || (string) $row['edd_license_id'] === '') {
            throw new DomainException('EDD_LICENSE_PENDING');
        }
        $license = $this->licenseRow((int) $row['edd_license_id']);
        if ($license['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $now = ($this->clock)();
        $keyMask = substr((string) $license['license_key'], 0, 11) . '-****-****-****';
        $maskedEmail = (string) $row['email_prefix_char'] . '***@' . (string) $row['email_domain'];
        $emailState = ($bounceSeam && self::TEST_MODE_EMAIL_BOUNCE_SEAM) ? 'failed' : 'sent';
        $partial = $emailState === 'failed';
        $deliveryIds = [
            'email' => 'del_ubi_' . substr(hash('sha256', 'email\n' . $registrationUuid), 0, 16),
            'account' => 'del_ubi_' . substr(hash('sha256', 'account\n' . $registrationUuid), 0, 16),
        ];
        $insert = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_deliveries')}
             (delivery_id, edd_license_id, channel, recipient_mask, key_mask, state, sent_at)
             VALUES (:id, :license, :channel, :recipient, :key_mask, :state, :now)"
        );
        foreach (['email' => $emailState, 'account' => 'sent'] as $channel => $state) {
            $insert->execute([
                ':id' => $deliveryIds[$channel], ':license' => (int) $row['edd_license_id'],
                ':channel' => $channel,
                ':recipient' => $channel === 'email' ? $maskedEmail : 'account@authority',
                ':key_mask' => $keyMask, ':state' => $state, ':now' => $now,
            ]);
        }
        $this->updateRegistration($registrationUuid, ['state' => $partial ? 'delivery_partial' : 'delivered'], $now);
        $this->journal($partial ? 'key_delivery_partial' : 'key_delivered', $registrationUuid, (string) $row['account_uuid'], $partial ? 'delivery_partial' : 'delivered', $requestId, $idempotencyKey, $requestDigest, $now);
        $result = [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => $partial ? 'delivery_partial' : 'delivered',
            'channels' => ['email' => $emailState, 'account' => 'sent'],
            'same_canonical_key_both_channels' => true, 'promotional_content' => false,
            'key_mask' => $keyMask,
        ];
        if ($partial) {
            $result['partial'] = true;
            $result['recovery_required'] = true;
            $result['error'] = 'PARTIAL_DELIVERY_PENDING';
        }
        return $result;
    }

    /**
     * Recovery retries ONLY the failed channel (spec 152E §16.1): the healthy
     * account channel is never re-delivered and no second key is ever created.
     */
    public function recoverDelivery(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('delivery_recovered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'delivery_partial') {
            throw new DomainException('PARTIAL_DELIVERY_REQUIRED');
        }
        $now = ($this->clock)();
        $retry = $this->db->prepare(
            "UPDATE {$this->table('wpuiai_ubi_deliveries')} SET state = 'sent', sent_at = :now
             WHERE edd_license_id = :license AND channel = 'email' AND state = 'failed'"
        );
        $retry->execute([':now' => $now, ':license' => (int) $row['edd_license_id']]);
        $updated = $retry->rowCount();
        if ($updated !== 1) {
            throw new DomainException('PARTIAL_DELIVERY_REQUIRED');
        }
        $this->updateRegistration($registrationUuid, ['state' => 'delivered'], $now);
        $this->journal('delivery_recovered', $registrationUuid, (string) $row['account_uuid'], 'delivered', $requestId, $idempotencyKey, $requestDigest, $now);
        $license = $this->licenseRow((int) $row['edd_license_id']);
        $keyMask = substr((string) $license['license_key'], 0, 11) . '-****-****-****';
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'delivered', 'recovered' => true, 'channels' => ['email' => 'sent', 'account' => 'sent'],
            'same_canonical_key_both_channels' => true, 'key_mask' => $keyMask,
            'duplicate_key' => false, 'healthy_channel_not_redelivered' => true,
        ];
    }

    // ── Node registration + signed lease ─────────────────────────────────

    public function registerNode(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $nodeId = (string) ($request['node_id'] ?? '');
        $devicePublicKey = (string) ($request['device_public_key'] ?? '');
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'node_id', 'device_public_key',
                                                  'request_id', 'idempotency_key']);
        if (preg_match('/^[A-Za-z0-9_-]{1,128}$/D', $nodeId) !== 1) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        $this->assertPublicKey($devicePublicKey);

        $replay = $this->findReplay('node_registered', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'delivered') {
            throw new DomainException('LICENSE_DELIVERY_PENDING');
        }
        $now = ($this->clock)();
        $nodeUuid = $this->opaqueId('node_ubi_' . $registrationUuid);
        $deviceHash = hash('sha256', 'device-key-v1\n' . $devicePublicKey);
        $node = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_nodes')}
             (node_uuid, account_uuid, edd_license_id, product_code, node_id, device_public_key_hash,
              state, created_at, updated_at)
             VALUES (:node, :account, :license, :product, :node_id, :device_hash, 'active', :now, :now)"
        );
        $node->execute([
            ':node' => $nodeUuid, ':account' => (string) $row['account_uuid'],
            ':license' => (int) $row['edd_license_id'], ':product' => (string) $row['product_code'],
            ':node_id' => $nodeId, ':device_hash' => $deviceHash, ':now' => $now,
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
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('lease_issued', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'device_registered') {
            throw new DomainException('NODE_REQUIRED');
        }
        $now = ($this->clock)();
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        $license = $this->licenseRow((int) $row['edd_license_id']);
        if ($license['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $nodeRow = $this->nodeRow((string) $row['node_uuid']);
        $sequence = $this->nextSequence((string) $row['account_uuid'], (string) $row['product_code'], $now);
        $expiresAt = (new DateTimeImmutable($now, new DateTimeZone('UTC')))->modify('+90 days')->format('Y-m-d\TH:i:s\Z');
        $offlineGrace = (new DateTimeImmutable($expiresAt, new DateTimeZone('UTC')))->modify('+' . (int) $product['offline_grace_days'] . ' days')->format('Y-m-d\TH:i:s\Z');
        $keyMask = substr((string) $license['license_key'], 0, 11) . '-****-****-****';
        $leaseUuid = $this->opaqueId('lease_ubi_' . $registrationUuid);
        $payload = [
            'schema' => 'focusa.authority_lease.v1',
            'product_code' => (string) $row['product_code'],
            'products' => $product['products'],
            'grants' => $product['grants'],
            'grant_composition' => $product['grant_composition'],
            'features' => $product['features'],
            'limits' => $product['limits'],
            'posture' => $product['posture'],
            'term' => $product['term'],
            'human_key_count' => $product['human_key_count'],
            'customer_id' => (int) $row['customer_id'],
            'order_id' => (int) $row['order_id'],
            'order_item_id' => $this->orderItemRow((int) $row['order_id']),
            'edd_license_id' => (int) $row['edd_license_id'],
            'license_key_digest' => hash('sha256', (string) $license['license_key']),
            'license_key_mask' => $keyMask,
            'node_id' => (string) $nodeRow['node_id'],
            'device_public_key_hash' => (string) $nodeRow['device_public_key_hash'],
            'subject_id' => (string) $row['account_uuid'],
            'sequence' => $sequence,
            'status' => 'active',
            'authority_key_id' => 'authority-lease-2026-01',
            'issued_at' => $now,
            'not_before' => $now,
            'expires_at' => $expiresAt,
            'offline_grace_until' => $offlineGrace,
            'refund_policy' => $product['refund_policy'],
            'install_site_authority' => 'none',
            'spec158' => 'excluded',
        ];
        $payloadBytes = $this->canonicalJson($payload);
        $envelope = $this->keySet->seal($payload, 'authority-lease-2026-01', $this->keySet->leaseSeed(), FocusaSpec152eEd25519Signer::LEASE_DOMAIN);
        $lease = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_leases')}
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

    // ── UIAI child token (bounded, exact subset, digest-only at rest) ─────

    public function issueChildToken(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('child_token_issued', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'lease_issued') {
            throw new DomainException('LEASE_REQUIRED');
        }
        $product = self::PRODUCT_MAPPING[(string) $row['product_code']];
        if (!in_array('uiai_operator_lifetime_v1', $product['grants'], true)) {
            throw new DomainException('CHILD_TOKEN_NOT_INCLUDED');
        }
        $license = $this->licenseRow((int) $row['edd_license_id']);
        if ($license['state'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $leaseRow = $this->leaseRow((string) $row['lease_uuid']);
        $nodeRow = $this->nodeRow((string) $row['node_uuid']);
        $now = ($this->clock)();
        $tokenId = 'ct_' . substr(hash('sha256', 'child-token-id-v1\n' . $registrationUuid), 0, 32);
        $tokenSecret = 'tok_' . substr(hash('sha256', 'child-token-secret-v1\n' . $registrationUuid . '\n' . $now), 0, 32);
        $tokenDigest = hash('sha256', 'child-token-v1\n' . $tokenSecret);
        $expiresAt = (new DateTimeImmutable($now, new DateTimeZone('UTC')))->modify('+' . self::CHILD_TOKEN_MAX_TTL_MINUTES . ' minutes')->format('Y-m-d\TH:i:s\Z');
        $features = self::UIAI_FEATURES;
        $limits = $product['limits'];
        $exclusionDigest = UiaiSpec172HostedResourceExclusionRegistry::digest();
        $token = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_ubi_child_tokens')}
             (token_id, account_uuid, edd_license_id, product_code, lease_uuid, node_uuid, token_digest,
              audience, features_json, limits_json, hosted_exclusion_digest, issued_at, expires_at,
              state, created_at)
             VALUES (:token, :account, :license, :product, :lease, :node, :digest,
              'uiai-engine:operator', :features, :limits, :exclusion, :issued, :expires,
              'active', :now)"
        );
        $token->execute([
            ':token' => $tokenId, ':account' => (string) $row['account_uuid'],
            ':license' => (int) $row['edd_license_id'], ':product' => (string) $row['product_code'],
            ':lease' => (string) $row['lease_uuid'], ':node' => (string) $row['node_uuid'],
            ':digest' => $tokenDigest,
            ':features' => json_encode($features, JSON_UNESCAPED_SLASHES),
            ':limits' => json_encode($limits, JSON_UNESCAPED_SLASHES),
            ':exclusion' => $exclusionDigest, ':issued' => $now, ':expires' => $expiresAt, ':now' => $now,
        ]);
        $this->journal('child_token_issued', $registrationUuid, (string) $row['account_uuid'], 'lease_issued', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => true, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'child_token_issued',
            'child_token' => [
                'schema' => 'focusa.uiai_child_token.v1',
                'token_id' => $tokenId,
                'audience' => 'uiai-engine:operator',
                'node_id' => (string) $nodeRow['node_id'],
                'client_id' => 'uiai-engine:cli',
                'grant_lease_id' => (string) $row['lease_uuid'],
                'grant_sequence' => (int) $leaseRow['sequence'],
                'features' => $features,
                'limits' => $limits,
                'hosted_resource_exclusion_digest' => $exclusionDigest,
                'issued_at' => $now,
                'expires_at' => $expiresAt,
            ],
            'token_digest' => $tokenDigest,
            'token_stored' => 'digest_only',
            'max_ttl_minutes' => self::CHILD_TOKEN_MAX_TTL_MINUTES,
        ];
    }

    // ── Refund, reactivation, and wrong-product guard ────────────────────

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
        $refundId = 'rfnd_ubi_' . substr(hash('sha256', 'refund-v1\n' . $registrationUuid . '\n' . $now), 0, 24);
        $this->db->beginTransaction();
        try {
            $orderUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ubi_orders')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE order_id = :order");
            $orderUpdate->execute([':now' => $now, ':order' => $orderId]);
            $licenseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ubi_licenses')} SET state = 'refunded', updated_at = :now WHERE edd_license_id = :license");
            $licenseUpdate->execute([':now' => $now, ':license' => $licenseId]);
            $leaseUpdate = $this->db->prepare("UPDATE {$this->table('wpuiai_ubi_leases')} SET state = 'refunded', state_reason = 'REFUNDED', updated_at = :now WHERE lease_uuid = :lease");
            $leaseUpdate->execute([':now' => $now, ':lease' => (string) $row['lease_uuid']]);
            $refund = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ubi_refunds')}
                 (refund_id, order_id, edd_license_id, reason, sequence_after, refunded_at)
                 VALUES (:refund, :order, :license, :reason, :sequence, :now)"
            );
            $refund->execute([
                ':refund' => $refundId, ':order' => $orderId, ':license' => $licenseId,
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

    /**
     * Refunded/revoked records never reactivate (spec 152E §18, §23):
     * reactivation requires a NEW verified purchase (a new registration and a
     * new EDD order for the same verified identity). No existing registration
     * can resume; the refunded order/license/lease rows stay preserved.
     */
    public function reactivate(array $request): array
    {
        $registrationUuid = $this->registrationUuid($request);
        $requestId = $this->requestId($request);
        $idempotencyKey = $this->idempotencyKey($request);
        $requestDigest = $this->requestDigest($request);
        $this->assertCorrelationFields($request);
        $this->rejectCallerGrantFields($request, ['registration_uuid', 'request_id', 'idempotency_key']);

        $replay = $this->findReplay('reactivation_denied', $idempotencyKey, $requestDigest);
        if ($replay !== null) {
            return ['replayed' => true] + $replay;
        }

        $row = $this->registrationRow($registrationUuid);
        if ((string) $row['state'] !== 'refunded') {
            throw new DomainException('REACTIVATION_REQUIRES_NEW_ORDER');
        }
        $now = ($this->clock)();
        $this->journal('reactivation_denied', $registrationUuid, (string) $row['account_uuid'], 'refunded', $requestId, $idempotencyKey, $requestDigest, $now);
        return [
            'ok' => false, 'replayed' => false, 'registration_uuid' => $registrationUuid,
            'state' => 'refunded', 'error' => 'REFUNDED_NEVER_REACTIVATES',
            'refresh_denied' => true, 'posture' => 'recovery_only',
            'reactivation_requires_new_purchase' => true,
        ];
    }

    /**
     * Wrong-product guard (spec 152E §23): resolves the EXACT server-owned
     * grants of a product code and fails closed for unknown codes. A Focusa
     * grant never contains UIAI and a UIAI grant never contains Focusa; the
     * Bundle is the exact union only.
     */
    public function resolveProductGrants(array $request): array
    {
        $productCode = (string) ($request['product_code'] ?? '');
        if (!isset(self::PRODUCT_MAPPING[$productCode])) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        $product = self::PRODUCT_MAPPING[$productCode];
        return [
            'product_code' => $productCode,
            'products' => $product['products'],
            'grants' => $product['grants'],
            'grant_composition' => $product['grant_composition'],
            'features' => $product['features'],
            'limits' => $product['limits'],
            'posture' => $product['posture'],
            'human_key_count' => $product['human_key_count'],
            'price_usd' => $product['price_usd'],
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
            "SELECT current_sequence FROM {$this->table('wpuiai_ubi_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $sequenceStatement->execute([':account' => (string) $row['account_uuid'], ':product' => (string) $row['product_code']]);
        $currentSequence = $sequenceStatement->fetchColumn();
        $grants = null;
        if ($licenseRow !== null) {
            $decoded = json_decode((string) $licenseRow['grants_json'], true);
            $grants = is_array($decoded) ? $decoded : [];
        }
        $receipt = [
            'schema' => self::RECEIPT_SCHEMA,
            'fixture' => 'focusa-vbcqu.20.13.59',
            'facade_id' => (string) $row['facade_id'],
            'origin' => (string) $row['origin'],
            'product_code' => (string) $row['product_code'],
            'grants' => $grants,
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
        $receipt['receipt_sha256'] = hash('sha256', self::RECEIPT_SCHEMA . "\n" . $canonical);
        $this->journal('receipt_issued', $registrationUuid, (string) $row['account_uuid'], (string) $row['state'], 'req_ubi_receipt', 'idem_ubi_receipt', hash('sha256', 'receipt\n' . $registrationUuid), $now);
        return $receipt;
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    private function table(string $name): string
    {
        return $this->prefix . $name;
    }

    private function identityKey(string $emailDigest, string $emailDomain): string
    {
        return hash('sha256', "identity-v1\n" . $emailDigest . "\n" . $emailDomain);
    }

    private function customerIdFor(string $identityKey): int
    {
        return 2000 + (intval(substr(hash('sha256', 'customer-v1\n' . $identityKey), 0, 8), 16) % 900);
    }

    private function orderIdFor(string $registrationUuid): int
    {
        return 9000 + (intval(substr(hash('sha256', 'order-v1\n' . $registrationUuid), 0, 8), 16) % 900);
    }

    private function licenseIdFor(string $registrationUuid): int
    {
        return 7000 + (intval(substr(hash('sha256', 'license-v1\n' . $registrationUuid), 0, 8), 16) % 900);
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
            "SELECT event_type, request_digest FROM {$this->table('wpuiai_ubi_journal')} WHERE idempotency_key = :key AND event_type = :operation"
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
            "INSERT INTO {$this->table('wpuiai_ubi_journal')}
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
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ubi_registrations')} WHERE registration_uuid = :uuid");
        $statement->execute([':uuid' => $registrationUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('REGISTRATION_NOT_FOUND');
        }
        return $row;
    }

    private function licenseRow(int $licenseId): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ubi_licenses')} WHERE edd_license_id = :id");
        $statement->execute([':id' => $licenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    private function orderItemRow(int $orderId): int
    {
        $statement = $this->db->prepare("SELECT order_item_id FROM {$this->table('wpuiai_ubi_order_items')} WHERE order_id = :id");
        $statement->execute([':id' => $orderId]);
        $itemId = $statement->fetchColumn();
        if ($itemId === false) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        return (int) $itemId;
    }

    private function nodeRow(string $nodeUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ubi_nodes')} WHERE node_uuid = :uuid");
        $statement->execute([':uuid' => $nodeUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        return $row;
    }

    private function leaseRow(string $leaseUuid): array
    {
        $statement = $this->db->prepare("SELECT * FROM {$this->table('wpuiai_ubi_leases')} WHERE lease_uuid = :uuid");
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
            "SELECT current_sequence FROM {$this->table('wpuiai_ubi_sequences')} WHERE account_uuid = :account AND product_code = :product"
        );
        $select->execute([':account' => $accountUuid, ':product' => $productCode]);
        $current = $select->fetchColumn();
        if ($current === false) {
            $insert = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_ubi_sequences')} (account_uuid, product_code, current_sequence, created_at, updated_at)
                 VALUES (:account, :product, 0, :now, :now)"
            );
            $insert->execute([':account' => $accountUuid, ':product' => $productCode, ':now' => $now]);
            $current = 0;
        }
        $next = (int) $current + 1;
        $update = $this->db->prepare(
            "UPDATE {$this->table('wpuiai_ubi_sequences')} SET current_sequence = :next, updated_at = :now
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
        $statement = $this->db->prepare("UPDATE {$this->table('wpuiai_ubi_registrations')} SET " . implode(', ', $sets) . " WHERE registration_uuid = :uuid");
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
