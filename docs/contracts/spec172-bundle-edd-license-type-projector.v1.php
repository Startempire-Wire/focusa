<?php
// Spec 172 Bundle composition (addendum sections 4.1, 7.3, 9.1-9.4, 10.1, 16.3, 17, and
// 21; atom focusa-vbcqu.20.15.15). The Bundle is ONE commerce SKU and ONE canonical EDD
// Software Licensing human key that resolves to the exact union of the two underlying
// Operator v1 License Types:
//
//   - `FocusaSpec172LicenseTypeRegistry` is the frozen License Type registry (PHP mirror
//     of docs/contracts/spec172-license-types.v1.yaml, composite_skus entry). It never
//     maintains a third hand-copied family list: the Bundle family set is DERIVED as the
//     exact union of the two underlying frozen records
//     (FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES plus
//     UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES), the Bundle grant set is the
//     exact two underlying License Type codes, and future products / future License
//     Types never enter.
//   - `FocusaSpec172BundleOrderSlAdapter` is the WPUIAI Bundle order/SL adapter: it
//     binds one verified complete $1,254.60 Bundle order/item (exactly one item; the
//     item must resolve to the server-owned composite Bundle offer) through the
//     canonical order-binding journal and issues exactly ONE canonical EDD Software
//     Licensing human key for the whole Bundle. Two standalone items in one order are
//     NEVER folded into a Bundle (BUNDLE_ITEM_COUNT_REQUIRED); standalone Operator items
//     never bind as a Bundle (LICENSE_TYPE_NOT_INCLUDED).
//   - `FocusaSpec172BundleOperatorProjector` is the composite Bundle projector: it
//     consumes exactly one issued canonical EDD Software Licensing key for one verified
//     complete eligible Bundle order and projects
//     `focusa_uiai_operator_bundle_lifetime_v1` from canonical EDD truth. The composite
//     decision carries both underlying Operator v1 grants (grant_composition
//     exact_union), the derived twelve-family union digest, one operator seat, the SAME
//     three shared operator_shared_v1 node identities (never six unrelated activations),
//     the server-owned 1254.60 USD price version, one human key, whole-order refunds,
//     and zero future products. A replay returns the identical decision; a duplicate
//     projection call for the same issued request returns the same projection with
//     existing=true and projections_created=0 (shared UNIQUE issuance_request_key
//     journal).
//   - Wrong product (a standalone Focusa or UIAI order can never project the Bundle),
//     wrong price, wrong account, refunded/revoked/pending orders, unissued requests,
//     revoked/tampered canonical licenses, checkout-disabled or drifted offers, bundle
//     offers whose grants are not the exact union, and caller-controlled commerce fields
//     create zero projections and never advance the authority sequence.
//
// Failures are public-safe stable codes (EDD_ORDER_PENDING, REFUNDED, REVOKED,
// EDD_ORDER_UNVERIFIED, ACCOUNT_EMAIL_MISMATCH, EMAIL_VERIFICATION_REQUIRED,
// ACCOUNT_MERGE_REVIEW_REQUIRED, FACADE_ORIGIN_DENIED, PRODUCT_MAPPING_REQUIRED,
// PRODUCT_NOT_INCLUDED, LICENSE_TYPE_NOT_INCLUDED, EDD_CHECKOUT_REQUIRED,
// EDD_LICENSE_UNUSABLE, CLIENT_COMMERCIAL_FIELDS_FORBIDDEN, IDEMPOTENCY_CONFLICT,
// ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED). The Bundle adapter adds three Bundle-scoped
// codes (BUNDLE_ITEM_COUNT_REQUIRED, BUNDLE_ORDER_INELIGIBLE,
// BUNDLE_KEY_ISSUANCE_FAILED).
//
// No plaintext leakage: journals and decisions carry only the 64-hex license-key digest
// plus a masked key; no raw email, raw payment transaction id, secret, key, credential,
// customer row, or card data is stored or returned.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php,
// docs/contracts/spec152e-edd-order-binding.v1.php,
// docs/contracts/spec152e-edd-license-issuance.v1.php,
// docs/contracts/spec172-edd-license-type-projector.v1.php (shared projection journal
// schema migration and the Focusa frozen family record),
// docs/contracts/spec172-uiai-edd-license-type-projector.v1.php (the UIAI frozen family
// record), and the server-owned dedicated downloads contract
// (docs/contracts/spec172-edd-operator-v1-downloads.v1.php) to be loaded first.
declare(strict_types=1);

/**
 * Frozen License Type registry (PHP mirror of the spec172-license-types.v1.yaml
 * composite_skus entry). The Bundle is one SKU that grants exactly the two underlying
 * Operator v1 License Types; the family set is DERIVED from the two underlying frozen
 * records, never a third hand-copied list.
 */
final class FocusaSpec172LicenseTypeRegistry
{
    public const SCHEMA = 'focusa.spec172.license_types.v1';
    public const VERSION = 1;
    public const AUTHORITY = 'docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md';

    public const FOCUSA_LICENSE_TYPE = 'focusa_operator_lifetime_v1';
    public const UIAI_LICENSE_TYPE = 'uiai_operator_lifetime_v1';
    public const BUNDLE_SKU = 'focusa_uiai_operator_bundle_lifetime_v1';

    public const FOCUSA_PRICE_USD = '697.00';
    public const UIAI_PRICE_USD = '697.00';
    public const BUNDLE_PRICE_USD = '1254.60';
    public const BUNDLE_AMOUNT_MINOR = 125460;
    public const STANDALONE_SUM_USD = '1394.00';
    public const DISCOUNT_BASIS_POINTS = 1000;

    public const OPERATOR_SEATS = 1;
    public const NODE_LIMIT = 3;
    public const NODE_SET = 'operator_shared_v1';
    public const TERM = 'lifetime';
    public const HUMAN_KEY_COUNT = 1;
    public const GRANT_COMPOSITION = 'exact_union';
    public const COMPONENT_REFUNDS_ALLOWED = false;
    public const FUTURE_PRODUCTS_INCLUDED = false;
    public const FUTURE_LICENSE_TYPES_INCLUDED = false;

    /** The exact two underlying Operator v1 License Type codes; no third grant. */
    public static function underlyingLicenseTypes(): array
    {
        return [self::FOCUSA_LICENSE_TYPE, self::UIAI_LICENSE_TYPE];
    }

    /** The two underlying product scopes of the Bundle SKU. */
    public static function underlyingProducts(): array
    {
        return ['focusa', 'uiai_engine'];
    }

    /** Focusa families come from the frozen Focusa Operator record — never copied here. */
    public static function focusaFamilies(): array
    {
        return FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES;
    }

    /** UIAI families come from the frozen UIAI Operator record — never copied here. */
    public static function uiaiFamilies(): array
    {
        return UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES;
    }

    /** Per-product family sets, derived exclusively from the two underlying records. */
    public static function familySets(): array
    {
        return [
            'focusa' => self::focusaFamilies(),
            'uiai_engine' => self::uiaiFamilies(),
        ];
    }

    /** Flat union family set (5 Focusa + 7 UIAI = 12), derived, never hand-copied. */
    public static function underlyingFamilies(): array
    {
        return array_merge(self::focusaFamilies(), self::uiaiFamilies());
    }

    /** Frozen digest over the derived union and the exact-grant composition. */
    public static function familyDigest(): string
    {
        return hash('sha256', FocusaSpec172LicenseTypeProjectionMigration::encodeCanonical([
            'sku' => self::BUNDLE_SKU,
            'grant_composition' => self::GRANT_COMPOSITION,
            'grants' => self::underlyingLicenseTypes(),
            'family_sets' => self::familySets(),
            'authority' => self::AUTHORITY,
        ]));
    }

    /** Server-owned Bundle price version: SKU, fixed price, and contract version. */
    public static function priceVersion(): string
    {
        return sprintf('%s.%s.v%s', self::BUNDLE_SKU, self::BUNDLE_PRICE_USD, self::VERSION);
    }

    /**
     * Fail-closed bundle composition validation against the server-owned offer record:
     * the offer must be the exact union of the two Operator v1 License Types with the
     * canonical price, one seat, three shared nodes, whole-order refunds, and zero
     * future products. Any drift is a mapping error, never a new grant.
     */
    public static function assertOfferComposition(array $offer): void
    {
        if ((string) ($offer['grant_composition'] ?? '') !== self::GRANT_COMPOSITION
            || (array) ($offer['grants'] ?? []) !== self::underlyingLicenseTypes()) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (($offer['component_refunds_allowed'] ?? false) === true) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (($offer['future_products_included'] ?? false) === true
            || ($offer['future_license_types_included'] ?? false) === true) {
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        if ((int) ($offer['amount_minor'] ?? 0) !== self::BUNDLE_AMOUNT_MINOR) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
    }
}

/**
 * WPUIAI Bundle order/SL adapter. Binds one verified complete $1,254.60 Bundle
 * order/item (exactly one order item resolving to the server-owned composite Bundle
 * offer) through the canonical order-binding journal and issues exactly ONE canonical
 * EDD Software Licensing human key for the whole Bundle. The adapter never folds two
 * standalone items into a Bundle, never accepts a standalone Operator item as a Bundle,
 * and never lets caller metadata select product, price, grants, limits, or rights.
 */
final class FocusaSpec172BundleOrderSlAdapter
{
    public const SCHEMA = 'focusa.spec172.bundle_order_sl_adapter.v1';
    public const SKU = 'focusa_uiai_operator_bundle_lifetime_v1';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'tier', 'products', 'product_code', 'license_type',
        'license_type_ref', 'capability_family', 'families', 'features', 'grants', 'limits',
        'node_limit', 'activation_limit', 'operator_seats', 'node_set', 'sale_status',
        'refund_policy', 'upgrade_policy', 'commercial_rights', 'evaluation_duration',
        'edd_download_id', 'edd_price_id', 'license_key', 'license_duration', 'expiration',
        'grant_composition', 'component_refunds_allowed', 'future_products_included',
        'future_license_types_included', 'human_key_count',
    ];

    public function __construct(
        private FocusaSpec152eEddOrderBindingService $binding,
        private FocusaSpec152eEddLicenseIssuanceService $issuance,
        private array $dedicatedDownloads,
    ) {
    }

    /**
     * Bind one verified $1,254.60 Bundle order/item and issue exactly one canonical EDD
     * Software Licensing human key. Required input mirrors the canonical order-binding
     * contract (order_id, order_status, customer_id, order_items with exactly one item,
     * payment_transactions, registration_uuid, facade_id, origin, request_id,
     * idempotency_key). Caller metadata never selects any product, price, License Type,
     * grant, limit, node, or commercial right.
     */
    public function bindAndIssue(array $input): array
    {
        $this->assertNoCallerControlledGrantFields($input);
        $items = $input['order_items'] ?? [];
        if (!is_array($items) || count($items) !== 1) {
            throw new DomainException('BUNDLE_ITEM_COUNT_REQUIRED');
        }
        $downloadId = (int) ($items[0]['download_id'] ?? 0);
        $offer = null;
        foreach (($this->dedicatedDownloads['records'] ?? []) as $candidate) {
            if ((int) ($candidate['edd_download_id'] ?? 0) === $downloadId) {
                $offer = $candidate;
                break;
            }
        }
        if ($offer === null) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        // The Bundle record carries the composite License Type reference as
        // composite_sku_ref (the two underlying grants live in `grants`); the
        // standalone records carry license_type_ref. Only the one composite SKU binds.
        $offerLicenseTypeRef = (string) ($offer['license_type_ref'] ?? '');
        if ($offerLicenseTypeRef === '') {
            $offerLicenseTypeRef = (string) ($offer['composite_sku_ref'] ?? '');
        }
        if ((string) ($offer['public_code'] ?? '') !== self::SKU || $offerLicenseTypeRef !== self::SKU) {
            throw new DomainException('LICENSE_TYPE_NOT_INCLUDED');
        }
        FocusaSpec172LicenseTypeRegistry::assertOfferComposition($offer);
        if (($offer['checkout_enabled'] ?? false) !== true || ($offer['sale_status'] ?? '') !== 'enabled') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }

        $bound = $this->binding->bindOrderComplete($input);
        if (($bound['decision'] ?? '') !== 'order_bound'
            || (int) ($bound['issuance_requests_settled'] ?? 0) !== 1
            || (int) ($bound['protected_items'] ?? 0) !== 1) {
            throw new DomainException('BUNDLE_ORDER_INELIGIBLE');
        }
        $handle = (string) ($bound['protected_items'][0]['issuance_request_handle'] ?? '');
        if (preg_match('/^(ir_)[0-9a-f]{32}$/D', $handle) !== 1) {
            throw new DomainException('BUNDLE_ORDER_INELIGIBLE');
        }

        $issued = $this->issuance->issue([
            'issuance_request_handle' => $handle,
            'request_id' => (string) ($input['request_id'] ?? '') . '.issue',
            'idempotency_key' => (string) ($input['idempotency_key'] ?? '') . '.issue',
        ]);
        if (($issued['decision'] ?? '') !== 'license_issued'
            || (int) ($issued['keys_created'] ?? 0) !== 1) {
            throw new DomainException('BUNDLE_KEY_ISSUANCE_FAILED');
        }

        return [
            'schema' => self::SCHEMA,
            'decision' => 'bundle_bound_and_issued',
            'sku' => self::SKU,
            'issuance_request_handle' => $handle,
            'human_key_count' => FocusaSpec172LicenseTypeRegistry::HUMAN_KEY_COUNT,
            'grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
            'grant_composition' => FocusaSpec172LicenseTypeRegistry::GRANT_COMPOSITION,
            'products' => FocusaSpec172LicenseTypeRegistry::underlyingProducts(),
            'price_usd' => FocusaSpec172LicenseTypeRegistry::BUNDLE_PRICE_USD,
            'amount_minor' => FocusaSpec172LicenseTypeRegistry::BUNDLE_AMOUNT_MINOR,
            'license_key_digest' => (string) ($issued['license_key_digest'] ?? ''),
            'license_key_mask' => (string) ($issued['license_key_mask'] ?? ''),
            'delivery' => $issued['delivery'] ?? null,
            'edd_license_id' => (int) ($issued['edd_license_id'] ?? 0),
            'order_id' => (int) ($input['order_id'] ?? 0),
            'customer_id' => (int) ($input['customer_id'] ?? 0),
        ];
    }

    private function assertNoCallerControlledGrantFields(array $input): void
    {
        foreach (self::CLIENT_CONTROLLED_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }
}

/**
 * Composite Bundle projector. Consumes exactly one issued canonical EDD Software
 * Licensing key for one verified complete eligible Bundle order and projects
 * `focusa_uiai_operator_bundle_lifetime_v1` as the exact union of the two underlying
 * Operator v1 License Types with three shared node identities, one seat, the
 * server-owned 1254.60 USD price version, the derived twelve-family union digest, and
 * zero future products. One eligible item produces exactly one projection forever.
 */
final class FocusaSpec172BundleOperatorProjector
{
    public const SCHEMA = 'focusa.spec172.license_type_projection.v1';
    public const RESULT_SCHEMA = 'focusa.spec172.bundle_operator_lifetime_projection.v1';
    public const VERSION = 1;
    public const SKU = 'focusa_uiai_operator_bundle_lifetime_v1';
    public const LICENSE_TYPE = self::SKU;
    public const TERM = 'lifetime';
    public const RETENTION_SECONDS = 2592000;
    public const KEY_PATTERN = '/^[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}$/D';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'currency', 'tier', 'products', 'product_code',
        'sku', 'license_type', 'license_type_ref', 'capability_family', 'families',
        'family_sets', 'features', 'grants', 'grants_union', 'limits', 'node_limit',
        'activation_limit', 'operator_seats', 'node_set', 'sale_status', 'refund_policy',
        'upgrade_policy', 'commercial_rights', 'evaluation_duration', 'edd_download_id',
        'edd_price_id', 'license_key', 'license_duration', 'expiration', 'price_version',
        'family_digest', 'grant_composition', 'component_refunds_allowed',
        'future_products_included', 'future_license_types_included', 'human_key_count',
    ];

    private const REQUEST_STATE_ISSUED = 'issued';
    private const BINDING_STATE_SETTLED = 'settled_pending_issuance';
    private const BINDING_STATE_BLOCKED = 'blocked';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172LicenseTypeProjectionMigration $schema,
        private FocusaSpec152eEddLicenseIssuanceMigration $issuanceSchema,
        private FocusaSpec152eEddOrderBindingMigration $bindingSchema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private array $dedicatedDownloads,
        callable $clock,
        private string $prefix = 'wp_',
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        if ($this->retention < 1) {
            throw new InvalidArgumentException('positive retention required');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Project `focusa_uiai_operator_bundle_lifetime_v1` for exactly one issued canonical
     * EDD license. Required input:
     *   - issuance_request_handle: the opaque ir_ handle journaled by the order-binding
     *     service and issued by the canonical SL issuance service
     *   - request_id, idempotency_key
     * Caller metadata never selects any product, price, License Type, grant, family,
     * feature, limit, node, seat, or commercial right. Returns a public-safe composite
     * projection decision; replays return the identical decision; duplicate projection
     * calls for the same issued request return the same projection with existing=true
     * and zero creations.
     */
    public function project(array $input): array
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
            'operation' => 'bundle_operator_lifetime_projection',
            'issuance_request_handle' => $issuanceRequestKey,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $request = $this->loadIssuanceRequest($issuanceRequestKey);
        if ($request['state'] !== self::REQUEST_STATE_ISSUED) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $issuanceRow = $this->loadIssuanceJournal($issuanceRequestKey);
        $binding = $this->loadBinding((string) $request['binding_key'], $issuanceRequestKey);
        if ($binding['binding_state'] === self::BINDING_STATE_BLOCKED) {
            throw new DomainException((string) ($binding['blocked_reason'] ?? 'EDD_LICENSE_UNUSABLE'));
        }
        if ($binding['binding_state'] !== self::BINDING_STATE_SETTLED) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }

        // Already projected: a duplicate projection call returns the same projection with
        // zero creations. Never a second projection for the same issuance request.
        $existing = $this->findByIssuanceRequestKey($issuanceRequestKey);
        if ($existing !== null) {
            return $this->existingProjectionDecision($existing);
        }

        $registration = $this->assertProjectionRegistration((string) $request['registration_uuid'], $binding, $request);
        $downloadId = (int) $binding['download_id'];
        $this->assertCanonicalOrder((int) $request['order_id'], (int) $request['customer_id'], $registration);
        $this->assertCanonicalOrderItem((int) $request['order_id'], (int) $request['order_item_id'], $downloadId);
        $licenseId = (int) $issuanceRow['edd_license_id'];
        $this->assertCanonicalLicense($licenseId, (string) $issuanceRow['license_key_digest']);
        $offer = $this->assertOfferMapping($request, $binding, $downloadId);

        return $this->recordProjection(
            $issuanceRow, $request, $binding, $registration, $offer, $licenseId,
            $requestId, $idempotencyKey, $digest,
        );
    }

    /** Bounded journal lookups for settlement/reconciliation. */
    public function projectionCount(): int
    {
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}");
        $statement->execute();
        return (int) $statement->fetchColumn();
    }

    /** Bounded: exact projection lookup by opaque projection handle (pr_). */
    public function findByProjectionKey(string $projectionKey): ?array
    {
        $this->assertToken($projectionKey, 64, 'projection');
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE projection_key = :key LIMIT 1");
        $statement->execute([':key' => $projectionKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /** Bounded: exact projection lookup by the source issuance-request handle (ir_). */
    public function findByIssuanceRequestKey(string $issuanceRequestKey): ?array
    {
        $this->assertToken($issuanceRequestKey, 64, 'issuance request');
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function recordProjection(
        array $issuanceRow,
        array $request,
        array $binding,
        array $registration,
        array $offer,
        int $licenseId,
        string $requestId,
        string $idempotencyKey,
        string $digest,
    ): array {
        $now = $this->now();
        $account = $this->accounts->findByUuid((string) $request['account_uuid']);
        $nextSequence = (int) $account['highest_entitlement_sequence'] + 1;
        $priceVersion = self::priceVersion($offer);
        $familyDigest = FocusaSpec172LicenseTypeRegistry::familyDigest();
        $projectionKey = self::opaqueToken('pr_');
        $projections = $this->schema->table('wpuiai_license_type_projections');
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';

        $decision = [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'license_type_projected',
            'projection_id' => $projectionKey,
            'registration_id' => (string) $request['registration_uuid'],
            'account_id' => (string) $request['account_uuid'],
            'customer_id' => (int) $request['customer_id'],
            'order_id' => (int) $request['order_id'],
            'order_item_id' => (int) $request['order_item_id'],
            'download_id' => (int) $binding['download_id'],
            'edd_license_id' => $licenseId,
            'license_key_digest' => (string) $issuanceRow['license_key_digest'],
            'license_key_mask' => (string) $issuanceRow['license_key_mask'],
            'issuance' => 'canonical_edd_software_licensing',
            'sku' => self::SKU,
            'product' => self::SKU,
            'license_type' => self::LICENSE_TYPE,
            'grant' => self::LICENSE_TYPE,
            'products' => FocusaSpec172LicenseTypeRegistry::underlyingProducts(),
            'grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
            'grants_union' => FocusaSpec172LicenseTypeRegistry::GRANT_COMPOSITION,
            'human_key_count' => FocusaSpec172LicenseTypeRegistry::HUMAN_KEY_COUNT,
            'price_version' => $priceVersion,
            'price_usd' => (string) $offer['price_usd'],
            'amount_minor' => (int) $offer['amount_minor'],
            'family_digest' => $familyDigest,
            'family_count' => count(FocusaSpec172LicenseTypeRegistry::underlyingFamilies()),
            'families' => FocusaSpec172LicenseTypeRegistry::underlyingFamilies(),
            'family_sets' => FocusaSpec172LicenseTypeRegistry::familySets(),
            'operator_seats' => FocusaSpec172LicenseTypeRegistry::OPERATOR_SEATS,
            'node_limit' => FocusaSpec172LicenseTypeRegistry::NODE_LIMIT,
            'node_set' => FocusaSpec172LicenseTypeRegistry::NODE_SET,
            'term' => self::TERM,
            'status' => 'active',
            'sequence' => $nextSequence,
            'component_refunds_allowed' => FocusaSpec172LicenseTypeRegistry::COMPONENT_REFUNDS_ALLOWED,
            'future_products_included' => FocusaSpec172LicenseTypeRegistry::FUTURE_PRODUCTS_INCLUDED,
            'future_license_types_included' => FocusaSpec172LicenseTypeRegistry::FUTURE_LICENSE_TYPES_INCLUDED,
            'existing' => false,
            'projections_created' => 1,
        ];

        $this->db->beginTransaction();
        try {
            $statement = $this->db->prepare("UPDATE {$accountsTable}
                SET highest_entitlement_sequence = :next, updated_at = :updated
                WHERE account_uuid = :uuid AND highest_entitlement_sequence < :guard");
            $statement->execute([
                ':next' => $nextSequence,
                ':updated' => $now,
                ':uuid' => (string) $request['account_uuid'],
                ':guard' => $nextSequence,
            ]);
            if ($statement->rowCount() !== 1) {
                throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
            }
            $retention = self::plusSeconds($now, $this->retention);
            $projectionStatement = $this->db->prepare("INSERT INTO {$projections}
                (projection_key, issuance_request_key, issuance_key, binding_key, registration_uuid,
                 account_uuid, customer_id, order_id, order_item_id, download_id, edd_license_id,
                 product_code, license_type_ref, price_version, family_digest, operator_seats,
                 node_limit, node_set, term, status, sequence, result_payload,
                 request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
                VALUES (:projection, :request_key, :issuance, :binding, :registration,
                        :account, :customer, :order, :item, :download, :license_id,
                        :product, :license_type, :price_version, :family_digest, :seats,
                        :node_limit, :node_set, :term, :status, :sequence, :payload,
                        :request, :idempotency, :request_digest, :created, :retention, :updated)");
            $projectionStatement->execute([
                ':projection' => $projectionKey,
                ':request_key' => (string) $request['issuance_request_key'],
                ':issuance' => (string) $issuanceRow['issuance_key'],
                ':binding' => (string) $binding['binding_key'],
                ':registration' => (string) $request['registration_uuid'],
                ':account' => (string) ($request['account_uuid'] ?? ''),
                ':customer' => (int) $request['customer_id'],
                ':order' => (int) $request['order_id'],
                ':item' => (int) $request['order_item_id'],
                ':download' => (int) $binding['download_id'],
                ':license_id' => $licenseId,
                ':product' => self::SKU,
                ':license_type' => self::LICENSE_TYPE,
                ':price_version' => $priceVersion,
                ':family_digest' => $familyDigest,
                ':seats' => FocusaSpec172LicenseTypeRegistry::OPERATOR_SEATS,
                ':node_limit' => FocusaSpec172LicenseTypeRegistry::NODE_LIMIT,
                ':node_set' => FocusaSpec172LicenseTypeRegistry::NODE_SET,
                ':term' => self::TERM,
                ':status' => 'active',
                ':sequence' => $nextSequence,
                ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':request_digest' => $digest,
                ':created' => $now,
                ':retention' => $retention,
                ':updated' => $now,
            ]);
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
        return $decision;
    }

    /** Duplicate projection call for the same issued request: same projection, zero new. */
    private function existingProjectionDecision(array $row): array
    {
        $decision = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $decision['decision'] = 'license_type_projected';
        $decision['existing'] = true;
        $decision['projections_created'] = 0;
        return $decision;
    }

    /**
     * Idempotent replay: same key returns the identical decision. The journaled payload
     * is the full original decision (existing=false, projections_created=1), so a replay
     * is byte-identical; a duplicate call with a NEW idempotency key routes through
     * existingProjectionDecision instead.
     */
    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_license_type_projections');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    /** The issuance request must exist in the order-binding journal and be issued. */
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

    /** The canonical SL issuance journal must hold the issued license for this request. */
    private function loadIssuanceJournal(string $issuanceRequestKey): array
    {
        $table = $this->issuanceSchema->table('wpuiai_edd_license_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE issuance_request_key = :key LIMIT 1");
        $statement->execute([':key' => $issuanceRequestKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false || $row['state'] !== 'issued') {
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
     * facade of the settled request. Fulfillment states from the SL issuance are accepted;
     * registrations that never entered checkout or are terminal fail closed.
     */
    private function assertProjectionRegistration(string $registrationUuid, array $binding, array $request): array
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
        $fulfilled = ['entitlement_issued', 'terminal_delivery_ready', 'device_registered', 'lease_issued'];
        $nonTerminal = ['checkout_pending'];
        if (!in_array($state, array_merge($fulfilled, $nonTerminal), true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
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
        $order = $this->edd->findOrderById($orderId);
        if ($order === null) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $status = (string) ($order['status'] ?? '');
        if (in_array($status, ['refunded', 'revoked'], true)) {
            throw new DomainException(strtoupper($status));
        }
        if (in_array($status, ['pending', 'processing'], true)) {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        if ($status !== 'complete') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((int) $order['customer_id'] !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $orderEmail = (string) ($order['email'] ?? '');
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
        $item = $this->edd->findOrderItemById($orderItemId);
        if ($item === null
            || (int) $item['order_id'] !== $orderId
            || (int) $item['product_id'] !== $downloadId
            || (int) $item['quantity'] < 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
    }

    /** Canonical EDD license truth: the license row exists, is active, and carries the exact digest. */
    private function assertCanonicalLicense(int $licenseId, string $expectedDigest): void
    {
        $license = $this->edd->findLicenseById($licenseId);
        if ($license === null || (string) $license['status'] !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $key = (string) $license['license_key'];
        if (preg_match(self::KEY_PATTERN, $key) !== 1
            || !hash_equals($expectedDigest, $this->keyDigest($key))) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
    }

    /**
     * Server-owned offer resolution at projection time: the dedicated Downloads contract
     * resolves by the settled download binding and must still carry the exact public
     * code, License Type ref, price id, amount, exact-grant union composition, and an
     * enabled mapping. Only the composite Bundle SKU projects here; any standalone
     * Operator offer (focusa or uiai) fails closed with LICENSE_TYPE_NOT_INCLUDED.
     */
    private function assertOfferMapping(array $request, array $binding, int $downloadId): array
    {
        $offer = null;
        foreach (($this->dedicatedDownloads['records'] ?? []) as $candidate) {
            if ((int) ($candidate['edd_download_id'] ?? 0) === $downloadId) {
                $offer = $candidate;
                break;
            }
        }
        // The Bundle record carries the composite License Type reference as
        // composite_sku_ref (the two underlying grants live in `grants`); standalone
        // records carry license_type_ref. The canonical Bundle identifier is the one
        // composite public code.
        $offerLicenseTypeRef = (string) ($offer['license_type_ref'] ?? '');
        if ($offerLicenseTypeRef === '') {
            $offerLicenseTypeRef = (string) ($offer['composite_sku_ref'] ?? '');
        }
        if ($offer === null
            || !hash_equals((string) ($offer['public_code'] ?? ''), (string) $request['product_code'])
            || !hash_equals((string) ($offer['edd_price_id'] ?? ''), (string) ($binding['price_id'] ?? ''))) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        if (!hash_equals(self::LICENSE_TYPE, $offerLicenseTypeRef)
            || !hash_equals(self::SKU, (string) ($offer['public_code'] ?? ''))) {
            throw new DomainException('LICENSE_TYPE_NOT_INCLUDED');
        }
        if ((array) ($offer['products'] ?? []) !== FocusaSpec172LicenseTypeRegistry::underlyingProducts()) {
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        FocusaSpec172LicenseTypeRegistry::assertOfferComposition($offer);
        if (($offer['checkout_enabled'] ?? false) !== true || ($offer['sale_status'] ?? '') !== 'enabled') {
            throw new DomainException('EDD_CHECKOUT_REQUIRED');
        }
        if ((string) $offer['license_duration'] !== self::TERM) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        return $offer;
    }

    /** Server-owned Bundle price version: SKU, fixed price, and contract version. */
    public static function priceVersion(array $offer): string
    {
        return sprintf(
            '%s.%s.v%s',
            self::LICENSE_TYPE,
            (string) $offer['price_usd'],
            (int) (self::VERSION),
        );
    }

    private function keyDigest(string $licenseKey): string
    {
        return hash('sha256', "focusa.spec152e.edd_license_issuance.key.v1\0" . $licenseKey);
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
        FocusaSpec172LicenseTypeProjectionMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec172LicenseTypeProjectionMigration::encodeCanonical($value));
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
