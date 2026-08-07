<?php
declare(strict_types=1);

$root = dirname(__DIR__);
$registry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

function resolve_facade(array $registry, array $request): array {
    foreach ($registry['request_contract']['forbidden'] as $field) {
        if (array_key_exists($field, $request)) {
            return ['ok' => false, 'error' => 'FACADE_REQUEST_FIELD_DENIED'];
        }
    }
    foreach ($registry['request_contract']['required'] as $field) {
        if (!array_key_exists($field, $request) || !is_string($request[$field]) || $request[$field] === '') {
            return ['ok' => false, 'error' => 'FACADE_REQUEST_INVALID'];
        }
    }

    $matches = array_values(array_filter(
        $registry['facades'],
        static fn(array $facade): bool => hash_equals($facade['facade_id'], $request['facade_id'])
    ));
    if (count($matches) !== 1) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_facade']];
    }
    $facade = $matches[0];
    if (!in_array($request['origin'], $facade['exact_origins'], true)) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_origin']];
    }
    if (!in_array($request['product_code'], $facade['products'], true)) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_product']];
    }
    if (!array_key_exists($request['route'], $registry['proxy_routes'])) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_route']];
    }
    if (!array_key_exists($request['callback_handle'], $facade['callbacks'])) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_callback']];
    }
    if (!in_array($request['locale'], $facade['locale']['allowed'], true)) {
        return ['ok' => false, 'error' => $registry['request_contract']['unknown_locale']];
    }

    return [
        'ok' => true,
        'facade_id' => $facade['facade_id'],
        'origin' => $request['origin'],
        'product_code' => $request['product_code'],
        'route' => $registry['proxy_routes'][$request['route']],
        'callback' => $request['origin'] . $facade['callbacks'][$request['callback_handle']],
        'sender_identity' => $facade['sender']['identity'],
        'locale' => $request['locale'],
    ];
}

function expect(bool $condition, string $message): void {
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function valid_request(array $overrides = []): array {
    return array_replace([
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'product_code' => 'focusa_operator_lifetime_v1',
        'route' => 'activation_start',
        'callback_handle' => 'success',
        'locale' => 'en-US',
        'timestamp' => '2026-08-07T00:00:00Z',
        'request_id' => 'req_synthetic_01',
        'idempotency_key' => 'idem_synthetic_01',
    ], $overrides);
}

expect($registry['schema'] === 'focusa.spec152e.facade_registry.v1', 'schema');
expect($registry['owner'] === 'WPUIAI/wpuiai', 'owner');
expect($registry['authority']['canonical'] === 'WPUIAI.com EDD', 'canonical authority');
expect($registry['authority']['facade_role'] === 'presenter_and_bounded_proxy_only', 'bounded facade role');
expect($registry['authority']['wildcard_authority'] === 'forbidden', 'wildcard authority forbidden');
expect($registry['authority']['entitlement_issuance'] === 'forbidden', 'facade issuance forbidden');
expect($registry['authority']['customer_or_commerce_truth'] === 'forbidden', 'facade commerce truth forbidden');
expect($registry['authority']['spec158'] === 'excluded', 'Spec 158 excluded');

$resolved = resolve_facade($registry, valid_request());
expect($resolved['ok'] === true, 'registered exact request accepted');
expect($resolved['route'] === '/v1/activation/start', 'route resolved server-side');
expect($resolved['callback'] === 'https://install.focusa.dev/activate/callback/success', 'callback resolved from exact origin and handle');
expect($resolved['sender_identity'] === 'focusa_install_transactional_v1', 'sender resolved server-side');

$negativeChecks = [
    'unknown facade' => [valid_request(['facade_id' => 'attacker_v1']), 'FACADE_ORIGIN_DENIED'],
    'unregistered origin' => [valid_request(['origin' => 'https://evil.invalid']), 'FACADE_ORIGIN_DENIED'],
    'subdomain widening' => [valid_request(['origin' => 'https://child.install.focusa.dev']), 'FACADE_ORIGIN_DENIED'],
    'wildcard origin' => [valid_request(['origin' => 'https://*.focusa.dev']), 'FACADE_ORIGIN_DENIED'],
    'cross-product widening' => [valid_request(['facade_id' => 'uiai_engine_v1', 'origin' => 'https://engine.focusa.dev']), 'FACADE_PRODUCT_DENIED'],
    'unknown product' => [valid_request(['product_code' => 'invented_product_v1']), 'FACADE_PRODUCT_DENIED'],
    'unknown route' => [valid_request(['route' => 'authority_issue']), 'FACADE_ROUTE_DENIED'],
    'arbitrary callback' => [valid_request(['callback_handle' => 'https://evil.invalid/callback']), 'FACADE_CALLBACK_DENIED'],
    'unknown locale' => [valid_request(['locale' => 'en-GB']), 'FACADE_LOCALE_DENIED'],
];
foreach ($negativeChecks as $name => [$request, $error]) {
    $decision = resolve_facade($registry, $request);
    expect($decision['ok'] === false && $decision['error'] === $error, "{$name} denied");
}

foreach ($registry['request_contract']['forbidden'] as $field) {
    $decision = resolve_facade($registry, valid_request([$field => 'attacker-controlled']));
    expect($decision['ok'] === false && $decision['error'] === 'FACADE_REQUEST_FIELD_DENIED', "caller field {$field} denied");
}
foreach ($registry['request_contract']['required'] as $field) {
    $request = valid_request();
    unset($request[$field]);
    $decision = resolve_facade($registry, $request);
    expect($decision['ok'] === false && $decision['error'] === 'FACADE_REQUEST_INVALID', "required field {$field}");
}

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.facade_registry_php_validation.v1',
    'facades' => $registry['counts']['facades'],
    'exact_origins' => $registry['counts']['exact_origins'],
    'negative_checks' => count($negativeChecks),
    'forbidden_caller_fields' => count($registry['request_contract']['forbidden']),
    'result' => 'passed',
], JSON_UNESCAPED_SLASHES) . "\n");
