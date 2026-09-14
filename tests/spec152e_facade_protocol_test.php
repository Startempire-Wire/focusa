<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-facade-protocol.v1.php';
$registry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

const SYNTHETIC_CREDENTIAL_ID = 'cred_synthetic_facade_01';
const SYNTHETIC_CREDENTIAL = 'public-synthetic-spec152e-vector-key-v1';
const NOW = 1786060800;

function expect_protocol(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

function unsigned_request(array $overrides = []): array
{
    return array_replace([
        'timestamp' => NOW,
        'nonce' => 'nonce_synthetic_0001',
        'request_id' => 'req_synthetic_protocol_01',
        'idempotency_key' => 'idem_synthetic_protocol_01',
        'registration_id' => 'reg_synthetic_01',
        'facade_id' => 'focusa_install_v1',
        'origin' => 'https://install.focusa.dev',
        'product_code' => 'focusa_operator_lifetime_v1',
        'action' => 'activation_start',
        'redirect_handle' => 'success',
        'body_sha256' => hash('sha256', '{}'),
    ], $overrides);
}

function signed_request(array $overrides = []): array
{
    return FocusaSpec152eFacadeProtocol::signRequest(
        unsigned_request($overrides),
        SYNTHETIC_CREDENTIAL_ID,
        SYNTHETIC_CREDENTIAL
    );
}

$credentialResolver = static function (string $credentialId): ?array {
    if (!hash_equals(SYNTHETIC_CREDENTIAL_ID, $credentialId)) {
        return null;
    }
    return [
        'facade_id' => 'focusa_install_v1',
        'credential' => SYNTHETIC_CREDENTIAL,
        'active' => true,
    ];
};
$seen = [];
$consumeNonce = static function (string $credentialId, string $facadeId, string $nonce, int $timestamp) use (&$seen): bool {
    $key = implode(':', [$credentialId, $facadeId, $nonce]);
    if (isset($seen[$key])) {
        return false;
    }
    $seen[$key] = $timestamp;
    return true;
};
$verify = static fn(array $request): array => FocusaSpec152eFacadeProtocol::verifyRequest(
    $request,
    $registry,
    $credentialResolver,
    $consumeNonce,
    NOW
);

$continuation = FocusaSpec152eFacadeProtocol::issueContinuationToken(
    SYNTHETIC_CREDENTIAL,
    'reg_synthetic_01',
    'focusa_install_v1',
    'activation_start',
    'nonce_synthetic_0001',
    NOW + 300
);
$valid = signed_request(['continuation_token' => $continuation]);
$accepted = $verify($valid);
expect_protocol($accepted['ok'] === true, 'registered authenticated facade request accepted');
expect_protocol($accepted['authority_route'] === '/v1/activation/start', 'action resolved through authority route registry');
expect_protocol($accepted['safe_redirect'] === 'https://install.focusa.dev/activate/callback/success', 'redirect handle resolved server-side');
expect_protocol($verify($valid)['error'] === 'FACADE_REPLAY_DENIED', 'nonce replay denied');

$tampered = signed_request(['nonce' => 'nonce_tamper_0001']);
$tampered['request_id'] = 'req_attacker_changed';
expect_protocol($verify($tampered)['error'] === 'FACADE_AUTH_FAILED', 'signed-field tamper denied');

$negativeChecks = [
    'timestamp skew' => [signed_request(['nonce' => 'nonce_skew_0001', 'timestamp' => NOW - 301]), 'FACADE_TIMESTAMP_DENIED'],
    'wrong origin' => [signed_request(['nonce' => 'nonce_origin_0001', 'origin' => 'https://evil.invalid']), 'FACADE_ORIGIN_DENIED'],
    'wrong product' => [signed_request(['nonce' => 'nonce_product_0001', 'product_code' => 'invented_product_v1']), 'FACADE_PRODUCT_DENIED'],
    'wrong action' => [signed_request(['nonce' => 'nonce_action_0001', 'action' => 'authority_issue']), 'FACADE_ACTION_DENIED'],
    'unsafe redirect' => [signed_request(['nonce' => 'nonce_redirect_0001', 'redirect_handle' => 'https://evil.invalid/callback']), 'FACADE_REDIRECT_DENIED'],
];
foreach ($negativeChecks as $name => [$request, $expectedError]) {
    $decision = $verify($request);
    expect_protocol($decision['ok'] === false && $decision['error'] === $expectedError, "{$name} denied");
}

$wrongBindingToken = FocusaSpec152eFacadeProtocol::issueContinuationToken(
    SYNTHETIC_CREDENTIAL,
    'reg_other_synthetic',
    'focusa_install_v1',
    'activation_start',
    'nonce_continuation_0001',
    NOW + 300
);
$wrongBinding = signed_request([
    'nonce' => 'nonce_continuation_0001',
    'continuation_token' => $wrongBindingToken,
]);
expect_protocol($verify($wrongBinding)['error'] === 'FACADE_CONTINUATION_DENIED', 'continuation registration mismatch denied');

$expiredToken = FocusaSpec152eFacadeProtocol::issueContinuationToken(
    SYNTHETIC_CREDENTIAL,
    'reg_synthetic_01',
    'focusa_install_v1',
    'activation_start',
    'nonce_expired_token_0001',
    NOW - 1
);
$expired = signed_request(['nonce' => 'nonce_expired_token_0001', 'continuation_token' => $expiredToken]);
expect_protocol($verify($expired)['error'] === 'FACADE_CONTINUATION_DENIED', 'expired continuation denied');

$forbiddenRedirect = signed_request(['nonce' => 'nonce_url_field_0001']);
$forbiddenRedirect['redirect_url'] = 'https://evil.invalid/callback';
$forbiddenDecision = $verify($forbiddenRedirect);
expect_protocol($forbiddenDecision['ok'] === false && $forbiddenDecision['error'] === 'FACADE_REQUEST_FIELD_DENIED', 'caller redirect URL field denied');

$masked = FocusaSpec152eFacadeProtocol::maskedEnvelope([
    'request_id' => 'req_synthetic_protocol_01',
    'state' => 'verification_pending',
    'email' => 'synthetic@invalid',
    'license_key' => 'must-not-pass',
    'credential' => 'must-not-pass',
]);
expect_protocol($masked['masked_email'] === 's***@invalid', 'email masked');
expect_protocol(!isset($masked['email'], $masked['license_key'], $masked['credential']), 'sensitive authority fields removed');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.facade_protocol_php_validation.v1',
    'positive_round_trips' => 1,
    'negative_checks' => count($negativeChecks) + 6,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");
