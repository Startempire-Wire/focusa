<?php
declare(strict_types=1);
// Exact verification for focusa-vbcqu.20.13.35: the EDD-bound signed lease
// issuer (spec 152E §7.5, §10, §11, §12, §15, §17, §18, §19, §20, §23).
// Issues signed leases only after verified account + usable EDD license +
// settled exact order/item + server-owned product grant + settled bound node;
// payload carries account/customer/order/item/license/product/features/limits/
// commercial/node/sequence/time/kid claims. Golden vectors are byte-exact and
// independently verified by tests/spec152e_lease_golden_vector_test.py.

require_once dirname(__DIR__) . '/docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';

const VECTOR_PATH = __DIR__ . '/../docs/contracts/spec152e-lease-golden-vectors.v1.json';
const RUST_FIXTURE_PATH = __DIR__ . '/../crates/focusa-license/tests/fixtures/spec152-authority-golden-vector.json';
const NOW = '2026-08-08T18:30:00Z';
const PAID_DEVICE_KEY = 'AbCdEfGhIjKlMnOpQrStUvWxYz0123456789ab_CDef';
const EVAL_DEVICE_KEY = 'bMgpSjs62S8N-Lb9mJHdwWjFQkoy7Pk5eVAzRZVpr1s';
const BUNDLE_DEVICE_KEY = 'MRbKpJXgHwdJWDgVS1GNes25jmCMkTxatw7Lk8ivo0c';

$positive = 0;
$negative = 0;

function expect_lease(bool $condition, string $message): void
{
    global $positive;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
    $positive++;
}

function expect_lease_throws(callable $operation, string $exception, string $message): void
{
    global $negative;
    try {
        $operation();
    } catch (Throwable $error) {
        if ($error instanceof $exception) {
            $negative++;
            return;
        }
        fwrite(STDERR, 'FAIL: ' . $message . ' (unexpected ' . get_class($error) . ': ' . $error->getMessage() . ")\n");
        exit(1);
    }
    fwrite(STDERR, 'FAIL: ' . $message . " (no exception thrown)\n");
    exit(1);
}

function expect_lease_domain(callable $operation, string $code, string $message): void
{
    global $negative;
    try {
        $operation();
    } catch (DomainException $error) {
        if ($error->getMessage() === $code) {
            $negative++;
            return;
        }
        fwrite(STDERR, 'FAIL: ' . $message . ' (unexpected DomainException code ' . $error->getMessage() . ")\n");
        exit(1);
    } catch (Throwable $error) {
        fwrite(STDERR, 'FAIL: ' . $message . ' (unexpected ' . get_class($error) . ': ' . $error->getMessage() . ")\n");
        exit(1);
    }
    fwrite(STDERR, 'FAIL: ' . $message . " (no exception thrown)\n");
    exit(1);
}

function seed_fixture(PDO $db): void
{
    $db->exec('CREATE TABLE wp_edd_customers (customer_id INTEGER PRIMARY KEY, email TEXT, name TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_orders (order_id INTEGER PRIMARY KEY, customer_id INTEGER, status TEXT, total TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_edd_order_items (order_item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER, price_id INTEGER, quantity INTEGER, subtotal TEXT, total TEXT)');
    $db->exec('CREATE TABLE wp_edd_licenses (license_id INTEGER PRIMARY KEY, customer_id INTEGER, download_id INTEGER, payment_id INTEGER, license_key TEXT, status TEXT, activation_limit INTEGER, expiration TEXT, date_created TEXT)');
    $db->exec('CREATE TABLE wp_wpuiai_authority_accounts (account_uuid TEXT PRIMARY KEY, customer_id INTEGER, status TEXT, status_reason TEXT, highest_entitlement_sequence INTEGER)');
    $db->exec('CREATE TABLE wp_wpuiai_authority_nodes (node_uuid TEXT PRIMARY KEY, account_uuid TEXT, edd_license_id INTEGER, product_code TEXT, device_public_key TEXT, assurance_class TEXT, status TEXT)');

    $db->exec("INSERT INTO wp_edd_customers VALUES (1001, 'c1001@example.invalid', 'Paid Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9001, 1001, 'complete', '697.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90011, 9001, 1001, 0, 1, '697.00', '697.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7001, 1001, 1001, 9001, 'F0C15A-0001-0001-0001-0001', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 1001, 'active', 'mailbox_verified', 41)");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-paid-golden-001', 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 7001, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");

    $db->exec("INSERT INTO wp_edd_customers VALUES (2002, 'c2002@example.invalid', 'Eval Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9002, 2002, 'complete', '0.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90022, 9002, 1004, 0, 1, '0.00', '0.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7002, 2002, 1004, 9002, 'E5A10000-0002-0002-0002-0002', 'active', 1, '2026-09-07T18:30:00Z', '2026-08-08T18:30:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e', 2002, 'active', 'account_promoted', 6)");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-eval-golden-001', 'b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e', 7002, 'focusa_evaluation', '" . EVAL_DEVICE_KEY . "', 'device_key_v1', 'active')");

    $db->exec("INSERT INTO wp_edd_customers VALUES (3003, 'c3003@example.invalid', 'Bundle Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9003, 3003, 'complete', '1254.60', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90033, 9003, 1003, 0, 1, '1254.60', '1254.60')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7003, 3003, 1003, 9003, 'BUNDLE-0003-0003-0003-0003', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f', 3003, 'active', 'mailbox_verified', 8)");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-bundle-golden-001', 'c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f', 7003, 'focusa_uiai_operator_bundle_lifetime_v1', '" . BUNDLE_DEVICE_KEY . "', 'device_key_v1', 'active')");

    // Negative-path fixtures: unverified account, revoked/expired licenses, a
    // pending order, a cross-customer license, a deactivated and a foreign node.
    // Licenses 7005-7009 belong to customer 1001 (matching account
    // e5f6a7b8-...-8091) so status/order/price checks are what reject; license
    // 7004 stays on customer 4004 for the cross-customer rejection.
    $db->exec("INSERT INTO wp_edd_customers VALUES (4004, 'c4004@example.invalid', 'Negative Fixture', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9004, 1001, 'complete', '697.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90044, 9004, 1001, 0, 1, '697.00', '697.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7004, 4004, 1001, 9004, 'F0C15A-0004-0004-0004-0004', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7005, 1001, 1001, 9004, 'F0C15A-0005-0005-0005-0005', 'revoked', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7006, 1001, 1001, 9004, 'F0C15A-0006-0006-0006-0006', 'active', 0, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7007, 1001, 1001, 9004, 'F0C15A-0007-0007-0007-0007', 'active', 3, '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9005, 1001, 'pending', '697.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90055, 9005, 1001, 0, 1, '697.00', '697.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7008, 1001, 1001, 9005, 'F0C15A-0008-0008-0008-0008', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_orders VALUES (9006, 1001, 'complete', '299.00', '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_edd_order_items VALUES (90066, 9006, 1001, 0, 1, '299.00', '299.00')");
    $db->exec("INSERT INTO wp_edd_licenses VALUES (7009, 1001, 1001, 9006, 'F0C15A-0009-0009-0009-0009', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
    $db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('d4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f80', 4004, 'pending', 'email_challenge_sent', 0)");
    $db->exec("INSERT INTO wp_wpuiai_authority_accounts VALUES ('e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091', 1001, 'active', 'mailbox_verified', 41)");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-deactivated-001', 'e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091', 7001, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'deactivated')");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-foreign-001', 'd4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f80', 7001, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
    $db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-wrongproduct-001', 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 7001, 'uiai_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
}

function build_issuer(PDO $db): FocusaSpec152eEddBoundLeaseIssuer
{
    $keySet = new FocusaSpec152eAuthorityKeySetSeam(
        implode('', array_map('chr', range(0, 31))),
        implode('', array_map('chr', range(32, 63))),
        static fn() => NOW,
    );
    $issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keySet, static fn() => NOW, 'wp_');
    $issuer->migrate('2026-08-08T05:00:00Z', ['source' => 'edd_bound_lease_issuer_test', 'work_item' => 'focusa-vbcqu.20.13.35']);
    $issuer->migrate('2026-08-08T05:01:00Z', ['source' => 'repeat_must_preserve_first_schema_application']);
    return $issuer;
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
seed_fixture($db);

$issuer = build_issuer($db);

$migrationRows = $db->query('SELECT * FROM wp_wpuiai_authority_lease_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_lease(count($migrationRows) === 1, 'repeated lease migration records one schema version');
expect_lease($migrationRows[0]['applied_at'] === '2026-08-08T05:00:00Z', 'repeated lease migration preserves first applied timestamp');
expect_lease(str_contains($migrationRows[0]['migration_provenance'], 'edd_bound_lease_issuer_test'), 'repeated lease migration preserves first provenance');
$vector = json_decode(file_get_contents(VECTOR_PATH), true, 512, JSON_THROW_ON_ERROR);
expect_lease($vector['schema'] === 'focusa.spec152e.lease_golden_vectors.v1', 'golden vector schema');
expect_lease($vector['fixture_kind'] === 'public_synthetic_nonproduction', 'golden vector fixture kind');

$request = static fn(string $account, string $product, string $node, string $devKey, string $ikey): array => [
    'account_uuid' => $account,
    'product_code' => $product,
    'node_id' => $node,
    'device_public_key' => $devKey,
    'idempotency_key' => $ikey,
    'request_id' => 'req-' . $ikey,
];

$paidRequest = $request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'lease-golden-paid-0001')
    + ['lease_uuid' => 'a1b2c3d4-0000-4000-8000-000000000001', 'lease_id' => 'lease-golden-paid-0001', 'issued_at' => NOW];
$evalRequest = $request('b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e', 'focusa_evaluation', 'node-eval-golden-001', EVAL_DEVICE_KEY, 'lease-golden-eval-0001')
    + ['lease_uuid' => 'b2c3d4e5-0000-4000-8000-000000000002', 'lease_id' => 'lease-golden-eval-0001', 'issued_at' => NOW];
$bundleRequest = $request('c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f', 'focusa_uiai_operator_bundle_lifetime_v1', 'node-bundle-golden-001', BUNDLE_DEVICE_KEY, 'lease-golden-bundle-0001')
    + ['lease_uuid' => 'c3d4e5f6-0000-4000-8000-000000000003', 'lease_id' => 'lease-golden-bundle-0001', 'issued_at' => NOW];

// ── Positive: paid / evaluation / bundle issuance byte-exact with golden vectors ──

$paid = $issuer->issueLease($paidRequest);
expect_lease($paid['envelope'] === $vector['vectors']['paid']['envelope'], 'paid envelope byte-exact with the golden vector');
expect_lease($paid['claims'] === $vector['vectors']['paid']['claims'], 'paid claims byte-exact with the golden vector');
expect_lease($paid['sequence'] === 42, 'paid lease server-derived sequence 42');
expect_lease($paid['posture'] === 'paid', 'paid lease posture');

$claims = $paid['claims'];
expect_lease($claims['schema'] === 'focusa.authority_lease.v1', 'payload schema is the canonical authority lease');
expect_lease($claims['product'] === 'focusa', 'payload product is the canonical focusa scope');
expect_lease($claims['subject_id'] === $claims['account_id'] && $claims['account_id'] === 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'account claim');
expect_lease($claims['customer_id'] === 1001, 'customer claim');
expect_lease($claims['order_id'] === 9001 && $claims['order_item_id'] === 90011, 'order/item claims');
expect_lease($claims['edd_license_id'] === 7001, 'license claim');
expect_lease($claims['node_id'] === 'node-paid-golden-001', 'node claim');
expect_lease($claims['authority_key_id'] === 'authority-lease-2026-01', 'kid claim');
expect_lease($claims['issued_at'] === NOW && $claims['not_before'] === NOW, 'time claims (issued/not-before)');
expect_lease($claims['expires_at'] === '2026-11-06T18:30:00Z', 'time claim (expiry = now + 90d)');
expect_lease($claims['offline_grace_until'] === '2026-12-06T18:30:00Z', 'time claim (offline grace = expiry + 30d)');
expect_lease($claims['status'] === 'active', 'status claim');
expect_lease($claims['features']['base_focusa'] === true && $claims['features']['premium_updates'] === true, 'paid feature grants');
expect_lease(($claims['limits']['operator_seats'] ?? null) === 1 && ($claims['limits']['node_limit'] ?? null) === 3, 'paid limits');
expect_lease($claims['commercial']['price_usd'] === '697.00' && $claims['commercial']['term'] === 'lifetime', 'paid commercial claims');
expect_lease($claims['commercial']['refund_policy'] === 'whole_order_30_days', 'paid refund policy claim');

$eval = $issuer->issueLease($evalRequest);
expect_lease($eval['envelope'] === $vector['vectors']['evaluation']['envelope'], 'evaluation envelope byte-exact with the golden vector');
expect_lease($eval['claims'] === $vector['vectors']['evaluation']['claims'], 'evaluation claims byte-exact with the golden vector');
expect_lease($eval['sequence'] === 7, 'evaluation lease server-derived sequence 7');
expect_lease($eval['claims']['posture'] === 'evaluation', 'evaluation posture claim');
expect_lease($eval['claims']['expires_at'] === '2026-09-07T18:30:00Z', 'evaluation expiry = now + 30d');
expect_lease($eval['claims']['offline_grace_until'] === null, 'evaluation has no offline grace');
expect_lease(($eval['claims']['limits']['operator_seats'] ?? null) === 1 && ($eval['claims']['limits']['node_limit'] ?? null) === 1, 'evaluation limits');
expect_lease($eval['claims']['commercial']['price_usd'] === '0.00', 'evaluation zero price');
expect_lease($eval['claims']['features']['automation'] === false, 'evaluation limited features');

$bundle = $issuer->issueLease($bundleRequest);
expect_lease($bundle['envelope'] === $vector['vectors']['bundle']['envelope'], 'bundle envelope byte-exact with the golden vector');
expect_lease($bundle['claims'] === $vector['vectors']['bundle']['claims'], 'bundle claims byte-exact with the golden vector');
expect_lease($bundle['sequence'] === 9, 'bundle lease server-derived sequence 9');
expect_lease($bundle['claims']['posture'] === 'bundle', 'bundle posture claim');
expect_lease($bundle['claims']['features']['base_uiai'] === true, 'bundle exact-union feature claim');
expect_lease($bundle['claims']['commercial']['price_usd'] === '1254.60', 'bundle commercial claim');

// ── Positive: existing verifier semantics on the issued envelopes ──

$verify = static function (array $envelope, array $context) use ($issuer): array {
    return $issuer->verifyEnvelope($envelope, $context);
};
$paidState = $verify($paid['envelope'], [
    'expected_product' => 'focusa',
    'expected_node_id' => 'node-paid-golden-001',
    'now' => NOW,
    'minimum_sequence' => 42,
]);
expect_lease($paidState['state'] === 'active', 'verifier accepts the paid lease as Active');
expect_lease($paidState['lease_id'] === 'lease-golden-paid-0001' && $paidState['sequence'] === 42, 'verifier snapshot lease id/sequence');
expect_lease($paidState['lease_digest'] === 'sha256:' . hash('sha256', FocusaSpec152eAuthorityKeySetSeam::decodePayload($paid['envelope']['payload_b64'])), 'verifier derived lease digest');
expect_lease($paidState['features']['automation'] === true && $paidState['limits']['node_limit'] === 3, 'verifier snapshot grants');

$evalState = $verify($eval['envelope'], [
    'expected_product' => 'focusa',
    'expected_node_id' => 'node-eval-golden-001',
    'now' => NOW,
    'minimum_sequence' => 7,
]);
expect_lease($evalState['state'] === 'active', 'verifier accepts the evaluation lease as Active');

$bundleState = $verify($bundle['envelope'], [
    'expected_product' => 'focusa',
    'expected_node_id' => 'node-bundle-golden-001',
    'now' => NOW,
    'minimum_sequence' => 9,
]);
expect_lease($bundleState['state'] === 'active', 'verifier accepts the bundle lease as Active');

$graceState = $verify($paid['envelope'], [
    'expected_product' => 'focusa',
    'expected_node_id' => 'node-paid-golden-001',
    'now' => '2026-12-01T00:00:00Z',
    'minimum_sequence' => 42,
]);
expect_lease($graceState['state'] === 'offline_grace', 'verifier maps post-expiry pre-grace-end to OfflineGrace');

expect_lease_domain(
    static fn() => $verify($paid['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => 'node-paid-golden-001',
        'now' => '2027-01-01T00:00:00Z', 'minimum_sequence' => 42,
    ]),
    'EXPIRED',
    'verifier rejects a lease past expiry and grace',
);

// ── Positive: idempotency and sequence ledger ──

$replay = $issuer->issueLease($paidRequest);
expect_lease($replay['envelope'] === $paid['envelope'], 'idempotent replay returns the byte-identical lease');
expect_lease($issuer->leaseCount() === 3, 'replay does not create a duplicate lease');

$ledger = $issuer->sequenceLedger('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1');
expect_lease($ledger !== null && (int) $ledger['current_sequence'] === 42, 'sequence ledger records the issued sequence');

$secondPaid = $issuer->issueLease(
    $request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'lease-second-paid-0001')
);
expect_lease($secondPaid['sequence'] === 43, 'second lease for the same account/product is strictly monotonic (43)');
expect_lease($secondPaid['claims']['previous_lease_digest'] === 'sha256:' . hash('sha256', FocusaSpec152eAuthorityKeySetSeam::decodePayload($paid['envelope']['payload_b64'])), 'previous digest equals the exact prior payload digest');

// ── Positive: refund lifecycle → sequence bump → stale prior lease, fresh issue ──

$db->exec("UPDATE wp_wpuiai_authority_accounts SET highest_entitlement_sequence = 45 WHERE account_uuid = 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d'");
expect_lease_domain(
    static fn() => $verify($paid['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => 'node-paid-golden-001',
        'now' => NOW, 'minimum_sequence' => 45,
    ]),
    'STALE_SEQUENCE',
    'refund/revoke/expiry sequence bump makes the prior lease stale',
);
$postRefund = $issuer->issueLease(
    $request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'lease-post-refund-0001')
);
expect_lease($postRefund['sequence'] === 46, 'fresh lease after the transition jumps past the bumped sequence');
$db->exec("UPDATE wp_wpuiai_authority_accounts SET highest_entitlement_sequence = 41 WHERE account_uuid = 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d'");

// ── Positive: verifier negative matrix (signed but invalid claims) ──

$negatives = $vector['negatives'];
$expectReason = static function (array $negative, string $code) use ($verify, $vector): void {
    $context = [
        'expected_product' => $negative['expected_product'] ?? 'focusa',
        'expected_node_id' => $negative['expected_node_id'] ?? 'node-paid-golden-001',
        'now' => $negative['now'] ?? NOW,
        'minimum_sequence' => $negative['minimum_sequence'] ?? null,
    ];
    if ($code === 'UNKNOWN_KEY') {
        expect_lease_domain(
            static fn() => $verify($negative['envelope'], $context),
            'UNKNOWN_KEY',
            'verifier rejects ' . $negative['case'],
        );
        return;
    }
    if ($code === 'INVALID_SIGNATURE') {
        expect_lease_domain(
            static fn() => $verify($negative['envelope'], $context),
            'INVALID_SIGNATURE',
            'verifier rejects ' . $negative['case'],
        );
        return;
    }
    expect_lease_domain(
        static fn() => $verify($negative['envelope'], $context),
        $code,
        'verifier rejects ' . $negative['case'],
    );
};
foreach ($negatives as $negativeVector) {
    $expectReason($negativeVector, [
        'wrong_product' => 'WRONG_PRODUCT',
        'stale_sequence' => 'STALE_SEQUENCE',
        'expired' => 'EXPIRED',
        'revoked' => 'REVOKED_LEASE',
        'refunded' => 'STALE_SEQUENCE',
        'unknown_key' => 'UNKNOWN_KEY',
        'unbound_node' => 'WRONG_NODE',
        'invalid_signature' => 'INVALID_SIGNATURE',
    ][$negativeVector['case']]);
}

expect_lease_domain(
    static fn() => $verify($paid['envelope'], [
        'expected_product' => 'focusa', 'expected_node_id' => 'node-paid-golden-001',
        'now' => '2026-08-02T00:00:00Z', 'minimum_sequence' => 42,
    ]),
    'NOT_YET_VALID',
    'verifier rejects a lease before not_before',
);

// ── Positive: pure signer byte-compat with the trusted Rust golden vector ──

$rustFixture = json_decode(file_get_contents(RUST_FIXTURE_PATH), true, 512, JSON_THROW_ON_ERROR);
$keySetSeam = new FocusaSpec152eAuthorityKeySetSeam(
    implode('', array_map('chr', range(0, 31))),
    implode('', array_map('chr', range(32, 63))),
    static fn() => NOW,
);
$keySetEnvelope = $keySetSeam->keySetEnvelope('2026-08-01T00:00:00Z', '2030-01-01T00:00:00Z', '2026-08-01T00:00:00Z', '2029-01-01T00:00:00Z');
expect_lease($keySetSeam->rootPublicKeyB64() === $rustFixture['root_public_key_b64'], 'root key reproduces the trusted Rust fixture root key');
expect_lease($keySetEnvelope === $rustFixture['key_set_envelope'], 'key-set envelope byte-identical to the trusted Rust golden vector (ed25519-dalek compatible)');
expect_lease($vector['key_set_envelope'] === $rustFixture['key_set_envelope'], 'published lease vectors reuse the trusted key set');
$rootVerify = FocusaSpec152eEd25519Signer::verify(
    base64_decode($rustFixture['root_public_key_b64']),
    base64_decode($rustFixture['key_set_envelope']['signature_b64']),
    FocusaSpec152eEd25519Signer::KEY_SET_DOMAIN,
    FocusaSpec152eAuthorityKeySetSeam::decodePayload($rustFixture['key_set_envelope']['payload_b64']),
);
expect_lease($rootVerify === true, 'pure-PHP verifier accepts the trusted Rust key-set signature');

// ── Negative: issuer fails closed before signing ──

$negRequest = $request('d4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f80', 'focusa_operator_lifetime_v1', 'node-foreign-001', PAID_DEVICE_KEY, 'neg-unverified-0001');
expect_lease_domain(static fn() => $issuer->issueLease($negRequest), 'EMAIL_VERIFICATION_REQUIRED', 'unverified account never issues');

$negRequest = $request('e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091', 'focusa_operator_lifetime_v1', 'node-deactivated-001', PAID_DEVICE_KEY, 'neg-account-0001');
expect_lease_domain(static fn() => $issuer->issueLease($negRequest), 'NODE_NOT_ACTIVE', 'deactivated node never issues');
expect_lease_domain(
    static fn() => $issuer->issueLease($request('e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091', 'focusa_operator_lifetime_v1', 'node-foreign-001', PAID_DEVICE_KEY, 'neg-foreign-node-0001')),
    'NODE_NOT_BOUND',
    'node bound to another account never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-wrongproduct-001', PAID_DEVICE_KEY, 'neg-wrongproduct-0001')),
    'NODE_NOT_BOUND',
    'node bound to another product never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', 'NotTheDeviceKey000000000000000000000000000001', 'neg-device-0001')),
    'NODE_PUBLIC_KEY_REQUIRED',
    'wrong device public key never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', 'short-key', 'neg-malformed-key-0001')),
    'NODE_PUBLIC_KEY_REQUIRED',
    'malformed device public key fails closed',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-missing-001', PAID_DEVICE_KEY, 'neg-missing-node-0001')),
    'NODE_NOT_FOUND',
    'unknown node never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-grants-0001') + ['features' => ['release' => true]]),
    'CALLER_CONTROLLED_GRANT_DENIED',
    'caller-supplied features are never accepted',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-price-0001') + ['price' => '9.99']),
    'CALLER_CONTROLLED_GRANT_DENIED',
    'caller-supplied price is never accepted',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-sequence-0001') + ['sequence' => 99]),
    'CALLER_CONTROLLED_GRANT_DENIED',
    'caller-supplied sequence is never accepted',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_unknown_product', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-product-0001')),
    'PRODUCT_MAPPING_REQUIRED',
    'unknown product code fails closed',
);
expect_lease_throws(
    static fn() => $issuer->issueLease($request('not-a-uuid', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-uuid-0001')),
    'InvalidArgumentException',
    'malformed account uuid fails closed',
);
expect_lease_throws(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, '')),
    'InvalidArgumentException',
    'empty idempotency key fails closed',
);
expect_lease_throws(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'neg-conflict-0001') + ['lease_uuid' => 'not-a-uuid']),
    'InvalidArgumentException',
    'malformed lease uuid fails closed',
);

// Idempotency conflict: same key, different request digest.
$issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'lease-conflict-0001'));
$conflictRequest = $request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-paid-golden-001', PAID_DEVICE_KEY, 'lease-conflict-0001');
$conflictRequest['node_id'] = 'node-other-001';
expect_lease_domain(
    static fn() => $issuer->issueLease($conflictRequest),
    'IDEMPOTENCY_CONFLICT',
    'changed reuse of an idempotency key fails closed',
);

// Refunded/revoked/expired licenses never issue.
$revokedAccount = 'e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091';
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-revoked-001', '{$revokedAccount}', 7005, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-zero-001', '{$revokedAccount}', 7006, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-expired-001', '{$revokedAccount}', 7007, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-pending-001', '{$revokedAccount}', 7008, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-price-001', '{$revokedAccount}', 7009, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-crosslicense-001', '{$revokedAccount}', 7004, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("INSERT INTO wp_edd_licenses VALUES (7010, 1001, 1001, 9001, 'F0C15A-0010-0010-0010-0010', 'active', 3, NULL, '2026-08-01T00:00:00Z')");
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-crosscustomer-001', 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 7010, 'focusa_operator_lifetime_v1', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
$db->exec("UPDATE wp_edd_licenses SET customer_id = 4004 WHERE license_id = 7010");
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-revoked-001', PAID_DEVICE_KEY, 'neg-revoked-0001')),
    'EDD_LICENSE_UNUSABLE',
    'revoked EDD license never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-zero-001', PAID_DEVICE_KEY, 'neg-zero-0001')),
    'EDD_LICENSE_UNUSABLE',
    'zero-capacity EDD license never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-expired-001', PAID_DEVICE_KEY, 'neg-expired-license-0001')),
    'EDD_LICENSE_UNUSABLE',
    'expired EDD license never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-pending-001', PAID_DEVICE_KEY, 'neg-pending-0001')),
    'EDD_ORDER_PENDING',
    'pending EDD order never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-price-001', PAID_DEVICE_KEY, 'neg-price-order-0001')),
    'EDD_ORDER_UNVERIFIED',
    'order item price mismatch never issues (exact price relationship)',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_operator_lifetime_v1', 'node-crosslicense-001', PAID_DEVICE_KEY, 'neg-cross-0001')),
    'LICENSE_ACCOUNT_MISMATCH',
    'license of another customer never issues',
);
expect_lease_domain(
    static fn() => $issuer->issueLease($request('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', 'focusa_operator_lifetime_v1', 'node-crosscustomer-001', PAID_DEVICE_KEY, 'neg-crosscustomer-0001')),
    'LICENSE_ACCOUNT_MISMATCH',
    'node bound to a license of another customer never issues',
);
$db->exec("INSERT INTO wp_wpuiai_authority_nodes VALUES ('node-eval-wronglicense-001', '{$revokedAccount}', 7001, 'focusa_evaluation', '" . PAID_DEVICE_KEY . "', 'device_key_v1', 'active')");
expect_lease_domain(
    static fn() => $issuer->issueLease($request($revokedAccount, 'focusa_evaluation', 'node-eval-wronglicense-001', PAID_DEVICE_KEY, 'neg-eval-download-0001')),
    'EDD_ORDER_UNVERIFIED',
    'evaluation node bound to a paid license download never issues',
);

// ── Positive: hygiene — no email, no license key, no secrets in any output ──

$raw = '';
$issuedResults = ['paid' => $paid, 'evaluation' => $eval, 'bundle' => $bundle];
foreach (['envelope', 'claims'] as $key) {
    foreach ($issuedResults as $result) {
        $raw .= json_encode($result[$key]);
    }
}
foreach ($vector['negatives'] as $negativeFixture) {
    $raw .= json_encode($negativeFixture['envelope']);
}
$raw .= $db->query('SELECT * FROM wp_wpuiai_authority_leases')->fetchAll(PDO::FETCH_ASSOC) ? json_encode($db->query('SELECT * FROM wp_wpuiai_authority_leases')->fetchAll(PDO::FETCH_ASSOC)) : '';
$raw .= $db->query('SELECT * FROM wp_wpuiai_authority_lease_sequences')->fetchAll(PDO::FETCH_ASSOC) ? json_encode($db->query('SELECT * FROM wp_wpuiai_authority_lease_sequences')->fetchAll(PDO::FETCH_ASSOC)) : '';
expect_lease(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $raw) !== 1, 'hygiene: no unmasked email in lease output');
expect_lease(preg_match('/[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}-[0-9A-F]{8}/', $raw) !== 1, 'hygiene: no license key material in lease output');
expect_lease(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $raw) !== 1, 'hygiene: no secret prefixes in lease output');
expect_lease(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/i', $raw) !== 1, 'hygiene: no synthetic focusa_live keys in lease output');

// ── Positive: rollback preservation journal (schema events) ──

$issuer->migrate('2026-08-08T05:00:00Z', ['source' => 'rollback_preservation_probe']);
$eventRows = $db->query('SELECT * FROM wp_wpuiai_authority_lease_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_lease(count($eventRows) === 1, 'rollback-preservation migration stays idempotent');

echo json_encode([
    'schema' => 'focusa.spec152e.edd_bound_lease_issuer_validation.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'leases_issued' => 6,
    'golden_vectors' => ['paid', 'evaluation', 'bundle'],
    'negatives_rejected' => count($negatives),
    'result' => 'passed_fail_closed',
], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), "\n";
