<?php
// Spec 172 Focusa paid lease fixture (atom focusa-vbcqu.20.15.13). Derives the signed
// lease payload — the machine execution credential — exclusively from the canonical
// Focusa Operator Lifetime v1 projection produced by the License Type projector:
//
//   - The fixture is server-shaped, never caller-shaped: product, license type, family
//     features, seat/node limits, price version, family digest, term, and sequence all
//     come from the frozen projector output and the frozen family record. A client can
//     never select product, price, License Type, family, feature, limit, node, or
//     commercial right.
//   - Lifetime entitlement with bounded credential lifetime (Spec 172 section 14):
//     `term` is `lifetime` while the credential refresh window is bounded (90 days) and
//     offline grace is bounded (30 days past the refresh window). Credential expiry never
//     destroys the underlying entitlement; it only ends the bounded credential.
//   - The payload schema and field names match the authority lease payload
//     (`focusa.authority_lease.v1`): lease_id, product, subject_id, node_id, sequence,
//     issued_at, not_before, expires_at, offline_grace_until, authority_key_id, status,
//     features, limits. Grant metadata (license_type_ref, price_version, family_digest,
//     node_set, term) is carried explicitly so the runtime cannot re-derive commercial
//     truth from stale or local tables.
//   - One verified human operator seat and three shared operator nodes
//     (operator_shared_v1) are the initial safe baseline (Spec 172 section 7.3); the
//     same three node identities are shared across Bundle products rather than creating
//     unrelated activations.
//   - Refund/revoke supersede by higher authority sequence: the fixture is active for a
//     given sequence and any higher-sequence revocation replaces it; the fixture never
//     fabricates a lower or equal sequence.
//   - No plaintext leakage: the fixture carries no raw email, license key, token,
//     credential, customer row, or card data.
//
// Requires docs/contracts/spec172-edd-license-type-projector.v1.php to be loaded first
// (frozen family record and family digest).
declare(strict_types=1);

final class FocusaSpec172FocusaPaidLeaseFixture
{
    public const SCHEMA = 'focusa.spec172.focusa_paid_lease_fixture.v1';
    public const LEASE_PAYLOAD_SCHEMA = 'focusa.authority_lease.v1';
    public const PRODUCT = 'focusa';
    public const LICENSE_TYPE = 'focusa_operator_lifetime_v1';
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
     * Build the canonical paid lease payload from exactly one accepted projection.
     * The node id is the operator's registered node binding (bounded token); everything
     * else is derived from the projection and the frozen family record.
     */
    public static function fromProjection(array $projection, string $nodeId, callable $clock): array
    {
        if (($projection['schema'] ?? '') !== FocusaSpec172FocusaOperatorProjector::RESULT_SCHEMA) {
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

        $features = [];
        foreach (FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES as $family) {
            $features[$family] = true;
        }

        return [
            'schema' => self::SCHEMA,
            'lease_payload' => [
                'schema' => self::LEASE_PAYLOAD_SCHEMA,
                'lease_id' => self::opaqueLeaseId(),
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
            ],
            'grant_metadata' => [
                'product' => self::PRODUCT,
                'license_type' => self::LICENSE_TYPE,
                'price_version' => (string) $projection['price_version'],
                'family_digest' => (string) $projection['family_digest'],
                'family_count' => count(FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES),
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
            || ($payload['product'] ?? '') !== self::PRODUCT
            || ($meta['license_type'] ?? '') !== self::LICENSE_TYPE
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
            || (string) $meta['family_digest'] !== FocusaSpec172FocusaOperatorProjector::familyDigest()) {
            throw new DomainException('FIXTURE_GRANT_MISMATCH');
        }
        if ((int) $meta['operator_seats'] !== self::OPERATOR_SEATS
            || (int) $meta['node_limit'] !== self::NODE_LIMIT
            || (string) $meta['node_set'] !== self::NODE_SET) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        if ((int) ($payload['limits']['operator_seats'] ?? 0) !== self::OPERATOR_SEATS
            || (int) ($payload['limits']['node_limit'] ?? 0) !== self::NODE_LIMIT) {
            throw new DomainException('FIXTURE_LIMIT_MISMATCH');
        }
        $families = array_keys((array) ($payload['features'] ?? []));
        sort($families, SORT_STRING);
        $frozen = FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES;
        sort($frozen, SORT_STRING);
        if ($families !== $frozen) {
            throw new DomainException('FIXTURE_FAMILY_MISMATCH');
        }
        foreach ((array) ($payload['features'] ?? []) as $feature => $enabled) {
            if ($enabled !== true) {
                throw new DomainException('FIXTURE_FAMILY_MISMATCH');
            }
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
