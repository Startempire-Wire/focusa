<?php
// 152E.06.03 Migrate legacy customers through verified identity promotion.
// Invites or challenges legacy EDD/install records, attaches records only after
// mailbox verification plus evidence-backed resolution, preserves purchase/license
// history, and quarantines conflicts without entitlement loss. Verified legacy
// customers merge once; unverified/conflicting records remain recoverable but can
// never activate new nodes. No unverified-email promotion, no local/self-issued
// entitlement, no client-controlled EDD price/grant, and no raw email or secret
// appears in any envelope or journal.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-account-promotion.v1.php';
require_once $root . '/docs/contracts/spec152e-legacy-activation-adapter.v1.php';
require_once $root . '/docs/contracts/spec152e-legacy-customer-migration.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_migration(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_migration_throws_code(callable $operation, string $code, string $message): void
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

function expect_migration_throws_type(callable $operation, string $exception, string $message): void
{
    global $negativeChecks;
    $negativeChecks++;
    try {
        $operation();
    } catch (Throwable $error) {
        if (!($error instanceof $exception)) {
            fwrite(STDERR, "FAIL: {$message} (got " . get_class($error) . ": {$error->getMessage()})\n");
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
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_customer_migration_test']);
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_customer_migration_test']);
$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_customer_migration_test']);
$promotionMigration = new FocusaSpec152eAccountPromotionMigration($db, 'wp_');
$promotionMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_customer_migration_test']);
$migrationSchema = new FocusaSpec152eLegacyCustomerMigrationSchema($db, 'wp_');
$migrationSchema->migrate('2026-08-08T00:00:00Z', ['source' => 'legacy_customer_migration_test']);

// EDD tables (simulated EDD 3.x schema).
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
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_number VARCHAR(32) NULL,
    status VARCHAR(32) NOT NULL,
    type VARCHAR(32) NOT NULL DEFAULT 'sale',
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL,
    user_id INTEGER NULL,
    customer_id BIGINT NOT NULL,
    email VARCHAR(100) NOT NULL DEFAULT ''
)");
$db->exec("CREATE TABLE wp_edd_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    product_name VARCHAR(191) NOT NULL DEFAULT '',
    quantity INTEGER NOT NULL DEFAULT 1
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active'
)");

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-08T00:01:00Z'))
        ->modify('+' . $clockTick . ' minutes')
        ->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
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
$legacy = new FocusaSpec152eLegacyActivationAdapter($db, $registrations, $registrationSecrets, $edd, $clock);
$migration = new FocusaSpec152eLegacyCustomerMigrationService($db, $migrationSchema, $promotion, $registrationSecrets, $clock);

$counts = static function () use ($db): array {
    return [
        'accounts' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn(),
        'customers' => (int) $db->query('SELECT COUNT(*) FROM wp_edd_customers')->fetchColumn(),
        'identities' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities')->fetchColumn(),
        'links' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_account_promotion_purchase_links')->fetchColumn(),
        'challenges' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_legacy_customer_challenges')->fetchColumn(),
        'attachments' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_legacy_customer_attachments')->fetchColumn(),
        'journal' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_legacy_customer_journal')->fetchColumn(),
    ];
};

$eddSnapshot = static function () use ($db): string {
    $tables = ['wp_edd_customers', 'wp_edd_customer_email_addresses', 'wp_edd_orders', 'wp_edd_order_items', 'wp_edd_licenses'];
    $out = [];
    foreach ($tables as $table) {
        $out[$table] = $db->query("SELECT * FROM {$table} ORDER BY id")->fetchAll(PDO::FETCH_ASSOC);
    }
    return json_encode($out, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
};

$registrationSeq = 0;
$createVerified = static function (string $email, string $facade, string $tag) use ($registrations, &$registrationSeq): array {
    $registrationSeq++;
    $created = $registrations->createPending([
        'email' => $email,
        'facade_id' => $facade,
        'presenter' => 'candidate.legacy.migration.test',
        'install_channel' => 'cli',
        'product_code' => 'focusa_operator',
        'safe_redirect_handle' => 'safe-' . $tag . '-' . $registrationSeq,
        'request_id' => 'req-' . $tag . '-' . $registrationSeq,
        'idempotency_key' => 'idem-' . $tag . '-' . $registrationSeq,
    ]);
    $uuid = $created['registration']['registration_uuid'];
    $verified = $registrations->verifyEmail(
        $uuid,
        $created['verification_secret'],
        'req-verify-' . $tag . '-' . $registrationSeq,
        'idem-verify-' . $tag . '-' . $registrationSeq,
    );
    return [
        'registration_uuid' => $uuid,
        'verified_at' => $verified['registration']['verified_at'],
    ];
};

$emailDigestOf = static fn(string $email): string =>
    $registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($email));

$insertLegacyCustomer = static function (string $email) use ($db): int {
    $db->exec("INSERT INTO wp_edd_customers (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
        VALUES (NULL, '{$email}', 'Legacy Customer', 99.50, 1, '', '2026-07-01T00:00:00Z', NULL)");
    return (int) $db->lastInsertId();
};
$insertLegacyOrder = static function (int $customerId, string $email, string $status, string $orderNumber) use ($db): int {
    $db->exec("INSERT INTO wp_edd_orders (order_number, status, type, date_created, date_completed, customer_id, email)
        VALUES ('{$orderNumber}', '{$status}', 'sale', '2026-07-01T00:01:00Z', '2026-07-01T00:02:00Z', {$customerId}, '{$email}')");
    return (int) $db->lastInsertId();
};
$insertLegacyLicense = static function (int $customerId, int $orderId, string $key, string $status) use ($db): int {
    $db->exec("INSERT INTO wp_edd_licenses (license_key, customer_id, order_id, product_id, status)
        VALUES ('{$key}', {$customerId}, {$orderId}, 453, '{$status}')");
    return (int) $db->lastInsertId();
};

// ── Legacy EDD/install fixtures (Spec 152E §22.1) ──────────────────────

// alpha: paid owner, complete order, active key -> verify_first (invite).
$alphaCustomer = $insertLegacyCustomer('legacy.alpha@example.invalid');
$alphaOrder = $insertLegacyOrder($alphaCustomer, 'legacy.alpha@example.invalid', 'complete', 'ORD-2001');
$db->exec("INSERT INTO wp_edd_order_items (order_id, product_id, product_name, quantity)
    VALUES ({$alphaOrder}, 453, 'WPUIAI Pro Lifetime', 1)");
$alphaItem = (int) $db->lastInsertId();
$alphaLicense = $insertLegacyLicense($alphaCustomer, $alphaOrder, 'fl_legacy_alpha_0001', 'active');

// echo: paid owner with a linked secondary email -> evidence_backed_import (invite).
$echoCustomer = $insertLegacyCustomer('legacy.echo@example.invalid');
$db->exec("INSERT INTO wp_edd_customer_email_addresses (customer_id, email, type, date_created)
    VALUES ({$echoCustomer}, 'legacy.echo.linked@example.invalid', 'secondary', '2026-07-02T00:00:00Z')");
$echoOrder = $insertLegacyOrder($echoCustomer, 'legacy.echo@example.invalid', 'complete', 'ORD-2005');
$echoLicense = $insertLegacyLicense($echoCustomer, $echoOrder, 'fl_legacy_echo_0001', 'active');

// delta: valid key owned by delta; install-site record reconciled via Stripe -> verify_first.
$deltaCustomer = $insertLegacyCustomer('legacy.delta@example.invalid');
$deltaOrder = $insertLegacyOrder($deltaCustomer, 'legacy.delta@example.invalid', 'complete', 'ORD-2004');
$deltaLicense = $insertLegacyLicense($deltaCustomer, $deltaOrder, 'fl_legacy_delta_0001', 'active');

// dup: duplicate paid record -> duplicate (challenge; stronger evidence required).
$dupCustomer = $insertLegacyCustomer('legacy.dup@example.invalid');
$dupOrder = $insertLegacyOrder($dupCustomer, 'legacy.dup@example.invalid', 'complete', 'ORD-2007');
$dupLicense = $insertLegacyLicense($dupCustomer, $dupOrder, 'fl_legacy_dup_0001', 'active');

// bravo: key revoked -> refunded_revoked (challenge; never attachable).
$bravoCustomer = $insertLegacyCustomer('legacy.bravo@example.invalid');
$bravoOrder = $insertLegacyOrder($bravoCustomer, 'legacy.bravo@example.invalid', 'complete', 'ORD-2002');
$bravoLicense = $insertLegacyLicense($bravoCustomer, $bravoOrder, 'fl_legacy_bravo_0001', 'revoked');

// charlie: order refunded -> refunded_revoked (challenge; never attachable).
$charlieCustomer = $insertLegacyCustomer('legacy.charlie@example.invalid');
$charlieOrder = $insertLegacyOrder($charlieCustomer, 'legacy.charlie@example.invalid', 'refunded', 'ORD-2003');
$charlieLicense = $insertLegacyLicense($charlieCustomer, $charlieOrder, 'fl_legacy_charlie_0001', 'active');

// synthetic: install-site key with no EDD license truth -> synthetic_quarantine.
// foxtrot: valid paid owner used only for pre-merge negative paths; stays open.

$fixturePath = $root . '/docs/contracts/spec152e-legacy-customer-fixture.v1.json';
$fixtureDigest = hash_file('sha256', $fixturePath);
$fixtureRaw = file_get_contents($fixturePath);
$fixture = json_decode($fixtureRaw, true, 512, JSON_THROW_ON_ERROR);

// ── Fixture validation (redacted, deterministic, replayable) ───────────

expect_migration($fixture['schema'] === 'focusa.spec152e.legacy_customer_fixture.v1', 'fixture schema is typed');
expect_migration($fixture['fixture_id'] === 'focusa-vbcqu.20.13.51', 'fixture id pins the atom');
expect_migration($fixture['authority']['canonical'] === 'WPUIAI.com EDD', 'fixture authority is canonical EDD');
expect_migration($fixture['authority']['spec158'] === 'excluded', 'fixture excludes Spec 158');
expect_migration($fixture['redaction']['raw_email'] === 'absent', 'fixture declares raw email absent');
expect_migration($fixture['redaction']['customer_payload_stored'] === false, 'fixture declares no customer payload');
$fixtureHandles = array_column($fixture['records'], 'handle');
expect_migration(count($fixtureHandles) === count(array_unique($fixtureHandles)), 'fixture record handles are unique');
foreach ($fixture['records'] as $record) {
    expect_migration(preg_match('/^rec_[a-z0-9_]{6,64}$/D', $record['handle']) === 1, 'fixture handle is bounded and opaque');
    expect_migration(in_array($record['surface'], FocusaSpec152eLegacyCustomerMigrationService::SURFACES, true), 'fixture surface is known');
    expect_migration(in_array($record['disposition'], FocusaSpec152eLegacyCustomerMigrationService::DISPOSITIONS, true), 'fixture disposition is known');
}
expect_migration(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $fixtureRaw) !== 1, 'fixture contains no email address');
expect_migration(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $fixtureRaw) !== 1, 'fixture contains no secret prefix');
expect_migration(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/i', $fixtureRaw) !== 1, 'fixture contains no synthetic focusa_live key');
expect_migration(preg_match('/^[0-9a-f]{64}$/D', $fixtureDigest) === 1, 'fixture digest is bounded sha256');
$fixtureByHandle = [];
foreach ($fixture['records'] as $record) {
    $fixtureByHandle[$record['handle']] = $record;
}

// ── Verified registrations for legacy owners ───────────────────────────

$regAlpha = $createVerified('legacy.alpha@example.invalid', 'focusa_install_v1', 'alpha');
$regEchoLinked = $createVerified('legacy.echo.linked@example.invalid', 'focusa_install_v1', 'echolinked');
$regDelta = $createVerified('legacy.delta@example.invalid', 'focusa_install_v1', 'delta');
$regDup = $createVerified('legacy.dup@example.invalid', 'focusa_install_v1', 'dup');
$regFoxtrot = $createVerified('legacy.foxtrot@example.invalid', 'focusa_install_v1', 'foxtrot');
$regBravo = $createVerified('legacy.bravo@example.invalid', 'focusa_install_v1', 'bravo');
$regCharlie = $createVerified('legacy.charlie@example.invalid', 'focusa_install_v1', 'charlie');
$regSynthetic = $createVerified('legacy.synthetic@example.invalid', 'focusa_install_v1', 'synthetic');
$regUnrelated = $createVerified('unrelated@example.invalid', 'focusa_install_v1', 'unrelated');

$pendingFoxtrot = $registrations->createPending([
    'email' => 'legacy.foxtrot@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'candidate.legacy.migration.test',
    'install_channel' => 'cli',
    'product_code' => 'focusa_operator',
    'safe_redirect_handle' => 'safe-foxtrot-pending',
    'request_id' => 'req-foxtrot-pending-0001',
    'idempotency_key' => 'idem-foxtrot-pending-0001',
]);
$pendingFoxtrotUuid = $pendingFoxtrot['registration']['registration_uuid'];

// ── Open invites/challenges from the redacted fixture ──────────────────

$open = static fn(string $handle, string $email) => $migration->openChallenge([
    'record_handle' => $handle,
    'surface' => $fixtureByHandle[$handle]['surface'],
    'disposition' => $fixtureByHandle[$handle]['disposition'],
    'email_lookup_digest' => $emailDigestOf($email),
    'legacy_evidence' => $fixtureByHandle[$handle]['evidence'],
    'request_id' => 'req-open-' . $handle . '-0001',
    'idempotency_key' => 'idem-open-' . $handle . '-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => $handle],
]);

$alphaChallenge = $open('rec_legacy_alpha', 'legacy.alpha@example.invalid');
$echoChallenge = $open('rec_legacy_echo', 'legacy.echo.linked@example.invalid');
$deltaChallenge = $open('rec_legacy_delta', 'legacy.delta@example.invalid');
$dupChallenge = $open('rec_legacy_dup', 'legacy.dup@example.invalid');
$syntheticChallenge = $open('rec_legacy_synthetic', 'legacy.synthetic@example.invalid');
$refundedChallenge = $open('rec_legacy_refunded', 'legacy.charlie@example.invalid');
$revokedChallenge = $open('rec_legacy_revoked', 'legacy.bravo@example.invalid');
$foxtrotChallenge = $open('rec_legacy_foxtrot', 'legacy.foxtrot@example.invalid');

expect_migration($alphaChallenge['action'] === 'challenge_opened', 'alpha invite/challenge opened');
expect_migration($alphaChallenge['mode'] === 'invite', 'verify_first records are invited');
expect_migration($echoChallenge['mode'] === 'invite', 'evidence_backed_import records are invited');
expect_migration($deltaChallenge['mode'] === 'invite', 'stripe-reconciled verify_first records are invited');
expect_migration($dupChallenge['mode'] === 'challenge', 'duplicate records are challenged');
expect_migration($syntheticChallenge['mode'] === 'challenge', 'synthetic records are challenged');
expect_migration($refundedChallenge['mode'] === 'challenge', 'refunded records are challenged');
expect_migration($revokedChallenge['mode'] === 'challenge', 'revoked records are challenged');
foreach ([$alphaChallenge, $echoChallenge, $deltaChallenge, $dupChallenge, $syntheticChallenge, $refundedChallenge, $revokedChallenge, $foxtrotChallenge] as $challenge) {
    expect_migration($challenge['schema'] === 'focusa.spec152e.legacy_customer_migration_result.v1', 'challenge returns the typed envelope');
    expect_migration($challenge['state'] === 'open', 'opened challenges start open');
    expect_migration($challenge['replayed'] === false && $challenge['existing'] === false, 'first open is not a replay');
    expect_migration(preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $challenge['challenge_uuid']) === 1, 'challenge carries an opaque UUID');
    $envelope = json_encode($challenge, JSON_THROW_ON_ERROR);
    expect_migration(!str_contains($envelope, '@example.invalid'), 'challenge envelope never leaks the raw email');
    expect_migration(!str_contains($envelope, 'legacy.'), 'challenge envelope never leaks the email local part');
}
$alphaChallengeJson = json_encode($alphaChallenge, JSON_THROW_ON_ERROR);
expect_migration(!str_contains($alphaChallengeJson, $emailDigestOf('legacy.alpha@example.invalid')), 'challenge envelope never echoes the keyed digest');

// One challenge per record handle: repeated open returns the same challenge.
$alphaReopen = $open('rec_legacy_alpha', 'legacy.alpha@example.invalid');
expect_migration($alphaReopen['challenge_uuid'] === $alphaChallenge['challenge_uuid'], 'repeated open returns the same challenge');
expect_migration($alphaReopen['replayed'] === true || $alphaReopen['existing'] === true, 'repeated open is a replay or dedupe, never a second challenge');
expect_migration($counts()['challenges'] === 8, 'exactly one challenge per record handle');
expect_migration($counts()['attachments'] === 0 && $counts()['accounts'] === 0 && $counts()['customers'] === 6, 'opening challenges attaches nothing and creates no account/customer');

// ── Positive: verified legacy customer merges once (alpha) ─────────────

$eddBeforeAlpha = $eddSnapshot();
$alphaAttach = $migration->attachVerified([
    'challenge_uuid' => $alphaChallenge['challenge_uuid'],
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'request_id' => 'req-attach-alpha-0001',
    'idempotency_key' => 'idem-attach-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_alpha'],
    'legacy_key' => 'fl_legacy_alpha_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_alpha']['evidence'],
    'prior_purchases' => [['order_id' => $alphaOrder, 'item_id' => $alphaItem, 'license_id' => $alphaLicense]],
]);
expect_migration($alphaAttach['action'] === 'legacy_customer_attached', 'verified legacy customer attaches');
expect_migration($alphaAttach['merged_once'] === true, 'attachment reports a single merge');
expect_migration($alphaAttach['record_handle'] === 'rec_legacy_alpha', 'attachment binds the record handle');
expect_migration($alphaAttach['edd_customer_id'] === $alphaCustomer, 'attachment binds the existing EDD customer');
expect_migration($alphaAttach['linked_orders'] === [$alphaOrder], 'purchase history links the EDD order');
expect_migration($alphaAttach['linked_licenses'] === [$alphaLicense], 'purchase history links the EDD license');
expect_migration($alphaAttach['replayed'] === false && $alphaAttach['existing'] === false, 'first attach is fresh');
$alphaAttachJson = json_encode($alphaAttach, JSON_THROW_ON_ERROR);
expect_migration(!str_contains($alphaAttachJson, '@example.invalid'), 'attachment never leaks the raw email');
expect_migration(!str_contains($alphaAttachJson, 'fl_legacy_alpha_0001'), 'attachment never leaks the license key');

// Replay with the same idempotency key and a new canonical key both dedupe.
$alphaReplay = $migration->attachVerified([
    'challenge_uuid' => $alphaChallenge['challenge_uuid'],
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'request_id' => 'req-attach-alpha-0001',
    'idempotency_key' => 'idem-attach-alpha-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_alpha'],
    'legacy_key' => 'fl_legacy_alpha_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_alpha']['evidence'],
    'prior_purchases' => [['order_id' => $alphaOrder, 'item_id' => $alphaItem, 'license_id' => $alphaLicense]],
]);
expect_migration($alphaReplay['replayed'] === true, 'same idempotency key replays the stored attachment');
expect_migration($alphaReplay['attachment_uuid'] === $alphaAttach['attachment_uuid'], 'replay returns the same attachment');
$alphaAgain = $migration->attachVerified([
    'challenge_uuid' => $alphaChallenge['challenge_uuid'],
    'registration_uuid' => $regAlpha['registration_uuid'],
    'verified_email' => 'legacy.alpha@example.invalid',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-08T00:30:00Z',
    'request_id' => 'req-attach-alpha-0002',
    'idempotency_key' => 'idem-attach-alpha-0002',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_alpha'],
    'legacy_key' => 'fl_legacy_alpha_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_alpha']['evidence'],
    'prior_purchases' => [['order_id' => $alphaOrder, 'item_id' => $alphaItem, 'license_id' => $alphaLicense]],
]);
expect_migration($alphaAgain['existing'] === true, 'a repeated canonical request returns the existing merge');
expect_migration($alphaAgain['attachment_uuid'] === $alphaAttach['attachment_uuid'], 'repeated request never merges a second time');
expect_migration($counts()['attachments'] === 1, 'verified legacy customer merges exactly once');
expect_migration($counts()['accounts'] === 1 && $counts()['identities'] === 1 && $counts()['links'] === 1, 'one account, identity, and purchase link per merge');
expect_migration($eddSnapshot() === $eddBeforeAlpha, 'alpha migration preserves EDD order/license/customer truth byte-identically');
$registrationAfter = $registrations->findByUuid($regAlpha['registration_uuid']);
expect_migration($registrationAfter['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'registration advances to account_promoted');

// ── Positive: linked legacy email migrates through verification (echo) ──

$eddBeforeEcho = $eddSnapshot();
$echoAttach = $migration->attachVerified([
    'challenge_uuid' => $echoChallenge['challenge_uuid'],
    'registration_uuid' => $regEchoLinked['registration_uuid'],
    'verified_email' => 'legacy.echo.linked@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:31:00Z',
    'request_id' => 'req-attach-echo-0001',
    'idempotency_key' => 'idem-attach-echo-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_echo'],
    'legacy_key' => 'fl_legacy_echo_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_echo']['evidence'],
    'prior_purchases' => [['order_id' => $echoOrder, 'license_id' => $echoLicense]],
]);
expect_migration($echoAttach['action'] === 'legacy_customer_attached', 'verified linked owner email attaches the legacy record');
expect_migration($echoAttach['edd_customer_id'] === $echoCustomer, 'linked-email merge resolves the existing EDD customer');
expect_migration($echoAttach['linked_orders'] === [$echoOrder] && $echoAttach['linked_licenses'] === [$echoLicense], 'echo history is linked and preserved');
expect_migration($eddSnapshot() === $eddBeforeEcho, 'echo migration preserves all EDD truth including the linked email row');
expect_migration($counts()['accounts'] === 2 && $counts()['identities'] === 2 && $counts()['links'] === 2, 'echo merge adds exactly one account/identity/link');

// ── Conflict quarantines without entitlement loss, then recovers (delta) ─

$eddBeforeDelta = $eddSnapshot();
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $deltaChallenge['challenge_uuid'],
        'registration_uuid' => $regDelta['registration_uuid'],
        'verified_email' => 'legacy.delta@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:32:00Z',
        'request_id' => 'req-attach-delta-conflict-0001',
        'idempotency_key' => 'idem-attach-delta-conflict-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_delta'],
        'legacy_key' => 'fl_legacy_delta_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_delta']['evidence'],
        'prior_purchases' => [['order_id' => $bravoOrder, 'license_id' => $bravoLicense]],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'conflicting paid records require review and are quarantined'
);
expect_migration($counts() === ['accounts' => 2, 'customers' => 6, 'identities' => 2, 'links' => 2, 'challenges' => 8, 'attachments' => 2, 'journal' => 11], 'the failed merge writes zero entitlement state');
expect_migration($eddSnapshot() === $eddBeforeDelta, 'the conflict never mutates EDD truth');
$deltaRow = $db->query("SELECT * FROM wp_wpuiai_legacy_customer_challenges WHERE challenge_uuid = '{$deltaChallenge['challenge_uuid']}'")->fetch(PDO::FETCH_ASSOC);
expect_migration($deltaRow['state'] === 'quarantined', 'conflicting record is journaled quarantined');
expect_migration($deltaRow['quarantine_reason'] === 'ACCOUNT_MERGE_REVIEW_REQUIRED', 'quarantine records the fail-closed reason');

$deltaReopen = $migration->reopenQuarantined([
    'challenge_uuid' => $deltaChallenge['challenge_uuid'],
    'request_id' => 'req-reopen-delta-0001',
    'idempotency_key' => 'idem-reopen-delta-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_delta'],
]);
expect_migration($deltaReopen['action'] === 'legacy_customer_reopened' && $deltaReopen['state'] === 'open', 'quarantined record is recoverable via reopen');

$deltaAttach = $migration->attachVerified([
    'challenge_uuid' => $deltaChallenge['challenge_uuid'],
    'registration_uuid' => $regDelta['registration_uuid'],
    'verified_email' => 'legacy.delta@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:33:00Z',
    'request_id' => 'req-attach-delta-0001',
    'idempotency_key' => 'idem-attach-delta-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_delta'],
    'legacy_key' => 'fl_legacy_delta_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_delta']['evidence'],
    'prior_purchases' => [['order_id' => $deltaOrder, 'license_id' => $deltaLicense]],
]);
expect_migration($deltaAttach['action'] === 'legacy_customer_attached', 'reopened record attaches after correct evidence');
expect_migration($counts()['accounts'] === 3 && $counts()['identities'] === 3 && $counts()['links'] === 3, 'delta adds exactly one account/identity/link');

// ── Duplicate record: review, synthetic-evidence quarantine, recovery ──

expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $dupChallenge['challenge_uuid'],
        'registration_uuid' => $regDup['registration_uuid'],
        'verified_email' => 'legacy.dup@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:34:00Z',
        'request_id' => 'req-attach-dup-conflict-0001',
        'idempotency_key' => 'idem-attach-dup-conflict-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_dup'],
        'legacy_key' => 'fl_legacy_dup_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_dup']['evidence'],
        'prior_purchases' => [['order_id' => $echoOrder, 'license_id' => $echoLicense]],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'duplicate records claiming foreign purchases enter review'
);
$migration->reopenQuarantined([
    'challenge_uuid' => $dupChallenge['challenge_uuid'],
    'request_id' => 'req-reopen-dup-0001',
    'idempotency_key' => 'idem-reopen-dup-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_dup'],
]);
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $dupChallenge['challenge_uuid'],
        'registration_uuid' => $regDup['registration_uuid'],
        'verified_email' => 'legacy.dup@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:35:00Z',
        'request_id' => 'req-attach-dup-synthetic-0001',
        'idempotency_key' => 'idem-attach-dup-synthetic-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_dup'],
        'legacy_key' => 'fl_legacy_dup_0001',
        'legacy_evidence' => ['kind' => 'synthetic', 'source' => 'custom_key_generator', 'record' => 'legacy-dup-synthetic'],
        'prior_purchases' => [['order_id' => $dupOrder, 'license_id' => $dupLicense]],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'synthetic evidence never attaches and quarantines the record'
);
$migration->reopenQuarantined([
    'challenge_uuid' => $dupChallenge['challenge_uuid'],
    'request_id' => 'req-reopen-dup-0002',
    'idempotency_key' => 'idem-reopen-dup-0002',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_dup'],
]);
$dupAttach = $migration->attachVerified([
    'challenge_uuid' => $dupChallenge['challenge_uuid'],
    'registration_uuid' => $regDup['registration_uuid'],
    'verified_email' => 'legacy.dup@example.invalid',
    'verification_method' => 'otp',
    'transactional_consent_at' => '2026-08-08T00:36:00Z',
    'request_id' => 'req-attach-dup-0001',
    'idempotency_key' => 'idem-attach-dup-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_dup'],
    'legacy_key' => 'fl_legacy_dup_0001',
    'legacy_evidence' => $fixtureByHandle['rec_legacy_dup']['evidence'],
    'prior_purchases' => [['order_id' => $dupOrder, 'license_id' => $dupLicense]],
]);
expect_migration($dupAttach['action'] === 'legacy_customer_attached', 'recovered duplicate record attaches with correct evidence');
expect_migration($counts()['accounts'] === 4 && $counts()['identities'] === 4 && $counts()['links'] === 4, 'dup adds exactly one account/identity/link');

// ── Negative: pre-merge gates keep records open and recoverable (foxtrot) ─

$eddBeforeFoxtrot = $eddSnapshot();
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $foxtrotChallenge['challenge_uuid'],
        'registration_uuid' => $pendingFoxtrotUuid,
        'verified_email' => 'legacy.foxtrot@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:37:00Z',
        'request_id' => 'req-attach-foxtrot-unverified-0001',
        'idempotency_key' => 'idem-attach-foxtrot-unverified-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_foxtrot'],
        'legacy_key' => 'fl_legacy_foxtrot_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_foxtrot']['evidence'],
        'prior_purchases' => [],
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'an unverified registration can never attach a legacy record'
);
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $foxtrotChallenge['challenge_uuid'],
        'registration_uuid' => $regUnrelated['registration_uuid'],
        'verified_email' => 'unrelated@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:38:00Z',
        'request_id' => 'req-attach-foxtrot-email-0001',
        'idempotency_key' => 'idem-attach-foxtrot-email-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_foxtrot'],
        'legacy_key' => 'fl_legacy_foxtrot_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_foxtrot']['evidence'],
        'prior_purchases' => [],
    ]),
    'ACCOUNT_EMAIL_MISMATCH',
    'a verified email that is not the legacy record email never attaches'
);
expect_migration_throws_type(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $foxtrotChallenge['challenge_uuid'],
        'registration_uuid' => $regFoxtrot['registration_uuid'],
        'verified_email' => 'legacy.foxtrot@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:39:00Z',
        'request_id' => 'req-attach-foxtrot-nokey-0001',
        'idempotency_key' => 'idem-attach-foxtrot-nokey-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_foxtrot'],
        'legacy_evidence' => $fixtureByHandle['rec_legacy_foxtrot']['evidence'],
        'prior_purchases' => [],
    ]),
    InvalidArgumentException::class,
    'an attach without a legacy key is malformed input'
);
$foxtrotRow = $db->query("SELECT * FROM wp_wpuiai_legacy_customer_challenges WHERE challenge_uuid = '{$foxtrotChallenge['challenge_uuid']}'")->fetch(PDO::FETCH_ASSOC);
expect_migration($foxtrotRow['state'] === 'open', 'pre-merge failures leave the record open and recoverable');
expect_migration($eddSnapshot() === $eddBeforeFoxtrot, 'pre-merge failures write zero EDD/authority state');
expect_migration($counts()['accounts'] === 4 && $counts()['identities'] === 4, 'no account or identity is created by failed attaches');

// ── Negative: quarantine-only dispositions never attach ────────────────

expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $syntheticChallenge['challenge_uuid'],
        'registration_uuid' => $regSynthetic['registration_uuid'],
        'verified_email' => 'legacy.synthetic@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:40:00Z',
        'request_id' => 'req-attach-synthetic-0001',
        'idempotency_key' => 'idem-attach-synthetic-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_synthetic'],
        'legacy_key' => 'fl_legacy_synthetic_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_synthetic']['evidence'],
        'prior_purchases' => [],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'synthetic records remain quarantined and can never attach'
);
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $refundedChallenge['challenge_uuid'],
        'registration_uuid' => $regCharlie['registration_uuid'],
        'verified_email' => 'legacy.charlie@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:41:00Z',
        'request_id' => 'req-attach-refunded-0001',
        'idempotency_key' => 'idem-attach-refunded-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_refunded'],
        'legacy_key' => 'fl_legacy_charlie_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_refunded']['evidence'],
        'prior_purchases' => [],
    ]),
    'EDD_LICENSE_UNUSABLE',
    'refunded records never attach and never reactivate'
);
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $revokedChallenge['challenge_uuid'],
        'registration_uuid' => $regBravo['registration_uuid'],
        'verified_email' => 'legacy.bravo@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:42:00Z',
        'request_id' => 'req-attach-revoked-0001',
        'idempotency_key' => 'idem-attach-revoked-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_revoked'],
        'legacy_key' => 'fl_legacy_bravo_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_revoked']['evidence'],
        'prior_purchases' => [],
    ]),
    'EDD_LICENSE_UNUSABLE',
    'revoked records never attach and never reactivate'
);
$syntheticRow = $db->query("SELECT * FROM wp_wpuiai_legacy_customer_challenges WHERE challenge_uuid = '{$syntheticChallenge['challenge_uuid']}'")->fetch(PDO::FETCH_ASSOC);
expect_migration($syntheticRow['state'] === 'quarantined' && $syntheticRow['quarantine_reason'] === 'EDD_ORDER_UNVERIFIED', 'synthetic record is journaled quarantined');

// ── Operator quarantine without verification stays recoverable (unresolved) ─

$unresolvedQuarantine = $migration->quarantineRecord([
    'record_handle' => 'rec_legacy_unresolved',
    'surface' => $fixtureByHandle['rec_legacy_unresolved']['surface'],
    'disposition' => $fixtureByHandle['rec_legacy_unresolved']['disposition'],
    'quarantine_reason' => 'EDD_ORDER_UNVERIFIED',
    'email_lookup_digest' => $emailDigestOf('legacy.unresolved@example.invalid'),
    'legacy_evidence' => $fixtureByHandle['rec_legacy_unresolved']['evidence'],
    'request_id' => 'req-quarantine-unresolved-0001',
    'idempotency_key' => 'idem-quarantine-unresolved-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_unresolved'],
]);
expect_migration($unresolvedQuarantine['action'] === 'legacy_customer_quarantined', 'unverified record is quarantined without entitlement loss');
expect_migration($unresolvedQuarantine['state'] === 'quarantined' && $unresolvedQuarantine['quarantine_reason'] === 'EDD_ORDER_UNVERIFIED', 'quarantine records the fail-closed reason');
expect_migration($counts()['challenges'] === 9, 'operator quarantine creates exactly one challenge');

$unresolvedChallengeUuid = $unresolvedQuarantine['challenge_uuid'];
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $unresolvedChallengeUuid,
        'registration_uuid' => $regFoxtrot['registration_uuid'],
        'verified_email' => 'legacy.unresolved@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:43:00Z',
        'request_id' => 'req-attach-unresolved-0001',
        'idempotency_key' => 'idem-attach-unresolved-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_unresolved'],
        'legacy_key' => 'fl_legacy_unresolved_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_unresolved']['evidence'],
        'prior_purchases' => [],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'a quarantined record cannot attach while quarantined'
);
$unresolvedReopen = $migration->reopenQuarantined([
    'challenge_uuid' => $unresolvedChallengeUuid,
    'request_id' => 'req-reopen-unresolved-0001',
    'idempotency_key' => 'idem-reopen-unresolved-0001',
    'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_unresolved'],
]);
expect_migration($unresolvedReopen['state'] === 'open', 'quarantined unresolved record is recoverable');
expect_migration_throws_code(
    fn() => $migration->attachVerified([
        'challenge_uuid' => $unresolvedChallengeUuid,
        'registration_uuid' => $regFoxtrot['registration_uuid'],
        'verified_email' => 'legacy.unresolved@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:44:00Z',
        'request_id' => 'req-attach-unresolved-0002',
        'idempotency_key' => 'idem-attach-unresolved-0002',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_unresolved'],
        'legacy_key' => 'fl_legacy_unresolved_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_unresolved']['evidence'],
        'prior_purchases' => [],
    ]),
    'EDD_ORDER_UNVERIFIED',
    'unresolved records re-quarantine at the disposition gate and can never attach'
);
expect_migration_throws_code(
    fn() => $migration->quarantineRecord([
        'record_handle' => 'rec_legacy_alpha',
        'surface' => $fixtureByHandle['rec_legacy_alpha']['surface'],
        'disposition' => $fixtureByHandle['rec_legacy_alpha']['disposition'],
        'quarantine_reason' => 'EDD_ORDER_UNVERIFIED',
        'email_lookup_digest' => $emailDigestOf('legacy.alpha@example.invalid'),
        'legacy_evidence' => $fixtureByHandle['rec_legacy_alpha']['evidence'],
        'request_id' => 'req-quarantine-alpha-0001',
        'idempotency_key' => 'idem-quarantine-alpha-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_alpha'],
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'an already-attached record can never be quarantined away'
);
expect_migration_throws_type(
    fn() => $migration->reopenQuarantined([
        'challenge_uuid' => $alphaChallenge['challenge_uuid'],
        'request_id' => 'req-reopen-alpha-0001',
        'idempotency_key' => 'idem-reopen-alpha-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'rec_legacy_alpha'],
    ]),
    OutOfBoundsException::class,
    'attached records are never reopened'
);
expect_migration_throws_type(
    fn() => $migration->attachVerified([
        'challenge_uuid' => '00000000-0000-4000-8000-000000000000',
        'registration_uuid' => $regFoxtrot['registration_uuid'],
        'verified_email' => 'legacy.foxtrot@example.invalid',
        'verification_method' => 'otp',
        'transactional_consent_at' => '2026-08-08T00:45:00Z',
        'request_id' => 'req-attach-missing-0001',
        'idempotency_key' => 'idem-attach-missing-0001',
        'migration_provenance' => ['source' => 'spec152e_candidate', 'record' => 'missing'],
        'legacy_key' => 'fl_legacy_missing_0001',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_alpha']['evidence'],
        'prior_purchases' => [],
    ]),
    OutOfBoundsException::class,
    'an unknown challenge fails closed'
);

// ── Quarantined / never-attached records cannot activate new nodes ────

expect_migration_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $pendingFoxtrotUuid,
        'verified_email' => 'legacy.foxtrot@example.invalid',
        'license_key' => 'fl_legacy_foxtrot_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_foxtrot']['evidence'],
        'request_id' => 'req-node-unverified-0001',
    ]),
    'EMAIL_VERIFICATION_REQUIRED',
    'an unverified legacy record cannot activate a new node'
);
expect_migration_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regBravo['registration_uuid'],
        'verified_email' => 'legacy.bravo@example.invalid',
        'license_key' => 'fl_legacy_bravo_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_revoked']['evidence'],
        'request_id' => 'req-node-revoked-0001',
    ]),
    'EDD_LICENSE_UNUSABLE',
    'a revoked legacy license cannot activate a new node'
);
expect_migration_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regCharlie['registration_uuid'],
        'verified_email' => 'legacy.charlie@example.invalid',
        'license_key' => 'fl_legacy_charlie_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_refunded']['evidence'],
        'request_id' => 'req-node-refunded-0001',
    ]),
    'EDD_ORDER_UNVERIFIED',
    'a refunded legacy order cannot activate a new node'
);
expect_migration_throws_code(
    fn() => $legacy->resolveForActivation([
        'registration_uuid' => $regSynthetic['registration_uuid'],
        'verified_email' => 'legacy.synthetic@example.invalid',
        'license_key' => 'fl_legacy_synthetic_0001',
        'purpose' => 'node_activation',
        'legacy_evidence' => $fixtureByHandle['rec_legacy_synthetic']['evidence'],
        'request_id' => 'req-node-synthetic-0001',
    ]),
    'EDD_LICENSE_UNVERIFIED',
    'a synthetic install-site key cannot activate a new node'
);

// ── Rollback is preservation-only ──────────────────────────────────────

$beforeRollback = $counts();
$rollback = $migrationSchema->preserveForRollback('2026-08-08T03:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_legacy_customer_migration_rollback',
]);
expect_migration($rollback['action'] === 'preserve', 'migration rollback is preservation-only');
expect_migration($counts() === $beforeRollback, 'rollback preserves all migration truth');
expect_migration((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_legacy_customer_migration_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

// ── Hygiene: journals store digests, never emails or secrets ───────────

$journalDump = json_encode($db->query('SELECT * FROM wp_wpuiai_legacy_customer_journal')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)
    . json_encode($db->query('SELECT * FROM wp_wpuiai_legacy_customer_challenges')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)
    . json_encode($db->query('SELECT * FROM wp_wpuiai_legacy_customer_attachments')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR);
expect_migration(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $journalDump) !== 1, 'no unmasked email in migration journals');
expect_migration(preg_match('/fl_legacy_[a-z0-9_]+/', $journalDump) !== 1, 'no license key material in migration journals');
expect_migration(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $journalDump) !== 1, 'no secret prefix in migration journals');
$journalEvents = $db->query('SELECT DISTINCT event_type FROM wp_wpuiai_legacy_customer_journal')->fetchAll(PDO::FETCH_COLUMN);
foreach (['invite_opened', 'challenge_opened', 'attached', 'quarantined', 'reopened'] as $eventType) {
    expect_migration(in_array($eventType, $journalEvents, true), "journal records {$eventType} events");
}
$finalState = $counts();
expect_migration($finalState['challenges'] === 9 && $finalState['attachments'] === 4, 'four verified legacy customers merged once across nine records');
expect_migration($finalState['customers'] === 6, 'no EDD customer rows were added or lost during migration');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.legacy_customer_migration_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'accounts' => $finalState['accounts'],
    'customers' => $finalState['customers'],
    'identities' => $finalState['identities'],
    'purchase_links' => $finalState['links'],
    'challenges' => $finalState['challenges'],
    'attachments' => $finalState['attachments'],
    'quarantined_records' => 4,
    'fixture_digest_sha256' => $fixtureDigest,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
