<?php
// Install.focusa.dev activation facade plugin/routes contract.
//
// The install site is a registered branded facade and bounded authority proxy only.
// Every activation operation (start, verify, offers, checkout, existing license,
// poll, refresh, nodes, account links) is forwarded to the WPUIAI.com EDD authority
// kernel through the authenticated facade protocol; responses are masked and
// proxied back. No local identity, payment, license, or lease decision and no
// durable co-equal authority. When the upstream authority is unavailable the
// facade fails closed with a bounded health/recovery envelope: no new local
// license, node, or lease is ever issued.
declare(strict_types=1);

final class FocusaSpec152eInstallFacadeRoutes
{
    public const SCHEMA = 'focusa.spec152e.install_facade_routes.v1';
    public const CONTRACT_VERSION = 1;
    public const FACADE_ID = 'focusa_install_v1';
    public const FACADE_ORIGIN = 'https://install.focusa.dev';
    public const AUTHORITY = 'WPUIAI.com EDD';

    /** The nine named install-site activation surfaces and their authority actions. */
    private const SURFACES = [
        'activation_start' => ['activation_start'],
        'activation_verify' => ['activation_verify'],
        'activation_offers' => ['activation_offers', 'activation_select_offer'],
        'activation_checkout' => ['activation_checkout'],
        'activation_existing_license' => ['activation_existing_license'],
        'activation_poll' => ['activation_poll'],
        'lease_refresh' => ['lease_refresh'],
        'nodes' => ['nodes_list', 'nodes_deactivate'],
        'account_manage_link' => ['account_manage_link'],
    ];

    /**
     * Install-site page route table. Every entry is an authority proxy route:
     * page + method -> registered facade path (when the facade defines one) and
     * authority proxy route. There is no local issuance route.
     *
     * @var array<string, array{method: string, page: string, facade_path: ?string, surface: string}>
     */
    private const PAGE_ROUTES = [
        'activation_start' => ['method' => 'POST', 'page' => '/activate', 'facade_path' => null, 'surface' => 'activation_start'],
        'activation_verify' => ['method' => 'POST', 'page' => '/activate/verify', 'facade_path' => 'verification', 'surface' => 'activation_verify'],
        'activation_offers' => ['method' => 'GET', 'page' => '/activate/offers', 'facade_path' => null, 'surface' => 'activation_offers'],
        'activation_select_offer' => ['method' => 'POST', 'page' => '/activate/offers', 'facade_path' => null, 'surface' => 'activation_offers'],
        'activation_checkout' => ['method' => 'POST', 'page' => '/activate/checkout', 'facade_path' => 'checkout', 'surface' => 'activation_checkout'],
        'activation_existing_license' => ['method' => 'POST', 'page' => '/activate/existing', 'facade_path' => null, 'surface' => 'activation_existing_license'],
        'activation_poll' => ['method' => 'POST', 'page' => '/activate/poll', 'facade_path' => null, 'surface' => 'activation_poll'],
        'lease_refresh' => ['method' => 'POST', 'page' => '/activate/refresh', 'facade_path' => null, 'surface' => 'lease_refresh'],
        'nodes_list' => ['method' => 'GET', 'page' => '/account/nodes', 'facade_path' => null, 'surface' => 'nodes'],
        'nodes_deactivate' => ['method' => 'POST', 'page' => '/account/nodes', 'facade_path' => null, 'surface' => 'nodes'],
        'account_manage_link' => ['method' => 'GET', 'page' => '/account', 'facade_path' => 'manage', 'surface' => 'account_manage_link'],
    ];

    /** Render-only install-site pages: they display authority state and never mutate. */
    private const RENDER_PAGES = [
        'success' => '/activate/success',
        'cancel' => '/activate/cancel',
        'recovery' => '/activate/recovery',
        'success_callback' => '/activate/callback/success',
        'cancel_callback' => '/activate/callback/cancel',
        'recovery_callback' => '/activate/callback/recovery',
    ];

    /** Exact operation input allowlists from the canonical activation call stack. */
    private const OPERATION_INPUTS = [
        'activation_start' => ['email', 'device_public_key', 'safe_redirect_handle'],
        'activation_verify' => ['registration_id', 'one_time_verifier'],
        'activation_offers' => ['registration_id'],
        'activation_select_offer' => ['registration_id', 'journey'],
        'activation_checkout' => ['registration_id', 'safe_redirect_handle'],
        'activation_existing_license' => ['registration_id', 'human_license_key', 'device_public_key'],
        'activation_poll' => ['registration_id', 'opaque_poll_credential', 'device_public_key'],
        'lease_refresh' => ['node_id', 'refresh_credential', 'current_sequence'],
        'nodes_list' => ['account_session'],
        'nodes_deactivate' => ['account_session', 'node_id'],
        'account_manage_link' => ['account_session', 'safe_redirect_handle'],
    ];

    /**
     * Fields the install facade must never accept from a caller: identity,
     * commerce, product, price, grant, limit, credential, or redirect decisions
     * belong to the authority kernel or the server-owned facade registry.
     */
    private const FORBIDDEN_CLIENT_FIELDS = [
        'email_verified', 'account_id', 'edd_customer_id', 'edd_download_id',
        'edd_price_id', 'order_id', 'license_id', 'price', 'tier', 'products',
        'grants', 'features', 'limits', 'node_limit', 'commercial_rights',
        'entitlement_sequence', 'lease', 'refund_status', 'sender_email',
        'callback_url', 'redirect_url', 'success_url', 'cancel_url',
        'authority', 'credential', 'secret',
    ];

    /** Public response fields a facade may pass through from an authority result. */
    private const PUBLIC_RESPONSE_FIELDS = [
        'request_id', 'registration_id', 'state', 'terminal', 'retry',
        'next_action', 'masked_email', 'safe_url', 'verification_delivery_status',
        'one_time_key_envelope', 'node_id', 'lease_envelope', 'error',
    ];

    public static function surfaces(): array
    {
        return self::SURFACES;
    }

    public static function pageRoutes(): array
    {
        return self::PAGE_ROUTES;
    }

    public static function renderPages(): array
    {
        return self::RENDER_PAGES;
    }

    /**
     * Resolve an install-site page + method to its authority proxy route.
     * Fails closed on unknown page, wrong method, unregistered origin, unknown
     * product, a registry/authority-route mismatch, or a facade-path mismatch.
     */
    public static function resolveRoute(string $page, string $method, string $origin, string $productCode, array $registry): array
    {
        $matches = [];
        foreach (self::PAGE_ROUTES as $action => $route) {
            if (hash_equals($route['page'], $page) && hash_equals($route['method'], strtoupper($method))) {
                $matches[] = [$action, $route];
            }
        }
        if (count($matches) !== 1) {
            return self::failure('INSTALL_ROUTE_DENIED', 'page_and_method_must_match_a_registered_install_route');
        }
        [$action, $route] = $matches[0];

        $facade = self::registeredFacade($registry, self::FACADE_ID, $origin);
        if ($facade === null) {
            return self::failure('FACADE_ORIGIN_DENIED', 'use_registered_facade');
        }
        if (!in_array($productCode, $facade['products'] ?? [], true)) {
            return self::failure('FACADE_PRODUCT_DENIED', 'select_supported_product');
        }
        $authorityRoute = $registry['proxy_routes'][$action] ?? null;
        if (!is_string($authorityRoute) || $authorityRoute === '') {
            return self::failure('FACADE_ROUTE_DENIED', 'authority_proxy_route_unregistered');
        }

        $facadePath = null;
        if ($route['facade_path'] !== null) {
            $facadePath = $facade['paths'][$route['facade_path']] ?? null;
            if (!is_string($facadePath) || !hash_equals($facadePath, $page)) {
                return self::failure('FACADE_ROUTE_DENIED', 'page_must_match_registered_facade_path');
            }
        }

        return [
            'ok' => true,
            'action' => $action,
            'surface' => $route['surface'],
            'page' => $page,
            'method' => $route['method'],
            'facade_id' => self::FACADE_ID,
            'origin' => $origin,
            'facade_path' => $facadePath,
            'authority_route' => $authorityRoute,
        ];
    }

    /**
     * Build the server-owned authority proxy request for an action. Only the
     * operation's exact input fields are forwarded; facade_id, origin, and
     * product_code come from the registry binding, never from the caller.
     */
    public static function proxyRequest(string $action, array $clientFields, array $registry, string $origin, string $productCode, string $requestId, string $idempotencyKey, int $timestamp): array
    {
        if (!isset(self::PAGE_ROUTES[$action], self::OPERATION_INPUTS[$action])) {
            return self::failure('FACADE_ACTION_DENIED', 'action_is_not_an_install_facade_proxy_operation');
        }
        foreach (array_keys($clientFields) as $field) {
            if (in_array($field, self::FORBIDDEN_CLIENT_FIELDS, true)) {
                return self::failure('FACADE_REQUEST_FIELD_DENIED', 'caller_may_not_set_authority_owned_field');
            }
        }
        $facade = self::registeredFacade($registry, self::FACADE_ID, $origin);
        if ($facade === null) {
            return self::failure('FACADE_ORIGIN_DENIED', 'use_registered_facade');
        }
        if (!in_array($productCode, $facade['products'] ?? [], true)) {
            return self::failure('FACADE_PRODUCT_DENIED', 'select_supported_product');
        }
        $authorityRoute = $registry['proxy_routes'][$action] ?? null;
        if (!is_string($authorityRoute) || $authorityRoute === '') {
            return self::failure('FACADE_ROUTE_DENIED', 'authority_proxy_route_unregistered');
        }
        if ($requestId === '' || strlen($requestId) > 512 || $idempotencyKey === '' || strlen($idempotencyKey) > 512) {
            return self::failure('FACADE_REQUEST_INVALID', 'request_correlation_required');
        }

        $fields = [];
        foreach (self::OPERATION_INPUTS[$action] as $input) {
            if (array_key_exists($input, $clientFields) && is_string($clientFields[$input]) && $clientFields[$input] !== '') {
                $fields[$input] = $clientFields[$input];
            }
        }
        $requiredInputs = self::requiredInputs($action);
        foreach ($requiredInputs as $input) {
            if (!array_key_exists($input, $fields)) {
                return self::failure('FACADE_REQUEST_INVALID', 'required_operation_input_missing');
            }
        }

        return [
            'ok' => true,
            'schema' => self::SCHEMA,
            'facade_id' => self::FACADE_ID,
            'origin' => $origin,
            'product_code' => $productCode,
            'action' => $action,
            'authority_route' => $authorityRoute,
            'request_id' => $requestId,
            'idempotency_key' => $idempotencyKey,
            'timestamp' => $timestamp,
            'fields' => $fields,
        ];
    }

    /**
     * Mask an authority result for install-site rendering. Only the bounded
     * public fields pass through; raw email, license keys, credentials, card
     * data, and internal EDD records are never forwarded.
     */
    public static function maskedResponse(array $authorityResult): array
    {
        $envelope = ['schema' => 'focusa.spec152e.masked_activation_envelope.v1'];
        foreach (self::PUBLIC_RESPONSE_FIELDS as $field) {
            if (array_key_exists($field, $authorityResult)) {
                $envelope[$field] = $authorityResult[$field];
            }
        }
        if (isset($authorityResult['email']) && is_string($authorityResult['email'])) {
            $envelope['masked_email'] = self::maskEmail($authorityResult['email']);
        }
        if (isset($envelope['masked_email']) && (!is_string($envelope['masked_email']) || strpos($envelope['masked_email'], '*') === false)) {
            unset($envelope['masked_email']);
        }
        return $envelope;
    }

    /**
     * Bounded health/recovery when the upstream authority is unavailable.
     * Never issues a local license, node, or lease; existing signed offline
     * policy is the only execution authority during the outage.
     */
    public static function authorityUnavailable(string $requestId, string $origin): array
    {
        return [
            'ok' => false,
            'status' => 503,
            'envelope' => [
                'schema' => 'focusa.spec152e.masked_activation_envelope.v1',
                'request_id' => $requestId,
                'state' => 'recovery_only',
                'terminal' => false,
                'retry' => true,
                'next_action' => 'retry_or_use_recovery',
                'safe_url' => $origin . self::RENDER_PAGES['recovery'],
                'error' => 'AUTHORITY_UNAVAILABLE',
            ],
        ];
    }

    /** Render decision for success/cancel/recovery pages from a masked envelope. */
    public static function renderPage(string $page, array $maskedEnvelope, string $origin): array
    {
        $keys = array_keys(self::RENDER_PAGES);
        if (!in_array($page, array_values(self::RENDER_PAGES), true)) {
            return self::failure('FACADE_ROUTE_DENIED', 'page_is_not_a_registered_render_page');
        }
        $label = array_search($page, self::RENDER_PAGES, true);
        if ($label === false) {
            return self::failure('FACADE_ROUTE_DENIED', 'page_is_not_a_registered_render_page');
        }
        if ($label === 'recovery' || $label === 'recovery_callback') {
            $maskedEnvelope['state'] = 'recovery_only';
        }
        return ['ok' => true, 'page' => $label, 'envelope' => $maskedEnvelope, 'safe_url' => $origin . $page];
    }

    private static function requiredInputs(string $action): array
    {
        $optional = ['device_public_key', 'safe_redirect_handle', 'journey', 'current_sequence', 'node_id'];
        return array_values(array_filter(self::OPERATION_INPUTS[$action], static fn(string $input): bool => !in_array($input, $optional, true)));
    }

    private static function registeredFacade(array $registry, string $facadeId, string $origin): ?array
    {
        foreach (($registry['facades'] ?? []) as $facade) {
            if (!is_array($facade) || !isset($facade['facade_id']) || !is_string($facade['facade_id'])
                || !hash_equals($facade['facade_id'], $facadeId)) {
                continue;
            }
            if (!in_array($origin, $facade['exact_origins'] ?? [], true)) {
                return null;
            }
            return $facade;
        }
        return null;
    }

    private static function maskEmail(string $email): string
    {
        $at = strrpos($email, '@');
        return $at === false || $at < 1 || $at === strlen($email) - 1
            ? '***' : substr($email, 0, 1) . '***@' . substr($email, $at + 1);
    }

    private static function failure(string $code, string $nextAction): array
    {
        return ['ok' => false, 'status' => 400, 'error' => $code, 'envelope' => [
            'schema' => 'focusa.spec152e.masked_error.v1',
            'error' => $code,
            'next_action' => $nextAction,
        ]];
    }
}
