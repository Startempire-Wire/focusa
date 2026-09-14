<?php
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-facade-security.v1.php';
$registry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';

const SECURITY_NOW = 1786060800;
const SECURITY_SECRET = 'synthetic-facade-security-test-secret-not-for-runtime';

function expect_security(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

$origin = 'https://install.focusa.dev';
$facadeId = 'focusa_install_v1';
$product = 'focusa_operator_lifetime_v1';
$sessionId = 'session_synthetic_security_0001';
$session = FocusaSpec152eFacadeSecurity::issueSession(
    $registry, SECURITY_SECRET, $facadeId, $origin, $sessionId, SECURITY_NOW
);
expect_security(str_contains($session['cookie'], '; Secure; HttpOnly; SameSite=Strict'), 'session cookie is secure, HttpOnly, and same-site strict');
expect_security(str_starts_with($session['cookie'], '__Host-focusa_facade='), 'session cookie is host-only');
expect_security($session['expires_at'] === SECURITY_NOW + 1800, 'session TTL is bounded');

$consumed = [];
$consumeCsrf = static function (string $boundFacade, string $boundSession, string $nonce, int $expiresAt) use (&$consumed): bool {
    $key = implode(':', [$boundFacade, $boundSession, $nonce]);
    if (isset($consumed[$key]) || $expiresAt < SECURITY_NOW) {
        return false;
    }
    $consumed[$key] = true;
    return true;
};
$allowRate = static fn(string $boundFacade, string $opaqueClientKey, string $route): bool =>
    $boundFacade !== '' && preg_match('/^[a-f0-9]{64}$/D', $opaqueClientKey) === 1 && $route !== '';
$csrfSequence = 0;
$request = static function (array $overrides = []) use (
    &$csrfSequence, $registry, $origin, $facadeId, $product, $session, $sessionId
): array {
    $csrfSequence++;
    $route = $overrides['route'] ?? 'activation_start';
    $csrfRoute = $route === 'authority_issue' ? 'activation_start' : $route;
    $csrf = FocusaSpec152eFacadeSecurity::issueCsrf(
        SECURITY_SECRET, $facadeId, $origin, $sessionId, $csrfRoute,
        'csrf_synthetic_' . str_pad((string) $csrfSequence, 4, '0', STR_PAD_LEFT), SECURITY_NOW
    );
    return array_replace([
        'facade_id' => $facadeId,
        'origin' => $origin,
        'route' => $route,
        'method' => 'POST',
        'product_code' => $product,
        'redirect_handle' => 'success',
        'session_token' => $session['token'],
        'csrf_token' => $csrf,
        'client_key' => 'synthetic-browser-client',
    ], $overrides);
};
$verify = static fn(array $candidate, ?callable $rate = null): array => FocusaSpec152eFacadeSecurity::verifyBrowserRequest(
    $candidate, $registry, SECURITY_SECRET, $consumeCsrf, $rate ?? $allowRate, SECURITY_NOW
);

$accepted = $verify($request());
expect_security($accepted['ok'] === true, 'registered exact-origin mutation accepted');
expect_security($accepted['authority_route'] === '/v1/activation/start', 'route resolves only through registry');
expect_security($accepted['safe_redirect'] === $origin . '/activate/callback/success', 'redirect handle resolves server-side');
expect_security($accepted['session_id'] === $sessionId, 'session remains bound to presenter session');
expect_security($accepted['response_headers']['Access-Control-Allow-Origin'] === $origin, 'CORS emits the exact origin');
expect_security($accepted['response_headers']['Access-Control-Allow-Credentials'] === 'true', 'credentialed CORS is explicit');
expect_security($accepted['response_headers']['Vary'] === 'Origin', 'origin response is cache-separated');
expect_security(str_contains($accepted['response_headers']['Content-Security-Policy'], "frame-ancestors 'none'"), 'CSP denies framing');
expect_security($accepted['response_headers']['Cache-Control'] === 'no-store', 'facade responses are not cached');

$negativeChecks = [
    'spoofed origin' => [$request(['origin' => 'https://evil.invalid']), 'FACADE_ORIGIN_DENIED'],
    'origin suffix spoof' => [$request(['origin' => 'https://install.focusa.dev.evil.invalid']), 'FACADE_ORIGIN_DENIED'],
    'wrong method' => [$request(['method' => 'GET']), 'FACADE_METHOD_DENIED'],
    'unknown route' => [$request(['route' => 'authority_issue']), 'FACADE_METHOD_DENIED'],
    'wrong product' => [$request(['product_code' => 'attacker_product_v1']), 'FACADE_PRODUCT_DENIED'],
    'caller redirect URL' => [$request(['redirect_handle' => 'https://evil.invalid/callback']), 'FACADE_REDIRECT_DENIED'],
    'unknown callback handle' => [$request(['redirect_handle' => 'attacker']), 'FACADE_REDIRECT_DENIED'],
    'tampered session' => [$request(['session_token' => $session['token'] . '0']), 'FACADE_SESSION_DENIED'],
    'missing CSRF' => [$request(['csrf_token' => null]), 'FACADE_CSRF_DENIED'],
];
foreach ($negativeChecks as $name => [$candidate, $expected]) {
    $decision = $verify($candidate);
    expect_security($decision['ok'] === false && $decision['error'] === $expected, "{$name} fails closed");
}

$crossFacadeSession = FocusaSpec152eFacadeSecurity::issueSession(
    $registry, SECURITY_SECRET, 'focusa_marketing_v1', 'https://focusa.dev',
    'session_synthetic_cross_facade', SECURITY_NOW
);
$crossFacade = $request(['session_token' => $crossFacadeSession['token']]);
expect_security($verify($crossFacade)['error'] === 'FACADE_SESSION_DENIED', 'cross-facade session is denied');

$replayRequest = $request();
expect_security($verify($replayRequest)['ok'] === true, 'fresh CSRF token accepted once');
expect_security($verify($replayRequest)['error'] === 'FACADE_CSRF_DENIED', 'CSRF replay denied');

$wrongScopeCsrf = FocusaSpec152eFacadeSecurity::issueCsrf(
    SECURITY_SECRET, $facadeId, $origin, $sessionId, 'activation_verify',
    'csrf_synthetic_wrong_scope', SECURITY_NOW
);
expect_security($verify($request(['csrf_token' => $wrongScopeCsrf]))['error'] === 'FACADE_CSRF_DENIED', 'CSRF route scope enforced');

$expiredSession = FocusaSpec152eFacadeSecurity::issueSession(
    $registry, SECURITY_SECRET, $facadeId, $origin, 'session_synthetic_expired', SECURITY_NOW - 1800
);
expect_security($verify($request(['session_token' => $expiredSession['token']]))['error'] === 'FACADE_SESSION_DENIED', 'session is denied at its expiry boundary');
$expiredCsrf = FocusaSpec152eFacadeSecurity::issueCsrf(
    SECURITY_SECRET, $facadeId, $origin, $sessionId, 'activation_start',
    'csrf_synthetic_expired', SECURITY_NOW - 600
);
expect_security($verify($request(['csrf_token' => $expiredCsrf]))['error'] === 'FACADE_CSRF_DENIED', 'CSRF token is denied at its expiry boundary');

$rateLimited = $verify($request(), static fn(string $f, string $c, string $r): bool => false);
expect_security($rateLimited['status'] === 429 && $rateLimited['error'] === 'ACTIVATION_REQUEST_ACCEPTED', 'rate limit is enumeration-safe');

$maskedKnown = FocusaSpec152eFacadeSecurity::maskedResponse([
    'ok' => false, 'error' => 'EMAIL_EXISTS', 'email' => 'known@invalid.example', 'customer_id' => 'customer-secret',
]);
$maskedUnknown = FocusaSpec152eFacadeSecurity::maskedResponse([
    'ok' => false, 'error' => 'EMAIL_UNKNOWN', 'email' => 'unknown@invalid.example',
]);
expect_security($maskedKnown === $maskedUnknown, 'known and unknown identity failures are indistinguishable');
$maskedSuccess = FocusaSpec152eFacadeSecurity::maskedResponse([
    'ok' => true, 'request_id' => 'req_synthetic_security', 'state' => 'email_challenge_sent',
    'email' => 'synthetic@invalid.example', 'license_key' => 'SYNTHETIC-MUST-NOT-PASS',
    'credential' => 'synthetic-must-not-pass', 'grants' => ['attacker'],
]);
expect_security($maskedSuccess['masked_email'] === 's***@invalid.example', 'successful response masks email');
expect_security(!isset($maskedSuccess['email'], $maskedSuccess['license_key'], $maskedSuccess['credential'], $maskedSuccess['grants']), 'authority and secret fields are removed');

foreach ([[1801, 'session'], [0, 'session']] as [$ttl, $kind]) {
    try {
        FocusaSpec152eFacadeSecurity::issueSession($registry, SECURITY_SECRET, $facadeId, $origin, 'session_ttl_test', SECURITY_NOW, $ttl);
        expect_security(false, "{$kind} invalid TTL rejected");
    } catch (InvalidArgumentException $expected) {
        expect_security(true, "{$kind} invalid TTL rejected");
    }
}

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.facade_security_adversarial_matrix.v1',
    'positive_checks' => 16,
    'negative_checks' => count($negativeChecks) + 10,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");
