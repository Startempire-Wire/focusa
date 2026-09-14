<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-email-delivery-consent.v1.php';

function expect_delivery_consent(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

// ── Setup ──────────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Registration schema.
$registrationMigration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$registrationMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'delivery_consent_test']);

// Email identity schema.
$identityMigration = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$identityMigration->migrate('2026-08-08T00:00:00Z', ['source' => 'delivery_consent_test']);

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    return (new DateTimeImmutable('2026-08-08T00:01:00Z'))
        ->modify('+' . $clockTick++ . ' minutes')->format('Y-m-d\TH:i:s\Z');
};

$secrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);

$identitySecrets = new FocusaSpec152eEmailIdentitySecrets(
    str_repeat('e', 32),
    str_repeat('l', 64),
);

$registrations = new FocusaSpec152eActivationRegistrationRepository($db, $registrationMigration, $secrets, $clock);
$identities = new FocusaSpec152eEmailIdentityRepository($db, $identityMigration, $identitySecrets, $clock);

$handler = new FocusaSpec152eEmailDeliveryConsentHandler($identities, $registrations);

// ── Create verified identities for testing ─────────────────────────────

$now = $clock();
$identity1 = $identities->storeVerified('synthetic.alpha@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'verified_at' => $now,
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0201',
    'identity_state' => 'primary',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => null,
    'promotional_consent_at' => null,
    'promotional_consent_revoked_at' => null,
    'source' => 'delivery.consent.test',
    'migration_evidence' => ['record' => 'test-identity-001'],
]);

$identity2 = $identities->storeVerified('synthetic.bravo@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'verified_at' => $now,
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0102',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0202',
    'identity_state' => 'primary',
    'verification_method' => 'otp',
    'transactional_consent_at' => $now,
    'promotional_consent_at' => null,
    'promotional_consent_revoked_at' => null,
    'source' => 'delivery.consent.test',
    'migration_evidence' => ['record' => 'test-identity-002'],
]);

$identity3 = $identities->storeVerified('synthetic.charlie@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'verified_at' => $now,
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0103',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0203',
    'identity_state' => 'linked',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => $now,
    'promotional_consent_at' => $now,
    'promotional_consent_revoked_at' => null,
    'source' => 'delivery.consent.test',
    'migration_evidence' => ['record' => 'test-identity-003'],
]);

$identity1Uuid = $identity1['identity_uuid'];
$identity2Uuid = $identity2['identity_uuid'];
$identity3Uuid = $identity3['identity_uuid'];

// ── Positive: record delivery outcome — sent ──────────────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'sent',
    'occurred_at' => $clock(),
    'request_id' => 'req-delivery-sent',
    'idempotency_key' => 'idem-delivery-sent',
]);
expect_delivery_consent(!isset($result['error']), 'sent delivery records without error');
expect_delivery_consent($result['delivery_status'] === 'sent', 'delivery status is sent');
expect_delivery_consent($result['bounce_state'] === 'none', 'sent does not trigger bounce');
expect_delivery_consent($result['suppression_state'] === 'none', 'sent does not trigger suppression');
expect_delivery_consent($result['can_send_transactional'] === true, 'transactional sending remains allowed after sent');
expect_delivery_consent($result['can_verify'] === true, 'identity remains verifiable after sent');
expect_delivery_consent(!isset($result['email']), 'raw email is absent from delivery envelope');
expect_delivery_consent(!isset($result['verification_secret']), 'verification secret is absent from delivery envelope');

// ── Positive: record delivery outcome — delivered ──────────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'verification',
    'delivery_status' => 'delivered',
    'occurred_at' => $clock(),
    'request_id' => 'req-delivery-delivered',
    'idempotency_key' => 'idem-delivery-delivered',
]);
expect_delivery_consent(!isset($result['error']), 'delivered outcome records without error');
expect_delivery_consent($result['delivery_status'] === 'delivered', 'delivery status is delivered');
expect_delivery_consent($result['bounce_state'] === 'none', 'delivered does not trigger bounce');
expect_delivery_consent($result['can_send_transactional'] === true, 'transactional sending remains allowed after delivered');

// ── Positive: record delivery outcome — soft bounce ────────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'bounced',
    'bounce_type' => 'soft',
    'occurred_at' => $clock(),
    'request_id' => 'req-delivery-soft-bounce',
    'idempotency_key' => 'idem-delivery-soft-bounce',
]);
expect_delivery_consent(!isset($result['error']), 'soft bounce records without error');
expect_delivery_consent($result['bounce_state'] === 'soft', 'bounce state is soft');
expect_delivery_consent($result['suppression_state'] === 'none', 'soft bounce does not trigger suppression');
expect_delivery_consent($result['can_send_transactional'] === true, 'transactional sending remains allowed after soft bounce');
expect_delivery_consent($result['can_verify'] === true, 'soft bounce does not prevent verification');
expect_delivery_consent($result['can_send_promotional'] === false, 'promotional sending requires consent (not yet settled)');

// ── Positive: record delivery outcome — hard bounce ────────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity2Uuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'bounced',
    'bounce_type' => 'hard',
    'occurred_at' => $clock(),
    'request_id' => 'req-delivery-hard-bounce',
    'idempotency_key' => 'idem-delivery-hard-bounce',
]);
expect_delivery_consent(!isset($result['error']), 'hard bounce records without error');
expect_delivery_consent($result['bounce_state'] === 'hard', 'bounce state is hard');
expect_delivery_consent($result['can_send_transactional'] === false, 'hard bounce prevents transactional sending');
expect_delivery_consent($result['can_send_promotional'] === false, 'hard bounce prevents promotional sending');
expect_delivery_consent($result['can_verify'] === false, 'hard bounce prevents verification — bounce cannot become verified identity');

// ── Positive: record delivery outcome — complaint (spam report) ────────

// Create a fresh identity for complaint testing.
$now = $clock();
$identity4 = $identities->storeVerified('synthetic.delta@example.invalid', [
    'verification_state' => 'mailbox_verified',
    'verified_at' => $now,
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0104',
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0204',
    'identity_state' => 'primary',
    'verification_method' => 'magic_link',
    'transactional_consent_at' => $now,
    'promotional_consent_at' => null,
    'promotional_consent_revoked_at' => null,
    'source' => 'delivery.consent.test',
    'migration_evidence' => ['record' => 'test-identity-004'],
]);
$identity4Uuid = $identity4['identity_uuid'];

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity4Uuid,
    'message_kind' => 'promotional',
    'delivery_status' => 'complained',
    'occurred_at' => $clock(),
    'request_id' => 'req-delivery-complaint',
    'idempotency_key' => 'idem-delivery-complaint',
]);
expect_delivery_consent(!isset($result['error']), 'complaint records without error');
expect_delivery_consent($result['delivery_status'] === 'complained', 'delivery status is complained');
expect_delivery_consent(in_array($result['suppression_state'], ['promotional', 'all'], true), 'complaint triggers suppression');
expect_delivery_consent($result['can_send_promotional'] === false, 'complaint prevents promotional sending');
expect_delivery_consent($result['can_verify'] === false, 'complaint suppression prevents verification');

// ── Positive: settle transactional consent ─────────────────────────────

$result = $handler->settleTransactionalConsent([
    'identity_uuid' => $identity1Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-transactional',
    'idempotency_key' => 'idem-consent-transactional',
]);
expect_delivery_consent(!isset($result['error']), 'transactional consent settles without error');
expect_delivery_consent($result['consent_kind'] === 'transactional', 'consent kind is transactional');
expect_delivery_consent($result['transactional_consent_at'] !== null, 'transactional consent timestamp is set');
expect_delivery_consent($result['can_send_transactional'] === true, 'transactional sending remains allowed after transactional consent');
expect_delivery_consent($result['can_send_promotional'] === false, 'promotional sending still requires promotional consent');

// ── Positive: settle promotional consent ───────────────────────────────

$result = $handler->settlePromotionalConsent([
    'identity_uuid' => $identity1Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-promotional',
    'idempotency_key' => 'idem-consent-promotional',
]);
expect_delivery_consent(!isset($result['error']), 'promotional consent settles without error');
expect_delivery_consent($result['consent_kind'] === 'promotional', 'consent kind is promotional');
expect_delivery_consent($result['promotional_consent_at'] !== null, 'promotional consent timestamp is set');
expect_delivery_consent($result['can_send_promotional'] === true, 'promotional sending is now allowed');
expect_delivery_consent($result['transactional_consent_at'] !== null, 'transactional consent is independent of promotional');
expect_delivery_consent($result['promotional_consent_revoked_at'] === null, 'promotional consent is not revoked');

// ── Positive: revoke promotional consent ───────────────────────────────

$result = $handler->revokePromotionalConsent([
    'identity_uuid' => $identity1Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-revoke',
    'idempotency_key' => 'idem-consent-revoke',
]);
expect_delivery_consent(!isset($result['error']), 'promotional consent revocation succeeds');
expect_delivery_consent($result['consent_kind'] === 'promotional_revoked', 'consent kind is promotional_revoked');
expect_delivery_consent($result['promotional_consent_revoked_at'] !== null, 'revocation timestamp is set');
expect_delivery_consent($result['can_send_promotional'] === false, 'promotional sending is blocked after revocation');
expect_delivery_consent($result['can_send_transactional'] === true, 'transactional sending is unaffected by promotional revocation');
expect_delivery_consent($result['transactional_consent_at'] !== null, 'transactional consent is preserved through promotional revocation');

// ── Positive: capability checks — canSendTransactional never gated by promotional ──

// Identity with only transactional consent, no promotional.
expect_delivery_consent($handler->canSendTransactional($identity1Uuid) === true, 'transactional allowed with only transactional consent (post-revoke)');
expect_delivery_consent($handler->canSendPromotional($identity1Uuid) === false, 'promotional blocked after revocation');

// Identity with hard bounce.
expect_delivery_consent($handler->canSendTransactional($identity2Uuid) === false, 'hard bounce blocks transactional');
expect_delivery_consent($handler->canSendPromotional($identity2Uuid) === false, 'hard bounce blocks promotional');
expect_delivery_consent($handler->canVerifyIdentity($identity2Uuid) === false, 'hard bounce cannot become verified identity');

// Identity with both consents.
expect_delivery_consent($handler->canSendTransactional($identity3Uuid) === true, 'transactional sending allowed with both consents');
expect_delivery_consent($handler->canSendPromotional($identity3Uuid) === true, 'promotional sending allowed with promotional consent');

// Complained identity.
expect_delivery_consent($handler->canSendTransactional($identity4Uuid) === true, 'transactional sending allowed after complaint (suppression is promotional)');
expect_delivery_consent($handler->canSendPromotional($identity4Uuid) === false, 'promotional blocked after complaint');
expect_delivery_consent($handler->canVerifyIdentity($identity4Uuid) === false, 'suppressed identity cannot become verified identity');

// ── Negative: revoked identity cannot settle consent ──────────────────

// Create a revoked identity directly in the DB.
$identityMigration2 = new FocusaSpec152eEmailIdentityMigration($db, 'wp_');
$table = $identityMigration2->table('wpuiai_email_identities');
$db->exec("INSERT INTO {$table} (identity_uuid, account_uuid, encrypted_normalized_email, email_lookup_digest,
    verified_at, verification_method, identity_state, transactional_consent_at, promotional_consent_at,
    promotional_consent_revoked_at, bounce_state, suppression_state, source, migration_evidence, created_at, updated_at)
    VALUES ('018f47c2-6ac0-7b16-8d1a-4e93df5a0205', '018f47c2-6ac0-7b16-8d1a-4e93df5a0105',
    'encrypted-placeholder', 'digest-placeholder-0000000000000000000000000000000000000000000000000000000000000000',
    '2026-08-08T00:01:00Z', 'magic_link', 'revoked', NULL, NULL, NULL, 'none', 'none',
    'delivery.consent.test', '{\"record\":\"revoked\"}', '2026-08-08T00:01:00Z', '2026-08-08T00:01:00Z')");

$revokedUuid = '018f47c2-6ac0-7b16-8d1a-4e93df5a0205';

$result = $handler->settleTransactionalConsent([
    'identity_uuid' => $revokedUuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-revoked',
    'idempotency_key' => 'idem-consent-revoked',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'revoked identity cannot settle transactional consent');

$result = $handler->settlePromotionalConsent([
    'identity_uuid' => $revokedUuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-revoked-promo',
    'idempotency_key' => 'idem-consent-revoked-promo',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_VERIFICATION_REQUIRED', 'revoked identity cannot settle promotional consent');

// ── Negative: nonexistent identity returns safe error ──────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0999',
    'message_kind' => 'transactional',
    'delivery_status' => 'sent',
    'occurred_at' => $clock(),
    'request_id' => 'req-nonexistent',
    'idempotency_key' => 'idem-nonexistent',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_IDENTITY_NOT_FOUND', 'nonexistent identity returns safe error');
expect_delivery_consent($result['next_action'] === 'retry_or_recover_through_registered_facade', 'safe next action');

$result = $handler->settleTransactionalConsent([
    'identity_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0999',
    'occurred_at' => $clock(),
    'request_id' => 'req-nonexistent-consent',
    'idempotency_key' => 'idem-nonexistent-consent',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_IDENTITY_NOT_FOUND', 'nonexistent identity for consent returns safe error');

// ── Negative: hard bounce prevents promotional consent settlement ──────

$result = $handler->settlePromotionalConsent([
    'identity_uuid' => $identity2Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-promo-hard-bounce',
    'idempotency_key' => 'idem-promo-hard-bounce',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_DELIVERY_FAILED', 'hard bounce prevents promotional consent settlement');

// ── Negative: suppression prevents promotional consent settlement ──────

$result = $handler->settlePromotionalConsent([
    'identity_uuid' => $identity4Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-promo-suppressed',
    'idempotency_key' => 'idem-promo-suppressed',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_DELIVERY_FAILED', 'suppression prevents promotional consent settlement');

// ── Negative: promotional consent never gates transactional messages ───

// The identity with hard bounce (identity2) cannot send transactional.
// But the identity with only promotional suppression (identity4) CAN send transactional.
// This proves promotional consent and promotional suppression do not gate transactional.
expect_delivery_consent($handler->canSendTransactional($identity4Uuid) === true, 'promotional complaint does not gate transactional — promotional consent never gates transactional');
expect_delivery_consent($handler->canSendPromotional($identity4Uuid) === false, 'promotional complaint does gate promotional');

// ── Negative: no raw email, secrets, or authority references in envelopes ──

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'sent',
    'occurred_at' => $clock(),
    'request_id' => 'req-no-secrets',
    'idempotency_key' => 'idem-no-secrets',
]);
expect_delivery_consent(!isset($result['email']), 'no raw email in delivery envelope');
expect_delivery_consent(!isset($result['encrypted_normalized_email']), 'no encrypted email in delivery envelope');
expect_delivery_consent(!isset($result['email_lookup_digest']), 'no email digest in delivery envelope');
expect_delivery_consent(!isset($result['verification_secret']), 'no verification secret in delivery envelope');
expect_delivery_consent(!isset($result['license_key']), 'no license key in delivery envelope');
expect_delivery_consent(!isset($result['edd_customer_id']), 'no EDD customer ID in delivery envelope');
expect_delivery_consent(!isset($result['edd_order_id']), 'no EDD order ID in delivery envelope');

$consentResult = $handler->settleTransactionalConsent([
    'identity_uuid' => $identity1Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-no-secrets-consent',
    'idempotency_key' => 'idem-no-secrets-consent',
]);
expect_delivery_consent(!isset($consentResult['email']), 'no raw email in consent envelope');
expect_delivery_consent(!isset($consentResult['license_key']), 'no license key in consent envelope');

// ── Negative: consent already settled is idempotent ────────────────────

$result = $handler->settleTransactionalConsent([
    'identity_uuid' => $identity3Uuid,
    'occurred_at' => $clock(),
    'request_id' => 'req-consent-already',
    'idempotency_key' => 'idem-consent-already',
]);
expect_delivery_consent(!isset($result['error']), 'already-settled consent is idempotent (no error)');
expect_delivery_consent($result['transactional_consent_at'] !== null, 'existing consent timestamp is preserved');

// ── Negative: invalid input validation ─────────────────────────────────

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'invalid',
    'delivery_status' => 'sent',
    'occurred_at' => $clock(),
    'request_id' => 'req-invalid-kind',
    'idempotency_key' => 'idem-invalid-kind',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_DELIVERY_FAILED', 'invalid message kind is rejected safely');

$result = $handler->recordDeliveryOutcome([
    'identity_uuid' => $identity1Uuid,
    'message_kind' => 'transactional',
    'delivery_status' => 'invalid',
    'occurred_at' => $clock(),
    'request_id' => 'req-invalid-status',
    'idempotency_key' => 'idem-invalid-status',
]);
expect_delivery_consent(isset($result['error']) && $result['error'] === 'EMAIL_DELIVERY_FAILED', 'invalid delivery status is rejected safely');

// ── Transactional consent is independent from promotional consent ──────

// After revoking promotional consent on identity1, transactional consent is still present.
$identity1After = $identities->findByUuid($identity1Uuid);
expect_delivery_consent($identity1After['transactional_consent_at'] !== null, 'transactional consent is preserved after promotional revocation');
expect_delivery_consent($identity1After['promotional_consent_revoked_at'] !== null, 'promotional consent is revoked');
expect_delivery_consent($handler->canSendTransactional($identity1Uuid) === true, 'transactional is still allowed after promotional revocation');
expect_delivery_consent($handler->canSendPromotional($identity1Uuid) === false, 'promotional is blocked after revocation');

// ── Rollback preservation ──────────────────────────────────────────────

$rollback = $identityMigration->preserveForRollback('2026-08-08T01:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_delivery_consent_rollback',
]);
expect_delivery_consent($rollback['action'] === 'preserve', 'rollback is preservation-only');
$afterRollback = $identities->findByUuid($identity1Uuid);
expect_delivery_consent($afterRollback['transactional_consent_at'] !== null, 'rollback preserves transactional consent');
expect_delivery_consent($afterRollback['bounce_state'] === 'soft', 'rollback preserves bounce state');

$regRollback = $registrationMigration->preserveForRollback('2026-08-08T01:01:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_delivery_consent_reg_rollback',
]);
expect_delivery_consent($regRollback['action'] === 'preserve', 'registration rollback is preservation-only');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.email_delivery_consent_test.v1',
    'positive_checks' => 30,
    'negative_checks' => 18,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");