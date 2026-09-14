<?php
// Spec 152E branded EDD checkout intent. The activation.checkout surface creates exactly
// one server-owned EDD checkout intent for a mailbox-verified, promoted registration and
// returns a branded facade checkout URL:
//
//   - The registration must be mailbox-verified, live, promoted (authority account + EDD
//     customer bound), and facade-bound before any intent is created. No unverified-email
//     promotion, no local/self-issued entitlement, and no independent facade authority.
//   - The product is resolved exclusively from the registration's product code through
//     the server-owned product registry (operator-approved, checkout-enabled mapping with
//     exact download and price). Clients never submit a product, price, amount, grant,
//     feature, limit, or commercial right; such fields fail closed.
//   - The branded facade URL is composed only of the facade's exact origin plus
//     allowlisted named paths from the facade return-handle registry. Caller-supplied
//     callback/redirect/success/cancel URLs and arbitrary handles are never honored.
//   - Exactly one intent is created per promoted registration/product. Replaying the
//     canonical request (same idempotency key) returns the same intent; a repeated
//     canonical request with a new idempotency key returns the existing intent instead of
//     creating a second one. Idempotency-key reuse with a different request fails with
//     IDEMPOTENCY_CONFLICT.
//   - Each intent binds registration, account, EDD customer, product, node request
//     (node UUID plus device-public-key hash), a server-owned EDD cart session
//     (synthetic fixture), and the idempotency journal. No raw email, raw device key,
//     license key, or secret is stored or returned.
//
// Failures are public-safe stable codes. The EDD cart/session adapter and the facade
// return-handle registry are separate surfaces in this contract. Issuance and EDD order
// completion stay deferred to their own atoms; this surface records the intent journal
// only. Rollback is preservation-only.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-account-promotion.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php,
// docs/contracts/spec152e-edd-product-registry.v1.php,
// docs/contracts/spec152e-facade-registry.v1.php, and
// docs/contracts/spec152e-verified-registration-token-validator.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddCheckoutIntentMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_checkout_intent.v1';
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
        $carts = $this->table('wpuiai_edd_checkout_cart_sessions');
        $intents = $this->table('wpuiai_edd_checkout_intents');
        $migrations = $this->table('wpuiai_edd_checkout_intent_schema_migrations');
        $events = $this->table('wpuiai_edd_checkout_intent_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$carts} (
            cart_reference VARCHAR(64) NOT NULL PRIMARY KEY,
            session_key VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            facade_id VARCHAR(96) NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            edd_download_id BIGINT NOT NULL,
            edd_price_id VARCHAR(191) NOT NULL,
            price_usd VARCHAR(32) NOT NULL,
            state VARCHAR(32) NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_checkout_cart_registration
            ON {$carts} (registration_uuid, product_code, state)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_checkout_cart_idempotency
            ON {$carts} (idempotency_key)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$intents} (
            intent_id VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            edd_customer_id BIGINT NOT NULL,
            facade_id VARCHAR(96) NOT NULL,
            origin VARCHAR(191) NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            edd_download_id BIGINT NOT NULL,
            edd_price_id VARCHAR(191) NOT NULL,
            price_usd VARCHAR(32) NOT NULL,
            node_uuid {$uuid} NULL,
            device_public_key_hash VARCHAR(64) NULL,
            cart_reference VARCHAR(64) NOT NULL,
            session_key VARCHAR(64) NOT NULL,
            return_handle VARCHAR(64) NOT NULL,
            return_url VARCHAR(512) NOT NULL,
            branded_checkout_url VARCHAR(512) NOT NULL,
            state VARCHAR(32) NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            settled_at VARCHAR(32) NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_checkout_intent_registration
            ON {$intents} (registration_uuid, product_code, state)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_checkout_intent_idempotency
            ON {$intents} (idempotency_key)");
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
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: intent and cart journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_checkout_intent_schema_events');
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

/**
 * Facade return-handle registry: resolves a named allowlisted return handle into the
 * facade's exact-origin branded path and composes the branded EDD checkout URL. The
 * allowlist comes only from the facade registry callbacks/paths; caller-supplied URLs and
 * arbitrary handles are never honored (FACADE_REDIRECT_DENIED). Facades are presenters
 * only: they never issue entitlement or own customer/commerce truth.
 */
final class FocusaSpec152eFacadeReturnHandleRegistry
{
    public const SCHEMA = 'focusa.spec152e.facade_return_handle_registry.v1';

    public function __construct(private array $facadeRegistry)
    {
    }

    /**
     * Resolve a named return handle to the facade's allowlisted branded return URL.
     *
     * Required input: facade_id, origin, return_handle (a named handle from the facade
     * registry callbacks, e.g. success/cancel/recovery).
     * Forbidden input: any caller-supplied URL field (callback_url, redirect_url,
     * success_url, cancel_url, return_url) or an absolute/relative URL as the handle.
     */
    public function resolve(array $input): array
    {
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $handle = (string) ($input['return_handle'] ?? '');
        $this->assertToken($facadeId, 96, 'facade');
        $this->assertToken($origin, 191, 'origin');
        $this->assertToken($handle, 64, 'return handle');
        foreach (['callback_url', 'redirect_url', 'success_url', 'cancel_url', 'return_url'] as $field) {
            if (array_key_exists($field, $input) && (string) $input[$field] !== '') {
                throw new DomainException('FACADE_REDIRECT_DENIED');
            }
        }
        // Handles are named allowlist keys only: no URLs, paths, or query strings.
        if ($handle === '' || preg_match('/^[A-Za-z0-9_-]{1,64}$/D', $handle) !== 1
            || str_contains($handle, '//') || str_contains($handle, '?')) {
            throw new DomainException('FACADE_REDIRECT_DENIED');
        }
        $facade = $this->facade($facadeId, $origin);
        $path = (string) ($facade['callbacks'][$handle] ?? '');
        if ($path === '' || !str_starts_with($path, '/')) {
            throw new DomainException('FACADE_REDIRECT_DENIED');
        }
        return [
            'schema' => self::SCHEMA,
            'facade_id' => $facadeId,
            'origin' => $origin,
            'return_handle' => $handle,
            'return_path' => $path,
            'return_url' => rtrim($origin, '/') . $path,
        ];
    }

    /** Branded EDD checkout URL: facade exact origin + allowlisted checkout path + opaque intent token. */
    public function brandedCheckoutUrl(string $facadeId, string $origin, string $intentToken): string
    {
        $facade = $this->facade($facadeId, $origin);
        $path = (string) ($facade['paths']['checkout'] ?? '');
        if ($path === '' || !str_starts_with($path, '/')) {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        $this->assertToken($intentToken, 64, 'intent token');
        return rtrim($origin, '/') . $path . '?intent=' . $intentToken;
    }

    /** Facades expose only the server-owned product allowlist from the facade registry. */
    public function assertFacadeSupports(string $facadeId, string $productCode): void
    {
        foreach ($this->facadeRegistry['facades'] as $facade) {
            if (hash_equals((string) $facade['facade_id'], $facadeId)) {
                if (in_array($productCode, $facade['products'], true)) {
                    return;
                }
                throw new DomainException('FACADE_PRODUCT_DENIED');
            }
        }
        throw new DomainException('FACADE_ORIGIN_DENIED');
    }

    /** Exact-origin facade lookup; wildcard authority is forbidden. */
    public function facade(string $facadeId, string $origin): array
    {
        foreach ($this->facadeRegistry['facades'] as $facade) {
            if (hash_equals((string) $facade['facade_id'], $facadeId)
                && in_array($origin, $facade['exact_origins'], true)) {
                return $facade;
            }
        }
        throw new DomainException('FACADE_ORIGIN_DENIED');
    }

    private static function assertToken(string $value, int $max, string $kind): void
    {
        if ($value === '' || strlen($value) > $max || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException("bounded {$kind} token required");
        }
    }
}

/**
 * EDD cart/session adapter: opens one server-owned synthetic EDD cart session for a
 * checkout intent and projects the synthetic order fixture. Download, price, and amount
 * come only from the resolved registry offer (passed by the checkout service); grant,
 * feature, limit, and commercial-right fields are never accepted. No EDD order/license
 * row is ever created here: the real order is created by EDD at payment and completion
 * stays deferred to the order-completion atom.
 */
final class FocusaSpec152eEddCartSessionAdapter
{
    public const SCHEMA = 'focusa.spec152e.edd_cart_session_adapter.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.edd_cart_session.v1';
    public const ORDER_FIXTURE_SCHEMA = 'focusa.spec152e.edd_order_fixture.v1';
    public const VERSION = 1;
    public const SESSION_TTL_SECONDS = 1800;
    public const SESSION_OPEN = 'open';

    /** Never accepted from any caller: grants and commercial policy stay server-owned. */
    private const CLIENT_CONTROLLED_FIELDS = [
        'grants', 'features', 'limits', 'node_limit', 'license_type', 'license_type_ref',
        'capability_family', 'families', 'sale_status', 'refund_policy', 'upgrade_policy',
        'commercial_rights', 'evaluation_duration', 'callback_url', 'redirect_url',
        'success_url', 'cancel_url', 'return_url',
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddCheckoutIntentMigration $schema,
        callable $clock,
        private int $sessionTtl = self::SESSION_TTL_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if ($this->sessionTtl < 1) {
            throw new InvalidArgumentException('positive cart session TTL required');
        }
    }

    /**
     * Open a server-owned cart session bound to the resolved registry offer. Only the
     * checkout service supplies product/download/price (resolved from the server-owned
     * product registry); direct callers cannot select them.
     *
     * Required input: registration_uuid, edd_customer_id, facade_id, product_code,
     * edd_download_id, edd_price_id, price_usd, request_id, idempotency_key.
     */
    public function openSession(array $input): array
    {
        $this->rejectClientControlledFields($input);
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $this->assertUuid($registrationUuid, 'registration');
        $customerId = $this->assertPositiveInt($input['edd_customer_id'] ?? null, 'EDD customer');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $productCode = (string) ($input['product_code'] ?? '');
        $downloadId = $this->assertPositiveInt($input['edd_download_id'] ?? null, 'EDD download');
        $priceId = (string) ($input['edd_price_id'] ?? '');
        $priceUsd = (string) ($input['price_usd'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertToken($facadeId, 96, 'facade');
        $this->assertToken($productCode, 128, 'product');
        $this->assertToken($priceId, 191, 'price');
        if (preg_match('/^\d{1,10}(\.\d{2})?$/D', $priceUsd) !== 1) {
            throw new InvalidArgumentException('server-owned USD price required');
        }
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $digest = $this->requestDigest([
            'operation' => 'open_cart_session',
            'registration_uuid' => $registrationUuid,
            'edd_customer_id' => $customerId,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'edd_download_id' => $downloadId,
            'edd_price_id' => $priceId,
            'price_usd' => $priceUsd,
            'request_id' => $requestId,
        ]);
        $replay = $this->replaySession($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }
        $cartReference = self::opaqueToken('cs_');
        $sessionKey = self::opaqueToken('sk_');
        $now = $this->now();
        $expires = self::plusSeconds($now, $this->sessionTtl);
        $table = $this->schema->table('wpuiai_edd_checkout_cart_sessions');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (cart_reference, session_key, registration_uuid, edd_customer_id, facade_id, product_code,
             edd_download_id, edd_price_id, price_usd, state, request_id, idempotency_key,
             request_digest, created_at, expires_at, updated_at)
            VALUES (:cart, :session, :registration, :customer, :facade, :product,
                    :download, :price, :amount, :state, :request, :idempotency,
                    :digest, :created, :expires, :updated)");
        $statement->execute([
            ':cart' => $cartReference,
            ':session' => $sessionKey,
            ':registration' => $registrationUuid,
            ':customer' => $customerId,
            ':facade' => $facadeId,
            ':product' => $productCode,
            ':download' => $downloadId,
            ':price' => $priceId,
            ':amount' => $priceUsd,
            ':state' => self::SESSION_OPEN,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':digest' => $digest,
            ':created' => $now,
            ':expires' => $expires,
            ':updated' => $now,
        ]);
        return [
            'schema' => self::RESULT_SCHEMA,
            'cart_reference' => $cartReference,
            'session_key' => $sessionKey,
            'registration_uuid' => $registrationUuid,
            'edd_customer_id' => $customerId,
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'edd_download_id' => $downloadId,
            'edd_price_id' => $priceId,
            'price_usd' => $priceUsd,
            'state' => self::SESSION_OPEN,
            'expires_at' => $expires,
            'replayed' => false,
        ];
    }

    public function findByCartReference(string $cartReference): array
    {
        $this->assertToken($cartReference, 64, 'cart reference');
        $table = $this->schema->table('wpuiai_edd_checkout_cart_sessions');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE cart_reference = :ref");
        $statement->execute([':ref' => $cartReference]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('cart session not found');
        }
        return $row;
    }

    /**
     * Synthetic EDD order fixture bound to the server-owned cart session. The fixture
     * carries no real order id (EDD creates the order at payment), reproduces the exact
     * registry price relationship, and contains no email, key, or secret. Order
     * completion itself stays deferred to the order-completion atom.
     */
    public function projectOrderFixture(string $cartReference): array
    {
        $cart = $this->findByCartReference($cartReference);
        $amount = (string) $cart['price_usd'];
        return [
            'schema' => self::ORDER_FIXTURE_SCHEMA,
            'fixture' => 'synthetic',
            'registration_uuid' => (string) $cart['registration_uuid'],
            'edd_customer_id' => (int) $cart['edd_customer_id'],
            'order' => [
                'order_id' => null,
                'status' => 'checkout_required',
                'customer_id' => (int) $cart['edd_customer_id'],
                'email' => null,
            ],
            'items' => [[
                'download_id' => (int) $cart['edd_download_id'],
                'price_id' => (string) $cart['edd_price_id'],
                'quantity' => 1,
                'unit_amount_usd' => $amount,
                'total_amount_usd' => $amount,
            ]],
            'total_amount_usd' => $amount,
            'entitlement_allowed' => true,
        ];
    }

    public function sessionCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_checkout_cart_sessions');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    private function rejectClientControlledFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function replaySession(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_checkout_cart_sessions');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'cart_reference' => (string) $row['cart_reference'],
            'session_key' => (string) $row['session_key'],
            'registration_uuid' => (string) $row['registration_uuid'],
            'edd_customer_id' => (int) $row['edd_customer_id'],
            'facade_id' => (string) $row['facade_id'],
            'product_code' => (string) $row['product_code'],
            'edd_download_id' => (int) $row['edd_download_id'],
            'edd_price_id' => (string) $row['edd_price_id'],
            'price_usd' => (string) $row['price_usd'],
            'state' => (string) $row['state'],
            'expires_at' => (string) $row['expires_at'],
            'replayed' => true,
        ];
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddCheckoutIntentMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddCheckoutIntentMigration::encodeCanonical($value));
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private static function assertToken(string $value, int $max, string $kind): void
    {
        if ($value === '' || strlen($value) > $max || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException("bounded {$kind} token required");
        }
    }

    private function assertPositiveInt(mixed $value, string $field): int
    {
        if (!is_int($value) || $value < 1) {
            throw new InvalidArgumentException("positive {$field} required");
        }
        return $value;
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
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

/**
 * Checkout service: creates exactly one branded EDD checkout intent from a verified,
 * promoted registration. Binds registration, account, EDD customer, product, node
 * request, and idempotency; returns a branded facade URL and never accepts a
 * client-controlled amount, price, product, grant, or redirect target.
 */
final class FocusaSpec152eEddCheckoutIntentService
{
    public const SCHEMA = 'focusa.spec152e.edd_checkout_intent.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.checkout_intent_result.v1';
    public const VERSION = 1;
    public const INTENT_TTL_SECONDS = 1800;
    public const STATE_CHECKOUT_REQUIRED = 'checkout_required';
    public const STATE_PAYMENT_PENDING = 'payment_pending';
    private const ACTIVE_INTENT_STATES = [self::STATE_CHECKOUT_REQUIRED, self::STATE_PAYMENT_PENDING];

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'tier', 'products', 'product_code', 'license_type',
        'license_type_ref', 'capability_family', 'families', 'features', 'grants', 'limits',
        'node_limit', 'sale_status', 'refund_policy', 'upgrade_policy', 'commercial_rights',
        'evaluation_duration', 'edd_download_id', 'edd_price_id',
    ];
    private const CALLER_REDIRECT_FIELDS = ['callback_url', 'redirect_url', 'success_url', 'cancel_url', 'return_url'];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddCheckoutIntentMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eEddCartSessionAdapter $cart,
        private FocusaSpec152eFacadeReturnHandleRegistry $returnHandles,
        private array $productRegistry,
        callable $clock,
        private int $intentTtl = self::INTENT_TTL_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if ($this->intentTtl < 1) {
            throw new InvalidArgumentException('positive checkout intent TTL required');
        }
    }

    /**
     * Create exactly one branded EDD checkout intent.
     *
     * Required input:
     *   - registration_uuid:  verified, live, promoted registration UUID
     *   - facade_id / origin: exact registered facade binding
     *   - return_handle:      named allowlisted facade callback handle (never a URL)
     *   - request_id / idempotency_key
     *
     * Optional input:
     *   - node_uuid:          opaque node UUID to bind (node request)
     *   - device_public_key:  device public key to bind; stored only as a digest
     *
     * Forbidden input: any client-controlled commerce field (price, amount, grants,
     * features, limits, product, edd ids) and any caller-supplied redirect URL.
     *
     * Returns a masked envelope: no raw email, no device key, no license/secret. A
     * repeated canonical request (new idempotency key) returns the existing intent.
     */
    public function createIntent(array $input): array
    {
        $this->rejectClientControlledFields($input);
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $returnHandle = (string) ($input['return_handle'] ?? '');
        $nodeUuid = ($input['node_uuid'] ?? null);
        $devicePublicKey = ($input['device_public_key'] ?? null);
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $this->assertToken($facadeId, 96, 'facade');
        $this->assertToken($origin, 191, 'origin');
        $this->assertToken($returnHandle, 64, 'return handle');
        if ($nodeUuid !== null && $nodeUuid !== '') {
            $this->assertUuid((string) $nodeUuid, 'node');
        }
        if ($devicePublicKey !== null && $devicePublicKey !== '') {
            $this->assertToken((string) $devicePublicKey, 191, 'device public key');
        }

        // The registration must be verified, live, promoted (customer+account bound),
        // and bound to exactly this facade.
        try {
            $registration = $this->registrations->findByUuid($registrationUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $now = $this->now();
        if (!in_array((string) $registration['state'], FocusaSpec152eVerifiedRegistrationTokenValidator::VERIFIED_NONTERMINAL_STATES, true)
            || (string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if ($registration['edd_customer_id'] === null || (int) $registration['edd_customer_id'] < 1
            || $registration['account_uuid'] === null) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }
        if (!hash_equals((string) $registration['facade_id'], $facadeId)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }

        // Server-owned product resolution: the product was bound at registration; the
        // caller can never select or price it here.
        $productCode = (string) $registration['product_code'];
        $offer = $this->resolveOffer($productCode);
        $this->returnHandles->assertFacadeSupports($facadeId, $productCode);

        // Branded facade return URL: allowlisted named handle only.
        $return = $this->returnHandles->resolve([
            'facade_id' => $facadeId,
            'origin' => $origin,
            'return_handle' => $returnHandle,
        ]);

        $deviceHash = ($devicePublicKey === null || $devicePublicKey === '') ? null : hash('sha256', (string) $devicePublicKey);
        $nodeUuidValue = ($nodeUuid === null || $nodeUuid === '') ? null : (string) $nodeUuid;
        $digest = $this->requestDigest([
            'operation' => 'create_checkout_intent',
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'origin' => $origin,
            'return_handle' => $returnHandle,
            'product_code' => $productCode,
            'node_uuid' => $nodeUuidValue,
            'device_public_key_hash' => $deviceHash,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayIntent($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        // Exactly one intent per promoted registration/product: a repeated canonical
        // request returns the existing intent instead of creating a second one.
        $existing = $this->findActiveIntent($registrationUuid, $productCode);
        if ($existing !== null) {
            return $this->presentIntent($existing, false, true);
        }

        // Server-owned synthetic cart session: download/price come only from the offer.
        $cart = $this->cart->openSession([
            'registration_uuid' => $registrationUuid,
            'edd_customer_id' => (int) $registration['edd_customer_id'],
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'edd_download_id' => (int) $offer['edd_download_id'],
            'edd_price_id' => (string) $offer['edd_price_id'],
            'price_usd' => (string) $offer['price_usd'],
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
        ]);

        $intentId = self::opaqueToken('it_');
        $brandedUrl = $this->returnHandles->brandedCheckoutUrl($facadeId, $origin, $intentId);
        $expires = self::plusSeconds($now, $this->intentTtl);
        $table = $this->schema->table('wpuiai_edd_checkout_intents');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (intent_id, registration_uuid, account_uuid, edd_customer_id, facade_id, origin,
             product_code, edd_download_id, edd_price_id, price_usd, node_uuid,
             device_public_key_hash, cart_reference, session_key, return_handle, return_url,
             branded_checkout_url, state, request_id, idempotency_key, request_digest,
             created_at, expires_at, settled_at, updated_at)
            VALUES (:intent, :registration, :account, :customer, :facade, :origin,
                    :product, :download, :price, :amount, :node,
                    :device_hash, :cart, :session, :handle, :return_url,
                    :branded, :state, :request, :idempotency, :digest,
                    :created, :expires, NULL, :updated)");
        $statement->execute([
            ':intent' => $intentId,
            ':registration' => $registrationUuid,
            ':account' => $registration['account_uuid'],
            ':customer' => (int) $registration['edd_customer_id'],
            ':facade' => $facadeId,
            ':origin' => $origin,
            ':product' => $productCode,
            ':download' => (int) $offer['edd_download_id'],
            ':price' => (string) $offer['edd_price_id'],
            ':amount' => (string) $offer['price_usd'],
            ':node' => $nodeUuidValue,
            ':device_hash' => $deviceHash,
            ':cart' => $cart['cart_reference'],
            ':session' => $cart['session_key'],
            ':handle' => $returnHandle,
            ':return_url' => $return['return_url'],
            ':branded' => $brandedUrl,
            ':state' => self::STATE_CHECKOUT_REQUIRED,
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':digest' => $digest,
            ':created' => $now,
            ':expires' => $expires,
            ':updated' => $now,
        ]);

        // Advance the registration toward checkout_pending through the legal state
        // machine path (account_promoted -> offer_selected -> checkout_pending). The
        // offer_selected hop uses a derived idempotency key so each hop journals
        // independently; a replay at the intent level returns before any transition.
        $state = (string) $registration['state'];
        $stateVersion = (int) $registration['state_version'];
        if ($state === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED) {
            $selected = $this->registrations->transition(
                $registrationUuid,
                $state,
                FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
                $stateVersion,
                $requestId,
                $idempotencyKey . ':offer',
                ['state_reason' => 'offer_selected_for_checkout', 'offer_code' => $productCode],
            );
            $state = FocusaSpec152eActivationRegistrationState::OFFER_SELECTED;
            $stateVersion = (int) $selected['registration']['state_version'];
        }
        if ($state === FocusaSpec152eActivationRegistrationState::OFFER_SELECTED) {
            $this->registrations->transition(
                $registrationUuid,
                $state,
                FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
                $stateVersion,
                $requestId,
                $idempotencyKey,
                [
                    'state_reason' => 'checkout_intent_created',
                    'edd_cart_reference' => $cart['cart_reference'],
                    'node_uuid' => $nodeUuidValue,
                    'device_public_key' => ($devicePublicKey === null || $devicePublicKey === '') ? null : (string) $devicePublicKey,
                ],
            );
        }

        return $this->presentIntent([
            'intent_id' => $intentId,
            'registration_uuid' => $registrationUuid,
            'account_uuid' => $registration['account_uuid'],
            'edd_customer_id' => (int) $registration['edd_customer_id'],
            'facade_id' => $facadeId,
            'product_code' => $productCode,
            'price_usd' => (string) $offer['price_usd'],
            'node_uuid' => $nodeUuidValue,
            'cart_reference' => $cart['cart_reference'],
            'session_key' => $cart['session_key'],
            'return_handle' => $returnHandle,
            'return_url' => $return['return_url'],
            'branded_checkout_url' => $brandedUrl,
            'state' => self::STATE_CHECKOUT_REQUIRED,
            'expires_at' => $expires,
        ], false, false);
    }

    public function intentCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_checkout_intents');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function findByIntentId(string $intentId): array
    {
        $this->assertToken($intentId, 64, 'intent');
        $table = $this->schema->table('wpuiai_edd_checkout_intents');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE intent_id = :intent");
        $statement->execute([':intent' => $intentId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new OutOfBoundsException('checkout intent not found');
        }
        return $row;
    }

    private function resolveOffer(string $productCode): array
    {
        foreach ($this->productRegistry['protected_offers'] as $offer) {
            if (hash_equals((string) $offer['public_code'], $productCode)) {
                if ((bool) $offer['checkout_enabled'] !== true || (string) $offer['mapping_status'] !== 'active') {
                    throw new DomainException('EDD_CHECKOUT_REQUIRED');
                }
                if ($offer['edd_download_id'] === null || $offer['edd_price_id'] === null
                    || (string) $offer['edd_price_id'] === '' || preg_match('/^\d{1,10}(\.\d{2})?$/D', (string) $offer['price_usd']) !== 1) {
                    throw new DomainException('EDD_CHECKOUT_REQUIRED');
                }
                return $offer;
            }
        }
        throw new DomainException('PRODUCT_MAPPING_REQUIRED');
    }

    private function findActiveIntent(string $registrationUuid, string $productCode): ?array
    {
        $table = $this->schema->table('wpuiai_edd_checkout_intents');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE registration_uuid = :registration AND product_code = :product
              AND state IN ('checkout_required', 'payment_pending')
            ORDER BY created_at ASC LIMIT 1");
        $statement->execute([':registration' => $registrationUuid, ':product' => $productCode]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function replayIntent(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_checkout_intents');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $this->presentIntent($row, true, false);
    }

    private function presentIntent(array $row, bool $replayed, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'intent' => [
                'intent_id' => (string) $row['intent_id'],
                'registration_id' => (string) $row['registration_uuid'],
                'account_id' => $row['account_uuid'] === null ? null : (string) $row['account_uuid'],
                'customer_id' => (int) $row['edd_customer_id'],
                'facade_id' => (string) $row['facade_id'],
                'product_code' => (string) $row['product_code'],
                'state' => (string) $row['state'],
                'next_action' => 'open_checkout',
                'branded_checkout_url' => (string) $row['branded_checkout_url'],
                'return_handle' => (string) $row['return_handle'],
                'return_url' => (string) $row['return_url'],
                'cart_reference' => (string) $row['cart_reference'],
                'session_key' => (string) $row['session_key'],
                'node_id' => $row['node_uuid'] === null ? null : (string) $row['node_uuid'],
                'price' => ['currency' => 'USD', 'amount_usd' => (string) $row['price_usd']],
                'expires_at' => (string) $row['expires_at'],
            ],
            'replayed' => $replayed,
            'existing' => $existing,
        ];
    }

    private function rejectClientControlledFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
        foreach (self::CALLER_REDIRECT_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('FACADE_REDIRECT_DENIED');
            }
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddCheckoutIntentMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddCheckoutIntentMigration::encodeCanonical($value));
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private static function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private static function assertToken(string $value, int $max, string $kind): void
    {
        if ($value === '' || strlen($value) > $max || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException("bounded {$kind} token required");
        }
    }

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
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
