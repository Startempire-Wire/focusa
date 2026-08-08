<?php
// Spec 152E EDD order-completion binding (addendum sections 11 completion gate, 17
// lifecycle hooks, and 3.2 mapping gaps). Binds EDD order completion idempotently to
// the verified registration/account:
//
//   - Settlement requires a complete, eligible order whose canonical EDD rows carry the
//     exact registration, account, customer, order item, download, price, product, and
//     a real order-linked payment transaction. Synthetic or unlinked payment IDs
//     (focusa_live_*, synthetic_*, manual/none markers, or transactions not bound to
//     this exact canonical order) fail closed with EDD_ORDER_UNVERIFIED and never
//     issue.
//   - One eligible order item settles one entitlement issuance request exactly once.
//     Duplicate completion events return the existing settlement (existing=true) and
//     never create a second issuance request; idempotency-key replays return the same
//     decision (replayed payload); reusing a key for a different request fails
//     IDEMPOTENCY_CONFLICT.
//   - Out-of-order events cannot issue: the canonical order row status is authoritative,
//     so a 'complete' event for an order whose canonical row is pending/refunded fails
//     closed, and a refunded/revoked event journals a durable blocked binding that a
//     later complete event can never over-settle (out_of_order, zero issuance).
//   - Issuance stays deferred to the verified issuance service: this handler settles the
//     bounded issuance-request journal only; it never creates an EDD license, key, or
//     lease. No raw email, raw payment transaction id, secret, or unmasked real-email
//     evidence is stored or returned (emails are keyed digests; payment identities are
//     keyed digests; all handles are opaque bounded tokens).
//
// Failures are public-safe stable codes (REFUNDED, REVOKED, EDD_ORDER_PENDING,
// EDD_ORDER_UNVERIFIED, EMAIL_VERIFICATION_REQUIRED, REGISTRATION_EXPIRED,
// EDD_CUSTOMER_RESOLUTION_FAILED, ACCOUNT_MERGE_REVIEW_REQUIRED,
// ACCOUNT_EMAIL_MISMATCH, FACADE_ORIGIN_DENIED, FACADE_PRODUCT_DENIED,
// PRODUCT_MAPPING_REQUIRED, EDD_CHECKOUT_REQUIRED, EDD_LICENSE_UNUSABLE,
// CLIENT_COMMERCIAL_FIELDS_FORBIDDEN, IDEMPOTENCY_CONFLICT). No new error code is
// introduced.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-verified-registration-token-validator.v1.php, and the
// server-owned product/facade registries to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddOrderBindingMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_order_binding.v1';
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
        $bindings = $this->table('wpuiai_edd_order_bindings');
        $requests = $this->table('wpuiai_edd_issuance_requests');
        $migrations = $this->table('wpuiai_edd_order_binding_schema_migrations');
        $events = $this->table('wpuiai_edd_order_binding_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$bindings} (
            binding_key VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            order_item_id BIGINT NOT NULL,
            download_id BIGINT NOT NULL,
            price_id VARCHAR(191) NOT NULL DEFAULT '',
            product_code VARCHAR(128) NOT NULL,
            license_type_ref VARCHAR(128) NOT NULL,
            facade_id VARCHAR(96) NULL,
            payment_gateway VARCHAR(64) NULL,
            payment_transaction_digest VARCHAR(64) NULL,
            binding_state VARCHAR(24) NOT NULL CHECK (binding_state IN ('settled_pending_issuance', 'blocked')),
            blocked_reason VARCHAR(64) NULL,
            issuance_request_key VARCHAR(64) NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (registration_uuid, order_id, order_item_id)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_order_binding_idempotency
            ON {$bindings} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_order_binding_blocked
            ON {$bindings} (registration_uuid, order_id, binding_state)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_order_binding_retention
            ON {$bindings} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$requests} (
            issuance_request_key VARCHAR(64) NOT NULL PRIMARY KEY,
            binding_key VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            order_item_id BIGINT NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            license_type_ref VARCHAR(128) NOT NULL,
            state VARCHAR(16) NOT NULL CHECK (state IN ('pending', 'issued', 'superseded', 'failed')),
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            UNIQUE (binding_key)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_issuance_request_registration
            ON {$requests} (registration_uuid, order_id)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_issuance_request_retention
            ON {$requests} (retention_until)");
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

    /** Rollback is preservation-only: binding and issuance-request journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_order_binding_schema_events');
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

final class FocusaSpec152eEddOrderBindingService
{
    public const SCHEMA = 'focusa.spec152e.edd_order_binding.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.edd_order_binding_decision.v1';
    public const VERSION = 1;
    public const RETENTION_SECONDS = 2592000;
    public const STATE_SETTLED = 'settled_pending_issuance';
    public const STATE_BLOCKED = 'blocked';
    public const REQUEST_STATE_PENDING = 'pending';

    private const CREDIT_PACK_REASON_PREFIX = 'credit_pack_';
    private const UNRELATED_DISPOSITION = 'quarantine';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'tier', 'products', 'product_code', 'license_type',
        'license_type_ref', 'capability_family', 'families', 'features', 'grants', 'limits',
        'node_limit', 'sale_status', 'refund_policy', 'upgrade_policy', 'commercial_rights',
        'evaluation_duration', 'edd_download_id', 'edd_price_id',
    ];

    private const SYNTHETIC_PAYMENT_MARKERS = [
        'synthetic_', 'focusa_live_', 'manual_', 'unlinked_', 'pending_', 'local_',
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddOrderBindingMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private array $productRegistry,
        private array $facadeRegistry,
        callable $clock,
        private string $eddPrefix = 'wp_',
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $eddPrefix) !== 1) {
            throw new InvalidArgumentException('invalid EDD table prefix');
        }
        if ($this->retention < 1) {
            throw new InvalidArgumentException('positive retention required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Bind an EDD order completion to the verified registration/account. Accepts only
     * complete, eligible orders whose canonical EDD rows carry the exact account,
     * registration, order item, price, and product binding, plus a real order-linked
     * payment transaction. Each eligible protected order item settles exactly one
     * entitlement issuance request; duplicates and out-of-order events never issue.
     *
     * Required input:
     *   - order_id (int), order_status, customer_id (int)
     *   - order_items: list of ['order_item_id' => int, 'download_id' => int,
     *     'price_id' => string, 'quantity' => int]
     *   - payment_transactions: list of ['gateway' => string, 'transaction_id' => string,
     *     'status' => string] — required when the order contains a protected item
     *   - registration_uuid, facade_id, origin (registration_uuid required when the
     *     order contains a protected item)
     *   - request_id, idempotency_key
     *
     * Returns a public-safe decision (never raw email, payment ids, secrets, or keys).
     * Replays with the same idempotency key return the same decision; a repeated
     * canonical request for the same order/registration/item returns the existing
     * settlement with existing=true and settles nothing new.
     */
    public function bindOrderComplete(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $orderId = $this->assertPositiveInt($input['order_id'] ?? null, 'order_id');
        $orderStatus = (string) ($input['order_status'] ?? '');
        $customerId = $this->assertPositiveInt($input['customer_id'] ?? null, 'customer_id');
        $items = $input['order_items'] ?? [];
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $paymentTransactions = $input['payment_transactions'] ?? [];
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        if ($orderStatus === '') {
            throw new InvalidArgumentException('order status is required');
        }
        if (!is_array($items) || $items === []) {
            throw new InvalidArgumentException('order items are required');
        }
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $this->assertToken($facadeId, 96, 'facade');
        $this->assertToken($origin, 191, 'origin');

        $digest = $this->requestDigest([
            'operation' => 'order_complete_binding',
            'order_id' => $orderId,
            'order_status' => $orderStatus,
            'customer_id' => $customerId,
            'order_items' => $items,
            'payment_transaction_digests' => $this->presentedPaymentDigests($paymentTransactions),
            'facade_id' => $facadeId,
            'origin' => $origin,
            'registration_uuid' => $registrationUuid,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        // Canonical EDD order row is the authoritative truth for settlement.
        $orderRow = $this->findOrderRow($orderId);
        if ($orderRow === null) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $rowStatus = (string) ($orderRow['status'] ?? '');

        // Terminal/out-of-order statuses (event or canonical row): journal a durable
        // blocked binding when protected items are present, then fail closed. A later
        // complete event can never over-settle a journaled terminal block.
        if (in_array($orderStatus, ['refunded', 'revoked'], true)
            || in_array($rowStatus, ['refunded', 'revoked'], true)) {
            $terminal = in_array($orderStatus, ['refunded', 'revoked'], true) ? $orderStatus : $rowStatus;
            $code = $terminal === 'refunded' ? 'REFUNDED' : 'REVOKED';
            $this->journalTerminalBlock($input, $orderId, $code, $digest, $requestId, $idempotencyKey);
            throw new DomainException($code);
        }
        if ($orderStatus !== 'complete') {
            if (in_array($orderStatus, ['pending', 'processing'], true)) {
                throw new DomainException('EDD_ORDER_PENDING');
            }
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ($rowStatus !== 'complete') {
            if (in_array($rowStatus, ['pending', 'processing'], true)) {
                throw new DomainException('EDD_ORDER_PENDING');
            }
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((int) $orderRow['customer_id'] !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        $protected = [];
        $excluded = [];
        $existingEntries = [];
        $registration = null;
        $accountId = null;
        $payment = null;
        foreach ($items as $item) {
            if (!is_array($item)) {
                throw new InvalidArgumentException('malformed order item');
            }
            $itemDownload = $this->assertPositiveInt($item['download_id'] ?? null, 'order item download_id');
            $mapping = $this->resolveDownloadMapping($itemDownload);
            if ($mapping['disposition'] === 'credit_pack') {
                $excluded[] = ['download_id' => $itemDownload, 'disposition' => 'credit_pack_excluded'];
                continue;
            }
            if ($mapping['disposition'] === 'unknown') {
                throw new DomainException('PRODUCT_MAPPING_REQUIRED');
            }
            if ($mapping['disposition'] === 'non_entitlement') {
                $excluded[] = ['download_id' => $itemDownload, 'disposition' => 'non_entitlement'];
                continue;
            }

            // First protected item: verified, live, promoted registration bound to the
            // exact facade, exact account/customer binding, exact verified order email,
            // and a real order-linked payment transaction.
            if ($registration === null) {
                $registration = $this->assertVerifiedRegistration($registrationUuid, $facadeId);
                $this->assertFacadeSupports($facadeId, $origin, '');
                $accountId = $this->assertAccountBinding($registration, $customerId);
                $this->assertOrderEmailBinding($registration, $orderRow);
                $payment = $this->assertPaymentBoundToOrder($orderId, $paymentTransactions);

                // A durable terminal block journaled earlier (out-of-order arrival) can
                // never be over-settled by this completion event.
                $orderBlocked = $this->findBlockedForOrder($registrationUuid, $orderId);
                if ($orderBlocked !== null) {
                    return $this->outOfOrderDecision(
                        $registrationUuid, $orderId, $accountId, (string) $orderBlocked['blocked_reason'], true,
                    );
                }
            }

            $offer = $mapping['offer'];
            $this->assertFacadeSupports($facadeId, $origin, (string) $offer['public_code']);
            if (!hash_equals((string) $registration['product_code'], (string) $offer['public_code'])) {
                throw new DomainException('FACADE_PRODUCT_DENIED');
            }
            if (!$offer['checkout_enabled'] || $offer['mapping_status'] !== 'active') {
                throw new DomainException('EDD_CHECKOUT_REQUIRED');
            }
            $itemPrice = (string) ($item['price_id'] ?? '');
            if ($itemPrice === '' || !hash_equals((string) $offer['edd_price_id'], $itemPrice)) {
                throw new DomainException('PRODUCT_MAPPING_REQUIRED');
            }
            $itemRowId = $this->assertPositiveInt($item['order_item_id'] ?? null, 'order item order_item_id');
            $this->assertCanonicalOrderItem($orderId, $itemRowId, $itemDownload);
            if ($this->hasEquivalentActiveLicense($customerId, $itemDownload)) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }

            $existingBinding = $this->findBindingByTriple($registrationUuid, $orderId, $itemRowId);
            if ($existingBinding !== null) {
                $existingEntries[] = $this->bindingEntry($existingBinding, true);
                continue;
            }
            $protected[] = [
                'order_item_id' => $itemRowId,
                'download_id' => $itemDownload,
                'product_code' => (string) $offer['public_code'],
                'price_id' => $itemPrice,
                'license_type_ref' => (string) $offer['license_type_ref'],
            ];
        }

        if ($protected === [] && $existingEntries === []) {
            return [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'no_entitlement',
                'order_id' => $orderId,
                'protected_items' => 0,
                'excluded_items' => $excluded,
                'issuance' => 'none',
                'facade_id' => $facadeId,
            ];
        }

        // Settle one entitlement issuance request per new eligible order item, exactly
        // once, inside one transaction. Duplicate events never add a second request.
        $newEntries = [];
        if ($protected !== []) {
            $newEntries = $this->settleBindings(
                $registrationUuid,
                $accountId,
                $customerId,
                $orderId,
                $facadeId,
                $payment,
                $protected,
                $excluded,
                $requestId,
                $idempotencyKey,
                $digest,
            );
        }

        $primaryProduct = (string) ($newEntries[0]['product_code'] ?? $existingEntries[0]['product_code'] ?? '');
        $entries = array_merge($newEntries, $existingEntries);
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'order_bound',
            'order_id' => $orderId,
            'registration_id' => $registrationUuid,
            'account_id' => $accountId,
            'customer_id' => $customerId,
            'facade_id' => $facadeId,
            'product_code' => $primaryProduct,
            'protected_items' => $entries,
            'excluded_items' => $excluded,
            'entitlement_allowed' => true,
            'issuance' => 'deferred_to_verified_issuance_service',
            'issuance_requests_settled' => count($newEntries),
            'payment_bound' => true,
            'existing' => count($newEntries) === 0,
        ];
    }

    /** Bounded journal lookups for settlement/reconciliation. */
    public function bindingCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}");
        $statement->execute();
        return (int) $statement->fetchColumn();
    }

    public function issuanceRequestCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_issuance_requests');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}");
        $statement->execute();
        return (int) $statement->fetchColumn();
    }

    /** Bounded: exact binding lookup by opaque binding key. */
    public function findByBindingKey(string $bindingKey): ?array
    {
        $this->assertToken($bindingKey, 64, 'binding');
        $table = $this->schema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE binding_key = :key LIMIT 1");
        $statement->execute([':key' => $bindingKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Bounded: exact issuance-request lookup by opaque handle. */
    public function findIssuanceRequestByKey(string $issuanceRequestKey): ?array
    {
        $this->assertToken($issuanceRequestKey, 64, 'issuance request');
        $table = $this->schema->table('wpuiai_edd_issuance_requests');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── private helpers ────────────────────────────────────────────────

    /**
     * Insert one binding row per new eligible protected item plus exactly one issuance
     * request per binding, atomically. Idempotent: rows already present for the
     * (registration, order, order item) triple are never duplicated.
     */
    private function settleBindings(
        string $registrationUuid,
        string $accountId,
        int $customerId,
        int $orderId,
        string $facadeId,
        array $payment,
        array $protected,
        array $excluded,
        string $requestId,
        string $idempotencyKey,
        string $digest,
    ): array {
        $entries = [];
        $now = $this->now();
        $retention = self::plusSeconds($now, $this->retention);
        $bindings = $this->schema->table('wpuiai_edd_order_bindings');
        $requests = $this->schema->table('wpuiai_edd_issuance_requests');
        $this->db->beginTransaction();
        try {
            foreach ($protected as $item) {
                $bindingKey = self::opaqueToken('ob_');
                $issuanceKey = self::opaqueToken('ir_');
                $decision = [
                    'schema' => self::RESULT_SCHEMA,
                    'decision' => 'order_bound',
                    'order_id' => $orderId,
                    'registration_id' => $registrationUuid,
                    'account_id' => $accountId,
                    'customer_id' => $customerId,
                    'facade_id' => $facadeId,
                    'product_code' => (string) $item['product_code'],
                    'protected_items' => [
                        [
                            'binding_key' => $bindingKey,
                            'order_item_id' => (int) $item['order_item_id'],
                            'download_id' => (int) $item['download_id'],
                            'product_code' => (string) $item['product_code'],
                            'price_id' => (string) $item['price_id'],
                            'license_type_ref' => (string) $item['license_type_ref'],
                            'issuance_request_handle' => $issuanceKey,
                            'existing' => false,
                        ],
                    ],
                    'excluded_items' => $excluded,
                    'entitlement_allowed' => true,
                    'issuance' => 'deferred_to_verified_issuance_service',
                    'issuance_requests_settled' => 1,
                    'payment_bound' => true,
                    'existing' => false,
                ];
                $statement = $this->db->prepare("INSERT INTO {$bindings}
                    (binding_key, registration_uuid, account_uuid, customer_id, order_id, order_item_id,
                     download_id, price_id, product_code, license_type_ref, facade_id, payment_gateway,
                     payment_transaction_digest, binding_state, blocked_reason, issuance_request_key,
                     result_payload, request_id, idempotency_key, request_digest, created_at,
                     retention_until, updated_at)
                    VALUES (:key, :registration, :account, :customer, :order, :item,
                            :download, :price, :product, :license_type, :facade, :gateway,
                            :payment_digest, :state, NULL, :issuance,
                            :payload, :request, :idempotency, :digest, :created,
                            :retention, :updated)");
                $statement->execute([
                    ':key' => $bindingKey,
                    ':registration' => $registrationUuid,
                    ':account' => $accountId,
                    ':customer' => $customerId,
                    ':order' => $orderId,
                    ':item' => (int) $item['order_item_id'],
                    ':download' => (int) $item['download_id'],
                    ':price' => (string) $item['price_id'],
                    ':product' => (string) $item['product_code'],
                    ':license_type' => (string) $item['license_type_ref'],
                    ':facade' => $facadeId,
                    ':gateway' => (string) $payment['gateway'],
                    ':payment_digest' => (string) $payment['transaction_digest'],
                    ':state' => self::STATE_SETTLED,
                    ':issuance' => $issuanceKey,
                    ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                    ':request' => $requestId,
                    ':idempotency' => $idempotencyKey,
                    ':digest' => $digest,
                    ':created' => $now,
                    ':retention' => $retention,
                    ':updated' => $now,
                ]);
                $requestStatement = $this->db->prepare("INSERT INTO {$requests}
                    (issuance_request_key, binding_key, registration_uuid, account_uuid, customer_id,
                     order_id, order_item_id, product_code, license_type_ref, state, created_at,
                     retention_until)
                    VALUES (:key, :binding, :registration, :account, :customer, :order, :item,
                            :product, :license_type, :state, :created, :retention)");
                $requestStatement->execute([
                    ':key' => $issuanceKey,
                    ':binding' => $bindingKey,
                    ':registration' => $registrationUuid,
                    ':account' => $accountId,
                    ':customer' => $customerId,
                    ':order' => $orderId,
                    ':item' => (int) $item['order_item_id'],
                    ':product' => (string) $item['product_code'],
                    ':license_type' => (string) $item['license_type_ref'],
                    ':state' => self::REQUEST_STATE_PENDING,
                    ':created' => $now,
                    ':retention' => $retention,
                ]);
                $entries[] = [
                    'binding_key' => $bindingKey,
                    'order_item_id' => (int) $item['order_item_id'],
                    'download_id' => (int) $item['download_id'],
                    'product_code' => (string) $item['product_code'],
                    'price_id' => (string) $item['price_id'],
                    'license_type_ref' => (string) $item['license_type_ref'],
                    'issuance_request_handle' => $issuanceKey,
                    'existing' => false,
                ];
            }
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        return $entries;
    }

    /**
     * Journal a durable blocked binding per protected item for a terminal refunded/
     * revoked event. The canonical row status already fails closed; the journal keeps a
     * later out-of-order complete event from ever settling. Settled bindings are never
     * overwritten (refund propagation stays with later lifecycle atoms).
     */
    private function journalTerminalBlock(array $input, int $orderId, string $code, string $digest, string $requestId, string $idempotencyKey): void
    {
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $items = $input['order_items'] ?? [];
        if ($registrationUuid === '' || !is_array($items)
            || preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $registrationUuid) !== 1) {
            return;
        }
        $now = $this->now();
        $retention = self::plusSeconds($now, $this->retention);
        $bindings = $this->schema->table('wpuiai_edd_order_bindings');
        foreach ($items as $item) {
            if (!is_array($item)) {
                continue;
            }
            try {
                $itemDownload = $this->assertPositiveInt($item['download_id'] ?? null, 'order item download_id');
                $itemRowId = $this->assertPositiveInt($item['order_item_id'] ?? null, 'order item order_item_id');
            } catch (InvalidArgumentException) {
                continue;
            }
            $mapping = $this->resolveDownloadMapping($itemDownload);
            if ($mapping['disposition'] !== 'protected') {
                continue;
            }
            if ($this->findBindingByTriple($registrationUuid, $orderId, $itemRowId) !== null) {
                continue;
            }
            $offer = $mapping['offer'];
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'out_of_order',
                'order_id' => $orderId,
                'registration_id' => $registrationUuid,
                'blocked_reason' => $code,
                'issuance' => 'none',
                'issuance_requests_settled' => 0,
                'existing' => true,
            ];
            $statement = $this->db->prepare("INSERT INTO {$bindings}
                (binding_key, registration_uuid, account_uuid, customer_id, order_id, order_item_id,
                 download_id, price_id, product_code, license_type_ref, facade_id, payment_gateway,
                 payment_transaction_digest, binding_state, blocked_reason, issuance_request_key,
                 result_payload, request_id, idempotency_key, request_digest, created_at,
                 retention_until, updated_at)
                VALUES (:key, :registration, NULL, :customer, :order, :item,
                        :download, :price, :product, :license_type, NULL, NULL,
                        NULL, :state, :reason, NULL,
                        :payload, :request, :idempotency, :digest, :created,
                        :retention, :updated)");
            $statement->execute([
                ':key' => self::opaqueToken('ob_'),
                ':registration' => $registrationUuid,
                ':customer' => (int) ($input['customer_id'] ?? 0),
                ':order' => $orderId,
                ':item' => $itemRowId,
                ':download' => $itemDownload,
                ':price' => (string) ($item['price_id'] ?? ''),
                ':product' => (string) $offer['public_code'],
                ':license_type' => (string) $offer['license_type_ref'],
                ':state' => self::STATE_BLOCKED,
                ':reason' => $code,
                ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':digest' => $digest,
                ':created' => $now,
                ':retention' => $retention,
                ':updated' => $now,
            ]);
        }
    }

    private function bindingEntry(array $binding, bool $existing): array
    {
        return [
            'binding_key' => (string) $binding['binding_key'],
            'order_item_id' => (int) $binding['order_item_id'],
            'download_id' => (int) $binding['download_id'],
            'product_code' => (string) $binding['product_code'],
            'price_id' => (string) $binding['price_id'],
            'license_type_ref' => (string) $binding['license_type_ref'],
            'issuance_request_handle' => (string) $binding['issuance_request_key'],
            'existing' => $existing,
        ];
    }

    private function outOfOrderDecision(string $registrationUuid, int $orderId, string $accountId, string $blockedReason, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'out_of_order',
            'order_id' => $orderId,
            'registration_id' => $registrationUuid,
            'account_id' => $accountId,
            'blocked_reason' => $blockedReason,
            'issuance' => 'none',
            'issuance_requests_settled' => 0,
            'entitlement_allowed' => false,
            'existing' => $existing,
        ];
    }

    /** Exact account binding: the order customer is the registration's promoted customer. */
    private function assertAccountBinding(array $registration, int $customerId): string
    {
        $orderAccount = $this->accounts->findByCustomerId($customerId);
        $accountId = (string) $registration['account_uuid'];
        if ((int) $registration['edd_customer_id'] !== $customerId
            || $orderAccount === null
            || !hash_equals($accountId, (string) $orderAccount['account_uuid'])) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }
        return $accountId;
    }

    /** Exact verified-email binding: the canonical order email matches the registration. */
    private function assertOrderEmailBinding(array $registration, array $orderRow): void
    {
        $orderEmail = (string) ($orderRow['email'] ?? '');
        if ($orderEmail === '') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderDigest = $this->emailLookupDigest($orderEmail);
        if (!hash_equals((string) $registration['email_lookup_digest'], $orderDigest)) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }
    }

    /**
     * Payment binding: every submitted payment transaction must be a real, complete
     * wp_edd_order_transactions row of this exact order, with a non-synthetic
     * transaction identity. Synthetic or unlinked payment IDs fail closed and never
     * issue. Returns the first matched payment identity for journaling.
     */
    private function assertPaymentBoundToOrder(int $orderId, array $transactions): array
    {
        if (!is_array($transactions) || $transactions === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $table = $this->eddPrefix . 'edd_order_transactions';
        $matched = [];
        foreach ($transactions as $transaction) {
            if (!is_array($transaction)) {
                throw new InvalidArgumentException('malformed payment transaction');
            }
            $gateway = (string) ($transaction['gateway'] ?? '');
            $transactionId = (string) ($transaction['transaction_id'] ?? '');
            $status = (string) ($transaction['status'] ?? '');
            if ($gateway === '' || $transactionId === '' || $status === '') {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            $this->assertToken($gateway, 64, 'payment gateway');
            $this->assertToken($transactionId, 191, 'payment transaction');
            if ($this->isSyntheticPayment($transactionId, $gateway)) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            if ($status !== 'complete') {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            $statement = $this->db->prepare("SELECT 1 FROM {$table}
                WHERE order_id = :order AND gateway = :gateway AND transaction_id = :txn AND status = 'complete' LIMIT 1");
            $statement->execute([
                ':order' => $orderId,
                ':gateway' => $gateway,
                ':txn' => $transactionId,
            ]);
            if ($statement->fetchColumn() === false) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            $matched[] = [
                'gateway' => $gateway,
                'transaction_digest' => $this->transactionDigest($gateway, $transactionId),
            ];
        }
        if ($matched === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return $matched[0];
    }

    /** Canonical order-item binding: the item row exists, belongs to this order, and is the exact download. */
    private function assertCanonicalOrderItem(int $orderId, int $orderItemId, int $downloadId): void
    {
        $table = $this->eddPrefix . 'edd_order_items';
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :item LIMIT 1");
        $statement->execute([':item' => $orderItemId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false
            || (int) $row['order_id'] !== $orderId
            || (int) $row['product_id'] !== $downloadId
            || (int) $row['quantity'] < 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
    }

    private function findOrderRow(int $orderId): ?array
    {
        $table = $this->eddPrefix . 'edd_orders';
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findBindingByTriple(string $registrationUuid, int $orderId, int $orderItemId): ?array
    {
        $table = $this->schema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE registration_uuid = :registration AND order_id = :order AND order_item_id = :item LIMIT 1");
        $statement->execute([
            ':registration' => $registrationUuid,
            ':order' => $orderId,
            ':item' => $orderItemId,
        ]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findBlockedForOrder(string $registrationUuid, int $orderId): ?array
    {
        $table = $this->schema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE registration_uuid = :registration AND order_id = :order AND binding_state = 'blocked' LIMIT 1");
        $statement->execute([':registration' => $registrationUuid, ':order' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /**
     * Resolve an EDD download to a bounded gate disposition:
     *   'protected'        → operator-approved protected offer (server-owned mapping)
     *   'credit_pack'      → excluded from entitlement forever
     *   'non_entitlement'  → unrelated/quarantined product: purchasable, never entitlement
     *   'unknown'          → not in the catalog and not mapped: fails closed
     */
    private function resolveDownloadMapping(int $downloadId): array
    {
        $entry = null;
        foreach (($this->productRegistry['current_edd_catalog']['entries'] ?? []) as $candidate) {
            if ((int) $candidate['download_id'] === $downloadId) {
                $entry = $candidate;
                break;
            }
        }
        if ($entry !== null) {
            $disposition = (string) ($entry['entitlement_disposition'] ?? 'unknown');
            if ($disposition === 'retire'
                && str_starts_with((string) ($entry['reason'] ?? ''), self::CREDIT_PACK_REASON_PREFIX)) {
                return ['download_id' => $downloadId, 'disposition' => 'credit_pack', 'entry' => $entry, 'reason' => $entry['reason'] ?? '', 'offer' => null];
            }
            if ($disposition === self::UNRELATED_DISPOSITION) {
                return ['download_id' => $downloadId, 'disposition' => 'non_entitlement', 'entry' => $entry, 'reason' => $entry['reason'] ?? '', 'offer' => null];
            }
        }
        $offer = $this->findActiveOfferByDownload($downloadId);
        if ($offer !== null) {
            return ['download_id' => $downloadId, 'disposition' => 'protected', 'entry' => $entry, 'reason' => null, 'offer' => $offer];
        }
        return ['download_id' => $downloadId, 'disposition' => 'unknown', 'entry' => $entry, 'reason' => null, 'offer' => null];
    }

    /** Server-owned mapping lookup: an offer resolves only through the registry. */
    private function findActiveOfferByDownload(int $downloadId): ?array
    {
        foreach (($this->productRegistry['protected_offers'] ?? []) as $offer) {
            if ((int) ($offer['edd_download_id'] ?? 0) === $downloadId) {
                return $offer;
            }
        }
        return null;
    }

    /** Registered-facade allowlist: exact origin and exact supported product allowlist. */
    private function assertFacadeSupports(string $facadeId, string $origin, string $productCode): void
    {
        if ($facadeId === '' || $origin === '') {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        $facade = null;
        foreach (($this->facadeRegistry['facades'] ?? []) as $candidate) {
            if (hash_equals((string) ($candidate['facade_id'] ?? ''), $facadeId)) {
                $facade = $candidate;
                break;
            }
        }
        if ($facade === null) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        $originAllowed = false;
        foreach (($facade['exact_origins'] ?? []) as $candidate) {
            if (is_string($candidate) && hash_equals($candidate, $origin)) {
                $originAllowed = true;
                break;
            }
        }
        if (!$originAllowed) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if ($productCode !== '' && !in_array($productCode, ($facade['products'] ?? []), true)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }
    }

    /**
     * Verified registration binding: the registration must be mailbox-verified,
     * non-terminal, unexpired, bound to the exact facade, and promoted (EDD customer
     * bound). Missing, malformed, or unknown registrations fail closed with
     * EMAIL_VERIFICATION_REQUIRED; verified-but-unpromoted fails with
     * EDD_CUSTOMER_RESOLUTION_FAILED.
     */
    private function assertVerifiedRegistration(string $registrationUuid, string $facadeId): array
    {
        if ($registrationUuid === ''
            || preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $registrationUuid) !== 1) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
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
        if ($facadeId !== '' && !hash_equals((string) $registration['facade_id'], $facadeId)) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        if ($registration['edd_customer_id'] === null) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }
        return $registration;
    }

    /** No existing equivalent active license: duplicates fail closed unless policy allows. */
    private function hasEquivalentActiveLicense(int $customerId, int $downloadId): bool
    {
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT 1 FROM {$table}
            WHERE customer_id = :customer AND product_id = :download AND status = 'active' LIMIT 1");
        $statement->execute([':customer' => $customerId, ':download' => $downloadId]);
        return $statement->fetchColumn() !== false;
    }

    /** Reject any caller-supplied grant/price/limit selection; only server-owned registry decides. */
    private function assertNoCallerControlledGrantFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function isSyntheticPayment(string $transactionId, string $gateway): bool
    {
        if ($transactionId === '0' || $transactionId === 'none') {
            return true;
        }
        foreach (self::SYNTHETIC_PAYMENT_MARKERS as $marker) {
            if (str_starts_with($transactionId, $marker) || str_starts_with($gateway, $marker)) {
                return true;
            }
        }
        return false;
    }

    private function presentedPaymentDigests(array $transactions): array
    {
        $digests = [];
        if (!is_array($transactions)) {
            return $digests;
        }
        foreach ($transactions as $transaction) {
            if (!is_array($transaction)) {
                continue;
            }
            $gateway = (string) ($transaction['gateway'] ?? '');
            $transactionId = (string) ($transaction['transaction_id'] ?? '');
            if ($gateway !== '' && $transactionId !== '') {
                $digests[] = $this->transactionDigest($gateway, $transactionId);
            }
        }
        return $digests;
    }

    private function transactionDigest(string $gateway, string $transactionId): string
    {
        return hash('sha256', "focusa.spec152e.edd_order_binding.payment.v1\0" . $gateway . "\0" . $transactionId);
    }

    private function emailLookupDigest(string $email): string
    {
        return $this->registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($email));
    }

    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE idempotency_key = :key AND binding_state = :state LIMIT 1");
        $statement->execute([':key' => $idempotencyKey, ':state' => self::STATE_SETTLED]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddOrderBindingMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddOrderBindingMigration::encodeCanonical($value));
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

    private function assertPositiveInt(mixed $value, string $field): int
    {
        if (!is_int($value) || $value < 1) {
            throw new InvalidArgumentException("positive {$field} required");
        }
        return $value;
    }

    private function assertToken(string $value, int $max, string $kind): void
    {
        if ($value === '' || strlen($value) > $max || preg_match('/[\r\n\x00]/', $value) === 1) {
            throw new InvalidArgumentException("bounded {$kind} token required");
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
