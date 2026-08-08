<?php
// Email delivery, bounce, suppression, and consent settlement handler.
// Records send/delivery/bounce/suppression without raw tokens; holds promotion
// when delivery cannot prove mailbox control; settles transactional consent
// independently from optional promotional consent.
declare(strict_types=1);

final class FocusaSpec152eEmailDeliveryConsentHandler
{
    public const SCHEMA = 'focusa.spec152e.email_delivery_consent_handler.v1';

    public function __construct(
        private FocusaSpec152eEmailIdentityRepository $identities,
        private FocusaSpec152eActivationRegistrationRepository $registrations,
    ) {
    }

    // ── delivery outcome ───────────────────────────────────────────────

    /**
     * Process a transactional email provider delivery callback.
     *
     * Required input:
     *   - identity_uuid:     email identity UUID
     *   - message_kind:      'verification' | 'transactional' | 'promotional'
     *   - delivery_status:   'sent' | 'delivered' | 'bounced' | 'complained'
     *   - bounce_type:       'soft' | 'hard' (only when delivery_status is 'bounced')
     *   - occurred_at:       canonical UTC timestamp
     *   - request_id:        bounded request ID
     *   - idempotency_key:   bounded idempotency key
     *
     * Returns a masked envelope (never contains raw email, tokens, or authority secrets).
     * Bounce/suppression does not create verified identity.
     * Promotional consent never gates required transactional messages.
     */
    public function recordDeliveryOutcome(array $input): array
    {
        try {
            return $this->doRecordDeliveryOutcome($input);
        } catch (InvalidArgumentException) {
            return $this->maskedFailure('EMAIL_DELIVERY_FAILED', (string) ($input['request_id'] ?? ''));
        }
    }

    private function doRecordDeliveryOutcome(array $input): array
    {
        $identityUuid = (string) ($input['identity_uuid'] ?? '');
        $this->assertUuid($identityUuid, 'identity');

        $messageKind = (string) ($input['message_kind'] ?? '');
        if (!in_array($messageKind, ['verification', 'transactional', 'promotional'], true)) {
            throw new InvalidArgumentException('bounded message kind required');
        }

        $deliveryStatus = (string) ($input['delivery_status'] ?? '');
        if (!in_array($deliveryStatus, ['sent', 'delivered', 'bounced', 'complained'], true)) {
            throw new InvalidArgumentException('bounded delivery status required');
        }

        $occurredAt = (string) ($input['occurred_at'] ?? '');
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($occurredAt);

        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertToken($requestId, 128, 'request_id');
        $this->assertToken($idempotencyKey, 128, 'idempotency_key');

        // Resolve identity.
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return $this->maskedFailure('EMAIL_IDENTITY_NOT_FOUND', $requestId);
        }

        // Determine bounce and suppression state from the delivery outcome.
        $bounceState = $identity['bounce_state'] ?? 'none';
        $suppressionState = $identity['suppression_state'] ?? 'none';

        if ($deliveryStatus === 'bounced') {
            $bounceType = (string) ($input['bounce_type'] ?? 'soft');
            if (!in_array($bounceType, ['soft', 'hard'], true)) {
                throw new InvalidArgumentException('bounded bounce type required');
            }
            // A hard bounce is higher severity; don't downgrade hard to soft.
            if ($bounceState !== 'hard' && $bounceType === 'hard') {
                $bounceState = 'hard';
            } elseif ($bounceState === 'none' && $bounceType === 'soft') {
                $bounceState = 'soft';
            }
        }

        if ($deliveryStatus === 'complained') {
            // Complaint/spam report suppresses the kind of message that was reported
            // plus any promotional messages.
            if ($messageKind === 'promotional' || $messageKind === 'verification') {
                $suppressionState = $this->maxSuppression($suppressionState, 'promotional');
            }
            if ($messageKind === 'transactional' || $messageKind === 'verification') {
                $suppressionState = $this->maxSuppression($suppressionState, 'transactional');
            }
            // A complaint on any message type also suppresses all future promotional.
            $suppressionState = $this->maxSuppression($suppressionState, 'promotional');
        }

        // Apply the delivery state.
        $updated = $this->identities->recordDeliveryState($identityUuid, $bounceState, $suppressionState, $occurredAt);

        return $this->buildMaskedDeliveryEnvelope($updated, $messageKind, $deliveryStatus, $requestId, $occurredAt);
    }

    // ── consent settlement ─────────────────────────────────────────────

    /**
     * Settle transactional consent.
     *
     * Transactional consent is required for operational messages (verification,
     * license delivery, account notifications). Settled independently from
     * promotional consent.
     *
     * Required input:
     *   - identity_uuid:     email identity UUID
     *   - occurred_at:       canonical UTC timestamp
     *   - request_id:        bounded request ID
     *   - idempotency_key:   bounded idempotency key
     */
    public function settleTransactionalConsent(array $input): array
    {
        $identityUuid = (string) ($input['identity_uuid'] ?? '');
        $this->assertUuid($identityUuid, 'identity');

        $occurredAt = (string) ($input['occurred_at'] ?? '');
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($occurredAt);

        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertToken($requestId, 128, 'request_id');
        $this->assertToken($idempotencyKey, 128, 'idempotency_key');

        // Resolve identity.
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return $this->maskedFailure('EMAIL_IDENTITY_NOT_FOUND', $requestId);
        }

        // Only verified identities can settle consent.
        if (($identity['verified_at'] ?? null) === null || !in_array($identity['identity_state'] ?? '', ['primary', 'linked'], true)) {
            return $this->maskedFailure('EMAIL_VERIFICATION_REQUIRED', $requestId);
        }

        // Bounce/suppression does not prevent transactional consent settlement.
        // Transactional consent is settled independently from promotional consent.
        try {
            $updated = $this->identities->settleConsent($identityUuid, 'transactional_consent_at', $occurredAt);
        } catch (DomainException) {
            // Consent already settled — idempotent, return current state.
            $updated = $this->identities->findByUuid($identityUuid);
        }

        return $this->buildMaskedConsentEnvelope($updated, 'transactional', $requestId, $occurredAt);
    }

    /**
     * Settle promotional consent.
     *
     * Promotional consent is optional and never gates required transactional
     * messages. Settled independently from transactional consent.
     *
     * Required input:
     *   - identity_uuid:     email identity UUID
     *   - occurred_at:       canonical UTC timestamp
     *   - request_id:        bounded request ID
     *   - idempotency_key:   bounded idempotency key
     */
    public function settlePromotionalConsent(array $input): array
    {
        $identityUuid = (string) ($input['identity_uuid'] ?? '');
        $this->assertUuid($identityUuid, 'identity');

        $occurredAt = (string) ($input['occurred_at'] ?? '');
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($occurredAt);

        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertToken($requestId, 128, 'request_id');
        $this->assertToken($idempotencyKey, 128, 'idempotency_key');

        // Resolve identity.
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return $this->maskedFailure('EMAIL_IDENTITY_NOT_FOUND', $requestId);
        }

        // Only verified identities can settle consent.
        if (($identity['verified_at'] ?? null) === null || !in_array($identity['identity_state'] ?? '', ['primary', 'linked'], true)) {
            return $this->maskedFailure('EMAIL_VERIFICATION_REQUIRED', $requestId);
        }

        // Hard bounce or suppression prevents promotional consent settlement.
        if ($identity['bounce_state'] === 'hard' || in_array($identity['suppression_state'], ['promotional', 'all'], true)) {
            return $this->maskedFailure('EMAIL_DELIVERY_FAILED', $requestId);
        }

        try {
            $updated = $this->identities->settleConsent($identityUuid, 'promotional_consent_at', $occurredAt);
        } catch (DomainException) {
            // Consent already settled — idempotent, return current state.
            $updated = $this->identities->findByUuid($identityUuid);
        }

        return $this->buildMaskedConsentEnvelope($updated, 'promotional', $requestId, $occurredAt);
    }

    /**
     * Revoke promotional consent.
     *
     * Does not affect transactional consent. Only affects promotional consent.
     */
    public function revokePromotionalConsent(array $input): array
    {
        $identityUuid = (string) ($input['identity_uuid'] ?? '');
        $this->assertUuid($identityUuid, 'identity');

        $occurredAt = (string) ($input['occurred_at'] ?? '');
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($occurredAt);

        $requestId = (string) ($input['request_id'] ?? '');
        $idempotencyKey = (string) ($input['idempotency_key'] ?? '');
        $this->assertToken($requestId, 128, 'request_id');
        $this->assertToken($idempotencyKey, 128, 'idempotency_key');

        try {
            $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return $this->maskedFailure('EMAIL_IDENTITY_NOT_FOUND', $requestId);
        }

        $updated = $this->identities->revokePromotionalConsent($identityUuid, $occurredAt);

        return $this->buildMaskedConsentEnvelope($updated, 'promotional_revoked', $requestId, $occurredAt);
    }

    // ── capability checks ──────────────────────────────────────────────

    /**
     * Check whether transactional messages can be sent to an identity.
     *
     * Transactional messages are never gated by promotional consent.
     * Returns true unless the identity has a hard bounce or relevant suppression.
     */
    public function canSendTransactional(string $identityUuid): bool
    {
        $this->assertUuid($identityUuid, 'identity');
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return false;
        }
        // Hard bounce prevents all messages.
        if (($identity['bounce_state'] ?? 'none') === 'hard') {
            return false;
        }
        // Transactional or all suppression prevents transactional messages.
        if (in_array($identity['suppression_state'] ?? 'none', ['transactional', 'all'], true)) {
            return false;
        }
        return true;
    }

    /**
     * Check whether promotional messages can be sent to an identity.
     *
     * Requires promotional consent AND no suppression of promotional messages.
     * Hard bounces suppress all messages.
     */
    public function canSendPromotional(string $identityUuid): bool
    {
        $this->assertUuid($identityUuid, 'identity');
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return false;
        }
        // Hard bounce prevents all messages.
        if (($identity['bounce_state'] ?? 'none') === 'hard') {
            return false;
        }
        // Promotional or all suppression prevents promotional messages.
        if (in_array($identity['suppression_state'] ?? 'none', ['promotional', 'all'], true)) {
            return false;
        }
        // Promotional consent must be settled and not revoked.
        if (($identity['promotional_consent_at'] ?? null) === null) {
            return false;
        }
        if (($identity['promotional_consent_revoked_at'] ?? null) !== null) {
            return false;
        }
        return true;
    }

    /**
     * Check whether a bounced/suppressed identity can be used for verification.
     *
     * Always returns false for hard bounces and suppressed identities.
     * Bounce/suppression cannot become verified identity.
     */
    public function canVerifyIdentity(string $identityUuid): bool
    {
        $this->assertUuid($identityUuid, 'identity');
        try {
            $identity = $this->identities->findByUuid($identityUuid);
        } catch (OutOfBoundsException) {
            return false;
        }
        // Hard bounce prevents verification.
        if (($identity['bounce_state'] ?? 'none') === 'hard') {
            return false;
        }
        // Any suppression prevents verification.
        if (($identity['suppression_state'] ?? 'none') !== 'none') {
            return false;
        }
        // Identity must be verified and in a valid state.
        if (($identity['verified_at'] ?? null) === null) {
            return false;
        }
        if (!in_array($identity['identity_state'] ?? '', ['primary', 'linked'], true)) {
            return false;
        }
        return true;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function maxSuppression(string $current, string $incoming): string
    {
        $order = ['none' => 0, 'transactional' => 1, 'promotional' => 2, 'all' => 3];
        $currentValue = $order[$current] ?? 0;
        $incomingValue = $order[$incoming] ?? 0;
        $merged = $currentValue | $incomingValue;
        foreach ($order as $label => $value) {
            if ($merged === $value) {
                return $label;
            }
        }
        // If both transactional and promotional bits are set, upgrade to all.
        if (($merged & 3) === 3) {
            return 'all';
        }
        return $current;
    }

    private function buildMaskedDeliveryEnvelope(array $identity, string $messageKind, string $deliveryStatus, string $requestId, string $occurredAt): array
    {
        return [
            'schema' => 'focusa.spec152e.masked_delivery_envelope.v1',
            'request_id' => $requestId,
            'identity_id' => $identity['identity_uuid'],
            'message_kind' => $messageKind,
            'delivery_status' => $deliveryStatus,
            'bounce_state' => $identity['bounce_state'],
            'suppression_state' => $identity['suppression_state'],
            'can_send_transactional' => $this->canSendTransactional($identity['identity_uuid']),
            'can_send_promotional' => $this->canSendPromotional($identity['identity_uuid']),
            'can_verify' => $this->canVerifyIdentity($identity['identity_uuid']),
            'occurred_at' => $occurredAt,
        ];
    }

    private function buildMaskedConsentEnvelope(array $identity, string $consentKind, string $requestId, string $occurredAt): array
    {
        return [
            'schema' => 'focusa.spec152e.masked_consent_envelope.v1',
            'request_id' => $requestId,
            'identity_id' => $identity['identity_uuid'],
            'consent_kind' => $consentKind,
            'transactional_consent_at' => $identity['transactional_consent_at'] ?? null,
            'promotional_consent_at' => $identity['promotional_consent_at'] ?? null,
            'promotional_consent_revoked_at' => $identity['promotional_consent_revoked_at'] ?? null,
            'can_send_transactional' => $this->canSendTransactional($identity['identity_uuid']),
            'can_send_promotional' => $this->canSendPromotional($identity['identity_uuid']),
            'can_verify' => $this->canVerifyIdentity($identity['identity_uuid']),
            'occurred_at' => $occurredAt,
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

    private function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
    }

    private function assertToken(string $value, int $maxLength, string $kind): void
    {
        if ($value === '' || strlen($value) > $maxLength || preg_match('/[\r\n]/', $value)) {
            throw new InvalidArgumentException("bounded {$kind} required");
        }
    }
}