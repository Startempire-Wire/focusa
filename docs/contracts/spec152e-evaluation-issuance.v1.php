<?php
// Spec 152E / Spec 172 binding overlay — Evaluation issuance as verified EDD-backed
// entitlement (atom 152E.02.06, focusa-vbcqu.20.13.20).
//
//   - The legacy "Evaluation" journey is a limited-access issuance path, NOT an EDD
//     commerce path. The binding Spec 172 overlay replaces expiring EDD-backed Evaluation
//     issuance with verified_no_license limited-access assertion issuance after mailbox
//     verification: no EDD order and no EDD Software Licensing key is ever created, no
//     zero-dollar fake license is ever created, and the resolved posture is the permanent
//     Spec 172 "no_automatic_expiry" limited posture (no countdown, no timed Evaluation).
//   - Dedicated EDD product/license mapping: `focusa_evaluation` has NO dedicated EDD
//     download, price, or Software Licensing mapping (the canonical registry assigns 0
//     downloads). A request that supplies EDD download/price/key, commercial fields,
//     grants, limits, or a duration fails closed with CLIENT_COMMERCIAL_FIELDS_FORBIDDEN;
//     an unknown product code fails closed with PRODUCT_MAPPING_REQUIRED. Callers never
//     select product, price, grant, limit, or right.
//   - Evaluation eligibility is authority-private and evaluated from verified identity,
//     EDD customer/order/license history, and device/refund state:
//       * unverified email/registration  -> EMAIL_VERIFICATION_REQUIRED, nothing created
//       * active paid EDD license        -> PAID_POSTURE_PRESERVED (paid posture preserved;
//                                           never downgraded to limited mode by this path)
//       * prior Evaluation (any node)    -> EVALUATION_NOT_ELIGIBLE (no duplicate; facade
//                                           switching resolves to the same account posture)
//       * refunded/revoked/expired rows  -> terminal history preserved and never
//                                           reactivated; only the limited posture may issue
//       * eligible verified account      -> exactly ONE verified_no_license posture and ONE
//                                           signed limited-access assertion, journaled with
//                                           reason and limits digest; no EDD order or key
//   - The issuance journal is bounded, idempotent, append-audited, preservation-only, and
//     redacted: no raw email, no EDD key material, no payment secret, and no unmasked
//     real-email evidence is accepted, stored, or returned.
//
// Requires docs/contracts/spec152e-activation-registration.v1.php,
// docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-edd-customer-adapter.v1.php,
// docs/contracts/spec172-verified-access-posture.v1.php, and
// docs/contracts/spec172-signed-access-assertion.v1.php to be loaded first.
declare(strict_types=1);

/**
 * Dedicated EDD product/license mapping for the Evaluation journey. The mapping is
 * server-owned and immutable: `focusa_evaluation` resolves to the verified_no_license
 * posture with NO dedicated EDD download, NO EDD price, and NO EDD Software Licensing key.
 * Callers cannot select the mapping, the product, a price, grants, limits, or a duration;
 * every such attempt fails closed and creates nothing.
 */
final class FocusaSpec152eEvaluationProductMapping
{
    public const SCHEMA = 'focusa.spec152e.evaluation_product_mapping.v1';
    public const VERSION = 1;

    /** Legacy presenter product code for the Evaluation journey (Spec 152E section 12). */
    public const EVALUATION_PRODUCT_CODE = 'focusa_evaluation';
    /** Canonical product scope the evaluation path resolves to. */
    public const CANONICAL_PRODUCT_CODE = 'focusa';
    /** Spec 172 posture kind: verified_no_license is an account posture, never a License Type. */
    public const RESOLVED_POSTURE = 'verified_no_license';
    /** Dedicated EDD product/license mapping: none (canonical registry assigns 0 downloads). */
    public const EDD_DOWNLOAD_ID = null;
    public const EDD_PRICE_ID = null;
    public const CREATES_EDD_LICENSE_KEY = false;
    /** Permanent Spec 172 limited posture: no countdown, no automatic expiry. */
    public const DURATION = 'no_automatic_expiry';
    public const GRANT_SOURCE = 'authority_signed_limited_access_assertion';

    public const FORBIDDEN_CALLER_FIELDS = [
        'edd_download_id', 'edd_price_id', 'price', 'amount', 'total', 'currency', 'tier',
        'license_type', 'license_type_ref', 'grants', 'features', 'limits', 'node_limit',
        'activation_limit', 'commercial_rights', 'evaluation_duration', 'product_name',
        'download_id', 'duration_days', 'product', 'products', 'node_limit_requested',
    ];

    /**
     * Resolve the server-owned evaluation product mapping from a presenter request.
     *
     * @throws DomainException CLIENT_COMMERCIAL_FIELDS_FORBIDDEN when the caller supplies
     *                         any EDD/commercial mapping field
     * @throws DomainException PRODUCT_MAPPING_REQUIRED for unknown product codes
     */
    public static function resolve(array $request): array
    {
        foreach (array_keys($request) as $field) {
            if (in_array($field, self::FORBIDDEN_CALLER_FIELDS, true)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
        $code = (string) ($request['product_code'] ?? '');
        if ($code !== self::EVALUATION_PRODUCT_CODE && $code !== self::CANONICAL_PRODUCT_CODE) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        return [
            'schema' => self::SCHEMA,
            'evaluation_product_code' => $code,
            'resolved_product_scope' => self::CANONICAL_PRODUCT_CODE,
            'resolved_posture' => self::RESOLVED_POSTURE,
            'edd_download_id' => self::EDD_DOWNLOAD_ID,
            'edd_price_id' => self::EDD_PRICE_ID,
            'creates_edd_license_key' => self::CREATES_EDD_LICENSE_KEY,
            'duration' => self::DURATION,
            'grant_source' => self::GRANT_SOURCE,
        ];
    }
}

/**
 * Canonical Evaluation eligibility matrix. Authority-private, server-owned, fail-closed.
 * The matrix is the single source of truth for the eligibility service decision and is
 * asserted verbatim by the acceptance test.
 */
final class FocusaSpec152eEvaluationEligibilityState
{
    public const SCHEMA = 'focusa.spec152e.evaluation_eligibility.v1';
    public const VERSION = 1;

    public const DECISION_LIMITED_ACCESS_ISSUED = 'limited_access_issued';
    public const DECISION_PAID_POSTURE_PRESERVED = 'paid_posture_preserved';
    public const DECISION_EVALUATION_NOT_ELIGIBLE = 'evaluation_not_eligible';
    public const DECISION_DENIED = 'denied';

    public const DECISIONS = [
        self::DECISION_LIMITED_ACCESS_ISSUED,
        self::DECISION_PAID_POSTURE_PRESERVED,
        self::DECISION_EVALUATION_NOT_ELIGIBLE,
        self::DECISION_DENIED,
    ];

    /** EDD Software Licensing statuses that are terminal and can never reactivate. */
    public const TERMINAL_LICENSE_STATUSES = ['expired', 'revoked', 'refunded', 'cancelled'];

    /** The eligibility matrix (rows asserted by tests/spec152e_evaluation_issuance_test.php). */
    public static function matrix(): array
    {
        return [
            ['case' => 'verified_eligible', 'verification' => 'verified', 'active_paid' => false, 'prior_evaluation' => false, 'terminal_history' => false, 'decision' => self::DECISION_LIMITED_ACCESS_ISSUED],
            ['case' => 'terminal_history_only', 'verification' => 'verified', 'active_paid' => false, 'prior_evaluation' => false, 'terminal_history' => true, 'decision' => self::DECISION_LIMITED_ACCESS_ISSUED, 'note' => 'terminal EDD records are preserved and never reactivated; only the limited posture issues'],
            ['case' => 'unverified_email', 'verification' => 'unverified', 'active_paid' => '-', 'prior_evaluation' => '-', 'decision' => self::DECISION_DENIED, 'error' => 'EMAIL_VERIFICATION_REQUIRED'],
            ['case' => 'active_paid_customer', 'verification' => 'verified', 'active_paid' => true, 'prior_evaluation' => '-', 'decision' => self::DECISION_PAID_POSTURE_PRESERVED, 'note' => 'paid posture preserved; no downgrade, no limited posture, no EDD key'],
            ['case' => 'prior_evaluation_duplicate', 'verification' => 'verified', 'active_paid' => false, 'prior_evaluation' => true, 'decision' => self::DECISION_EVALUATION_NOT_ELIGIBLE, 'note' => 'one Evaluation per verified account; no duplicate or facade-switched trial'],
            ['case' => 'caller_mapping_control', 'verification' => '-', 'active_paid' => '-', 'prior_evaluation' => '-', 'decision' => self::DECISION_DENIED, 'error' => 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN'],
            ['case' => 'unknown_product_code', 'verification' => '-', 'active_paid' => '-', 'prior_evaluation' => '-', 'decision' => self::DECISION_DENIED, 'error' => 'PRODUCT_MAPPING_REQUIRED'],
        ];
    }
}

/**
 * Evaluation issuance journal schema: bounded, idempotent, append-audited, and
 * preservation-only. The journal proves the Spec 172 overlay: every evaluation decision
 * records `edd_order_id = NULL` and `edd_license_id = NULL` — no EDD order and no EDD
 * Software Licensing key is ever created for an Evaluation.
 */
final class FocusaSpec152eEvaluationIssuanceMigration
{
    public const SCHEMA = 'focusa.spec152e.evaluation_issuance.v1';
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
        $issuances = $this->table('wpuiai_evaluation_issuances');
        $migrations = $this->table('wpuiai_evaluation_issuance_schema_migrations');
        $events = $this->table('wpuiai_evaluation_issuance_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $decisionList = "'" . implode("','", FocusaSpec152eEvaluationEligibilityState::DECISIONS) . "'";

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$issuances} (
            evaluation_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            identity_uuid {$uuid} NOT NULL,
            registration_uuid {$uuid} NOT NULL,
            edd_customer_id BIGINT NOT NULL,
            product_scope VARCHAR(32) NOT NULL,
            evaluation_product_code VARCHAR(64) NOT NULL,
            decision VARCHAR(32) NOT NULL CHECK (decision IN ({$decisionList})),
            error_code VARCHAR(64) NULL,
            posture_uuid {$uuid} NULL,
            assertion_uuid {$uuid} NULL,
            duration VARCHAR(32) NOT NULL,
            edd_order_id BIGINT NULL,
            edd_license_id BIGINT NULL,
            reason VARCHAR(191) NOT NULL,
            limits_digest VARCHAR(64) NOT NULL,
            node_uuid VARCHAR(64) NOT NULL,
            facade_id VARCHAR(96) NOT NULL,
            presenter VARCHAR(96) NOT NULL,
            install_channel VARCHAR(96) NOT NULL,
            authority_sequence BIGINT NOT NULL,
            issued_at VARCHAR(32) NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            result_payload TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            retention_until VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_evaluation_issuance_idempotency
            ON {$issuances} (idempotency_key)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_evaluation_issuance_account
            ON {$issuances} (account_uuid, issued_at)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_evaluation_issuance_posture
            ON {$issuances} (posture_uuid)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_evaluation_issuance_retention
            ON {$issuances} (retention_until)");
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

    /** Rollback is preservation-only: evaluation journals are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_evaluation_issuance_schema_events');
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
        if ($nullable && ($timestamp === null || $timestamp === '')) {
            return;
        }
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/', $timestamp) !== 1) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        ksort($value);
        return json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }
}

/**
 * Evaluation issuance service. Evaluates verified-account eligibility from
 * customer/order/license history and device/refund state, then issues exactly one
 * verified_no_license posture and one signed limited-access assertion per eligible
 * verified account. No EDD order, no EDD Software Licensing key, and no zero-dollar fake
 * license is ever created. Unverified, paid-downgrade, duplicate, facade-switched, and
 * unknown-product requests fail closed and create no entitlement.
 */
final class FocusaSpec152eEvaluationIssuanceService
{
    public const SCHEMA = 'focusa.spec152e.evaluation_issuance.v1';
    public const VERSION = 1;
    public const SIGNER = 'wpuiai.spec152e.evaluation.v1';
    public const SIGNATURE_ALGORITHM = FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM;
    public const INITIAL_SEQUENCE = 1;
    public const RETENTION_SECONDS = 7776000;

    private const UNVERIFIED_REGISTRATION_STATES = [
        FocusaSpec152eActivationRegistrationState::ATTEMPT_CREATED,
        FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
    ];

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eEvaluationIssuanceMigration $schema,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eAuthorityAccountRepository $accounts,
        private FocusaSpec152eEddCustomerAdapter $edd,
        private FocusaSpec172VerifiedAccessPostureMigration $postureSchema,
        private FocusaSpec172VerifiedAccessPostureRepository $postures,
        private FocusaSpec172SignedAccessAssertionRepository $assertions,
        callable $clock,
        private string $eddPrefix = 'wp_',
        private int $retentionSeconds = self::RETENTION_SECONDS,
    ) {
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $this->eddPrefix) !== 1) {
            throw new InvalidArgumentException('invalid EDD table prefix');
        }
        if ($this->retentionSeconds < 1) {
            throw new InvalidArgumentException('positive retention required');
        }
    }

    /**
     * Evaluate one Evaluation request and issue limited access when eligible.
     *
     * Required input:
     *   - product_code: 'focusa_evaluation' (legacy) | 'focusa' (canonical)
     *   - registration_uuid, account_uuid, identity_uuid (canonical opaque UUIDs)
     *   - verification_state: 'mailbox_verified' | 'account_promoted'
     *   - verified_at: canonical UTC timestamp
     *   - node_uuid, node_digest (device binding)
     *   - facade_id, presenter, install_channel
     *   - request_id, idempotency_key
     *   - signature_algorithm, signature (bounded opaque server-signed envelope)
     *   - issued_at, refresh_at: canonical UTC timestamps
     *   - migration_provenance: evidence array
     *
     * Returns a public-safe decision (opaque references only). Replays with the same
     * idempotency key return the identical result; a different request body on the same
     * key throws IDEMPOTENCY_CONFLICT. Denied policy decisions throw their stable code
     * and journal an audit row.
     *
     * @throws DomainException EMAIL_VERIFICATION_REQUIRED, PAID_POSTURE_PRESERVED,
     *                         EVALUATION_NOT_ELIGIBLE, EDD_CUSTOMER_RESOLUTION_FAILED,
     *                         ENTITLEMENT_REQUIRED, ACCOUNT_EMAIL_MISMATCH,
     *                         IDEMPOTENCY_CONFLICT, CLIENT_COMMERCIAL_FIELDS_FORBIDDEN,
     *                         PRODUCT_MAPPING_REQUIRED
     */
    public function requestEvaluation(array $input): array
    {
        $mapping = FocusaSpec152eEvaluationProductMapping::resolve($input);

        $registrationUuid = $this->assertUuid((string) ($input['registration_uuid'] ?? ''), 'registration');
        $accountUuid = $this->assertUuid((string) ($input['account_uuid'] ?? ''), 'account');
        $identityUuid = $this->assertUuid((string) ($input['identity_uuid'] ?? ''), 'identity');
        $verificationState = (string) ($input['verification_state'] ?? '');
        if (!in_array($verificationState, ['mailbox_verified', 'account_promoted'], true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $verifiedAt = (string) ($input['verified_at'] ?? '');
        self::assertTimestamp($verifiedAt);
        $nodeUuid = $this->assertNodeUuid((string) ($input['node_uuid'] ?? ''));
        $nodeDigest = $this->assertNodeDigest((string) ($input['node_digest'] ?? ''));
        $facadeId = $this->assertToken((string) ($input['facade_id'] ?? ''), 96, 'facade');
        $presenter = $this->assertToken((string) ($input['presenter'] ?? ''), 96, 'presenter');
        $installChannel = $this->assertToken((string) ($input['install_channel'] ?? ''), 96, 'install channel');
        $requestId = $this->assertToken((string) ($input['request_id'] ?? ''), 191, 'request id');
        $idempotencyKey = $this->assertToken((string) ($input['idempotency_key'] ?? ''), 191, 'idempotency key');
        if ((string) ($input['signature_algorithm'] ?? '') !== self::SIGNATURE_ALGORITHM) {
            throw new InvalidArgumentException('server-owned signature algorithm required');
        }
        $signature = (string) ($input['signature'] ?? '');
        if ($signature === '' || strlen($signature) > 512 || preg_match('/[\r\n\x00]/', $signature)) {
            throw new InvalidArgumentException('bounded opaque signature required');
        }
        $issuedAt = (string) ($input['issued_at'] ?? '');
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        self::assertTimestamp($issuedAt);
        self::assertTimestamp($refreshAt);
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encodedProvenance = $this->schema->encodeCanonical($provenance);

        $digest = $this->digest([
            'operation' => 'evaluation_issuance',
            'registration_uuid' => $registrationUuid,
            'account_uuid' => $accountUuid,
            'identity_uuid' => $identityUuid,
            'verification_state' => $verificationState,
            'verified_at' => $verifiedAt,
            'product_code' => $mapping['evaluation_product_code'],
            'node_uuid' => $nodeUuid,
            'facade_id' => $facadeId,
            'presenter' => $presenter,
            'install_channel' => $installChannel,
            'issued_at' => $issuedAt,
            'migration_provenance' => json_decode($encodedProvenance, true, 512, JSON_THROW_ON_ERROR),
        ]);
        $replay = $this->replayDecision($idempotencyKey, $digest);
        if ($replay !== null) {
            return $replay;
        }

        // Verified registration is mandatory: no unverified-email promotion.
        $registration = $this->registrations->findByUuid($registrationUuid);
        $registrationState = (string) ($registration['state'] ?? '');
        $registrationVerification = (string) ($registration['verification_state'] ?? '');
        if ($registrationVerification !== 'mailbox_verified'
            || !is_string($registration['verified_at'] ?? null) || $registration['verified_at'] === ''
            || in_array($registrationState, self::UNVERIFIED_REGISTRATION_STATES, true)
            || FocusaSpec152eActivationRegistrationState::isTerminal($registrationState)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }

        try {
            $account = $this->accounts->findByUuid($accountUuid);
        } catch (OutOfBoundsException) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        if ($account === null || (int) ($account['edd_customer_id'] ?? 0) < 1) {
            throw new DomainException('ENTITLEMENT_REQUIRED');
        }
        if (!hash_equals((string) ($registration['account_uuid'] ?? ''), $accountUuid)) {
            throw new DomainException('ACCOUNT_EMAIL_MISMATCH');
        }
        $customerId = (int) $account['edd_customer_id'];
        if ((int) ($registration['edd_customer_id'] ?? 0) !== $customerId) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }
        $customer = $this->edd->findCustomerById($customerId);
        if ($customer === null) {
            throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
        }

        // Customer/order/license history and device/refund state consultation.
        $licenseState = $this->licenseStateSummary($customerId);
        $orderCount = $this->orderCount($customerId);
        $terminalCount = (int) ($licenseState['terminal'] ?? 0);

        $now = ($this->clock)();
        self::assertTimestamp($now);

        // Active paid posture is preserved: a paid customer is never downgraded to the
        // limited Evaluation posture through this path.
        if ((int) ($licenseState['active'] ?? 0) > 0) {
            return $this->journalDecision([
                'mapping' => $mapping,
                'registrationUuid' => $registrationUuid,
                'accountUuid' => $accountUuid,
                'identityUuid' => $identityUuid,
                'customerId' => $customerId,
                'decision' => FocusaSpec152eEvaluationEligibilityState::DECISION_PAID_POSTURE_PRESERVED,
                'errorCode' => 'PAID_POSTURE_PRESERVED',
                'nodeUuid' => $nodeUuid,
                'facadeId' => $facadeId,
                'presenter' => $presenter,
                'installChannel' => $installChannel,
                'authoritySequence' => (int) ($account['highest_entitlement_sequence'] ?? 0),
                'issuedAt' => $issuedAt,
                'now' => $now,
                'requestId' => $requestId,
                'idempotencyKey' => $idempotencyKey,
                'digest' => $digest,
                'reason' => sprintf('paid posture preserved; active_licenses=%d terminal_licenses=%d orders=%d', (int) $licenseState['active'], $terminalCount, $orderCount),
                'postureUuid' => null,
                'assertionUuid' => null,
                'provenance' => $encodedProvenance,
            ], true);
        }

        // Prior Evaluation: one limited posture per verified account. Facade switching
        // and repeat requests resolve to the same account posture and never duplicate.
        $existingPosture = $this->existingFocusaPosture($accountUuid);
        if ($existingPosture !== null) {
            return $this->journalDecision([
                'mapping' => $mapping,
                'registrationUuid' => $registrationUuid,
                'accountUuid' => $accountUuid,
                'identityUuid' => $identityUuid,
                'customerId' => $customerId,
                'decision' => FocusaSpec152eEvaluationEligibilityState::DECISION_EVALUATION_NOT_ELIGIBLE,
                'errorCode' => 'EVALUATION_NOT_ELIGIBLE',
                'nodeUuid' => $nodeUuid,
                'facadeId' => $facadeId,
                'presenter' => $presenter,
                'installChannel' => $installChannel,
                'authoritySequence' => (int) ($account['highest_entitlement_sequence'] ?? 0),
                'issuedAt' => $issuedAt,
                'now' => $now,
                'requestId' => $requestId,
                'idempotencyKey' => $idempotencyKey,
                'digest' => $digest,
                'reason' => sprintf('prior evaluation exists; active_licenses=%d terminal_licenses=%d orders=%d', (int) $licenseState['active'], $terminalCount, $orderCount),
                'postureUuid' => null,
                'assertionUuid' => null,
                'provenance' => $encodedProvenance,
            ], true);
        }

        // Eligible: issue exactly one verified_no_license posture and one signed
        // limited-access assertion. Terminal (refunded/revoked/expired/cancelled) EDD
        // rows are preserved and never reactivated; no EDD order and no EDD key is created.
        $posture = $this->postures->recordPosture([
            'account_uuid' => $accountUuid,
            'identity_uuid' => $identityUuid,
            'registration_uuid' => $registrationUuid,
            'verification_state' => $verificationState,
            'verified_at' => $verifiedAt,
            'product_scope' => $mapping['resolved_product_scope'],
            'node_uuid' => $nodeUuid,
            'node_digest' => $nodeDigest,
            'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor($mapping['resolved_product_scope']),
            'signer' => self::SIGNER,
            'sequence' => self::INITIAL_SEQUENCE,
            'issued_at' => $issuedAt,
            'refresh_at' => $refreshAt,
            'migration_provenance' => ['source' => 'evaluation_issuance', 'record' => $idempotencyKey],
        ]);
        $assertion = $this->assertions->recordAssertion([
            'posture_uuid' => $posture['posture_uuid'],
            'product_scope' => $mapping['resolved_product_scope'],
            'node_uuid' => $nodeUuid,
            'family_allowlist' => json_decode($posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR),
            'sequence' => self::INITIAL_SEQUENCE,
            'signature_algorithm' => self::SIGNATURE_ALGORITHM,
            'signature' => $signature,
            'issued_at' => $issuedAt,
            'refresh_at' => $refreshAt,
            'signer' => self::SIGNER,
            'migration_provenance' => ['source' => 'evaluation_issuance', 'record' => $idempotencyKey],
        ]);
        $limitsDigest = $this->digest(json_decode($posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR));

        return $this->journalDecision([
            'mapping' => $mapping,
            'registrationUuid' => $registrationUuid,
            'accountUuid' => $accountUuid,
            'identityUuid' => $identityUuid,
            'customerId' => $customerId,
            'decision' => FocusaSpec152eEvaluationEligibilityState::DECISION_LIMITED_ACCESS_ISSUED,
            'errorCode' => null,
            'nodeUuid' => $nodeUuid,
            'facadeId' => $facadeId,
            'presenter' => $presenter,
            'installChannel' => $installChannel,
            'authoritySequence' => (int) $posture['sequence'],
            'issuedAt' => $issuedAt,
            'now' => $now,
            'requestId' => $requestId,
            'idempotencyKey' => $idempotencyKey,
            'digest' => $digest,
            'reason' => sprintf('limited access issued; active_licenses=%d terminal_licenses=%d orders=%d', (int) $licenseState['active'], $terminalCount, $orderCount),
            'postureUuid' => (string) $posture['posture_uuid'],
            'assertionUuid' => (string) $assertion['assertion_uuid'],
            'provenance' => $encodedProvenance,
            'limitsDigest' => $limitsDigest,
        ], false);
    }

    // ── Journaling ───────────────────────────────────────────────────

    /**
     * Persist the decision audit row and return the public-safe result. Denied policy
     * decisions throw their stable code AFTER the audit row is committed; issued
     * decisions return normally.
     */
    private function journalDecision(array $v, bool $denied): array
    {
        $mapping = $v['mapping'];
        $limitsDigest = $v['limitsDigest'] ?? null;
        if ($limitsDigest === null) {
            $allowlist = FocusaSpec172VerifiedAccessPostureState::allowlistFor($mapping['resolved_product_scope']);
            $limitsDigest = $this->digest($allowlist);
        }
        $result = [
            'schema' => self::SCHEMA,
            'decision' => $v['decision'],
            'error_code' => $v['errorCode'],
            'account_uuid' => $v['accountUuid'],
            'identity_uuid' => $v['identityUuid'],
            'registration_uuid' => $v['registrationUuid'],
            'edd_customer_id' => $v['customerId'],
            'evaluation_product_code' => $mapping['evaluation_product_code'],
            'product_scope' => $mapping['resolved_product_scope'],
            'posture_uuid' => $v['postureUuid'],
            'assertion_uuid' => $v['assertionUuid'],
            'node_uuid' => $v['nodeUuid'],
            'duration' => $mapping['duration'],
            'edd_order_id' => null,
            'edd_license_id' => null,
            'creates_edd_license_key' => $mapping['creates_edd_license_key'],
            'grant_source' => $mapping['grant_source'],
            'limits_digest' => $limitsDigest,
            'authority_sequence' => $v['authoritySequence'],
            'issued_at' => $v['issuedAt'],
            'request_id' => $v['requestId'],
            'idempotency_key' => $v['idempotencyKey'],
        ];
        $payload = json_encode($result, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);

        $table = $this->schema->table('wpuiai_evaluation_issuances');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (evaluation_uuid, account_uuid, identity_uuid, registration_uuid, edd_customer_id,
             product_scope, evaluation_product_code, decision, error_code, posture_uuid, assertion_uuid,
             duration, edd_order_id, edd_license_id, reason, limits_digest, node_uuid, facade_id,
             presenter, install_channel, authority_sequence, issued_at, request_id, idempotency_key,
             request_digest, result_payload, created_at, retention_until)
            VALUES (:uuid, :account, :identity, :registration, :customer, :product, :eval_code,
                    :decision, :error_code, :posture, :assertion, :duration, :edd_order, :edd_license,
                    :reason, :limits, :node, :facade, :presenter, :channel, :sequence, :issued,
                    :request, :idempotency, :digest, :payload, :created, :retention)");
        $statement->execute([
            ':uuid' => self::uuid(),
            ':account' => $v['accountUuid'],
            ':identity' => $v['identityUuid'],
            ':registration' => $v['registrationUuid'],
            ':customer' => $v['customerId'],
            ':product' => $mapping['resolved_product_scope'],
            ':eval_code' => $mapping['evaluation_product_code'],
            ':decision' => $v['decision'],
            ':error_code' => $v['errorCode'],
            ':posture' => $v['postureUuid'],
            ':assertion' => $v['assertionUuid'],
            ':duration' => $mapping['duration'],
            ':edd_order' => null,
            ':edd_license' => null,
            ':reason' => mb_substr($v['reason'], 0, 191),
            ':limits' => $limitsDigest,
            ':node' => $v['nodeUuid'],
            ':facade' => $v['facadeId'],
            ':presenter' => $v['presenter'],
            ':channel' => $v['installChannel'],
            ':sequence' => $v['authoritySequence'],
            ':issued' => $v['issuedAt'],
            ':request' => $v['requestId'],
            ':idempotency' => $v['idempotencyKey'],
            ':digest' => $v['digest'],
            ':payload' => $payload,
            ':created' => $v['now'],
            ':retention' => $this->retentionUntil($v['now']),
        ]);

        if ($denied) {
            throw new DomainException((string) $v['errorCode']);
        }
        return $result;
    }

    // ── History consultation ─────────────────────────────────────────

    /** License history summary: active (paid-preserving) vs terminal rows for the customer. */
    private function licenseStateSummary(int $customerId): array
    {
        $table = $this->eddPrefix . 'edd_licenses';
        $statement = $this->db->prepare("SELECT status, COUNT(*) AS c FROM {$table} WHERE customer_id = :customer GROUP BY status");
        $statement->execute([':customer' => $customerId]);
        $active = 0;
        $terminal = 0;
        foreach ($statement->fetchAll(PDO::FETCH_ASSOC) as $row) {
            $status = strtolower((string) ($row['status'] ?? ''));
            if (in_array($status, FocusaSpec152eEvaluationEligibilityState::TERMINAL_LICENSE_STATUSES, true)) {
                $terminal += (int) $row['c'];
            } else {
                $active += (int) $row['c'];
            }
        }
        return ['active' => $active, 'terminal' => $terminal];
    }

    /** Order history count for the customer (bounded consultation only). */
    private function orderCount(int $customerId): int
    {
        $table = $this->eddPrefix . 'edd_orders';
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE customer_id = :customer");
        $statement->execute([':customer' => $customerId]);
        return (int) $statement->fetchColumn();
    }

    /** Device/refund state: any existing focusa-scope limited posture for the account. */
    private function existingFocusaPosture(string $accountUuid): ?array
    {
        $table = $this->postureSchema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT * FROM {$table}
            WHERE account_uuid = :account AND product_scope = :product ORDER BY created_at DESC LIMIT 1");
        $statement->execute([
            ':account' => $accountUuid,
            ':product' => FocusaSpec152eEvaluationProductMapping::CANONICAL_PRODUCT_CODE,
        ]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    // ── Idempotency ──────────────────────────────────────────────────

    private function replayDecision(string $idempotencyKey, string $digest): ?array
    {
        $table = $this->schema->table('wpuiai_evaluation_issuances');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key ORDER BY created_at DESC LIMIT 1");
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals((string) $row['request_digest'], $digest)) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode((string) $row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    // ── Validation helpers ───────────────────────────────────────────

    public static function assertTimestamp(?string $timestamp): void
    {
        FocusaSpec152eEvaluationIssuanceMigration::assertTimestamp($timestamp);
    }

    private function assertUuid(string $uuid, string $kind): string
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
        return $uuid;
    }

    private function assertNodeUuid(string $nodeUuid): string
    {
        if ($nodeUuid === '' || strlen($nodeUuid) > 64 || preg_match('/[\r\n\x00]/', $nodeUuid)) {
            throw new InvalidArgumentException('bounded node UUID required');
        }
        return $nodeUuid;
    }

    private function assertNodeDigest(string $nodeDigest): string
    {
        if (preg_match('/^[a-f0-9]{64}$/D', $nodeDigest) !== 1) {
            throw new InvalidArgumentException('canonical node digest required');
        }
        return $nodeDigest;
    }

    private function assertToken(string $value, int $maxLength, string $kind): string
    {
        if ($value === '' || strlen($value) > $maxLength || preg_match('/[\r\n\x00]/', $value)) {
            throw new InvalidArgumentException("bounded {$kind} required");
        }
        return $value;
    }

    private function digest(array $value): string
    {
        return hash('sha256', json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE));
    }

    private function retentionUntil(string $now): string
    {
        $timestamp = DateTimeImmutable::createFromFormat('Y-m-d\TH:i:s\Z', $now, new DateTimeZone('UTC'));
        if ($timestamp === false) {
            $timestamp = DateTimeImmutable::createFromFormat('Y-m-d\TH:i:s.u\Z', $now, new DateTimeZone('UTC'));
        }
        if ($timestamp === false) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
        return $timestamp->modify('+' . $this->retentionSeconds . ' seconds')->format('Y-m-d\TH:i:s\Z');
    }

    public static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        $hex = bin2hex($bytes);
        return sprintf('%s-%s-%s-%s-%s',
            substr($hex, 0, 8), substr($hex, 8, 4), substr($hex, 12, 4),
            substr($hex, 16, 4), substr($hex, 20, 12));
    }
}
