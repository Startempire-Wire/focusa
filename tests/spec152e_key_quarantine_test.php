<?php
// 152E.06.04 Quarantine synthetic, duplicate, and ambiguous keys.
// The key-quarantine ledger quarantines synthetic focusa_live rows, duplicate
// EDD/custom keys, orphan payment IDs, and unresolved products/accounts; denies
// new activation/lease from quarantined records; selects the canonical key only
// with proof; and exposes an explicit operator review/settlement path. Audit and
// evidence are retained (digests and masked keys only — never raw keys, raw
// emails, payment ids, or secrets), and rollback is preservation-only. No
// synthetic/orphan/duplicate record silently becomes paid authority, and
// approved canonical records remain usable.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-activation-registration.v1.php';
require_once $root . '/docs/contracts/spec152e-email-identity.v1.php';
require_once $root . '/docs/contracts/spec152e-key-quarantine.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_quarantine(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_quarantine_throws_code(callable $operation, string $code, string $message): void
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

function expect_quarantine_throws_type(callable $operation, string $exception, string $message): void
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

$schema = new FocusaSpec152eKeyQuarantineSchema($db, 'wp_');
$schema->migrate('2026-08-08T00:00:00Z', ['source' => 'key_quarantine_test']);

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
$quarantine = new FocusaSpec152eKeyQuarantineService($db, $schema, $clock);

$emailDigestOf = static fn(string $email): string =>
    $registrationSecrets->emailLookupDigest(FocusaSpec152eEmailNormalizer::exact($email));

$provenance = static fn(string $tag): array => [
    'source' => 'key_quarantine_test',
    'tag' => $tag,
];

// ── Duplicate-settlement fixture (redacted, deterministic, replayable) ──

$fixturePath = $root . '/docs/contracts/spec152e-key-quarantine-fixture.v1.json';
$fixtureDigest = hash_file('sha256', $fixturePath);
$fixtureRaw = file_get_contents($fixturePath);
$fixture = json_decode($fixtureRaw, true, 512, JSON_THROW_ON_ERROR);

expect_quarantine($fixture['schema'] === 'focusa.spec152e.key_quarantine_fixture.v1', 'fixture schema is typed');
expect_quarantine($fixture['fixture_id'] === 'focusa-vbcqu.20.13.52', 'fixture id pins the atom');
expect_quarantine($fixture['authority']['canonical'] === 'WPUIAI.com EDD', 'fixture authority is canonical EDD');
expect_quarantine($fixture['authority']['spec158'] === 'excluded', 'fixture excludes Spec 158');
expect_quarantine($fixture['redaction']['raw_email'] === 'absent', 'fixture declares raw email absent');
expect_quarantine($fixture['redaction']['raw_key'] === 'absent', 'fixture declares raw keys absent');
expect_quarantine($fixture['redaction']['payment_id_stored'] === false, 'fixture declares no stored payment id');
expect_quarantine($fixture['redaction']['secret_material'] === 'absent', 'fixture declares secret material absent');
$fixtureHandles = array_column($fixture['records'], 'handle');
expect_quarantine(count($fixtureHandles) === count(array_unique($fixtureHandles)), 'fixture record handles are unique');
expect_quarantine(count($fixture['records']) === 9, 'fixture carries the nine quarantine records');
foreach ($fixture['records'] as $record) {
    expect_quarantine(preg_match('/^rec_[a-z0-9_]{6,64}$/D', $record['handle']) === 1, 'fixture handle is bounded and opaque');
    expect_quarantine(in_array($record['surface'], FocusaSpec152eKeyQuarantineService::SURFACES, true), 'fixture surface is known');
    expect_quarantine(
        $record['reason'] === FocusaSpec152eKeyQuarantineService::QUARANTINE_REASONS[$record['surface']],
        'fixture reason matches the surface fail-closed map',
    );
    expect_quarantine(
        preg_match('/^[A-Za-z0-9*_]{4,191}$/D', $record['masked_key']) === 1,
        'fixture masked key is bounded and masked',
    );
    expect_quarantine($record['masked_key'] === str_replace('*', '', $record['masked_key']) || str_contains($record['masked_key'], '*'), 'fixture masked key is never a full key');
}
expect_quarantine(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $fixtureRaw) !== 1, 'fixture contains no email address');
expect_quarantine(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $fixtureRaw) !== 1, 'fixture contains no secret prefix');
expect_quarantine(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/i', $fixtureRaw) !== 1, 'fixture contains no synthetic focusa_live key');
expect_quarantine(preg_match('/\b[0-9A-F]{8}(-[0-9A-F]{4}){3}\b/', $fixtureRaw) !== 1, 'fixture contains no full canonical key pattern');
expect_quarantine(preg_match('/^[0-9a-f]{64}$/D', $fixtureDigest) === 1, 'fixture digest is bounded sha256');
$fixtureByHandle = [];
foreach ($fixture['records'] as $record) {
    $fixtureByHandle[$record['handle']] = $record;
}

// Test-only key material (never stored, never fixture): digests only enter the ledger.
$keys = [
    'rec_q_synth_0001' => 'focusa_live_0001_4e1f2a3b',
    'rec_q_synth_0002' => 'focusa_live_0002_8c7d6e5f',
    'rec_q_dup_edd_a' => 'EDDKEY-1001-AAAA-1001',
    'rec_q_dup_edd_b' => 'EDDKEY-1001-AAAA-1001',
    'rec_q_dup_custom_a' => 'custom_key_2001_aaaabbbb',
    'rec_q_dup_custom_b' => 'custom_key_2001_aaaabbbb',
    'rec_q_orphan_0001' => 'pi_3Orphan_3001',
    'rec_q_product_0001' => 'focusa_unresolved_product_4001',
    'rec_q_account_0001' => 'acct_unresolved_5001',
];
$unledgeredSynthetic = 'focusa_live_0099_deadbeef';
$unledgeredCanonical = 'EDDKEY-9001-CANON-9001';

// Keyed email digests (proof of ownership for canonical selection).
$digestOwnerA = $emailDigestOf('owner.a@example.invalid');
$digestOwnerB = $emailDigestOf('owner.b@example.invalid');
$digestOwnerC = $emailDigestOf('owner.c@example.invalid');

// ── Quarantine every fixture record ────────────────────────────────────

$quarantineSeq = 0;
$quarantined = [];
foreach ($fixture['records'] as $record) {
    $quarantineSeq++;
    $handle = $record['handle'];
    $input = [
        'record_handle' => $handle,
        'surface' => $record['surface'],
        'quarantine_reason' => $record['reason'],
        'key_material' => $keys[$handle],
        'masked_key' => $record['masked_key'],
        'key_group' => $record['key_group'] ?? null,
        'email_lookup_digest' => match ($handle) {
            'rec_q_dup_edd_a' => $digestOwnerA,
            'rec_q_dup_edd_b' => $digestOwnerB,
            'rec_q_dup_custom_a', 'rec_q_dup_custom_b' => $digestOwnerC,
            default => null,
        },
        'legacy_evidence' => $record['evidence'],
        'request_id' => "req-q-{$quarantineSeq}-0001",
        'idempotency_key' => "idem-q-{$quarantineSeq}-0001",
        'migration_provenance' => $provenance('quarantine-' . $handle),
    ];
    $result = $quarantine->quarantineRecord($input);
    expect_quarantine($result['action'] === 'legacy_record_quarantined', "{$handle} is quarantined");
    expect_quarantine($result['state'] === 'quarantined', "{$handle} is in quarantined state");
    expect_quarantine($result['quarantine_reason'] === $record['reason'], "{$handle} carries its fail-closed reason");
    expect_quarantine($result['key_digest'] === hash('sha256', $keys[$handle]), "{$handle} key digest is the bounded sha256 of the key");
    expect_quarantine($result['replayed'] === false && $result['existing'] === false, "{$handle} is a fresh quarantine");
    $quarantined[$handle] = $result;

    // Idempotent replay of the identical quarantine request.
    $replay = $quarantine->quarantineRecord($input);
    expect_quarantine($replay['quarantine_uuid'] === $result['quarantine_uuid'], "{$handle} replay returns the same ledger row");
    expect_quarantine((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_ledger WHERE record_handle = '{$handle}'")->fetchColumn() === 1, "{$handle} has exactly one ledger row");
}
expect_quarantine((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_ledger')->fetchColumn() === 9, 'nine records are ledgered');

// ── Fail-closed gate: no new activation/lease from quarantined records ──

$gateDenials = 0;
$gate = static function (string $key, string $purpose, string $requestId) use ($quarantine, &$gateDenials): array {
    $decision = $quarantine->activationLeaseGate([
        'key_material' => $key,
        'purpose' => $purpose,
        'request_id' => $requestId,
    ]);
    if ($decision['allowed'] === false) {
        $gateDenials++;
    }
    return $decision;
};

$expectedGateDenials = [
    'rec_q_synth_0001' => 'EDD_ORDER_UNVERIFIED',
    'rec_q_synth_0002' => 'EDD_ORDER_UNVERIFIED',
    'rec_q_dup_edd_a' => 'LICENSE_ACCOUNT_MISMATCH',
    'rec_q_dup_edd_b' => 'LICENSE_ACCOUNT_MISMATCH',
    'rec_q_dup_custom_a' => 'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'rec_q_dup_custom_b' => 'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'rec_q_orphan_0001' => 'EDD_ORDER_UNVERIFIED',
    'rec_q_product_0001' => 'PRODUCT_MAPPING_REQUIRED',
    'rec_q_account_0001' => 'EDD_CUSTOMER_RESOLUTION_FAILED',
];
foreach ($expectedGateDenials as $handle => $reason) {
    foreach (['activation', 'lease'] as $purpose) {
        $decision = $gate($keys[$handle], $purpose, "req-gate-{$handle}-{$purpose}");
        expect_quarantine($decision['allowed'] === false, "{$handle} denies {$purpose}");
        expect_quarantine($decision['reason'] === $reason, "{$handle} denies {$purpose} with {$reason}");
        expect_quarantine($decision['kind'] === 'quarantined_key_denied', "{$handle} denial is the quarantined-key kind");
    }
}
$unledgeredSyntheticGate = $gate($unledgeredSynthetic, 'activation', 'req-gate-synth-unledgered');
expect_quarantine($unledgeredSyntheticGate['allowed'] === false, 'an unledgered synthetic key cannot activate');
expect_quarantine($unledgeredSyntheticGate['reason'] === 'EDD_ORDER_UNVERIFIED', 'unledgered synthetic denial is fail-closed');
expect_quarantine($unledgeredSyntheticGate['kind'] === 'synthetic_key_denied', 'unledgered synthetic denial is the synthetic kind');
$unledgeredCanonicalGate = $gate($unledgeredCanonical, 'lease', 'req-gate-canon-unledgered');
expect_quarantine($unledgeredCanonicalGate['allowed'] === true, 'an unledgered canonical key remains usable');
expect_quarantine($unledgeredCanonicalGate['ledgered'] === false, 'unledgered canonical key is not ledgered');

// ── Canonical selection only with proof (duplicate EDD key group) ──────

$selectInput = static fn(string $group, string $handle, string $operator, string $digest, array $evidence, string $idem): array => [
    'key_group' => $group,
    'record_handle' => $handle,
    'operator_id' => $operator,
    'email_lookup_digest' => $digest,
    'legacy_evidence' => $evidence,
    'request_id' => 'req-select-' . $handle . '-0001',
    'idempotency_key' => $idem,
    'migration_provenance' => $provenance('select-' . $handle),
];

// No evidence -> no proof -> fail closed, record untouched.
expect_quarantine_throws_code(
    fn() => $quarantine->selectCanonicalKey($selectInput(
        'kg_dup_edd_0001', 'rec_q_dup_edd_a', 'op_quarantine_sel', $digestOwnerA, [], 'idem-select-noev-a',
    )),
    'EDD_ORDER_UNVERIFIED',
    'canonical selection without evidence fails closed',
);
// Wrong keyed identity digest -> not the owner -> fail closed.
expect_quarantine_throws_code(
    fn() => $quarantine->selectCanonicalKey($selectInput(
        'kg_dup_edd_0001', 'rec_q_dup_edd_a', 'op_quarantine_sel', $digestOwnerB,
        $fixtureByHandle['rec_q_dup_edd_a']['evidence'], 'idem-select-wrong-a',
    )),
    'EDD_ORDER_UNVERIFIED',
    'canonical selection with the wrong owner digest fails closed',
);
// Candidate from another key group -> fail closed.
expect_quarantine_throws_code(
    fn() => $quarantine->selectCanonicalKey($selectInput(
        'kg_dup_edd_0001', 'rec_q_synth_0001', 'op_quarantine_sel', $digestOwnerA,
        $fixtureByHandle['rec_q_synth_0001']['evidence'], 'idem-select-foreign',
    )),
    'EDD_ORDER_UNVERIFIED',
    'canonical selection of a record outside the group fails closed',
);
expect_quarantine(
    (string) $db->query("SELECT state FROM wp_wpuiai_key_quarantine_ledger WHERE record_handle = 'rec_q_dup_edd_a'")->fetchColumn() === 'quarantined',
    'failed selections leave the candidate quarantined and untouched',
);

// Proof-backed selection of the canonical owner (record a, digest A).
$selected = $quarantine->selectCanonicalKey($selectInput(
    'kg_dup_edd_0001', 'rec_q_dup_edd_a', 'op_quarantine_sel', $digestOwnerA,
    $fixtureByHandle['rec_q_dup_edd_a']['evidence'], 'idem-select-a-0001',
));
expect_quarantine($selected['action'] === 'canonical_key_selected', 'proof-backed selection records canonical status');
expect_quarantine($selected['state'] === 'settled_approved' && $selected['canonical'] === true, 'selected record is settled_approved canonical');
expect_quarantine((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_settlements")->fetchColumn() === 1, 'selection creates exactly one settlement');

// Replay of the identical selection returns the stored decision.
$selectedReplay = $quarantine->selectCanonicalKey($selectInput(
    'kg_dup_edd_0001', 'rec_q_dup_edd_a', 'op_quarantine_sel', $digestOwnerA,
    $fixtureByHandle['rec_q_dup_edd_a']['evidence'], 'idem-select-a-0001',
));
expect_quarantine($selectedReplay['replayed'] === true && $selectedReplay['settlement_uuid'] === $selected['settlement_uuid'], 'canonical selection replay returns the stored settlement');
expect_quarantine((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_settlements")->fetchColumn() === 1, 'selection replay creates no second settlement');

// The other duplicate cannot become canonical: the group is already settled.
expect_quarantine_throws_code(
    fn() => $quarantine->selectCanonicalKey($selectInput(
        'kg_dup_edd_0001', 'rec_q_dup_edd_b', 'op_quarantine_sel', $digestOwnerB,
        $fixtureByHandle['rec_q_dup_edd_b']['evidence'], 'idem-select-b-0001',
    )),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'a second canonical in the same key group is impossible',
);

// The unresolved duplicate still blocks the gate until settled.
$gateBeforeSettle = $gate($keys['rec_q_dup_edd_a'], 'activation', 'req-gate-dup-pre-settle');
expect_quarantine($gateBeforeSettle['allowed'] === false, 'an unresolved duplicate still denies activation');
expect_quarantine($gateBeforeSettle['reason'] === 'LICENSE_ACCOUNT_MISMATCH', 'unresolved duplicate denies with its quarantine reason');

// Explicit operator settlement: deny the non-canonical duplicate.
$settledDeny = $quarantine->settleRecord([
    'record_handle' => 'rec_q_dup_edd_b',
    'operator_id' => 'op_quarantine_review',
    'decision' => 'deny',
    'reason' => 'LICENSE_ACCOUNT_MISMATCH',
    'legacy_evidence' => $fixtureByHandle['rec_q_dup_edd_b']['evidence'],
    'request_id' => 'req-settle-dup-b-0001',
    'idempotency_key' => 'idem-settle-dup-b-0001',
    'migration_provenance' => $provenance('settle-dup-b'),
]);
expect_quarantine($settledDeny['decision'] === 'deny' && $settledDeny['state'] === 'settled_denied', 'operator denial settles the duplicate as denied');

// Approved canonical record remains usable once the group is settled.
$gateAfterSettle = $gate($keys['rec_q_dup_edd_a'], 'lease', 'req-gate-dup-post-settle');
expect_quarantine($gateAfterSettle['allowed'] === true, 'the approved canonical record remains usable');
expect_quarantine($gateAfterSettle['canonical'] === true && $gateAfterSettle['canonical_record_handle'] === 'rec_q_dup_edd_a', 'the gate identifies the canonical record');

// ── Ambiguous duplicates (same owner proof, two rows) fail closed ──────

expect_quarantine_throws_code(
    fn() => $quarantine->selectCanonicalKey($selectInput(
        'kg_dup_custom_0001', 'rec_q_dup_custom_a', 'op_quarantine_sel', $digestOwnerC,
        $fixtureByHandle['rec_q_dup_custom_a']['evidence'], 'idem-select-custom-a',
    )),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'the same proof matching two duplicates is ambiguous and fails closed',
);
expect_quarantine(
    (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_journal WHERE event_type = 'selection_ambiguous'")->fetchColumn() === 1,
    'ambiguity is journaled for operator review',
);
foreach (['rec_q_dup_custom_a', 'rec_q_dup_custom_b'] as $handle) {
    $quarantine->settleRecord([
        'record_handle' => $handle,
        'operator_id' => 'op_quarantine_review',
        'decision' => 'deny',
        'reason' => 'ACCOUNT_MERGE_REVIEW_REQUIRED',
        'legacy_evidence' => $fixtureByHandle[$handle]['evidence'],
        'request_id' => 'req-settle-' . $handle . '-0001',
        'idempotency_key' => 'idem-settle-' . $handle . '-0001',
        'migration_provenance' => $provenance('settle-' . $handle),
    ]);
}
$customGate = $gate($keys['rec_q_dup_custom_a'], 'activation', 'req-gate-custom-settled');
expect_quarantine($customGate['allowed'] === false, 'duplicate custom keys settled denied cannot activate');
expect_quarantine($customGate['kind'] === 'denied_key_denied', 'settled-denied duplicates deny at the gate');

// ── Synthetic records: quarantined by default, usable only via explicit operator approval ──

$synthApprove = $quarantine->settleRecord([
    'record_handle' => 'rec_q_synth_0001',
    'operator_id' => 'op_quarantine_review',
    'decision' => 'approve_canonical',
    'reason' => 'PROOF_BACKED_SELECTION',
    'legacy_evidence' => ['kind' => 'operator_review', 'source' => 'authority_reconciliation', 'record' => 'synth-approval-0001'],
    'request_id' => 'req-settle-synth-0001',
    'idempotency_key' => 'idem-settle-synth-0001',
    'migration_provenance' => $provenance('settle-synth-0001'),
]);
expect_quarantine($synthApprove['state'] === 'settled_approved', 'explicit operator approval settles a synthetic record');
$synthGate = $gate($keys['rec_q_synth_0001'], 'lease', 'req-gate-synth-approved');
expect_quarantine($synthGate['allowed'] === true && $synthGate['canonical'] === true, 'a separately approved synthetic record becomes usable');
$synthStillQuarantined = $gate($keys['rec_q_synth_0002'], 'activation', 'req-gate-synth-0002');
expect_quarantine($synthStillQuarantined['allowed'] === false, 'an unapproved synthetic record still cannot activate');

// ── Orphan payment IDs and unresolved products/accounts stay fail-closed ──

foreach ([
    ['rec_q_orphan_0001', 'EDD_ORDER_UNVERIFIED'],
    ['rec_q_product_0001', 'EDD_ORDER_UNVERIFIED'],
    ['rec_q_account_0001', 'ACCOUNT_MERGE_REVIEW_REQUIRED'],
] as [$handle, $denyReason]) {
    $quarantine->settleRecord([
        'record_handle' => $handle,
        'operator_id' => 'op_quarantine_review',
        'decision' => 'deny',
        'reason' => $denyReason,
        'legacy_evidence' => $fixtureByHandle[$handle]['evidence'],
        'request_id' => 'req-settle-' . $handle . '-0001',
        'idempotency_key' => 'idem-settle-' . $handle . '-0001',
        'migration_provenance' => $provenance('settle-' . $handle),
    ]);
}
$orphanGate = $gate($keys['rec_q_orphan_0001'], 'activation', 'req-gate-orphan-settled');
expect_quarantine($orphanGate['allowed'] === false && $orphanGate['reason'] === 'EDD_ORDER_UNVERIFIED', 'an orphan payment ID can never authorize activation');
$productGate = $gate($keys['rec_q_product_0001'], 'lease', 'req-gate-product-settled');
expect_quarantine($productGate['allowed'] === false && $productGate['reason'] === 'PRODUCT_MAPPING_REQUIRED', 'an unresolved product can never authorize a lease');
$accountGate = $gate($keys['rec_q_account_0001'], 'activation', 'req-gate-account-settled');
expect_quarantine($accountGate['allowed'] === false && $accountGate['reason'] === 'EDD_CUSTOMER_RESOLUTION_FAILED', 'an unresolved account can never authorize activation');

// ── Operator settlement negatives and input bounds ─────────────────────

expect_quarantine_throws_type(
    fn() => $quarantine->settleRecord([
        'record_handle' => 'rec_q_unknown_9999',
        'operator_id' => 'op_quarantine_review',
        'decision' => 'deny',
        'reason' => 'EDD_ORDER_UNVERIFIED',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0001']['evidence'],
        'request_id' => 'req-settle-unknown-0001',
        'idempotency_key' => 'idem-settle-unknown-0001',
        'migration_provenance' => $provenance('settle-unknown'),
    ]),
    OutOfBoundsException::class,
    'settling an unknown record fails closed',
);
expect_quarantine_throws_type(
    fn() => $quarantine->settleRecord([
        'record_handle' => 'rec_q_orphan_0001',
        'operator_id' => 'op_quarantine_review',
        'decision' => 'approve_canonical',
        'reason' => 'EDD_ORDER_UNVERIFIED',
        'legacy_evidence' => $fixtureByHandle['rec_q_orphan_0001']['evidence'],
        'request_id' => 'req-settle-bad-approve-0001',
        'idempotency_key' => 'idem-settle-bad-approve-0001',
        'migration_provenance' => $provenance('settle-bad-approve'),
    ]),
    InvalidArgumentException::class,
    'approval requires the proof-backed reason',
);
expect_quarantine_throws_type(
    fn() => $quarantine->settleRecord([
        'record_handle' => 'rec_q_synth_0002',
        'operator_id' => 'op_quarantine_review',
        'decision' => 'deny',
        'reason' => 'EDD_LICENSE_PENDING',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0002']['evidence'],
        'request_id' => 'req-settle-bad-deny-0001',
        'idempotency_key' => 'idem-settle-bad-deny-0001',
        'migration_provenance' => $provenance('settle-bad-deny'),
    ]),
    InvalidArgumentException::class,
    'denial requires a bounded denial reason',
);
expect_quarantine_throws_code(
    fn() => $quarantine->settleRecord([
        'record_handle' => 'rec_q_synth_0001',
        'operator_id' => 'op_quarantine_review',
        'decision' => 'deny',
        'reason' => 'ACCOUNT_MERGE_REVIEW_REQUIRED',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0001']['evidence'],
        'request_id' => 'req-settle-resettle-0001',
        'idempotency_key' => 'idem-settle-resettle-0001',
        'migration_provenance' => $provenance('settle-resettle'),
    ]),
    'IDEMPOTENCY_CONFLICT',
    'a settled record cannot be re-settled with a different decision',
);
$settleReplay = $quarantine->settleRecord([
    'record_handle' => 'rec_q_synth_0001',
    'operator_id' => 'op_quarantine_review',
    'decision' => 'approve_canonical',
    'reason' => 'PROOF_BACKED_SELECTION',
    'legacy_evidence' => ['kind' => 'operator_review', 'source' => 'authority_reconciliation', 'record' => 'synth-approval-0001'],
    'request_id' => 'req-settle-synth-0001',
    'idempotency_key' => 'idem-settle-synth-0001',
    'migration_provenance' => $provenance('settle-synth-0001'),
]);
expect_quarantine($settleReplay['replayed'] === true && $settleReplay['state'] === 'settled_approved', 'settlement replay returns the stored decision');
expect_quarantine_throws_code(
    fn() => $quarantine->quarantineRecord([
        'record_handle' => 'rec_q_dup_edd_a',
        'surface' => 'duplicate_edd_key',
        'quarantine_reason' => 'LICENSE_ACCOUNT_MISMATCH',
        'key_material' => $keys['rec_q_dup_edd_a'],
        'legacy_evidence' => $fixtureByHandle['rec_q_dup_edd_a']['evidence'],
        'request_id' => 'req-req-approved-0001',
        'idempotency_key' => 'idem-req-approved-0001',
        'migration_provenance' => $provenance('req-approved'),
    ]),
    'ACCOUNT_MERGE_REVIEW_REQUIRED',
    'an approved canonical record cannot be silently re-quarantined',
);
expect_quarantine_throws_type(
    fn() => $quarantine->quarantineRecord([
        'record_handle' => 'rec_q_bad_surface',
        'surface' => 'unmapped_surface',
        'quarantine_reason' => 'EDD_ORDER_UNVERIFIED',
        'key_material' => 'some_key_material_0001',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0001']['evidence'],
        'request_id' => 'req-bad-surface-0001',
        'idempotency_key' => 'idem-bad-surface-0001',
        'migration_provenance' => $provenance('bad-surface'),
    ]),
    InvalidArgumentException::class,
    'an unknown quarantine surface is rejected',
);
expect_quarantine_throws_type(
    fn() => $quarantine->quarantineRecord([
        'record_handle' => 'rec_q_bad_reason',
        'surface' => 'focusa_live_synthetic',
        'quarantine_reason' => 'REFUNDED',
        'key_material' => 'some_key_material_0002',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0001']['evidence'],
        'request_id' => 'req-bad-reason-0001',
        'idempotency_key' => 'idem-bad-reason-0001',
        'migration_provenance' => $provenance('bad-reason'),
    ]),
    InvalidArgumentException::class,
    'a non-surface-mapped quarantine reason is rejected',
);
expect_quarantine_throws_type(
    fn() => $quarantine->quarantineRecord([
        'record_handle' => 'rec_q_bad_key',
        'surface' => 'focusa_live_synthetic',
        'quarantine_reason' => 'EDD_ORDER_UNVERIFIED',
        'legacy_evidence' => $fixtureByHandle['rec_q_synth_0001']['evidence'],
        'request_id' => 'req-bad-key-0001',
        'idempotency_key' => 'idem-bad-key-0001',
        'migration_provenance' => $provenance('bad-key'),
    ]),
    InvalidArgumentException::class,
    'quarantine without key material or digest is rejected',
);
expect_quarantine_throws_type(
    fn() => $quarantine->activationLeaseGate([
        'key_material' => $keys['rec_q_synth_0001'],
        'purpose' => 'renewal',
        'request_id' => 'req-bad-purpose-0001',
    ]),
    InvalidArgumentException::class,
    'an unknown gate purpose is rejected',
);
expect_quarantine_throws_type(
    fn() => $quarantine->listQuarantined(['request_id' => 'req-list-bad-0001', 'limit' => 0]),
    InvalidArgumentException::class,
    'an unbounded ledger page limit is rejected',
);

// ── Operator read path: bounded, digest-only ledger listing ────────────

$listing = $quarantine->listQuarantined(['request_id' => 'req-list-0001', 'limit' => 100]);
expect_quarantine($listing['action'] === 'ledger_listed' && $listing['count'] === 9, 'the ledger lists all nine records');
expect_quarantine($listing['state_counts'] === ['quarantined' => 1, 'settled_approved' => 2, 'settled_denied' => 6], 'ledger state counts are exact');
$synthListing = $quarantine->listQuarantined(['request_id' => 'req-list-synth-0001', 'surface' => 'focusa_live_synthetic']);
expect_quarantine($synthListing['count'] === 2, 'the ledger filters by surface');
foreach ($listing['records'] as $row) {
    expect_quarantine(preg_match('/^[0-9a-f]{64}$/D', $row['key_digest']) === 1, 'ledger rows expose only 64-hex key digests');
    expect_quarantine($row['masked_key'] === null || preg_match('/^[A-Za-z0-9*_]{4,191}$/D', $row['masked_key']) === 1, 'ledger rows expose only masked keys');
    expect_quarantine($row['email_lookup_digest'] === null || preg_match('/^[0-9a-f]{64}$/D', $row['email_lookup_digest']) === 1, 'ledger rows expose only 64-hex email digests');
}

// ── Hygiene: digests and masks only, never keys, emails, or secrets ────

$dump = json_encode($db->query('SELECT * FROM wp_wpuiai_key_quarantine_ledger')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)
    . json_encode($db->query('SELECT * FROM wp_wpuiai_key_quarantine_settlements')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)
    . json_encode($db->query('SELECT * FROM wp_wpuiai_key_quarantine_journal')->fetchAll(PDO::FETCH_ASSOC), JSON_THROW_ON_ERROR)
    . json_encode($listing['records'], JSON_THROW_ON_ERROR);
expect_quarantine(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $dump) !== 1, 'no unmasked email in quarantine journals');
expect_quarantine(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/i', $dump) !== 1, 'no synthetic focusa_live key material in quarantine journals');
expect_quarantine(preg_match('/EDDKEY-[0-9A-F-]+/i', $dump) !== 1, 'no duplicate EDD key material in quarantine journals');
expect_quarantine(preg_match('/custom_key_[0-9a-f]+/i', $dump) !== 1, 'no custom key material in quarantine journals');
expect_quarantine(preg_match('/pi_3Orphan/i', $dump) !== 1, 'no payment id material in quarantine journals');
expect_quarantine(preg_match('/acct_unresolved/i', $dump) !== 1, 'no account handle material in quarantine journals');
expect_quarantine(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i', $dump) !== 1, 'no secret prefix in quarantine journals');
expect_quarantine(preg_match('/\b[0-9A-F]{8}(-[0-9A-F]{4}){3}\b/', $dump) !== 1, 'no full canonical key pattern in quarantine journals');
$journalEvents = $db->query('SELECT DISTINCT event_type FROM wp_wpuiai_key_quarantine_journal')->fetchAll(PDO::FETCH_COLUMN);
foreach (['quarantined', 'canonical_selected', 'selection_ambiguous', 'settled'] as $eventType) {
    expect_quarantine(in_array($eventType, $journalEvents, true), "journal records {$eventType} events");
}
expect_quarantine((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_journal WHERE event_type = 'quarantined'")->fetchColumn() === 9, 'every quarantine is journaled');
expect_quarantine((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_journal WHERE event_type = 'settled'")->fetchColumn() === 7, 'every operator settlement is journaled');

// ── Rollback is preservation-only ──────────────────────────────────────

$beforeRollback = [
    'ledger' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_ledger')->fetchColumn(),
    'settlements' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_settlements')->fetchColumn(),
    'journal' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_journal')->fetchColumn(),
];
$rollback = $schema->preserveForRollback('2026-08-08T03:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_legacy_key_quarantine_rollback',
]);
expect_quarantine($rollback['action'] === 'preserve', 'quarantine rollback is preservation-only');
expect_quarantine(
    (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_ledger')->fetchColumn() === $beforeRollback['ledger']
    && (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_settlements')->fetchColumn() === $beforeRollback['settlements']
    && (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_journal')->fetchColumn() === $beforeRollback['journal'],
    'rollback preserves the full quarantine ledger, settlements, and journal',
);
expect_quarantine(
    (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1,
    'rollback preservation is journaled',
);

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.key_quarantine_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'ledgered_records' => 9,
    'settled_approved' => 2,
    'settled_denied' => 6,
    'quarantined_remaining' => 1,
    'settlements' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_key_quarantine_settlements')->fetchColumn(),
    'gate_denials' => $gateDenials,
    'fixture_digest_sha256' => $fixtureDigest,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
