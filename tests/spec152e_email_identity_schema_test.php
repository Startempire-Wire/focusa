<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';

function expect_identity(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_throws_identity(callable $operation, string $exception, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect_identity($error instanceof $exception, $message . ' exception type');
        return;
    }
    expect_identity(false, $message);
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$migration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$migration->migrate('2026-08-07T05:00:00Z', ['source' => 'synthetic_fixture', 'work_item' => 'focusa-vbcqu.20.13.6']);
$migration->migrate('2026-08-07T05:01:00Z', ['source' => 'replay']);
$rows = $db->query('SELECT * FROM wp_wpuiai_email_identity_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_identity(count($rows) === 1, 'migration is idempotent');
expect_identity($rows[0]['applied_at'] === '2026-08-07T05:00:00Z', 'migration preserves first application');

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    return (new DateTimeImmutable('2026-08-07T05:02:00Z'))
        ->modify('+' . $clockTick++ . ' minutes')->format('Y-m-d\TH:i:s\Z');
};
$secrets = new FocusaSpec152eEmailIdentitySecrets(str_repeat('e', 32), str_repeat('l', 64));
$repository = new FocusaSpec152eEmailIdentityRepository($db, $migration, $secrets, $clock);
$base = [
    'verification_state' => 'mailbox_verified',
    'verified_at' => '2026-08-07T05:01:30Z',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0201',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0202',
    'identity_state' => 'primary', 'verification_method' => 'magic_link',
    'transactional_consent_at' => '2026-08-07T05:01:40Z',
    'promotional_consent_at' => null, 'promotional_consent_revoked_at' => null,
    'source' => 'synthetic.fixture', 'migration_evidence' => ['record' => 'synthetic-identity-001'],
];
$stored = $repository->storeVerified(" dot.tag+one@EXAMPLE.invalid ", $base);
$replay = $repository->storeVerified('dot.tag+one@example.invalid', $base);
expect_identity($replay === $stored, 'equivalent canonical input is idempotent');
expect_identity($repository->findExact('dot.tag+one@example.invalid') === $stored, 'exact keyed lookup resolves identity');
expect_identity($repository->findExact('dottagone@example.invalid') === null, 'provider dot collapsing is forbidden');
expect_identity($repository->findExact('dot.tag+two@example.invalid') === null, 'provider plus collapsing is forbidden');
expect_identity(!array_key_exists('encrypted_normalized_email', $stored) && !array_key_exists('email_lookup_digest', $stored), 'safe result excludes secret identity fields');
expect_identity((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities')->fetchColumn() === 1, 'replay creates no duplicate');

$linked = $base;
$linked['identity_uuid'] = '018f47c2-6ac0-7b16-8d1a-4e93df5a0203';
$linked['identity_state'] = 'linked';
$linked['promotional_consent_at'] = '2026-08-07T05:01:50Z';
$repository->storeVerified('dot.tag+two@example.invalid', $linked);
expect_identity((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_email_identities WHERE account_uuid = '018f47c2-6ac0-7b16-8d1a-4e93df5a0201'")->fetchColumn() === 2, 'distinct valid aliases remain distinct');
$anotherPrimary = $base;
$anotherPrimary['identity_uuid'] = '018f47c2-6ac0-7b16-8d1a-4e93df5a0204';
expect_throws_identity(static fn() => $repository->storeVerified('third@example.invalid', $anotherPrimary), PDOException::class, 'one primary identity per account is enforced');

$pending = $base;
$pending['identity_uuid'] = '018f47c2-6ac0-7b16-8d1a-4e93df5a0205';
$pending['verification_state'] = 'email_verification_pending';
expect_throws_identity(static fn() => $repository->storeVerified('pending@example.invalid', $pending), DomainException::class, 'unverified identity cannot be promoted');
expect_identity((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_email_identities WHERE email_lookup_digest IS NOT NULL')->fetchColumn() === 2, 'unverified identity is not persisted');
$delivery = $repository->recordDeliveryState($linked['identity_uuid'], 'hard', 'promotional', '2026-08-07T05:03:00Z');
expect_identity($delivery['bounce_state'] === 'hard' && $delivery['suppression_state'] === 'promotional', 'bounce and suppression state are recorded');
expect_identity($delivery['transactional_consent_at'] !== null && $delivery['promotional_consent_at'] !== null, 'transactional and promotional consent timestamps remain separate');
expect_identity($repository->revealForAuthenticatedWorkflow($base['identity_uuid']) === 'dot.tag+one@example.invalid', 'encrypted identity decrypts only through explicit workflow');
$beforeRollback = $repository->findExact('dot.tag+one@example.invalid');
$rollback = $migration->preserveForRollback('2026-08-07T05:04:00Z', ['reason' => 'synthetic_rollback_proof']);
expect_identity($rollback['action'] === 'preserve', 'rollback is preservation-only');
expect_identity($repository->findExact('dot.tag+one@example.invalid') === $beforeRollback, 'rollback preserves verified identity');
expect_identity((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_email_identity_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.email_identity_schema_validation.v1',
    'migration_versions' => 1, 'verified_identities' => 2,
    'assertion_groups' => 12, 'raw_email_logging' => false, 'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
