<?php
declare(strict_types=1);

require_once dirname(__DIR__) . '/docs/contracts/spec152e-activation-registration.v1.php';

function expect_registration_transition(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_registration_transition_throws(callable $operation, string $code, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect_registration_transition($error->getMessage() === $code, $message . ' error code');
        return;
    }
    expect_registration_transition(false, $message);
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$migration = new FocusaSpec152eActivationRegistrationMigration($db, 'wp_');
$migration->migrate('2026-08-07T05:00:00Z', ['source' => 'candidate_transition_reducer']);
$secrets = new FocusaSpec152eActivationRegistrationSecrets(
    str_repeat('e', 32),
    str_repeat('v', 32),
    str_repeat('p', 32),
);
$now = '2026-08-07T05:02:00Z';
$clock = static function () use (&$now): string {
    return $now;
};
$repository = new FocusaSpec152eActivationRegistrationRepository($db, $migration, $secrets, $clock);

$created = $repository->createPending([
    'email' => 'synthetic.transition@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'agent_json',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'request_id' => 'req-transition-0001',
    'idempotency_key' => 'idem-transition-0001',
]);
$id = $created['registration']['registration_uuid'];
$credential = $created['poll_credential'];
$challenge = $created['verification_secret'];
expect_registration_transition($created['registration']['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT, 'create enters challenge state');
expect_registration_transition($created['registration']['state_version'] === 1, 'create records the challenge transition at version one');

$replayCreate = $repository->createPending([
    'email' => 'synthetic.transition@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'agent_json',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'request_id' => 'req-transition-0001',
    'idempotency_key' => 'idem-transition-0001',
]);
expect_registration_transition($replayCreate['replayed'] === true, 'identical create request replays');
expect_registration_transition((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_activation_registrations')->fetchColumn() === 1, 'create replay does not duplicate a registration');
expect_registration_transition_throws(
    static fn() => $repository->createPending([
        'registration_uuid' => $id,
        'email' => 'synthetic.transition@example.invalid',
        'facade_id' => 'focusa_install_v1',
        'presenter' => 'agent_json',
        'install_channel' => 'source_build',
        'product_code' => 'focusa_operator',
        'request_id' => 'req-transition-0002',
        'idempotency_key' => 'idem-transition-0001',
    ]),
    'IDEMPOTENCY_CONFLICT',
    'changed create request identity cannot reuse an idempotency key'
);

expect_registration_transition_throws(
    static fn() => $repository->transition(
        $id,
        FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
        FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        1,
        'req-illegal-0001',
        'idem-illegal-0001',
        ['account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101', 'edd_customer_id' => 41001]
    ),
    'INVALID_REGISTRATION_TRANSITION',
    'unverified challenge cannot promote to an account'
);
expect_registration_transition_throws(
    static fn() => $repository->transition(
        $id,
        FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
        FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
        1,
        'req-illegal-0002',
        'idem-illegal-0002'
    ),
    'EMAIL_VERIFICATION_REQUIRED',
    'generic reducer cannot self-assert mailbox verification'
);
expect_registration_transition_throws(
    static fn() => $repository->transition(
        $id,
        FocusaSpec152eActivationRegistrationState::EMAIL_CHALLENGE_SENT,
        FocusaSpec152eActivationRegistrationState::DENIED,
        1,
        'req-illegal-0003',
        'idem-illegal-0003',
        ['account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101', 'edd_customer_id' => 41001]
    ),
    'PENDING_AUTHORITY_FIELD_DENIED',
    'failed pending attempts cannot attach customer authority'
);
expect_registration_transition_throws(
    static fn() => $repository->promoteVerified($id, '018f47c2-6ac0-7b16-8d1a-4e93df5a0101', 41001, 'req-promote-0000', 'idem-promote-0000'),
    'EMAIL_VERIFICATION_REQUIRED',
    'unverified registration cannot promote'
);

expect_registration_transition_throws(
    static fn() => $repository->verifyEmail($id, 'wrong-verifier', 'req-verify-bad-01', 'idem-verify-bad-01'),
    'EMAIL_VERIFICATION_FAILED',
    'wrong verifier fails closed'
);
$afterBadVerifier = $repository->findByUuid($id);
expect_registration_transition((int) $afterBadVerifier['verification_attempts'] === 1, 'failed verifier attempt is bounded and counted');
expect_registration_transition((int) $afterBadVerifier['state_version'] === 2, 'failed verifier update is CAS-guarded');
expect_registration_transition_throws(
    static fn() => $repository->verifyEmail($id, 'wrong-verifier', 'req-verify-bad-01', 'idem-verify-bad-01'),
    'EMAIL_VERIFICATION_FAILED',
    'replayed failed verifier is rejected without another attempt'
);
$afterBadReplay = $repository->findByUuid($id);
expect_registration_transition((int) $afterBadReplay['verification_attempts'] === 1, 'replayed failed verifier does not increment attempts');
expect_registration_transition((int) $afterBadReplay['state_version'] === 2, 'replayed failed verifier does not advance state version');

$verified = $repository->verifyEmail($id, $challenge, 'req-verify-good-01', 'idem-verify-good-01');
expect_registration_transition($verified['registration']['state'] === FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED, 'valid verifier enters email_verified');
expect_registration_transition($verified['registration']['verification_state'] === 'mailbox_verified', 'valid verifier records mailbox verification');
expect_registration_transition($verified['registration']['verification_challenge_hash'] === null, 'verification hash is single-use');
expect_registration_transition_throws(
    static fn() => $repository->transition($id, FocusaSpec152eActivationRegistrationState::EMAIL_VERIFIED,
        FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, (int) $verified['registration']['state_version'],
        'req-promotion-extra-authority-01', 'idem-promotion-extra-authority-01', [
            'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0101',
            'edd_customer_id' => 41001,
            'edd_license_id' => 503,
        ]),
    'PENDING_AUTHORITY_FIELD_DENIED',
    'account promotion cannot attach commerce or entitlement references'
);
$verifiedReplay = $repository->verifyEmail($id, $challenge, 'req-verify-good-01', 'idem-verify-good-01');
expect_registration_transition($verifiedReplay['replayed'] === true, 'verification replay is idempotent');
expect_registration_transition((int) $verifiedReplay['registration']['state_version'] === (int) $verified['registration']['state_version'], 'verification replay does not advance state');
expect_registration_transition_throws(
    static fn() => $repository->verifyEmail($id, 'different-verifier', 'req-verify-good-01', 'idem-verify-good-01'),
    'IDEMPOTENCY_CONFLICT',
    'changed verification payload fails closed under the same idempotency key'
);
expect_registration_transition_throws(
    static fn() => $repository->verifyEmail($id, $challenge, 'req-verify-good-02', 'idem-verify-good-01'),
    'IDEMPOTENCY_CONFLICT',
    'changed verification request identity fails closed under the same idempotency key'
);

$accountUuid = '018f47c2-6ac0-7b16-8d1a-4e93df5a0101';
$promoted = $repository->promoteVerified($id, $accountUuid, 41001, 'req-promote-good-01', 'idem-promote-good-01');
expect_registration_transition($promoted['registration']['state'] === FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, 'verified mailbox can attach canonical account references');
expect_registration_transition($promoted['registration']['account_uuid'] === $accountUuid, 'account reference is recorded only after verification');
expect_registration_transition((int) $promoted['registration']['edd_customer_id'] === 41001, 'EDD customer reference is recorded only after verification');
$promotedReplay = $repository->promoteVerified($id, $accountUuid, 41001, 'req-promote-good-01', 'idem-promote-good-01');
expect_registration_transition($promotedReplay['replayed'] === true, 'account promotion replay is idempotent');
expect_registration_transition_throws(
    static fn() => $repository->promoteVerified($id, '018f47c2-6ac0-7b16-8d1a-4e93df5a0102', 41002, 'req-promote-good-01', 'idem-promote-good-01'),
    'IDEMPOTENCY_CONFLICT',
    'changed account promotion cannot reuse an idempotency key'
);
expect_registration_transition_throws(
    static fn() => $repository->promoteVerified($id, $accountUuid, 41001, 'req-promote-good-02', 'idem-promote-good-01'),
    'IDEMPOTENCY_CONFLICT',
    'changed account promotion request identity fails closed under the same idempotency key'
);

$version = (int) $promoted['registration']['state_version'];
expect_registration_transition_throws(
    static fn() => $repository->transition($id, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, $version, 'req-account-binding-0001', 'idem-account-binding-0001', [
            'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0102',
        ]),
    'ACCOUNT_BINDING_CONFLICT',
    'canonical account binding cannot be replaced after promotion'
);
expect_registration_transition_throws(
    static fn() => $repository->transition($id, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, $version - 1, 'req-stale-0001', 'idem-stale-0001'),
    'REGISTRATION_STATE_CONFLICT',
    'stale expected version is denied by compare-and-set'
);
$offer = $repository->transition($id, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
    FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, $version, 'req-offer-0001', 'idem-offer-0001',
    ['offer_code' => 'focusa_operator', 'journey' => 'purchase']);
expect_registration_transition_throws(
    static fn() => $repository->transition($id, FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED,
        FocusaSpec152eActivationRegistrationState::OFFER_SELECTED, $version, 'req-offer-0002', 'idem-offer-0001',
        ['offer_code' => 'focusa_operator', 'journey' => 'purchase']),
    'IDEMPOTENCY_CONFLICT',
    'changed transition request identity fails closed under the same idempotency key'
);
$checkout = $repository->transition($id, FocusaSpec152eActivationRegistrationState::OFFER_SELECTED,
    FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING, (int) $offer['registration']['state_version'],
    'req-checkout-0001', 'idem-checkout-0001', ['edd_cart_reference' => 'cart_opaque_0001']);
$issued = $repository->transition($id, FocusaSpec152eActivationRegistrationState::CHECKOUT_PENDING,
    FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED, (int) $checkout['registration']['state_version'],
    'req-edd-0001', 'idem-edd-0001', ['edd_order_id' => 501, 'edd_order_item_id' => 502, 'edd_license_id' => 503]);
$delivery = $repository->transition($id, FocusaSpec152eActivationRegistrationState::ENTITLEMENT_ISSUED,
    FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY, (int) $issued['registration']['state_version'],
    'req-delivery-0001', 'idem-delivery-0001');
$node = $repository->transition($id, FocusaSpec152eActivationRegistrationState::TERMINAL_DELIVERY_READY,
    FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED, (int) $delivery['registration']['state_version'],
    'req-node-0001', 'idem-node-0001', [
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0102',
        'device_public_key' => 'synthetic-device-public-key-0001',
    ]);
$lease = $repository->transition($id, FocusaSpec152eActivationRegistrationState::DEVICE_REGISTERED,
    FocusaSpec152eActivationRegistrationState::LEASE_ISSUED, (int) $node['registration']['state_version'],
    'req-lease-0001', 'idem-lease-0001');
$delivered = $repository->transition($id, FocusaSpec152eActivationRegistrationState::LEASE_ISSUED,
    FocusaSpec152eActivationRegistrationState::DELIVERED, (int) $lease['registration']['state_version'],
    'req-delivered-01', 'idem-delivered-01');
expect_registration_transition($delivered['registration']['state'] === FocusaSpec152eActivationRegistrationState::DELIVERED, 'legal reducer path reaches delivered');
expect_registration_transition((int) $delivered['registration']['edd_order_id'] === 501, 'EDD order reference survives transitions');
expect_registration_transition($delivered['registration']['terminal_delivery_status'] === 'delivered', 'delivery status is state-bound');
expect_registration_transition($delivered['registration']['node_uuid'] === '018f47c2-6ac0-7b16-8d1a-4e93df5a0102', 'node reference survives transitions');
expect_registration_transition((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_activation_registration_transitions')->fetchColumn() === 10, 'each legal state transition has one journal row');

$poll = $repository->poll($id, $credential, 'req-poll-0001', 'idem-poll-0001');
expect_registration_transition($poll['snapshot']['state'] === FocusaSpec152eActivationRegistrationState::DELIVERED, 'valid poll returns a bounded state snapshot');
expect_registration_transition(!isset($poll['snapshot']['poll_credential_hash'], $poll['snapshot']['encrypted_normalized_email']), 'poll never returns credential or email storage fields');
$pollReplay = $repository->poll($id, $credential, 'req-poll-0001', 'idem-poll-0001');
expect_registration_transition($pollReplay['replayed'] === true, 'poll replay is idempotent');
expect_registration_transition_throws(
    static fn() => $repository->poll($id, $credential, 'req-poll-0002', 'idem-poll-0001'),
    'IDEMPOTENCY_CONFLICT',
    'changed poll request identity fails closed under the same idempotency key'
);
expect_registration_transition_throws(
    static fn() => $repository->poll($id, 'different-poll-credential', 'req-poll-0001', 'idem-poll-0001'),
    'IDEMPOTENCY_CONFLICT',
    'changed poll credential cannot reuse an idempotency key'
);

$now = '2026-08-07T05:03:00Z';
expect_registration_transition_throws(
    static fn() => $repository->transition($id, FocusaSpec152eActivationRegistrationState::DELIVERED,
        FocusaSpec152eActivationRegistrationState::ACCOUNT_PROMOTED, (int) $delivered['registration']['state_version'],
        'req-illegal-terminal-01', 'idem-illegal-terminal-01'),
    'INVALID_REGISTRATION_TRANSITION',
    'terminal registration cannot move backward into account promotion'
);

// A short-TTL fixture proves cleanup marks expiry, rejects polling, and eventually bounds retention.
$shortRepository = new FocusaSpec152eActivationRegistrationRepository($db, $migration, $secrets, $clock, 60, 30, 60, 120);
$now = '2026-08-07T06:00:00Z';
$short = $shortRepository->createPending([
    'email' => 'synthetic.expiry@example.invalid',
    'facade_id' => 'focusa_install_v1',
    'presenter' => 'terminal',
    'install_channel' => 'source_build',
    'product_code' => 'focusa_operator',
    'request_id' => 'req-expiry-0001',
    'idempotency_key' => 'idem-expiry-0001',
]);
$shortId = $short['registration']['registration_uuid'];
$shortCredential = $short['poll_credential'];
$now = '2026-08-07T06:01:01Z';
expect_registration_transition_throws(
    static fn() => $shortRepository->promoteVerified($shortId, $accountUuid, 41003, 'req-expiry-promote-01', 'idem-expiry-promote-01'),
    'REGISTRATION_EXPIRED',
    'expired pending record cannot promote to an account'
);
expect_registration_transition_throws(
    static fn() => $shortRepository->poll($shortId, $shortCredential, 'req-expiry-poll-01', 'idem-expiry-poll-01'),
    'REGISTRATION_EXPIRED',
    'expired pending record cannot poll'
);
$cleanup = $shortRepository->cleanup($now);
expect_registration_transition($cleanup['expired'] === 1, 'cleanup uses a guarded expiry transition');
$expired = $shortRepository->findByUuid($shortId);
expect_registration_transition($expired['state'] === FocusaSpec152eActivationRegistrationState::EXPIRED, 'cleanup records terminal expiry');
expect_registration_transition($expired['account_uuid'] === null && $expired['edd_customer_id'] === null, 'expiry retains pending no-customer posture');
expect_registration_transition_throws(
    static fn() => $shortRepository->poll($shortId, $shortCredential, 'req-expiry-poll-02', 'idem-expiry-poll-02'),
    'REGISTRATION_EXPIRED',
    'terminal expired state cannot poll'
);
$now = '2026-08-07T06:04:01Z';
$retentionCleanup = $shortRepository->cleanup($now);
expect_registration_transition($retentionCleanup['deleted'] === 1, 'cleanup deletes only unpromoted records past retention');
expect_registration_transition_throws(
    static fn() => $shortRepository->findByUuid($shortId),
    'activation registration not found',
    'bounded cleanup removes expired pending storage after retention'
);

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.registration_transition_validation.v1',
    'legal_transition_rows' => 10,
    'final_state' => $delivered['registration']['state'],
    'expired_cleanup' => $cleanup['expired'],
    'retention_deleted' => $retentionCleanup['deleted'],
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
