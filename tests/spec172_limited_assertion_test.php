<?php
// 172.02.02 Issue, refresh, revoke, and recover limited-access assertions.
// The WPUIAI service signs real Ed25519 assertions ONLY from active verified
// postures; refresh rotates the bounded credential without access expiry; revoke
// is preservation-only; recover re-issues from re-verified identity at a higher
// monotonic sequence. The Focusa client/store verifies presented claims and
// persists only valid assertions. Unverified, tampered, stale-sequence,
// wrong-node, unknown-family, and paid-family claims fail closed. The cross-
// language fixture is regenerated deterministically and must stay byte-identical;
// the Python vector test verifies the same signatures with its own Ed25519.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec172-verified-access-posture.v1.php';
require_once $root . '/docs/contracts/spec172-signed-access-assertion.v1.php';
require_once $root . '/docs/contracts/spec172-limited-access-assertion-service.v1.php';
require_once $root . '/docs/contracts/spec172-focusa-authority-client.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_limited(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_limited_throws(callable $operation, string $code, string $message): void
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

function fixture_encode(array $value): string
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
    return json_encode($normalize($value), JSON_THROW_ON_ERROR | JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES);
}

// ── Fixed synthetic fixture inputs (non-production) ────────────────────

const SPEC172_FIXTURE_PATH = __DIR__ . '/../docs/contracts/spec172-limited-access-assertion-vectors.v1.json';
const SPEC172_FIXTURE_SEED = '4242424242424242424242424242424242424242424242424242424242424242';

const P_ALPHA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0201';
const A_ALPHA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0202';
const I_ALPHA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0203';
const P_BETA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0801';
const A_BETA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0802';
const I_BETA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0803';
const P_GAMMA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0601';
const A_GAMMA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0602';
const I_GAMMA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0603';
const P_DELTA = '018f47c2-6ac0-7b16-8d1a-4e93df5a0701';
const P_UNKNOWN = '018f47c2-6ac0-7b16-8d1a-4e93df5a0000';

/**
 * Deterministically build the cross-language fixture: fixed synthetic postures,
 * canonical registries, and signed vectors. The signature over each presented
 * payload is a real Ed25519 signature (fixed seed), so Python verifies the same
 * bytes with its own Ed25519 verifier.
 */
function build_spec172_fixture(FocusaSpec172LimitedAssertionSigner $signer): array
{
    $focusaLimited = FocusaSpec172VerifiedAccessPostureState::FOCUSA_LIMITED_FAMILIES;
    $uiaiLimited = FocusaSpec172VerifiedAccessPostureState::UIAI_LIMITED_FAMILIES;
    $permanent = FocusaSpec172VerifiedAccessPostureState::PERMANENT_FAMILIES;
    $focusaAllowlist = FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa');
    $uiaiAllowlist = FocusaSpec172VerifiedAccessPostureState::allowlistFor('uiai_engine');

    $alpha = [
        'posture_uuid' => P_ALPHA,
        'account_uuid' => A_ALPHA,
        'identity_uuid' => I_ALPHA,
        'product_scope' => 'focusa',
        'node_uuid' => 'node-alpha-0001',
        'family_allowlist' => $focusaAllowlist,
        'sequence' => 2,
        'status' => 'issued',
        'status_reason' => 'mailbox_verified',
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T12:00:00Z',
        'expiry' => 'none',
    ];
    $beta = [
        'posture_uuid' => P_BETA,
        'account_uuid' => A_BETA,
        'identity_uuid' => I_BETA,
        'product_scope' => 'uiai_engine',
        'node_uuid' => 'node-beta-0001',
        'family_allowlist' => $uiaiAllowlist,
        'sequence' => 1,
        'status' => 'issued',
        'status_reason' => 'mailbox_verified',
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T12:00:00Z',
        'expiry' => 'none',
    ];
    $gamma = [
        'posture_uuid' => P_GAMMA,
        'account_uuid' => A_GAMMA,
        'identity_uuid' => I_GAMMA,
        'product_scope' => 'focusa',
        'node_uuid' => 'node-gamma-0001',
        'family_allowlist' => $focusaAllowlist,
        'sequence' => 1,
        'status' => 'revoked',
        'status_reason' => 'lost_device',
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T00:02:00Z',
        'expiry' => 'none',
    ];
    $delta = [
        'posture_uuid' => P_DELTA,
        'account_uuid' => A_GAMMA,
        'identity_uuid' => I_GAMMA,
        'product_scope' => 'focusa',
        'node_uuid' => 'node-delta-0001',
        'family_allowlist' => $focusaAllowlist,
        'sequence' => 3,
        'status' => 'issued',
        'status_reason' => 'recovered_verified_identity',
        'issued_at' => '2026-08-10T00:00:00Z',
        'refresh_at' => '2026-08-10T00:00:00Z',
        'expiry' => 'none',
    ];

    // Signed presented claim helper: signs the canonical payload with the fixed seed.
    $presented = static function (array $fields, string $signerLabel) use ($signer): array {
        $fields['signer'] = $signerLabel;
        $payload = FocusaSpec172LimitedAssertionPayload::build($fields);
        $presented = $payload;
        $presented['signature'] = $signer->sign($payload);
        return $presented;
    };

    $roundtripFields = [
        'posture_uuid' => P_ALPHA,
        'account_uuid' => A_ALPHA,
        'identity_uuid' => I_ALPHA,
        'product_scope' => 'focusa',
        'node_uuid' => 'node-alpha-0001',
        'family_allowlist' => $focusaAllowlist,
        'sequence' => 2,
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T12:00:00Z',
    ];
    $roundtrip = $presented($roundtripFields, FocusaSpec172LimitedAssertionService::SIGNER_ISSUE);

    $tamperedSignature = $roundtrip;
    $tamperedSignature['signature'] = ($tamperedSignature['signature'][0] === '0' ? '1' : '0') . substr($tamperedSignature['signature'], 1);

    $tamperedPayload = $roundtrip;
    $tamperedPayload['family_allowlist'] = array_values(array_unique(array_merge($focusaAllowlist, ['release_proof'])));

    $vectors = [
        ['id' => 'issue_valid_roundtrip', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'valid', 'presented' => $roundtrip],
        ['id' => 'tampered_signature', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'SIGNATURE_INVALID', 'presented' => $tamperedSignature],
        ['id' => 'tampered_payload_widened_family', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'SIGNATURE_INVALID', 'presented' => $tamperedPayload],
        ['id' => 'refresh_rotates_no_expiry_no_widening', 'at' => '2026-08-09T12:00:00Z', 'expected' => 'valid', 'presented' => $presented(array_merge($roundtripFields, ['refresh_at' => '2026-08-09T12:00:00Z']), FocusaSpec172LimitedAssertionService::SIGNER_REFRESH)],
        ['id' => 'stale_sequence_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', 'presented' => $presented(array_merge($roundtripFields, ['sequence' => 1, 'refresh_at' => '2026-08-08T00:02:00Z']), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'wrong_node_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'NODE_LIMIT_REACHED', 'presented' => $presented(array_merge($roundtripFields, ['node_uuid' => 'node-intruder-0001']), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'unknown_family_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'CAPABILITY_FAMILY_NOT_INCLUDED', 'presented' => $presented(array_merge($roundtripFields, ['family_allowlist' => ['unknown_family']]), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'paid_family_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'CAPABILITY_FAMILY_NOT_INCLUDED', 'presented' => $presented(array_merge($roundtripFields, ['family_allowlist' => array_values(array_unique(array_merge($focusaAllowlist, ['release_proof'])))]), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'wrong_product_scope_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'ENTITLEMENT_PRODUCT_MISMATCH', 'presented' => $presented(array_merge($roundtripFields, ['product_scope' => 'uiai_engine']), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'wrong_account_binding_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'ASSERTION_TAMPERED', 'presented' => $presented(array_merge($roundtripFields, ['account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0999']), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'revoked_assertion_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'VERIFIED_LIMITED_ACCESS', 'presented' => $presented([
            'posture_uuid' => P_GAMMA, 'account_uuid' => A_GAMMA, 'identity_uuid' => I_GAMMA,
            'product_scope' => 'focusa', 'node_uuid' => 'node-gamma-0001',
            'family_allowlist' => $focusaAllowlist, 'sequence' => 1,
            'issued_at' => '2026-08-08T00:02:00Z', 'refresh_at' => '2026-08-08T00:02:00Z',
        ], FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'unverified_account_rejected', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'EMAIL_VERIFICATION_REQUIRED', 'presented' => $presented(array_merge($roundtripFields, ['posture_uuid' => P_UNKNOWN, 'sequence' => 1, 'refresh_at' => '2026-08-08T00:02:00Z']), FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'recover_reissues_replacement', 'at' => '2026-08-10T00:00:00Z', 'expected' => 'valid', 'presented' => $presented([
            'posture_uuid' => P_DELTA, 'account_uuid' => A_GAMMA, 'identity_uuid' => I_GAMMA,
            'product_scope' => 'focusa', 'node_uuid' => 'node-delta-0001',
            'family_allowlist' => $focusaAllowlist, 'sequence' => 3,
            'issued_at' => '2026-08-10T00:00:00Z', 'refresh_at' => '2026-08-10T00:00:00Z',
        ], FocusaSpec172LimitedAssertionService::SIGNER_RECOVERY)],
        ['id' => 'uiai_roundtrip_valid', 'at' => '2026-08-08T12:00:00Z', 'expected' => 'valid', 'presented' => $presented([
            'posture_uuid' => P_BETA, 'account_uuid' => A_BETA, 'identity_uuid' => I_BETA,
            'product_scope' => 'uiai_engine', 'node_uuid' => 'node-beta-0001',
            'family_allowlist' => $uiaiAllowlist, 'sequence' => 1,
            'issued_at' => '2026-08-08T00:02:00Z', 'refresh_at' => '2026-08-08T12:00:00Z',
        ], FocusaSpec172LimitedAssertionService::SIGNER_ISSUE)],
        ['id' => 'stale_credential_window_refresh_required', 'at' => '2026-08-20T00:00:00Z', 'expected' => 'CREDENTIAL_REFRESH_REQUIRED', 'presented' => $roundtrip],
    ];

    return [
        'schema' => 'focusa.spec172.limited_access_assertion_vectors.v1',
        'fixture_kind' => 'public_synthetic_nonproduction',
        'algorithm' => FocusaSpec172LimitedAssertionSigner::ALGORITHM,
        'payload_schema' => FocusaSpec172LimitedAssertionPayload::SCHEMA,
        'seed_hex' => SPEC172_FIXTURE_SEED,
        'public_key_hex' => $signer->publicKeyHex(),
        'signer_issue' => FocusaSpec172LimitedAssertionService::SIGNER_ISSUE,
        'signer_refresh' => FocusaSpec172LimitedAssertionService::SIGNER_REFRESH,
        'signer_recovery' => FocusaSpec172LimitedAssertionService::SIGNER_RECOVERY,
        'registries' => [
            'focusa' => ['limited' => $focusaLimited, 'permanent' => $permanent],
            'uiai_engine' => ['limited' => $uiaiLimited, 'permanent' => $permanent],
        ],
        'postures' => [$alpha, $beta, $gamma, $delta],
        'vectors' => $vectors,
    ];
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);
$postureMigration = new FocusaSpec172VerifiedAccessPostureMigration($db, 'wp_');
$postureMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);
$assertionMigration = new FocusaSpec172SignedAccessAssertionMigration($db, 'wp_');
$assertionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'limited_assertion_test']);

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
$signer = FocusaSpec172LimitedAssertionSigner::fromSeed(SPEC172_FIXTURE_SEED);
$service = new FocusaSpec172LimitedAssertionService($db, $postures, $assertions, $signer, $postureMigration, $clock);
$routes = new FocusaSpec172LimitedAssertionRoutes($service);
$client = new FocusaSpec172LimitedAssertionClientStore($db, $signer);

// ── Fixture determinism (byte-identical regeneration) ──────────────────

$fixture = build_spec172_fixture($signer);
$expectedFixture = fixture_encode($fixture) . "\n";
$committedFixture = file_exists(SPEC172_FIXTURE_PATH) ? file_get_contents(SPEC172_FIXTURE_PATH) : null;
if (getenv('SPEC172_WRITE_FIXTURE') === '1') {
    file_put_contents(SPEC172_FIXTURE_PATH, $expectedFixture);
    fwrite(STDOUT, "fixture written to " . SPEC172_FIXTURE_PATH . "\n");
    exit(0);
}
expect_limited($committedFixture === $expectedFixture, 'cross-language fixture regenerates byte-identically');
expect_limited(
    $fixture['registries']['focusa']['limited'] === FocusaSpec172VerifiedAccessPostureState::FOCUSA_LIMITED_FAMILIES
    && $fixture['registries']['uiai_engine']['limited'] === FocusaSpec172VerifiedAccessPostureState::UIAI_LIMITED_FAMILIES
    && $fixture['registries']['focusa']['permanent'] === FocusaSpec172VerifiedAccessPostureState::PERMANENT_FAMILIES,
    'fixture registries mirror the server-owned canonical limited-mode registries',
);

// ── 1. Issue only after verified account and node binding ──────────────

$seq = 0;
$promoteVerified = static function (string $email, string $tag) use ($db, $registrations, $promotion, &$seq): array {
    $seq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'candidate.limited.assertion.test',
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
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => $tag . '-' . $seq],
    ]);
    return [
        'account_uuid' => $result['account_uuid'],
        'identity_uuid' => $result['identity_uuid'],
        'registration_uuid' => $result['registration_id'],
        'verified_at' => $verified['registration']['verified_at'],
    ];
};

$verifiedAlpha = $promoteVerified('limited.alpha@example.invalid', 'alpha');
$alphaPosture = $postures->recordPosture([
    'account_uuid' => $verifiedAlpha['account_uuid'],
    'identity_uuid' => $verifiedAlpha['identity_uuid'],
    'registration_uuid' => $verifiedAlpha['registration_uuid'],
    'verification_state' => 'account_promoted',
    'verified_at' => $verifiedAlpha['verified_at'],
    'product_scope' => 'focusa',
    'node_uuid' => 'node-alpha-0001',
    'node_digest' => hash('sha256', 'node-alpha'),
    'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'),
    'signer' => 'wpuiai.spec172.issue.v1',
    'sequence' => 1,
    'issued_at' => '2026-08-08T00:02:00Z',
    'refresh_at' => '2026-08-08T00:02:00Z',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-posture'],
]);

$issuedAlpha = $service->issue([
    'posture_uuid' => $alphaPosture['posture_uuid'],
    'issued_at' => '2026-08-08T00:02:00Z',
    'refresh_at' => '2026-08-08T00:02:00Z',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-issue-1'],
]);
expect_limited($issuedAlpha['verdict'] === 'valid', 'issue returns a valid verdict');
expect_limited((int) $issuedAlpha['sequence'] === 1, 'first issue binds server-owned monotonic sequence 1');
expect_limited($issuedAlpha['product_scope'] === 'focusa', 'issue carries the posture product scope');
expect_limited($issuedAlpha['node_uuid'] === 'node-alpha-0001', 'issue carries the registered node binding');
expect_limited($issuedAlpha['signer'] === FocusaSpec172LimitedAssertionService::SIGNER_ISSUE, 'issue uses the server-owned issue signer');
expect_limited(preg_match('/^[0-9a-f]{128}$/D', (string) $issuedAlpha['signature']) === 1, 'issue returns a real 64-byte Ed25519 signature');
expect_limited($signer->verify(
    FocusaSpec172LimitedAssertionPayload::build([
        'posture_uuid' => $issuedAlpha['posture_uuid'],
        'account_uuid' => $verifiedAlpha['account_uuid'],
        'identity_uuid' => $verifiedAlpha['identity_uuid'],
        'product_scope' => $issuedAlpha['product_scope'],
        'node_uuid' => $issuedAlpha['node_uuid'],
        'family_allowlist' => $issuedAlpha['family_allowlist'],
        'sequence' => (int) $issuedAlpha['sequence'],
        'issued_at' => $issuedAlpha['issued_at'],
        'refresh_at' => $issuedAlpha['refresh_at'],
        'signer' => $issuedAlpha['signer'],
    ]),
    (string) $issuedAlpha['signature'],
), 'issued signature verifies under the server public key');

// ── 1b. Verifier: valid round-trip, tampered, wrong-node, wrong-product ──

$presentAlpha = static function (array $row, int $sequence, string $accountUuid) use ($verifiedAlpha): array {
    return [
        'posture_uuid' => $row['posture_uuid'],
        'account_uuid' => $accountUuid,
        'identity_uuid' => $verifiedAlpha['identity_uuid'],
        'product_scope' => $row['product_scope'],
        'node_uuid' => $row['node_uuid'],
        'family_allowlist' => is_array($row['family_allowlist'])
            ? $row['family_allowlist']
            : json_decode((string) $row['family_allowlist'], true, 512, JSON_THROW_ON_ERROR),
        'sequence' => $sequence,
        'issued_at' => $row['issued_at'],
        'refresh_at' => $row['refresh_at'],
        'signer' => $row['signer'],
        'signature' => $row['signature'],
    ];
};

$verify1 = $service->verify($presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']), '2026-08-08T00:02:00Z');
expect_limited($verify1['verdict'] === 'valid', 'issued assertion round-trips through the verifier');

// Tampered signature and tampered payload fail closed.
$tamperedSig = $issuedAlpha;
$tamperedSig['signature'] = ($tamperedSig['signature'][0] === '0' ? '1' : '0') . substr($tamperedSig['signature'], 1);
$verifyTampered = $service->verify($presentAlpha($tamperedSig, 1, $verifiedAlpha['account_uuid']), '2026-08-08T00:02:00Z');
expect_limited($verifyTampered['verdict'] === 'denied' && $verifyTampered['code'] === 'SIGNATURE_INVALID', 'tampered signature fails closed');

$tamperedPayload = $presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']);
$tamperedPayload['family_allowlist'] = array_values(array_unique(array_merge($tamperedPayload['family_allowlist'], ['release_proof'])));
$verifyTamperedPayload = $service->verify($tamperedPayload, '2026-08-08T00:02:00Z');
expect_limited($verifyTamperedPayload['verdict'] === 'denied' && $verifyTamperedPayload['code'] === 'SIGNATURE_INVALID', 'tampered payload fails closed');

$wrongNode = $presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']);
$wrongNode['node_uuid'] = 'node-intruder-0001';
$verifyWrongNode = $service->verify($wrongNode, '2026-08-08T00:02:00Z');
expect_limited($verifyWrongNode['verdict'] === 'denied' && $verifyWrongNode['code'] === 'SIGNATURE_INVALID', 'wrong-node claim (unsigned for that payload) fails closed');

// A validly re-signed wrong-node claim is caught by the stored-row binding check.
$wrongNodeSigned = $presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']);
$wrongNodeSigned['node_uuid'] = 'node-intruder-0001';
$wrongNodeSigned['signature'] = $signer->sign(FocusaSpec172LimitedAssertionPayload::build($wrongNodeSigned));
$verifyWrongNodeSigned = $service->verify($wrongNodeSigned, '2026-08-08T00:02:00Z');
expect_limited($verifyWrongNodeSigned['verdict'] === 'denied' && $verifyWrongNodeSigned['code'] === 'ASSERTION_TAMPERED', 'validly-signed wrong-node claim fails the stored-row binding check');

$wrongProduct = $presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']);
$wrongProduct['product_scope'] = 'uiai_engine';
$verifyWrongProduct = $service->verify($wrongProduct, '2026-08-08T00:02:00Z');
expect_limited($verifyWrongProduct['verdict'] === 'denied' && $verifyWrongProduct['code'] === 'SIGNATURE_INVALID', 'wrong-product claim (unsigned for that payload) fails closed');

// Caller cannot widen the family allowlist on issue: caller input is ignored and
// the server-owned posture allowlist is signed instead.
$widenAttempt = $service->issue([
    'posture_uuid' => $alphaPosture['posture_uuid'],
    'family_allowlist' => ['release_proof'],
    'issued_at' => '2026-08-08T00:02:00Z',
    'refresh_at' => '2026-08-08T00:02:00Z',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-issue-widen'],
]);
expect_limited((int) $widenAttempt['sequence'] === 2, 're-issue rotates to the next monotonic sequence');
expect_limited($widenAttempt['family_allowlist'] === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'caller-supplied family input is ignored; allowlist stays server-owned');
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === 2, 'widen attempt creates a server-owned assertion row');

// ── 2. Refresh rotates the bounded credential without access expiry ─────

$refreshedAlpha = $service->refresh([
    'posture_uuid' => $alphaPosture['posture_uuid'],
    'refresh_at' => '2026-08-08T12:00:00Z',
    'idempotency_key' => 'idem-refresh-alpha-1',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-refresh-1'],
]);
expect_limited((int) $refreshedAlpha['sequence'] === 3, 'refresh rotates to the next monotonic sequence');
expect_limited($refreshedAlpha['status'] === 'refreshed', 'refresh marks the bounded credential refreshed');
expect_limited($refreshedAlpha['refresh_at'] === '2026-08-08T12:00:00Z', 'refresh binds a fresh bounded refresh window');
expect_limited($refreshedAlpha['family_allowlist'] === $issuedAlpha['family_allowlist'], 'refresh does not widen the family allowlist');
$alphaPostureAfter = $postures->findByUuid($alphaPosture['posture_uuid']);
expect_limited((int) $alphaPostureAfter['sequence'] === 3, 'posture sequence advances in lockstep with the refreshed credential');
expect_limited((string) $alphaPostureAfter['status'] === 'issued', 'refresh imposes no access expiry on the permanent posture');

$refreshReplay = $service->refresh([
    'posture_uuid' => $alphaPosture['posture_uuid'],
    'refresh_at' => '2026-08-08T12:00:00Z',
    'idempotency_key' => 'idem-refresh-alpha-1',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-refresh-1'],
]);
expect_limited($refreshReplay === $refreshedAlpha, 'refresh replay returns the same credential');

$verifyCurrent = $service->verify($presentAlpha($refreshedAlpha, 3, $verifiedAlpha['account_uuid']), '2026-08-08T12:00:00Z');
expect_limited($verifyCurrent['verdict'] === 'valid', 'refreshed credential verifies');

$verifyStale = $service->verify($presentAlpha($issuedAlpha, 1, $verifiedAlpha['account_uuid']), '2026-08-08T12:00:00Z');
expect_limited($verifyStale['verdict'] === 'denied' && $verifyStale['code'] === 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED', 'stale pre-refresh assertion fails closed');

$verifyWindow = $service->verify($presentAlpha($refreshedAlpha, 3, $verifiedAlpha['account_uuid']), '2026-08-20T00:00:00Z');
expect_limited($verifyWindow['verdict'] === 'denied' && $verifyWindow['code'] === 'CREDENTIAL_REFRESH_REQUIRED', 'elapsed bounded refresh window fails closed until refresh');

// ── 3. Revoke lost device / account abuse is preservation-only ─────────

$assertionRowsBeforeRevoke = (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn();
$revokedAlpha = $service->revoke([
    'posture_uuid' => $alphaPosture['posture_uuid'],
    'reason' => 'lost_device',
    'occurred_at' => '2026-08-08T13:00:00Z',
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-revoke-1'],
]);
expect_limited($revokedAlpha['verdict'] === 'revoked', 'revoke flips the current credential status');
$revokedPosture = $postures->findByUuid($alphaPosture['posture_uuid']);
expect_limited($revokedPosture['status'] === 'revoked' && $revokedPosture['status_reason'] === 'lost_device', 'posture is revoked with the explicit reason');
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === $assertionRowsBeforeRevoke, 'revoke preserves every assertion row');
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 1, 'revoke preserves every posture row');

$verifyRevoked = $service->verify($presentAlpha($refreshedAlpha, 3, $verifiedAlpha['account_uuid']), '2026-08-08T13:00:00Z');
expect_limited($verifyRevoked['verdict'] === 'denied' && $verifyRevoked['code'] === 'VERIFIED_LIMITED_ACCESS', 'revoked credential fails closed');

expect_limited_throws(
    static fn() => $service->issue([
        'posture_uuid' => $alphaPosture['posture_uuid'],
        'issued_at' => '2026-08-08T13:01:00Z',
        'refresh_at' => '2026-08-08T13:01:00Z',
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'post-revoke-issue'],
    ]),
    'VERIFIED_LIMITED_ACCESS',
    'issuing after revoke is denied',
);

expect_limited_throws(
    static fn() => $service->issue([
        'posture_uuid' => P_UNKNOWN,
        'issued_at' => '2026-08-08T00:02:00Z',
        'refresh_at' => '2026-08-08T00:02:00Z',
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'unknown-issue'],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'issue without a verified posture is denied',
);

// ── 4. Recover from verified identity ──────────────────────────────────

expect_limited_throws(
    static fn() => $service->recover([
        'account_uuid' => P_UNKNOWN,
        'product_scope' => 'focusa',
        'recovery_verified_at' => '2026-08-10T00:00:00Z',
        'node_uuid' => 'node-recover-unknown-1',
        'node_digest' => hash('sha256', 'node-recover-unknown'),
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'recover-unknown'],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'recovery for an unknown account is denied',
);

expect_limited_throws(
    static fn() => $service->recover([
        'account_uuid' => $verifiedAlpha['account_uuid'],
        'product_scope' => 'uiai_engine',
        'recovery_verified_at' => '2026-08-10T00:00:00Z',
        'node_uuid' => 'node-recover-widen-1',
        'node_digest' => hash('sha256', 'node-recover-widen'),
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'recover-widen'],
    ]),
    'PRODUCT_NOT_INCLUDED',
    'recovery cannot widen into a product scope the account never held',
);

expect_limited_throws(
    static fn() => $service->recover([
        'account_uuid' => $verifiedAlpha['account_uuid'],
        'product_scope' => 'focusa',
        'recovery_verified_at' => '2026-08-08T12:30:00Z',
        'node_uuid' => 'node-recover-stale-1',
        'node_digest' => hash('sha256', 'node-recover-stale'),
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'recover-stale'],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'stale recovery proof is denied',
);

expect_limited_throws(
    static fn() => $service->recover([
        'account_uuid' => $verifiedAlpha['account_uuid'],
        'product_scope' => 'focusa',
        'recovery_verified_at' => '2026-08-10T00:00:00Z',
        'node_uuid' => 'node-alpha-0001',
        'node_digest' => hash('sha256', 'node-alpha'),
        'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'recover-old-node'],
    ]),
    'NODE_LIMIT_REACHED',
    'recovery cannot reactivate a node that is already registered to the account',
);

$recovered = $service->recover([
    'account_uuid' => $verifiedAlpha['account_uuid'],
    'product_scope' => 'focusa',
    'recovery_verified_at' => '2026-08-10T00:00:00Z',
    'node_uuid' => 'node-alpha-recovered-1',
    'node_digest' => hash('sha256', 'node-alpha-recovered'),
    'migration_provenance' => ['source' => 'limited_assertion_test', 'record' => 'alpha-recover-1'],
]);
expect_limited($recovered['verdict'] === 'valid', 'recovery issues a replacement assertion');
expect_limited((int) $recovered['sequence'] === 4, 'recovery starts at a strictly higher monotonic account sequence');
expect_limited($recovered['node_uuid'] === 'node-alpha-recovered-1', 'recovery binds the freshly registered node');
expect_limited($recovered['signer'] === FocusaSpec172LimitedAssertionService::SIGNER_RECOVERY, 'recovery uses the recovery signer');
expect_limited($recovered['family_allowlist'] === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'recovery never widens into paid families');
$recoveredPosture = $postures->findByUuid($recovered['posture_uuid']);
expect_limited($recoveredPosture['status'] === 'issued' && (int) $recoveredPosture['sequence'] === 4, 'recovered posture is active at the higher sequence');
expect_limited((string) $postures->findByUuid($alphaPosture['posture_uuid'])['status'] === 'revoked', 'the revoked posture stays revoked after recovery');
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 2, 'recovery preserves the revoked posture row');

$verifyRecovered = $service->verify($presentAlpha($recovered, 4, $verifiedAlpha['account_uuid']), '2026-08-10T00:00:00Z');
expect_limited($verifyRecovered['verdict'] === 'valid', 'recovered assertion verifies');

// ── 5. No EDD key / no zero-dollar license ──────────────────────────────

expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn() === 0, 'no EDD Software Licensing key is created');
$eddFree = $assertions->assertEddFree();
expect_limited($eddFree['edd_free'] === true, 'assertion and posture schemas are EDD-free by introspection');

// ── 6. Client/store: only verified assertions are persisted ────────────

$fixturePostures = [];
foreach ($fixture['postures'] as $fixturePosture) {
    $fixturePostures[(string) $fixturePosture['posture_uuid']] = $fixturePosture;
}
$validCount = 0;
$deniedCount = 0;
foreach ($fixture['vectors'] as $vector) {
    $posture = $fixturePostures[(string) $vector['presented']['posture_uuid']] ?? null;
    $result = $client->verifyAndStore($vector['presented'], $posture, (string) $vector['at']);
    if ($vector['expected'] === 'valid') {
        $validCount++;
        expect_limited($result['verdict'] === 'valid', "fixture vector {$vector['id']} must be valid");
    } else {
        $deniedCount++;
        expect_limited($result['verdict'] === 'denied' && $result['code'] === $vector['expected'], "fixture vector {$vector['id']} must fail closed with {$vector['expected']} (got {$result['code']})");
    }
}
// Four vectors are valid; roundtrip and refresh share (posture, sequence) so the
// local mirror keeps one deduplicated row per (posture, sequence) → 3 rows.
expect_limited($validCount === 4 && $deniedCount === count($fixture['vectors']) - 4, 'fixture vectors split into valid and fail-closed sets');
expect_limited($client->storeCount() === 3, 'client store persists only valid assertions (deduplicated by posture+sequence)');
expect_limited($client->storedPostures() === [P_ALPHA, P_DELTA, P_BETA], 'client store mirrors only verified postures');
expect_limited($client->storedAllowlistForPosture(P_ALPHA) === FocusaSpec172VerifiedAccessPostureState::allowlistFor('focusa'), 'client store keeps the canonical allowlist (no widening)');
expect_limited($client->storedCountForPosture(P_GAMMA) === 0, 'revoked assertions are never persisted by the client store');
expect_limited($client->storedCountForPosture(P_UNKNOWN) === 0, 'unverified assertions are never persisted by the client store');

// Self-issued (unsigned) claim must be rejected and never stored.
$selfIssued = $fixture['vectors'][0]['presented'];
$selfIssued['signature'] = str_repeat('00', 64);
$selfIssued['family_allowlist'] = ['release_proof'];
$selfIssuedResult = $client->verifyAndStore($selfIssued, $fixturePostures[P_ALPHA], '2026-08-08T12:00:00Z');
expect_limited($selfIssuedResult['verdict'] === 'denied' && $selfIssuedResult['code'] === 'SIGNATURE_INVALID', 'self-issued unsigned claim fails closed');
expect_limited($client->storeCount() === 3, 'self-issued claim is never persisted');

// ── 7. Routes ──────────────────────────────────────────────────────────

$routeVerify = $routes->route('POST', '/wpuiai/v1/spec172/assertions/verify', $presentAlpha($recovered, 4, $verifiedAlpha['account_uuid']) + ['at' => '2026-08-10T00:00:00Z']);
expect_limited($routeVerify['verdict'] === 'valid', 'verify route dispatches to the verifier');
$routeVerifyDenied = $routes->route('POST', '/wpuiai/v1/spec172/assertions/verify', $presentAlpha($refreshedAlpha, 3, $verifiedAlpha['account_uuid']) + ['at' => '2026-08-08T13:00:00Z']);
expect_limited($routeVerifyDenied['verdict'] === 'denied' && $routeVerifyDenied['code'] === 'VERIFIED_LIMITED_ACCESS', 'verify route masks revoked-credential denials');
$routeUnknown = $routes->route('POST', '/wpuiai/v1/spec172/assertions/nonexistent', []);
expect_limited($routeUnknown['verdict'] === 'denied' && $routeUnknown['code'] === 'ROUTE_NOT_FOUND', 'unknown route fails closed');
$routeMethod = $routes->route('GET', '/wpuiai/v1/spec172/assertions/issue', []);
expect_limited($routeMethod['verdict'] === 'denied' && $routeMethod['code'] === 'ROUTE_NOT_FOUND', 'wrong method fails closed');
$routeStatus = $routes->route('GET', '/wpuiai/v1/spec172/assertions/status', ['posture_uuid' => $alphaPosture['posture_uuid']]);
expect_limited($routeStatus['verdict'] === 'status' && $routeStatus['status'] === 'revoked', 'status route reports the bounded posture status');

// No raw email, key material, or customer data in returned envelopes.
$serialized = json_encode([$issuedAlpha, $refreshedAlpha, $recovered, $verify1, $routeStatus], JSON_THROW_ON_ERROR);
expect_limited(!preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $serialized), 'no raw email appears in any returned envelope');

// ── 8. Rollback contract is preservation-only ──────────────────────────

$postureMigration->preserveForRollback('2026-08-10T01:00:00Z', ['software_target' => 'prior_candidate', 'reason' => 'synthetic_rollback_proof']);
$assertionMigration->preserveForRollback('2026-08-10T01:00:00Z', ['software_target' => 'prior_candidate', 'reason' => 'synthetic_rollback_proof']);
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_verified_access_postures')->fetchColumn() === 2, 'rollback preserves every posture row');
expect_limited((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions')->fetchColumn() === $assertionRowsBeforeRevoke + 1, 'rollback preserves every assertion row');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.limited_assertion_validation.v1',
    'fixture' => 'byte_exact_cross_language',
    'vectors' => count($fixture['vectors']),
    'valid' => $validCount,
    'denied' => $deniedCount,
    'issues' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions WHERE signer = 'wpuiai.spec172.issue.v1'")->fetchColumn(),
    'refreshes' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions WHERE signer = 'wpuiai.spec172.refresh.v1'")->fetchColumn(),
    'revokes' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions WHERE status = 'revoked'")->fetchColumn(),
    'recoveries' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_signed_access_assertions WHERE signer = 'wpuiai.spec172.recovery.v1'")->fetchColumn(),
    'edd_licenses' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_licenses')->fetchColumn(),
    'edd_free' => true,
    'client_stored' => $client->storeCount(),
    'checks' => $positiveChecks + $negativeChecks,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
