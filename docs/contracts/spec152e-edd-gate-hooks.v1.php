<?php
// Spec 152E EDD gate hooks. Replaces the broad edd_complete_purchase hook and gates the
// EDD add-to-cart and checkout surfaces for Focusa/UIAI protected products:
//
//   - Protected offers exist only in the server-owned product registry; a mapping must be
//     explicitly operator-approved (mapping_status 'active', checkout_enabled, exact
//     download and price). No caller metadata ever selects a product, price, grant, limit,
//     or commercial right.
//   - Raw add-to-cart, unknown downloads, credit packs, and wrong-facade paths can never
//     reach Focusa/UIAI entitlement. Unrelated/quarantined EDD products remain purchasable
//     but are proven non-entitlement: no Focusa/UIAI grant can attach.
//   - Protected add-to-cart and checkout require a single-use verified-registration gate
//     token (or a journaled cart-gate binding) bound to the exact registration, facade, and
//     product.
//   - Order completion requires complete order status, verified registration/account
//     binding (order email must match the verified registration email digest), allowlisted
//     product mapping with the exact order-item price relationship, idempotent issuance
//     state, and no existing equivalent active license. Issuance itself stays deferred to
//     the verified issuance service: this hook only records a bounded decision.
//
// Failures are public-safe stable codes. No raw email, raw token, license key, or secret is
// ever returned or logged. Rollback is preservation-only.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php, and
// docs/contracts/spec152e-verified-registration-token-validator.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eEddGateDecisionMigration
{
    public const SCHEMA = 'focusa.spec152e.edd_gate_decision.v1';
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
        $decisions = $this->table('wpuiai_edd_gate_decisions');
        $migrations = $this->table('wpuiai_edd_gate_decision_schema_migrations');
        $events = $this->table('wpuiai_edd_gate_decision_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$decisions} (
            decision_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(32) NOT NULL CHECK (operation IN ('cart_gate', 'checkout_gate', 'order_complete_gate')),
            registration_uuid {$uuid} NULL,
            facade_id VARCHAR(96) NULL,
            product_code VARCHAR(128) NULL,
            download_id BIGINT NULL,
            order_id BIGINT NULL,
            decision VARCHAR(32) NOT NULL,
            error_code VARCHAR(64) NULL,
            state_reason VARCHAR(191) NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_gate_decision_registration
            ON {$decisions} (registration_uuid, operation, created_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_gate_decision_cart_binding
            ON {$decisions} (registration_uuid, download_id, operation)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_edd_gate_decision_idempotency
            ON {$decisions} (idempotency_key)");
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

    /** Rollback is preservation-only: gate decision journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_edd_gate_decision_schema_events');
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

final class FocusaSpec152eEddGateHooks
{
    public const SCHEMA = 'focusa.spec152e.edd_gate_hooks.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.edd_gate_decision.v1';
    public const VERSION = 1;
    public const RETENTION_SECONDS = 2592000;

    private const CREDIT_PACK_REASON_PREFIX = 'credit_pack_';
    private const UNRELATED_DISPOSITION = 'quarantine';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEddGateDecisionMigration $schema,
        private FocusaSpec152eVerifiedRegistrationTokenValidator $tokens,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
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
     * EDD add-to-cart gate (edd_add_to_cart surface). Denies raw, unknown, credit-pack, and
     * wrong-facade paths for protected offers; allows unrelated non-entitlement products
     * without ever attaching Focusa/UIAI entitlement; allows a protected cart only with a
     * single-use verified-registration gate token bound to the exact registration, facade,
     * and product.
     *
     * Required input:
     *   - download_id (int), facade_id, origin, product_code (client claim)
     *   - registration_uuid, verified_token (required for protected downloads)
     *   - request_id, idempotency_key
     */
    public function gateAddToCart(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $downloadId = $this->assertPositiveInt($input['download_id'] ?? null, 'download_id');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $productCode = (string) ($input['product_code'] ?? '');
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $rawToken = (string) ($input['verified_token'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $digest = $this->requestDigest([
            'operation' => 'cart_gate',
            'download_id' => $downloadId,
            'facade_id' => $facadeId,
            'origin' => $origin,
            'product_code' => $productCode,
            'registration_uuid' => $registrationUuid,
            'verified_token_hash' => $rawToken === '' ? '' : FocusaSpec152eVerifiedRegistrationTokenValidator::tokenHash($rawToken),
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision('cart_gate', $idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $mapping = $this->resolveDownloadMapping($downloadId);
        if ($mapping['disposition'] === 'credit_pack' || $mapping['disposition'] === 'unknown') {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if ($mapping['disposition'] === 'non_entitlement') {
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'non_entitlement_allowed',
                'protected' => false,
                'entitlement_allowed' => false,
                'download_id' => $downloadId,
                'state_reason' => (string) $mapping['reason'],
                'facade_id' => $facadeId,
            ];
            $this->recordDecision('cart_gate', $decision, $idempotencyKey, $digest,
                $registrationUuid === '' ? null : $registrationUuid, null, $downloadId, $requestId);
            return $decision;
        }

        // Protected offer: facade allowlist, then verified registration context, then the
        // single-use gate token. All checks run before the token is consumed.
        $offer = $mapping['offer'];
        $this->assertFacadeSupports($facadeId, $origin, (string) $offer['public_code']);
        $this->assertVerifiedRegistration($registrationUuid, $facadeId, (string) $offer['public_code'], false);
        if ($productCode !== '' && !hash_equals((string) $offer['public_code'], $productCode)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }
        if ($rawToken === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $this->tokens->validate([
            'registration_token' => $rawToken,
            'registration_uuid' => $registrationUuid,
            'facade_id' => $facadeId,
            'product_code' => (string) $offer['public_code'],
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'consume' => true,
        ]);

        $decision = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'cart_gate_passed',
            'protected' => true,
            'entitlement_allowed' => true,
            'download_id' => $downloadId,
            'product_code' => (string) $offer['public_code'],
            'checkout_enabled' => (bool) $offer['checkout_enabled'],
            'facade_id' => $facadeId,
        ];
        $this->recordDecision('cart_gate', $decision, $idempotencyKey, $digest, $registrationUuid, null, $downloadId, $requestId);
        return $decision;
    }

    /**
     * EDD checkout gate (checkout surface). Requires an operator-approved active mapping
     * with the exact server-owned price, a verified registration binding, and either a
     * fresh single-use gate token or a journaled cart-gate binding for the same
     * registration, download, and facade. Client-controlled prices and grants are denied.
     *
     * Required input:
     *   - download_id (int), price_id, facade_id, origin, product_code (client claim)
     *   - registration_uuid, optional verified_token
     *   - request_id, idempotency_key
     */
    public function gateCheckout(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $downloadId = $this->assertPositiveInt($input['download_id'] ?? null, 'download_id');
        $priceId = (string) ($input['price_id'] ?? '');
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $productCode = (string) ($input['product_code'] ?? '');
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $rawToken = (string) ($input['verified_token'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);

        $digest = $this->requestDigest([
            'operation' => 'checkout_gate',
            'download_id' => $downloadId,
            'price_id' => $priceId,
            'facade_id' => $facadeId,
            'origin' => $origin,
            'product_code' => $productCode,
            'registration_uuid' => $registrationUuid,
            'verified_token_hash' => $rawToken === '' ? '' : FocusaSpec152eVerifiedRegistrationTokenValidator::tokenHash($rawToken),
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision('checkout_gate', $idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $mapping = $this->resolveDownloadMapping($downloadId);
        if ($mapping['disposition'] === 'credit_pack' || $mapping['disposition'] === 'unknown') {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if ($mapping['disposition'] === 'non_entitlement') {
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'non_entitlement_allowed',
                'protected' => false,
                'entitlement_allowed' => false,
                'download_id' => $downloadId,
                'state_reason' => (string) $mapping['reason'],
                'facade_id' => $facadeId,
            ];
            $this->recordDecision('checkout_gate', $decision, $idempotencyKey, $digest,
                $registrationUuid === '' ? null : $registrationUuid, null, $downloadId, $requestId);
            return $decision;
        }

        $offer = $mapping['offer'];
        $this->assertFacadeSupports($facadeId, $origin, (string) $offer['public_code']);
        $this->assertVerifiedRegistration($registrationUuid, $facadeId, (string) $offer['public_code'], false);
        if ($productCode !== '' && !hash_equals((string) $offer['public_code'], $productCode)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }
        if (!$offer['checkout_enabled'] || $offer['mapping_status'] !== 'active') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        if ($priceId === '' || !hash_equals((string) $offer['edd_price_id'], $priceId)) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }

        // Verified registration context at checkout: a fresh single-use token or the
        // journaled cart-gate binding from this registration's protected add-to-cart.
        if ($rawToken !== '') {
            $this->tokens->validate([
                'registration_token' => $rawToken,
                'registration_uuid' => $registrationUuid,
                'facade_id' => $facadeId,
                'product_code' => (string) $offer['public_code'],
                'request_id' => $requestId,
                'idempotency_key' => $idempotencyKey,
                'consume' => true,
            ]);
        } elseif (!$this->hasCartGateBinding($registrationUuid, $downloadId, $facadeId)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }

        $decision = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'checkout_gate_passed',
            'protected' => true,
            'entitlement_allowed' => true,
            'download_id' => $downloadId,
            'price_id' => $priceId,
            'product_code' => (string) $offer['public_code'],
            'facade_id' => $facadeId,
        ];
        $this->recordDecision('checkout_gate', $decision, $idempotencyKey, $digest, $registrationUuid, null, $downloadId, $requestId);
        return $decision;
    }

    /**
     * EDD order-completion gate (replaces the broad edd_complete_purchase hook). Requires
     * complete order status; verified registration/account binding with the order email
     * matching the verified registration; allowlisted product mapping with the exact
     * order-item price relationship; idempotent issuance state; and no existing equivalent
     * active license. The decision is journaled and replayable; entitlement issuance itself
     * is deferred to the verified issuance service. Unrelated and credit-pack order items
     * are proven non-entitlement and never issue Focusa/UIAI entitlement; an order with no
     * protected items completes with no entitlement and requires no registration.
     *
     * Required input:
     *   - order_id (int), order_status, customer_id (int), order_email
     *   - order_items: list of ['download_id' => int, 'price_id' => string, 'quantity' => int]
     *   - registration_uuid, facade_id, origin (registration_uuid required only when the
     *     order contains a protected item)
     *   - request_id, idempotency_key
     */
    public function handleOrderComplete(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $orderId = $this->assertPositiveInt($input['order_id'] ?? null, 'order_id');
        $orderStatus = (string) ($input['order_status'] ?? '');
        $customerId = $this->assertPositiveInt($input['customer_id'] ?? null, 'customer_id');
        $orderEmail = (string) ($input['order_email'] ?? '');
        $items = $input['order_items'] ?? [];
        $facadeId = (string) ($input['facade_id'] ?? '');
        $origin = (string) ($input['origin'] ?? '');
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
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

        $digest = $this->requestDigest([
            'operation' => 'order_complete_gate',
            'order_id' => $orderId,
            'order_status' => $orderStatus,
            'customer_id' => $customerId,
            'order_email_lookup_digest' => $orderEmail === '' ? '' : $this->emailLookupDigest($orderEmail),
            'order_items' => $items,
            'facade_id' => $facadeId,
            'origin' => $origin,
            'registration_uuid' => $registrationUuid,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision('order_complete_gate', $idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        if (in_array($orderStatus, ['refunded', 'revoked'], true)) {
            throw new DomainException($orderStatus === 'refunded' ? 'REFUNDED' : 'REVOKED');
        }
        if ($orderStatus !== 'complete') {
            if (in_array($orderStatus, ['pending', 'processing'], true)) {
                throw new DomainException('EDD_ORDER_PENDING');
            }
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        $registration = null;
        $protected = [];
        $excluded = [];
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

            // Protected item: verified registration/account binding is mandatory.
            if ($registration === null) {
                $registration = $this->assertVerifiedRegistration($registrationUuid, $facadeId, '', true);
                $this->assertFacadeSupports($facadeId, $origin, '');
                if ($orderEmail === '') {
                    throw new DomainException('EDD_ORDER_UNVERIFIED');
                }
                $orderDigest = $this->emailLookupDigest($orderEmail);
                if (!hash_equals((string) $registration['email_lookup_digest'], $orderDigest)) {
                    throw new DomainException('EDD_ORDER_UNVERIFIED');
                }
            }

            $offer = $mapping['offer'];
            $this->assertFacadeSupports($facadeId, $origin, (string) $offer['public_code']);
            if (!hash_equals((string) $registration['product_code'], (string) $offer['public_code'])) {
                throw new DomainException('FACADE_PRODUCT_DENIED');
            }
            if (!$offer['checkout_enabled'] || $offer['mapping_status'] !== 'active') {
                throw new DomainException('PRODUCT_MAPPING_REQUIRED');
            }
            $itemPrice = (string) ($item['price_id'] ?? '');
            if ($itemPrice === '' || !hash_equals((string) $offer['edd_price_id'], $itemPrice)) {
                throw new DomainException('PRODUCT_MAPPING_REQUIRED');
            }
            if ($this->hasEquivalentActiveLicense($customerId, $itemDownload)) {
                throw new DomainException('EDD_LICENSE_UNUSABLE');
            }
            $protected[] = [
                'download_id' => $itemDownload,
                'product_code' => (string) $offer['public_code'],
                'price_id' => $itemPrice,
                'license_type_ref' => (string) $offer['license_type_ref'],
            ];
        }

        if ($protected === []) {
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'no_entitlement',
                'order_id' => $orderId,
                'protected_items' => 0,
                'excluded_items' => $excluded,
                'issuance' => 'none',
                'facade_id' => $facadeId,
            ];
        } else {
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'entitlement_ready',
                'order_id' => $orderId,
                'customer_id' => $customerId,
                'protected_items' => $protected,
                'excluded_items' => $excluded,
                'issuance' => 'deferred_to_verified_issuance_service',
                'facade_id' => $facadeId,
            ];
        }
        $this->recordDecision('order_complete_gate', $decision, $idempotencyKey, $digest,
            $registrationUuid === '' ? null : $registrationUuid, $orderId, null, $requestId);
        return $decision;
    }

    /** Bounded journal lookups for settlement/reconciliation. */
    public function decisionCount(string $operation): int
    {
        $table = $this->schema->table('wpuiai_edd_gate_decisions');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE operation = :operation");
        $statement->execute([':operation' => $operation]);
        return (int) $statement->fetchColumn();
    }

    /** Bounded: has this registration a journaled cart-gate binding for the download/facade? */
    public function hasCartGateBinding(string $registrationUuid, int $downloadId, string $facadeId): bool
    {
        $this->assertUuid($registrationUuid, 'registration');
        $table = $this->schema->table('wpuiai_edd_gate_decisions');
        $statement = $this->db->prepare("SELECT 1 FROM {$table}
            WHERE operation = 'cart_gate' AND registration_uuid = :registration
              AND download_id = :download AND facade_id = :facade AND decision = 'cart_gate_passed' LIMIT 1");
        $statement->execute([
            ':registration' => $registrationUuid,
            ':download' => $downloadId,
            ':facade' => $facadeId,
        ]);
        return $statement->fetchColumn() !== false;
    }

    // ── private helpers ────────────────────────────────────────────────

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
     * Verified registration binding: the registration must be mailbox-verified, non-terminal,
     * unexpired, and bound to the exact facade and product. With $requireAccount, the
     * registration must already carry its EDD customer (account_promoted or later), because
     * entitlement settles only against a verified account-bound registration. Missing,
     * malformed, or unknown registrations fail closed with EMAIL_VERIFICATION_REQUIRED.
     */
    private function assertVerifiedRegistration(string $registrationUuid, string $facadeId, string $productCode, bool $requireAccount): array
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
        if ($productCode !== '' && !hash_equals((string) $registration['product_code'], $productCode)) {
            throw new DomainException('FACADE_PRODUCT_DENIED');
        }
        if ($requireAccount && $registration['edd_customer_id'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
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
        $forbidden = [
            'price', 'tier', 'products', 'license_type', 'license_type_ref', 'capability_family',
            'families', 'features', 'limits', 'node_limit', 'sale_status', 'refund_policy',
            'upgrade_policy', 'commercial_rights', 'evaluation_duration', 'grants',
            'edd_download_id', 'edd_price_id',
        ];
        foreach ($forbidden as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }

    private function recordDecision(
        string $operation,
        array $decision,
        string $idempotencyKey,
        string $digest,
        ?string $registrationUuid,
        ?int $orderId,
        ?int $downloadId,
        string $requestId,
    ): void {
        $table = $this->schema->table('wpuiai_edd_gate_decisions');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (decision_key, operation, registration_uuid, facade_id, product_code, download_id, order_id,
             decision, error_code, state_reason, result_payload, request_id, idempotency_key,
             request_digest, created_at, retention_until)
            VALUES (:key, :operation, :registration, :facade, :product, :download, :order,
                    :decision, NULL, :reason, :payload, :request, :idempotency,
                    :digest, :created, :retention)");
        $statement->execute([
            ':key' => hash('sha256', $operation . "\n" . $idempotencyKey . "\n" . $digest),
            ':operation' => $operation,
            ':registration' => $registrationUuid,
            ':facade' => (string) ($decision['facade_id'] ?? ''),
            ':product' => (string) ($decision['product_code'] ?? ''),
            ':download' => $downloadId,
            ':order' => $orderId,
            ':decision' => (string) $decision['decision'],
            ':reason' => (string) ($decision['state_reason'] ?? ''),
            ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':request' => $requestId,
            ':idempotency' => $idempotencyKey,
            ':digest' => $digest,
            ':created' => $this->now(),
            ':retention' => self::plusSeconds($this->now(), $this->retention),
        ]);
    }

    private function replayDecision(string $operation, string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_edd_gate_decisions');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE idempotency_key = :key AND operation = :operation");
        $statement->execute([':key' => $idempotencyKey, ':operation' => $operation]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    private function emailLookupDigest(string $email): string
    {
        return $this->registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($email));
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eEddGateDecisionMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eEddGateDecisionMigration::encodeCanonical($value));
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
