<?php
declare(strict_types=1);

require_once dirname(__DIR__) . '/docs/contracts/spec152e-authority-node.v1.php';

$positive = 0;
$negative = 0;

function expect_node(bool $condition, string $message): void
{
    global $positive;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
    $positive++;
}

function expect_node_throws(callable $operation, string $exception, string $message): void
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

function expect_domain(callable $operation, string $code, string $message): void
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

function expect_no_throw(callable $operation, string $message): void
{
    try {
        $operation();
    } catch (Throwable $error) {
        expect_node(false, $message . ' (' . get_class($error) . ': ' . $error->getMessage() . ')');
    }
}

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$migration = new FocusaSpec152eAuthorityNodeMigration($db, 'wp_');
$migration->migrate('2026-08-08T05:00:00Z', [
    'source' => 'candidate_node_repository',
    'work_item' => 'focusa-vbcqu.20.13.34',
]);
$migration->migrate('2026-08-08T05:01:00Z', [
    'source' => 'repeat_must_preserve_first_schema_application',
]);

$migrationRows = $db->query('SELECT * FROM wp_wpuiai_authority_node_schema_migrations')->fetchAll(PDO::FETCH_ASSOC);
expect_node(count($migrationRows) === 1, 'repeated migration records one schema version');
expect_node($migrationRows[0]['applied_at'] === '2026-08-08T05:00:00Z', 'repeated migration preserves first applied timestamp');
expect_node(str_contains($migrationRows[0]['migration_provenance'], 'candidate_node_repository'), 'repeated migration preserves first provenance');

$columns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_authority_nodes)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $columns[$column['name']] = $column;
}
$required = [
    'node_uuid', 'account_uuid', 'edd_license_id', 'product_code', 'device_public_key',
    'assurance_class', 'status', 'status_reason', 'activated_at', 'last_seen_at',
    'deactivated_at', 'reservation_id', 'settlement_id', 'migration_provenance',
    'created_at', 'updated_at',
];
foreach ($required as $field) {
    expect_node(isset($columns[$field]), "node schema contains {$field}");
}
$reservationColumns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_authority_node_reservations)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $reservationColumns[$column['name']] = $column;
}
foreach (['reservation_id', 'node_uuid', 'account_uuid', 'edd_license_id', 'product_code', 'node_limit', 'state', 'idempotency_key', 'request_digest', 'reserved_at', 'settled_at', 'released_at', 'settlement_id'] as $field) {
    expect_node(isset($reservationColumns[$field]), "reservation schema contains {$field}");
}
$limitColumns = [];
foreach ($db->query('PRAGMA table_info(wp_wpuiai_authority_node_limits)')->fetchAll(PDO::FETCH_ASSOC) as $column) {
    $limitColumns[$column['name']] = $column;
}
foreach (['account_uuid', 'product_code', 'node_limit', 'reserved_count'] as $field) {
    expect_node(isset($limitColumns[$field]), "limit ledger schema contains {$field}");
}

// Canonical truth the node repository reads (created by prior atoms in production).
$db->exec("CREATE TABLE wp_wpuiai_authority_accounts (
    account_uuid TEXT NOT NULL PRIMARY KEY,
    edd_customer_id BIGINT NOT NULL UNIQUE,
    status VARCHAR(32) NOT NULL,
    status_reason VARCHAR(191) NOT NULL
)");
$db->exec("CREATE TABLE wp_edd_licenses (
    id BIGINT NOT NULL PRIMARY KEY,
    license_key TEXT NOT NULL,
    customer_id BIGINT NOT NULL,
    activation_limit BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL,
    expiration TEXT NULL
)");
$seedAccount = static function (string $uuid, int $customer, string $status, string $reason) use ($db): void {
    $db->prepare('INSERT INTO wp_wpuiai_authority_accounts (account_uuid, edd_customer_id, status, status_reason) VALUES (:uuid, :customer, :status, :reason)')
        ->execute([':uuid' => $uuid, ':customer' => $customer, ':status' => $status, ':reason' => $reason]);
};
$seedLicense = static function (int $id, int $customer, int $limit, string $status, ?string $expiration) use ($db): void {
    $db->prepare('INSERT INTO wp_edd_licenses (id, license_key, customer_id, activation_limit, status, expiration) VALUES (:id, :key, :customer, :limit, :status, :expiration)')
        ->execute([
            ':id' => $id,
            ':key' => 'FOCUSA-SEED-' . str_pad((string) $id, 4, '0', STR_PAD_LEFT),
            ':customer' => $customer,
            ':limit' => $limit,
            ':status' => $status,
            ':expiration' => $expiration,
        ]);
};

$accountA = '018f47c2-6ac0-7b16-8d1a-4e93df5a01aa';
$accountB = '018f47c2-6ac0-7b16-8d1a-4e93df5a01bb';
$accountC = '018f47c2-6ac0-7b16-8d1a-4e93df5a01cc';
$accountD = '018f47c2-6ac0-7b16-8d1a-4e93df5a01dd';
$seedAccount($accountA, 41001, 'active', 'mailbox_verified');
$seedAccount($accountB, 41002, 'active', 'account_promoted');
$seedAccount($accountC, 41003, 'active', 'email_verification_pending');
$seedAccount($accountD, 41004, 'suspended', 'refunded');
$seedLicense(9001, 41001, 3, 'active', null);                  // account A usable, limit 3, lifetime
$seedLicense(9002, 41002, 2, 'active', '2027-08-08 00:00:00'); // account B usable, limit 2
$seedLicense(9003, 41003, 1, 'active', null);                  // account C (unverified)
$seedLicense(9004, 41004, 1, 'revoked', null);                 // revoked
$seedLicense(9005, 41001, 3, 'active', '2020-01-01 00:00:00'); // expired
$seedLicense(9006, 41001, 0, 'active', null);                  // zero capacity

$clockTick = 0;
$clock = static function () use (&$clockTick): string {
    $timestamp = (new DateTimeImmutable('2026-08-08T05:02:00Z'))
        ->modify('+' . $clockTick . ' minutes')
        ->format('Y-m-d\TH:i:s\Z');
    $clockTick++;
    return $timestamp;
};
$repository = new FocusaSpec152eAuthorityNodeRepository($db, $migration, $clock);
$deviceKey = static fn(string $seed): string => substr(strtr(base64_encode(hash('sha256', 'device-' . $seed, true)), '+/', '-_'), 0, 43);
$provenance = static fn(string $tag): array => ['source' => 'synthetic_node_fixture', 'fixture' => $tag];
$attempt = static fn(array $overrides): array => array_merge([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1001',
    'account_uuid' => $accountA,
    'edd_license_id' => 9001,
    'product_code' => 'focusa_operator_lifetime_v1',
    'device_public_key' => $deviceKey('a1'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-reg-0001',
    'migration_provenance' => $provenance('a1'),
], $overrides);

// ---- Positive: atomic registration, binding, and settlement ----
$node1 = $repository->registerNode($attempt([]));
expect_node($node1['node_uuid'] === '018f47c2-6ac0-7b16-8d1a-4e93df5a1001', 'registerNode returns the requested opaque node UUID');
expect_node($node1['account_uuid'] === $accountA, 'node binds the verified authority account');
expect_node((int) $node1['edd_license_id'] === 9001, 'node binds the canonical EDD license');
expect_node($node1['product_code'] === 'focusa_operator_lifetime_v1', 'node binds the server-owned product code');
expect_node(hash_equals($node1['device_public_key'], $deviceKey('a1')), 'node binds the device public key');
expect_node($node1['assurance_class'] === 'device_key_v1', 'node records the device assurance class');
expect_node($node1['status'] === 'active', 'node is active after registration');
expect_node($node1['activated_at'] === '2026-08-08T05:03:00Z', 'activation timestamp comes from the authority clock');
expect_node($node1['last_seen_at'] === null, 'last-seen is null until a heartbeat');
expect_node($node1['settlement_id'] !== null, 'node carries a settlement reference');
expect_node(str_starts_with($node1['reservation_id'], 'nr_'), 'node carries a bounded reservation reference');
$reservation1 = $db->query("SELECT * FROM wp_wpuiai_authority_node_reservations WHERE reservation_id = '{$node1['reservation_id']}'")->fetch(PDO::FETCH_ASSOC);
expect_node($reservation1['state'] === 'settled', 'registration settles its reservation');
expect_node((int) $reservation1['node_limit'] === 3, 'reservation records the server-owned node limit from the EDD license');
expect_node($reservation1['settled_at'] === '2026-08-08T05:03:00Z', 'settlement timestamp is journaled');
$ledger = $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1');
expect_node((int) $ledger['node_limit'] === 3, 'limit ledger stores the EDD product node limit');
expect_node((int) $ledger['reserved_count'] === 1, 'limit ledger reserves exactly one slot');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn() === 1, 'one node row exists');

// Idempotent replay of the exact same registration.
$replayed = $repository->registerNode($attempt([]));
expect_node($replayed['node_uuid'] === $node1['node_uuid'], 'idempotent replay returns the same node');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn() === 1, 'replay creates no duplicate node');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 1, 'replay does not double-reserve');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn() === 1, 'replay creates no duplicate reservation');

// Changed reuse of the same idempotency key fails closed.
$conflict = $attempt(['device_public_key' => $deviceKey('a1-changed'), 'idempotency_key' => 'idem-reg-0001']);
expect_domain(static fn() => $repository->registerNode($conflict), 'IDEMPOTENCY_CONFLICT', 'changed request cannot reuse a registration key');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn() === 1, 'idempotency conflict is atomic');

// Fill to the limit with two more devices.
$node2 = $repository->registerNode($attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1002',
    'device_public_key' => $deviceKey('a2'),
    'idempotency_key' => 'idem-reg-0002',
    'migration_provenance' => $provenance('a2'),
]));
expect_node($node2['status'] === 'active', 'second device registers within the limit');
$node3 = $repository->registerNode($attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1003',
    'device_public_key' => $deviceKey('a3'),
    'idempotency_key' => 'idem-reg-0003',
    'migration_provenance' => $provenance('a3'),
]));
expect_node($node3['status'] === 'active', 'third device registers at the limit');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 3, 'ledger is at the limit after three nodes');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn() === 3, 'three settled reservations exist');

// The fourth device cannot exceed the EDD product node limit.
$beforeExhaustion = (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn();
expect_domain(
    static fn() => $repository->registerNode($attempt([
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1004',
        'device_public_key' => $deviceKey('a4'),
        'idempotency_key' => 'idem-reg-0004',
        'migration_provenance' => $provenance('a4'),
    ])),
    'NODE_LIMIT_EXHAUSTED',
    'fourth activation beyond the node limit is denied'
);
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn() === $beforeExhaustion, 'denied attempt creates no reservation');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn() === 3, 'denied attempt creates no node');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 3, 'denied attempt leaves the counter exactly at the limit');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_idempotency')->fetchColumn() === 3, 'denied attempt journals no idempotency');

// ---- Race fixture: the reservation counter (not the node count) is the limit authority ----
$raceReservation1 = $repository->reserve([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2001',
    'account_uuid' => $accountB,
    'edd_license_id' => 9002,
    'product_code' => 'uiai_operator_lifetime_v1',
    'device_public_key' => $deviceKey('b1'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-race-reserve-0001',
    'migration_provenance' => $provenance('b1'),
]);
expect_node($raceReservation1['state'] === 'reserved', 'two-phase reserve creates a pending reservation');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'pending reservation occupies one slot');
$raceReplay = $repository->reserve([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2001',
    'account_uuid' => $accountB,
    'edd_license_id' => 9002,
    'product_code' => 'uiai_operator_lifetime_v1',
    'device_public_key' => $deviceKey('b1'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-race-reserve-0001',
    'migration_provenance' => $provenance('b1'),
]);
expect_node($raceReplay['reservation_id'] === $raceReservation1['reservation_id'], 'reserve replay returns the identical reservation');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'reserve replay does not double-reserve');
$raceReservation2 = $repository->reserve([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2002',
    'account_uuid' => $accountB,
    'edd_license_id' => 9002,
    'product_code' => 'uiai_operator_lifetime_v1',
    'device_public_key' => $deviceKey('b2'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-race-reserve-0002',
    'migration_provenance' => $provenance('b2'),
]);
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 2, 'two pending reservations fill the two-slot limit');

// A third concurrent activation is denied even though no node row exists yet:
// the atomic counter is the authority.
$beforeRace = (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn();
expect_domain(
    static fn() => $repository->registerNode([
        'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2003',
        'account_uuid' => $accountB,
        'edd_license_id' => 9002,
        'product_code' => 'uiai_operator_lifetime_v1',
        'device_public_key' => $deviceKey('b3'),
        'assurance_class' => 'device_key_v1',
        'idempotency_key' => 'idem-race-reg-0003',
        'migration_provenance' => $provenance('b3'),
    ]),
    'NODE_LIMIT_EXHAUSTED',
    'concurrent activation cannot exceed the reserved limit'
);
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn() === $beforeRace, 'denied concurrent attempt leaks no reservation');
expect_node((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$accountB}'")->fetchColumn() === 0, 'denied concurrent attempt creates no node');

expect_node((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_node_idempotency WHERE idempotency_key = 'idem-race-reg-0003'")->fetchColumn() === 0, 'denied attempt leaves no idempotency residue for retry');

// Release settles exactly: one committed reservation frees one slot.
$released = $repository->releaseReservation((string) $raceReservation1['reservation_id'], 'operator_cancelled', 'idem-race-release-0001');
expect_node($released['state'] === 'released', 'release marks the reservation released');
expect_node($released['released_at'] !== null, 'release journals the release timestamp');
expect_node($released['release_reason'] === 'operator_cancelled', 'release records the explicit reason');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'release decrements the counter exactly once');
$releaseReplay = $repository->releaseReservation((string) $raceReservation1['reservation_id'], 'operator_cancelled', 'idem-race-release-0001');
expect_node($releaseReplay['state'] === 'released', 'release replay is idempotent');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'release replay never double-decrements');
expect_domain(
    static fn() => $repository->releaseReservation((string) $raceReservation1['reservation_id'], 'again', 'idem-race-release-0002'),
    'RESERVATION_NOT_PENDING',
    'a settled/released reservation cannot be released twice'
);

// The freed slot now accepts the previously denied device.
$raceNode = $repository->registerNode([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2003',
    'account_uuid' => $accountB,
    'edd_license_id' => 9002,
    'product_code' => 'uiai_operator_lifetime_v1',
    'device_public_key' => $deviceKey('b3'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-race-reg-0003',
    'migration_provenance' => $provenance('b3'),
]);
expect_node($raceNode['status'] === 'active', 'freed slot accepts a new activation');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 2, 'ledger back at the limit after settlement');

// Two-phase settlement: a committed reservation settles into an active node.
$settledNode = $repository->settleReservation((string) $raceReservation2['reservation_id'], 'ns_settle_b2', 'idem-race-settle-0002');
expect_node($settledNode['status'] === 'active', 'two-phase settlement activates the node');
expect_node(hash_equals($settledNode['device_public_key'], $deviceKey('b2')), 'settled node carries the reserved device key');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 2, 'settled reservation keeps its slot');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] <= 2, 'settlement never exceeds the limit');
$settleReplay = $repository->settleReservation((string) $raceReservation2['reservation_id'], 'ns_settle_b2', 'idem-race-settle-0002');
expect_node($settleReplay['node_uuid'] === $settledNode['node_uuid'], 'settlement replay is idempotent');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations WHERE state = \'settled\'')->fetchColumn() === 5, 'settled reservation state is journaled');

// Explicit management: deactivate preserves history and frees the slot.
expect_domain(
    static fn() => $repository->deactivateNode([
        'node_uuid' => (string) $raceNode['node_uuid'],
        'account_uuid' => $accountA,
        'status_reason' => 'owner_choice',
        'idempotency_key' => 'idem-deact-wrong-account',
    ]),
    'NODE_NOT_FOUND',
    'a different account cannot deactivate another account node'
);
$deactivated = $repository->deactivateNode([
    'node_uuid' => (string) $raceNode['node_uuid'],
    'account_uuid' => $accountB,
    'status_reason' => 'owner_choice',
    'idempotency_key' => 'idem-deact-b3',
]);
expect_node($deactivated['status'] === 'deactivated', 'deactivation marks the node deactivated');
expect_node($deactivated['deactivated_at'] !== null, 'deactivation journals the timestamp');
expect_node((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE node_uuid = '{$raceNode['node_uuid']}'")->fetchColumn() === 1, 'device history is preserved, never deleted');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'deactivation frees the slot');
$deactivatedReplay = $repository->deactivateNode([
    'node_uuid' => (string) $raceNode['node_uuid'],
    'account_uuid' => $accountB,
    'status_reason' => 'owner_choice',
    'idempotency_key' => 'idem-deact-b3',
]);
expect_node($deactivatedReplay['status'] === 'deactivated', 'deactivation replay is idempotent');
expect_node((int) $repository->limitLedger($accountB, 'uiai_operator_lifetime_v1')['reserved_count'] === 1, 'deactivation replay never double-frees');
expect_domain(
    static fn() => $repository->deactivateNode([
        'node_uuid' => (string) $raceNode['node_uuid'],
        'account_uuid' => $accountB,
        'status_reason' => 'owner_choice',
        'idempotency_key' => 'idem-deact-b3-again',
    ]),
    'NODE_NOT_ACTIVE',
    'an already-deactivated node cannot be deactivated again'
);
expect_domain(
    static fn() => $repository->recordLastSeen((string) $raceNode['node_uuid'], 'idem-lastseen-b3'),
    'NODE_NOT_ACTIVE',
    'last-seen heartbeats require an active node'
);

// The same device may register a fresh node after explicit deactivation.
$reRegistered = $repository->registerNode([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a2004',
    'account_uuid' => $accountB,
    'edd_license_id' => 9002,
    'product_code' => 'uiai_operator_lifetime_v1',
    'device_public_key' => $deviceKey('b3'),
    'assurance_class' => 'device_key_v1',
    'idempotency_key' => 'idem-race-reg-b3-again',
    'migration_provenance' => $provenance('b3-again'),
]);
expect_node($reRegistered['status'] === 'active', 'deactivated device can register a new node');
expect_node((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_nodes WHERE account_uuid = '{$accountB}'")->fetchColumn() === 3, 'device history accumulates both old and new nodes');

// Last-seen heartbeat.
$touched = $repository->recordLastSeen((string) $reRegistered['node_uuid'], 'idem-lastseen-b3-again');
expect_node($touched['last_seen_at'] !== null, 'heartbeat records last-seen');
$touchedReplay = $repository->recordLastSeen((string) $reRegistered['node_uuid'], 'idem-lastseen-b3-again');
expect_node($touchedReplay['last_seen_at'] === $touched['last_seen_at'], 'heartbeat replay is idempotent');

// Device history listing includes active and deactivated nodes.
$history = $repository->listNodes($accountB);
expect_node(count($history) === 3, 'listNodes returns the full device history');
$statuses = array_column($history, 'status');
expect_node(in_array('active', $statuses, true) && in_array('deactivated', $statuses, true), 'device history preserves both states');

// Account A: deactivate node1 and confirm the ledger slot frees for the 4th device.
$node1Deactivated = $repository->deactivateNode([
    'node_uuid' => (string) $node1['node_uuid'],
    'account_uuid' => $accountA,
    'status_reason' => 'owner_choice',
    'idempotency_key' => 'idem-deact-a1',
]);
expect_node($node1Deactivated['status'] === 'deactivated', 'account A node deactivates cleanly');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 2, 'deactivation releases one of three slots');
$node4 = $repository->registerNode($attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a1004',
    'device_public_key' => $deviceKey('a4'),
    'idempotency_key' => 'idem-reg-0004',
    'migration_provenance' => $provenance('a4'),
]));
expect_node($node4['status'] === 'active', 'freed slot registers the previously denied device');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 3, 'ledger at the limit again');

// ---- Negative matrix: only a verified account and usable EDD license register nodes ----
$unverified = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3001',
    'account_uuid' => $accountC,
    'edd_license_id' => 9003,
    'device_public_key' => $deviceKey('c1'),
    'idempotency_key' => 'idem-neg-unverified',
    'migration_provenance' => $provenance('c1'),
]);
expect_domain(static fn() => $repository->registerNode($unverified), 'EMAIL_VERIFICATION_REQUIRED', 'unverified account cannot register a node');
$suspended = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3002',
    'account_uuid' => $accountD,
    'edd_license_id' => 9004,
    'device_public_key' => $deviceKey('d1'),
    'idempotency_key' => 'idem-neg-suspended',
    'migration_provenance' => $provenance('d1'),
]);
expect_domain(static fn() => $repository->registerNode($suspended), 'EMAIL_VERIFICATION_REQUIRED', 'suspended account cannot register a node');
$unknownAccount = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3003',
    'account_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a0eee',
    'device_public_key' => $deviceKey('x1'),
    'idempotency_key' => 'idem-neg-unknown-account',
    'migration_provenance' => $provenance('x1'),
]);
expect_domain(static fn() => $repository->registerNode($unknownAccount), 'ACCOUNT_NOT_FOUND', 'unknown account fails closed');
$unknownLicense = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3004',
    'edd_license_id' => 9999,
    'device_public_key' => $deviceKey('a5'),
    'idempotency_key' => 'idem-neg-unknown-license',
    'migration_provenance' => $provenance('a5'),
]);
expect_domain(static fn() => $repository->registerNode($unknownLicense), 'EDD_LICENSE_UNUSABLE', 'unknown license fails closed');
$revokedLicense = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3005',
    'edd_license_id' => 9004,
    'device_public_key' => $deviceKey('a6'),
    'idempotency_key' => 'idem-neg-revoked-license',
    'migration_provenance' => $provenance('a6'),
]);
expect_domain(static fn() => $repository->registerNode($revokedLicense), 'EDD_LICENSE_UNUSABLE', 'revoked license fails closed');
$mismatched = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3006',
    'edd_license_id' => 9002,
    'device_public_key' => $deviceKey('a7'),
    'idempotency_key' => 'idem-neg-mismatch',
    'migration_provenance' => $provenance('a7'),
]);
expect_domain(static fn() => $repository->registerNode($mismatched), 'LICENSE_ACCOUNT_MISMATCH', 'a key and unrelated account cannot activate a node');
$expired = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3007',
    'edd_license_id' => 9005,
    'device_public_key' => $deviceKey('a8'),
    'idempotency_key' => 'idem-neg-expired',
    'migration_provenance' => $provenance('a8'),
]);
expect_domain(static fn() => $repository->registerNode($expired), 'EDD_LICENSE_UNUSABLE', 'expired license fails closed');
$zeroCapacity = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3008',
    'edd_license_id' => 9006,
    'device_public_key' => $deviceKey('a9'),
    'idempotency_key' => 'idem-neg-zero-capacity',
    'migration_provenance' => $provenance('a9'),
]);
expect_domain(static fn() => $repository->registerNode($zeroCapacity), 'EDD_LICENSE_UNUSABLE', 'zero-capacity license fails closed');
$unknownProduct = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a3009',
    'product_code' => 'focusa_operator_lifetime_v9',
    'device_public_key' => $deviceKey('a10'),
    'idempotency_key' => 'idem-neg-product',
    'migration_provenance' => $provenance('a10'),
]);
expect_domain(static fn() => $repository->registerNode($unknownProduct), 'PRODUCT_MAPPING_REQUIRED', 'unknown product code fails closed');
$badDeviceKey = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a300a',
    'device_public_key' => 'not-a-real-device-key!',
    'idempotency_key' => 'idem-neg-device-key',
    'migration_provenance' => $provenance('a11'),
]);
expect_domain(static fn() => $repository->registerNode($badDeviceKey), 'NODE_PUBLIC_KEY_REQUIRED', 'malformed device key fails closed');
$deviceInUse = $attempt([
    'node_uuid' => '018f47c2-6ac0-7b16-8d1a-4e93df5a300b',
    'device_public_key' => $deviceKey('a2'),
    'idempotency_key' => 'idem-neg-device-in-use',
    'migration_provenance' => $provenance('a12'),
]);
expect_domain(static fn() => $repository->registerNode($deviceInUse), 'DEVICE_PUBLIC_KEY_IN_USE', 'an active device cannot register twice');
expect_node((int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn() === 7, 'all negative attempts leave node state unchanged');
expect_node((int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'] === 3, 'all negative attempts leave the counter unchanged');

// Input validation fails closed.
expect_node_throws(
    static fn() => $repository->registerNode($attempt(['node_uuid' => 'not-a-uuid', 'idempotency_key' => 'idem-neg-bad-uuid'])),
    InvalidArgumentException::class,
    'malformed node UUID is rejected'
);
expect_node_throws(
    static fn() => $repository->registerNode($attempt(['idempotency_key' => 'short'])),
    InvalidArgumentException::class,
    'unbounded idempotency key is rejected'
);
expect_node_throws(
    static fn() => $repository->registerNode($attempt(['edd_license_id' => 0, 'idempotency_key' => 'idem-neg-bad-license'])),
    InvalidArgumentException::class,
    'non-positive license ID is rejected'
);
expect_node_throws(
    static fn() => $repository->deactivateNode([
        'node_uuid' => (string) $node1['node_uuid'],
        'account_uuid' => $accountA,
        'status_reason' => "bad\x00reason",
        'idempotency_key' => 'idem-neg-bad-reason',
    ]),
    InvalidArgumentException::class,
    'unbounded status reason is rejected'
);

// Rollback is preservation-only and journaled.
$beforeRollback = $repository->findNodeByUuid((string) $node2['node_uuid']);
$rollback = $migration->preserveForRollback('2026-08-08T06:00:00Z', [
    'software_target' => 'prior_candidate',
    'reason' => 'synthetic_rollback_proof',
]);
expect_node($rollback['action'] === 'preserve', 'rollback contract is preservation-only');
$afterRollback = $repository->findNodeByUuid((string) $node2['node_uuid']);
expect_node($afterRollback === $beforeRollback, 'rollback preserves nodes, bindings, reservations, counters, and provenance');
expect_node((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_authority_node_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation is journaled');

// Redaction: no email, no license key, no unmasked secrets in node state.
$scan = '';
foreach (['wp_wpuiai_authority_nodes', 'wp_wpuiai_authority_node_reservations', 'wp_wpuiai_authority_node_limits', 'wp_wpuiai_authority_node_idempotency'] as $table) {
    foreach ($db->query("SELECT * FROM {$table}")->fetchAll(PDO::FETCH_ASSOC) as $row) {
        $scan .= json_encode($row, JSON_THROW_ON_ERROR);
    }
}
$scan .= json_encode([
    $node1, $node2, $node3, $node4, $raceNode, $reRegistered, $settledNode,
    $reservation1, $raceReservation1, $raceReservation2, $released, $history,
], JSON_THROW_ON_ERROR);
expect_node(strpos($scan, '@') === false, 'no email address appears in any node record or result');
expect_node(strpos($scan, 'FOCUSA-SEED') === false, 'no EDD license key material appears in any node record or result');
expect_node(strpos($scan, 'cus_') === false && strpos($scan, 'sk_') === false, 'no secret material appears in any node record or result');

// Server-owned registry agreement: the allowlist matches the frozen spec 172 registry.
$registry = require dirname(__DIR__) . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$protectedCodes = [];
foreach ($registry['protected_offers'] as $offer) {
    $protectedCodes[] = $offer['public_code'];
    expect_node((int) $offer['node_limit'] === 3, 'frozen registry node limit is server-owned at 3');
}
sort($protectedCodes);
$allowlist = FocusaSpec152eAuthorityNodeRepository::SERVER_OWNED_PRODUCTS;
sort($allowlist);
expect_node($allowlist === $protectedCodes, 'node product allowlist equals the frozen registry protected offers');

$result = [
    'schema' => 'focusa.spec152e.authority_node_reservation_validation.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'nodes_registered' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_nodes')->fetchColumn(),
    'reservations' => (int) $db->query('SELECT COUNT(*) FROM wp_wpuiai_authority_node_reservations')->fetchColumn(),
    'ledger_account_a' => (int) $repository->limitLedger($accountA, 'focusa_operator_lifetime_v1')['reserved_count'],
    'limit_fixtures' => ['atomic_register', 'two_phase_reserve_settle_release', 'concurrent_activation_denied', 'device_history_preserved'],
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($result, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
