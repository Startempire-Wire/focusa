<?php
declare(strict_types=1);

require_once dirname(__DIR__) . '/docs/contracts/spec152e-activation-registration.v1.php';

function expect_registration_schema(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_registration_schema_throws(callable $operation, string $message): void
{
    try {
        $operation();
    } catch (Throwable) {
        return;
    }
    expect_registration_schema(false, $message);
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$migration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$migration->migrate('2026-08-07T05:00:00Z', [
    'source' => 'candidate_registration_repository',
    'work_item' => 'focusa-vbcqu.20.13.7',
]);
$migration->migrate('2026-08-07T05:01:00Z', [
    'source' => 'repeat_must_preserve_first_schema_application',
]);

$migrations = $db->query('SELECT * FROM wp_wpuiai_activation_registration_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_registration_schema(count($migrations) === 1, 'migration is version-idempotent');
expect_registration_schema($migrations[0]['applied_at'] === '2026-08-07T05:00:00Z', 'migration preserves first application time');
expect_registration_schema(str_contains($migrations[0]['migration_provenance'], 'candidate_registration_repository'), 'migration preserves first provenance');

$columns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_activation_registrations)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $columns[$column['name']] = $column;
}
$required = [
    'registration_uuid', 'account_uuid', 'edd_customer_id', 'facade_id', 'presenter', 'install_channel',
    'product_code', 'safe_redirect_handle', 'state', 'state_reason', 'state_version',
    'encrypted_normalized_email', 'email_lookup_digest', 'verification_state', 'verification_challenge_hash',
    'verification_challenge_issued_at', 'verification_challenge_expires_at', 'verification_attempts', 'verified_at',
    'offer_code', 'journey', 'edd_cart_reference', 'edd_order_id', 'edd_order_item_id', 'edd_license_id',
    'node_uuid', 'device_public_key', 'poll_credential_hash', 'poll_credential_issued_at',
    'poll_credential_expires_at', 'terminal_delivery_status', 'delivery_attempts', 'delivery_ready_at',
    'delivered_at', 'delivery_failure_reason', 'request_id', 'idempotency_key', 'request_digest',
    'created_at', 'expires_at', 'settled_at', 'updated_at',
];
foreach ($required as $field) {
    expect_registration_schema(isset($columns[$field]), "registration schema contains {$field}");
}
expect_registration_schema($columns['encrypted_normalized_email']['notnull'] === 1, 'encrypted email is required');
expect_registration_schema($columns['email_lookup_digest']['notnull'] === 1, 'email lookup digest is required');
expect_registration_schema($columns['verification_challenge_hash']['notnull'] === 0, 'verification hash is nullable after single use');
expect_registration_schema($columns['poll_credential_hash']['notnull'] === 0, 'poll hash is nullable for delivery-safe records');

$transitionColumns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_activation_registration_transitions)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $transitionColumns[$column['name']] = true;
}
foreach (['registration_uuid', 'from_state', 'to_state', 'expected_version', 'result_version', 'request_id', 'idempotency_key', 'transition_digest', 'occurred_at', 'retention_until'] as $field) {
    expect_registration_schema(isset($transitionColumns[$field]), "transition journal contains {$field}");
}
$idempotencyColumns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_activation_registration_idempotency)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $idempotencyColumns[$column['name']] = true;
}
foreach (['idempotency_key', 'operation', 'registration_uuid', 'request_id', 'request_digest', 'result_state', 'result_version', 'created_at', 'retention_until'] as $field) {
    expect_registration_schema(isset($idempotencyColumns[$field]), "idempotency journal contains {$field}");
}

$secretStore = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);
$clock = static fn(): string => '2026-08-07T05:02:00Z';
$repository = new FocusaSpec152eActivationRegistrationRepository($db, $migration, $secretStore, $clock);
$created = $repository->createPending([
    'email' => 'synthetic.operator@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'safe_redirect_handle' => 'success',
    'request_id' => 'req-schema-0001',
    'idempotency_key' => 'idem-schema-0001',
]);
$row = $created['registration'];
expect_registration_schema($row['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'new registration sends a verification challenge');
expect_registration_schema($row['account_uuid'] === null && $row['edd_customer_id'] === null, 'pending registration has no account or EDD customer');
expect_registration_schema($row['edd_order_id'] === null && $row['edd_license_id'] === null && $row['node_uuid'] === null, 'pending registration has no commerce, node, or entitlement reference');
expect_registration_schema($row['encrypted_normalized_email'] !== 'synthetic.operator@example.invalid', 'email is not stored in plaintext');
expect_registration_schema(strlen($row['encrypted_normalized_email']) > 32, 'encrypted email has an authenticated envelope');
expect_registration_schema((bool) preg_match('/^[a-f0-9]{64}$/D', $row['email_lookup_digest']), 'email lookup is a keyed digest');
expect_registration_schema((bool) preg_match('/^[a-f0-9]{64}$/D', $row['verification_challenge_hash']), 'verification secret is stored only as a hash');
expect_registration_schema((bool) preg_match('/^[a-f0-9]{64}$/D', $row['poll_credential_hash']), 'poll credential is stored only as a hash');
expect_registration_schema($row['verification_challenge_hash'] !== $created['verification_secret'], 'verification plaintext is absent from storage');
expect_registration_schema($row['poll_credential_hash'] !== $created['poll_credential'], 'poll plaintext is absent from storage');
expect_registration_schema($secretStore->decryptEmail($row['encrypted_normalized_email']) === 'synthetic.operator@example.invalid', 'encrypted email decrypts only with the injected key');
expect_registration_schema($row['expires_at'] === '2026-08-07T05:32:00Z', 'registration TTL is bounded');
expect_registration_schema($row['verification_challenge_expires_at'] === '2026-08-07T05:17:00Z', 'verification TTL is bounded separately');
expect_registration_schema($row['poll_credential_expires_at'] === '2026-08-07T05:32:00Z', 'poll TTL is bounded by registration TTL');
expect_registration_schema_throws(static function () use ($db, $row): void {
    $statement = $db->prepare('UPDATE wp_wpuiai_activation_registrations SET edd_order_id = 9001 WHERE registration_uuid = :registration');
    $statement->execute([':registration' => $row['registration_uuid']]);
}, 'database constraints reject commerce references on pending records');

$stored = $repository->findByUuid($row['registration_uuid']);
$public = FocusaSpec152eActivationRegistrationPresenter::snapshot($stored);
expect_registration_schema(!isset($public['encrypted_normalized_email']), 'presenter does not expose encrypted identity');
expect_registration_schema(!isset($public['email_lookup_digest'], $public['verification_challenge_hash'], $public['poll_credential_hash']), 'presenter does not expose lookup or credential hashes');
expect_registration_schema(!isset($public['edd_customer_id'], $public['edd_order_id'], $public['edd_license_id']), 'presenter does not expose EDD internals');
expect_registration_schema($public['registration_id'] === $row['registration_uuid'], 'presenter returns only the opaque registration handle');

$rollbackBefore = $repository->findByUuid($row['registration_uuid']);
$rollback = $migration->preserveForRollback('2026-08-07T05:03:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_registration_rollback',
]);
$rollbackAfter = $repository->findByUuid($row['registration_uuid']);
expect_registration_schema($rollback['action'] === 'preserve', 'rollback is preservation-only');
expect_registration_schema($rollbackAfter === $rollbackBefore, 'rollback preserves registration and credential hashes');
expect_registration_schema((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_activation_registration_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback is journaled');

expect_registration_schema_throws(
    static fn() => $repository->createPending([
        'email' => 'synthetic.operator@example.invalid',
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'terminal',
        'install_channel' => 'source_build',
        'product_code' => 'focusa_operator',
        'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
        'request_id' => 'req-schema-0002',
        'idempotency_key' => 'idem-schema-0002',
    ]),
    'pending creation rejects caller-supplied account authority'
);

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.registration_schema_validation.v1',
    'registration_state' => $row['state'],
    'required_columns' => count($required),
    'transition_columns' => count($transitionColumns),
    'idempotency_columns' => count($idempotencyColumns),
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
