<?php
// 152E.02.09 Reconcile EDD, accounts, registrations, licenses, nodes, and leases.
// A bounded reconciliation command/job compares canonical EDD truth (customers, orders,
// licenses, subscriptions) against the authority account registry, lifecycle
// projections, signed outbox events, and the verified-access lease/node registries.
// Missing callbacks cannot leave stale access permanently active: canonical terminal
// transitions with no matching lifecycle projection are projected through the
// Spec 152E lifecycle projector (strictly monotonic sequence; terminal states never
// reactivate) and missing signed outbox events are appended from canonical rows.
// Repairs are evidence-safe and preservation-only: nothing is deleted, refund/revoke/
// sequence truth is never rolled back, and a canonical row that would reactivate a
// terminal projection fails closed (LICENSE_TERMINAL_REACTIVATION_DENIED) into
// quarantine. Ambiguous/conflicting records are quarantined with an exact reason:
// customers with no verified account are never promoted (raw email match alone never
// transfers ownership), duplicate/conflicting account links require operator merge
// review, synthetic and unverifiable records quarantine unless separately approved,
// and leases/nodes without signed evidence quarantine. The job is dry-run/apply,
// idempotent, and converges: repeated apply runs repair every safe fixture exactly
// once and leave only the stable quarantine set. No raw email, payment secret,
// license key, or unmasked real-email evidence is accepted or stored; no client-
// controlled price/amount/grant/feature/limit/tier/download field is accepted.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';
require_once $root . '/docs/contracts/spec152e-edd-lifecycle-projection.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-outbox.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-reconciliation.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_reconcile(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_reconcile_throws(callable $operation, string $code, string $message): void
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

function summary_value(array $report, string $key): mixed
{
    return $report['summary'][$key] ?? null;
}

function count_classification(array $report, string $classification): int
{
    $count = 0;
    foreach ($report['findings'] as $finding) {
        if (($finding['classification'] ?? '') === $classification) {
            $count++;
        }
    }
    return $count;
}

function table_count(PDO $db, string $table): int
{
    return (int) $db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

$accountMigration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$accountMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_reconciliation_test']);
$lifecycleMigration = new FocusaSpec152eEddLifecycleProjectionMigration($db, 'wp_');
$lifecycleMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_reconciliation_test']);
$outboxMigration = new FocusaSpec152eAuthorityOutboxMigration($db, 'wp_');
$outboxMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_reconciliation_test']);
$reconciliationMigration = new FocusaSpec152eAuthorityReconciliationMigration($db, 'wp_');
$reconciliationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'authority_reconciliation_test']);

// Canonical EDD fixture tables (mirrors the lifecycle-projection fixtures).
$db->exec("CREATE TABLE wp_edd_customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '',
    purchase_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    purchase_count INTEGER NOT NULL DEFAULT 0,
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status VARCHAR(32) NOT NULL,
    customer_id BIGINT NOT NULL,
    date_created VARCHAR(32) NOT NULL,
    date_completed VARCHAR(32) NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key VARCHAR(191) NOT NULL,
    customer_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    order_id BIGINT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    date_created VARCHAR(32) NOT NULL
)");
// Verified-access lease/node registries (Spec 172 posture schema, read by the reconciler).
$db->exec("CREATE TABLE wp_wpuiai_verified_access_nodes (
    node_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
    account_uuid VARCHAR(36) NOT NULL,
    node_digest VARCHAR(64) NOT NULL,
    registered_at VARCHAR(32) NOT NULL,
    migration_provenance TEXT NOT NULL
)");
$db->exec("CREATE TABLE wp_wpuiai_verified_access_postures (
    posture_uuid VARCHAR(64) NOT NULL PRIMARY KEY,
    account_uuid VARCHAR(36) NOT NULL,
    identity_uuid VARCHAR(36) NOT NULL,
    registration_uuid VARCHAR(36) NOT NULL,
    product_scope VARCHAR(32) NOT NULL,
    node_uuid VARCHAR(64) NOT NULL,
    sequence BIGINT NOT NULL,
    issued_at VARCHAR(32) NOT NULL,
    refresh_at VARCHAR(32) NOT NULL,
    signer VARCHAR(64) NOT NULL,
    status VARCHAR(16) NOT NULL,
    status_reason VARCHAR(191) NOT NULL,
    migration_provenance TEXT NOT NULL,
    created_at VARCHAR(32) NOT NULL,
    updated_at VARCHAR(32) NOT NULL
)");

$nowValue = '2026-08-08T00:01:00Z';
$clock = static function () use (&$nowValue): string {
    return $nowValue;
};
$tick = static function (int $seconds) use (&$nowValue): void {
    $nowValue = gmdate('Y-m-d\TH:i:s\Z', (int) (new DateTimeImmutable($nowValue))->format('U') + $seconds);
};

$accounts = new FocusaSpec152eAuthorityAccountRepository($db, $accountMigration, $clock);
$secret = 'spec152e-reconciliation-hmac-test-secret-v1';
$signer = new FocusaSpec152eAuthorityEventSigner($secret);
$eventSchema = new FocusaSpec152eAuthorityEventSchema();
$hook = new FocusaSpec152eEddAuthorityHook($db, $outboxMigration, $eventSchema, $signer, $accounts, 'wp_', $clock);
$projector = new FocusaSpec152eEddLifecycleProjector($db, $accounts, $lifecycleMigration, 'wp_', $clock);
$classifier = new FocusaSpec152eDiscrepancyClassifier();
$reconciler = new FocusaSpec152eAuthorityReconciler($db, $reconciliationMigration, $accounts, $projector, $hook, $classifier, 'wp_', $clock);

// Insert canonical EDD fixtures.
$insertCustomer = $db->prepare("INSERT INTO wp_edd_customers (email, name, date_created) VALUES (:email, :name, :created)");
$insertCustomer->execute([':email' => 'customer101@example.test', ':name' => 'Test Customer 101', ':created' => '2026-08-08T00:00:00Z']);
$customerIds = ['c101' => (int) $db->lastInsertId()];
foreach (['c102' => 'customer102@example.test', 'c103' => 'customer103@example.test', 'c104' => 'customer104@example.test', 'c105' => 'customer105@example.test'] as $key => $email) {
    $insertCustomer->execute([':email' => $email, ':name' => "Test {$key}", ':created' => '2026-08-08T00:00:00Z']);
    $customerIds[$key] = (int) $db->lastInsertId();
}

$orderIds = [];
$orderStatuses = ['o1001' => 'completed', 'o1002' => 'refunded', 'o1003' => 'revoked', 'o1004' => 'cancelled', 'o1005' => 'completed', 'o1006' => 'pending', 'o1007' => 'completed', 'o1009' => 'completed'];
foreach ($orderStatuses as $key => $status) {
    $db->prepare("INSERT INTO wp_edd_orders (status, customer_id, date_created) VALUES (:status, :customer, :created)")
        ->execute([':status' => $status, ':customer' => $customerIds['c101'], ':created' => '2026-08-08T00:00:00Z']);
    $orderIds[$key] = (int) $db->lastInsertId();
}

$licenseIds = [];
$licenseSpecs = [
    'L1' => ['order' => 'o1001', 'status' => 'active'],
    'L2' => ['order' => 'o1002', 'status' => 'active'],
    'L3' => ['order' => 'o1003', 'status' => 'active'],
    'L4' => ['order' => 'o1004', 'status' => 'active'],
    'L5' => ['order' => null, 'status' => 'active'],
    'L7' => ['order' => 'o1007', 'status' => 'active'],
    'L8' => ['order' => 'o1009', 'status' => 'revoked'],
];
foreach ($licenseSpecs as $key => $spec) {
    $db->prepare("INSERT INTO wp_edd_licenses (license_key, customer_id, product_id, order_id, status, date_created)
        VALUES (:key, :customer, :product, :order, :status, :created)")
        ->execute([
            ':key' => strtoupper('FOCUSA-REC-') . str_pad((string) (count($licenseIds) + 1), 4, '0', STR_PAD_LEFT) . '-TESTKEY',
            ':customer' => $customerIds['c101'],
            ':product' => 2000 + count($licenseIds) + 1,
            ':order' => $spec['order'] === null ? null : $orderIds[$spec['order']],
            ':status' => $spec['status'],
            ':created' => '2026-08-08T00:00:00Z',
        ]);
    $licenseIds[$key] = (int) $db->lastInsertId();
}
$db->prepare("INSERT INTO wp_edd_subscriptions (customer_id, product_id, status, date_created) VALUES (:customer, 2009, 'cancelled', :created)")
    ->execute([':customer' => $customerIds['c101'], ':created' => '2026-08-08T00:00:00Z']);
$subscriptionIds = ['s2001' => (int) $db->lastInsertId()];
$db->prepare("INSERT INTO wp_edd_subscriptions (customer_id, product_id, status, date_created) VALUES (:customer, 2010, 'active', :created)")
    ->execute([':customer' => $customerIds['c101'], ':created' => '2026-08-08T00:00:00Z']);
$subscriptionIds['s2002'] = (int) $db->lastInsertId();

// Authority accounts: one healthy (c101), one missing (c102), a duplicate pair (c103),
// and a conflicting stripe-link pair (c104/c105). Fixed UUIDs keep the fixtures and the
// bounded result handle deterministic and replayable.
$accountUuids = [
    'A1' => '00000000-0000-4000-8000-0000000000a1',
    'A3a' => '00000000-0000-4000-8000-0000000003a0',
    'A4a' => '00000000-0000-4000-8000-0000000004a0',
    'A4b' => '00000000-0000-4000-8000-0000000004b0',
];
$provenance = FocusaSpec152eAuthorityAccountMigration::encodeProvenance(['source' => 'authority_reconciliation_test']);
$insertAccount = $db->prepare("INSERT INTO wp_wpuiai_authority_accounts
    (account_uuid, edd_customer_id, wordpress_user_id, stripe_customer_id, status, status_reason,
     highest_entitlement_sequence, migration_provenance, created_at, updated_at)
    VALUES (:uuid, :customer, NULL, :stripe, 'active', 'mailbox_verified', 0, :provenance, :created, :updated)");
foreach ([['A1', 'c101', 'cus_test_101'], ['A3a', 'c103', 'cus_test_103'], ['A4a', 'c104', 'cus_conflict'], ['A4b', 'c105', 'cus_conflict']] as [$key, $customerKey, $stripe]) {
    $insertAccount->execute([
        ':uuid' => $accountUuids[$key], ':customer' => $customerIds[$customerKey], ':stripe' => $stripe,
        ':provenance' => $provenance, ':created' => '2026-08-08T00:00:00Z', ':updated' => '2026-08-08T00:00:00Z',
    ]);
}
// Legacy install-site duplicate link (Spec 22.1 inventory): an install-site account
// record sharing the EDD customer already covered by authority account A3a. The
// authority registry forbids same-customer duplicates, so the duplicate vector is the
// legacy surface; it must quarantine for merge review, never auto-merge.
$db->exec("CREATE TABLE wp_install_site_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    edd_customer_id BIGINT NOT NULL,
    stripe_customer_id VARCHAR(191) NULL,
    install_account_ref VARCHAR(64) NOT NULL,
    migration_provenance TEXT NOT NULL
)");
$db->prepare("INSERT INTO wp_install_site_accounts (edd_customer_id, stripe_customer_id, install_account_ref, migration_provenance)
    VALUES (:customer, 'cus_test_103_legacy', 'install_103', :prov)")
    ->execute([':customer' => $customerIds['c103'], ':prov' => $provenance]);

/** Append one signed outbox event inside a caller-owned transaction (fixture helper). */
$appendEvent = static function (array $input) use ($db, $hook): void {
    $db->beginTransaction();
    try {
        $hook->append($input + ['request_id' => 'reconcile_fixture', 'idempotency_key' => 'pre:' . (string) ($input['event_type'] ?? 'unknown') . ':' . ($input['order_id'] ?? $input['license_id'] ?? $input['subscription_id'] ?? $input['lease_uuid'] ?? $input['node_uuid'] ?? 'x')]);
        $db->commit();
    } catch (Throwable $error) {
        if ($db->inTransaction()) {
            $db->rollBack();
        }
        throw $error;
    }
};

/** Pre-apply one canonical lifecycle projection (fixture helper). */
$projectEvent = static function (array $input) use ($projector): void {
    $input['request_id'] = 'reconcile_fixture';
    $input['idempotency_key'] = 'pre:' . $input['surface'] . ':' . ($input['order_id'] ?? $input['license_id'] ?? $input['subscription_id']) . ':' . $input['transition'];
    $result = $projector->project($input);
    if (($result['decision'] ?? '') === 'denied') {
        throw new RuntimeException('fixture projection denied: ' . ($result['error_code'] ?? 'none'));
    }
};

// Healthy and missing-callback fixtures on account A1.
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1001']]);
$appendEvent(['event_type' => 'order_completed', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1001']]);
// Missing refund/revoke/cancel callbacks: order complete was journaled, the terminal callback never fired.
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1002']]);
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1003']]);
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1004']]);
// o1007: canonical row says completed, but the projection is already terminal refunded (conflict fixture).
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1007']]);
$projectEvent(['surface' => 'order', 'transition' => 'refund', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1007']]);
// o1009: complete journaled with license scope; the license-revoke callback never fired.
$projectEvent(['surface' => 'order', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1009'], 'license_id' => $licenseIds['L8']]);
$appendEvent(['event_type' => 'order_completed', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'order_id' => $orderIds['o1009'], 'license_id' => $licenseIds['L8']]);
// s2001: subscription active was journaled; the cancel callback never fired.
$projectEvent(['surface' => 'subscription', 'transition' => 'complete', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'subscription_id' => $subscriptionIds['s2001']]);

// Verified-access nodes and postures (leases).
$nodeUuids = ['N401' => '00000000-0000-4000-8000-000000000401', 'N402' => '00000000-0000-4000-8000-000000000402'];
$postureUuids = ['P301' => '00000000-0000-4000-8000-000000000301', 'P302' => '00000000-0000-4000-8000-000000000302', 'P303' => '00000000-0000-4000-8000-000000000303', 'P304' => '00000000-0000-4000-8000-000000000304', 'P305' => '00000000-0000-4000-8000-000000000305'];
$insertNode = $db->prepare("INSERT INTO wp_wpuiai_verified_access_nodes (node_uuid, account_uuid, node_digest, registered_at, migration_provenance) VALUES (:uuid, :account, :digest, :at, :prov)");
foreach ($nodeUuids as $key => $uuid) {
    $insertNode->execute([':uuid' => $uuid, ':account' => $accountUuids['A1'], ':digest' => hash('sha256', $key), ':at' => '2026-08-08T00:00:00Z', ':prov' => $provenance]);
}
$insertPosture = $db->prepare("INSERT INTO wp_wpuiai_verified_access_postures
    (posture_uuid, account_uuid, identity_uuid, registration_uuid, product_scope, node_uuid, sequence,
     issued_at, refresh_at, signer, status, status_reason, migration_provenance, created_at, updated_at)
    VALUES (:uuid, :account, :identity, :registration, :scope, :node, :sequence, :issued, :refresh, :signer, :status, :reason, :prov, :created, :updated)");
$postureRows = [
    'P301' => ['node' => $nodeUuids['N401'], 'status' => 'issued', 'reason' => 'issued', 'at' => '2026-08-08T00:00:00Z'],
    'P302' => ['node' => $nodeUuids['N401'], 'status' => 'issued', 'reason' => 'issued', 'at' => '2026-08-08T00:01:00Z'],
    'P303' => ['node' => $nodeUuids['N402'], 'status' => 'issued', 'reason' => 'issued', 'at' => '2026-08-08T00:02:00Z'],
    'P304' => ['node' => $nodeUuids['N402'], 'status' => 'superseded', 'reason' => 'superseded', 'at' => '2026-08-08T00:03:00Z'],
    'P305' => ['node' => $nodeUuids['N401'], 'status' => 'revoked', 'reason' => 'revoked', 'at' => '2026-08-08T00:04:00Z'],
];
foreach ($postureRows as $key => $row) {
    $insertPosture->execute([
        ':uuid' => $postureUuids[$key], ':account' => $accountUuids['A1'],
        ':identity' => '00000000-0000-4000-8000-000000000001', ':registration' => '00000000-0000-4000-8000-000000000002',
        ':scope' => 'focusa', ':node' => $row['node'], ':sequence' => 1,
        ':issued' => '2026-08-08T00:00:00Z', ':refresh' => '2026-08-08T00:00:00Z', ':signer' => 'wpuiai.spec152e.lease.v1',
        ':status' => $row['status'], ':reason' => $row['reason'], ':prov' => $provenance,
        ':created' => $row['at'], ':updated' => $row['at'],
    ]);
}
// Signed issuance evidence for P301 (bound to L2) and P302 (bound to L3); P303 has none.
$appendEvent(['event_type' => 'lease_issued', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'lease_uuid' => $postureUuids['P301'], 'license_id' => $licenseIds['L2'], 'node_uuid' => $nodeUuids['N401']]);
$appendEvent(['event_type' => 'lease_issued', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'lease_uuid' => $postureUuids['P302'], 'license_id' => $licenseIds['L3'], 'node_uuid' => $nodeUuids['N401']]);
$appendEvent(['event_type' => 'lease_superseded', 'account_uuid' => $accountUuids['A1'], 'edd_customer_id' => $customerIds['c101'], 'lease_uuid' => $postureUuids['P304'], 'license_id' => $licenseIds['L1'], 'node_uuid' => $nodeUuids['N402']]);

$baseline = [
    'customers' => table_count($db, 'wp_edd_customers'),
    'orders' => table_count($db, 'wp_edd_orders'),
    'licenses' => table_count($db, 'wp_edd_licenses'),
    'subscriptions' => table_count($db, 'wp_edd_subscriptions'),
    'outbox' => table_count($db, 'wp_wpuiai_authority_outbox'),
    'lifecycle' => table_count($db, 'wp_wpuiai_edd_lifecycle_events'),
];
$baselineSequence = (int) $db->query("SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = '{$accountUuids['A1']}'")->fetchColumn();

// ── Discrepancy classifier ────────────────────────────────────────────

expect_reconcile(count(FocusaSpec152eDiscrepancyClassifier::CATEGORIES) === 11, 'classifier category registry is bounded at 11 categories');
expect_reconcile(count(FocusaSpec152eDiscrepancyClassifier::CLASSIFICATIONS) === 5, 'classifier classifications are bounded at 5');
expect_reconcile(FocusaSpec152eDiscrepancyClassifier::ENTITY_TYPES === ['customer', 'order', 'license', 'subscription', 'node', 'lease', 'account', 'refund'], 'entity types are exactly the eight reconcilable surfaces');
$classified = $classifier->classify([
    'category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => $postureUuids['P301'],
    'account_uuid' => $accountUuids['A1'], 'reason' => 'stale lease bound to terminal license must be superseded', 'evidence_ref' => 'verified_access_postures:lease_probe',
]);
expect_reconcile($classified['classification'] === 'repair_outbox' && $classified['severity'] === 'critical', 'classifier derives bounded classification and severity from the category registry');
expect_reconcile($classified['schema'] === FocusaSpec152eDiscrepancyClassifier::SCHEMA, 'classified finding carries the bounded schema');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'invented_category', 'entity_type' => 'order', 'entity_ref' => '1', 'reason' => 'r', 'evidence_ref' => 'e']), 'RECONCILIATION_CATEGORY_UNKNOWN', 'unknown category fails closed');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'invented', 'entity_ref' => '1', 'reason' => 'r', 'evidence_ref' => 'e']), 'RECONCILIATION_ENTITY_UNKNOWN', 'unknown entity type fails closed');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => '1', 'severity' => 'info', 'reason' => 'r', 'evidence_ref' => 'e']), 'RECONCILIATION_SEVERITY_MISMATCH', 'caller-selected severity cannot override the registry');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => '1', 'reason' => '', 'evidence_ref' => 'e']), 'RECONCILIATION_EVIDENCE_REQUIRED', 'empty reason fails closed');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => '1', 'reason' => str_repeat('x', 192), 'evidence_ref' => 'e']), 'RECONCILIATION_REASON_TOO_LONG', 'unbounded reason fails closed');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => '1', 'reason' => 'customer101@example.test must review', 'evidence_ref' => 'e']), 'RECONCILIATION_RAW_EMAIL_FORBIDDEN', 'raw email in a reason is forbidden');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => 'has spaces', 'reason' => 'r', 'evidence_ref' => 'e']), 'opaque entity reference required', 'non-opaque entity reference fails closed');
expect_reconcile_throws(static fn () => $classifier->classify(['category' => 'stale_lease', 'entity_type' => 'lease', 'entity_ref' => '1', 'reason' => 'r', 'evidence_ref' => 'not opaque !']), 'opaque evidence reference required', 'non-opaque evidence reference fails closed');

// ── Dry-run applies nothing ───────────────────────────────────────────

$dryRun = $reconciler->run('dry_run');
expect_reconcile(summary_value($dryRun, 'repairs_applied') === 0, 'dry-run applies zero repairs');
expect_reconcile(summary_value($dryRun, 'would_repair') === summary_value($dryRun, 'repairable') && summary_value($dryRun, 'repairable') > 0, 'dry-run reports the exact would-be repair set');
expect_reconcile(summary_value($dryRun, 'converged') === false, 'dry-run does not claim convergence before repairs');
expect_reconcile(summary_value($dryRun, 'findings_total') === 25, 'dry-run reports exactly 25 bounded findings across all surfaces');
expect_reconcile(summary_value($dryRun, 'quarantined_new') === 7, 'dry-run quarantines exactly the 7 ambiguous/conflicting records');
expect_reconcile(table_count($db, 'wp_wpuiai_authority_outbox') === $baseline['outbox'], 'dry-run appends no outbox events');
expect_reconcile(table_count($db, 'wp_wpuiai_edd_lifecycle_events') === $baseline['lifecycle'], 'dry-run applies no lifecycle projections');
expect_reconcile((int) $db->query("SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = '{$accountUuids['A1']}'")->fetchColumn() === $baselineSequence, 'dry-run never advances the authority sequence');

// ── Apply converges every safe fixture ────────────────────────────────

$applyRun = $reconciler->run('apply');
expect_reconcile(summary_value($applyRun, 'findings_total') === 25, 'apply run reports the same bounded findings before repair');
expect_reconcile(summary_value($applyRun, 'repairable') === 18, 'apply run schedules exactly 18 evidence-safe repairs');
expect_reconcile(summary_value($applyRun, 'repairs_applied') === 18, 'apply run applies exactly 18 repairs');
expect_reconcile(summary_value($applyRun, 'quarantined_new') === 7, 'apply run quarantines exactly the 7 ambiguous/conflicting records');
expect_reconcile(summary_value($applyRun, 'converged') === true, 'apply run converges: no repairable finding remains');
expect_reconcile(count_classification($applyRun, 'repair_projection') === 7, 'exactly 7 lifecycle projections are repaired');
expect_reconcile(count_classification($applyRun, 'repair_outbox') === 11, 'exactly 11 signed outbox events are appended');
$outboxAfterApply = table_count($db, 'wp_wpuiai_authority_outbox');
$lifecycleAfterApply = table_count($db, 'wp_wpuiai_edd_lifecycle_events');
expect_reconcile($outboxAfterApply === $baseline['outbox'] + 11, 'outbox grows by exactly the 11 appended signed events');
expect_reconcile($lifecycleAfterApply === $baseline['lifecycle'] + 7, 'lifecycle journal grows by exactly the 7 applied projections');
$sequenceAfterApply = (int) $db->query("SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = '{$accountUuids['A1']}'")->fetchColumn();
expect_reconcile($sequenceAfterApply === $baselineSequence + 7, 'authority sequence advanced by exactly 7 (strictly monotonic, never rolled back)');

// Repaired projection truth is visible and durable.
$refundProjection = $db->prepare("SELECT license_state FROM wp_wpuiai_edd_lifecycle_events WHERE order_id = :order AND decision IN ('applied','replayed') ORDER BY result_sequence DESC LIMIT 1");
$refundProjection->execute([':order' => $orderIds['o1002']]);
expect_reconcile((string) $refundProjection->fetchColumn() === 'refunded', 'refunded order projection is repaired to the canonical terminal state');
$revokeProjection = $db->prepare("SELECT license_state FROM wp_wpuiai_edd_lifecycle_events WHERE license_id = :license AND decision IN ('applied','replayed') ORDER BY result_sequence DESC LIMIT 1");
$revokeProjection->execute([':license' => $licenseIds['L8']]);
expect_reconcile((string) $revokeProjection->fetchColumn() === 'revoked', 'revoked license projection is repaired to the canonical terminal state');
$cancelProjection = $db->prepare("SELECT license_state FROM wp_wpuiai_edd_lifecycle_events WHERE subscription_id = :sub AND decision IN ('applied','replayed') ORDER BY result_sequence DESC LIMIT 1");
$cancelProjection->execute([':sub' => $subscriptionIds['s2001']]);
expect_reconcile((string) $cancelProjection->fetchColumn() === 'cancelled', 'cancelled subscription projection is repaired to the canonical terminal state');

// Signed outbox events were appended with opaque refs only.
$leaseEvent = $db->prepare("SELECT event_type, license_id, lease_uuid FROM wp_wpuiai_authority_outbox WHERE lease_uuid = :lease AND event_type IN ('lease_superseded','lease_revoked') ORDER BY created_at DESC LIMIT 1");
$leaseEvent->execute([':lease' => $postureUuids['P301']]);
$leaseRow = $leaseEvent->fetch(PDO::FETCH_ASSOC);
expect_reconcile($leaseRow !== false && $leaseRow['event_type'] === 'lease_superseded' && (int) $leaseRow['license_id'] === $licenseIds['L2'], 'stale lease bound to a refunded license is superseded with an exact signed event');
$leaseEvent->execute([':lease' => $postureUuids['P302']]);
$leaseRow = $leaseEvent->fetch(PDO::FETCH_ASSOC);
expect_reconcile($leaseRow !== false && $leaseRow['event_type'] === 'lease_revoked' && (int) $leaseRow['license_id'] === $licenseIds['L3'], 'stale lease bound to a revoked license is revoked with an exact signed event');
$nodeEvent = $db->prepare("SELECT event_type, node_uuid FROM wp_wpuiai_authority_outbox WHERE node_uuid = :node AND event_type = 'node_deactivated' LIMIT 1");
$nodeEvent->execute([':node' => $nodeUuids['N401']]);
expect_reconcile($nodeEvent->fetchColumn() !== false, 'stale node with a terminal posture is deactivated with a signed event');

// ── Repeated runs converge and quarantine stays stable ───────────────

$secondApply = $reconciler->run('apply');
expect_reconcile(summary_value($secondApply, 'repairable') === 0, 'second apply finds zero repairable discrepancies');
expect_reconcile(summary_value($secondApply, 'repairs_applied') === 0, 'second apply repairs nothing (idempotent)');
expect_reconcile(summary_value($secondApply, 'quarantined_new') === 0, 'second apply adds no duplicate quarantine rows');
expect_reconcile(summary_value($secondApply, 'stable_quarantine') === 7, 'second apply acknowledges the 7 stable quarantine records with exact reasons');
expect_reconcile(summary_value($secondApply, 'converged') === true, 'second apply converges with only the stable quarantine set');

$postApplyDryRun = $reconciler->run('dry_run');
expect_reconcile(summary_value($postApplyDryRun, 'repairable') === 0 && summary_value($postApplyDryRun, 'would_repair') === 0, 'dry-run after apply reports zero would-be repairs (converged)');
expect_reconcile(summary_value($postApplyDryRun, 'stable_quarantine') === 7, 'dry-run after apply sees the identical stable quarantine set');
expect_reconcile(summary_value($postApplyDryRun, 'converged') === true, 'post-apply dry-run converges');

// Exact quarantine reasons are preserved and stable.
$quarantineRows = $db->query('SELECT entity_type, entity_ref, reason FROM wp_wpuiai_reconciliation_quarantine ORDER BY entity_ref')->fetchAll(PDO::FETCH_ASSOC);
expect_reconcile(count($quarantineRows) === 7, 'quarantine table holds exactly the 7 records');
$reasons = array_column($quarantineRows, 'reason');
$expectedReasons = [
    'verified mailbox plus evidence-backed purchase linkage required; raw email match alone never transfers ownership',
    'ACCOUNT_MERGE_REVIEW_REQUIRED legacy install-site record duplicates an authority account link',
    'ACCOUNT_MERGE_REVIEW_REQUIRED shared stripe customer link conflicts across accounts',
    'LICENSE_TERMINAL_REACTIVATION_DENIED canonical row would reactivate a terminal projection; operator review required',
    'synthetic record quarantined unless separately approved; verified identity plus matching order item required',
    'lease has no signed issuance event; operator review required',
];
foreach ($expectedReasons as $expectedReason) {
    expect_reconcile(in_array($expectedReason, $reasons, true), "quarantine holds exact reason: {$expectedReason}");
}

// ── Terminal truth is never rolled back and never reactivated ─────────

$terminalConflict = null;
foreach ($secondApply['quarantine'] as $row) {
    if ($row['entity_ref'] === (string) $orderIds['o1007']) {
        $terminalConflict = $row;
        break;
    }
}
expect_reconcile($terminalConflict !== null, 'terminal-reactivation conflict is quarantined with an exact reason');
expect_reconcile($terminalConflict !== null && str_contains($terminalConflict['reason'], 'LICENSE_TERMINAL_REACTIVATION_DENIED'), 'terminal-reactivation conflict reason names the fail-closed code');
$o1007Projection = $db->prepare("SELECT license_state FROM wp_wpuiai_edd_lifecycle_events WHERE order_id = :order AND decision IN ('applied','replayed') ORDER BY result_sequence DESC LIMIT 1");
$o1007Projection->execute([':order' => $orderIds['o1007']]);
expect_reconcile((string) $o1007Projection->fetchColumn() === 'refunded', 'conflicted projection stays terminal refunded; the canonical completed row is never reactivated');
expect_reconcile((int) $db->query("SELECT highest_entitlement_sequence FROM wp_wpuiai_authority_accounts WHERE account_uuid = '{$accountUuids['A1']}'")->fetchColumn() === $sequenceAfterApply, 'post-conflict runs never change the authority sequence');
expect_reconcile((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_reconciliation_quarantine")->fetchColumn() === 7, 'quarantine is preservation-only: no quarantine row is ever deleted');

// Canonical EDD truth is preserved byte-for-byte.
expect_reconcile(table_count($db, 'wp_edd_customers') === $baseline['customers'], 'customer truth preserved');
expect_reconcile(table_count($db, 'wp_edd_orders') === $baseline['orders'], 'order truth preserved');
expect_reconcile(table_count($db, 'wp_edd_licenses') === $baseline['licenses'], 'license truth preserved');
expect_reconcile(table_count($db, 'wp_edd_subscriptions') === $baseline['subscriptions'], 'subscription truth preserved');
expect_reconcile((string) $db->query("SELECT status FROM wp_edd_orders WHERE id = {$orderIds['o1002']}")->fetchColumn() === 'refunded', 'refund truth on the canonical row is untouched');
expect_reconcile((string) $db->query("SELECT status FROM wp_edd_licenses WHERE id = {$licenseIds['L8']}")->fetchColumn() === 'revoked', 'revoke truth on the canonical row is untouched');

// ── Bounded scope and fail-closed negatives ───────────────────────────

$customerScope = $reconciler->run('apply', ['customer']);
expect_reconcile(summary_value($customerScope, 'repairable') === 0, 'customer-only scope never repairs');
expect_reconcile(summary_value($customerScope, 'stable_quarantine') === 4, 'customer-only scope sees exactly the 4 account-link quarantines');
$licenseScope = $reconciler->run('dry_run', ['license']);
expect_reconcile(summary_value($licenseScope, 'repairable') === 0, 'license scope after convergence has no repairable findings');
expect_reconcile_throws(static fn () => $reconciler->run('invented_mode'), 'RECONCILIATION_MODE_UNKNOWN', 'unknown reconciliation mode fails closed');
expect_reconcile_throws(static fn () => $reconciler->run('apply', ['invented_surface']), 'RECONCILIATION_SCOPE_UNKNOWN', 'unknown scope surface fails closed');
expect_reconcile_throws(static fn () => $reconciler->run('apply', ['price' => 9.99]), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled commerce fields in scope are forbidden');
expect_reconcile_throws(static fn () => $reconciler->run('apply', ['grants' => ['focusa']]), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'client-controlled grants in scope are forbidden');

// ── Redaction: no email, no secrets, no key material anywhere ─────────

$allStoredJson = '';
foreach (['wp_wpuiai_reconciliation_runs', 'wp_wpuiai_reconciliation_findings', 'wp_wpuiai_reconciliation_repairs', 'wp_wpuiai_reconciliation_quarantine'] as $table) {
    $allStoredJson .= json_encode($db->query("SELECT * FROM {$table}")->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR);
}
foreach ([$dryRun, $applyRun, $secondApply, $postApplyDryRun] as $report) {
    $allStoredJson .= json_encode($report, JSON_THROW_ON_ERROR);
}
expect_reconcile(strpos($allStoredJson, '@') === false, 'no raw email in any reconciliation storage or report');
expect_reconcile(strpos($allStoredJson, 'cus_test_101') === false, 'no payment/customer secret in reconciliation storage or reports');
expect_reconcile(strpos($allStoredJson, 'FOCUSA-REC-') === false, 'no license key material in reconciliation storage or reports');
expect_reconcile(strpos($allStoredJson, 'example.test') === false, 'no unmasked email evidence in reconciliation storage or reports');
expect_reconcile(strpos($allStoredJson, $secret) === false, 'server signing secret never leaves the signer');
expect_reconcile(strpos($allStoredJson, $provenance) === false, 'migration provenance strings never leak into findings or reports');

// ── Deterministic immutable result handle ─────────────────────────────

$handleA = $reconciler->run('dry_run')['result_handle'];
$handleB = $reconciler->run('dry_run')['result_handle'];
expect_reconcile($handleA === $handleB, 'result handle is deterministic across identical runs');
expect_reconcile(preg_match('/^[0-9a-f]{64}$/D', $handleA) === 1, 'result handle is a bounded SHA-256');
$storedHandle = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_reconciliation_runs WHERE result_handle = '{$handleA}'")->fetchColumn();
expect_reconcile($storedHandle >= 1, 'every run persists its immutable result handle');

// Rollback preservation: runs, findings, repairs, and quarantine survive.
$preserved = $reconciliationMigration->preserveForRollback('2026-08-08T00:02:00Z', ['source' => 'authority_reconciliation_test']);
expect_reconcile($preserved['action'] === 'preserve' && $preserved['event_key'] !== '', 'rollback is preservation-only (schema event recorded)');
expect_reconcile(table_count($db, 'wp_wpuiai_reconciliation_runs') >= 6, 'reconciliation runs are never deleted by rollback');
expect_reconcile(table_count($db, 'wp_wpuiai_reconciliation_findings') >= 1 && table_count($db, 'wp_wpuiai_reconciliation_repairs') >= 1, 'findings and repair journals are never deleted by rollback');

// ── Summary ───────────────────────────────────────────────────────────

$summary = [
    'schema' => 'focusa.spec152e.authority_reconciliation_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'categories' => count(FocusaSpec152eDiscrepancyClassifier::CATEGORIES),
    'classifications' => count(FocusaSpec152eDiscrepancyClassifier::CLASSIFICATIONS),
    'entity_types' => count(FocusaSpec152eDiscrepancyClassifier::ENTITY_TYPES),
    'dry_run_findings' => summary_value($dryRun, 'findings_total'),
    'dry_run_would_repair' => summary_value($dryRun, 'would_repair'),
    'dry_run_applied' => summary_value($dryRun, 'repairs_applied'),
    'apply_findings' => summary_value($applyRun, 'findings_total'),
    'repairable' => summary_value($applyRun, 'repairable'),
    'repairs_applied' => summary_value($applyRun, 'repairs_applied'),
    'projection_repairs' => count_classification($applyRun, 'repair_projection'),
    'outbox_repairs' => count_classification($applyRun, 'repair_outbox'),
    'quarantined_new' => summary_value($applyRun, 'quarantined_new'),
    'stable_quarantine' => summary_value($secondApply, 'stable_quarantine'),
    'converged_apply1' => summary_value($applyRun, 'converged'),
    'converged_apply2' => summary_value($secondApply, 'converged'),
    'converged_dry_after' => summary_value($postApplyDryRun, 'converged'),
    'sequence_advance' => $sequenceAfterApply - $baselineSequence,
    'outbox_appended' => $outboxAfterApply - $baseline['outbox'],
    'lifecycle_applied' => $lifecycleAfterApply - $baseline['lifecycle'],
    'terminal_conflict_quarantined' => $terminalConflict !== null,
    'canonical_truth_preserved' => ['customers', 'orders', 'licenses', 'subscriptions'],
    'storage' => 'opaque_refs_only_no_email_no_secrets_no_keys',
    'result_handle' => $handleA,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
