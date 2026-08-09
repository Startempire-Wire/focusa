<?php
// Spec 172 UIAI grant/child-token fixture (atom focusa-vbcqu.20.15.14). Derives the
// signed UIAI grant payload — the machine execution credential — and the bounded
// operator-node child token exclusively from the canonical UIAI Operator Lifetime v1
// projection produced by the License Type projector:
//
//   - The fixture is server-shaped, never caller-shaped: product, license type, local
//     family features, seat/node limits, price version, family digest, hosted-resource
//     exclusions, term, and sequence all come from the frozen projector output and the
//     frozen UIAI family and hosted-resource exclusion records. A client can never
//     select product, price, License Type, family, feature, limit, node, hosted right,
//     or commercial right.
//   - Lifetime entitlement with bounded credential lifetime (Spec 172 section 14):
//     `term` is `lifetime` while the grant refresh window is bounded (90 days) and
//     offline grace is bounded (30 days past the refresh window). Credential expiry
//     never destroys the underlying entitlement; it only ends the bounded credential.
//   - Explicit local/hosted boundary (Spec 172 sections 6.3, 7.2): the grant carries
//     the eight frozen hosted/metered exclusions (unlimited hosted compute, paid
//     proxies, third-party API consumption, paid model usage, managed hosting, resale,
//     redistribution, product embedding) all denied, plus the frozen exclusion digest.
//     Hosted/metered rights remain denied on every credential; a requested hosted
//     resource resolves to HOSTED_RESOURCE_NOT_INCLUDED.
//   - The child token is the bounded node-scoped credential for the operator's
//     registered node: 15-minute maximum TTL (matching the runtime child-token bound in
//     crates/focusa-license/src/uiai_child_token.rs), an exact subset of the granted
//     local families, the same seat/node limits, and the carried hosted-resource
//     exclusions. The child token never expands the grant and never expires past the
//     grant credential window.
//   - One verified human operator seat and three shared operator nodes
//     (operator_shared_v1) are the initial safe baseline (Spec 172 section 7.3).
//   - Refund/revoke supersede by higher authority sequence: the fixture is active for a
//     given sequence and any higher-sequence revocation replaces it; the fixture never
//     fabricates a lower or equal sequence.
//   - No plaintext leakage: the fixture carries no raw email, license key, token,
//     credential, customer row, or card data.
//
// Requires docs/contracts/spec172-uiai-edd-license-type-projector.v1.php and
// docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php to be loaded
// first (frozen family record, family digest, and exclusion registry).
declare(strict_types=1);

final class UiaiSpec172UiaiGrantChildTokenFixture
{
    public const SCHEMA = 'focusa.spec172.uiai_grant_child_token_fixture.v1';
    public const GRANT_PAYLOAD_SCHEMA = 'focusa.uiai_grant.v1';
    public const CHILD_TOKEN_SCHEMA = 'focusa.uiai_child_token.v1';
    public const PRODUCT = 'uiai_engine';
    public const LICENSE_TYPE = 'uiai_operator_lifetime_v1';
    public const TERM = 'lifetime';
    public const STATUS = 'active';

    /** Bounded credential refresh window for the lifetime entitlement (Spec 172 section 14). */
    public const REFRESH_WINDOW_DAYS = 90;
    /** Bounded offline grace past the refresh window; never expands products or limits. */
    public const OFFLINE_GRACE_DAYS = 30;
    /** Bounded child-token TTL matching the runtime broker bound (uiai_child_token.rs). */
    public const CHILD_TOKEN_MAX_TTL_MINUTES = 15;

    public const OPERATOR_SEATS = 1;
    public const NODE_LIMIT = 3;
    public const NODE_SET = 'operator_shared_v1';

    /**
     * Build the canonical UIAI grant + child-token fixture from exactly one accepted
     * projection. The node id is the operator's registered node binding and the client
     * id is the presenting client; both are bounded tokens. Everything else is derived
     * from the projection and the frozen family/exclusion records.
     */
    public static function fromProjection(array $projection, string $nodeId, string $clientId, callable $clock): array
    {
        if (($projection['schema'] ?? '') !== UiaiSpec172UiaiOperatorProjector::RESULT_SCHEMA) {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        if (($projection['decision'] ?? '') !== 'license_type_projected'
            || ($projection['status'] ?? '') !== 'active') {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        self::assertNodeId($nodeId);
        self::assertClientId($clientId);

        $issuedAt = (string) $clock();
        FocusaSpec172LicenseTypeProjectionMigration::assertTimestamp($issuedAt);
        $expiresAt = self::plusDays($issuedAt, self::REFRESH_WINDOW_DAYS);
        $offlineGraceUntil = self::plusDays($expiresAt, self::OFFLINE_GRACE_DAYS);

        $features = [];
        foreach (UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES as $family) {
            $features[$family] = true;
        }
        $hostedResources = [];
        foreach (UiaiSpec172HostedResourceExclusionRegistry::exclusionList() as $resource) {
            $hostedResources[$resource] = false;
        }
        $exclusionDigest = UiaiSpec172HostedResourceExclusionRegistry::digest();

        $grantId = self::opaqueGrantId();
        $childToken = [
            'schema' => self::CHILD_TOKEN_SCHEMA,
            'token_id' => self::opaqueChildTokenId(),
            'token' => self::opaqueChildTokenSecret(),
            'audience' => 'uiai-engine:operator',
            'node_id' => $nodeId,
            'client_id' => $clientId,
            'grant_lease_id' => $grantId,
            'grant_sequence' => (int) $projection['sequence'],
            'features' => array_keys($features),
            'limits' => [
                'operator_seats' => self::OPERATOR_SEATS,
                'node_limit' => self::NODE_LIMIT,
            ],
            'hosted_resource_exclusion_digest' => $exclusionDigest,
            'issued_at' => $issuedAt,
            'expires_at' => self::plusMinutes($issuedAt, self::CHILD_TOKEN_MAX_TTL_MINUTES),
        ];

        return [
            'schema' => self::SCHEMA,
            'grant' => [
                'schema' => self::GRANT_PAYLOAD_SCHEMA,
                'grant_id' => $grantId,
                'product' => self::PRODUCT,
                'subject_id' => (string) $projection['account_id'],
                'node_id' => $nodeId,
                'sequence' => (int) $projection['sequence'],
                'issued_at' => $issuedAt,
                'not_before' => $issuedAt,
                'expires_at' => $expiresAt,
                'offline_grace_until' => $offlineGraceUntil,
                'authority_key_id' => (string) ($projection['authority_key_id'] ?? 'authority-lease-2026-01'),
                'status' => self::STATUS,
                'features' => $features,
                'limits' => [
                    'operator_seats' => self::OPERATOR_SEATS,
                    'node_limit' => self::NODE_LIMIT,
                ],
                'hosted_resources' => $hostedResources,
                'hosted_resource_exclusion_digest' => $exclusionDigest,
            ],
            'child_token' => $childToken,
            'grant_metadata' => [
                'product' => self::PRODUCT,
                'license_type' => self::LICENSE_TYPE,
                'price_version' => (string) $projection['price_version'],
                'family_digest' => (string) $projection['family_digest'],
                'family_count' => count(UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES),
                'operator_seats' => self::OPERATOR_SEATS,
                'node_limit' => self::NODE_LIMIT,
                'node_set' => self::NODE_SET,
                'term' => self::TERM,
                'refund_policy' => 'whole_order_30_days',
            ],
            'hosted_resource_exclusions' => UiaiSpec172HostedResourceExclusionRegistry::exclusionList(),
            'hosted_resource_exclusion_digest' => $exclusionDigest,
        ];
    }

    /** Fail-closed validation of the derived fixture against its accepted projection. */
    public static function validate(array $fixture, array $projection): void
    {
        if (($fixture['schema'] ?? '') !== self::SCHEMA) {
            throw new DomainException('FIXTURE_SCHEMA_MISMATCH');
        }
        $grant = $fixture['grant'] ?? [];
        $meta = $fixture['grant_metadata'] ?? [];
        if (($grant['schema'] ?? '') !== self::GRANT_PAYLOAD_SCHEMA
            || ($grant['product'] ?? '') !== self::PRODUCT
            || ($meta['license_type'] ?? '') !== self::LICENSE_TYPE
            || ($grant['status'] ?? '') !== self::STATUS) {
            throw new DomainException('FIXTURE_SCOPE_MISMATCH');
        }
        if ((int) $grant['sequence'] !== (int) $projection['sequence'] || (int) $grant['sequence'] < 1) {
            throw new DomainException('FIXTURE_SEQUENCE_MISMATCH');
        }
        if ((string) $grant['subject_id'] !== (string) $projection['account_id']) {
            throw new DomainException('FIXTURE_SUBJECT_MISMATCH');
        }
        if ((string) $meta['price_version'] !== (string) $projection['price_version']
            || (string) $meta['family_digest'] !== (string) $projection['family_digest']
            || (string) $meta['family_digest'] !== UiaiSpec172UiaiOperatorProjector::familyDigest()) {
            throw new DomainException('FIXTURE_GRANT_MISMATCH');
        }
        if ((int) $meta['operator_seats'] !== self::OPERATOR_SEATS
            || (int) $meta['node_limit'] !== self::NODE_LIMIT
            || (string) $meta['node_set'] !== self::NODE_SET) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        if ((int) ($grant['limits']['operator_seats'] ?? 0) !== self::OPERATOR_SEATS
            || (int) ($grant['limits']['node_limit'] ?? 0) !== self::NODE_LIMIT) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        $families = array_keys((array) ($grant['features'] ?? []));
        sort($families, SORT_STRING);
        $frozen = UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES;
        sort($frozen, SORT_STRING);
        if ($families !== $frozen) {
            throw new DomainException('FIXTURE_FAMILY_MISMATCH');
        }
        foreach ((array) ($grant['features'] ?? []) as $feature => $enabled) {
            if ($enabled !== true) {
                throw new DomainException('FIXTURE_FAMILY_MISMATCH');
            }
        }
        self::assertHostedResourceBoundary($grant, $fixture, $projection);
        $issued = new DateTimeImmutable((string) $grant['issued_at'], new DateTimeZone('UTC'));
        $expires = new DateTimeImmutable((string) $grant['expires_at'], new DateTimeZone('UTC'));
        $grace = new DateTimeImmutable((string) $grant['offline_grace_until'], new DateTimeZone('UTC'));
        if ($expires <= $issued || $grace <= $expires) {
            throw new DomainException('FIXTURE_CREDENTIAL_WINDOW_INVALID');
        }
        self::assertNodeId((string) $grant['node_id']);
        self::assertChildToken($fixture['child_token'] ?? [], $grant, $issued);
    }

    /**
     * Explicit local/hosted boundary: the grant carries exactly the frozen exclusion
     * list, all denied, with the frozen exclusion digest, and the fixture-level
     * exclusion fields must match the registry.
     */
    private static function assertHostedResourceBoundary(array $grant, array $fixture, array $projection): void
    {
        $expected = UiaiSpec172HostedResourceExclusionRegistry::exclusionList();
        $hosted = array_keys((array) ($grant['hosted_resources'] ?? []));
        sort($hosted, SORT_STRING);
        sort($expected, SORT_STRING);
        if ($hosted !== $expected) {
            throw new DomainException('FIXTURE_HOSTED_RESOURCE_MISMATCH');
        }
        foreach ((array) ($grant['hosted_resources'] ?? []) as $resource => $granted) {
            if ($granted !== false) {
                throw new DomainException('FIXTURE_HOSTED_RESOURCE_MISMATCH');
            }
        }
        $digest = UiaiSpec172HostedResourceExclusionRegistry::digest();
        if ((string) ($grant['hosted_resource_exclusion_digest'] ?? '') !== $digest
            || (string) ($fixture['hosted_resource_exclusion_digest'] ?? '') !== $digest
            || (string) ($projection['hosted_resource_exclusion_digest'] ?? '') !== $digest) {
            throw new DomainException('FIXTURE_HOSTED_RESOURCE_MISMATCH');
        }
    }

    /** The child token is bounded, node/client-scoped, an exact subset, and never outlives the grant window. */
    private static function assertChildToken(array $token, array $grant, DateTimeImmutable $grantIssued): void
    {
        if (($token['schema'] ?? '') !== self::CHILD_TOKEN_SCHEMA
            || (string) $token['node_id'] !== (string) $grant['node_id']
            || (string) $token['grant_lease_id'] !== (string) $grant['grant_id']
            || (int) $token['grant_sequence'] !== (int) $grant['sequence']) {
            throw new DomainException('FIXTURE_CHILD_TOKEN_MISMATCH');
        }
        $grantedFamilies = array_keys((array) ($grant['features'] ?? []));
        foreach ((array) ($token['features'] ?? []) as $feature) {
            if (!in_array($feature, $grantedFamilies, true)) {
                throw new DomainException('FIXTURE_CHILD_TOKEN_MISMATCH');
            }
        }
        if ((int) ($token['limits']['operator_seats'] ?? 0) !== self::OPERATOR_SEATS
            || (int) ($token['limits']['node_limit'] ?? 0) !== self::NODE_LIMIT) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        if ((string) ($token['hosted_resource_exclusion_digest'] ?? '') !== UiaiSpec172HostedResourceExclusionRegistry::digest()) {
            throw new DomainException('FIXTURE_HOSTED_RESOURCE_MISMATCH');
        }
        $issued = new DateTimeImmutable((string) $token['issued_at'], new DateTimeZone('UTC'));
        $expires = new DateTimeImmutable((string) $token['expires_at'], new DateTimeZone('UTC'));
        $maxExpiry = $grantIssued->modify('+' . self::CHILD_TOKEN_MAX_TTL_MINUTES . ' minutes');
        if ($issued >= $expires || $expires > $maxExpiry) {
            throw new DomainException('FIXTURE_CREDENTIAL_WINDOW_INVALID');
        }
        self::assertNodeId((string) $token['node_id']);
        self::assertClientId((string) $token['client_id']);
    }

    private static function assertNodeId(string $nodeId): void
    {
        if ($nodeId === '' || strlen($nodeId) > 128
            || preg_match('/[\r\n@\x00]/', $nodeId) === 1) {
            throw new InvalidArgumentException('bounded node id required');
        }
    }

    private static function assertClientId(string $clientId): void
    {
        if ($clientId === '' || strlen($clientId) > 128
            || preg_match('/[\r\n@\x00]/', $clientId) === 1) {
            throw new InvalidArgumentException('bounded client id required');
        }
    }

    private static function opaqueGrantId(): string
    {
        return 'uiai_grant_' . bin2hex(random_bytes(12));
    }

    private static function opaqueChildTokenId(): string
    {
        return 'ct_' . bin2hex(random_bytes(12));
    }

    private static function opaqueChildTokenSecret(): string
    {
        return bin2hex(random_bytes(24));
    }

    private static function plusDays(string $timestamp, int $days): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $days . ' days')->format('Y-m-d\TH:i:s\Z');
    }

    private static function plusMinutes(string $timestamp, int $minutes): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $minutes . ' minutes')->format('Y-m-d\TH:i:s\Z');
    }
}
