<?php
// Activation start handler: validates product/facade, creates pending registration,
// delivers branded verification challenge, and returns enumeration-resistant status.
// Does not create EDD customer, checkout, license, node, or lease.
declare(strict_types=1);

final class FocusaSpec152eActivationStartHandler
{
    public const SCHEMA = 'focusa.spec152e.activation_start_handler.v1';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eChallengeService $challenges,
        private FocusaSpec152eTransactionalMailAdapter $mail,
        private FocusaSpec152eRateLimiter $rateLimiter,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Handle an activation start request.
     *
     * Required input:
     *   - facade_id:         registered facade ID
     *   - origin:            exact facade origin
     *   - product_code:      public product code
     *   - presenter:         presenter identifier (terminal, agent_json, browser, cli, etc.)
     *   - install_channel:   install channel (source_build, official_installer, etc.)
     *   - email:             submitted email address
     *   - request_id:        bounded request ID
     *   - idempotency_key:   bounded idempotency key
     *   - safe_redirect_handle: optional redirect handle
     *   - device_public_key: optional device public key
     *
     * Requires:
     *   - facade_registry:   full facade registry (from spec152e-facade-registry.v1.php)
     *   - product_registry:  full product registry (from spec152e-edd-product-registry.v1.php)
     *   - opaque_client_key: opaque key derived from the caller (session hash, IP hash, etc.)
     *
     * Returns a masked envelope (never contains raw email, verifier, or authority secrets).
     */
    public function start(array $input, array $facadeRegistry, array $productRegistry, string $opaqueClientKey): array
    {
        // 1. Validate facade and origin.
        $facade = $this->resolveFacade($facadeRegistry, (string) ($input['facade_id'] ?? ''), (string) ($input['origin'] ?? ''));
        if ($facade === null) {
            return $this->maskedFailure('FACADE_ORIGIN_DENIED', (string) ($input['request_id'] ?? ''));
        }

        // 2. Validate product code against facade allowlist and product registry.
        $productCode = (string) ($input['product_code'] ?? '');
        if (!in_array($productCode, $facade['products'] ?? [], true)) {
            return $this->maskedFailure('FACADE_PRODUCT_DENIED', (string) ($input['request_id'] ?? ''));
        }
        $product = $this->resolveProduct($productRegistry, $productCode);
        if ($product === null) {
            return $this->maskedFailure('PRODUCT_MAPPING_REQUIRED', (string) ($input['request_id'] ?? ''));
        }

        // 3. Rate limit — enumeration resistant. Counts the attempt regardless of outcome.
        $route = 'activation_start';
        if (!$this->rateLimiter->allow($facade['facade_id'], $opaqueClientKey, $route)) {
            return $this->maskedFailure('ACTIVATION_REQUEST_ACCEPTED', (string) ($input['request_id'] ?? ''));
        }

        // 4. Validate email.
        $email = (string) ($input['email'] ?? '');
        try {
            $normalized = FocusaSpec152eEmailNormalizer::exact($email);
        } catch (InvalidArgumentException) {
            // Invalid email still returns the same masked response — enumeration resistant.
            return $this->maskedFailure('ACTIVATION_REQUEST_ACCEPTED', (string) ($input['request_id'] ?? ''));
        }

        // 5. Create pending registration.
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $presenter = (string) ($input['presenter'] ?? '');
        $installChannel = (string) ($input['install_channel'] ?? '');

        try {
            $result = $this->registrations->createPending([
                'email' => $email,
                'facade_id' => $facade['facade_id'],
                'presenter' => $presenter !== '' ? $presenter : 'terminal',
                'install_channel' => $installChannel !== '' ? $installChannel : 'source_build',
                'product_code' => $productCode,
                'safe_redirect_handle' => $input['safe_redirect_handle'] ?? null,
                'device_public_key' => $input['device_public_key'] ?? null,
                'request_id' => $requestId,
                'idempotency_key' => $idempotencyKey,
            ]);
        } catch (InvalidArgumentException | DomainException $error) {
            // Input validation failure — still enumeration resistant.
            return $this->maskedFailure('ACTIVATION_REQUEST_ACCEPTED', $requestId);
        }

        $registration = $result['registration'];
        $verificationSecret = $result['verification_secret'] ?? null;
        $pollCredential = $result['poll_credential'] ?? null;
        $replayed = $result['replayed'] ?? false;

        // 6. Only send the challenge on the first attempt (not replay).
        $deliveryStatus = 'none';
        if (!$replayed && $verificationSecret !== null) {
            $deliveryStatus = $this->deliverChallenge(
                $facade,
                $normalized,
                $registration['registration_uuid'],
                $verificationSecret,
                $registration['verification_challenge_expires_at'],
                $presenter,
                $productCode,
            );
        }

        // 7. Build masked envelope.
        $envelope = $this->buildMaskedEnvelope($registration, $pollCredential ?? '', $deliveryStatus, $replayed);

        return $envelope;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function resolveFacade(array $registry, string $facadeId, string $origin): ?array
    {
        foreach (($registry['facades'] ?? []) as $facade) {
            if (!is_array($facade) || !isset($facade['facade_id'])) {
                continue;
            }
            if (hash_equals($facade['facade_id'], $facadeId)
                && in_array($origin, $facade['exact_origins'] ?? [], true)) {
                return $facade;
            }
        }
        return null;
    }

    private function resolveProduct(array $registry, string $productCode): ?array
    {
        foreach (($registry['protected_offers'] ?? []) as $offer) {
            if (is_array($offer) && ($offer['public_code'] ?? '') === $productCode) {
                return $offer;
            }
        }
        return null;
    }

    private function deliverChallenge(
        array $facade,
        string $normalizedEmail,
        string $registrationUuid,
        string $verificationSecret,
        string $expiresAt,
        string $presenter,
        string $productCode,
    ): string {
        $kind = FocusaSpec152eChallengeService::challengeKind($facade, $presenter);
        $now = ($this->clock)();
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($now);

        if ($kind === 'magic_link') {
            $verificationPath = $facade['paths']['verification'] ?? '/activate/verify';
            $origin = $facade['exact_origins'][0] ?? '';
            $challenge = $this->challenges->generateMagicLink(
                $facade['facade_id'],
                $verificationPath,
                $registrationUuid,
                $origin,
                $now,
                $expiresAt,
            );
            // Override the stored hash with our own generated one.
            // (The registration already has a hash; we need the challenge to match it.
            // In practice this is wired through the ChallengeService hash key matching the
            // RegistrationSecrets verification key. For the contract, we use the secret
            // from the registration and build the magic link ourselves.)
            $magicLink = $this->buildMagicLink($facade, $registrationUuid, $verificationSecret, $expiresAt);
            $mailInput = [
                'facade' => $facade,
                'to' => $normalizedEmail,
                'challenge_kind' => 'magic_link',
                'magic_link' => $magicLink,
                'expires_at' => $expiresAt,
                'registration_id' => $registrationUuid,
                'product_code' => $productCode,
            ];
        } else {
            $mailInput = [
                'facade' => $facade,
                'to' => $normalizedEmail,
                'challenge_kind' => 'otp',
                'otp_code' => $verificationSecret,
                'expires_at' => $expiresAt,
                'registration_id' => $registrationUuid,
                'product_code' => $productCode,
            ];
        }

        try {
            $delivery = $this->mail->sendVerificationChallenge($mailInput);
            return $delivery['delivery_status'] ?? 'failed';
        } catch (Throwable) {
            return 'failed';
        }
    }

    private function buildMagicLink(array $facade, string $registrationUuid, string $verifier, string $expiresAt): string
    {
        $verificationPath = $facade['paths']['verification'] ?? '/activate/verify';
        $origin = $facade['exact_origins'][0] ?? '';
        $params = http_build_query([
            'registration' => $registrationUuid,
            'token' => $verifier,
        ]);
        return $origin . $verificationPath . '?' . $params;
    }

    private function buildMaskedEnvelope(array $registration, string $pollCredential, string $deliveryStatus, bool $replayed): array
    {
        $snapshot = FocusaSpec152eActivationRegistrationPresenter::snapshot($registration);
        $envelope = [
            'schema' => 'focusa.spec152e.masked_activation_envelope.v1',
            'request_id' => $registration['request_id'],
            'registration_id' => $registration['registration_uuid'],
            'state' => $snapshot['state'],
            'terminal' => $snapshot['terminal'],
            'retry' => $snapshot['retry'],
            'next_action' => $snapshot['next_action'],
            'verification_delivery_status' => $deliveryStatus,
        ];

        // Masked email: first character + ***@domain
        $email = $registration['encrypted_normalized_email'] ?? '';
        if ($email !== '') {
            // We cannot decrypt here (handler doesn't have the key), so we use the public
            // snapshot which doesn't expose email. The caller (facade security layer) will
            // mask the email before returning to the client.
        }

        return $envelope;
    }

    private function maskedFailure(string $code, string $requestId): array
    {
        return [
            'schema' => 'focusa.spec152e.masked_error.v1',
            'error' => $code,
            'request_id' => $requestId,
            'next_action' => 'retry_or_recover_through_registered_facade',
        ];
    }
}

/**
 * Lightweight activation start result presenter for direct use in tests and
 * facade security layers. Returns only the safe public fields.
 */
final class FocusaSpec152eActivationStartPresenter
{
    public const SCHEMA = 'focusa.spec152e.activation_start_result.v1';

    /**
     * Present a safe, enumeration-resistant result from the start handler.
     * Never exposes raw email, verifier, or poll credential.
     */
    public static function present(array $handlerResult, ?string $normalizedEmail = null): array
    {
        // If it's an error, return the masked error as-is.
        if (isset($handlerResult['error'])) {
            return $handlerResult;
        }

        $result = [
            'schema' => self::SCHEMA,
            'request_id' => $handlerResult['request_id'] ?? '',
            'registration_id' => $handlerResult['registration_id'] ?? '',
            'state' => $handlerResult['state'] ?? 'attempt_created',
            'terminal' => $handlerResult['terminal'] ?? false,
            'retry' => $handlerResult['retry'] ?? ['posture' => 'safe_retry'],
            'next_action' => $handlerResult['next_action'] ?? 'continue_activation',
            'verification_delivery_status' => $handlerResult['verification_delivery_status'] ?? 'none',
        ];

        // Mask email if provided.
        if ($normalizedEmail !== null) {
            $result['masked_email'] = self::maskEmail($normalizedEmail);
        }

        return $result;
    }

    private static function maskEmail(string $email): string
    {
        $at = strrpos($email, '@');
        if ($at === false || $at < 1 || $at === strlen($email) - 1) {
            return '***';
        }
        return substr($email, 0, 1) . '***@' . substr($email, $at + 1);
    }
}