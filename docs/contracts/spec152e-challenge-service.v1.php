<?php
// Challenge service: generates single-use magic links and OTP codes, hashes them for
// storage, and validates submitted verifiers. Verifiers are never stored in plaintext.
declare(strict_types=1);

final class FocusaSpec152eChallengeService
{
    public const SCHEMA = 'focusa.spec152e.challenge_service.v1';
    public const OTP_LENGTH = 6;
    public const VERIFIER_TOKEN_BYTES = 32;
    public const MAX_VERIFIER_ATTEMPTS = 5;

    private string $verificationKey;

    public function __construct(string $verificationKey)
    {
        if (strlen($verificationKey) < 32) {
            throw new InvalidArgumentException('independent verification key required');
        }
        $this->verificationKey = $verificationKey;
    }

    /**
     * Generate a single-use magic-link challenge.
     *
     * Returns:
     *   - verifier:        plaintext verifier token (must be sent, never stored)
     *   - verifier_hash:   hash for storage (never reveals the plaintext)
     *   - magic_link:      full branded URL containing the verifier token
     *   - issued_at:       canonical UTC timestamp
     *   - expires_at:      canonical UTC timestamp
     */
    public function generateMagicLink(
        string $facadeId,
        string $verificationPath,
        string $registrationUuid,
        string $origin,
        string $issuedAt,
        string $expiresAt,
    ): array {
        $this->assertFacadeId($facadeId);
        $this->assertPath($verificationPath);
        $this->assertUuid($registrationUuid, 'registration');
        $this->assertOrigin($origin);
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($issuedAt);
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($expiresAt);

        $verifier = self::opaqueToken();
        $verifierHash = $this->hash($verifier);
        $params = http_build_query([
            'registration' => $registrationUuid,
            'token' => $verifier,
        ]);
        $magicLink = $origin . $verificationPath . '?' . $params;

        return [
            'verifier' => $verifier,
            'verifier_hash' => $verifierHash,
            'magic_link' => $magicLink,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
        ];
    }

    /**
     * Generate a single-use OTP code challenge.
     *
     * Returns:
     *   - verifier:        plaintext OTP code (must be sent, never stored)
     *   - verifier_hash:   hash for storage (never reveals the plaintext)
     *   - code:            human-readable OTP code (same as verifier for OTP)
     *   - issued_at:       canonical UTC timestamp
     *   - expires_at:      canonical UTC timestamp
     */
    public function generateOtp(
        string $facadeId,
        string $issuedAt,
        string $expiresAt,
    ): array {
        $this->assertFacadeId($facadeId);
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($issuedAt);
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($expiresAt);

        $code = self::generateOtpCode();
        $verifierHash = $this->hash($code);

        return [
            'verifier' => $code,
            'verifier_hash' => $verifierHash,
            'code' => $code,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
        ];
    }

    /**
     * Validate a submitted verifier against a stored hash.
     * Returns true only when the verifier matches the hash.
     */
    public function validate(string $verifier, string $storedHash): bool
    {
        if ($verifier === '' || strlen($verifier) > 256 || preg_match('/[\r\n]/', $verifier)) {
            return false;
        }
        if (!preg_match('/^[a-f0-9]{64}$/D', $storedHash)) {
            return false;
        }
        return hash_equals($storedHash, $this->hash($verifier));
    }

    /**
     * Hash a verifier for storage. The same key is used for both magic links and OTP codes.
     */
    public function hash(string $verifier): string
    {
        return hash_hmac('sha256', "focusa.spec152e.registration.verification.v1\0" . $verifier, $this->verificationKey);
    }

    /**
     * Determine whether the challenge should be a magic link or an OTP code
     * based on the presenter capabilities in the facade registry.
     */
    public static function challengeKind(array $facade, string $presenter): string
    {
        $capabilities = $facade['presenter_capabilities'] ?? [];
        if (in_array('terminal_continuation', $capabilities, true)
            && in_array($presenter, ['terminal', 'agent_json', 'cli'], true)) {
            return 'otp';
        }
        return 'magic_link';
    }

    // ── private helpers ────────────────────────────────────────────────

    private static function opaqueToken(): string
    {
        return rtrim(strtr(base64_encode(random_bytes(self::VERIFIER_TOKEN_BYTES)), '+/', '-_'), '=');
    }

    private static function generateOtpCode(): string
    {
        return str_pad((string) random_int(0, 10 ** self::OTP_LENGTH - 1), self::OTP_LENGTH, '0', STR_PAD_LEFT);
    }

    private function assertFacadeId(string $facadeId): void
    {
        if ($facadeId === '' || strlen($facadeId) > 96 || preg_match('/[\r\n]/', $facadeId)) {
            throw new InvalidArgumentException('bounded facade ID required');
        }
    }

    private function assertPath(string $path): void
    {
        if ($path === '' || strlen($path) > 256 || !str_starts_with($path, '/') || preg_match('/[\r\n]/', $path)) {
            throw new InvalidArgumentException('bounded verification path required');
        }
    }

    public static function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
    }

    private function assertOrigin(string $origin): void
    {
        if (!preg_match('#^https://[a-z0-9.-]+$#D', $origin)) {
            throw new InvalidArgumentException('exact HTTPS origin required');
        }
    }
}