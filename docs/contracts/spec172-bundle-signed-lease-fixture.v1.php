<?php
// Spec 172 Bundle signed lease claims (addendum sections 7.3, 9.1-9.4, 14, and 21;
// atom focusa-vbcqu.20.15.15). Derives the signed lease payload — the machine
// execution credential for a Bundle account — exclusively from the composite Bundle
// projection produced by the Bundle License Type projector:
//
//   - The fixture is server-shaped, never caller-shaped: the SKU, both underlying
//     Operator v1 grants, the derived union family set, seat/node limits, price
//     version, family digest, term, and sequence all come from the frozen registry and
//     the frozen projection output. A client can never select product, price, License
//     Type, grant, family, feature, limit, node, or commercial right.
//   - The Bundle lease claims carry BOTH underlying product grants explicitly
//     (`focusa_operator_lifetime_v1` and `uiai_operator_lifetime_v1`) as the exact
//     union; the features are the derived union of the two underlying frozen family
//     records (5 Focusa + 7 UIAI = 12), never a third hand-copied list.
//   - The Bundle uses the SAME three shared operator node identities
//     (operator_shared_v1, node_limit 3, one seat) for both products — never six
//     unrelated activations — and exactly one canonical human key for the whole Bundle.
//   - Future products and future License Types are excluded by default
//     (future_products_included=false, future_license_types_included=false) and
//     component-level refunds are not supported in v1 (component_refunds_allowed=false,
//     whole-order 30-day refund policy).
//   - Lifetime entitlement with bounded credential lifetime (Spec 172 section 14):
//     `term` is `lifetime` while the credential refresh window is bounded (90 days) and
//     offline grace is bounded (30 days past the refresh window). Credential expiry never
//     destroys the underlying entitlement; it only ends the bounded credential.
//   - No plaintext leakage: the fixture carries no raw email, license key, token,
//     credential, customer row, or card data.
//
// Requires docs/contracts/spec172-edd-license-type-projector.v1.php and
// docs/contracts/spec172-bundle-edd-license-type-projector.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec172BundleSignedLeaseFixture
{
    public const SCHEMA = 'focusa.spec172.bundle_signed_lease_fixture.v1';
    public const LEASE_PAYLOAD_SCHEMA = 'focusa.bundle_signed_lease.v1';
    public const SKU = 'focusa_uiai_operator_bundle_lifetime_v1';
    public const TERM = 'lifetime';
    public const STATUS = 'active';

    /** Bounded credential refresh window for the lifetime entitlement (Spec 172 section 14). */
    public const REFRESH_WINDOW_DAYS = 90;
    /** Bounded offline grace past the refresh window; never expands products or limits. */
    public const OFFLINE_GRACE_DAYS = 30;

    public const OPERATOR_SEATS = 1;
    public const NODE_LIMIT = 3;
    public const NODE_SET = 'operator_shared_v1';

    /**
     * Build the canonical Bundle signed lease payload from exactly one accepted
     * composite Bundle projection. The node id is the operator's registered node binding
     * (bounded token); everything else is derived from the projection and the frozen
     * License Type registry.
     */
    public static function fromProjection(array $projection, string $nodeId, callable $clock): array
    {
        if (($projection['schema'] ?? '') !== FocusaSpec172BundleOperatorProjector::RESULT_SCHEMA) {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        if (($projection['decision'] ?? '') !== 'license_type_projected'
            || ($projection['status'] ?? '') !== 'active') {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        self::assertNodeId($nodeId);

        $issuedAt = (string) $clock();
        FocusaSpec172LicenseTypeProjectionMigration::assertTimestamp($issuedAt);
        $expiresAt = self::plusDays($issuedAt, self::REFRESH_WINDOW_DAYS);
        $offlineGraceUntil = self::plusDays($expiresAt, self::OFFLINE_GRACE_DAYS);

        $grants = [];
        foreach (FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes() as $code) {
            $grants[$code] = true;
        }
        $features = [];
        foreach (FocusaSpec172LicenseTypeRegistry::underlyingFamilies() as $family) {
            $features[$family] = true;
        }

        return [
            'schema' => self::SCHEMA,
            'lease_payload' => [
                'schema' => self::LEASE_PAYLOAD_SCHEMA,
                'lease_id' => self::opaqueLeaseId(),
                'product' => self::SKU,
                'subject_id' => (string) $projection['account_id'],
                'node_id' => $nodeId,
                'sequence' => (int) $projection['sequence'],
                'issued_at' => $issuedAt,
                'not_before' => $issuedAt,
                'expires_at' => $expiresAt,
                'offline_grace_until' => $offlineGraceUntil,
                'authority_key_id' => (string) ($projection['authority_key_id'] ?? 'authority-lease-2026-01'),
                'status' => self::STATUS,
                'grants' => $grants,
                'features' => $features,
                'family_sets' => FocusaSpec172LicenseTypeRegistry::familySets(),
                'limits' => [
                    'operator_seats' => self::OPERATOR_SEATS,
                    'node_limit' => self::NODE_LIMIT,
                ],
                'node_set' => self::NODE_SET,
                'human_key_count' => (int) $projection['human_key_count'],
                'future_products_included' => false,
                'future_license_types_included' => false,
                'component_refunds_allowed' => false,
            ],
            'grant_metadata' => [
                'product' => self::SKU,
                'license_type' => (string) $projection['license_type'],
                'grants' => FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(),
                'price_version' => (string) $projection['price_version'],
                'family_digest' => (string) $projection['family_digest'],
                'family_count' => count(FocusaSpec172LicenseTypeRegistry::underlyingFamilies()),
                'operator_seats' => self::OPERATOR_SEATS,
                'node_limit' => self::NODE_LIMIT,
                'node_set' => self::NODE_SET,
                'term' => self::TERM,
                'refund_policy' => 'whole_order_30_days',
            ],
        ];
    }

    /** Fail-closed validation of the derived fixture against its accepted projection. */
    public static function validate(array $fixture, array $projection): void
    {
        if (($fixture['schema'] ?? '') !== self::SCHEMA) {
            throw new DomainException('FIXTURE_SCHEMA_MISMATCH');
        }
        $payload = $fixture['lease_payload'] ?? [];
        $meta = $fixture['grant_metadata'] ?? [];
        if (($payload['schema'] ?? '') !== self::LEASE_PAYLOAD_SCHEMA
            || ($payload['product'] ?? '') !== self::SKU
            || ($meta['license_type'] ?? '') !== FocusaSpec172LicenseTypeRegistry::BUNDLE_SKU
            || ($payload['status'] ?? '') !== self::STATUS) {
            throw new DomainException('FIXTURE_SCOPE_MISMATCH');
        }
        if ((int) $payload['sequence'] !== (int) $projection['sequence'] || (int) $payload['sequence'] < 1) {
            throw new DomainException('FIXTURE_SEQUENCE_MISMATCH');
        }
        if ((string) $payload['subject_id'] !== (string) $projection['account_id']) {
            throw new DomainException('FIXTURE_SUBJECT_MISMATCH');
        }
        if ((string) $meta['price_version'] !== (string) $projection['price_version']
            || (string) $meta['family_digest'] !== (string) $projection['family_digest']
            || (string) $meta['family_digest'] !== FocusaSpec172LicenseTypeRegistry::familyDigest()) {
            throw new DomainException('FIXTURE_GRANT_MISMATCH');
        }
        if ((int) $meta['operator_seats'] !== self::OPERATOR_SEATS
            || (int) $meta['node_limit'] !== self::NODE_LIMIT
            || (string) $meta['node_set'] !== self::NODE_SET) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        if ((int) ($payload['limits']['operator_seats'] ?? 0) !== self::OPERATOR_SEATS
            || (int) ($payload['limits']['node_limit'] ?? 0) !== self::NODE_LIMIT
            || (string) ($payload['node_set'] ?? '') !== self::NODE_SET) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        // The Bundle grant set is the exact union of both Operator records: no grant may
        // be added, removed, or replaced (a future License Type never enters).
        $expectedGrants = FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes();
        sort($expectedGrants, SORT_STRING);
        $payloadGrants = array_keys((array) ($payload['grants'] ?? []));
        sort($payloadGrants, SORT_STRING);
        $metaGrants = (array) ($meta['grants'] ?? []);
        sort($metaGrants, SORT_STRING);
        if ($payloadGrants !== $expectedGrants || $metaGrants !== $expectedGrants) {
            throw new DomainException('FIXTURE_GRANT_UNION_MISMATCH');
        }
        foreach ((array) ($payload['grants'] ?? []) as $code => $granted) {
            if (!in_array($code, FocusaSpec172LicenseTypeRegistry::underlyingLicenseTypes(), true)
                || $granted !== true) {
                throw new DomainException('FIXTURE_GRANT_UNION_MISMATCH');
            }
        }
        // The family set is the derived union of the two underlying records: no extra
        // family from a future product may appear.
        $expectedFamilies = FocusaSpec172LicenseTypeRegistry::underlyingFamilies();
        sort($expectedFamilies, SORT_STRING);
        $payloadFamilies = array_keys((array) ($payload['features'] ?? []));
        sort($payloadFamilies, SORT_STRING);
        if ($payloadFamilies !== $expectedFamilies) {
            throw new DomainException('FIXTURE_FAMILY_MISMATCH');
        }
        foreach ((array) ($payload['features'] ?? []) as $feature => $enabled) {
            if ($enabled !== true) {
                throw new DomainException('FIXTURE_FAMILY_MISMATCH');
            }
        }
        if ((array) ($payload['family_sets'] ?? []) !== FocusaSpec172LicenseTypeRegistry::familySets()) {
            throw new DomainException('FIXTURE_FAMILY_MISMATCH');
        }
        if ((int) ($payload['human_key_count'] ?? 0) !== FocusaSpec172LicenseTypeRegistry::HUMAN_KEY_COUNT
            || (int) ($payload['human_key_count'] ?? 0) !== (int) $projection['human_key_count']) {
            throw new DomainException('FIXTURE_HUMAN_KEY_MISMATCH');
        }
        if (($payload['future_products_included'] ?? false) === true
            || ($payload['future_license_types_included'] ?? false) === true) {
            throw new DomainException('FIXTURE_FUTURE_PRODUCT_MISMATCH');
        }
        if (($payload['component_refunds_allowed'] ?? false) === true) {
            throw new DomainException('FIXTURE_COMPONENT_REFUND_MISMATCH');
        }
        $issued = new DateTimeImmutable((string) $payload['issued_at'], new DateTimeZone('UTC'));
        $expires = new DateTimeImmutable((string) $payload['expires_at'], new DateTimeZone('UTC'));
        $grace = new DateTimeImmutable((string) $payload['offline_grace_until'], new DateTimeZone('UTC'));
        if ($expires <= $issued || $grace <= $expires) {
            throw new DomainException('FIXTURE_CREDENTIAL_WINDOW_INVALID');
        }
        self::assertNodeId((string) $payload['node_id']);
    }

    private static function assertNodeId(string $nodeId): void
    {
        if ($nodeId === '' || strlen($nodeId) > 128
            || preg_match('/[\r\n@\x00]/', $nodeId) === 1) {
            throw new InvalidArgumentException('bounded node id required');
        }
    }

    private static function opaqueLeaseId(): string
    {
        return 'lease_' . bin2hex(random_bytes(12));
    }

    private static function plusDays(string $timestamp, int $days): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $days . ' days')->format('Y-m-d\TH:i:s\Z');
    }
}
