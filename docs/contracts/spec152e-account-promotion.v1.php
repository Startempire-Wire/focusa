<?php
// Atomic verified-account promotion and merge. One idempotent transaction resolves or
// creates the authority account and EDD customer, links the verified email identity, the
// optional WordPress user, and prior evidence-backed orders/licenses, persists consent,
// and advances the registration to account_promoted. Conflicts enter review with no
// partial writes. Unverified, mismatched, or conflicting input never promotes.
declare(strict_types=1);

final class FocusaSpec152eAccountPromotionMigration
{
    public const SCHEMA = 'focusa.spec152e.account_promotion.v1';
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
        $idempotency = $this->table('wpuiai_account_promotion_idempotency');
        $links = $this->table('wpuiai_account_promotion_purchase_links');
        $migrations = $this->table('wpuiai_account_promotion_schema_migrations');
        $events = $this->table('wpuiai_account_promotion_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(64) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            account_uuid {$uuid} NOT NULL,
            identity_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            result_payload TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$links} (
            link_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            edd_order_id BIGINT NOT NULL,
            edd_order_item_id BIGINT NULL,
            edd_license_id BIGINT NOT NULL,
            evidence_digest VARCHAR(64) NOT NULL,
            linked_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL,
            UNIQUE (edd_order_id),
            UNIQUE (edd_license_id)
        )");
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

    /** Rollback is preservation-only: promotion idempotency and purchase-link journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_account_promotion_schema_events');
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

final class FocusaSpec152eAccountPromotionService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.account_promotion_result.v1';
    public const RETENTION_SECONDS = 2592000;
    private const PAID_ORDER_STATUSES = ['complete'];
    private const UNUSABLE_LICENSE_STATUSES = ['revoked', 'disabled'];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eAccountPromotionMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eEmailIdentityRepository $identities,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private FocusaSpec152eEmailIdentitySecrets $identitySecrets,
        private FocusaSpec152eActivationRegistrationSecrets $registrationSecrets,
        callable $clock,
        private int $retention = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if ($this->retention < 1) {
            throw new InvalidArgumentException('positive promotion retention is required');
        }
    }

    /**
     * Promote a mailbox-verified registration in one authority transaction.
     *
     * Required input:
     *   - registration_uuid:         verified registration UUID
     *   - verified_email:            exact email that was verified (must bind the registration)
     *   - verification_method:       'magic_link' | 'otp'
     *   - transactional_consent_at:  canonical UTC timestamp (recorded separately from promotional)
     *   - request_id / idempotency_key
     *   - migration_provenance:      evidence array
     *
     * Optional input:
     *   - promotional_consent_at:    canonical UTC timestamp or null
     *   - wordpress_user_id:         optional WordPress user to link without duplicates
     *   - stripe_customer_id:        optional Stripe customer reference
     *   - prior_purchases:           evidence-backed list of
     *                                ['order_id' => int, 'item_id' => int|null, 'license_id' => int]
     *
     * Returns a masked envelope: no raw email, no secrets, no authority credentials.
     * Replays with the same idempotency key return the same bounded result.
     */
    public function promoteVerified(array $input): array
    {
        $registrationUuid = $this->assertUuid((string) ($input['registration_uuid'] ?? ''), 'registration');
        $verifiedEmail = (string) ($input['verified_email'] ?? '');
        if ($verifiedEmail === '') {
            throw new InvalidArgumentException('verified email is required');
        }
        $normalized = FocusaSpec152eEmailNormalizer::exact($verifiedEmail);
        $verificationMethod = (string) ($input['verification_method'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_]{1,31}$/D', $verificationMethod) !== 1) {
            throw new InvalidArgumentException('verification method required');
        }
        $transactional = (string) ($input['transactional_consent_at'] ?? '');
        FocusaSpec152eAccountPromotionMigration::assertTimestamp($transactional);
        $promotional = $input['promotional_consent_at'] ?? null;
        FocusaSpec152eAccountPromotionMigration::assertTimestamp($promotional, true);
        $wpUserId = $this->optionalInt($input['wordpress_user_id'] ?? null, 'wordpress user');
        $stripeCustomerId = $this->optionalToken($input['stripe_customer_id'] ?? null, 191);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $priorPurchases = $this->validatePriorPurchases($input['prior_purchases'] ?? []);
        $registrationDigest = $this->registrationSecrets->emailLookupDigest($normalized);
        return $this->promoteWithLegacy($input, 'verified', null);
    }

    /**
     * Merge a legacy EDD paid record (key + order + license) into the verified account.
     * Identical to promoteVerified except that the legacy key must resolve to a usable
     * license whose owner has the verified email, the evidence must be strong (synthetic
     * records remain quarantined), and conflicting paid records enter review. Never
     * transfers ownership by raw email match; existing order/license state is preserved.
     */
    public function mergeLegacyVerified(array $input): array
    {
        $legacyKey = (string) ($input['legacy_key'] ?? '');
        if ($legacyKey === '' || strlen($legacyKey) > 191 || preg_match('/[\r\n\x00]/', $legacyKey)) {
            throw new InvalidArgumentException('bounded legacy EDD license key required');
        }
        $evidence = $input['legacy_evidence'] ?? [];
        if (!is_array($evidence) || $evidence === []) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $evidenceDigest = FocusaSpec152eLegacyActivationAdapter::validateLegacyEvidence($evidence);
        return $this->promoteWithLegacy($input, 'legacy_merge', [
            'key' => $legacyKey,
            'evidence' => $evidence,
            'evidence_digest' => $evidenceDigest,
        ]);
    }

    private function promoteWithLegacy(array $input, string $mode, ?array $legacy): array
    {
        $operation = $mode === 'legacy_merge' ? 'legacy_merge' : 'promote_verified';
        $registrationUuid = $this->assertUuid((string) ($input['registration_uuid'] ?? ''), 'registration');
        $verifiedEmail = (string) ($input['verified_email'] ?? '');
        if ($verifiedEmail === '') {
            throw new InvalidArgumentException('verified email is required');
        }
        $normalized = FocusaSpec152eEmailNormalizer::exact($verifiedEmail);
        $verificationMethod = (string) ($input['verification_method'] ?? '');
        if (preg_match('/^[a-z][a-z0-9_]{1,31}$/D', $verificationMethod) !== 1) {
            throw new InvalidArgumentException('verification method required');
        }
        $transactional = (string) ($input['transactional_consent_at'] ?? '');
        FocusaSpec152eAccountPromotionMigration::assertTimestamp($transactional);
        $promotional = $input['promotional_consent_at'] ?? null;
        FocusaSpec152eAccountPromotionMigration::assertTimestamp($promotional, true);
        $wpUserId = $this->optionalInt($input['wordpress_user_id'] ?? null, 'wordpress user');
        $stripeCustomerId = $this->optionalToken($input['stripe_customer_id'] ?? null, 191);
        $requestId = $this->assertRequestId((string) ($input['request_id'] ?? ''));
        $idempotencyKey = $this->assertIdempotencyKey((string) ($input['idempotency_key'] ?? ''));
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $priorPurchases = $this->validatePriorPurchases($input['prior_purchases'] ?? []);
        $registrationDigest = $this->registrationSecrets->emailLookupDigest($normalized);
        $digestInput = [
            'operation' => $operation,
            'registration_uuid' => $registrationUuid,
            'email_lookup_digest' => $registrationDigest,
            'verification_method' => $verificationMethod,
            'wordpress_user_id' => $wpUserId,
            'stripe_customer_id' => $stripeCustomerId,
            'transactional_consent_at' => $transactional,
            'promotional_consent_at' => $promotional,
            'prior_purchases' => $priorPurchases,
            'migration_provenance' => $provenance,
            'request_id' => $requestId,
        ];
        if ($legacy !== null) {
            $digestInput['legacy_key'] = $legacy['key'];
            $digestInput['legacy_evidence'] = $legacy['evidence'];
        }
        $digest = $this->digest($digestInput);

        return $this->transaction(function () use ($operation, $legacy, $registrationUuid, $normalized, $verificationMethod, $transactional, $promotional, $wpUserId, $stripeCustomerId, $requestId, $idempotencyKey, $provenance, $priorPurchases, $registrationDigest, $digest): array {
            return $this->promoteInTransaction([
                'operation' => $operation,
                'legacy' => $legacy,
                'registration_uuid' => $registrationUuid,
                'normalized' => $normalized,
                'verification_method' => $verificationMethod,
                'transactional_consent_at' => $transactional,
                'promotional_consent_at' => $promotional,
                'wordpress_user_id' => $wpUserId,
                'stripe_customer_id' => $stripeCustomerId,
                'request_id' => $requestId,
                'idempotency_key' => $idempotencyKey,
                'migration_provenance' => $provenance,
                'prior_purchases' => $priorPurchases,
                'registration_digest' => $registrationDigest,
                'digest' => $digest,
            ]);
        });
    }

    private function promoteInTransaction(array $p): array
    {
        $replay = $this->replay($p['idempotency_key'], $p['digest'], $p['operation']);
        if ($replay !== null) {
            $result = json_decode($replay['result_payload'], true, 512, JSON_THROW_ON_ERROR);
            $result['replayed'] = true;
            return $result;
        }

        $registration = $this->registrations->findByUuid($p['registration_uuid']);
        $now = $this->now();
        if ($registration['state'] !== FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED
            || $registration['verification_state'] !== 'mailbox_verified'
            || $registration['verified_at'] === null) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($registration['expires_at'] !== null && $now >= (string) $registration['expires_at']) {
            throw new DomainException('REGISTRATION_EXPIRED');
        }
        if (!hash_equals((string) $registration['email_lookup_digest'], $p['registration_digest'])) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }

        // Legacy merge gate: the key must resolve to a usable license whose owner has the
        // verified email; the evidence must point at the exact record; conflicting paid
        // records enter review. Raw email match alone never transfers ownership.
        $resolvedLegacy = null;
        if ($p['operation'] === 'legacy_merge') {
            $resolvedLegacy = $this->resolveLegacyMergeRecord($p['legacy']['key'], $p['normalized'], $p['prior_purchases']);
        }

        // 1. Resolve or create the EDD customer from the exact verified email.
        $customer = $this->edd->findCustomerByEmail($p['normalized']);
        if ($customer === null) {
            $customerId = $this->edd->createCustomerInTransaction($p['normalized'], $p['wordpress_user_id'], $p['stripe_customer_id'], $this->encodeCanonical($p['migration_provenance']), $now);
            $customerResolution = 'new';
        } else {
            $customerId = (int) $customer['id'];
            $customerResolution = 'existing';
        }

        // 2. Resolve or create the authority account for that customer.
        $resolved = $this->accounts->resolveForPromotionInTransaction($customerId, $p['wordpress_user_id'], $p['stripe_customer_id'], $this->encodeCanonical($p['migration_provenance']), (string) $registration['verified_at']);
        $account = $resolved['account'];
        $accountResolution = $resolved['resolution'];

        // 3. Link the optional WordPress and Stripe references without creating duplicates.
        if ($p['wordpress_user_id'] !== null) {
            $owner = $this->accounts->findByWordpressUserId($p['wordpress_user_id']);
            if ($owner !== null && !hash_equals((string) $owner['account_uuid'], (string) $account['account_uuid'])) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
        }
        if ($p['stripe_customer_id'] !== null) {
            $owner = $this->accounts->findByStripeCustomerId($p['stripe_customer_id']);
            if ($owner !== null && !hash_equals((string) $owner['account_uuid'], (string) $account['account_uuid'])) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
        }

        // 4. Resolve or create the verified email identity bound to the authority account.
        $existingIdentity = $this->identities->findExact($p['normalized']);
        if ($existingIdentity !== null) {
            if (!hash_equals((string) $existingIdentity['account_uuid'], (string) $account['account_uuid'])) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
            $identity = $this->identities->settleConsentAtPromotionInTransaction((string) $existingIdentity['identity_uuid'], $p['transactional_consent_at'], $p['promotional_consent_at'], $now);
            $identityUuid = (string) $identity['identity_uuid'];
            $identityState = (string) $identity['identity_state'];
        } else {
            $identityState = $this->identities->hasPrimaryForAccount((string) $account['account_uuid']) ? 'linked' : 'primary';
            $identity = $this->identities->storeVerifiedInTransaction($p['normalized'], [
                'verification_state' => 'mailbox_verified',
                'verified_at' => (string) $registration['verified_at'],
                'account_uuid' => (string) $account['account_uuid'],
                'identity_uuid' => self::uuid(),
                'identity_state' => $identityState,
                'verification_method' => $p['verification_method'],
                'transactional_consent_at' => $p['transactional_consent_at'],
                'promotional_consent_at' => $p['promotional_consent_at'],
                'promotional_consent_revoked_at' => null,
                'source' => 'account.promotion',
                'migration_evidence' => $p['migration_provenance'],
            ]);
            $identityUuid = (string) $identity['identity_uuid'];
        }

        // 5. Link prior evidence-backed EDD orders and licenses to the account.
        $linkedOrders = [];
        $linkedLicenses = [];
        foreach ($p['prior_purchases'] as $purchase) {
            $linkProvenance = $p['migration_provenance'];
            if ($p['legacy'] !== null) {
                $linkProvenance = array_merge($linkProvenance, [
                    'legacy_evidence' => $p['legacy']['evidence'],
                    'legacy_evidence_digest' => $p['legacy']['evidence_digest'],
                ]);
            }
            $link = $this->linkPriorPurchaseInTransaction($account, $customerId, $purchase, $linkProvenance, $now);
            $linkedOrders[] = (int) $link['edd_order_id'];
            if ($link['edd_license_id'] !== null) {
                $linkedLicenses[] = (int) $link['edd_license_id'];
            }
        }

        // 6. Advance the registration to account_promoted (idempotent, caller-owned transaction).
        $this->registrations->promoteVerifiedInTransaction($p['registration_uuid'], (string) $account['account_uuid'], $customerId, $p['request_id'], $p['idempotency_key']);

        $result = [
            'schema' => self::RESULT_SCHEMA,
            'registration_id' => $p['registration_uuid'],
            'account_uuid' => (string) $account['account_uuid'],
            'identity_uuid' => $identityUuid,
            'edd_customer_id' => $customerId,
            'customer_resolution' => $customerResolution,
            'account_resolution' => $accountResolution,
            'identity_state' => $identityState,
            'transactional_consent_at' => $p['transactional_consent_at'],
            'promotional_consent_at' => $p['promotional_consent_at'],
            'linked_orders' => $linkedOrders,
            'linked_licenses' => $linkedLicenses,
            'replayed' => false,
        ];
        if ($resolvedLegacy !== null) {
            $result['legacy_merge'] = true;
            $result['legacy_license_id'] = (int) $resolvedLegacy['license_id'];
            $result['legacy_order_id'] = (int) $resolvedLegacy['order_id'];
        }
        $this->recordIdempotency($p['idempotency_key'], $p['digest'], $result, $p['registration_uuid'], (string) $account['account_uuid'], $identityUuid, $customerId, $now, $p['operation']);
        return $result;
    }

    /**
     * Resolve the legacy key inside the promotion transaction and require stronger
     * evidence for conflicting paid records. Read-only over EDD truth: order/license
     * rows are never modified here.
     */
    private function resolveLegacyMergeRecord(string $legacyKey, string $normalized, array $priorPurchases): array
    {
        $license = $this->edd->findLicenseByKey($legacyKey);
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
        if (!$this->edd->customerHasEmail($customerId, $normalized)) {
            throw new DomainException('LICENSE_ACCOUNT_MISMATCH');
        }
        $orderId = $license['order_id'] !== null ? (int) $license['order_id'] : 0;
        if ($orderId < 1) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $order = $this->edd->findOrderById($orderId);
        if ($order === null || (int) $order['customer_id'] !== $customerId
            || !in_array((string) $order['status'], self::PAID_ORDER_STATUSES, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        // Stronger evidence for conflicting paid records: the submitted evidence must
        // point at the exact resolved order/license, and every claimed purchase must be
        // owned by the resolved legacy customer. Anything else enters review.
        $licenseId = (int) $license['id'];
        $expected = false;
        foreach ($priorPurchases as $purchase) {
            if ((int) $purchase['order_id'] === $orderId && (int) $purchase['license_id'] === $licenseId) {
                $expected = true;
            }
            $claimedOrder = $this->edd->findOrderById((int) $purchase['order_id']);
            $claimedLicense = $this->edd->findLicenseById((int) $purchase['license_id']);
            if ($claimedOrder === null || $claimedLicense === null
                || (int) $claimedOrder['customer_id'] !== $customerId
                || (int) $claimedLicense['customer_id'] !== $customerId) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
        }
        if (!$expected) {
            throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
        }
        return [
            'license_id' => $licenseId,
            'order_id' => $orderId,
            'customer_id' => $customerId,
        ];
    }

    // ── prior-purchase linkage ───────────────────────────────────────────

    private function linkPriorPurchaseInTransaction(array $account, int $customerId, array $purchase, array $provenance, string $now): array
    {
        $orderId = (int) $purchase['order_id'];
        $licenseId = (int) $purchase['license_id'];
        $itemId = $purchase['item_id'] ?? null;

        $order = $this->fetchOrder($orderId);
        if ($order === null || (int) $order['customer_id'] !== $customerId
            || !in_array((string) $order['status'], self::PAID_ORDER_STATUSES, true)) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if ($itemId !== null) {
            $item = $this->fetchOrderItem($itemId);
            if ($item === null || (int) $item['order_id'] !== $orderId) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
        }
        $license = $this->fetchLicense($licenseId);
        if ($license === null || (int) $license['customer_id'] !== $customerId
            || in_array((string) $license['status'], self::UNUSABLE_LICENSE_STATUSES, true)) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }

        $table = $this->schema->table('wpuiai_account_promotion_purchase_links');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE edd_order_id = :order OR edd_license_id = :license");
        $statement->execute([':order' => $orderId, ':license' => $licenseId]);
        $existing = $statement->fetch(PDO::FETCH_ASSOC);
        if ($existing !== false) {
            if (!hash_equals((string) $existing['account_uuid'], (string) $account['account_uuid'])) {
                throw new DomainException('ACCOUNT_MERGE_REVIEW_REQUIRED');
            }
            return $existing;
        }

        $evidenceDigest = $this->digest([
            'account_uuid' => $account['account_uuid'],
            'edd_customer_id' => $customerId,
            'edd_order_id' => $orderId,
            'edd_order_item_id' => $itemId,
            'edd_license_id' => $licenseId,
            'migration_provenance' => $provenance,
        ]);
        $statement = $this->db->prepare("INSERT INTO {$table}
            (link_uuid, account_uuid, edd_customer_id, edd_order_id, edd_order_item_id, edd_license_id,
             evidence_digest, linked_at, migration_provenance)
            VALUES (:link, :account, :customer, :order, :item, :license, :evidence, :linked, :provenance)");
        $statement->execute([
            ':link' => self::uuid(),
            ':account' => $account['account_uuid'],
            ':customer' => $customerId,
            ':order' => $orderId,
            ':item' => $itemId,
            ':license' => $licenseId,
            ':evidence' => $evidenceDigest,
            ':linked' => $now,
            ':provenance' => $this->encodeCanonical($provenance),
        ]);
        return [
            'account_uuid' => $account['account_uuid'],
            'edd_order_id' => $orderId,
            'edd_order_item_id' => $itemId,
            'edd_license_id' => $licenseId,
        ];
    }

    private function fetchOrder(int $orderId): ?array
    {
        $table = $this->edd->table('edd_orders');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $orderId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function fetchOrderItem(int $itemId): ?array
    {
        $table = $this->edd->table('edd_order_items');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $itemId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    private function fetchLicense(int $licenseId): ?array
    {
        $table = $this->edd->table('edd_licenses');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $licenseId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── validation helpers ───────────────────────────────────────────────

    private function validatePriorPurchases(mixed $value): array
    {
        if (!is_array($value)) {
            throw new InvalidArgumentException('bounded prior purchase evidence required');
        }
        $result = [];
        foreach ($value as $entry) {
            if (!is_array($entry)) {
                throw new InvalidArgumentException('bounded prior purchase evidence required');
            }
            $orderId = filter_var($entry['order_id'] ?? null, FILTER_VALIDATE_INT);
            $licenseId = filter_var($entry['license_id'] ?? null, FILTER_VALIDATE_INT);
            $itemId = $entry['item_id'] ?? null;
            if ($orderId === false || $orderId < 1 || $licenseId === false || $licenseId < 1) {
                throw new InvalidArgumentException('evidence-backed order and license references required');
            }
            if ($itemId !== null) {
                $itemId = filter_var($itemId, FILTER_VALIDATE_INT);
                if ($itemId === false || $itemId < 1) {
                    throw new InvalidArgumentException('bounded order item reference required');
                }
            }
            $result[] = ['order_id' => (int) $orderId, 'item_id' => $itemId === null ? null : (int) $itemId, 'license_id' => (int) $licenseId];
        }
        return $result;
    }

    private function optionalInt(mixed $value, string $kind): ?int
    {
        if ($value === null) {
            return null;
        }
        $parsed = filter_var($value, FILTER_VALIDATE_INT);
        if ($parsed === false || $parsed < 1) {
            throw new InvalidArgumentException('positive ' . $kind . ' ID required');
        }
        return (int) $parsed;
    }

    private function optionalToken(mixed $value, int $maxLength): ?string
    {
        if ($value === null) {
            return null;
        }
        $token = (string) $value;
        if ($token === '' || strlen($token) > $maxLength || preg_match('/[\r\n\x00]/', $token)) {
            throw new InvalidArgumentException('bounded optional token required');
        }
        return $token;
    }

    private function assertUuid(string $uuid, string $kind): string
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
        return $uuid;
    }

    private function assertRequestId(string $requestId): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $requestId) !== 1) {
            throw new InvalidArgumentException('bounded request ID required');
        }
        return $requestId;
    }

    private function assertIdempotencyKey(string $key): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
        return $key;
    }

    private function digest(array $value): string
    {
        return hash('sha256', $this->encodeCanonical($value));
    }

    private function encodeCanonical(array $value): string
    {
        return FocusaSpec152eAccountPromotionMigration::encodeCanonical($value);
    }

    private function now(): string
    {
        $now = ($this->clock)();
        FocusaSpec152eAccountPromotionMigration::assertTimestamp($now);
        return $now;
    }

    private function transaction(callable $callback): mixed
    {
        $this->db->beginTransaction();
        try {
            $result = $callback();
            $this->db->commit();
            return $result;
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    private function replay(string $key, string $digest, string $operation): ?array
    {
        $table = $this->schema->table('wpuiai_account_promotion_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $key]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($operation, (string) $row['operation'])
            || !hash_equals($digest, (string) $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $key, string $digest, array $result, string $registrationUuid, string $accountUuid, string $identityUuid, int $customerId, string $now, string $operation): void
    {
        $table = $this->schema->table('wpuiai_account_promotion_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, registration_uuid, account_uuid, identity_uuid,
             edd_customer_id, result_payload, created_at, retention_until)
            VALUES (:key, :operation, :digest, :registration, :account, :identity, :customer, :payload, :created, :retention)");
        $statement->execute([
            ':key' => $key,
            ':operation' => $operation,
            ':digest' => $digest,
            ':registration' => $registrationUuid,
            ':account' => $accountUuid,
            ':identity' => $identityUuid,
            ':customer' => $customerId,
            ':payload' => json_encode($result, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':created' => $now,
            ':retention' => $this->plusSeconds($now, $this->retention),
        ]);
    }

    private function plusSeconds(string $timestamp, int $seconds): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $seconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }
}
