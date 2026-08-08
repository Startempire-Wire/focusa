<?php
// Spec 152E canonical EDD Software Licensing key issuance (addendum sections 8, 11, 15,
// 16, 17, and 3.2 mapping gaps). Consumes exactly one settled issuance request from the
// order-binding journal and produces exactly ONE canonical EDD Software Licensing key per
// eligible order item, linked to the verified account/registration and the canonical EDD
// order/item rows:
//
//   - Issuance starts only from a settled_pending_issuance binding plus a pending
//     issuance request journaled by the order-completion binding service. The canonical
//     EDD order row (status complete, exact customer, exact verified email digest) and the
//     exact canonical order item row are re-verified at issuance time; refunded/revoked/
//     pending canonical truth fails closed and never issues.
//   - One eligible order item produces exactly one canonical EDD SL key, forever. A replay
//     with the same idempotency key returns the identical decision; re-issuance for an
//     already-issued request (for example a delivery retry with a new key) returns the
//     same canonical key with existing=true and keys_created=0. No second key is ever
//     created for the same issuance request, binding, or order item.
//   - Custom duplicate keys are impossible: the adapter never creates focusa_live_* or any
//     synthetic-prefixed key (all issued keys are standard EDD SL format), and it fails
//     closed with EDD_LICENSE_UNUSABLE when any synthetic legacy key (focusa_live_*,
//     synthetic_*, local_*, eval_*) or any existing active license already exists for the
//     same customer/download item. Legacy synthetic rows are quarantine migration input,
//     never co-equal authority, and are preserved — never deleted, never overwritten.
//   - Registration fulfillment: after the license is journaled, the registration advances
//     checkout_pending -> entitlement_issued with the canonical edd_order_id,
//     edd_order_item_id, and edd_license_id references (idempotent by the issuance
//     idempotency key). Registrations that never entered checkout (not checkout_pending)
//     fail closed with EDD_CHECKOUT_REQUIRED and never issue.
//   - No plaintext leakage: journals store only the 64-hex license-key digest plus a
//     masked key; the full key is returned exactly once in the bounded fulfillment
//     delivery envelope (schema focusa.spec152e.edd_license_delivery.v1) to the server-side
//     email/terminal delivery path and is never written to any journal. No raw email, raw
//     payment transaction id, secret, or unmasked real-email evidence is stored or
//     returned.
//
// Failures are public-safe stable codes (EDD_ORDER_PENDING, EDD_ORDER_UNVERIFIED,
// REFUNDED, REVOKED, EMAIL_VERIFICATION_REQUIRED, REGISTRATION_EXPIRED,
// EDD_CUSTOMER_RESOLUTION_FAILED, ACCOUNT_MERGE_REVIEW_REQUIRED, ACCOUNT_EMAIL_MISMATCH,
// FACADE_ORIGIN_DENIED, PRODUCT_MAPPING_REQUIRED, EDD_CHECKOUT_REQUIRED,
// EDD_LICENSE_UNUSABLE, EDD_LICENSE_PENDING, CLIENT_COMMERCIAL_FIELDS_FORBIDDEN,
// IDEMPOTENCY_CONFLICT). No new error code is introduced.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php,
// docs/contracts/spec152e-verified-registration-token-validator.v1.php,
// docs/contracts/spec152e-edd-order-binding.v1.php, and the server-owned product registry
// to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddLicenseIssuanceMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_license_issuance.v1';
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
        $issuances = $this->table('wpuiai_edd_license_issuances');
        $migrations = $this->table('wpuiai_edd_license_issuance_schema_migrations');
        $events = $this->table('wpuiai_edd_license_issuance_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$issuances} (
            issuance_key VARCHAR(64) NOT NULL PRIMARY KEY,
            issuance_request_key VARCHAR(64) NOT NULL,
            binding_key VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NULL,
            customer_id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            order_item_id BIGINT NOT NULL,
            download_id BIGINT NOT NULL,
            product_code VARCHAR(128) NOT NULL,
            license_type_ref VARCHAR(128) NOT NULL,
            edd_license_id BIGINT NOT NULL,
            license_key_digest VARCHAR(64) NOT NULL,
            license_key_mask VARCHAR(64) NOT NULL,
            state VARCHAR(16) NOT NULL CHECK (state IN ('issued')),
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (issuance_request_key)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_license_issuance_idempotency
            ON {$issuances} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_license_issuance_retention
            ON {$issuances} (retention_until)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_license_issuance_customer
            ON {$issuances} (customer_id, download_id, state)");
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

    /** Rollback is preservation-only: issuance journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_license_issuance_schema_events');
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

final class FocusaSpec152eEddLicenseIssuanceService
{
    public const SCHEMA = 'focusa.spec152e.edd_license_issuance.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.edd_license_issuance_decision.v1';
    public const DELIVERY_SCHEMA = 'focusa.spec152e.edd_license_delivery.v1';
    public const VERSION = 1;
    public const RETENTION_SECONDS = 2592000;

    /** Canonical EDD Software Licensing key shape: four 8-char uppercase hex groups. */
    public const KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';

    private const SYNTHETIC_KEY_PREFIXES = ['focusa_live_', 'synthetic_', 'local_', 'eval_'];
    private const REQUEST_STATE_PENDING = 'pending';
    private const REQUEST_STATE_ISSUED = 'issued';
    private const BINDING_STATE_SETTLED = 'settled_pending_issuance';
    private const BINDING_STATE_BLOCKED = 'blocked';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'tier', 'products', 'product_code', 'license_type',
        'license_type_ref', 'capability_family', 'families', 'features', 'grants', 'limits',
        'node_limit', 'activation_limit', 'sale_status', 'refund_policy', 'upgrade_policy',
        'commercial_rights', 'evaluation_duration', 'edd_download_id', 'edd_price_id',
        'license_key', 'license_duration', 'expiration',
    ];

    private const REGISTRATION_FULFILLED_STATES = [
        'entitlement_issued', 'terminal_delivery_ready', 'device_registered', 'lease_issued',
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddLicenseIssuanceMigration $schema,
        private FocusaSpec152eEddOrderBindingMigration $bindingSchema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private array $productRegistry,
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
     * Issue the canonical EDD Software Licensing key for exactly one settled issuance
     * request. One eligible order item produces exactly one key, linked to the verified
     * account/registration and the canonical EDD order/item rows; duplicates, replays, and
     * synthetic legacy keys never produce a second key.
     *
     * Required input:
     *   - issuance_request_handle: the opaque ir_ handle journaled by the order-binding
     *     service (exactly one pending issuance request)
     *   - request_id, idempotency_key
     *
     * Returns a public-safe decision. The full key appears only inside the bounded
     * fulfillment delivery envelope (never in journals); replays return the identical
     * decision; a re-issuance for an already-issued request returns the same key with
     * existing=true and zero keys created.
     */
    public function issue(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $issuanceRequestKey = (string) ($input['issuance_request_handle'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        if (preg_match('/^(ir_)[0-9a-f]{32}$/D', $issuanceRequestKey) !== 1) {
            throw new InvalidArgumentException('bounded issuance request handle required');
        }
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $digest = $this->requestDigest([
            'operation' => 'canonical_license_issuance',
            'issuance_request_handle' => $issuanceRequestKey,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $request = $this->loadIssuanceRequest($issuanceRequestKey);
        $binding = $this->loadBinding((string) $request['binding_key'], $issuanceRequestKey);
        if ($binding['binding_state'] === self::BINDING_STATE_BLOCKED) {
            throw new DomainException((string) ($binding['blocked_reason'] ?? 'EDD_LICENSE_UNUSABLE'));
        }
        if ($binding['binding_state'] !== self::BINDING_STATE_SETTLED) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }

        // Already issued: a delivery retry with a different idempotency key returns the
        // same canonical key with zero keys created. Never a second key.
        if ($request['state'] === self::REQUEST_STATE_ISSUED) {
            return $this->existingIssuedDecision($issuanceRequestKey);
        }
        if ($request['state'] !== self::REQUEST_STATE_PENDING) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }

        $registration = $this->assertIssuanceRegistration((string) $request['registration_uuid'], $binding, $request);

        // Canonical EDD order truth is authoritative at issuance time.
        $downloadId = (int) $binding['download_id'];
        $this->assertCanonicalOrder((int) $request['order_id'], (int) $request['customer_id'], $registration);
        $this->assertCanonicalOrderItem((int) $request['order_id'], (int) $request['order_item_id'], $downloadId);

        // Server-owned offer is the only grant/price/duration authority for this item.
        $offer = $this->assertOfferMapping($request, $binding, $downloadId);

        // Duplicate-key regression: no canonical key is ever created next to an existing
        // active license or any synthetic legacy key for the same customer/download item.
        // An active canonical license from the same order is a sibling order-item key (one
        // key per eligible order item) and is allowed; any other active license or any
        // synthetic key (focusa_live_* or prefix) blocks issuance and is preserved.
        $this->assertNoExistingLicenseForItem((int) $request['customer_id'], $downloadId, (int) $request['order_id']);

        $licenseKey = $this->generateUniqueKey();
        $licenseId = $this->issueCanonicalLicense($request, $binding, $offer, $licenseKey, $digest, $requestId, $idempotencyKey);

        // Registration fulfillment: checkout_pending -> entitlement_issued with the
        // canonical license reference. Idempotent by the issuance idempotency key. A
        // registration that already advanced (same-order sibling item) is left untouched;
        // a registration that never entered checkout was already rejected.
        if ((string) $registration['state'] === FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING) {
            $this->fulfillRegistration($registration, (int) $request['order_id'], (int) $request['order_item_id'], $licenseId, $requestId, $idempotencyKey);
        }

        return $this->decision($request, $downloadId, $licenseId, $licenseKey, false);
    }

    /** Bounded journal lookups for settlement/reconciliation. */
    public function issuanceCount(): int
    {
        $table = $this->schema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}");
        $statement->execute();
        return (int) $statement->fetchColumn();
    }

    /** Bounded: exact issuance lookup by opaque issuance handle (ki_). */
    public function findByIssuanceKey(string $issuanceKey): ?array
    {
        $this->assertToken($issuanceKey, 64, 'issuance');
        $table = $this->schema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Bounded: exact issuance lookup by the source issuance-request handle (ir_). */
    public function findByIssuanceRequestKey(string $issuanceRequestKey): ?array
    {
        $this->assertToken($issuanceRequestKey, 64, 'issuance request');
        $table = $this->schema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function issueCanonicalLicense(
        array $request,
        array $binding,
        array $offer,
        string $licenseKey,
        string $digest,
        string $requestId,
        string $idempotencyKey,
    ): int {
        $now = $this->now();
        $issuanceKey = self::opaqueToken('ki_');
        $licenses = $this->edd->table('edd_licenses');
        $requests = $this->bindingSchema->table('wpuiai_edd_issuance_requests');
        $issuances = $this->schema->table('wpuiai_edd_license_issuances');
        $duration = self::offerDurationFields($offer);
        $downloadId = (int) $binding['download_id'];
        $this->db->beginTransaction();
        try {
            $statement = $this->db->prepare("INSERT INTO {$licenses}
                (license_key, customer_id, user_id, product_id, order_id, license_length,
                 license_unit, expiration, activation_count, activation_limit, status, date_created)
                VALUES (:key, :customer, NULL, :download, :order, :length,
                        :unit, :expiration, 0, :activation_limit, 'active', :created)");
            $statement->execute([
                ':key' => $licenseKey,
                ':customer' => (int) $request['customer_id'],
                ':download' => $downloadId,
                ':order' => (int) $request['order_id'],
                ':length' => $duration['license_length'],
                ':unit' => $duration['license_unit'],
                ':expiration' => $duration['expiration'],
                ':activation_limit' => (int) ($offer['node_limit'] ?? 0),
                ':created' => $now,
            ]);
            $licenseId = (int) $this->db->lastInsertId();

            $requestStatement = $this->db->prepare("UPDATE {$requests}
                SET state = :state WHERE issuance_request_key = :key AND state = :expected");
            $requestStatement->execute([
                ':state' => self::REQUEST_STATE_ISSUED,
                ':key' => (string) $request['issuance_request_key'],
                ':expected' => self::REQUEST_STATE_PENDING,
            ]);
            if ($requestStatement->rowCount() !== 1) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }

            $decision = $this->decision($request, $downloadId, $licenseId, $licenseKey, false);
            $stored = $decision;
            unset($stored['delivery']);
            $retention = self::plusSeconds($now, $this->retention);
            $issuanceStatement = $this->db->prepare("INSERT INTO {$issuances}
                (issuance_key, issuance_request_key, binding_key, registration_uuid, account_uuid,
                 customer_id, order_id, order_item_id, download_id, product_code, license_type_ref,
                 edd_license_id, license_key_digest, license_key_mask, state, result_payload,
                 request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
                VALUES (:issuance, :request_key, :binding, :registration, :account,
                        :customer, :order, :item, :download, :product, :license_type,
                        :license_id, :digest, :mask, 'issued', :payload,
                        :request, :idempotency, :request_digest, :created, :retention, :updated)");
            $issuanceStatement->execute([
                ':issuance' => $issuanceKey,
                ':request_key' => (string) $request['issuance_request_key'],
                ':binding' => (string) $binding['binding_key'],
                ':registration' => (string) $request['registration_uuid'],
                ':account' => (string) ($request['account_uuid'] ?? ''),
                ':customer' => (int) $request['customer_id'],
                ':order' => (int) $request['order_id'],
                ':item' => (int) $request['order_item_id'],
                ':download' => $downloadId,
                ':product' => (string) $request['product_code'],
                ':license_type' => (string) $request['license_type_ref'],
                ':license_id' => $licenseId,
                ':digest' => $this->keyDigest($licenseKey),
                ':mask' => self::maskKey($licenseKey),
                ':payload' => json_encode($stored, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':retention' => $retention,
                ':updated' => $now,
            ]);
            $this->db->commit();
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
        return $licenseId;
    }

    /**
     * Registration fulfillment: advance checkout_pending -> entitlement_issued with the
     * canonical order/item/license references, idempotently by the issuance idempotency
     * key. A registration that already advanced (for example on a delivery retry) is left
     * untouched; a registration that never entered checkout was already rejected by
     * assertIssuanceRegistration.
     */
    private function fulfillRegistration(array $registration, int $orderId, int $orderItemId, int $licenseId, string $requestId, string $idempotencyKey): void
    {
        $state = (string) $registration['state'];
        if ($state !== FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING) {
            return;
        }
        try {
            $this->registrations->transition(
                (string) $registration['registration_uuid'],
                $state,
                FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
                (int) $registration['state_version'],
                $requestId,
                $idempotencyKey,
                [
                    'state_reason' => 'canonical_edd_license_issued',
                    'edd_order_id' => $orderId,
                    'edd_order_item_id' => $orderItemId,
                    'edd_license_id' => $licenseId,
                ],
            );
        } catch (DomainException $error) {
            // The registration already advanced past checkout_pending; the license is
            // journaled and the same key is returned. Any other failure propagates after
            // issuance; a retry with the same key replays the stored decision.
            if ($error->getMessage() === 'REGISTRATION_STATE_CONFLICT') {
                return;
            }
            throw $error;
        }
    }

    /** The issuance request must exist in the order-binding journal and be pending/issued. */
    private function loadIssuanceRequest(string $issuanceRequestKey): array
    {
        $table = $this->bindingSchema->table('wpuiai_edd_issuance_requests');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    /** The binding must be settled for this exact issuance request (or journaled terminal). */
    private function loadBinding(string $bindingKey, string $issuanceRequestKey): array
    {
        $table = $this->bindingSchema->table('wpuiai_edd_order_bindings');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE binding_key = :key LIMIT 1");
        $statement->execute([':key' => $bindingKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || !hash_equals((string) ($row['issuance_request_key'] ?? ''), $issuanceRequestKey)) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $row;
    }

    /**
     * Registration must still be mailbox-verified and bound to the exact account/customer/
     * facade of the settled request. The fulfillment state is per registration: the first
     * order item advances checkout_pending -> entitlement_issued; sibling items of the
     * same order (one canonical key per eligible order item) may still issue while the
     * registration is in a non-terminal fulfillment state and are left untouched by the
     * transition. Registrations that never entered checkout fail closed with
     * EDD_CHECKOUT_REQUIRED; terminal/delivered registrations fail closed.
     */
    private function assertIssuanceRegistration(string $registrationUuid, array $binding, array $request): array
    {
        try {
            $registration = $this->registrations->findByUuid($registrationUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ((string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $state = (string) $registration['state'];
        if (in_array($state, self::REGISTRATION_FULFILLED_STATES, true)) {
            // Already fulfilled by a sibling order item of the same order; no transition.
        } elseif (in_array($state, FocusaSpec152eVerifiedRegistrationTokenValidator::VERIFIED_NONTERMINAL_STATES, true)) {
            if ($state !== FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING) {
                throw new DomainException('EDD_CHECKOUT_REQUIRED');
            }
        } else {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $now = $this->now();
        if ($state === FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING
            && $now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        $accountId = (string) ($request['account_uuid'] ?? '');
        if ($accountId === ''
            || !hash_equals($accountId, (string) $registration['account_uuid'])
            || (int) $registration['edd_customer_id'] !== (int) $request['customer_id']) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }
        if (!hash_equals((string) ($binding['facade_id'] ?? ''), (string) $registration['facade_id'])) {
            throw new DomainException('FACADE_ORIGIN_DENIED');
        }
        return $registration;
    }

    /** Canonical EDD order truth: complete status, exact customer, exact verified email digest. */
    private function assertCanonicalOrder(int $orderId, int $customerId, array $registration): void
    {
        $table = $this->eddPrefix . 'edd_orders';
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $status = (string) ($row['status'] ?? '');
        if (in_array($status, ['refunded', 'revoked'], true)) {
            throw new DomainException(strtoupper($status));
        }
        if (in_array($status, ['pending', 'processing'], true)) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        if ($status !== 'complete') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((int) $row['customer_id'] !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderEmail = (string) ($row['email'] ?? '');
        if ($orderEmail === '') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderDigest = $this->registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($orderEmail));
        if (!hash_equals((string) $registration['email_lookup_digest'], $orderDigest)) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }
    }

    /** Canonical order-item binding: the item row exists, belongs to this order, exact download. */
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

    /**
     * Server-owned offer resolution at issuance time: the offer resolves by the settled
     * product code and must still carry the exact download, price, license type, and an
     * operator-approved active/checkout-enabled mapping. Caller metadata never selects any
     * grant, price, duration, or limit.
     */
    private function assertOfferMapping(array $request, array $binding, int $downloadId): array
    {
        $offer = null;
        foreach (($this->productRegistry['protected_offers'] ?? []) as $candidate) {
            if (hash_equals((string) ($candidate['public_code'] ?? ''), (string) $request['product_code'])) {
                $offer = $candidate;
                break;
            }
        }
        if ($offer === null
            || (int) ($offer['edd_download_id'] ?? 0) !== $downloadId
            || !hash_equals((string) ($offer['license_type_ref'] ?? ''), (string) $request['license_type_ref'])
            || !hash_equals((string) ($offer['edd_price_id'] ?? ''), (string) ($binding['price_id'] ?? ''))) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (($offer['mapping_status'] ?? '') !== 'active' || ($offer['checkout_enabled'] ?? false) !== true) {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        self::offerDurationFields($offer); // validates the duration is issuable
        return $offer;
    }

    /**
     * Duplicate-key regression: a canonical key is never created when an active license
     * already exists for the same customer/download item from another order, and never
     * next to a synthetic legacy key (focusa_live_* or any synthetic prefix) regardless
     * of its status. An active canonical license from the same order is a sibling
     * order-item key (one key per eligible order item) and never blocks. Legacy synthetic
     * rows are preserved for migration — never deleted, never overwritten.
     */
    private function assertNoExistingLicenseForItem(int $customerId, int $downloadId, int $orderId): void
    {
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT license_key, status, order_id FROM {$table}
            WHERE customer_id = :customer AND product_id = :download ORDER BY id");
        $statement->execute([':customer' => $customerId, ':download' => $downloadId]);
        foreach ($statement->fetchAll(PDO::FETCH_ASSOC) as $row) {
            $key = (string) $row['license_key'];
            if ($this->isSyntheticKey($key)) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }
            if ((string) $row['status'] === 'active' && (int) $row['order_id'] !== $orderId) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }
        }
    }

    /** Re-issuance for an already-issued request: same canonical key, zero keys created. */
    private function existingIssuedDecision(string $issuanceRequestKey): array
    {
        $row = $this->findByIssuanceRequestKey($issuanceRequestKey);
        if ($row === null) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $licenseKey = $this->loadCanonicalKey((int) $row['edd_license_id'], (string) $row['license_key_digest']);
        $request = $this->loadIssuanceRequest($issuanceRequestKey);
        return $this->decision($request, (int) $row['download_id'], (int) $row['edd_license_id'], $licenseKey, true);
    }

    /** Idempotent replay: same key returns the identical decision (rebuilt from canonical truth). */
    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        $licenseKey = $this->loadCanonicalKey((int) $row['edd_license_id'], (string) $row['license_key_digest']);
        $decision = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $decision['delivery'] = [
            'schema' => self::DELIVERY_SCHEMA,
            'license_key' => $licenseKey,
            'channel' => 'bounded_fulfillment_handoff',
        ];
        return $decision;
    }

    /** Canonical EDD Software Licensing storage is the only key source for replays/retries. */
    private function loadCanonicalKey(int $licenseId, string $expectedDigest): string
    {
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT license_key FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $licenseId]);
        $key = $statement->fetchColumn();
        if ($key === false || !hash_equals($expectedDigest, $this->keyDigest((string) $key))) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return (string) $key;
    }

    /** Standard EDD SL key generation; never a synthetic prefix; unique in wp_edd_licenses. */
    private function generateUniqueKey(): string
    {
        $table = $this->eddPrefix . 'edd_licenses';
        for ($attempt = 0; $attempt < 5; $attempt++) {
            $key = strtoupper(bin2hex(random_bytes(16)));
            $formatted = implode('-', str_split($key, 8));
            $statement = $this->db->prepare("SELECT 1 FROM {$table} WHERE license_key = :key LIMIT 1");
            $statement->execute([':key' => $formatted]);
            if ($statement->fetchColumn() === false) {
                return $formatted;
            }
        }
        throw new DomainException('EDD_LICENSE_UNUSABLE');
    }

    private function decision(array $request, int $downloadId, int $licenseId, string $licenseKey, bool $existing): array
    {
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'license_issued',
            'registration_id' => (string) $request['registration_uuid'],
            'account_id' => (string) ($request['account_uuid'] ?? ''),
            'customer_id' => (int) $request['customer_id'],
            'order_id' => (int) $request['order_id'],
            'order_item_id' => (int) $request['order_item_id'],
            'download_id' => $downloadId,
            'product_code' => (string) $request['product_code'],
            'license_type_ref' => (string) $request['license_type_ref'],
            'edd_license_id' => $licenseId,
            'license_key_digest' => $this->keyDigest($licenseKey),
            'license_key_mask' => self::maskKey($licenseKey),
            'issuance' => 'canonical_edd_software_licensing',
            'keys_created' => $existing ? 0 : 1,
            'existing' => $existing,
            'delivery' => [
                'schema' => self::DELIVERY_SCHEMA,
                'license_key' => $licenseKey,
                'channel' => 'bounded_fulfillment_handoff',
            ],
        ];
    }

    /** Offer duration -> EDD SL license fields; only server-owned registry durations issue. */
    private static function offerDurationFields(array $offer): array
    {
        $duration = (string) ($offer['license_duration'] ?? '');
        if ($duration === 'lifetime') {
            return ['license_length' => 0, 'license_unit' => 'years', 'expiration' => null];
        }
        throw new DomainException('PRODUCT_MAPPING_REQUIRED');
    }

    private function isSyntheticKey(string $key): bool
    {
        foreach (self::SYNTHETIC_KEY_PREFIXES as $prefix) {
            if (str_starts_with($key, $prefix)) {
                return true;
            }
        }
        return false;
    }

    private function keyDigest(string $licenseKey): string
    {
        return hash('sha256', "focusa.spec152e.edd_license_issuance.key.v1\0" . $licenseKey);
    }

    private static function maskKey(string $licenseKey): string
    {
        $parts = explode('-', $licenseKey);
        $tail = (string) end($parts);
        return '********-********-********-' . substr($tail, -4);
    }

    private function assertNoCallerControlledGrantFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddLicenseIssuanceMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddLicenseIssuanceMigration::encodeCanonical($value));
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
