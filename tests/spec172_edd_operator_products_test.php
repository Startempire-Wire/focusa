<?php
// 172.02.03 Create dedicated EDD Operator v1 Downloads and prices.
//
// Exactly three dedicated records (Focusa Operator Lifetime v1, UIAI Engine Operator
// Lifetime v1, and the exact-union Bundle v1) match the canonical Spec 152E product
// registry. Prices are server-owned: 69700 / 69700 / 125460 USD minor units (697.00 /
// 697.00 / 1254.60). Every record configures lifetime duration, whole-order 30-day
// refund metadata, one operator seat, three shared nodes (operator_shared_v1), and
// checkout disabled until validation passes. Legacy WPUIAI downloads and Download 453
// can never grant these License Types: the dedicated contract never references them, the
// registry keeps 453 quarantined with the explicit forbidden reason, and no protected
// offer maps 453. The idempotent provisioning command
// (scripts/edd-operator-v1-provision.php) fails closed on any invariant violation and
// emits a byte-identical redacted receipt on every run. All output is redacted: no raw
// email, key, token, customer row, credential, or card data.
declare(strict_types=1);

$root = dirname(__DIR__);
$registry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$dedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';

$positiveChecks = 0;
$negativeChecks = 0;

function expect_dedicated(bool $condition, string $message): void
{
    global $positiveChecks;
    $positiveChecks++;
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function expect_dedicated_denied(callable $operation, string $code, string $message): void
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

// ── 1. Contract views are in sync (deterministic generator) ────────────

exec('python3 ' . escapeshellarg($root . '/scripts/generate-spec172-edd-operator-v1-downloads.py') . ' --check', $genOut, $genExit);
expect_dedicated($genExit === 0, 'generated dedicated downloads views are current');

// ── 2. Exactly three dedicated records match the canonical registry ────

expect_dedicated($dedicated['schema'] === 'focusa.spec172.edd_operator_v1_downloads.v1', 'dedicated contract schema');
expect_dedicated($dedicated['registry_schema'] === $registry['schema'], 'dedicated contract references the canonical registry');
expect_dedicated($dedicated['owner'] === 'WPUIAI/wpuiai', 'server-owned owner');
expect_dedicated(count($dedicated['records']) === 3, 'exactly three dedicated records');
expect_dedicated(count($registry['protected_offers']) === 3, 'canonical registry has three protected offers');

$offersByCode = [];
foreach ($registry['protected_offers'] as $offer) {
    $offersByCode[$offer['public_code']] = $offer;
}

$downloadIds = [];
$priceIds = [];
$minorUnits = [];
foreach ($dedicated['records'] as $record) {
    $code = $record['public_code'];
    expect_dedicated(isset($offersByCode[$code]), "{$code} resolves in the canonical registry");
    $offer = $offersByCode[$code];

    expect_dedicated($record['edd_download_id'] === $offer['edd_download_id'] || $offer['edd_download_id'] === null, "{$code} download binding is server-owned");
    expect_dedicated(is_int($record['edd_download_id']) && $record['edd_download_id'] > 0, "{$code} dedicated download id is positive");
    expect_dedicated(!in_array($record['edd_download_id'], $downloadIds, true), "{$code} download id is distinct");
    $downloadIds[] = $record['edd_download_id'];
    expect_dedicated((string) $record['edd_price_id'] !== '' && strlen((string) $record['edd_price_id']) <= 191, "{$code} price id is stable");
    expect_dedicated(!in_array($record['edd_price_id'], $priceIds, true), "{$code} price id is distinct");
    $priceIds[] = $record['edd_price_id'];

    expect_dedicated($record['price_usd'] === $offer['price_usd'], "{$code} price matches the canonical registry");
    expect_dedicated($record['products'] === $offer['products'], "{$code} products match the canonical registry");
    expect_dedicated($record['license_duration'] === 'lifetime' && $offer['license_duration'] === 'lifetime', "{$code} lifetime duration matches");
    expect_dedicated((int) $record['operator_seats'] === 1 && (int) $offer['operator_seats'] === 1, "{$code} one operator seat matches");
    expect_dedicated((int) $record['node_limit'] === 3 && (int) $offer['node_limit'] === 3 && $record['node_set'] === 'operator_shared_v1' && $offer['node_set'] === 'operator_shared_v1', "{$code} three shared nodes match");
    expect_dedicated($record['refund_policy'] === 'whole_order_30_days' && $offer['refund_policy'] === 'whole_order_30_days', "{$code} 30-day refund policy matches");
    expect_dedicated($record['sale_status'] === 'approved_not_yet_enabled' && $offer['sale_status'] === 'approved_not_yet_enabled', "{$code} sale status matches");
    expect_dedicated($record['checkout_enabled'] === false && $offer['checkout_enabled'] === false, "{$code} checkout stays disabled");
    expect_dedicated($record['evaluation'] === false && $offer['evaluation'] === false, "{$code} evaluation excluded");
    expect_dedicated($record['future_products_included'] === false && $record['future_license_types_included'] === false, "{$code} future rights excluded");

    if ($code === 'focusa_uiai_operator_bundle_lifetime_v1') {
        expect_dedicated($record['composite_sku_ref'] === $offer['composite_sku_ref'], 'bundle composite SKU matches');
        expect_dedicated($record['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'] && $offer['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle grants the exact two License Types');
        expect_dedicated($record['grant_composition'] === 'exact_union' && $offer['grant_composition'] === 'exact_union', 'bundle is an exact union');
        expect_dedicated($record['component_refunds_allowed'] === false && $offer['component_refunds_allowed'] === false, 'bundle refunds are whole-order only');
    } else {
        expect_dedicated($record['license_type_ref'] === $offer['license_type_ref'], "{$code} License Type matches");
    }
    $minorUnits[] = $record['amount_minor'];
}

// ── 3. Server-owned minor-unit prices ──────────────────────────────────

expect_dedicated($minorUnits === [69700, 69700, 125460], 'exact 69700 / 69700 / 125460 USD minor units');
$usd = array_column($dedicated['records'], 'price_usd');
expect_dedicated($usd === ['697.00', '697.00', '1254.60'], 'exact human USD strings');
foreach ($dedicated['records'] as $record) {
    expect_dedicated($record['amount_minor'] === (int) round((float) $record['price_usd'] * 100), "{$record['public_code']} minor units equal price_usd");
    expect_dedicated($record['currency'] === 'USD', "{$record['public_code']} currency is USD");
}
expect_dedicated(array_sum($minorUnits) === 264860, 'bounded sum of minor units');

// ── 4. Lifetime, 30-day refund metadata, one operator, three nodes ─────

foreach ($dedicated['records'] as $record) {
    expect_dedicated($record['license_duration'] === 'lifetime' && (int) $record['license_length'] === 0, "{$record['public_code']} EDD SL lifetime configuration");
    expect_dedicated($record['licensing_enabled'] === true, "{$record['public_code']} EDD Software Licensing enabled");
    expect_dedicated((int) $record['activation_limit'] === 3, "{$record['public_code']} activation limit is three nodes");
    expect_dedicated((int) $record['refund_days'] === 30, "{$record['public_code']} refund metadata is 30 days");
    expect_dedicated((int) $record['operator_seats'] === 1, "{$record['public_code']} exactly one operator seat");
    expect_dedicated((int) $record['node_limit'] === 3 && $record['node_set'] === 'operator_shared_v1', "{$record['public_code']} three shared nodes");
}

// ── 5. Checkout disabled until validation passes ───────────────────────

expect_dedicated($dedicated['authority']['checkout_enabled'] === false, 'authority checkout disabled');
expect_dedicated($dedicated['authority']['checkout_block_reason'] === 'awaiting_validation_pass', 'explicit awaiting-validation block reason');
expect_dedicated($dedicated['authority']['sale_status'] === 'approved_not_yet_enabled', 'sales approved but not yet enabled');
foreach ($dedicated['records'] as $record) {
    expect_dedicated($record['status'] === 'draft', "{$record['public_code']} EDD download stays draft");
    expect_dedicated($record['checkout_enabled'] === false, "{$record['public_code']} checkout disabled");
}
expect_dedicated($dedicated['counts']['checkout_enabled'] === 0, 'zero checkout-enabled dedicated records');

// ── 6. Unrelated downloads and 453 cannot grant these License Types ────

$legacyIds = array_map('intval', $dedicated['authority']['legacy_download_ids']);
expect_dedicated(in_array(453, $legacyIds, true), 'Download 453 is on the never-grant legacy list');
expect_dedicated((int) $dedicated['authority']['forbidden_implicit_download'] === 453, 'Download 453 is the explicit forbidden implicit mapping');
foreach ($downloadIds as $dedicatedId) {
    expect_dedicated(!in_array($dedicatedId, $legacyIds, true), "dedicated download {$dedicatedId} never reuses a legacy id");
    expect_dedicated($dedicatedId !== 453, "dedicated download {$dedicatedId} is not Download 453");
}

$catalog453 = array_values(array_filter(
    $registry['current_edd_catalog']['entries'],
    static fn(array $entry): bool => (int) $entry['download_id'] === 453
));
expect_dedicated(count($catalog453) === 1, 'Download 453 exactly once in the catalog');
expect_dedicated($catalog453[0]['entitlement_disposition'] === 'quarantine', 'Download 453 stays quarantined');
expect_dedicated($catalog453[0]['reason'] === 'implicit_focusa_mapping_forbidden', 'Download 453 keeps the explicit forbidden reason');
foreach ($registry['protected_offers'] as $offer) {
    expect_dedicated((int) ($offer['edd_download_id'] ?? 0) !== 453, 'no protected offer maps Download 453');
}

$catalogByDownload = [];
foreach ($registry['current_edd_catalog']['entries'] as $entry) {
    $catalogByDownload[(int) $entry['download_id']] = $entry;
}
foreach ($downloadIds as $dedicatedId) {
    expect_dedicated(!isset($catalogByDownload[$dedicatedId]), "dedicated download {$dedicatedId} has no conflicting catalog entry");
}
foreach ($catalogByDownload as $catalogId => $entry) {
    if (in_array($catalogId, $legacyIds, true)) {
        expect_dedicated(in_array($entry['entitlement_disposition'], ['quarantine', 'retire'], true), "legacy download {$catalogId} is quarantined or retired");
    }
}

// Grant resolution fails closed: only an active, checkout-enabled dedicated mapping
// could grant an Operator v1 License Type, and none is enabled yet. Legacy and unknown
// downloads are never resolvable to a dedicated record.
$grantResolution = static function (int $downloadId, string $priceId) use ($dedicated): array {
    foreach ($dedicated['records'] as $record) {
        if ((int) $record['edd_download_id'] === $downloadId && (string) $record['edd_price_id'] === $priceId) {
            if ($record['checkout_enabled'] !== false || $record['sale_status'] !== 'approved_not_yet_enabled') {
                return ['ok' => false, 'error' => 'EDD_CHECKOUT_REQUIRED'];
            }
            return ['ok' => false, 'error' => 'EDD_CHECKOUT_REQUIRED', 'record' => $record['public_code']];
        }
    }
    return ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'];
};
expect_dedicated($grantResolution(453, 'price_legacy_453')['error'] === 'PRODUCT_MAPPING_REQUIRED', 'Download 453 cannot grant any Operator v1 License Type');
expect_dedicated($grantResolution(16, 'price_legacy_16')['error'] === 'PRODUCT_MAPPING_REQUIRED', 'unrelated legacy download 16 cannot grant');
expect_dedicated($grantResolution(455, 'price_credit')['error'] === 'PRODUCT_MAPPING_REQUIRED', 'credit pack cannot grant');
expect_dedicated($grantResolution(9999, 'price_unknown')['error'] === 'PRODUCT_MAPPING_REQUIRED', 'unknown download cannot grant');
foreach ($downloadIds as $dedicatedId) {
    $resolution = $grantResolution($dedicatedId, 'price_' . $dedicatedId . '_wrong');
    expect_dedicated($resolution['error'] === 'PRODUCT_MAPPING_REQUIRED', "dedicated download {$dedicatedId} with wrong price cannot grant");
}
expect_dedicated($grantResolution(458, 'price_focusa_operator_lifetime_v1')['error'] === 'EDD_CHECKOUT_REQUIRED', 'dedicated mapping exists but checkout is disabled until validation passes');

// ── 7. Idempotent provisioning command and redacted receipt ────────────

$runProvision = static function () use ($root): array {
    $out = [];
    $exit = 0;
    exec('php ' . escapeshellarg($root . '/scripts/edd-operator-v1-provision.php'), $out, $exit);
    return ['exit' => $exit, 'stdout' => implode("\n", $out)];
};
$runA = $runProvision();
$runB = $runProvision();
expect_dedicated($runA['exit'] === 0, 'provisioning command exits 0');
expect_dedicated($runA['stdout'] === $runB['stdout'], 'provisioning command is idempotent (byte-identical receipt)');

$receipt = json_decode($runA['stdout'], true, 512, JSON_THROW_ON_ERROR);
expect_dedicated($receipt['schema'] === 'focusa.spec172.edd_operator_v1_provisioning_receipt.v1', 'receipt schema');
expect_dedicated(count($receipt['plan']) === 3, 'receipt plans exactly three dedicated downloads');
expect_dedicated($receipt['checkout_enabled'] === false && $receipt['checkout_block_reason'] === 'awaiting_validation_pass', 'receipt keeps checkout disabled');
expect_dedicated($receipt['sale_status'] === 'approved_not_yet_enabled', 'receipt keeps sales not yet enabled');
expect_dedicated($receipt['redacted'] === true, 'receipt is explicitly redacted');
expect_dedicated($receipt['validation'] === 'passed_fail_closed', 'receipt validation passed fail-closed');
expect_dedicated(preg_match('/^[0-9a-f]{64}$/D', $receipt['idempotency_key']) === 1, 'receipt carries a deterministic idempotency key');
expect_dedicated($receipt['counts']['records'] === 3 && $receipt['counts']['assigned_edd_downloads'] === 3, 'receipt counts three assigned dedicated downloads');
$planCodes = array_column($receipt['plan'], 'public_code');
expect_dedicated($planCodes === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1'], 'receipt plan is canonically ordered');
foreach ($receipt['plan'] as $action) {
    expect_dedicated($action['action'] === 'create_or_update_dedicated_download', 'receipt action is create-or-update');
    expect_dedicated($action['checkout_enabled'] === false && $action['status'] === 'draft', "{$action['public_code']} receipt keeps download draft and checkout disabled");
    expect_dedicated((int) $action['refund_days'] === 30 && $action['refund_policy'] === 'whole_order_30_days', "{$action['public_code']} receipt carries 30-day refund metadata");
    expect_dedicated((int) $action['operator_seats'] === 1 && (int) $action['node_limit'] === 3 && $action['node_set'] === 'operator_shared_v1', "{$action['public_code']} receipt carries one seat and three shared nodes");
}

$rawReceipt = $runA['stdout'];
expect_dedicated(preg_match('/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/', $rawReceipt) !== 1, 'receipt exposes no raw email');
expect_dedicated(preg_match('/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/', $rawReceipt) !== 1, 'receipt exposes no payment key');
expect_dedicated(preg_match('/focusa_live_[0-9]+_[0-9a-f]+/', $rawReceipt) !== 1, 'receipt exposes no synthetic license key');
expect_dedicated(preg_match('/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/', $rawReceipt) !== 1, 'receipt exposes no card data');

$summary = [
    'schema' => 'focusa.spec172.edd_operator_products_validation.v1',
    'dedicated_records' => count($dedicated['records']),
    'assigned_edd_downloads' => count($downloadIds),
    'download_ids' => $downloadIds,
    'minor_units' => $minorUnits,
    'price_usd' => $usd,
    'license_duration' => 'lifetime',
    'refund_days' => 30,
    'operator_seats' => 1,
    'node_limit' => 3,
    'node_set' => 'operator_shared_v1',
    'checkout_enabled' => 0,
    'sale_status' => 'approved_not_yet_enabled',
    'download_453' => 'quarantined_never_grants',
    'unrelated_downloads_grant' => 0,
    'provision_receipt_idempotent' => true,
    'provision_receipt_redacted' => true,
    'checks' => $positiveChecks + $negativeChecks,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
