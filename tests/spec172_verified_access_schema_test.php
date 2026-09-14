<?php
// 172.02.01 Verified-no-license authority-account posture and assertion schema.
// Verified promotion creates exactly one account posture; unverified input creates
// none; no assertion row can be interpreted as an EDD license (schema introspection
// guard). Posture and assertion rows bind account, verified identity, product scope,
// node, family allowlist, sequence, issue/refresh times, signer, and status. No EDD
// Software Licensing key and no zero-dollar fake license is ever created.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_posture(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_posture_throws(callable $operation, string $code, string $message): void
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'verified_access_schema_test']);

// Migration idempotency: re-running every migration records one journal version each.
$registrationMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'verified_access_schema_replay']);
$postureMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'verified_access_schema_replay']);
$assertionMigration->migrate('2026-08-08T00:01:00Z', ['source' => 'verified_access_schema_replay']);
expect_posture(
    (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_schema_migrations')->fetchColumn() === 1,
    'repeated posture migration records one schema version',
);
expect_posture(
    (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertion_schema_migrations')->fetchColumn() === 1,
    'repeated assertion migration records one schema version',
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
// EDD license truth table exists only to prove this atom never touches it.
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active'
)");

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

$seq = 0;
$promoteVerified = static function (string $email, string $tag) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.verified.access.schema.test',
        'install_channel' => 'cli',
        'product_code' => 'focusa',
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
        'migration_provenance' => ['source' => 'verified_access_schema_test', 'record' => $tag . '-' . $seq],
    ]);
    return [
        'account_uuid' => $result['account_uuid'],
        'identity_uuid' => $result['identity_uuid'],
        'registration_uuid' => $result['registration_id'],
        'verified_at' => $verified['registration']['verified_at'],
    ];
};

$postureInput = static function (array $verified, string $scope, string $node, string $tag): array {
    return [
        'account_uuid' => $verified['account_uuid'],
        'identity_uuid' => $verified['identity_uuid'],
        'registration_uuid' => $verified['registration_uuid'],
        'verification_state' => 'account_promoted',
        'verified_at' => $verified['verified_at'],
        'product_scope' => $scope,
        'node_uuid' => $node,
        'node_digest' => hash('sha256', 'node-' . $tag),
        'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor($scope),
        'signer' => 'wpuiai.spec172.issue.v1',
        'sequence' => 1,
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T00:02:00Z',
        'migration_provenance' => ['source' => 'verified_access_schema_test', 'record' => $tag],
    ];
};

$signature = 'sig_spec172_verified_access_' . str_repeat('a', 40);
$assertionInput = static function (array $posture, int $sequence) use ($signature): array {
    return [
        'posture_uuid' => $posture['posture_uuid'],
        'product_scope' => $posture['product_scope'],
        'node_uuid' => $posture['node_uuid'],
        'family_allowlist' => json_decode($posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR),
        'sequence' => $sequence,
        'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
        'signature' => $signature,
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T00:02:00Z',
        'signer' => 'wpuiai.spec172.issue.v1',
        'migration_provenance' => ['source' => 'verified_access_schema_test', 'record' => 'assertion-' . $sequence],
    ];
};

// ── 1. Verified promotion creates exactly one account posture ───────────

$verified = $promoteVerified('posture.alpha@example.invalid', 'alpha');
$input = $postureInput($verified, 'focusa', 'node-alpha-0001', 'alpha');
$posture = $postures->recordPosture($input);
expect_posture((int) $posture['sequence'] === 1, 'posture starts at authority sequence 1');
expect_posture($posture['status'] === 'issued', 'posture status is server-owned issued');
expect_posture($posture['status_reason'] === 'mailbox_verified', 'posture status reason records mailbox verification');
expect_posture($posture['product_scope'] === 'focusa', 'posture binds the focusa product scope');
expect_posture($posture['node_uuid'] === 'node-alpha-0001', 'posture binds the registered operator node');
expect_posture($posture['account_uuid'] === $verified['account_uuid'], 'posture binds the promoted authority account');
expect_posture($posture['identity_uuid'] === $verified['identity_uuid'], 'posture binds the verified identity');
expect_posture($posture['registration_uuid'] === $verified['registration_uuid'], 'posture binds the verified registration');
expect_posture($posture['signer'] === 'wpuiai.spec172.issue.v1', 'posture binds the server-owned signer');
$allowlist = json_decode($posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
expect_posture(
    $allowlist === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'posture stores the canonical explicit limited-mode allowlist',
);
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'verified promotion creates exactly one posture row');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_nodes')->fetchColumn() === 1, 'one registered operator node row');

$replayed = $postures->recordPosture($input);
expect_posture($replayed === $posture, 'identical posture record is idempotent');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'posture replay creates no duplicate');
expect_posture($postures->countForAccount($verified['account_uuid']) === 1, 'one account posture total');

// ── 2. Unverified input creates none ────────────────────────────────────

$unverified = $postureInput($verified, 'focusa', 'node-unverified-0001', 'unverified');
$unverified['verification_state'] = 'email_verification_pending';
$unverified['verified_at'] = null;
expect_posture_throws(
    static fn() => $postures->recordPosture($unverified),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified input is denied',
);
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'unverified input creates no posture');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_nodes')->fetchColumn() === 1, 'unverified input registers no node');

$blank = $postureInput($verified, 'focusa', 'node-blank-0001', 'blank');
unset($blank['verified_at'], $blank['verification_state']);
expect_posture_throws(
    static fn() => $postures->recordPosture($blank),
    'EMAIL_VERIFICATION_REQUIRED',
    'missing verification proof is denied',
);
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'missing verification proof creates no posture');

// ── 3. Caller-controlled product/family/scope fail closed ───────────────

$fakeProduct = $postureInput($verified, 'focusa', 'node-fake-0001', 'fake');
$fakeProduct['product_scope'] = 'focusa_evaluation';
expect_posture_throws(
    static fn() => $postures->recordPosture($fakeProduct),
    'PRODUCT_NOT_INCLUDED',
    'caller-controlled fake product code is denied',
);
$fakeProduct['product_scope'] = 'focusa';
$fakeProduct['family_allowlist'] = array_merge(FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), ['automation']);
expect_posture_throws(
    static fn() => $postures->recordPosture($fakeProduct),
    'CAPABILITY_FAMILY_NOT_INCLUDED',
    'blocked paid family claim is denied',
);
$fakeProduct['family_allowlist'] = ['unknown_family'];
expect_posture_throws(
    static fn() => $postures->recordPosture($fakeProduct),
    'CAPABILITY_FAMILY_NOT_INCLUDED',
    'unknown family claim is denied',
);
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'denied posture attempts create no rows');

// ── 4. No EDD key and no zero-dollar fake license ───────────────────────

expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === 0, 'no EDD Software Licensing key is created');
$eddFree = $assertions->assertEddFree();
expect_posture($eddFree['edd_free'] === true, 'assertion and posture schemas are EDD-free by introspection');

// ── 5. Signed assertion model from the posture ──────────────────────────

$assertion = $assertions->recordAssertion($assertionInput($posture, 1));
expect_posture($assertion['posture_uuid'] === $posture['posture_uuid'], 'assertion binds the posture');
expect_posture($assertion['account_uuid'] === $posture['account_uuid'], 'assertion binds the authority account from the posture');
expect_posture($assertion['identity_uuid'] === $posture['identity_uuid'], 'assertion binds the verified identity from the posture');
expect_posture($assertion['product_scope'] === 'focusa', 'assertion carries the server-owned product scope');
expect_posture($assertion['node_uuid'] === 'node-alpha-0001', 'assertion carries the posture node');
expect_posture((int) $assertion['sequence'] === 1, 'assertion carries monotonic sequence 1');
expect_posture($assertion['status'] === 'issued', 'assertion status is issued');
expect_posture($assertion['signature_algorithm'] === 'ed25519.spec172.v1', 'assertion model carries the server-owned signature algorithm');
expect_posture(preg_match('/^[0-9a-f]{64}$/D', (string) $assertion['content_digest']) === 1, 'assertion model carries a deterministic content digest');
expect_posture($assertion['previous_assertion_uuid'] === null, 'first assertion has no predecessor');

$assertionReplay = $assertions->recordAssertion($assertionInput($posture, 1));
expect_posture($assertionReplay === $assertion, 'assertion replay is idempotent');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === 1, 'assertion replay creates no duplicate row');

// Deterministic model digest: same canonical fields produce the same digest.
$digestA = FocusaSpec172SignedAccessAssertionRepository::modelDigest([
    'schema' => FocusaSpec172SignedAccessAssertionMigration::SCHEMA,
    'sequence' => 1,
    'family_allowlist' => ['manual_mission'],
]);
$digestB = FocusaSpec172SignedAccessAssertionRepository::modelDigest([
    'family_allowlist' => ['manual_mission'],
    'sequence' => 1,
    'schema' => FocusaSpec172SignedAccessAssertionMigration::SCHEMA,
]);
expect_posture($digestA === $digestB, 'signed assertion model digest is canonical and order-independent');

// ── 6. Tampered, wrong-node, wrong-product, paid-family, stale-sequence ──

$wrongProduct = $assertionInput($posture, 2);
$wrongProduct['product_scope'] = 'uiai_engine';
expect_posture_throws(
    static fn() => $assertions->recordAssertion($wrongProduct),
    'ENTITLEMENT_PRODUCT_MISMATCH',
    'product scope not owned by the posture is denied',
);

$wrongNode = $assertionInput($posture, 2);
$wrongNode['node_uuid'] = 'node-intruder-0001';
expect_posture_throws(
    static fn() => $assertions->recordAssertion($wrongNode),
    'NODE_LIMIT_REACHED',
    'node not bound to the posture is denied',
);

$paidFamily = $assertionInput($posture, 2);
$paidFamily['family_allowlist'] = array_merge(
    json_decode($posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR),
    ['release_proof'],
);
expect_posture_throws(
    static fn() => $assertions->recordAssertion($paidFamily),
    'CAPABILITY_FAMILY_NOT_INCLUDED',
    'paid family claim outside the limited allowlist is denied',
);

$stale = $assertionInput($posture, 0);
expect_posture_throws(
    static fn() => $assertions->recordAssertion($stale),
    'positive assertion sequence required',
    'non-positive sequence is denied',
);

// ── 7. Bounded-credential refresh without access expiry ─────────────────

$refreshed = $assertions->refreshAssertion([
    'posture_uuid' => $posture['posture_uuid'],
    'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
    'signature' => 'sig_spec172_refresh_' . str_repeat('b', 40),
    'refresh_at' => '2026-08-08T12:00:00Z',
    'idempotency_key' => 'idem-refresh-0001',
    'migration_provenance' => ['source' => 'verified_access_schema_test', 'record' => 'refresh-1'],
]);
expect_posture((int) $refreshed['sequence'] === 2, 'refresh rotates to the next monotonic sequence');
expect_posture($refreshed['status'] === 'refreshed', 'refresh marks the assertion refreshed');
expect_posture($refreshed['refresh_at'] === '2026-08-08T12:00:00Z', 'refresh binds a fresh bounded refresh window');
expect_posture((string) $refreshed['previous_assertion_uuid'] === (string) $assertion['assertion_uuid'], 'refresh links the previous assertion');
$postureAfterRefresh = $postures->findByUuid($posture['posture_uuid']);
expect_posture((int) $postureAfterRefresh['sequence'] === 2, 'posture sequence advances in lockstep with the assertion');
expect_posture((string) $postureAfterRefresh['status'] === 'issued', 'refresh imposes no access expiry on the permanent posture');

$refreshReplay = $assertions->refreshAssertion([
    'posture_uuid' => $posture['posture_uuid'],
    'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
    'signature' => 'sig_spec172_refresh_' . str_repeat('b', 40),
    'refresh_at' => '2026-08-08T12:00:00Z',
    'idempotency_key' => 'idem-refresh-0001',
    'migration_provenance' => ['source' => 'verified_access_schema_test', 'record' => 'refresh-1'],
]);
expect_posture($refreshReplay === $refreshed, 'refresh replay returns the same assertion');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === 2, 'refresh replay creates no duplicate');

$rollback = $assertionInput($posture, 1);
expect_posture($assertions->recordAssertion($rollback) === $assertion, 'post-refresh replay of the original assertion is idempotent');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === 2, 'post-refresh replay creates no duplicate row');

// Gap scenario: a stale non-existent lower sequence must fail closed.
$verifiedBeta = $promoteVerified('posture.beta@example.invalid', 'beta');
$betaPosture = $postures->recordPosture($postureInput($verifiedBeta, 'uiai_engine', 'node-beta-0001', 'beta'));
$assertions->recordAssertion($assertionInput($betaPosture, 1));
$assertions->recordAssertion($assertionInput($betaPosture, 5));
expect_posture_throws(
    static fn() => $assertions->recordAssertion($assertionInput($betaPosture, 3)),
    'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED',
    'stale non-existent lower sequence is denied',
);
$assertionRowsBeforeRevoke = (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn();

// ── 8. Revoke is preservation-only ──────────────────────────────────────

$revoked = $assertions->revokeAssertion($posture['posture_uuid'], 'lost_device', '2026-08-08T13:00:00Z', ['source' => 'verified_access_schema_test', 'record' => 'revoke-1']);
expect_posture($revoked['status'] === 'revoked', 'current assertion status becomes revoked');
$revokedPosture = $postures->findByUuid($posture['posture_uuid']);
expect_posture($revokedPosture['status'] === 'revoked', 'posture status becomes revoked');
expect_posture($revokedPosture['status_reason'] === 'lost_device', 'posture keeps the explicit revoke reason');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === $assertionRowsBeforeRevoke, 'revoke preserves every assertion row');
expect_posture((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 2, 'revoke preserves every posture row');

$revokeReplay = $assertions->revokeAssertion($posture['posture_uuid'], 'lost_device', '2026-08-08T13:00:00Z', ['source' => 'verified_access_schema_test', 'record' => 'revoke-1']);
expect_posture($revokeReplay['status'] === 'revoked', 'revoke replay is idempotent');

$postRevokeIssue = $assertionInput($posture, 3);
expect_posture_throws(
    static fn() => $assertions->recordAssertion($postRevokeIssue),
    'VERIFIED_LIMITED_ACCESS',
    'issuing against a revoked posture is denied',
);

// ── 9. Rollback contract is preservation-only ───────────────────────────

$postureBefore = $postures->findByUuid($posture['posture_uuid']);
$rollback = $postureMigration->preserveForRollback('2026-08-08T14:00:00Z', ['software_target' => 'prior_candidate', 'reason' => 'synthetic_rollback_proof']);
$assertionRollback = $assertionMigration->preserveForRollback('2026-08-08T14:00:00Z', ['software_target' => 'prior_candidate', 'reason' => 'synthetic_rollback_proof']);
expect_posture($rollback['action'] === 'preserve', 'posture rollback contract is preservation-only');
expect_posture($assertionRollback['action'] === 'preserve', 'assertion rollback contract is preservation-only');
expect_posture(
    $postures->findByUuid($posture['posture_uuid']) === $postureBefore,
    'rollback preserves posture, identity, node, sequence, and journals',
);

$summary = [
    'schema' => 'focusa.spec172.verified_access_schema_validation.v1',
    'postures' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn(),
    'assertions' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn(),
    'unverified_created' => 0,
    'edd_licenses' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn(),
    'edd_free' => true,
    'posture_bindings' => ['account', 'identity', 'registration', 'product_scope', 'node', 'family_allowlist', 'sequence', 'issued_at', 'refresh_at', 'signer', 'status'],
    'checks' => $positiveChecks + $negativeChecks,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
