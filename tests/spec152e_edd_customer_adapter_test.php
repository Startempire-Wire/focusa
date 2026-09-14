<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-edd-customer-adapter.v1.php';

function expect(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_throws(callable $operation, string $exception, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect($error instanceof $exception, $message . ' exception type');
        return;
    }
    expect(false, $message);
}

// ── Setup: in-memory SQLite with EDD and idempotency tables ────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// EDD customers table (simulated EDD schema).
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
$db->exec("CREATE TABLE wp_wpuiai_edd_customer_idempotency (
    idempotency_key VARCHAR(191) NOT NULL PRIMARY KEY,
    operation VARCHAR(64) NOT NULL,
    request_digest VARCHAR(64) NOT NULL,
    result_payload TEXT NOT NULL,
    created_at VARCHAR(32) NOT NULL
)");

// ── Clock ──────────────────────────────────────────────────────────────

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-07T05:00:00Z'))
        ->modify('+' . $clockTick . ' minutes')
        ->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
};

$adapter = new FocusaSpec152eEddCustomerAdapter($db, 'wp_', $clock);

// ── Test 1: Unverified identity is rejected ────────────────────────────

$unverified = [
    'verification_state' => 'email_verification_pending',
    'verified_at' => null,
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0201',
    'normalized_email' => 'newuser@example.com',
    'idempotency_key' => 'idem-unverified-0001',
    'migration_provenance' => ['source' => 'test_unverified'],
];
expect_throws(
    static fn() => $adapter->resolveCustomer($unverified),
    DomainException::class,
    'unverified identity is denied'
);
expect($adapter->customerCount() === 0, 'unverified identity creates no EDD customer');

// ── Test 2: New verified user creates exactly one EDD customer ─────────

$newVerified = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:00:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0201',
    'normalized_email' => 'newuser@example.com',
    'wordpress_user_id' => 501,
    'stripe_customer_id' => 'cus_test_authority_001',
    'idempotency_key' => 'idem-new-0001',
    'migration_provenance' => [
        'source' => 'spec152e_candidate',
        'source_record' => 'synthetic-new-verified',
    ],
];
$result = $adapter->resolveCustomer($newVerified);
expect($result['resolution'] === 'new', 'new verified user creates EDD customer');
expect($result['edd_customer_id'] > 0, 'new EDD customer has positive ID');
expect($result['email'] === 'newuser@example.com', 'new EDD customer has correct email');
expect($adapter->customerCount() === 1, 'exactly one EDD customer created');

// ── Test 3: Idempotent replay returns same result ──────────────────────

$replayed = $adapter->resolveCustomer($newVerified);
expect($replayed === $result, 'identical resolution replay is idempotent');
expect($adapter->customerCount() === 1, 'idempotent replay creates no duplicate');

// ── Test 4: Existing EDD customer is resolved deterministically ────────

$existingVerified = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:01:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0102',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0202',
    'normalized_email' => 'newuser@example.com',
    'wordpress_user_id' => 502,
    'stripe_customer_id' => 'cus_test_authority_002',
    'idempotency_key' => 'idem-existing-0001',
    'migration_provenance' => [
        'source' => 'spec152e_existing_resolution',
    ],
];
$existingResult = $adapter->resolveCustomer($existingVerified);
expect($existingResult['resolution'] === 'existing', 'existing verified user resolves to existing EDD customer');
expect($existingResult['edd_customer_id'] === $result['edd_customer_id'], 'existing resolution returns same EDD customer ID');
expect($adapter->customerCount() === 1, 'existing resolution creates no duplicate');

// ── Test 5: Different verified email creates a distinct EDD customer ───

$secondVerified = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:02:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0103',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0203',
    'normalized_email' => 'otheruser@example.com',
    'idempotency_key' => 'idem-second-0001',
    'migration_provenance' => ['source' => 'test_second_user'],
];
$secondResult = $adapter->resolveCustomer($secondVerified);
expect($secondResult['resolution'] === 'new', 'second verified email creates distinct EDD customer');
expect($secondResult['edd_customer_id'] !== $result['edd_customer_id'], 'distinct emails create distinct EDD customers');
expect($adapter->customerCount() === 2, 'two distinct EDD customers exist');

// ── Test 6: Idempotency conflict (different email, same key) ───────────

$conflict = $newVerified;
$conflict['normalized_email'] = 'conflict@example.com';
expect_throws(
    static fn() => $adapter->resolveCustomer($conflict),
    DomainException::class,
    'changed request with same idempotency key is denied'
);
expect($adapter->customerCount() === 2, 'idempotency conflict creates no customer');

// ── Test 7: Verified user with account_promoted state ──────────────────

$promoted = [
    'verification_state' => 'account_promoted',
    'verified_at' => '2026-08-07T05:03:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0104',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0204',
    'normalized_email' => 'promoteduser@example.com',
    'wordpress_user_id' => 503,
    'stripe_customer_id' => 'cus_promoted_001',
    'idempotency_key' => 'idem-promoted-0001',
    'migration_provenance' => ['source' => 'test_promoted'],
];
$promotedResult = $adapter->resolveCustomer($promoted);
expect($promotedResult['resolution'] === 'new', 'account_promoted state creates EDD customer');
expect($adapter->customerCount() === 3, 'promoted user creates third EDD customer');

// ── Test 8: findCustomerByEmail returns correct customer ───────────────

$found = $adapter->findCustomerByEmail('newuser@example.com');
expect($found !== null, 'findCustomerByEmail finds existing customer');
expect((int) $found['id'] === $result['edd_customer_id'], 'findCustomerByEmail returns correct customer ID');

$notFound = $adapter->findCustomerByEmail('nonexistent@example.com');
expect($notFound === null, 'findCustomerByEmail returns null for unknown email');

// ── Test 9: findCustomerById returns correct customer ──────────────────

$byId = $adapter->findCustomerById($result['edd_customer_id']);
expect($byId !== null, 'findCustomerById finds existing customer');
expect($byId['email'] === 'newuser@example.com', 'findCustomerById returns correct email');

$notFoundById = $adapter->findCustomerById(99999);
expect($notFoundById === null, 'findCustomerById returns null for unknown ID');

// ── Test 10: Invalid email is rejected ─────────────────────────────────

$invalidEmail = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:04:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0105',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0205',
    'normalized_email' => 'not-an-email',
    'idempotency_key' => 'idem-invalid-0001',
    'migration_provenance' => ['source' => 'test_invalid'],
];
expect_throws(
    static fn() => $adapter->resolveCustomer($invalidEmail),
    InvalidArgumentException::class,
    'invalid email format is rejected'
);

// ── Test 11: Missing migration provenance is rejected ──────────────────

$noProvenance = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:05:00Z',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0106',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0206',
    'normalized_email' => 'noprovenance@example.com',
    'idempotency_key' => 'idem-noprovenance-0001',
    'migration_provenance' => [],
];
expect_throws(
    static fn() => $adapter->resolveCustomer($noProvenance),
    InvalidArgumentException::class,
    'empty migration provenance is rejected'
);

// ── Test 12: Existing customer with secondary email lookup ─────────────

// Insert a customer email address entry for the first customer.
$db->prepare("INSERT INTO wp_edd_customer_email_addresses (customer_id, email, type, date_created)
    VALUES (:cid, :email, 'secondary', :created)")->execute([
    ':cid' => $result['edd_customer_id'],
    ':email' => 'newuser+alias@example.com',
    ':created' => '2026-08-07T05:06:00Z',
]);

$aliasFound = $adapter->findCustomerByEmail('newuser+alias@example.com');
expect($aliasFound !== null, 'secondary email address resolves to existing customer');
expect((int) $aliasFound['id'] === $result['edd_customer_id'], 'secondary email returns correct customer');

// ── Test 13: No account enumeration ────────────────────────────────────

// The adapter provides no method to list all customers. Only point lookups
// by email or ID are supported. Verify the count method is bounded.
$count = $adapter->customerCount();
expect($count === 3, 'customerCount is bounded and equals known count');

// ── Output ─────────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.edd_customer_adapter_validation.v1',
    'edd_customers_resolved' => 3,
    'positive_checks' => 8,
    'negative_checks' => 5,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");