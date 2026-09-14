<?php
declare(strict_types=1);

$root = dirname(__DIR__);
$registry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';

function resolve_offer(array $registry, array $request): array {
    $forbidden = array_flip($registry['authority']['caller_controls_forbidden']);
    foreach (array_keys($request) as $field) {
        if (isset($forbidden[$field])) {
            return ['ok' => false, 'error' => 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN'];
        }
    }
    $code = $request['product_code'] ?? '';
    $matches = array_values(array_filter(
        $registry['protected_offers'],
        static fn(array $offer): bool => hash_equals($offer['public_code'], (string) $code)
    ));
    if (count($matches) !== 1) {
        return ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED'];
    }
    $offer = $matches[0];
    if (!$offer['checkout_enabled'] || $offer['mapping_status'] !== 'active' || $offer['edd_download_id'] === null) {
        return ['ok' => false, 'error' => 'PRODUCT_MAPPING_REQUIRED', 'product_code' => $offer['public_code']];
    }
    return [
        'ok' => true,
        'product_code' => $offer['public_code'],
        'edd_download_id' => $offer['edd_download_id'],
        'edd_price_id' => $offer['edd_price_id'],
        'products' => $offer['products'],
        'features' => $offer['features'],
        'node_limit' => $offer['node_limit'],
    ];
}

function expect(bool $condition, string $message): void {
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

expect($registry['schema'] === 'focusa.spec152e.edd_product_registry.v1', 'schema');
expect($registry['authority']['customer_commerce_human_key_refund_entitlement'] === 'WPUIAI.com EDD', 'authority');
expect(count($registry['protected_offers']) === 3, 'three paid offers');
expect($registry['counts']['checkout_enabled'] === 0, 'no invented checkout mapping');
expect($registry['counts']['assigned_edd_downloads'] === 0, 'no invented EDD download');

$codes = array_column($registry['protected_offers'], 'public_code');
sort($codes);
expect($codes === ['focusa_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1', 'uiai_operator_lifetime_v1'], 'exact Spec 172 paid offer codes');
foreach ($codes as $code) {
    $decision = resolve_offer($registry, ['product_code' => $code]);
    expect($decision['ok'] === false && $decision['error'] === 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', "caller-controlled {$code} denied");
}
expect(resolve_offer($registry, ['product_code' => 'invented'])['error'] === 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', 'caller-controlled unknown code denied');

foreach ($registry['authority']['caller_controls_forbidden'] as $field) {
    $decision = resolve_offer($registry, ['product_code' => 'focusa_operator_lifetime_v1', $field => 'attacker-value']);
    expect($decision['error'] === 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', "caller field {$field} denied");
}

$limited = $registry['verified_no_license'];
expect($limited['kind'] === 'account_runtime_posture', 'verified no-license is a posture');
expect($limited['is_license_type'] === false, 'verified no-license is not a License Type');
expect($limited['checkout_enabled'] === false && $limited['edd_software_license_key'] === false, 'limited posture has no checkout or EDD key');
expect($limited['anonymous_access'] === false, 'anonymous product capability forbidden');

$offers = array_column($registry['protected_offers'], null, 'public_code');
expect($offers['focusa_operator_lifetime_v1']['price_usd'] === '697.00', 'Focusa exact price');
expect($offers['uiai_operator_lifetime_v1']['price_usd'] === '697.00', 'UIAI exact price');
$bundle = $offers['focusa_uiai_operator_bundle_lifetime_v1'];
expect($bundle['price_usd'] === '1254.60', 'Bundle exact price');
expect($bundle['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'Bundle exact grant union');
expect($bundle['grant_composition'] === 'exact_union', 'Bundle composition policy');
expect($bundle['component_refunds_allowed'] === false, 'Bundle whole-order refunds only');
foreach ($registry['protected_offers'] as $offer) {
    expect($offer['mapping_status'] === 'approved_policy_blocked_edd_mapping', 'policy approved but EDD mapping blocked');
    expect($offer['sale_status'] === 'approved_not_yet_enabled', 'sale approved but not enabled');
    expect($offer['refund_policy'] === 'whole_order_30_days', 'exact refund policy');
    expect($offer['upgrade_policy'] === 'explicit_upgrade_or_cross_grade_required_existing_operator_v1_preserved', 'exact upgrade policy');
    expect($offer['operator_seats'] === 1 && $offer['node_limit'] === 3, 'one seat and three nodes');
    expect($offer['future_products_included'] === false && $offer['future_license_types_included'] === false, 'future rights excluded');
}

$download453 = array_values(array_filter(
    $registry['current_edd_catalog']['entries'],
    static fn(array $entry): bool => $entry['download_id'] === 453
));
expect(count($download453) === 1, 'Download 453 exactly once');
expect($download453[0]['entitlement_disposition'] === 'quarantine', 'Download 453 quarantined');
expect($download453[0]['reason'] === 'implicit_focusa_mapping_forbidden', 'Download 453 not authority');
foreach ($registry['protected_offers'] as $offer) {
    expect($offer['edd_download_id'] !== 453, 'protected offers never map Download 453');
}
foreach ($registry['legacy_record_classes'] as $legacy) {
    expect(in_array($legacy['disposition'], ['migrate', 'quarantine', 'retire'], true), 'legacy disposition bounded');
}

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.product_registry_php_validation.v1',
    'protected_offers' => count($codes),
    'blocked_unassigned' => 3,
    'forbidden_caller_fields' => count($registry['authority']['caller_controls_forbidden']),
    'legacy_classes' => count($registry['legacy_record_classes']),
    'result' => 'passed',
], JSON_UNESCAPED_SLASHES) . "\n");
