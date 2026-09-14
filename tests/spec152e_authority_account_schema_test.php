<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-authority-account.v1.php';

function expect_account(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_throws_account(callable $operation, string $exception, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect_account($error instanceof $exception, $message . ' exception type');
        return;
    }
    expect_account(false, $message);
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$migration = new FocusaSpec152eAuthorityAccountMigration($db, 'wp_');
$migration->migrate('2026-08-07T04:40:00Z', [
    'source' => 'candidate_repository',
    'work_item' => 'focusa-vbcqu.20.13.5',
]);
$migration->migrate('2026-08-07T04:41:00Z', [
    'source' => 'repeat_must_not_replace_first_application',
]);

$migrationRows = $db->query('SELECT * FROM wp_wpuiai_authority_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_account(count($migrationRows) === 1, 'repeated migration records one schema version');
expect_account($migrationRows[0]['applied_at'] === '2026-08-07T04:40:00Z', 'repeated migration preserves first applied timestamp');
expect_account(str_contains($migrationRows[0]['migration_provenance'], 'candidate_repository'), 'repeated migration preserves first provenance');

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-07T04:42:00Z'))
        ->modify('+' . $clockTick . ' minutes')
        ->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
};
$repository = new FocusaSpec152eAuthorityAccountRepository($db, $migration, $clock);

$pending = [
    'verification_state' => 'email_verification_pending',
    'verified_at' => null,
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'edd_customer_id' => 41001,
    'idempotency_key' => 'idem-pending-0001',
    'migration_provenance' => ['source' => 'synthetic_pending'],
];
expect_throws_account(
    static fn() => $repository->promoteVerified($pending),
    DomainException::class,
    'unverified pending attempt is denied'
);
expect_account((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn() === 0, 'pending denial creates no account');
expect_account((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_account_idempotency')->fetchColumn() === 0, 'pending denial consumes no idempotency key');

$verified = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T04:41:30Z',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'edd_customer_id' => 41001,
    'wordpress_user_id' => 501,
    'stripe_customer_id' => 'cus_synthetic_authority_001',
    'idempotency_key' => 'idem-promote-0001',
    'migration_provenance' => [
        'source' => 'spec152e_candidate_migration',
        'source_record' => 'synthetic-edd-customer-41001',
    ],
];
$created = $repository->promoteVerified($verified);
expect_account($created['account_uuid'] === $verified['account_uuid'], 'verified promotion creates requested opaque account');
expect_account((int) $created['edd_customer_id'] === 41001, 'account links canonical EDD customer');
expect_account((int) $created['highest_entitlement_sequence'] === 0, 'account starts at sequence zero');
expect_account($created['created_at'] === '2026-08-07T04:42:00Z', 'created timestamp comes from authority clock');
expect_account($created['updated_at'] === $created['created_at'], 'initial timestamps are equal');
expect_account(str_contains($created['migration_provenance'], 'synthetic-edd-customer-41001'), 'account retains migration provenance');

$replayed = $repository->promoteVerified($verified);
expect_account($replayed === $created, 'identical promotion replay is idempotent');
expect_account((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn() === 1, 'promotion replay creates no duplicate');

$secondResolution = $verified;
$secondResolution['account_uuid'] = '018f47c2-6ac0-7b16-8d1a-4e93df5a0102';
$secondResolution['idempotency_key'] = 'idem-promote-0002';
$resolved = $repository->promoteVerified($secondResolution);
expect_account($resolved['account_uuid'] === $created['account_uuid'], 'one EDD customer resolves to one authority account');
expect_account((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts WHERE edd_customer_id = 41001')->fetchColumn() === 1, 'EDD customer/account uniqueness is enforced');

$conflict = $verified;
$conflict['edd_customer_id'] = 41002;
expect_throws_account(
    static fn() => $repository->promoteVerified($conflict),
    DomainException::class,
    'changed request cannot reuse promotion idempotency key'
);
expect_account((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_accounts')->fetchColumn() === 1, 'idempotency conflict is atomic');

$advanced = $repository->advanceSequence($created['account_uuid'], 7, 'idem-sequence-0001');
expect_account((int) $advanced['highest_entitlement_sequence'] === 7, 'sequence advances monotonically');
expect_account($advanced['created_at'] === $created['created_at'], 'sequence advance preserves created timestamp');
expect_account($advanced['updated_at'] === '2026-08-07T04:44:00Z', 'sequence advance updates authority timestamp');
$advancedReplay = $repository->advanceSequence($created['account_uuid'], 7, 'idem-sequence-0001');
expect_account((int) $advancedReplay['highest_entitlement_sequence'] === 7, 'sequence replay is idempotent');
expect_throws_account(
    static fn() => $repository->advanceSequence($created['account_uuid'], 6, 'idem-sequence-0002'),
    DomainException::class,
    'sequence rollback is denied'
);

$beforeRollback = $repository->findByUuid($created['account_uuid']);
$rollback = $migration->preserveForRollback('2026-08-07T04:50:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_rollback_proof',
]);
$afterRollback = $repository->findByUuid($created['account_uuid']);
expect_account($rollback['action'] === 'preserve', 'rollback contract is preservation-only');
expect_account($afterRollback === $beforeRollback, 'rollback preserves account, EDD link, sequence, provenance, and timestamps');
expect_account((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.authority_account_schema_validation.v1',
    'migration_versions' => count($migrationRows),
    'authority_accounts' => 1,
    'highest_sequence' => 7,
    'assertion_groups' => 7,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
