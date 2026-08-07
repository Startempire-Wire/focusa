<?php
// Public facade security middleware contract. Runtime secrets and counters are injected.
declare(strict_types=1);

final class FocusaSpec152eFacadeSecurity
{
    public const SCHEMA = 'focusa.spec152e.facade_security.v1';
    public const SESSION_TTL_SECONDS = 1800;
    public const CSRF_TTL_SECONDS = 600;

    private const ROUTE_METHODS = [
        'activation_start' => 'POST',
        'activation_verify' => 'POST',
        'activation_offers' => 'GET',
        'activation_select_offer' => 'POST',
        'activation_checkout' => 'POST',
        'activation_existing_license' => 'POST',
        'activation_poll' => 'POST',
        'lease_refresh' => 'POST',
        'nodes_list' => 'GET',
        'nodes_deactivate' => 'POST',
        'account_manage_link' => 'GET',
    ];

    private const PUBLIC_RESPONSE_FIELDS = [
        'request_id', 'registration_id', 'state', 'terminal', 'retry',
        'next_action', 'verification_delivery_status', 'masked_email',
        'one_time_key_envelope', 'node_id', 'lease_envelope',
    ];

    public static function issueSession(
        array $registry,
        string $secret,
        string $facadeId,
        string $origin,
        string $sessionId,
        int $now,
        int $ttl = self::SESSION_TTL_SECONDS
    ): array {
        self::assertFacadeOrigin($registry, $facadeId, $origin);
        $token = self::issueToken($secret, 'session', $facadeId, $origin, $sessionId, '*', $now, $ttl, self::SESSION_TTL_SECONDS);
        return [
            'token' => $token,
            'cookie' => '__Host-focusa_facade=' . $token . '; Path=/; Max-Age=' . $ttl . '; Secure; HttpOnly; SameSite=Strict',
            'expires_at' => $now + $ttl,
        ];
    }

    public static function issueCsrf(
        string $secret,
        string $facadeId,
        string $origin,
        string $sessionId,
        string $route,
        string $nonce,
        int $now,
        int $ttl = self::CSRF_TTL_SECONDS
    ): string {
        if (!isset(self::ROUTE_METHODS[$route]) || self::ROUTE_METHODS[$route] !== 'POST') {
            throw new InvalidArgumentException('CSRF route must be a registered mutation');
        }
        return self::issueToken($secret, 'csrf', $facadeId, $origin, $sessionId, $route, $now, $ttl, self::CSRF_TTL_SECONDS, $nonce);
    }

    /**
     * $consumeCsrf atomically accepts a previously unseen CSRF nonce.
     * $allowRate accepts (facade id, opaque client key, route) without receiving email/customer data.
     */
    public static function verifyBrowserRequest(
        array $request,
        array $registry,
        string $secret,
        callable $consumeCsrf,
        callable $allowRate,
        int $now
    ): array {
        $required = ['facade_id', 'origin', 'route', 'method', 'product_code', 'redirect_handle', 'session_token'];
        foreach ($required as $field) {
            if (!isset($request[$field]) || !is_string($request[$field]) || $request[$field] === '') {
                return self::failure('FACADE_REQUEST_DENIED');
            }
        }
        $facade = self::facade($registry, $request['facade_id']);
        if ($facade === null || !in_array($request['origin'], $facade['exact_origins'] ?? [], true)) {
            return self::failure('FACADE_ORIGIN_DENIED');
        }
        if (!isset(self::ROUTE_METHODS[$request['route']], $registry['proxy_routes'][$request['route']])
            || !hash_equals(self::ROUTE_METHODS[$request['route']], strtoupper($request['method']))) {
            return self::failure('FACADE_METHOD_DENIED');
        }
        if (!in_array($request['product_code'], $facade['products'] ?? [], true)) {
            return self::failure('FACADE_PRODUCT_DENIED');
        }
        if (!isset($facade['callbacks'][$request['redirect_handle']])) {
            return self::failure('FACADE_REDIRECT_DENIED');
        }

        $session = self::verifyToken(
            $request['session_token'], $secret, 'session', $request['facade_id'],
            $request['origin'], '*', $now, self::SESSION_TTL_SECONDS
        );
        if (!$session['ok']) {
            return self::failure('FACADE_SESSION_DENIED');
        }
        $clientKey = isset($request['client_key']) && is_string($request['client_key'])
            ? hash('sha256', $request['client_key']) : hash('sha256', $session['subject']);
        if ($allowRate($request['facade_id'], $clientKey, $request['route']) !== true) {
            return self::failure('ACTIVATION_REQUEST_ACCEPTED', 429);
        }

        if (self::ROUTE_METHODS[$request['route']] === 'POST') {
            if (!isset($request['csrf_token']) || !is_string($request['csrf_token'])) {
                return self::failure('FACADE_CSRF_DENIED');
            }
            $csrf = self::verifyToken(
                $request['csrf_token'], $secret, 'csrf', $request['facade_id'],
                $request['origin'], $request['route'], $now, self::CSRF_TTL_SECONDS,
                $session['subject']
            );
            if (!$csrf['ok'] || $consumeCsrf($request['facade_id'], $session['subject'], $csrf['nonce'], $csrf['expires_at']) !== true) {
                return self::failure('FACADE_CSRF_DENIED');
            }
        }

        return [
            'ok' => true,
            'authority_route' => $registry['proxy_routes'][$request['route']],
            'safe_redirect' => $request['origin'] . $facade['callbacks'][$request['redirect_handle']],
            'response_headers' => self::responseHeaders($request['origin']),
            'session_id' => $session['subject'],
        ];
    }

    public static function responseHeaders(string $origin): array
    {
        if (!preg_match('#^https://[a-z0-9.-]+$#D', $origin)) {
            throw new InvalidArgumentException('origin must be an exact HTTPS origin');
        }
        return [
            'Access-Control-Allow-Origin' => $origin,
            'Access-Control-Allow-Credentials' => 'true',
            'Access-Control-Allow-Methods' => 'GET, POST',
            'Access-Control-Allow-Headers' => 'Content-Type, X-CSRF-Token, X-Request-ID, Idempotency-Key',
            'Vary' => 'Origin',
            'Content-Security-Policy' => "default-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'; object-src 'none'",
            'Referrer-Policy' => 'no-referrer',
            'X-Content-Type-Options' => 'nosniff',
            'Cache-Control' => 'no-store',
        ];
    }

    public static function maskedResponse(array $authorityResult): array
    {
        if (($authorityResult['ok'] ?? true) !== true || isset($authorityResult['error'])) {
            return self::failure('ACTIVATION_REQUEST_ACCEPTED')['envelope'];
        }
        $result = ['schema' => 'focusa.spec152e.masked_activation_envelope.v1'];
        foreach (self::PUBLIC_RESPONSE_FIELDS as $field) {
            if (array_key_exists($field, $authorityResult)) {
                $result[$field] = $authorityResult[$field];
            }
        }
        if (isset($authorityResult['email']) && is_string($authorityResult['email'])) {
            $result['masked_email'] = self::maskEmail($authorityResult['email']);
        }
        if (isset($result['masked_email']) && (!is_string($result['masked_email']) || strpos($result['masked_email'], '*') === false)) {
            unset($result['masked_email']);
        }
        return $result;
    }

    private static function issueToken(
        string $secret, string $kind, string $facadeId, string $origin,
        string $subject, string $route, int $now, int $ttl, int $maxTtl,
        ?string $nonce = null
    ): string {
        if ($secret === '' || $ttl < 1 || $ttl > $maxTtl) {
            throw new InvalidArgumentException('invalid token key or TTL');
        }
        foreach ([$kind, $facadeId, $origin, $subject, $route] as $claim) {
            if ($claim === '' || strlen($claim) > 512 || preg_match('/[\r\n]/', $claim)) {
                throw new InvalidArgumentException('invalid token claim');
            }
        }
        $claims = [
            'v' => 1, 'kind' => $kind, 'facade_id' => $facadeId, 'origin' => $origin,
            'subject' => $subject, 'route' => $route, 'nonce' => $nonce ?? $sessionNonce = bin2hex(random_bytes(16)),
            'iat' => $now, 'exp' => $now + $ttl,
        ];
        $payload = self::base64UrlEncode(json_encode($claims, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES));
        return $payload . '.' . hash_hmac('sha256', "facade-security-v1\n" . $payload, $secret);
    }

    private static function verifyToken(
        string $token, string $secret, string $kind, string $facadeId,
        string $origin, string $route, int $now, int $maxTtl,
        ?string $subject = null
    ): array {
        $parts = explode('.', $token);
        if (count($parts) !== 2 || !preg_match('/^[A-Za-z0-9_-]+$/D', $parts[0]) || !preg_match('/^[a-f0-9]{64}$/D', $parts[1])
            || !hash_equals(hash_hmac('sha256', "facade-security-v1\n" . ($parts[0] ?? ''), $secret), $parts[1] ?? '')) {
            return ['ok' => false];
        }
        $decoded = self::base64UrlDecode($parts[0]);
        try {
            $claims = $decoded === null ? null : json_decode($decoded, true, 32, JSON_THROW_ON_ERROR);
        } catch (JsonException $error) {
            return ['ok' => false];
        }
        if (!is_array($claims) || ($claims['v'] ?? null) !== 1 || ($claims['kind'] ?? null) !== $kind
            || ($claims['facade_id'] ?? null) !== $facadeId || ($claims['origin'] ?? null) !== $origin
            || ($claims['route'] ?? null) !== $route || !is_int($claims['iat'] ?? null) || !is_int($claims['exp'] ?? null)
            || $claims['iat'] > $now || $claims['exp'] <= $now || $claims['exp'] - $claims['iat'] > $maxTtl
            || !is_string($claims['subject'] ?? null) || !is_string($claims['nonce'] ?? null)
            || ($subject !== null && !hash_equals($subject, $claims['subject']))) {
            return ['ok' => false];
        }
        return ['ok' => true, 'subject' => $claims['subject'], 'nonce' => $claims['nonce'], 'expires_at' => $claims['exp']];
    }

    private static function assertFacadeOrigin(array $registry, string $facadeId, string $origin): void
    {
        $facade = self::facade($registry, $facadeId);
        if ($facade === null || !in_array($origin, $facade['exact_origins'] ?? [], true)) {
            throw new InvalidArgumentException('unregistered facade origin');
        }
    }

    private static function facade(array $registry, string $facadeId): ?array
    {
        foreach (($registry['facades'] ?? []) as $facade) {
            if (is_array($facade) && isset($facade['facade_id']) && is_string($facade['facade_id']) && hash_equals($facade['facade_id'], $facadeId)) {
                return $facade;
            }
        }
        return null;
    }

    private static function failure(string $code, int $status = 400): array
    {
        return ['ok' => false, 'status' => $status, 'error' => $code, 'envelope' => [
            'schema' => 'focusa.spec152e.masked_error.v1',
            'error' => $code,
            'next_action' => 'retry_or_recover_through_registered_facade',
        ]];
    }

    private static function maskEmail(string $email): string
    {
        $at = strrpos($email, '@');
        return $at === false || $at < 1 || $at === strlen($email) - 1
            ? '***' : substr($email, 0, 1) . '***@' . substr($email, $at + 1);
    }

    private static function base64UrlEncode(string $value): string
    {
        return rtrim(strtr(base64_encode($value), '+/', '-_'), '=');
    }

    private static function base64UrlDecode(string $value): ?string
    {
        $padding = (4 - strlen($value) % 4) % 4;
        $decoded = base64_decode(strtr($value . str_repeat('=', $padding), '-_', '+/'), true);
        return $decoded === false ? null : $decoded;
    }
}
