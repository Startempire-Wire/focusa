<?php
// Exact verification for atom focusa-vbcqu.20.13.27: install.focusa.dev
// activation facade routes/pages — authority proxy only.
declare(strict_types=1);

$root = dirname(__DIR__);
require_once $root . '/docs/contracts/spec152e-install-facade-routes.v1.php';
$registry = require $root . '/docs/contracts/spec152e-facade-registry.v1.php';
$fixture = json_decode(
    file_get_contents($root . '/tests/fixtures/spec152e/install-facade-integration-fixtures.v1.json'),
    true,
    512,
    JSON_THROW_ON_ERROR
);

const INSTALL_ORIGIN = 'https://install.focusa.dev';
const INSTALL_PRODUCT = 'focusa_operator_lifetime_v1';
const NOW = 1786060800;

function expect_install(bool $condition, string $message): void
{
    if (!$condition) {
        fwrite(STDERR, "FAIL: {$message}\n");
        exit(1);
    }
}

$positive = 0;
$negative = 0;
$pass = static function (bool $condition, string $message) use (&$positive): void {
    expect_install($condition, $message);
    $positive++;
};
$reject = static function (bool $condition, string $message) use (&$negative): void {
    expect_install($condition, $message);
    $negative++;
};

// --- Contract surface and route table is exact ---------------------------------
$pass($fixture['schema'] === 'focusa.spec152e.install_facade_integration_fixtures.v1', 'fixture schema');
$pass($fixture['synthetic_only'] === true, 'fixture is synthetic only');
$pass($fixture['facade_id'] === FocusaSpec152eInstallFacadeRoutes::FACADE_ID, 'facade id is focusa_install_v1');
$pass($fixture['origin'] === FocusaSpec152eInstallFacadeRoutes::FACADE_ORIGIN, 'facade origin is install.focusa.dev');
$pass($fixture['authority'] === FocusaSpec152eInstallFacadeRoutes::AUTHORITY, 'authority is WPUIAI.com EDD');
$pass($fixture['local_issuance'] === 'unreachable', 'local issuance is unreachable');

$surfaces = FocusaSpec152eInstallFacadeRoutes::surfaces();
$pageRoutes = FocusaSpec152eInstallFacadeRoutes::pageRoutes();
$renderPages = FocusaSpec152eInstallFacadeRoutes::renderPages();
$pass(count($surfaces) === 9, 'exactly nine named install surfaces');
$pass(count($pageRoutes) === 11, 'eleven authority proxy actions under the nine surfaces');
$pass(count($renderPages) === 6, 'six render-only pages');
$pass($fixture['render_pages'] === array_values($renderPages), 'fixture render pages match contract');

$surfaceNames = array_keys($surfaces);
$expectedSurfaces = [
    'activation_start', 'activation_verify', 'activation_offers', 'activation_checkout',
    'activation_existing_license', 'activation_poll', 'lease_refresh', 'nodes',
    'account_manage_link',
];
$pass($surfaceNames === $expectedSurfaces, 'named surfaces are start verify offers checkout existing poll refresh nodes account-links');

// Every action resolves to the authority proxy route registered for the facade.
foreach ($pageRoutes as $action => $route) {
    $pass(
        is_string($registry['proxy_routes'][$action] ?? null) && str_starts_with($registry['proxy_routes'][$action], '/v1/'),
        "{$action} resolves to an authority proxy route"
    );
}
// Fixture surfaces must agree exactly with the contract surface/action mapping.
foreach ($surfaces as $surface => $actions) {
    sort($actions);
    $fixtureActions = [];
    foreach ($fixture['surfaces'] as $entry) {
        if ($entry['name'] === $surface) {
            $fixtureActions = array_column($entry['actions'], 'action');
            sort($fixtureActions);
        }
    }
    $pass($actions === $fixtureActions, "fixture actions match contract surface {$surface}");
}
$registered = array_values(array_filter(
    $registry['facades'],
    static fn(array $facade): bool => $facade['facade_id'] === FocusaSpec152eInstallFacadeRoutes::FACADE_ID
));
$pass(count($registered) === 1, 'focusa_install_v1 registered exactly once');
$pass($fixture['products'] === $registered[0]['products'], 'fixture products match the registered facade allowlist');

// --- Positive: install-site page resolution is an authority proxy only ---------
foreach ($fixture['surfaces'] as $surfaceEntry) {
    foreach ($surfaceEntry['actions'] as $actionEntry) {
        $decision = FocusaSpec152eInstallFacadeRoutes::resolveRoute(
            $surfaceEntry['page'],
            $actionEntry['method'],
            INSTALL_ORIGIN,
            INSTALL_PRODUCT,
            $registry
        );
        $pass($decision['ok'] === true, "{$actionEntry['action']} page resolves");
        $pass($decision['action'] === $actionEntry['action'], "{$actionEntry['action']} action identity");
        $pass($decision['authority_route'] === $actionEntry['authority_route'], "{$actionEntry['action']} authority route");
        $pass($decision['surface'] === $surfaceEntry['name'], "{$actionEntry['action']} surface name");
    }
}

// Registered facade paths are enforced for pages that map to them.
foreach ([
    'activation_verify' => '/activate/verify',
    'activation_checkout' => '/activate/checkout',
    'account_manage_link' => '/account',
] as $action => $page) {
    $decision = FocusaSpec152eInstallFacadeRoutes::resolveRoute($page, $pageRoutes[$action]['method'], INSTALL_ORIGIN, INSTALL_PRODUCT, $registry);
    $pass($decision['ok'] === true, "{$action} maps to registered facade path {$page}");
    $pass($decision['facade_path'] === $page, "{$action} facade path equals registry path");
}

// --- Positive: server-owned proxy request building ------------------------------
$start = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'activation_start',
    ['email' => 'synthetic@invalid.example', 'device_public_key' => 'synth-device-key-01', 'safe_redirect_handle' => 'success'],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_synthetic_install_01',
    'idem_synthetic_install_01',
    NOW
);
$pass($start['ok'] === true, 'activation start proxy request built');
$pass($start['facade_id'] === 'focusa_install_v1', 'facade id comes from the server binding');
$pass($start['origin'] === INSTALL_ORIGIN, 'origin comes from the server binding');
$pass($start['product_code'] === INSTALL_PRODUCT, 'product comes from the server binding');
$pass($start['fields']['email'] === 'synthetic@invalid.example', 'email forwarded as operation input');
$pass($start['authority_route'] === '/v1/activation/start', 'start proxies to the authority route');

$verify = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'activation_verify',
    ['registration_id' => 'reg_synthetic_install_01', 'one_time_verifier' => '483921'],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_synthetic_install_02',
    'idem_synthetic_install_02',
    NOW
);
$pass($verify['ok'] === true && $verify['fields']['one_time_verifier'] === '483921', 'verify proxies verifier input');
$pass($verify['authority_route'] === '/v1/activation/verify', 'verify proxies to the authority route');

$refresh = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'lease_refresh',
    ['node_id' => 'node_synthetic_install_01', 'refresh_credential' => 'opaque-synthetic-refresh', 'current_sequence' => '12'],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_synthetic_install_03',
    'idem_synthetic_install_03',
    NOW
);
$pass($refresh['ok'] === true && $refresh['authority_route'] === '/v1/lease/refresh', 'lease refresh proxies to the authority route');

$nodes = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'nodes_deactivate',
    ['account_session' => 'synthetic-session-01', 'node_id' => 'node_synthetic_install_01'],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_synthetic_install_04',
    'idem_synthetic_install_04',
    NOW
);
$pass($nodes['ok'] === true && $nodes['authority_route'] === '/v1/nodes/deactivate', 'node deactivation proxies to the authority route');

// --- Positive: masked authority responses and bounded outage --------------------
$masked = FocusaSpec152eInstallFacadeRoutes::maskedResponse([
    'request_id' => 'req_synthetic_install_01',
    'registration_id' => 'reg_synthetic_install_01',
    'state' => 'email_verification_pending',
    'terminal' => false,
    'retry' => false,
    'next_action' => 'verify_email',
    'email' => 'synthetic@invalid.example',
    'full_license_key' => 'FOCUSA-SYNTH-NOT-A-KEY',
    'credential' => 'synthetic-not-a-credential',
    'card_pan' => '4242 4242 4242 4242',
    'edd_internal_record' => 'wp_edd_orders:12345',
]);
$pass($masked['masked_email'] === 's***@invalid.example', 'authority email is masked');
$pass(!isset($masked['email'], $masked['full_license_key'], $masked['credential'], $masked['card_pan'], $masked['edd_internal_record']), 'sensitive authority fields never forwarded');
$pass($masked['state'] === 'email_verification_pending' && $masked['next_action'] === 'verify_email', 'public envelope fields pass through');

$outage = FocusaSpec152eInstallFacadeRoutes::authorityUnavailable('req_synthetic_install_99', INSTALL_ORIGIN);
$pass($outage['ok'] === false && $outage['status'] === 503, 'authority outage fails closed with 503');
$pass($outage['envelope']['error'] === 'AUTHORITY_UNAVAILABLE', 'outage error code is AUTHORITY_UNAVAILABLE');
$pass($outage['envelope']['retry'] === true && $outage['envelope']['next_action'] === 'retry_or_use_recovery', 'outage is a bounded safe retry');
$pass($outage['envelope']['state'] === 'recovery_only', 'outage presents recovery posture');
$pass($outage['envelope']['safe_url'] === INSTALL_ORIGIN . '/activate/recovery', 'outage links the registered recovery page');
$pass(!isset($outage['envelope']['one_time_key_envelope'], $outage['envelope']['lease_envelope'], $outage['envelope']['node_id']), 'outage never issues license node or lease');

$renderSuccess = FocusaSpec152eInstallFacadeRoutes::renderPage('/activate/success', $masked, INSTALL_ORIGIN);
$pass($renderSuccess['ok'] === true && $renderSuccess['page'] === 'success', 'success render page resolves');
$renderRecovery = FocusaSpec152eInstallFacadeRoutes::renderPage('/activate/recovery', $masked, INSTALL_ORIGIN);
$pass($renderRecovery['ok'] === true && $renderRecovery['envelope']['state'] === 'recovery_only', 'recovery render page enforces recovery posture');

// --- Negative: fail-closed route, origin, product, and method checks -----------
$unknownPage = FocusaSpec152eInstallFacadeRoutes::resolveRoute('/admin', 'GET', INSTALL_ORIGIN, INSTALL_PRODUCT, $registry);
$reject($unknownPage['ok'] === false && $unknownPage['error'] === 'INSTALL_ROUTE_DENIED', 'unknown install page denied');
$methodMismatch = FocusaSpec152eInstallFacadeRoutes::resolveRoute('/activate', 'GET', INSTALL_ORIGIN, INSTALL_PRODUCT, $registry);
$reject($methodMismatch['ok'] === false && $methodMismatch['error'] === 'INSTALL_ROUTE_DENIED', 'page method mismatch denied');
$wrongOrigin = FocusaSpec152eInstallFacadeRoutes::resolveRoute('/activate', 'POST', 'https://evil.invalid', INSTALL_PRODUCT, $registry);
$reject($wrongOrigin['ok'] === false && $wrongOrigin['error'] === 'FACADE_ORIGIN_DENIED', 'unregistered origin denied');
$wrongProduct = FocusaSpec152eInstallFacadeRoutes::resolveRoute('/activate', 'POST', INSTALL_ORIGIN, 'invented_product_v1', $registry);
$reject($wrongProduct['ok'] === false && $wrongProduct['error'] === 'FACADE_PRODUCT_DENIED', 'unknown product denied');

// Local direct issuance is unreachable: no non-/v1/ route and no local issue action.
foreach ($pageRoutes as $action => $route) {
    $reject(str_starts_with($registry['proxy_routes'][$action], '/v1/'), "{$action} is authority-proxied only");
}
$reject(!isset($pageRoutes['local_issue']), 'local issuance action absent from install routes');
$localIssue = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'local_issue',
    [],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_attacker_01',
    'idem_attacker_01',
    NOW
);
$reject($localIssue['ok'] === false && $localIssue['error'] === 'FACADE_ACTION_DENIED', 'local issuance action rejected');

// Caller-controlled EDD/price/grant/credential fields are rejected.
foreach ([
    'edd_download_id', 'edd_price_id', 'price', 'tier', 'products', 'grants',
    'features', 'limits', 'node_limit', 'commercial_rights', 'entitlement_sequence',
    'lease', 'refund_status', 'email_verified', 'account_id', 'edd_customer_id',
    'order_id', 'license_id', 'sender_email', 'callback_url', 'redirect_url',
    'success_url', 'cancel_url', 'authority', 'credential', 'secret',
] as $field) {
    $decision = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
        'activation_start',
        ['email' => 'synthetic@invalid.example', $field => 'attacker-controlled'],
        $registry,
        INSTALL_ORIGIN,
        INSTALL_PRODUCT,
        'req_attacker_02',
        'idem_attacker_02',
        NOW
    );
    $reject($decision['ok'] === false && $decision['error'] === 'FACADE_REQUEST_FIELD_DENIED', "caller field {$field} denied");
}

// Required operation inputs must be present.
$missingInput = FocusaSpec152eInstallFacadeRoutes::proxyRequest(
    'activation_verify',
    ['one_time_verifier' => '483921'],
    $registry,
    INSTALL_ORIGIN,
    INSTALL_PRODUCT,
    'req_synthetic_install_05',
    'idem_synthetic_install_05',
    NOW
);
$reject($missingInput['ok'] === false && $missingInput['error'] === 'FACADE_REQUEST_INVALID', 'missing required operation input denied');

// Unmasked email is never produced in a renderable response.
$unmasked = FocusaSpec152eInstallFacadeRoutes::maskedResponse([
    'request_id' => 'req_synthetic_install_06',
    'email' => 'someone@example.com',
    'masked_email' => 's***@example.com',
]);
$reject($unmasked['masked_email'] === 's***@example.com' && !isset($unmasked['email']), 'response carries only masked email');

// --- Fixture hygiene: no secrets, no real emails, no license-shaped evidence ----
$fixtureRaw = json_encode($fixture, JSON_UNESCAPED_SLASHES);
$reject(preg_match('/(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+/', $fixtureRaw) !== 1, 'fixture contains no secret-shaped values');
$reject(preg_match('/^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$/m', $fixtureRaw) !== 1, 'fixture contains no real email addresses');
$reject(preg_match('/FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}/', $fixtureRaw) !== 1, 'fixture contains no license-shaped evidence');

$actionCount = 0;
foreach ($fixture['surfaces'] as $entry) {
    $actionCount += count($entry['actions']);
}
$pass($actionCount === 11, 'fixture covers all eleven authority proxy actions');

fwrite(STDOUT, json_encode([
    'schema' => 'focusa.spec152e.install_facade_php_validation.v1',
    'surfaces' => count($surfaces),
    'actions' => count($pageRoutes),
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'result' => 'passed_fail_closed',
], JSON_UNESCAPED_SLASHES) . "\n");
