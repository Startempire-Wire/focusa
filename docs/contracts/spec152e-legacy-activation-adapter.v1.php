<?php
// Legacy customer/key activation adapter. Gates new-node activation, reissue, terminal
// delivery, and account-merge pre-check for legacy EDD customers: mailbox verification is
// mandatory before any of them, the legacy EDD Software Licensing key must resolve to a
// usable license owned by the verified identity, and the resolution must be evidence-backed.
// A key and an unrelated verified email cannot activate a node; raw matching email alone
// never transfers ownership. Synthetic/unknown records remain quarantined. The adapter is
// read-only: EDD order/license/customer truth is never mutated here.
declare(strict_types=1);

final class FocusaSpec152eLegacyActivationAdapter
{
    public const SCHEMA = 'focusa.spec152e.legacy_activation_adapter.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.legacy_activation_resolution.v1';
    public const VERSION = 1;

    public const PURPOSES = ['node_activation', 'reissue', 'terminal_delivery', 'account_merge'];
    public const LEGACY_EVIDENCE_KINDS = ['purchase_evidence', 'stripe_reconciliation', 'install_site_migration'];
    private const UNUSABLE_LICENSE_STATUSES = ['revoked', 'disabled'];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eEddCustomerAdapter $edd,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Resolve a legacy EDD Software Licensing key for one gated purpose after mailbox
     * verification. Returns a masked, deterministic decision envelope; fails closed with
     * public-safe codes. Never returns the raw key, the raw email, or any secret.
     *
     * Required input:
     *   - registration_uuid:  mailbox-verified registration UUID
     *   - verified_email:     exact email that was verified (must bind the registration)
     *   - license_key:        legacy EDD Software Licensing key to resolve
     *   - purpose:            'node_activation' | 'reissue' | 'terminal_delivery' | 'account_merge'
     *   - legacy_evidence:    evidence-backed legacy record provenance (kind/source/record)
     *   - request_id:         bounded request ID
     */
    public function resolveForActivation(array $input): array
    {
        $this->assertUuid((string) ($input['registration_uuid'] ?? ''), 'registration');
        $purpose = (string) ($input['purpose'] ?? '');
        if (!in_array($purpose, self::PURPOSES, true)) {
            throw new InvalidArgumentException('legacy resolution purpose required');
        }
        $normalized = FocusaSpec152eEmailNormalizer::exact((string) ($input['verified_email'] ?? ''));
        $licenseKey = (string) ($input['license_key'] ?? '');
        if ($licenseKey === '' || strlen($licenseKey) > 191 || preg_match('/[\r\n\x00]/', $licenseKey)) {
            throw new InvalidArgumentException('bounded legacy EDD license key required');
        }
        $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $evidence = $input['legacy_evidence'] ?? [];
        if (!is_array($evidence) || $evidence === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $evidenceDigest = self::validateLegacyEvidence($evidence);

        $registration = $this->registrations->findByUuid((string) $input['registration_uuid']);
        $now = ($this->clock)();
        self::assertTimestamp($now);
        if (!in_array((string) $registration['state'], [
            FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
            FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        ], true) || (string) $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($registration['expires_at'] !== null && $now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        $registrationDigest = $this->registrationSecrets->emailLookupDigest($normalized);
        if (!hash_equals((string) $registration['email_lookup_digest'], $registrationDigest)) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }

        $license = $this->edd->findLicenseByKey($licenseKey);
        if ($license === null) {
            throw new DomainException('EDD_LICENSE_UNVERIFIED');
        }
        if (in_array((string) $license['status'], self::UNUSABLE_LICENSE_STATUSES, true)) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $customerId = (int) $license['customer_id'];
        if ($customerId < 1 || (int) $license['product_id'] < 1) {
            throw new DomainException('EDD_LICENSE_UNVERIFIED');
        }
        $customer = $this->edd->findCustomerById($customerId);
        if ($customer === null) {
            throw new DomainException('EDD_LICENSE_UNVERIFIED');
        }
        // A key and an unrelated verified email cannot activate a node; raw matching
        // email alone does not transfer ownership. The verified email must be an owner
        // email of the license's customer (primary or verified linked address).
        if (!$this->edd->customerHasEmail($customerId, $normalized)) {
            throw new DomainException('LICENSE_ACCOUNT_MISMATCH');
        }
        $orderId = $license['order_id'] !== null ? (int) $license['order_id'] : 0;
        if ($orderId < 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $order = $this->edd->findOrderById($orderId);
        if ($order === null || (int) $order['customer_id'] !== $customerId
            || (string) $order['status'] !== 'complete') {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        // Existing order/license state is preserved: the adapter only reads and returns
        // masked references; statuses, purchase values, and refund/revoke truth stay put.
        return [
            'schema' => self::RESULT_SCHEMA,
            'purpose' => $purpose,
            'verification_required' => false,
            'owner_match' => true,
            'node_activation_allowed' => $purpose === 'node_activation',
            'reissue_allowed' => $purpose === 'reissue',
            'terminal_delivery_allowed' => $purpose === 'terminal_delivery',
            'account_merge_allowed' => $purpose === 'account_merge',
            'license_id' => (int) $license['id'],
            'customer_id' => $customerId,
            'order_id' => $orderId,
            'product_id' => (int) $license['product_id'],
            'status' => (string) $license['status'],
            'evidence_digest' => $evidenceDigest,
        ];
    }

    /**
     * Validate evidence-backed legacy provenance. Synthetic or unknown kinds, missing
     * source/record, and unbounded values remain quarantined (EDD_ORDER_UNVERIFIED).
     * Returns the bounded evidence digest used to pin the resolution.
     */
    public static function validateLegacyEvidence(array $evidence): string
    {
        if ($evidence === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $kind = (string) ($evidence['kind'] ?? '');
        if (!in_array($kind, self::LEGACY_EVIDENCE_KINDS, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $source = (string) ($evidence['source'] ?? '');
        $record = (string) ($evidence['record'] ?? '');
        if ($source === '' || $record === ''
            || strlen($source) > 191 || strlen($record) > 191
            || preg_match('/[\r\n\x00]/', $source) === 1
            || preg_match('/[\r\n\x00]/', $record) === 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return hash('sha256', FocusaSpec152eAccountPromotionMigration::encodeCanonical($evidence));
    }

    public static function assertTimestamp(string $timestamp): void
    {
        $parsed = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
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
}
