<?php
// Verification completion handler: validates the submitted verifier against the stored
// challenge hash, enforces facade/origin binding, expiry, attempt limits, and replay
// defense. Only a live matching challenge reaches email_verified; all negative cases
// return enumeration-resistant safe errors.
declare(strict_types=1);

final class FocusaSpec152eVerificationCompleteHandler
{
    public const SCHEMA = 'focusa.spec152e.verification_complete_handler.v1';
    public const MAX_VERIFICATION_ATTEMPTS = 5;

    public function __construct(
        private FocusaSpec152eActivationRegistrationRepository $registrations,
        private FocusaSpec152eRateLimiter $rateLimiter,
    ) {
    }

    /**
     * Complete a verification challenge.
     *
     * Required input:
     *   - registration_uuid:  registration UUID from the challenge-delivery step
     *   - verifier:           the submitted magic-link token or OTP code
     *   - facade_id:          registered facade ID
     *   - origin:             exact facade origin
     *   - request_id:         bounded request ID
     *   - idempotency_key:    bounded idempotency key
     *
     * Requires:
     *   - facade_registry:   full facade registry (from spec152e-facade-registry.v1.php)
     *   - opaque_client_key: opaque key derived from the caller (session hash, IP hash, etc.)
     *
     * Returns a masked envelope (never contains raw email, verifier, or authority secrets).
     * Only a live matching challenge bound to the registration/facade reaches email_verified.
     * Wrong, expired, replayed, and cross-facade tokens are rejected with stable safe errors.
     */
    public function complete(array $input, array $facadeRegistry, string $opaqueClientKey): array
    {
        // 1. Validate facade and origin.
        $facade = $this->resolveFacade($facadeRegistry, (string) ($input['facade_id'] ?? ''), (string) ($input['origin'] ?? ''));
        if ($facade === null) {
            return $this->maskedFailure('FACADE_ORIGIN_DENIED', (string) ($input['request_id'] ?? ''));
        }

        // 2. Rate limit — enumeration resistant. Counts the attempt regardless of outcome.
        $route = 'verification_complete';
        if (!$this->rateLimiter->allow($facade['facade_id'], $opaqueClientKey, $route)) {
            return $this->maskedFailure('ACTIVATION_REQUEST_ACCEPTED', (string) ($input['request_id'] ?? ''));
        }

        // 3. Extract and validate input fields.
        $registrationUuid = (string) ($input['registration_uuid'] ?? '');
        $verifier = (string) ($input['verifier'] ?? '');
        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');

        // 4. Validate verifier bounds.
        if ($verifier === '' || strlen($verifier) > 256 || preg_match('/[\r\n]/', $verifier)) {
            return $this->maskedFailure('EMAIL_VERIFICATION_FAILED', $requestId);
        }

        // 5. Look up the registration — enumeration resistant.
        // InvalidArgumentException (bad UUID format) and OutOfBoundsException (not found)
        // both map to the same safe, non-enumerating error.
        try {
            $registration = $this->registrations->findByUuid($registrationUuid);
        } catch (InvalidArgumentException | OutOfBoundsException) {
            return $this->maskedFailure('EMAIL_VERIFICATION_REQUIRED', $requestId);
        }

        // 6. Cross-facade binding: the registration must be on the same facade.
        if (!hash_equals((string) $registration['facade_id'], $facade['facade_id'])) {
            return $this->maskedFailure('EMAIL_VERIFICATION_REQUIRED', $requestId);
        }

        // 7. Enforce max verification attempts before calling the repository.
        // The repository increments on failure; we check here so the handler
        // can return a stable error before the CAS version bump.
        if ((int) $registration['verification_attempts'] >= self::MAX_VERIFICATION_ATTEMPTS) {
            return $this->maskedFailure('EMAIL_VERIFICATION_FAILED', $requestId);
        }

        // 8. Attempt verification through the repository.
        try {
            $result = $this->registrations->verifyEmail($registrationUuid, $verifier, $requestId, $idempotencyKey);
        } catch (DomainException $error) {
            $code = $error->getMessage();
            // Stable safe errors from the repository.
            if (in_array($code, [
                'EMAIL_VERIFICATION_FAILED',
                'EMAIL_VERIFICATION_EXPIRED',
                'EMAIL_VERIFICATION_REQUIRED',
                'REGISTRATION_EXPIRED',
            ], true)) {
                return $this->maskedFailure($code, $requestId);
            }
            // Any unexpected error is mapped to a safe, non-enumerating fallback.
            return $this->maskedFailure('EMAIL_VERIFICATION_FAILED', $requestId);
        } catch (InvalidArgumentException $error) {
            return $this->maskedFailure('EMAIL_VERIFICATION_FAILED', $requestId);
        }

        // 9. Build masked envelope from the verified registration.
        return $this->buildMaskedEnvelope($result['registration'], $requestId, $result['replayed'] ?? false);
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

    private function buildMaskedEnvelope(array $registration, string $requestId, bool $replayed): array
    {
        $snapshot = FocusaSpec152eActivationRegistrationPresenter::snapshot($registration);
        return [
            'schema' => 'focusa.spec152e.masked_verification_envelope.v1',
            'request_id' => $requestId,
            'registration_id' => $registration['registration_uuid'],
            'state' => $snapshot['state'],
            'terminal' => $snapshot['terminal'],
            'retry' => $snapshot['retry'],
            'next_action' => $snapshot['next_action'],
            'replayed' => $replayed,
        ];
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