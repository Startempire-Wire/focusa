<?php
// Spec 172 paid -> limited assertion transition fixtures (addendum sections 9.3, 17,
// and 21; atom focusa-vbcqu.20.15.16). Derives the paid (bundle signed lease)
// credential exclusively from one accepted ACTIVE composite Bundle projection and,
// after a terminal settlement (refund/chargeback/revoke), derives the limited-mode
// posture:
//
//   - A still-mailbox-verified account returns to `verified_no_license` with a signed
//     limited-access assertion carrying ONLY the frozen limited family allowlists
//     (Focusa manual_* six families, UIAI public_* six families, from
//     docs/contracts/spec172-verified-limited-access.v1.yaml) plus the permanent
//     safety allowances (read_projection, basic_customer_data_export, account_control,
//     device_control, license_status, diagnostics, repair, rollback,
//     stable_security_update, uninstall). Paid families, price, and grants are never
//     included after a refund/chargeback/revoke.
//   - An unverified account returns to `unverified` (registration only) — never a
//     grant.
//   - A stale paid credential captured before the settlement can never reactivate:
//     the paid assertion is rejected once the Bundle is terminal
//     (PAID_GRANT_REVOKED) or once the account sequence has moved past it
//     (STALE_CREDENTIAL_SUPERSEDED). Chargeback/revoke therefore propagate through
//     stale cache.
//   - The lifecycle transition matrix (section 21) is exposed here as the single
//     authority for the evidence record.
//
// No raw email, key, token, customer row, credential, or card data is carried.
// Requires docs/contracts/spec172-refund-downgrade-settlement.v1.php and
// docs/contracts/spec172-limited-access-assertion-service.v1.php to be loaded first.
declare(strict_types=1);

/**
 * Paid → limited assertion transition fixture. The paid credential derives from the
 * ACTIVE composite projection; the limited-mode posture derives from the applied
 * terminal settlement; a stale paid credential fails validation closed.
 */
final class FocusaSpec172AssertionTransitionFixture
{
    public const SCHEMA = 'focusa.spec172.assertion_transition_fixture.v1';
    public const PAID_PAYLOAD_SCHEMA = 'focusa.bundle_signed_lease.v1';
    public const LIMITED_PAYLOAD_SCHEMA = 'focusa.spec172.limited_access_assertion.v1';
    public const BUNDLE_SKU = 'focusa_uiai_operator_bundle_lifetime_v1';
    public const GRANTS = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'];
    public const BUNDLE_RESULT_SCHEMA = 'focusa.spec172.bundle_operator_lifetime_projection.v1';
    public const REFRESH_WINDOW_DAYS = 90;
    public const OFFLINE_GRACE_DAYS = 30;
    public const OPERATOR_SEATS = 1;
    public const NODE_LIMIT = 3;
    public const NODE_SET = 'operator_shared_v1';
    public const LIMITED_SIGNER = 'wpuiai.spec172.limited_transition.v1';

    // Frozen limited-mode allowlists (docs/contracts/spec172-verified-limited-access.v1.yaml):
    // verified_no_license limited families plus the permanent safety allowances. Paid
    // families are never included after a refund/chargeback/revoke.
    public const FOCUSA_LIMITED_FAMILIES = [
        'manual_project', 'manual_mission', 'manual_focus_state', 'manual_workpoint',
        'manual_trajectory', 'manual_basic_evidence',
    ];
    public const UIAI_LIMITED_FAMILIES = [
        'public_search', 'source_to_markdown', 'public_page_read', 'accessibility_snapshot',
        'screenshot', 'basic_diagnostics',
    ];
    public const PERMANENT_ALLOWANCES = [
        'read_projection', 'basic_customer_data_export', 'account_control', 'device_control',
        'license_status', 'diagnostics', 'repair', 'rollback', 'stable_security_update',
        'uninstall',
    ];

    public function __construct(
        private FocusaSpec172LimitedAssertionSigner $limitedSigner,
        private string $publicKeyHex = '',
        private string $secretKeyHex = '',
    ) {
        if ($publicKeyHex === '') {
            $this->publicKeyHex = $limitedSigner->publicKeyHex();
        }
        if ($secretKeyHex === '') {
            $this->secretKeyHex = hash('sha256', 'focusa.spec172.limited_transition.v1');
        }
    }

    /** The canonical lifecycle transition matrix (single authority, section 21). */
    public static function transitionMatrix(): array
    {
        return FocusaSpec172RefundDowngradeSettler::TRANSITION_MATRIX;
    }

    /**
     * Derive the paid credential exclusively from one accepted ACTIVE composite Bundle
     * projection. After a terminal settlement the projection no longer yields a paid
     * assertion (PAID_GRANT_REVOKED): the paid grants are removed.
     */
    public function paidAssertion(array $projection, string $nodeId, callable $clock): array
    {
        if (($projection['schema'] ?? '') !== self::BUNDLE_RESULT_SCHEMA) {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        if (($projection['decision'] ?? '') !== 'license_type_projected'
            || ($projection['status'] ?? '') !== 'active') {
            throw new DomainException('PAID_GRANT_REVOKED');
        }
        self::assertNodeId($nodeId);
        $issuedAt = (string) $clock();
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($issuedAt);
        $expiresAt = self::plusDays($issuedAt, self::REFRESH_WINDOW_DAYS);
        $offlineGraceUntil = self::plusDays($expiresAt, self::OFFLINE_GRACE_DAYS);

        $grants = [];
        foreach ((array) ($projection['grants'] ?? []) as $code) {
            $grants[(string) $code] = true;
        }
        $features = [];
        foreach ((array) ($projection['families'] ?? []) as $family) {
            $features[(string) $family] = true;
        }

        return [
            'schema' => self::SCHEMA,
            'kind' => 'paid',
            'assertion_payload' => [
                'schema' => self::PAID_PAYLOAD_SCHEMA,
                'lease_id' => self::opaqueToken('ls_'),
                'product' => self::BUNDLE_SKU,
                'subject_id' => (string) ($projection['account_id'] ?? ''),
                'node_id' => $nodeId,
                'sequence' => (int) ($projection['sequence'] ?? 0),
                'issued_at' => $issuedAt,
                'not_before' => $issuedAt,
                'expires_at' => $expiresAt,
                'offline_grace_until' => $offlineGraceUntil,
                'authority_key_id' => (string) ($projection['authority_key_id'] ?? 'authority-lease-2026-01'),
                'status' => 'active',
                'grants' => $grants,
                'features' => $features,
                'family_sets' => (array) ($projection['family_sets'] ?? []),
                'limits' => [
                    'operator_seats' => self::OPERATOR_SEATS,
                    'node_limit' => self::NODE_LIMIT,
                ],
                'node_set' => self::NODE_SET,
                'human_key_count' => (int) ($projection['human_key_count'] ?? 0),
                'future_products_included' => false,
                'future_license_types_included' => false,
                'component_refunds_allowed' => false,
            ],
            'grant_metadata' => [
                'product' => self::BUNDLE_SKU,
                'license_type' => (string) ($projection['license_type'] ?? ''),
                'grants' => (array) ($projection['grants'] ?? []),
                'price_version' => (string) ($projection['price_version'] ?? ''),
                'family_digest' => (string) ($projection['family_digest'] ?? ''),
                'refund_policy' => 'whole_order_30_days',
            ],
        ];
    }

    /**
     * Derive the limited-mode posture from an applied terminal settlement. A
     * still-mailbox-verified account returns to `verified_no_license` with a signed
     * limited assertion carrying ONLY the frozen limited families and permanent safety
     * allowances; an unverified account returns to `unverified` (registration only).
     */
    public function limitedPosture(array $settlement, string $nodeId, callable $clock): array
    {
        if (($settlement['decision'] ?? '') !== 'applied') {
            throw new DomainException('SETTLEMENT_NOT_APPLIED');
        }
        if (!in_array((string) ($settlement['to_state'] ?? ''), ['refunded', 'revoked'], true)) {
            throw new DomainException('SETTLEMENT_NOT_TERMINAL');
        }
        self::assertNodeId($nodeId);
        $posture = (string) ($settlement['limited_posture'] ?? '');
        if (!in_array($posture, ['verified_no_license', 'unverified'], true)) {
            throw new DomainException('LIMITED_POSTURE_UNKNOWN');
        }
        if ($posture === 'unverified') {
            return [
                'schema' => self::SCHEMA,
                'kind' => 'unverified',
                'product_access' => 'registration_only',
                'paid_grants_active' => false,
                'permanent_allowances' => self::PERMANENT_ALLOWANCES,
            ];
        }

        $families = array_values(array_unique(array_merge(
            self::FOCUSA_LIMITED_FAMILIES,
            self::UIAI_LIMITED_FAMILIES,
            self::PERMANENT_ALLOWANCES,
        )));
        sort($families, SORT_STRING);
        $issuedAt = (string) ($settlement['created_at'] ?? '');
        FocusaSpec172RefundDowngradeMigration::assertTimestamp($issuedAt);
        $refreshAt = self::plusDays($issuedAt, self::REFRESH_WINDOW_DAYS);
        $payload = FocusaSpec172LimitedAssertionPayload::build([
            'posture_uuid' => self::opaqueToken('po_'),
            'account_uuid' => (string) ($settlement['account_id'] ?? ''),
            'identity_uuid' => self::opaqueIdentity((string) ($settlement['account_id'] ?? '')),
            'product_scope' => self::BUNDLE_SKU,
            'node_uuid' => $nodeId,
            'family_allowlist' => $families,
            'sequence' => (int) ($settlement['result_sequence'] ?? 0),
            'issued_at' => $issuedAt,
            'refresh_at' => $refreshAt,
            'signer' => self::LIMITED_SIGNER,
        ]);
        $signature = $this->limitedSigner->sign($payload);
        return [
            'schema' => self::SCHEMA,
            'kind' => 'verified_no_license',
            'paid_grants_active' => false,
            'grants_revoked' => (int) ($settlement['grants_revoked'] ?? 0),
            'sequence' => (int) ($settlement['result_sequence'] ?? 0),
            'families_allowed' => $families,
            'paid_families_excluded' => true,
            'permanent_allowances' => self::PERMANENT_ALLOWANCES,
            'assertion' => $payload,
            'signature' => $signature,
            'signature_algorithm' => FocusaSpec172LimitedAssertionSigner::ALGORITHM,
        ];
    }

    /** Verify the derived limited assertion (bounded; no raw email or secrets). */
    public function verifyLimited(array $posture): array
    {
        if (($posture['kind'] ?? '') !== 'verified_no_license') {
            return ['valid' => false, 'error_code' => 'VERIFIED_LIMITED_ACCESS'];
        }
        $payload = $posture['assertion'] ?? [];
        $signature = (string) ($posture['signature'] ?? '');
        if (($payload['schema'] ?? '') !== self::LIMITED_PAYLOAD_SCHEMA
            || (string) ($payload['product_scope'] ?? '') !== self::BUNDLE_SKU) {
            return ['valid' => false, 'error_code' => 'LIMITED_SCOPE_MISMATCH'];
        }
        if (!$this->limitedSigner->verify($payload, $signature)) {
            return ['valid' => false, 'error_code' => 'LIMITED_SIGNATURE_INVALID'];
        }
        $allowed = (array) ($payload['family_allowlist'] ?? []);
        foreach ($allowed as $family) {
            if (!in_array($family, array_merge(self::FOCUSA_LIMITED_FAMILIES, self::UIAI_LIMITED_FAMILIES, self::PERMANENT_ALLOWANCES), true)) {
                return ['valid' => false, 'error_code' => 'LIMITED_FAMILY_WIDENING_DENIED'];
            }
        }
        return ['valid' => true, 'error_code' => null];
    }

    /**
     * A stale paid credential cannot reactivate a settled Bundle: a paid assertion whose
     * sequence is below the account's current highest authority sequence, or that is
     * presented while the Bundle is terminal, fails closed (STALE_CREDENTIAL_SUPERSEDED /
     * PAID_GRANT_REVOKED). Chargeback/revoke therefore propagate through stale cache.
     */
    public static function validatePaidAssertion(array $assertion, int $highestSequence, string $effectiveState): void
    {
        $payload = $assertion['assertion_payload'] ?? [];
        if (($payload['schema'] ?? '') !== self::PAID_PAYLOAD_SCHEMA
            || ($payload['status'] ?? '') !== 'active') {
            throw new DomainException('PAID_GRANT_REVOKED');
        }
        if ($effectiveState !== 'active') {
            throw new DomainException('PAID_GRANT_REVOKED');
        }
        if ((int) ($payload['sequence'] ?? 0) < $highestSequence) {
            throw new DomainException('STALE_CREDENTIAL_SUPERSEDED');
        }
        $grantCodes = array_keys((array) ($payload['grants'] ?? []));
        sort($grantCodes, SORT_STRING);
        $expected = self::GRANTS;
        sort($expected, SORT_STRING);
        if ($grantCodes !== $expected) {
            throw new DomainException('PAID_GRANT_UNION_MISMATCH');
        }
        if ((int) ($payload['human_key_count'] ?? 0) !== 1
            || ($payload['component_refunds_allowed'] ?? false) === true
            || ($payload['future_products_included'] ?? false) === true) {
            throw new DomainException('PAID_ASSERTION_INVALID');
        }
    }

    private static function opaqueIdentity(string $accountUuid): string
    {
        return 'id_' . substr(hash('sha256', 'focusa.spec172.limited_identity.v1' . $accountUuid), 0, 32);
    }

    private static function assertNodeId(string $nodeId): void
    {
        if ($nodeId === '' || strlen($nodeId) > 96 || preg_match('/[\r\n\x00@]/', $nodeId) === 1) {
            throw new InvalidArgumentException('bounded node id required');
        }
    }

    private static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    private static function plusDays(string $timestamp, int $days): string
    {
        return (new DateTimeImmutable($timestamp, new DateTimeZone('UTC')))
            ->modify('+' . $days . ' days')->format('Y-m-d\TH:i:s\Z');
    }
}
