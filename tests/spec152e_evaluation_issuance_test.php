<?php
// 152E.02.06 Evaluation issuance as verified EDD-backed entitlement (Spec 172 binding
// overlay). The legacy Evaluation journey issues verified_no_license limited access: an
// eligible verified account receives exactly ONE posture and ONE signed limited-access
// assertion, journaled with reason/limits; no EDD order and no EDD Software Licensing key
// is ever created (no zero-dollar fake license). Eligibility is evaluated from verified
// identity, EDD customer/order/license history, and device/refund state. Unverified
// input, active-paid downgrades, prior-evaluation duplicates, facade-switched repeats,
// caller-controlled EDD mappings, and unknown product codes all fail closed and create
// no entitlement.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';
require_once $root . '/docs/contracts/spec152e-evaluation-issuance.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_eval(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_eval_throws(callable $operation, string $code, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    try {
        $operation();
    } catch (Throwable $error) {
        if ($error->getMessage() !== $code) {
            fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n");
            exit(1);
        }
        return;
    }
    fwrite(STDERR, "FAIL: {$message}\n");
    exit(1);
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);
$evaluationMigration = new FocusaSpec152eEvaluationIssuanceMigration($db, 'wp_');
$evaluationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'evaluation_issuance_test']);

// Migration idempotency: re-running records one journal version each.
$evaluationMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'evaluation_issuance_replay']);
expect_eval(
    (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_evaluation_issuance_schema_migrations')->fetchColumn() === 1,
    'repeated evaluation issuance migration records one schema version',
);

$db->exec("CREATE TABLE wp_edd_customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NULL,
    email VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '',
    purchase_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    date_created VARCHAR(32) NOT NULL,
    stripe_customer_id VARCHAR(191) NULL
)");
$db->exec("CREATE TABLE wp_edd_customer_email_addresses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL,
    type VARCHAR(20) NOT NULL DEFAULT 'secondary',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active'
)");
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status VARCHAR(32) NOT NULL,
    customer_id BIGINT NOT NULL,
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL
)");
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1
)");

$insertLicense = $db->prepare("INSERT INTO wp_edd_licenses
    (license_key, customer_id, order_id, product_id, status)
    VALUES (:key, :customer, :order, :product, :status)");

$nowValue = '2026-08-08T00:01:00Z';
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};

$registrationSecrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);
$identitySecrets = new FocusaSpec152eEmailIdentitySecrets(
    str_repeat('e', 32),
    str_repeat('l', 64),
);

$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $registrationSecrets, $clock, attemptTtl: 86400, verificationTtl: 3600, pollTtl: 3600);
$identities = new FocusaSpec152eEmailIdentityRepository($db, $identityMigration, $identitySecrets, $clock);
$accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
$edd = new FocusaSpec152eEddCustomerAdapter($db, 'wp_', $clock);
$promotion = new FocusaSpec152eAccountPromotionService(
    $db,
    $promotionMigration,
    $registrations,
    $identities,
    $accounts,
    $edd,
    $identitySecrets,
    $registrationSecrets,
    $clock,
);
$postures = new FocusaSpec172VerifiedAccessPostureRepository($db, $postureMigration, $clock);
$assertions = new FocusaSpec172SignedAccessAssertionRepository($db, $assertionMigration, $postureMigration, $clock);
$service = new FocusaSpec152eEvaluationIssuanceService(
    $db,
    $evaluationMigration,
    $registrations,
    $accounts,
    $edd,
    $postureMigration,
    $postures,
    $assertions,
    $clock,
    'wp_',
);

$seq = 0;
$promoteVerified = static function (string $email, string $tag) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.evaluation.issuance.test',
        'install_channel' => 'cli',
        'product_code' => 'focusa_evaluation',
        'safe_redirect_handle' => 'success',
        'request_id' => 'req-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-' . $tag . '-' . $seq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $seq,
        'idem-verify-' . $tag . '-' . $seq,
    );
    $result = $promotion->promoteVerified([
        'registration_uuid' => $uuid,
        'verified_email' => $email,
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:01:00Z',
        'request_id' => 'req-promote-' . $tag . '-' . $seq,
        'idempotency_key' => 'idem-promote-' . $tag . '-' . $seq,
        'migration_provenance' => ['source' => 'evaluation_issuance_test', 'record' => $tag . '-' . $seq],
    ]);
    return [
        'account_uuid' => $result['account_uuid'],
        'identity_uuid' => $result['identity_uuid'],
        'registration_uuid' => $result['registration_id'],
        'verified_at' => $verified['registration']['verified_at'],
    ];
};

$signature = 'sig_spec152e_evaluation_' . str_repeat('a', 40);
$evalSeq = 0;
$evaluationInput = static function (array $verified, string $node, string $tag, string $productCode = 'focusa_evaluation') use (&$evalSeq, $signature): array {
    $evalSeq++;
    return [
        'product_code' => $productCode,
        'registration_uuid' => $verified['registration_uuid'],
        'account_uuid' => $verified['account_uuid'],
        'identity_uuid' => $verified['identity_uuid'],
        'verification_state' => 'account_promoted',
        'verified_at' => $verified['verified_at'],
        'node_uuid' => $node,
        'node_digest' => hash('sha256', 'node-' . $node),
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.evaluation.issuance.test',
        'install_channel' => 'cli',
        'request_id' => 'req-eval-' . $tag . '-' . $evalSeq,
        'idempotency_key' => 'idem-eval-' . $tag . '-' . $evalSeq,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $signature,
        'issued_at' => '2026-08-08T00:05:00Z',
        'refresh_at' => '2026-08-08T00:05:00Z',
        'migration_provenance' => ['source' => 'evaluation_issuance_test', 'record' => $tag . '-' . $evalSeq],
    ];
};

$counts = static function (string $table): int {
    global $db;
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
};

// ── 0. Canonical eligibility matrix and dedicated EDD product mapping ──

$matrix = FocusaSpec152eEvaluationEligibilityState::matrix();
expect_eval(count($matrix) === 7, 'eligibility matrix has exactly 7 rows');
$byCase = [];
foreach ($matrix as $row) {
    $byCase[(string) $row['case']] = $row;
}
expect_eval($byCase['verified_eligible']['decision'] === 'limited_access_issued', 'verified eligible row issues limited access');
expect_eval($byCase['terminal_history_only']['decision'] === 'limited_access_issued', 'terminal-history row issues limited access only');
expect_eval($byCase['unverified_email']['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'unverified row fails closed');
expect_eval($byCase['active_paid_customer']['decision'] === 'paid_posture_preserved', 'active-paid row preserves paid posture');
expect_eval($byCase['prior_evaluation_duplicate']['decision'] === 'evaluation_not_eligible', 'prior-evaluation row denies duplicates');
expect_eval($byCase['caller_mapping_control']['error'] === 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'caller mapping control denied');
expect_eval($byCase['unknown_product_code']['error'] === 'PRODUCT_MAPPING_REQUIRED', 'unknown product code denied');

$mapping = FocusaSpec152eEvaluationProductMapping::resolve(['product_code' => 'focusa_evaluation']);
expect_eval($mapping['resolved_product_scope'] === 'focusa', 'evaluation maps to the focusa product scope');
expect_eval($mapping['resolved_posture'] === 'verified_no_license', 'evaluation maps to the verified_no_license posture');
expect_eval($mapping['edd_download_id'] === null, 'no dedicated EDD download for evaluation');
expect_eval($mapping['edd_price_id'] === null, 'no dedicated EDD price for evaluation');
expect_eval($mapping['creates_edd_license_key'] === false, 'evaluation never creates an EDD Software Licensing key');
expect_eval($mapping['duration'] === 'no_automatic_expiry', 'evaluation resolves to the permanent no-automatic-expiry posture');
expect_eval($mapping['grant_source'] === 'authority_signed_limited_access_assertion', 'grant source is the authority-signed limited-access assertion');
$canonicalMapping = FocusaSpec152eEvaluationProductMapping::resolve(['product_code' => 'focusa']);
expect_eval($canonicalMapping['evaluation_product_code'] === 'focusa', 'canonical focusa code resolves through the evaluation mapping');

// ── 1. Eligible verified customer receives exactly one limited-access issuance ──

$alpha = $promoteVerified('eval.alpha@example.invalid', 'alpha');
$alphaInput = $evaluationInput($alpha, 'node-alpha-0001', 'alpha');
$alphaResult = $service->requestEvaluation($alphaInput);
expect_eval($alphaResult['decision'] === 'limited_access_issued', 'eligible verified customer receives limited access');
expect_eval($alphaResult['error_code'] === null, 'issued decision carries no error code');
expect_eval($alphaResult['evaluation_product_code'] === 'focusa_evaluation', 'result records the legacy evaluation product code');
expect_eval($alphaResult['product_scope'] === 'focusa', 'result carries the focusa product scope');
expect_eval($alphaResult['posture_uuid'] !== null, 'issued decision binds a posture');
expect_eval($alphaResult['assertion_uuid'] !== null, 'issued decision binds an assertion');
expect_eval($alphaResult['node_uuid'] === 'node-alpha-0001', 'issued decision binds the device node');
expect_eval($alphaResult['duration'] === 'no_automatic_expiry', 'issued decision is permanent, not timed');
expect_eval($alphaResult['edd_order_id'] === null, 'no EDD order is created');
expect_eval($alphaResult['edd_license_id'] === null, 'no EDD license is created');
expect_eval($alphaResult['creates_edd_license_key'] === false, 'no EDD key flag is false');
expect_eval($alphaResult['authority_sequence'] === 1, 'first evaluation issues authority sequence 1');
expect_eval($counts('wp_wpuiai_verified_access_postures') === 1, 'eligible customer creates exactly one posture');
expect_eval($counts('wp_wpuiai_signed_access_assertions') === 1, 'eligible customer creates exactly one assertion');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 1, 'exactly one evaluation issuance journal row');
expect_eval($counts('wp_wpuiai_verified_access_nodes') === 1, 'exactly one registered device node');
expect_eval($counts('wp_edd_licenses') === 0, 'no EDD Software Licensing key is created');

$postureRow = $db->query('SELECT * FROM wp_wpuiai_verified_access_postures')->fetch(PDO::FETCH_ASSOC);
expect_eval($postureRow['posture_uuid'] === $alphaResult['posture_uuid'], 'journaled posture matches the posture table');
$allowlist = json_decode($postureRow['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
expect_eval(
    $allowlist === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'posture stores the canonical explicit limited-mode allowlist',
);

// Idempotent replay: identical request returns the byte-identical result with no duplicates.
$alphaReplay = $service->requestEvaluation($alphaInput);
expect_eval($alphaReplay === $alphaResult, 'idempotent replay returns the identical result');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 1, 'replay creates no duplicate journal row');
expect_eval($counts('wp_wpuiai_verified_access_postures') === 1, 'replay creates no duplicate posture');
expect_eval($counts('wp_wpuiai_signed_access_assertions') === 1, 'replay creates no duplicate assertion');

// ── 2. Terminal refund/revoke history is preserved and never reactivated ──

$beta = $promoteVerified('eval.beta@example.invalid', 'beta');
$betaAccount = $accounts->findByUuid($beta['account_uuid']);
$betaCustomerId = (int) $betaAccount['edd_customer_id'];
$insertLicense->execute([
    ':key' => strtoupper('FOCUSA-REFUNDED-') . str_pad('1', 4, '0', STR_PAD_LEFT) . '-TESTKEY',
    ':customer' => $betaCustomerId,
    ':order' => null,
    ':product' => 453,
    ':status' => 'refunded',
]);
expect_eval($counts('wp_edd_licenses') === 1, 'terminal refunded history row seeded');
$betaResult = $service->requestEvaluation($evaluationInput($beta, 'node-beta-0001', 'beta'));
expect_eval($betaResult['decision'] === 'limited_access_issued', 'terminal-history-only customer receives limited access');
expect_eval($counts('wp_edd_licenses') === 1, 'refunded record is preserved and never reactivated');
$betaLicenseStatus = $db->query("SELECT status FROM wp_edd_licenses WHERE customer_id = {$betaCustomerId}")->fetchColumn();
expect_eval($betaLicenseStatus === 'refunded', 'refunded EDD license stays terminal (no reactivation)');
expect_eval($counts('wp_wpuiai_verified_access_postures') === 2, 'terminal-history customer creates its own posture');

// ── 3. Unverified and missing-proof requests fail closed ──────────────

$gamma = $registrations->createPending([
    'email' => 'eval.gamma@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'candidate.evaluation.issuance.test',
    'install_channel' => 'cli',
    'product_code' => 'focusa_evaluation',
    'safe_redirect_handle' => 'success',
    'request_id' => 'req-gamma-1',
    'idempotency_key' => 'idem-gamma-1',
]);
$gammaVerified = [
    'registration_uuid' => $gamma['registration']['registration_uuid'],
    'account_uuid' => '00000000-0000-4000-8000-000000000001',
    'identity_uuid' => '00000000-0000-4000-8000-000000000002',
    'verified_at' => '2026-08-08T00:01:00Z',
];
$gammaInput = $evaluationInput($gammaVerified, 'node-gamma-0001', 'gamma');
expect_eval_throws(
    static fn() => $service->requestEvaluation($gammaInput),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration is denied',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 2, 'unverified request creates no posture');
expect_eval($counts('wp_wpuiai_signed_access_assertions') === 2, 'unverified request creates no assertion');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 2, 'unverified request journals no decision');

$delta = $promoteVerified('eval.delta@example.invalid', 'delta');
$deltaInput = $evaluationInput($delta, 'node-delta-0001', 'delta');
$deltaInput['verification_state'] = 'email_verification_pending';
unset($deltaInput['verified_at']);
expect_eval_throws(
    static fn() => $service->requestEvaluation($deltaInput),
    'EMAIL_VERIFICATION_REQUIRED',
    'missing verification proof is denied',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 2, 'missing-proof request creates no posture');

// ── 4. Caller-controlled mapping and unknown product codes fail closed ──

$zetaLater = $promoteVerified('eval.zeta@example.invalid', 'zeta');
$mappingControl = $evaluationInput($zetaLater, 'node-zeta-0001', 'zeta');
$mappingControl['edd_download_id'] = 453;
expect_eval_throws(
    static fn() => $service->requestEvaluation($mappingControl),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied EDD download is denied',
);
$mappingControl2 = $evaluationInput($zetaLater, 'node-zeta-0001', 'zeta');
$mappingControl2['price'] = '0.00';
expect_eval_throws(
    static fn() => $service->requestEvaluation($mappingControl2),
    'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN',
    'caller-supplied price is denied',
);
$unknownProduct = $evaluationInput($zetaLater, 'node-zeta-0001', 'zeta', 'uiai_engine');
expect_eval_throws(
    static fn() => $service->requestEvaluation($unknownProduct),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown product code is denied',
);
$unknownProduct2 = $evaluationInput($zetaLater, 'node-zeta-0001', 'zeta', 'invented_product');
expect_eval_throws(
    static fn() => $service->requestEvaluation($unknownProduct2),
    'PRODUCT_MAPPING_REQUIRED',
    'invented product code is denied',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 2, 'denied mapping attempts create no posture');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 2, 'denied mapping attempts create no journal row');

// ── 5. Active paid customer: paid posture preserved, never downgraded ──

$epsilon = $promoteVerified('eval.epsilon@example.invalid', 'epsilon');
$epsilonAccount = $accounts->findByUuid($epsilon['account_uuid']);
$epsilonCustomerId = (int) $epsilonAccount['edd_customer_id'];
$insertLicense->execute([
    ':key' => strtoupper('FOCUSA-PAID-') . str_pad('1', 4, '0', STR_PAD_LEFT) . '-TESTKEY',
    ':customer' => $epsilonCustomerId,
    ':order' => null,
    ':product' => 453,
    ':status' => 'active',
]);
$epsilonBefore = $counts('wp_edd_licenses');
$epsilonInput = $evaluationInput($epsilon, 'node-epsilon-0001', 'epsilon');
expect_eval_throws(
    static fn() => $service->requestEvaluation($epsilonInput),
    'PAID_POSTURE_PRESERVED',
    'active paid customer is never downgraded through the evaluation path',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 2, 'paid customer gets no limited posture');
expect_eval($counts('wp_wpuiai_signed_access_assertions') === 2, 'paid customer gets no assertion');
expect_eval($counts('wp_edd_licenses') === $epsilonBefore, 'paid license row is untouched (no downgrade, no duplicate)');
$epsilonLicenseStatus = $db->query("SELECT status FROM wp_edd_licenses WHERE customer_id = {$epsilonCustomerId}")->fetchColumn();
expect_eval($epsilonLicenseStatus === 'active', 'paid license remains active and is not downgraded');
$paidJournal = $db->query("SELECT * FROM wp_wpuiai_evaluation_issuances WHERE decision = 'paid_posture_preserved'")->fetch(PDO::FETCH_ASSOC);
expect_eval($paidJournal !== false, 'paid-preserved decision is journaled for audit');
expect_eval($paidJournal['error_code'] === 'PAID_POSTURE_PRESERVED', 'paid-preserved journal records the stable error code');
expect_eval($paidJournal['edd_order_id'] === null && $paidJournal['edd_license_id'] === null, 'paid-preserved journal creates no EDD order or key');

// ── 6. Prior Evaluation: duplicates and facade switching fail closed ──

$zetaInput = $evaluationInput($zetaLater, 'node-zeta-0001', 'zeta');
$zetaResult = $service->requestEvaluation($zetaInput);
expect_eval($zetaResult['decision'] === 'limited_access_issued', 'verified zeta receives its one limited access');
expect_eval($counts('wp_wpuiai_verified_access_postures') === 3, 'zeta creates the third posture');

// Facade-switched repeat: same account, new request key, different node/facade.
$switched = $evaluationInput($zetaLater, 'node-zeta-0002', 'zeta-facade');
$switched['facade_id'] = 'forge.focusa.dev_v1';
$switched['presenter'] = 'forge.presenter';
expect_eval_throws(
    static fn() => $service->requestEvaluation($switched),
    'EVALUATION_NOT_ELIGIBLE',
    'facade-switched repeat is denied',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 3, 'facade-switched repeat creates no duplicate posture');
expect_eval($counts('wp_wpuiai_signed_access_assertions') === 3, 'facade-switched repeat creates no duplicate assertion');
$duplicateJournal = $db->query("SELECT * FROM wp_wpuiai_evaluation_issuances WHERE decision = 'evaluation_not_eligible'")->fetch(PDO::FETCH_ASSOC);
expect_eval($duplicateJournal !== false, 'duplicate evaluation is journaled for audit');
expect_eval($duplicateJournal['error_code'] === 'EVALUATION_NOT_ELIGIBLE', 'duplicate journal records the stable error code');

// ── 7. Idempotency conflicts and identity mismatches fail closed ──────

$theta = $promoteVerified('eval.theta@example.invalid', 'theta');
$conflictInput = $evaluationInput($theta, 'node-theta-0001', 'theta');
$conflictInput['idempotency_key'] = $zetaInput['idempotency_key'];
$conflictInput['node_uuid'] = 'node-theta-9999';
$conflictInput['node_digest'] = hash('sha256', 'node-theta-9999');
expect_eval_throws(
    static fn() => $service->requestEvaluation($conflictInput),
    'IDEMPOTENCY_CONFLICT',
    'same idempotency key with a different body is denied',
);

$iota = $promoteVerified('eval.iota@example.invalid', 'iota');
$mismatchInput = $evaluationInput($iota, 'node-iota-0001', 'iota');
$mismatchInput['account_uuid'] = $alpha['account_uuid'];
expect_eval_throws(
    static fn() => $service->requestEvaluation($mismatchInput),
    'ACCOUNT_EMAIL_MISMATCH',
    'registration/account mismatch is denied',
);

$kappa = $promoteVerified('eval.kappa@example.invalid', 'kappa');
$unknownAccountInput = $evaluationInput($kappa, 'node-kappa-0001', 'kappa');
$unknownAccountInput['account_uuid'] = '99999999-9999-4999-8999-999999999999';
expect_eval_throws(
    static fn() => $service->requestEvaluation($unknownAccountInput),
    'ENTITLEMENT_REQUIRED',
    'unknown authority account is denied before any creation',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 3, 'identity mismatch attempts create no posture');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 5, 'identity mismatch attempts create no journal row');

$lambda = $promoteVerified('eval.lambda@example.invalid', 'lambda');
$corruptedCustomerInput = $evaluationInput($lambda, 'node-lambda-0001', 'lambda');
$update = $db->prepare("UPDATE wp_wpuiai_activation_registrations SET edd_customer_id = 999999
    WHERE registration_uuid = :registration");
$update->execute([':registration' => $lambda['registration_uuid']]);
expect_eval_throws(
    static fn() => $service->requestEvaluation($corruptedCustomerInput),
    'EDD_CUSTOMER_RESOLUTION_FAILED',
    'registration/customer mismatch is denied',
);
expect_eval($counts('wp_wpuiai_verified_access_postures') === 3, 'customer mismatch creates no posture');

// ── 8. No EDD key, EDD-free schemas, redacted journals, preservation ──

expect_eval($counts('wp_edd_licenses') === 2, 'only the two seeded history licenses exist; the service created none');
$eddFree = $assertions->assertEddFree();
expect_eval($eddFree['edd_free'] === true, 'posture and assertion schemas remain EDD-free');
$journalRows = $db->query('SELECT * FROM wp_wpuiai_evaluation_issuances ORDER BY created_at')->fetchAll(PDO::FETCH_ASSOC);
expect_eval(count($journalRows) === 5, 'exactly five evaluation decisions journaled');
$decisions = array_count_values(array_column($journalRows, 'decision'));
expect_eval(($decisions['limited_access_issued'] ?? 0) === 3, 'three limited-access issuances journaled');
expect_eval(($decisions['paid_posture_preserved'] ?? 0) === 1, 'one paid-posture-preserved decision journaled');
expect_eval(($decisions['evaluation_not_eligible'] ?? 0) === 1, 'one duplicate-denial decision journaled');
foreach ($journalRows as $row) {
    expect_eval($row['edd_order_id'] === null, 'every journal row records no EDD order');
    expect_eval($row['edd_license_id'] === null, 'every journal row records no EDD license');
    expect_eval($row['duration'] === 'no_automatic_expiry', 'every journal row records the permanent duration');
    foreach ($row as $column => $value) {
        if (is_string($value)) {
            expect_eval(
                strpos($value, '@') === false && strpos($value, 'FOCUSA-') === false && strpos($value, 'cus_') === false,
                "journal is redacted (no raw email, key material, or customer secret in {$column})",
            );
        }
    }
}

$rollback = $evaluationMigration->preserveForRollback('2026-08-08T14:00:00Z', ['software_target' => 'prior_candidate', 'reason' => 'synthetic_rollback_proof']);
expect_eval($rollback['action'] === 'preserve', 'evaluation issuance rollback contract is preservation-only');
expect_eval($counts('wp_wpuiai_evaluation_issuances') === 5, 'rollback preserves every journal row');

$summary = [
    'schema' => 'focusa.spec152e.evaluation_issuance_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'limited_access_issued' => (int) ($decisions['limited_access_issued'] ?? 0),
    'paid_posture_preserved' => (int) ($decisions['paid_posture_preserved'] ?? 0),
    'evaluation_not_eligible' => (int) ($decisions['evaluation_not_eligible'] ?? 0),
    'postures' => $counts('wp_wpuiai_verified_access_postures'),
    'assertions' => $counts('wp_wpuiai_signed_access_assertions'),
    'nodes' => $counts('wp_wpuiai_verified_access_nodes'),
    'edd_licenses' => $counts('wp_edd_licenses'),
    'edd_orders_created' => 0,
    'edd_free' => true,
    'duration' => 'no_automatic_expiry',
    'matrix_rows' => count($matrix),
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
