<?php
// Spec 152E checkout email integrity (addendum section 6.4). The EDD checkout
// validation/completion surface locks the checkout email to the mailbox-verified
// registration identity:
//
//   - A protected order completes only when the order email matches the verified
//     registration email digest, or when it is an already mailbox-verified identity
//     safely linked to the same promoted authority account. Both paths require prior
//     mailbox control; payment success alone can never promote an email or issue a key.
//   - A changed, blank, or conflicting checkout email (or an order placed under a
//     customer/account that does not match the registration) holds fulfillment: the
//     handler journals a bounded, public-safe hold with an opaque review handle,
//     marks entitlement_allowed false, and issues nothing. The hold persists across
//     repeats and retries until a separately verified link review releases it.
//   - Release requires the held email to be an existing mailbox-verified identity of
//     the exact same authority account, plus a bounded opaque evidence handle from the
//     verification/link review. Evidence kind is fixed to 'verified_link_review';
//     payment, operator, and facade evidence can never release a hold. Releasing
//     marks entitlement_ready but issuance stays deferred to the verified issuance
//     service: this handler never creates an EDD order, license, key, or lease.
//   - Journals store only keyed digests and opaque handles: no raw email, no secret,
//     no unmasked real-email evidence. Rollback is preservation-only.
//
// Failures are public-safe stable codes (EMAIL_VERIFICATION_REQUIRED,
// ACCOUNT_EMAIL_MISMATCH, ACCOUNT_MERGE_REVIEW_REQUIRED, EDD_ORDER_UNVERIFIED,
// EDD_ORDER_PENDING, REFUNDED, REVOKED, PRODUCT_MAPPING_REQUIRED,
// EDD_CHECKOUT_REQUIRED, FACADE_ORIGIN_DENIED, FACADE_PRODUCT_DENIED,
// EDD_LICENSE_UNUSABLE, IDEMPOTENCY_CONFLICT). No new error code is introduced.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-email-identity.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-verified-registration-token-validator.v1.php, and the
// server-owned product/facade registries to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eCheckoutEmailIntegrityMigration
{
    public const SCHEMA = 'focusa.spec152e.checkout_email_integrity.v1';
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
        $holds = $this->table('wpuiai_checkout_email_integrity_holds');
        $releases = $this->table('wpuiai_checkout_email_integrity_releases');
        $migrations = $this->table('wpuiai_checkout_email_integrity_schema_migrations');
        $events = $this->table('wpuiai_checkout_email_integrity_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$holds} (
            hold_key VARCHAR(64) NOT NULL PRIMARY KEY,
            registration_uuid {$uuid} NOT NULL,
            order_id BIGINT NOT NULL,
            account_uuid {$uuid} NULL,
            customer_id BIGINT NULL,
            facade_id VARCHAR(96) NULL,
            product_code VARCHAR(128) NULL,
            expected_email_lookup_digest VARCHAR(64) NULL,
            order_email_lookup_digest VARCHAR(64) NULL,
            mismatch_kind VARCHAR(16) NOT NULL CHECK (mismatch_kind IN ('none', 'blank', 'changed', 'conflicting', 'account')),
            hold_state VARCHAR(20) NOT NULL CHECK (hold_state IN ('passed', 'held', 'released', 'no_entitlement')),
            review_handle VARCHAR(64) NULL,
            error_code VARCHAR(64) NULL,
            released_identity_uuid {$uuid} NULL,
            released_at VARCHAR(32) NULL,
            result_payload TEXT NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            UNIQUE (registration_uuid, order_id)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_checkout_email_hold_idempotency
            ON {$holds} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_checkout_email_hold_order
            ON {$holds} (order_id, hold_state)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_checkout_email_hold_retention
            ON {$holds} (retention_until)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$releases} (
            release_key VARCHAR(64) NOT NULL PRIMARY KEY,
            hold_key VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            resolved_identity_uuid {$uuid} NOT NULL,
            evidence_kind VARCHAR(32) NOT NULL,
            evidence_handle VARCHAR(64) NOT NULL,
            decided_by VARCHAR(32) NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            UNIQUE (hold_key, idempotency_key)
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_checkout_email_release_idempotency
            ON {$releases} (idempotency_key)");
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

    /** Rollback is preservation-only: holds and release journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_checkout_email_integrity_schema_events');
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
        if (!is_string($timestamp)
            || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/D', $timestamp) !== 1) {
            throw new InvalidArgumentException('bounded UTC timestamp required');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        $normalize = static function (mixed $item) use (&$normalize): mixed {
            if (is_array($item)) {
                ksort($item, SORT_STRING);
                foreach ($item as $key => $nested) {
                    $item[$key] = $normalize($nested);
                }
                return $item;
            }
            return $item;
        };
        return json_encode($normalize($value), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }
}

final class FocusaSpec152eCheckoutEmailIntegrityService
{
    public const SCHEMA = 'focusa.spec152e.checkout_email_integrity.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.checkout_email_integrity_decision.v1';
    public const VERSION = 1;
    public const RETENTION_SECONDS = 2592000;
    public const RELEASE_EVIDENCE_KIND = 'verified_link_review';
    public const DECIDED_BY = 'verified_link_review';
    public const HOLD_STATE_HELD = 'held';
    public const HOLD_STATE_RELEASED = 'released';

    private const CREDIT_PACK_REASON_PREFIX = 'credit_pack_';
    private const UNRELATED_DISPOSITION = 'quarantine';
    private const MISMATCH_NONE = 'none';
    private const MISMATCH_BLANK = 'blank';
    private const MISMATCH_CHANGED = 'changed';
    private const MISMATCH_CONFLICTING = 'conflicting';
    private const MISMATCH_ACCOUNT = 'account';

    private const CLIENT_CONTROLLED_FIELDS = [
        'price', 'amount', 'total', 'tier', 'products', 'product_code', 'license_type',
        'license_type_ref', 'capability_family', 'families', 'features', 'grants', 'limits',
        'node_limit', 'sale_status', 'refund_policy', 'upgrade_policy', 'commercial_rights',
        'evaluation_duration', 'edd_download_id', 'edd_price_id',
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eCheckoutEmailIntegrityMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        private FocusaSpec152eEmailIdentityRepository $identities,
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
     * Checkout email integrity assessment for an EDD order completion/validation.
     * Protected orders require a verified, promoted registration bound to the exact
     * facade/product, an order customer/account that matches the registration, and an
     * order email that matches the verified registration identity (or an already
     * mailbox-verified identity safely linked to the same account). A changed, blank,
     * or conflicting email holds fulfillment: the assessment journals a bounded hold
     * with an opaque review handle and entitlement_allowed false; nothing is issued.
     * The hold persists across repeats until a separate verified-link review releases
     * it. Unrelated/credit-pack order items never carry Focusa/UIAI entitlement.
     *
     * Required input:
     *   - order_id (int), order_status, customer_id (int), order_email
     *   - order_items: list of ['download_id' => int, 'price_id' => string, 'quantity' => int]
     *   - registration_uuid, facade_id, origin (registration_uuid required when the
     *     order contains a protected item)
     *   - request_id, idempotency_key
     *
     * Returns a public-safe decision (never raw email, secrets, or keys). Replays with
     * the same idempotency key return the same decision; a repeated canonical request
     * for the same order/registration returns the existing decision.
     */
    public function assessOrder(array $input): array
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
        $this->assertToken($facadeId, 96, 'facade');
        $this->assertToken($origin, 191, 'origin');

        $digest = $this->requestDigest([
            'operation' => 'assess_order',
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
        $replay = $this->replayDecision('assess_order', $idempotencyKey, $digest);
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

        $protected = [];
        $excluded = [];
        $registration = null;
        $accountId = null;
        $expectedDigest = null;
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
            // exact facade, plus the exact facade origin from the facade registry.
            if ($registration === null) {
                $registration = $this->assertVerifiedRegistration($registrationUuid, $facadeId, true);
                $this->assertFacadeSupports($facadeId, $origin, '');
                $accountId = (string) $registration['account_uuid'];
                $expectedDigest = (string) $registration['email_lookup_digest'];
            }

            // Protected item: facade/product binding, server-owned price, no duplicate.
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
            // Unrelated order: no protected item, no identity requirement, no entitlement.
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'no_entitlement',
                'order_id' => $orderId,
                'protected_items' => 0,
                'excluded_items' => $excluded,
                'issuance' => 'none',
                'facade_id' => $facadeId,
            ];
            return $this->recordAssessment(
                $registrationUuid, $orderId, $facadeId, $decision,
                self::MISMATCH_NONE, 'no_entitlement', null, null, null,
                $requestId, $idempotencyKey, $digest,
            );
        }

        // Account integrity: the order must settle against the registration's promoted
        // customer/account. Any other customer or account is an account mismatch hold.
        $registrationCustomer = (int) $registration['edd_customer_id'];
        $orderAccount = $this->accounts->findByCustomerId($customerId);
        $accountMismatch = $registrationCustomer !== $customerId
            || $orderAccount === null
            || !hash_equals($accountId, (string) $orderAccount['account_uuid']);
        if ($accountMismatch) {
            $decision = $this->heldDecision(
                $registrationUuid, $orderId, $accountId, $customerId, $facadeId, $protected, $excluded,
                self::MISMATCH_ACCOUNT, 'ACCOUNT_MERGE_REVIEW_REQUIRED', $expectedDigest, null,
            );
            return $this->recordAssessment(
                $registrationUuid, $orderId, $facadeId, $decision,
                self::MISMATCH_ACCOUNT, self::HOLD_STATE_HELD, 'ACCOUNT_MERGE_REVIEW_REQUIRED',
                $expectedDigest, null, $requestId, $idempotencyKey, $digest,
            );
        }

        // Checkout email integrity: lock the order email to the verified identity.
        $primaryProduct = (string) $protected[0]['product_code'];
        if ($orderEmail === '') {
            $decision = $this->heldDecision(
                $registrationUuid, $orderId, $accountId, $customerId, $facadeId, $protected, $excluded,
                self::MISMATCH_BLANK, 'EDD_ORDER_UNVERIFIED', $expectedDigest, null,
            );
            return $this->recordAssessment(
                $registrationUuid, $orderId, $facadeId, $decision,
                self::MISMATCH_BLANK, self::HOLD_STATE_HELD, 'EDD_ORDER_UNVERIFIED',
                $expectedDigest, null, $requestId, $idempotencyKey, $digest,
            );
        }

        $orderDigest = $this->emailLookupDigest($orderEmail);
        if (hash_equals($expectedDigest, $orderDigest)) {
            // Matching verified checkout email: proceeds.
            $decision = [
                'schema' => self::RESULT_SCHEMA,
                'decision' => 'email_integrity_passed',
                'order_id' => $orderId,
                'registration_id' => $registrationUuid,
                'account_id' => $accountId,
                'customer_id' => $customerId,
                'facade_id' => $facadeId,
                'product_code' => $primaryProduct,
                'protected_items' => $protected,
                'excluded_items' => $excluded,
                'entitlement_allowed' => true,
                'issuance' => 'deferred_to_verified_issuance_service',
                'email_matches_verified_identity' => true,
            ];
            return $this->recordAssessment(
                $registrationUuid, $orderId, $facadeId, $decision,
                self::MISMATCH_NONE, 'passed', null, $expectedDigest, $orderDigest,
                $requestId, $idempotencyKey, $digest,
            );
        }

        // Changed email: resolve against the verified identity registry. A verified
        // identity of the exact same account is already safely linked (prior mailbox
        // control); a verified identity of a different account is a conflict; no
        // verified identity at all is an unverified promotion attempt.
        $identity = $this->identities->findExact($orderEmail);
        if ($identity !== null) {
            $verifiedIdentity = $identity['verified_at'] !== null
                && in_array((string) ($identity['identity_state'] ?? ''), ['primary', 'linked'], true);
            if ($verifiedIdentity && hash_equals($accountId, (string) $identity['account_uuid'])) {
                // Already verified and safely linked to the same account: proceeds.
                $decision = [
                    'schema' => self::RESULT_SCHEMA,
                    'decision' => 'email_integrity_passed',
                    'order_id' => $orderId,
                    'registration_id' => $registrationUuid,
                    'account_id' => $accountId,
                    'customer_id' => $customerId,
                    'facade_id' => $facadeId,
                    'product_code' => $primaryProduct,
                    'protected_items' => $protected,
                    'excluded_items' => $excluded,
                    'entitlement_allowed' => true,
                    'issuance' => 'deferred_to_verified_issuance_service',
                    'email_matches_verified_identity' => true,
                    'verified_identity_id' => (string) $identity['identity_uuid'],
                ];
                return $this->recordAssessment(
                    $registrationUuid, $orderId, $facadeId, $decision,
                    self::MISMATCH_NONE, 'passed', null, $expectedDigest, $orderDigest,
                    $requestId, $idempotencyKey, $digest,
                );
            }
            $kind = self::MISMATCH_CONFLICTING;
            $code = 'ACCOUNT_MERGE_REVIEW_REQUIRED';
        } else {
            $kind = self::MISMATCH_CHANGED;
            $code = 'ACCOUNT_EMAIL_MISMATCH';
        }

        $decision = $this->heldDecision(
            $registrationUuid, $orderId, $accountId, $customerId, $facadeId, $protected, $excluded,
            $kind, $code, $expectedDigest, $orderDigest,
        );
        return $this->recordAssessment(
            $registrationUuid, $orderId, $facadeId, $decision,
            $kind, self::HOLD_STATE_HELD, $code, $expectedDigest, $orderDigest,
            $requestId, $idempotencyKey, $digest,
        );
    }

    /**
     * Verification/link review release: resolves a held fulfillment only after the held
     * order email is proven to be an existing mailbox-verified identity of the exact
     * same authority account, with a bounded opaque evidence handle from the review.
     * Evidence kind is fixed to 'verified_link_review'; payment/operator/facade
     * evidence can never release a hold. Releasing marks entitlement_ready but
     * issuance stays deferred to the verified issuance service.
     *
     * Required input:
     *   - hold_key: opaque hold handle returned by assessOrder
     *   - order_email: the exact held checkout email (re-normalized and re-hashed)
     *   - resolved_identity_uuid: the mailbox-verified identity of that email
     *   - release_evidence_handle: bounded opaque evidence handle from the review
     *   - request_id, idempotency_key
     *   - evidence_kind: optional; only 'verified_link_review' is accepted
     */
    public function releaseHold(array $input): array
    {
        $holdKey = (string) ($input['hold_key'] ?? '');
        $orderEmail = (string) ($input['order_email'] ?? '');
        $identityUuid = (string) ($input['resolved_identity_uuid'] ?? '');
        $evidenceHandle = (string) ($input['release_evidence_handle'] ?? '');
        $evidenceKind = (string) ($input['evidence_kind'] ?? self::RELEASE_EVIDENCE_KIND);
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertRequestId($requestId);
        $this->assertIdempotencyKey($idempotencyKey);
        $this->assertToken($holdKey, 64, 'hold');
        $this->assertToken($evidenceHandle, 64, 'release evidence handle');
        if ($orderEmail === '') {
            throw new InvalidArgumentException('order email is required');
        }
        $this->assertUuid($identityUuid, 'identity');
        if (!hash_equals(self::RELEASE_EVIDENCE_KIND, $evidenceKind)) {
            // Payment success alone can never release a hold: only a verified link
            // review may. No local/self-issued entitlement can bypass the hold.
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }

        $digest = $this->requestDigest([
            'operation' => 'release_hold',
            'hold_key' => $holdKey,
            'order_email_lookup_digest' => $this->emailLookupDigest($orderEmail),
            'resolved_identity_uuid' => $identityUuid,
            'evidence_kind' => $evidenceKind,
            'evidence_handle' => $evidenceHandle,
            'request_id' => $requestId,
        ]);
        $replay = $this->replayDecision('release_hold', $idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        $hold = $this->findHoldByKey($holdKey);
        if ($hold === null) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ((string) $hold['hold_state'] === self::HOLD_STATE_RELEASED) {
            $release = $this->findReleaseByHoldKey($holdKey);
            if ($release === null) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            return $this->presentReleased($hold, $release, false, true);
        }
        if ((string) $hold['hold_state'] !== self::HOLD_STATE_HELD) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ($hold['order_email_lookup_digest'] === null
            || !hash_equals((string) $hold['order_email_lookup_digest'], $this->emailLookupDigest($orderEmail))) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }

        // The resolved identity must be the mailbox-verified identity of exactly the
        // held email, bound to the exact same authority account.
        $identity = $this->identities->findExact($orderEmail);
        if ($identity === null
            || !hash_equals((string) $identity['identity_uuid'], $identityUuid)
            || $identity['verified_at'] === null
            || !in_array((string) ($identity['identity_state'] ?? ''), ['primary', 'linked'], true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!hash_equals((string) $hold['account_uuid'], (string) $identity['account_uuid'])) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }

        $now = $this->now();
        $releaseKey = self::opaqueToken('rl_');
        $holds = $this->schema->table('wpuiai_checkout_email_integrity_holds');
        $releases = $this->schema->table('wpuiai_checkout_email_integrity_releases');
        $this->db->beginTransaction();
        try {
            $update = $this->db->prepare("UPDATE {$holds}
                SET hold_state = :state, released_identity_uuid = :identity, released_at = :released, updated_at = :updated
                WHERE hold_key = :hold AND hold_state = 'held'");
            $update->execute([
                ':state' => self::HOLD_STATE_RELEASED,
                ':identity' => $identityUuid,
                ':released' => $now,
                ':updated' => $now,
                ':hold' => $holdKey,
            ]);
            if ($update->rowCount() !== 1) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            $statement = $this->db->prepare("INSERT INTO {$releases}
                (release_key, hold_key, registration_uuid, resolved_identity_uuid, evidence_kind,
                 evidence_handle, decided_by, request_id, idempotency_key, request_digest, created_at)
                VALUES (:key, :hold, :registration, :identity, :kind, :evidence, :decided,
                        :request, :idempotency, :digest, :created)");
            $statement->execute([
                ':key' => $releaseKey,
                ':hold' => $holdKey,
                ':registration' => (string) $hold['registration_uuid'],
                ':identity' => $identityUuid,
                ':kind' => $evidenceKind,
                ':evidence' => $evidenceHandle,
                ':decided' => self::DECIDED_BY,
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':digest' => $digest,
                ':created' => $now,
            ]);
            $release = [
                'release_key' => $releaseKey,
                'hold_key' => $holdKey,
                'registration_uuid' => (string) $hold['registration_uuid'],
                'resolved_identity_uuid' => $identityUuid,
                'evidence_kind' => $evidenceKind,
                'evidence_handle' => $evidenceHandle,
                'decided_by' => self::DECIDED_BY,
                'request_id' => $requestId,
                'idempotency_key' => $idempotencyKey,
                'request_digest' => $digest,
                'created_at' => $now,
            ];
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }

        return $this->presentReleased($hold, $release, false, false);
    }

    /** Bounded journal lookups for settlement/reconciliation and tests. */
    public function holdCount(): int
    {
        $table = $this->schema->table('wpuiai_checkout_email_integrity_holds');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function releaseCount(): int
    {
        $table = $this->schema->table('wpuiai_checkout_email_integrity_releases');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function findByHoldKey(string $holdKey): ?array
    {
        $this->assertToken($holdKey, 64, 'hold');
        return $this->findHoldByKey($holdKey);
    }

    // ── private helpers ────────────────────────────────────────────────

    private function heldDecision(
        string $registrationUuid,
        int $orderId,
        string $accountId,
        int $customerId,
        string $facadeId,
        array $protected,
        array $excluded,
        string $mismatchKind,
        string $errorCode,
        ?string $expectedDigest,
        ?string $orderDigest,
    ): array {
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'fulfillment_held',
            'order_id' => $orderId,
            'registration_id' => $registrationUuid,
            'account_id' => $accountId,
            'customer_id' => $customerId,
            'facade_id' => $facadeId,
            'product_code' => (string) ($protected[0]['product_code'] ?? ''),
            'protected_items' => $protected,
            'excluded_items' => $excluded,
            'entitlement_allowed' => false,
            'issuance' => 'held_until_email_verified',
            'hold_key' => self::opaqueToken('fh_'),
            'review_handle' => self::opaqueToken('hr_'),
            'mismatch_kind' => $mismatchKind,
            'error_code' => $errorCode,
            'replayed' => false,
            'existing' => false,
        ];
    }

    private function presentReleased(array $hold, array $release, bool $replayed, bool $existing): array
    {
        $held = json_decode((string) $hold['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        return [
            'schema' => self::RESULT_SCHEMA,
            'decision' => 'fulfillment_released',
            'order_id' => (int) ($held['order_id'] ?? $hold['order_id']),
            'registration_id' => (string) $hold['registration_uuid'],
            'account_id' => (string) $hold['account_uuid'],
            'customer_id' => (int) ($held['customer_id'] ?? 0),
            'facade_id' => (string) ($held['facade_id'] ?? ''),
            'product_code' => (string) ($held['product_code'] ?? ''),
            'hold_key' => (string) $hold['hold_key'],
            'review_handle' => (string) $hold['review_handle'],
            'mismatch_kind' => (string) $hold['mismatch_kind'],
            'resolved_identity_uuid' => (string) $release['resolved_identity_uuid'],
            'evidence_kind' => (string) $release['evidence_kind'],
            'evidence_handle' => (string) $release['evidence_handle'],
            'decided_by' => (string) $release['decided_by'],
            'entitlement_allowed' => true,
            'issuance' => 'deferred_to_verified_issuance_service',
            'replayed' => $replayed,
            'existing' => $existing,
        ];
    }

    /**
     * Journal one assessment. Exactly one row per (registration, order): a repeated
     * canonical request returns the stored decision (existing=true) instead of
     * recording a second one. Returns the decision to hand to the caller.
     */
    private function recordAssessment(
        string $registrationUuid,
        int $orderId,
        string $facadeId,
        array $decision,
        string $mismatchKind,
        string $holdState,
        ?string $errorCode,
        ?string $expectedDigest,
        ?string $orderDigest,
        string $requestId,
        string $idempotencyKey,
        string $digest,
    ): array {
        $now = $this->now();
        $holds = $this->schema->table('wpuiai_checkout_email_integrity_holds');
        $this->db->beginTransaction();
        try {
            $existing = $this->db->prepare("SELECT result_payload FROM {$holds}
                WHERE registration_uuid = :registration AND order_id = :order LIMIT 1");
            $existing->execute([':registration' => $registrationUuid, ':order' => $orderId]);
            $payload = $existing->fetchColumn();
            if (is_string($payload)) {
                $stored = json_decode($payload, true, 512, JSON_THROW_ON_ERROR);
                $stored['replayed'] = false;
                $stored['existing'] = true;
                $this->db->commit();
                return $stored;
            }

            $insert = $this->db->prepare("INSERT INTO {$holds}
                (hold_key, registration_uuid, order_id, account_uuid, customer_id, facade_id, product_code,
                 expected_email_lookup_digest, order_email_lookup_digest, mismatch_kind, hold_state,
                 review_handle, error_code, released_identity_uuid, released_at, result_payload,
                 request_id, idempotency_key, request_digest, created_at, retention_until, updated_at)
                VALUES (:key, :registration, :order, :account, :customer, :facade, :product,
                        :expected, :order_digest, :kind, :state, :review, :error, NULL, NULL, :payload,
                        :request, :idempotency, :digest, :created, :retention, :updated)");
            $decision['replayed'] = false;
            $decision['existing'] = false;
            $insert->execute([
                ':key' => (string) ($decision['hold_key'] ?? self::opaqueToken('fh_')),
                ':registration' => $registrationUuid,
                ':order' => $orderId,
                ':account' => $decision['account_id'] ?? null,
                ':customer' => $decision['customer_id'] ?? null,
                ':facade' => $facadeId,
                ':product' => $decision['product_code'] ?? null,
                ':expected' => $expectedDigest,
                ':order_digest' => $orderDigest,
                ':kind' => $mismatchKind,
                ':state' => $holdState,
                ':review' => $decision['review_handle'] ?? null,
                ':error' => $errorCode,
                ':payload' => json_encode($decision, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
                ':request' => $requestId,
                ':idempotency' => $idempotencyKey,
                ':digest' => $digest,
                ':created' => $now,
                ':retention' => self::plusSeconds($now, $this->retention),
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

    private function findHoldByKey(string $holdKey): ?array
    {
        $table = $this->schema->table('wpuiai_checkout_email_integrity_holds');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE hold_key = :key");
        $statement->execute([':key' => $holdKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function findReleaseByHoldKey(string $holdKey): ?array
    {
        $table = $this->schema->table('wpuiai_checkout_email_integrity_releases');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE hold_key = :key ORDER BY created_at DESC LIMIT 1");
        $statement->execute([':key' => $holdKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function replayDecision(string $operation, string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_checkout_email_integrity_holds');
        if ($operation === 'release_hold') {
            $table = $this->schema->table('wpuiai_checkout_email_integrity_releases');
        }
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        if ($operation === 'release_hold') {
            $hold = $this->findHoldByKey((string) $row['hold_key']);
            if ($hold === null) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            return $this->presentReleased($hold, $row, true, false);
        }
        $result = json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
        $result['replayed'] = true;
        $result['existing'] = false;
        return $result;
    }

    /** Resolve an EDD download to a bounded gate disposition (server-owned registry only). */
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
     * Verified registration binding: mailbox-verified, non-terminal, unexpired, and
     * promoted (authority account + EDD customer bound). Missing, malformed, and
     * unknown registrations fail closed with EMAIL_VERIFICATION_REQUIRED.
     */
    private function assertVerifiedRegistration(string $registrationUuid, string $facadeId, bool $requireAccount): array
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
        if ($requireAccount
            && ($registration['edd_customer_id'] === null || (int) $registration['edd_customer_id'] < 1
                || $registration['account_uuid'] === null)) {
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

    private function emailLookupDigest(string $email): string
    {
        return $this->registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($email));
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eCheckoutEmailIntegrityMigration::assertTimestamp($now);
        return $now;
    }

    private function requestDigest(array $value): string
    {
        return hash('sha256', FocusaSpec152eCheckoutEmailIntegrityMigration::encodeCanonical($value));
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

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
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
