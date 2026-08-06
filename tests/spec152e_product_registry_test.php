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
expect(count($registry['protected_offers']) === 4, 'four protected offers');
expect($registry['counts']['checkout_enabled'] === 0, 'no invented checkout mapping');
expect($registry['counts']['assigned_edd_downloads'] === 0, 'no invented EDD download');

$codes = array_column($registry['protected_offers'], 'public_code');
sort($codes);
expect($codes === ['focusa_evaluation', 'focusa_operator', 'focusa_uiai_bundle', 'uiai_engine_operator'], 'exact codes');
foreach ($codes as $code) {
    $decision = resolve_offer($registry, ['product_code' => $code]);
    expect($decision['ok'] === false && $decision['error'] === 'PRODUCT_MAPPING_REQUIRED', "{$code} fails closed while unassigned");
}
expect(resolve_offer($registry, ['product_code' => 'invented'])['error'] === 'PRODUCT_MAPPING_REQUIRED', 'unknown code denied');

foreach ($registry['authority']['caller_controls_forbidden'] as $field) {
    $decision = resolve_offer($registry, ['product_code' => 'focusa_operator', $field => 'attacker-value']);
    expect($decision['error'] === 'CLIENT_COMMERCIAL_FIELDS_FORBIDDEN', "caller field {$field} denied");
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
    'blocked_unassigned' => 4,
    'forbidden_caller_fields' => count($registry['authority']['caller_controls_forbidden']),
    'legacy_classes' => count($registry['legacy_record_classes']),
    'result' => 'passed',
], JSON_UNESCAPED_SLASHES) . "\n");
