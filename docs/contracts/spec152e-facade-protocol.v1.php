<?php
// Public protocol adapter. Credentials are injected at runtime and never stored here.
declare(strict_types=1);

final class FocusaSpec152eFacadeProtocol
{
    public const SCHEMA = 'focusa.spec152e.facade_protocol.v1';
    public const MAX_SKEW_SECONDS = 300;

    private const SIGNED_FIELDS = [
        'schema', 'credential_id', 'timestamp', 'nonce', 'request_id',
        'idempotency_key', 'registration_id', 'facade_id', 'origin',
        'product_code', 'action', 'redirect_handle', 'continuation_token',
        'body_sha256',
    ];

    private const ALLOWED_REQUEST_FIELDS = [
        'schema', 'credential_id', 'timestamp', 'nonce', 'request_id',
        'idempotency_key', 'registration_id', 'facade_id', 'origin',
        'product_code', 'action', 'redirect_handle', 'continuation_token',
        'body_sha256', 'signature',
    ];

    public static function signRequest(array $request, string $credentialId, string $credential): array
    {
        $signed = self::normalizeRequest($request + [
            'schema' => self::SCHEMA,
            'credential_id' => $credentialId,
            'continuation_token' => '',
            'redirect_handle' => '',
        ]);
        $signed['signature'] = self::mac($credential, self::canonicalRequest($signed));
        return $signed;
    }

    /**
     * $credentialResolver returns ['facade_id' => string, 'credential' => string, 'active' => bool].
     * $consumeNonce atomically returns true only for a previously unseen credential/facade/nonce tuple.
     */
    public static function verifyRequest(
        array $request,
        array $registry,
        callable $credentialResolver,
        callable $consumeNonce,
        int $now
    ): array {
        if (array_diff(array_keys($request), self::ALLOWED_REQUEST_FIELDS) !== []) {
            return self::failure('FACADE_REQUEST_FIELD_DENIED');
        }
        try {
            $signed = self::normalizeRequest($request);
        } catch (InvalidArgumentException $error) {
            return self::failure('FACADE_REQUEST_INVALID');
        }
        if (!isset($request['signature']) || !is_string($request['signature']) || !preg_match('/^[a-f0-9]{64}$/D', $request['signature'])) {
            return self::failure('FACADE_AUTH_FAILED');
        }
        if (abs($now - $signed['timestamp']) > self::MAX_SKEW_SECONDS) {
            return self::failure('FACADE_TIMESTAMP_DENIED');
        }

        $credential = $credentialResolver($signed['credential_id']);
        if (!is_array($credential) || ($credential['active'] ?? false) !== true
            || !isset($credential['facade_id'], $credential['credential'])
            || !is_string($credential['facade_id']) || !is_string($credential['credential'])
            || !hash_equals($credential['facade_id'], $signed['facade_id'])
            || !hash_equals(self::mac($credential['credential'], self::canonicalRequest($signed)), $request['signature'])) {
            return self::failure('FACADE_AUTH_FAILED');
        }

        $facade = self::registeredFacade($registry, $signed['facade_id']);
        if ($facade === null || !in_array($signed['origin'], $facade['exact_origins'], true)) {
            return self::failure('FACADE_ORIGIN_DENIED');
        }
        if (!in_array($signed['product_code'], $facade['products'], true)) {
            return self::failure('FACADE_PRODUCT_DENIED');
        }
        if (!isset($registry['proxy_routes'][$signed['action']])) {
            return self::failure('FACADE_ACTION_DENIED');
        }
        if (!isset($facade['callbacks'][$signed['redirect_handle']])) {
            return self::failure('FACADE_REDIRECT_DENIED');
        }
        if ($signed['continuation_token'] !== '') {
            $tokenDecision = self::verifyContinuationToken(
                $signed['continuation_token'],
                $credential['credential'],
                $signed['registration_id'],
                $signed['facade_id'],
                $signed['action'],
                $signed['nonce'],
                $now
            );
            if (!$tokenDecision['ok']) {
                return $tokenDecision;
            }
        }
        if ($consumeNonce($signed['credential_id'], $signed['facade_id'], $signed['nonce'], $signed['timestamp']) !== true) {
            return self::failure('FACADE_REPLAY_DENIED');
        }

        return [
            'ok' => true,
            'request' => $signed,
            'authority_route' => $registry['proxy_routes'][$signed['action']],
            'safe_redirect' => $signed['origin'] . $facade['callbacks'][$signed['redirect_handle']],
        ];
    }

    public static function issueContinuationToken(
        string $credential,
        string $registrationId,
        string $facadeId,
        string $action,
        string $nonce,
        int $expiresAt
    ): string {
        $claims = self::validateClaims([$registrationId, $facadeId, $action, $nonce, (string) $expiresAt]);
        $payload = self::base64UrlEncode(implode("\n", $claims));
        return $payload . '.' . self::mac($credential, 'continuation-v1' . "\n" . $payload);
    }

    public static function verifyContinuationToken(
        string $token,
        string $credential,
        string $registrationId,
        string $facadeId,
        string $action,
        string $nonce,
        int $now
    ): array {
        $parts = explode('.', $token);
        if (count($parts) !== 2 || !preg_match('/^[A-Za-z0-9_-]+$/D', $parts[0]) || !preg_match('/^[a-f0-9]{64}$/D', $parts[1])) {
            return self::failure('FACADE_CONTINUATION_DENIED');
        }
        if (!hash_equals(self::mac($credential, 'continuation-v1' . "\n" . $parts[0]), $parts[1])) {
            return self::failure('FACADE_CONTINUATION_DENIED');
        }
        $decoded = self::base64UrlDecode($parts[0]);
        $claims = $decoded === null ? [] : explode("\n", $decoded);
        if (count($claims) !== 5 || !ctype_digit($claims[4]) || (int) $claims[4] < $now) {
            return self::failure('FACADE_CONTINUATION_DENIED');
        }
        foreach ([$registrationId, $facadeId, $action, $nonce] as $index => $expected) {
            if (!hash_equals($expected, $claims[$index])) {
                return self::failure('FACADE_CONTINUATION_DENIED');
            }
        }
        return ['ok' => true, 'expires_at' => (int) $claims[4]];
    }

    public static function maskedEnvelope(array $authorityResult): array
    {
        $allowed = ['request_id', 'registration_id', 'state', 'terminal', 'retry', 'next_action', 'safe_url', 'verification_delivery_status', 'one_time_key_envelope', 'node_id', 'lease_envelope', 'error'];
        $envelope = ['schema' => 'focusa.spec152e.masked_activation_envelope.v1'];
        foreach ($allowed as $field) {
            if (array_key_exists($field, $authorityResult)) {
                $envelope[$field] = $authorityResult[$field];
            }
        }
        if (isset($authorityResult['email']) && is_string($authorityResult['email'])) {
            $envelope['masked_email'] = self::maskEmail($authorityResult['email']);
        } elseif (isset($authorityResult['masked_email']) && is_string($authorityResult['masked_email']) && strpos($authorityResult['masked_email'], '*') !== false) {
            $envelope['masked_email'] = $authorityResult['masked_email'];
        }
        return $envelope;
    }

    public static function canonicalRequest(array $request): string
    {
        $values = [];
        foreach (self::SIGNED_FIELDS as $field) {
            if (!array_key_exists($field, $request)) {
                throw new InvalidArgumentException('missing signed field');
            }
            $values[] = (string) $request[$field];
        }
        return implode("\n", $values);
    }

    private static function normalizeRequest(array $request): array
    {
        $normalized = [];
        foreach (self::SIGNED_FIELDS as $field) {
            if (!array_key_exists($field, $request) || (!is_string($request[$field]) && !is_int($request[$field]))) {
                throw new InvalidArgumentException('invalid signed field');
            }
            $value = (string) $request[$field];
            if ($value === '' && !in_array($field, ['continuation_token'], true)) {
                throw new InvalidArgumentException('empty signed field');
            }
            if (strpos($value, "\n") !== false || strpos($value, "\r") !== false || strlen($value) > 4096) {
                throw new InvalidArgumentException('unsafe signed field');
            }
            $normalized[$field] = $value;
        }
        if ($normalized['schema'] !== self::SCHEMA || !ctype_digit($normalized['timestamp'])
            || !preg_match('/^[a-f0-9]{64}$/D', $normalized['body_sha256'])) {
            throw new InvalidArgumentException('invalid protocol field');
        }
        $normalized['timestamp'] = (int) $normalized['timestamp'];
        return $normalized;
    }

    private static function registeredFacade(array $registry, string $facadeId): ?array
    {
        foreach (($registry['facades'] ?? []) as $facade) {
            if (isset($facade['facade_id']) && is_string($facade['facade_id']) && hash_equals($facade['facade_id'], $facadeId)) {
                return $facade;
            }
        }
        return null;
    }

    private static function validateClaims(array $claims): array
    {
        foreach ($claims as $claim) {
            if ($claim === '' || strpos($claim, "\n") !== false || strpos($claim, "\r") !== false || strlen($claim) > 512) {
                throw new InvalidArgumentException('invalid continuation claim');
            }
        }
        return $claims;
    }

    private static function maskEmail(string $email): string
    {
        $at = strrpos($email, '@');
        if ($at === false || $at === 0 || $at === strlen($email) - 1) {
            return '***';
        }
        return substr($email, 0, 1) . '***@' . substr($email, $at + 1);
    }

    private static function mac(string $credential, string $message): string
    {
        return hash_hmac('sha256', $message, $credential);
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

    private static function failure(string $code): array
    {
        return ['ok' => false, 'error' => $code, 'envelope' => [
            'schema' => 'focusa.spec152e.masked_error.v1',
            'error' => $code,
            'next_action' => 'retry_or_recover_through_registered_facade',
        ]];
    }
}
