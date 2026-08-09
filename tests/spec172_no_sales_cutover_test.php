<?php
// 172.05.01 Execute no-sales clean cutover and quarantine legacy products.
// Exact verification: php tests/spec172_no_sales_cutover_test.php
//
// Drives the Spec 172 no-sales cutover canary
// (docs/contracts/spec172-no-sales-cutover.v1.php) against the CURRENT
// accepted state: the migration-preserving no-sales inventory decision
// (docs/contracts/spec172-no-sales-inventory.v1.json: zero_sales_proven=false,
// clean_cutover_allowed=false, status=migration_preserving_path_selected) is
// REQUIRED, the dedicated Operator v1 EDD mappings (Downloads 458/459/460,
// checkout disabled, sale_status approved_not_yet_enabled) are never enabled
// before validation, direct install-site/Gravity issuance is disabled with the
// exact authority denial codes, and old WPUIAI / Download 453 / synthetic /
// credit-pack / refunded / revoked records are quarantined, retired, or
// preserved without ever granting an Operator v1 License Type. The canary also
// proves: a missing/malformed proof fails closed with zero writes
// (ZERO_SALES_PROOF_REQUIRED); a genuine sale stops the cutover and requires a
// customer-rights mapping (GENUINE_SALE_REQUIRES_CUSTOMER_RIGHTS_MAPPING);
// the accepted clean-cutover canary enables mappings ONLY after validation
// while checkout stays disabled; dry runs write zero rows; replays return the
// byte-identical stored receipt with zero writes (idempotent); the
// reconciliation receipt proves one canonical paid authority, zero split
// issuance, zero legacy grants, preserved refund/revoke truth, and a valid
// journal chain; and the rollback rehearsal proves rollback can never restore
// split issuance or stale refund truth (preservation-only schema, no enable
// path). All output is redacted: no raw email, key, token, customer row,
// credential, or card data.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec172-no-sales-cutover.v1.php';
require_once $root . '/docs/contracts/spec152e-authority-cutover.v1.php';
require_once $root . '/docs/contracts/spec152e-install-facade-routes.v1.php';

$inventory = json_decode((string) file_get_contents($root . '/docs/contracts/spec172-no-sales-inventory.v1.json'), true, 512, JSON_THROW_ON_ERROR);
$dedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';
$registry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$cutoverFixture = json_decode((string) file_get_contents($root . '/docs/contracts/spec152e-authority-cutover-fixture.v1.json'), true, 512, JSON_THROW_ON_ERROR);

$positiveChecks = 0;
$negativeChecks = 0;

function expect_cutover(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_cutover_throws(callable $operation, string $code, string $message): void
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

function assert_no_sensitive(array $payload, string $message): void
{
    $raw = json_encode($payload, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    expect_cutover(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $raw) !== 1, "{$message}: no raw email");
    expect_cutover(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $raw) !== 1, "{$message}: no payment key");
    expect_cutover(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/', $raw) !== 1, "{$message}: no synthetic license key");
    expect_cutover(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $raw) !== 1, "{$message}: no card data");
}

// ── 1. Read-only cross-checks against the accepted authority contracts ──

expect_cutover($inventory['schema'] === 'focusa.spec172.no_sales_inventory.v1', 'inventory schema is canonical');
expect_cutover($inventory['inventory_id'] === 'focusa-vbcqu.20.15.6', 'inventory is the accepted no-sales inventory atom');
expect_cutover($inventory['decision']['zero_sales_proven'] === false, 'inventory proves zero sales is NOT yet proven');
expect_cutover($inventory['decision']['clean_cutover_allowed'] === false, 'inventory forbids destructive clean cutover');
expect_cutover($inventory['decision']['status'] === 'migration_preserving_path_selected', 'inventory selected the migration-preserving path');
expect_cutover($inventory['decision']['implementation_may_continue'] === true, 'implementation may continue on the preserving path');

expect_cutover($dedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1', 'dedicated downloads contract schema');
expect_cutover($dedicated['owner'] === 'WPUIAI/wpuiai', 'dedicated mappings are server-owned');
expect_cutover(count($dedicated['records']) === 3, 'exactly three dedicated Operator v1 records');
expect_cutover($dedicated['authority']['checkout_enabled'] === false && $dedicated['authority']['checkout_block_reason'] === 'awaiting_validation_pass', 'dedicated checkout stays disabled until validation');
expect_cutover($dedicated['authority']['sale_status'] === 'approved_not_yet_enabled', 'dedicated sales are approved but not yet enabled');
expect_cutover((int) $dedicated['authority']['forbidden_implicit_download'] === 453, 'Download 453 is the explicit forbidden implicit mapping');
expect_cutover(in_array(453, array_map('intval', $dedicated['authority']['legacy_download_ids']), true), 'Download 453 is on the never-grant legacy list');

$protectedOffersByCode = [];
foreach ($registry['protected_offers'] as $offer) {
    $protectedOffersByCode[(string) $offer['public_code']] = $offer;
}
expect_cutover(count($protectedOffersByCode) === 3, 'canonical registry has three protected offers');
foreach ($protectedOffersByCode as $offer) {
    expect_cutover(($offer['mapping_status'] ?? '') === 'approved_policy_blocked_edd_mapping', "{$offer['public_code']} mapping stays blocked pending validation");
    expect_cutover($offer['checkout_enabled'] === false, "{$offer['public_code']} checkout stays disabled");
    expect_cutover(($offer['sale_status'] ?? '') === 'approved_not_yet_enabled', "{$offer['public_code']} sale status not yet enabled");
    expect_cutover((int) ($offer['edd_download_id'] ?? 0) !== 453, "{$offer['public_code']} never maps Download 453");
}
$catalog453 = array_values(array_filter(
    $registry['current_edd_catalog']['entries'],
    static fn(array $entry): bool => (int) $entry['download_id'] === 453
));
expect_cutover(count($catalog453) === 1, 'Download 453 appears exactly once in the legacy catalog');
expect_cutover($catalog453[0]['entitlement_disposition'] === 'quarantine' && $catalog453[0]['reason'] === 'implicit_focusa_mapping_forbidden', 'Download 453 stays quarantined with the explicit forbidden reason');
$legacyClasses = [];
foreach ($registry['legacy_record_classes'] as $class) {
    $legacyClasses[(string) $class['id']] = $class;
}
expect_cutover(in_array($legacyClasses['edd_focusa_live_synthetic']['disposition'] ?? '', ['quarantine'], true), 'synthetic Focusa records are quarantined');
expect_cutover(in_array($legacyClasses['install_stripe_active_focusa']['disposition'] ?? '', ['migrate'], true), 'active Stripe records are migration-class preserved');
expect_cutover(($legacyClasses['install_refunded_focusa']['disposition'] ?? '') === 'retire' && ($legacyClasses['install_revoked_focusa']['disposition'] ?? '') === 'retire', 'refunded/revoked records are retired terminal');

expect_cutover(count($cutoverFixture['denied_issuance_surfaces']) === 6, 'cutover fixture denies six direct issuance surfaces');
expect_cutover($cutoverFixture['authority']['new_issuance'] === 'edd_authority_only', 'cutover fixture asserts EDD-only issuance');
$fixtureRetained = $cutoverFixture['retained_recovery_surfaces'];
expect_cutover($fixtureRetained !== [] && count($fixtureRetained) >= 9, 'cutover fixture retains bounded recovery surfaces');
foreach ($fixtureRetained as $surface) {
    expect_cutover($surface['grants_entitlement'] === false, "{$surface['surface']} retained surface never grants entitlement");
}

$facadeSurfaces = FocusaSpec152eInstallFacadeRoutes::surfaces();
expect_cutover($facadeSurfaces !== [], 'install facade registers activation surfaces');
$localIssuance = ['install_site_create', 'install_site_payment', 'install_site_webhook', 'wpuiai_custom_issue'];
foreach (array_keys($facadeSurfaces) as $facadeSurface) {
    expect_cutover(!in_array($facadeSurface, $localIssuance, true), "install facade surface {$facadeSurface} is an authority proxy, never local issuance");
}

// ── 2. Build the canary input from the server-owned contracts ──────────

$mappings = [];
foreach ($dedicated['records'] as $record) {
    $mappings[] = [
        'public_code' => (string) $record['public_code'],
        'edd_download_id' => (int) $record['edd_download_id'],
        'edd_price_id' => (string) $record['edd_price_id'],
        'price_usd' => (string) $record['price_usd'],
        'checkout_enabled' => (bool) $record['checkout_enabled'],
        'sale_status' => (string) $record['sale_status'],
        'status' => (string) $record['status'],
        'title' => (string) $record['title'],
    ];
}

$issuerDenied = [];
foreach ($cutoverFixture['denied_issuance_surfaces'] as $entry) {
    $name = (string) $entry['surface'];
    expect_cutover(isset(FocusaSpec152eAuthorityCutoverService::DENIED_SURFACES[$name]), "{$name} resolves in the authority cutover contract");
    $issuerDenied[] = [
        'surface' => $name,
        'route' => (string) $entry['route'],
        'denial_code' => FocusaSpec152eAuthorityCutoverService::DENIED_SURFACES[$name]['code'],
        'next_action' => FocusaSpec152eAuthorityCutoverService::DENIED_SURFACES[$name]['next_action'],
    ];
}
$issuerRetained = [];
foreach ($fixtureRetained as $entry) {
    $issuerRetained[] = [
        'surface' => (string) $entry['surface'],
        'route' => (string) $entry['route'],
        'retained_for' => (string) $entry['retained_for'],
        'grants_entitlement' => false,
    ];
}
$issuer = array_merge($issuerDenied, $issuerRetained);

$legacy = [];
$catalogEntry = 1;
foreach ($registry['current_edd_catalog']['entries'] as $entry) {
    $downloadId = (int) $entry['download_id'];
    $legacy[] = [
        'record_handle' => 'rec_legacy_dl_' . str_pad((string) $catalogEntry, 4, '0', STR_PAD_LEFT),
        'download_id' => $downloadId,
        'disposition' => (string) $entry['entitlement_disposition'],
        'reason' => (string) $entry['reason'],
        'evidence_digest' => hash('sha256', 'legacy-download-' . $downloadId),
    ];
    $catalogEntry++;
}
$classRecords = [
    ['record_handle' => 'rec_legacy_synth_0001', 'disposition' => 'quarantine', 'reason' => 'EDD_ORDER_UNVERIFIED'],
    ['record_handle' => 'rec_legacy_api_0001', 'disposition' => 'quarantine', 'reason' => 'operator_and_purchase_evidence_review'],
    ['record_handle' => 'rec_legacy_stripe_0001', 'disposition' => 'migrate', 'reason' => 'verify_stripe_payment_refund_customer_and_product_evidence'],
    ['record_handle' => 'rec_legacy_refund_0001', 'disposition' => 'retire', 'reason' => 'preserve_refund_history_never_reactivate'],
    ['record_handle' => 'rec_legacy_revoke_0001', 'disposition' => 'retire', 'reason' => 'preserve_revocation_history_never_reactivate'],
];
foreach ($classRecords as $record) {
    $record['evidence_digest'] = hash('sha256', 'legacy-class-' . $record['record_handle']);
    $legacy[] = $record;
}
expect_cutover(count($legacy) === count($registry['current_edd_catalog']['entries']) + count($classRecords), 'legacy disposition registry covers catalog + class records');

// ── 3. Setup ──────────────────────────────────────────────────────────

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$schema = new FocusaSpec172NoSalesCutoverSchema($db, 'wp_');
$schema->migrate('2026-08-09T00:00:00Z', ['source' => 'spec172_no_sales_cutover_test']);
$clock = static fn(): string => '2026-08-09T00:00:00Z';
$canary = new FocusaSpec172NoSalesCutoverCanary($db, $schema, $clock);

$provenance = ['source' => 'spec172_no_sales_cutover_test', 'inventory_id' => 'focusa-vbcqu.20.15.6'];

$proofBlocked = [
    'accepted' => true,
    'inventory_id' => 'focusa-vbcqu.20.15.6',
    'zero_sales_proven' => false,
    'clean_cutover_allowed' => false,
    'decision_status' => 'migration_preserving_path_selected',
];

$baseInput = [
    'run_handle' => 'run_spec172_cutover_real_state',
    'zero_sales_proof' => $proofBlocked,
    'dedicated_mappings' => $mappings,
    'issuer_disablements' => $issuer,
    'legacy_records' => $legacy,
    'genuine_sale_observed' => false,
    'migration_provenance' => $provenance,
];

// ── 4. CURRENT REAL STATE: migration-preserving cutover canary ─────────

$runsBefore = $canary->countRows('wpuiai_spec172_cutover_runs');
$result = $canary->canaryCutover($baseInput);
$runsAfter = $canary->countRows('wpuiai_spec172_cutover_runs');
expect_cutover($runsAfter === $runsBefore + 1, 'canary run is recorded exactly once');
expect_cutover($result['replayed'] === false, 'first run is not a replay');
expect_cutover($result['decision'] === 'migration_preserving_path_selected', 'current real state selects the migration-preserving path');

$receipt = $result['receipt'];
expect_cutover($receipt['schema'] === FocusaSpec172NoSalesCutoverCanary::RESULT_SCHEMA, 'cutover receipt schema');
expect_cutover($receipt['cutover_version'] === 'focusa-vbcqu.20.15.32', 'receipt is versioned to this atom');
expect_cutover($receipt['zero_sales_proven'] === false && $receipt['clean_cutover_allowed'] === false, 'receipt reflects unproven zero sales');
expect_cutover($receipt['clean_cutover_blocked_reason'] === 'ZERO_SALES_PROOF_REQUIRED', 'clean cutover is blocked while zero-sales proof is unproven');
expect_cutover($receipt['authority']['canonical_paid_authority'] === 'WPUIAI.com EDD' && $receipt['authority']['split_issuance'] === false, 'one canonical paid authority, zero split issuance');
expect_cutover(preg_match('/^[0-9a-f]{64}$/D', $receipt['idempotency_key']) === 1, 'receipt carries a deterministic idempotency key');
expect_cutover(preg_match('/^[0-9a-f]{64}$/D', $receipt['receipt_digest']) === 1, 'receipt carries a 64-hex digest');

// Dedicated EDD mappings stay blocked and checkout stays disabled.
expect_cutover($receipt['counts']['dedicated_mappings'] === 3, 'three dedicated mappings in the receipt');
expect_cutover($receipt['counts']['mappings_blocked'] === 3 && $receipt['counts']['mappings_enabled'] === 0, 'all dedicated mappings stay blocked pre-validation');
$codesSeen = [];
foreach ($receipt['mappings'] as $mapping) {
    $codesSeen[] = $mapping['public_code'];
    expect_cutover($mapping['mapping_status'] === FocusaSpec172NoSalesCutoverCanary::MAPPING_BLOCKED, "{$mapping['public_code']} mapping blocked");
    expect_cutover($mapping['checkout_enabled'] === false, "{$mapping['public_code']} checkout disabled");
    expect_cutover($mapping['sale_status'] === 'approved_not_yet_enabled', "{$mapping['public_code']} sale status not enabled");
    expect_cutover(in_array((int) $mapping['edd_download_id'], [458, 459, 460], true), "{$mapping['public_code']} uses a dedicated Download");
}
expect_cutover($codesSeen === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1'], 'mappings are canonically ordered');

// Direct install-site / Gravity / Stripe / local-eval issuance is disabled.
expect_cutover($receipt['counts']['issuance_surfaces_disabled'] === 6, 'six direct issuance surfaces disabled');
expect_cutover($receipt['counts']['retained_recovery_surfaces'] === count($fixtureRetained), 'all retained recovery surfaces preserved');
$deniedSeen = [];
foreach ($receipt['issuer_disablements'] as $surface) {
    expect_cutover($surface['grants_entitlement'] === false, "{$surface['surface']} never grants entitlement");
    if ($surface['denial_code'] !== null) {
        $deniedSeen[] = $surface['surface'];
        expect_cutover(in_array($surface['denial_code'], ['INSTALL_SITE_ISSUANCE_DISABLED', 'STRIPE_DIRECT_FLOW_DENIED', 'LOCAL_EVALUATION_DENIED'], true), "{$surface['surface']} carries an exact denial code");
        expect_cutover($surface['next_action'] !== null && $surface['retained_for'] === null, "{$surface['surface']} denied with next action");
    } else {
        expect_cutover(in_array($surface['retained_for'], ['validation', 'recovery'], true), "{$surface['surface']} retained for validation or recovery");
    }
}
expect_cutover(count($deniedSeen) === 6, 'all six denied surfaces audited');

// Legacy disposition registry: quarantine/retire never grant; migration preserved.
$quarantined = 0;
$retired = 0;
$preserved = 0;
$neverGrant = 0;
foreach ($receipt['legacy_disposition'] as $record) {
    if ($record['record_state'] === 'quarantined') {
        $quarantined++;
    } elseif ($record['record_state'] === 'retired') {
        $retired++;
    } else {
        $preserved++;
    }
    if ($record['never_grant']) {
        $neverGrant++;
    }
    if ((int) $record['download_id'] === 453) {
        expect_cutover($record['disposition'] === 'quarantine' && $record['reason'] === 'implicit_focusa_mapping_forbidden' && $record['never_grant'] === true, 'Download 453 quarantined and never grants');
    }
    if (strpos($record['reason'], 'credit_pack') !== false) {
        expect_cutover($record['record_state'] === 'retired' && $record['never_grant'] === true, 'credit packs retired and never grant');
    }
    if (strpos($record['reason'], 'preserve_refund') !== false || strpos($record['reason'], 'preserve_revoc') !== false) {
        expect_cutover($record['record_state'] === 'retired' && $record['never_grant'] === true, 'refunded/revoked records stay retired terminal');
    }
    if ($record['disposition'] === 'migrate') {
        expect_cutover($record['record_state'] === 'preserved' && $record['never_grant'] === false, 'migration-class records preserved, never granted here');
    }
}
expect_cutover($quarantined >= 8 && $retired >= 5 && $preserved === 1, 'legacy catalog is quarantined/retired with one preserved migration class');
expect_cutover($neverGrant >= count($legacy) - 1, 'all legacy records except the migration class never grant');
expect_cutover($receipt['counts']['legacy_quarantined'] === $quarantined && $receipt['counts']['legacy_retired'] === $retired && $receipt['counts']['legacy_preserved'] === $preserved, 'receipt counts match the disposition registry');
expect_cutover($receipt['redacted'] === true && $receipt['validation'] === 'passed_fail_closed', 'receipt is redacted and passed fail-closed');
assert_no_sensitive($receipt, 'cutover receipt');

// Idempotent replay: byte-identical stored receipt, zero new rows.
$rowsBeforeReplay = [];
foreach (['wpuiai_spec172_cutover_runs', 'wpuiai_spec172_cutover_mappings', 'wpuiai_spec172_issuer_disabled', 'wpuiai_spec172_legacy_disposition', 'wpuiai_spec172_quarantine_ledger', 'wpuiai_spec172_cutover_journal'] as $table) {
    $rowsBeforeReplay[$table] = $canary->countRows($table);
}
$replay = $canary->canaryCutover($baseInput);
expect_cutover($replay['replayed'] === true, 'replay is detected');
expect_cutover(json_encode($replay['receipt'], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) === json_encode($receipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES), 'replay returns the byte-identical stored receipt');
foreach (['wpuiai_spec172_cutover_runs', 'wpuiai_spec172_cutover_mappings', 'wpuiai_spec172_issuer_disabled', 'wpuiai_spec172_legacy_disposition', 'wpuiai_spec172_quarantine_ledger', 'wpuiai_spec172_cutover_journal'] as $table) {
    expect_cutover($canary->countRows($table) === $rowsBeforeReplay[$table], "replay writes zero rows to {$table}");
}

// A different payload on the same run handle fails closed.
$tampered = $baseInput;
$tampered['legacy_records'] = array_slice($legacy, 0, count($legacy) - 1);
expect_cutover_throws(static fn() => $canary->canaryCutover($tampered), 'RUN_ALREADY_STARTED', 'changed payload on an existing run_handle fails closed');
expect_cutover($canary->journalChainValid(), 'journal digest chain is valid after the real-state run');

// ── 5. Missing/malformed proof fails closed with zero writes ──────────

$noProof = $baseInput;
unset($noProof['zero_sales_proof']);
$rowsBeforeFail = $canary->countRows('wpuiai_spec172_cutover_runs');
expect_cutover_throws(static fn() => $canary->canaryCutover($noProof), 'ZERO_SALES_PROOF_REQUIRED', 'missing zero-sales proof fails closed');
$noProofAccepted = $baseInput;
$noProofAccepted['zero_sales_proof'] = ['accepted' => false, 'inventory_id' => 'focusa-vbcqu.20.15.6', 'zero_sales_proven' => false, 'clean_cutover_allowed' => false, 'decision_status' => 'migration_preserving_path_selected'];
expect_cutover_throws(static fn() => $canary->canaryCutover($noProofAccepted), 'ZERO_SALES_PROOF_REQUIRED', 'unaccepted proof fails closed');
$contradictoryProof = $baseInput;
$contradictoryProof['zero_sales_proof'] = ['accepted' => true, 'inventory_id' => 'focusa-vbcqu.20.15.6', 'zero_sales_proven' => false, 'clean_cutover_allowed' => true, 'decision_status' => 'contradictory'];
expect_cutover_throws(static fn() => $canary->canaryCutover($contradictoryProof), 'ZERO_SALES_PROOF_REQUIRED', 'clean cutover allowed without proven zero sales fails closed');
expect_cutover($canary->countRows('wpuiai_spec172_cutover_runs') === $rowsBeforeFail, 'failed proofs write zero run rows');

// Caller-supplied commercial fields are rejected.
$callerGrant = $baseInput;
$callerGrant['dedicated_mappings'] = $mappings;
$callerGrant['dedicated_mappings'][0]['operator_seats'] = 9;
expect_cutover_throws(static fn() => $canary->canaryCutover($callerGrant), 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'caller-controlled commercial field fails closed');

// ── 6. Dry run: identical decision, zero writes ───────────────────────

$rowsBeforeDry = [];
foreach (['wpuiai_spec172_cutover_runs', 'wpuiai_spec172_cutover_mappings', 'wpuiai_spec172_issuer_disabled', 'wpuiai_spec172_legacy_disposition', 'wpuiai_spec172_quarantine_ledger', 'wpuiai_spec172_cutover_journal'] as $table) {
    $rowsBeforeDry[$table] = $canary->countRows($table);
}
$dry = $canary->dryRunCutover($baseInput);
expect_cutover($dry['dry_run'] === true && $dry['writes'] === 0, 'dry run is write-free');
expect_cutover($dry['decision'] === 'migration_preserving_path_selected', 'dry run matches the real-state decision');
expect_cutover($dry['mapping_status'] === FocusaSpec172NoSalesCutoverCanary::MAPPING_BLOCKED, 'dry run keeps mappings blocked');
foreach (['wpuiai_spec172_cutover_runs', 'wpuiai_spec172_cutover_mappings', 'wpuiai_spec172_issuer_disabled', 'wpuiai_spec172_legacy_disposition', 'wpuiai_spec172_quarantine_ledger', 'wpuiai_spec172_cutover_journal'] as $table) {
    expect_cutover($canary->countRows($table) === $rowsBeforeDry[$table], "dry run writes zero rows to {$table}");
}

// ── 7. A genuine sale stops the cutover and requires customer-rights mapping ──

$genuineInput = $baseInput;
$genuineInput['run_handle'] = 'run_spec172_cutover_genuine_sale';
$genuineInput['genuine_sale_observed'] = true;
$genuineResult = $canary->canaryCutover($genuineInput);
expect_cutover($genuineResult['decision'] === 'stopped_requiring_customer_rights_mapping', 'a genuine sale stops the cutover');
expect_cutover($genuineResult['receipt']['clean_cutover_blocked_reason'] === FocusaSpec172NoSalesCutoverCanary::GENUINE_SALE_CODE, 'genuine sale requires an explicit customer-rights mapping');
expect_cutover($genuineResult['receipt']['counts']['mappings_enabled'] === 0, 'no mapping is enabled when a genuine sale appears');
expect_cutover($genuineResult['receipt']['counts']['issuance_surfaces_disabled'] === 6, 'issuance stays disabled when a genuine sale appears');
expect_cutover($genuineResult['receipt']['genuine_sale_observed'] === true, 'genuine sale flag preserved in the receipt');
expect_cutover($canary->countRows('wpuiai_spec172_quarantine_ledger') > 0, 'legacy preservation ledger retained alongside the stop');

// ── 8. Accepted zero-sales proof: clean cutover enables mappings only after validation ──

$proofAccepted = [
    'accepted' => true,
    'inventory_id' => 'focusa-vbcqu.20.15.6',
    'zero_sales_proven' => true,
    'clean_cutover_allowed' => true,
    'decision_status' => 'zero_sales_proven_clean_cutover_accepted',
];
$cleanInput = $baseInput;
$cleanInput['run_handle'] = 'run_spec172_cutover_clean_canary';
$cleanInput['zero_sales_proof'] = $proofAccepted;
$cleanResult = $canary->canaryCutover($cleanInput);
expect_cutover($cleanResult['decision'] === 'clean_cutover_executed', 'accepted proof executes the clean cutover canary');
expect_cutover($cleanResult['receipt']['clean_cutover_blocked_reason'] === null, 'clean cutover no longer blocked');
expect_cutover($cleanResult['receipt']['counts']['mappings_enabled'] === 3 && $cleanResult['receipt']['counts']['mappings_blocked'] === 0, 'dedicated mappings enabled only after validation');
foreach ($cleanResult['receipt']['mappings'] as $mapping) {
    expect_cutover($mapping['mapping_status'] === FocusaSpec172NoSalesCutoverCanary::MAPPING_ENABLED, "{$mapping['public_code']} mapping enabled after validation");
    expect_cutover($mapping['checkout_enabled'] === false && $mapping['sale_status'] === 'approved_not_yet_enabled', "{$mapping['public_code']} checkout still disabled");
}
expect_cutover($cleanResult['receipt']['authority']['split_issuance'] === false, 'clean cutover keeps one canonical paid authority');
assert_no_sensitive($cleanResult['receipt'], 'clean cutover receipt');

// ── 9. Rollback-safe reconciliation receipts ──────────────────────────

$reconReal = $canary->reconcile([
    'recon_handle' => 'recon_spec172_real_state',
    'run_handle' => 'run_spec172_cutover_real_state',
    'migration_provenance' => $provenance,
]);
expect_cutover($reconReal['replayed'] === false, 'first reconciliation is recorded');
expect_cutover($reconReal['result'] === 'passed_fail_closed', 'real-state reconciliation passes fail-closed');
$reconReceipt = $reconReal['receipt'];
expect_cutover($reconReceipt['schema'] === FocusaSpec172NoSalesCutoverCanary::RECON_SCHEMA, 'reconciliation receipt schema');
foreach (['receipt_intact', 'one_canonical_paid_authority', 'checkout_still_disabled', 'legacy_zero_grant', 'refund_revoke_truth_preserved', 'journal_chain_valid'] as $finding) {
    expect_cutover($reconReceipt['findings'][$finding] === true, "reconciliation finding {$finding} holds");
}
expect_cutover($reconReceipt['counts']['non_dedicated_mappings'] === 0 && $reconReceipt['counts']['live_checkout_mappings'] === 0, 'zero non-dedicated and zero live-checkout mappings');
expect_cutover($reconReceipt['counts']['granting_surfaces'] === 0 && $reconReceipt['counts']['legacy_granting'] === 0 && $reconReceipt['counts']['adverse_active'] === 0, 'zero split issuance, zero legacy grants, zero adverse active');
assert_no_sensitive($reconReceipt, 'reconciliation receipt');

$reconRowsBefore = $canary->countRows('wpuiai_spec172_reconciliation_runs');
$reconReplay = $canary->reconcile([
    'recon_handle' => 'recon_spec172_real_state',
    'run_handle' => 'run_spec172_cutover_real_state',
    'migration_provenance' => $provenance,
]);
expect_cutover($reconReplay['replayed'] === true, 'reconciliation replay is detected');
expect_cutover(json_encode($reconReplay['receipt'], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) === json_encode($reconReceipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES), 'reconciliation replay returns the byte-identical receipt');
expect_cutover($canary->countRows('wpuiai_spec172_reconciliation_runs') === $reconRowsBefore, 'reconciliation replay writes zero rows');

// The clean-cutover canary run also reconciles: one canonical authority with
// checkout still disabled and zero legacy grants.
$reconClean = $canary->reconcile([
    'recon_handle' => 'recon_spec172_clean_canary',
    'run_handle' => 'run_spec172_cutover_clean_canary',
    'migration_provenance' => $provenance,
]);
expect_cutover($reconClean['result'] === 'passed_fail_closed', 'clean-cutover canary reconciliation passes (checkout still disabled)');
expect_cutover($reconClean['receipt']['findings']['one_canonical_paid_authority'] === true, 'clean canary keeps one canonical paid authority');

// Drift (a live-checkout mapping row) fails the reconciliation fail-closed.
$driftRun = 'run_spec172_cutover_drift';
$driftInput = $baseInput;
$driftInput['run_handle'] = $driftRun;
$canary->canaryCutover($driftInput);
$db->exec("UPDATE wp_wpuiai_spec172_cutover_mappings SET checkout_enabled = 'true' WHERE run_handle = '{$driftRun}' AND public_code = 'focusa_operator_lifetime_v1'");
$reconRowsBefore = $canary->countRows('wpuiai_spec172_reconciliation_runs');
expect_cutover_throws(
    static fn() => $canary->reconcile(['recon_handle' => 'recon_spec172_drift', 'run_handle' => $driftRun, 'migration_provenance' => $provenance]),
    'RECONCILIATION_MISMATCH',
    'a live-checkout mapping row fails reconciliation closed',
);
expect_cutover($canary->countRows('wpuiai_spec172_reconciliation_runs') === $reconRowsBefore, 'failed reconciliation writes zero rows');

// ── 10. Rollback-safe proof ───────────────────────────────────────────

$rollback = $canary->proveRollback([
    'proof_handle' => 'proof_spec172_real_state',
    'run_handle' => 'run_spec172_cutover_real_state',
    'migration_provenance' => $provenance,
]);
expect_cutover($rollback['replayed'] === false, 'first rollback proof is recorded');
expect_cutover($rollback['verdict'] === 'preservation_only_no_split_issuance_no_stale_refund', 'rollback verdict is preservation-only');
$rollbackReceipt = $rollback['receipt'];
expect_cutover($rollbackReceipt['schema'] === FocusaSpec172NoSalesCutoverCanary::ROLLBACK_SCHEMA, 'rollback proof schema');
expect_cutover($rollbackReceipt['rehearsal']['receipt_intact'] === true && $rollbackReceipt['rehearsal']['journal_chain_valid'] === true, 'receipt and journal survive the rollback rehearsal');
expect_cutover($rollbackReceipt['rehearsal']['split_issuance_restorable'] === false, 'rollback cannot restore split issuance');
expect_cutover($rollbackReceipt['rehearsal']['stale_refund_truth_restorable'] === false, 'rollback cannot restore stale refund truth');
expect_cutover($rollbackReceipt['rehearsal']['destructive_statements'] === 0, 'rollback rehearsal proves zero destructive statements');
expect_cutover($rollbackReceipt['rehearsal']['enable_path'] === 'ISSUANCE_SURFACE_ENABLE_DENIED', 'no issuance enable path exists');
expect_cutover(in_array('customers', $rollbackReceipt['rehearsal']['preserved'], true) && in_array('refunds', $rollbackReceipt['rehearsal']['preserved'], true), 'customer/refund truth is preserved on rollback');
expect_cutover($rollbackReceipt['counts']['grant_capable_rows'] === 0, 'zero grant-capable rows in the ledger');
assert_no_sensitive($rollbackReceipt, 'rollback proof');

$proofRowsBefore = $canary->countRows('wpuiai_spec172_rollback_proof');
$rollbackReplay = $canary->proveRollback([
    'proof_handle' => 'proof_spec172_real_state',
    'run_handle' => 'run_spec172_cutover_real_state',
    'migration_provenance' => $provenance,
]);
expect_cutover($rollbackReplay['replayed'] === true, 'rollback proof replay is detected');
expect_cutover(json_encode($rollbackReplay['receipt'], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) === json_encode($rollbackReceipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES), 'rollback proof replay returns the byte-identical receipt');
expect_cutover($canary->countRows('wpuiai_spec172_rollback_proof') === $proofRowsBefore, 'rollback proof replay writes zero rows');

// No enable path: any attempt to re-enable a disabled issuance surface fails closed.
expect_cutover_throws(static fn() => $canary->enableIssuanceSurface(['surface' => 'install_site_create']), 'ISSUANCE_SURFACE_ENABLE_DENIED', 'no issuance surface can ever be re-enabled');
$schema->assertPreservationOnly();
expect_cutover(true, 'schema is preservation-only (no DELETE/TRUNCATE/DROP/UPDATE)');
expect_cutover($canary->journalChainValid(), 'journal digest chain is valid after all runs');

// ── Summary ───────────────────────────────────────────────────────────

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec172.no_sales_cutover_test.v1',
    'positive_checks' => $positiveChecks,
    'negative_checks' => $negativeChecks,
    'decision_current_state' => 'migration_preserving_path_selected',
    'clean_cutover_blocked_reason' => 'ZERO_SALES_PROOF_REQUIRED',
    'dedicated_mappings' => 3,
    'mappings_enabled' => 0,
    'checkout_enabled' => 0,
    'issuance_surfaces_disabled' => 6,
    'retained_recovery_surfaces' => count($fixtureRetained),
    'download_453' => 'quarantined_never_grants',
    'legacy_quarantined' => $quarantined,
    'legacy_retired' => $retired,
    'legacy_preserved' => $preserved,
    'legacy_never_grant' => $neverGrant,
    'genuine_sale_stops_cutover' => true,
    'clean_cutover_canary_mappings_enabled_after_validation' => 3,
    'clean_cutover_canary_checkout_disabled' => true,
    'reconciliation_passed_fail_closed' => true,
    'rollback_preservation_only' => true,
    'idempotent_replays' => true,
    'journal_chain_valid' => true,
    'redacted' => true,
    'result' => 'passed_fail_closed',
], JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
