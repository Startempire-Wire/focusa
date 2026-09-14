<?php
// Spec 172 limited-access assertion service/routes (atom focusa-vbcqu.20.15.11,
// 172.02.02). This is the WPUIAI authority issuer/verifier for the permanent
// verified_no_license posture with bounded credentials.
//
//   issue    - sign a limited-access assertion ONLY from an existing active
//              verified posture (verified account + registered node binding).
//              Sequence and family allowlist are server-owned; the caller never
//              controls product, family, feature, limit, node grant, or sequence.
//   refresh  - rotate the current bounded credential to the next monotonic
//              sequence with a fresh refresh window, WITHOUT imposing any access
//              expiry on the permanent posture and WITHOUT widening the allowlist.
//   revoke   - preservation-only revoke for lost device / account abuse.
//   recover  - issue a replacement posture + assertion from re-verified identity
//              (fresh verified_at, same account, fresh registered node, higher
//              monotonic account sequence). Never widens into paid families and
//              never reuses a revoked node as active.
//   verify   - verifier: signature (Ed25519), stored-row binding, posture status,
//              product scope, node, family allowlist, monotonic sequence, and the
//              bounded credential refresh window. Everything fails closed.
//
// Signatures are real RFC 8032 Ed25519 (SHA-512 + Curve25519) implemented in pure
// PHP over gmp, so the same signed vectors verify with the Python cryptography
// Ed25519 verifier (cross-language fixture). No EDD Software Licensing key, no
// zero-dollar fake license, no anonymous product capability, and no local/self-
// issued grant is ever created by this service.
declare(strict_types=1);

/** RFC 8032 Ed25519 implemented in pure PHP over gmp (SHA-512 + Curve25519). */
final class FocusaSpec172Ed25519
{
    // p = 2^255 - 19
    private const P = '57896044618658097711785492504343953926634992332820282019728792003956564819949';
    // L = order of the base point
    private const L = '7237005577332262213973186563042994240857116359379907606001950938285454250989';
    // d = -121665/121666 mod p
    private const D = '37095705934669439343138083508754565189542113879843219016388785533085940283555';
    // I = sqrt(-1) mod p = 2^((p-1)/4)
    private const I = '19681161376707505956807079304988542015446066515923890162744021073123829784752';
    private const BY = '46316835694926478169428394003475163141307993866256225615783033603165251855960';
    private const BX = '15112221349535400772501151409588531511454012693041857206046113283949847762202';

    /** @return array{public_key: string, secret_key: string} (32-byte public, 64-byte secret) */
    public static function keypair(string $seed): array
    {
        if (strlen($seed) !== 32) {
            throw new InvalidArgumentException('32-byte Ed25519 seed required');
        }
        $h = hash('sha512', $seed, true);
        $a = self::clamp(substr($h, 0, 32));
        $publicKey = self::encodePoint(self::scalarMult($a, self::basePoint()));
        return ['public_key' => $publicKey, 'secret_key' => $h];
    }

    public static function sign(string $message, string $publicKey, string $secretKey): string
    {
        $a = self::clamp(substr($secretKey, 0, 32));
        $r = self::modL(hash('sha512', substr($secretKey, 32, 32) . $message, true));
        $R = self::encodePoint(self::scalarMult($r, self::basePoint()));
        $k = self::modL(hash('sha512', $R . $publicKey . $message, true));
        $S = gmp_mod(gmp_add($r, gmp_mul($k, $a)), self::gmp(self::L));
        return $R . self::gmpToLe($S, 32);
    }

    public static function verify(string $message, string $publicKey, string $signature): bool
    {
        if (strlen($publicKey) !== 32 || strlen($signature) !== 64) {
            return false;
        }
        $A = self::decodePoint($publicKey);
        $R = self::decodePoint(substr($signature, 0, 32));
        if ($A === null || $R === null) {
            return false;
        }
        $S = self::leToGmp(substr($signature, 32, 32));
        if (gmp_cmp($S, self::gmp(self::L)) >= 0) {
            return false;
        }
        $k = self::modL(hash('sha512', substr($signature, 0, 32) . $publicKey . $message, true));
        // Cofactored verification: [8][S]B == [8]R + [8][k]A
        $lhs = self::scalarMult(gmp_mul($S, 8), self::basePoint());
        $rhs = self::pointAdd(self::scalarMult(gmp_init(8), $R), self::scalarMult(gmp_mul($k, 8), $A));
        return hash_equals(self::encodePoint($lhs), self::encodePoint($rhs));
    }

    // ── point arithmetic ─────────────────────────────────────────────────

    /** @return array{GMP, GMP} */
    private static function pointAdd(array $p, array $q): array
    {
        $mod = self::gmp(self::P);
        $d = self::gmp(self::D);
        $x1y2 = gmp_mod(gmp_mul($p[0], $q[1]), $mod);
        $y1x2 = gmp_mod(gmp_mul($p[1], $q[0]), $mod);
        $x1x2 = gmp_mod(gmp_mul($p[0], $q[0]), $mod);
        $y1y2 = gmp_mod(gmp_mul($p[1], $q[1]), $mod);
        $dxxyy = gmp_mod(gmp_mul($d, gmp_mul($x1x2, $y1y2)), $mod);
        $den1 = gmp_mod(gmp_add(1, $dxxyy), $mod);
        $den2 = gmp_mod(gmp_sub(1, $dxxyy), $mod);
        $inv = gmp_invert(gmp_mul($den1, $den2), $mod);
        if ($inv === false) {
            throw new RuntimeException('ed25519 addition denominator not invertible');
        }
        $invDen1 = gmp_mod(gmp_mul($inv, $den2), $mod);
        $invDen2 = gmp_mod(gmp_mul($inv, $den1), $mod);
        $x3 = gmp_mod(gmp_mul(gmp_add($x1y2, $y1x2), $invDen1), $mod);
        // a = -1 twisted Edwards curve: y3 = (y1*y2 + x1*x2) / (1 - d*x1*x2*y1*y2)
        $y3 = gmp_mod(gmp_mul(gmp_add($y1y2, $x1x2), $invDen2), $mod);
        return [$x3, $y3];
    }

    /** @return array{GMP, GMP} */
    private static function scalarMult(GMP $scalar, array $point): array
    {
        $result = [gmp_init(0), gmp_init(1)];
        $addend = $point;
        for ($i = 0; $i < 256; $i++) {
            if (gmp_intval(gmp_mod($scalar, 2)) === 1) {
                $result = self::pointAdd($result, $addend);
            }
            $addend = self::pointAdd($addend, $addend);
            $scalar = gmp_div_q($scalar, 2);
        }
        return $result;
    }

    /** @return array{GMP, GMP} */
    private static function basePoint(): array
    {
        return [self::gmp(self::BX), self::gmp(self::BY)];
    }

    /** @return array{GMP, GMP}|null */
    private static function decodePoint(string $encoded): ?array
    {
        if (strlen($encoded) !== 32) {
            return null;
        }
        $sign = (ord($encoded[31]) >> 7) & 1;
        $bytes = $encoded;
        $bytes[31] = chr(ord($bytes[31]) & 0x7f);
        $y = self::leToGmp($bytes);
        $mod = self::gmp(self::P);
        if (gmp_cmp($y, $mod) >= 0) {
            return null;
        }
        $y2 = gmp_mod(gmp_mul($y, $y), $mod);
        $u = gmp_mod(gmp_sub($y2, 1), $mod);
        $v = gmp_mod(gmp_add(gmp_mul(self::gmp(self::D), $y2), 1), $mod);
        $v2 = gmp_mod(gmp_mul($v, $v), $mod);
        $v3 = gmp_mod(gmp_mul($v2, $v), $mod);
        $v7 = gmp_mod(gmp_mul($v3, gmp_mul($v3, $v)), $mod);
        $x = gmp_mod(gmp_mul(gmp_mul($u, $v3), gmp_powm(gmp_mul($u, $v7), gmp_sub(gmp_pow(2, 252), 3), $mod)), $mod);
        $vx2 = gmp_mod(gmp_mul($v, gmp_mul($x, $x)), $mod);
        if (gmp_cmp($vx2, $u) !== 0) {
            if (gmp_cmp($vx2, gmp_mod(gmp_neg($u), $mod)) !== 0) {
                return null;
            }
            $x = gmp_mod(gmp_mul($x, self::gmp(self::I)), $mod);
        }
        if (gmp_cmp($x, 0) === 0 && $sign === 1) {
            return null;
        }
        if (gmp_intval(gmp_mod($x, 2)) !== $sign) {
            $x = gmp_mod(gmp_neg($x), $mod);
        }
        if (!self::isOnCurve($x, $y)) {
            return null;
        }
        return [$x, $y];
    }

    /** @return array{GMP, GMP} */
    private static function encodePoint(array $point): string
    {
        $bytes = self::gmpToLe($point[1], 32);
        if (gmp_intval(gmp_mod($point[0], 2)) === 1) {
            $bytes[31] = chr(ord($bytes[31]) | 0x80);
        }
        return $bytes;
    }

    private static function isOnCurve(GMP $x, GMP $y): bool
    {
        $mod = self::gmp(self::P);
        $x2 = gmp_mod(gmp_mul($x, $x), $mod);
        $y2 = gmp_mod(gmp_mul($y, $y), $mod);
        $lhs = gmp_mod(gmp_add(gmp_neg($x2), $y2), $mod);
        $rhs = gmp_mod(gmp_add(1, gmp_mul(self::gmp(self::D), gmp_mul($x2, $y2))), $mod);
        return gmp_cmp($lhs, $rhs) === 0;
    }

    // ── scalar helpers ───────────────────────────────────────────────────

    private static function clamp(string $bytes): GMP
    {
        $bytes[0] = chr(ord($bytes[0]) & 248);
        $bytes[31] = chr((ord($bytes[31]) & 127) | 64);
        return self::leToGmp($bytes);
    }

    private static function modL(string $digest): GMP
    {
        return gmp_mod(self::leToGmp($digest), self::gmp(self::L));
    }

    private static function leToGmp(string $bytes): GMP
    {
        $value = gmp_init(0, 10);
        for ($i = strlen($bytes) - 1; $i >= 0; $i--) {
            $value = gmp_add(gmp_mul($value, 256), ord($bytes[$i]));
        }
        return $value;
    }

    private static function gmpToLe(GMP $value, int $length): string
    {
        $out = '';
        for ($i = 0; $i < $length; $i++) {
            $out .= chr(gmp_intval(gmp_mod($value, 256)));
            $value = gmp_div_q($value, 256);
        }
        return $out;
    }

    private static function gmp(string $decimal): GMP
    {
        return gmp_init($decimal, 10);
    }
}

/** Canonical signing payload for a limited-access assertion. */
final class FocusaSpec172LimitedAssertionPayload
{
    public const SCHEMA = 'focusa.spec172.limited_access_assertion.v1';

    /** Build the canonical signed payload from presented fields (signature excluded). */
    public static function build(array $presented): array
    {
        $sequence = filter_var($presented['sequence'] ?? null, FILTER_VALIDATE_INT);
        if ($sequence === false || $sequence < 1) {
            throw new InvalidArgumentException('positive assertion sequence required');
        }
        foreach (['posture_uuid', 'account_uuid', 'identity_uuid', 'product_scope', 'node_uuid', 'issued_at', 'refresh_at', 'signer'] as $field) {
            if (!is_string($presented[$field] ?? null) || $presented[$field] === '') {
                throw new InvalidArgumentException("assertion {$field} required");
            }
        }
        return [
            'schema' => self::SCHEMA,
            'algorithm' => FocusaSpec172LimitedAssertionSigner::ALGORITHM,
            'posture_uuid' => $presented['posture_uuid'],
            'account_uuid' => $presented['account_uuid'],
            'identity_uuid' => $presented['identity_uuid'],
            'product_scope' => $presented['product_scope'],
            'node_uuid' => $presented['node_uuid'],
            'family_allowlist' => self::sortedFamilies($presented['family_allowlist'] ?? null),
            'sequence' => $sequence,
            'issued_at' => $presented['issued_at'],
            'refresh_at' => $presented['refresh_at'],
            'signer' => $presented['signer'],
        ];
    }

    /** Deterministic canonical JSON: sorted keys, no whitespace, no slash escaping. */
    public static function encodeCanonical(array $value): string
    {
        $normalize = static function (mixed $item) use (&$normalize): mixed {
            if (!is_array($item)) {
                return $item;
            }
            if (!array_is_list($item)) {
                ksort($item, SORT_STRING);
            }
            foreach ($item as $key => $child) {
                $item[$key] = $normalize($child);
            }
            return $item;
        };
        return json_encode($normalize($value), JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }

    /** @return list<string> sorted canonical family codes */
    public static function sortedFamilies(mixed $value): array
    {
        if (!is_array($value) || $value === []) {
            throw new InvalidArgumentException('explicit family allowlist required');
        }
        $result = [];
        foreach ($value as $family) {
            if (!is_string($family) || preg_match('/^[a-z][a-z0-9_]{1,63}$/D', $family) !== 1) {
                throw new InvalidArgumentException('registered family code required');
            }
            $result[] = $family;
        }
        sort($result, SORT_STRING);
        return array_values(array_unique($result));
    }
}

/** Server-owned Ed25519 signer for limited-access assertions. */
final class FocusaSpec172LimitedAssertionSigner
{
    public const ALGORITHM = 'ed25519.spec172.v1';

    public function __construct(private string $publicKeyHex, private string $secretKeyHex)
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $publicKeyHex) !== 1) {
            throw new InvalidArgumentException('canonical Ed25519 public key required');
        }
        if (preg_match('/^[0-9a-f]{128}$/D', $secretKeyHex) !== 1) {
            throw new InvalidArgumentException('canonical Ed25519 secret key required');
        }
    }

    public static function fromSeed(string $seedHex): self
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $seedHex) !== 1) {
            throw new InvalidArgumentException('32-byte Ed25519 seed required');
        }
        $keypair = FocusaSpec172Ed25519::keypair(hex2bin($seedHex));
        return new self(bin2hex($keypair['public_key']), bin2hex($keypair['secret_key']));
    }

    public function publicKeyHex(): string
    {
        return $this->publicKeyHex;
    }

    public function sign(array $payload): string
    {
        $message = FocusaSpec172LimitedAssertionPayload::encodeCanonical($payload);
        return bin2hex(FocusaSpec172Ed25519::sign($message, hex2bin($this->publicKeyHex), hex2bin($this->secretKeyHex)));
    }

    public function verify(array $payload, string $signatureHex): bool
    {
        if (preg_match('/^[0-9a-f]{128}$/D', $signatureHex) !== 1) {
            return false;
        }
        $message = FocusaSpec172LimitedAssertionPayload::encodeCanonical($payload);
        return FocusaSpec172Ed25519::verify($message, hex2bin($this->publicKeyHex), hex2bin($signatureHex));
    }
}

/**
 * WPUIAI limited-access assertion issuer/verifier. Issue only from an active
 * verified posture; refresh rotates the bounded credential without access expiry;
 * revoke is preservation-only; recover re-issues from re-verified identity at a
 * higher monotonic account sequence. Verification fails closed on unverified,
 * tampered, stale-sequence, wrong-node, unknown-family, and paid-family claims.
 */
final class FocusaSpec172LimitedAssertionService
{
    public const SIGNER_ISSUE = 'wpuiai.spec172.issue.v1';
    public const SIGNER_REFRESH = 'wpuiai.spec172.refresh.v1';
    public const SIGNER_RECOVERY = 'wpuiai.spec172.recovery.v1';
    public const NODE_LIMIT = 3;

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec172VerifiedAccessPostureRepository $postures,
        private FocusaSpec172SignedAccessAssertionRepository $assertions,
        private FocusaSpec172LimitedAssertionSigner $signer,
        private FocusaSpec172VerifiedAccessPostureMigration $postureSchema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Issue one signed limited-access assertion from an active verified posture.
     * Sequence and allowlist are server-owned; the caller cannot widen families,
     * change product scope, bind another node, or choose the sequence.
     */
    public function issue(array $input): array
    {
        $posture = $this->requireActivePosture((string) ($input['posture_uuid'] ?? ''));
        $latest = $this->assertions->findLatestByPosture((string) $posture['posture_uuid']);
        $sequence = $latest === null ? (int) $posture['sequence'] : (int) $latest['sequence'] + 1;
        $families = json_decode((string) $posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
        $issuedAt = (string) ($input['issued_at'] ?? '');
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($issuedAt);
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($refreshAt);
        $signature = $this->signer->sign(FocusaSpec172LimitedAssertionPayload::build([
            'posture_uuid' => $posture['posture_uuid'],
            'account_uuid' => $posture['account_uuid'],
            'identity_uuid' => $posture['identity_uuid'],
            'product_scope' => $posture['product_scope'],
            'node_uuid' => $posture['node_uuid'],
            'family_allowlist' => $families,
            'sequence' => $sequence,
            'issued_at' => $issuedAt,
            'refresh_at' => $refreshAt,
            'signer' => self::SIGNER_ISSUE,
        ]));
        $row = $this->assertions->recordAssertion([
            'posture_uuid' => $posture['posture_uuid'],
            'product_scope' => $posture['product_scope'],
            'node_uuid' => $posture['node_uuid'],
            'family_allowlist' => $families,
            'sequence' => $sequence,
            'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
            'signature' => $signature,
            'issued_at' => $issuedAt,
            'refresh_at' => $refreshAt,
            'signer' => self::SIGNER_ISSUE,
            'migration_provenance' => $this->provenance($input),
        ]);
        return $this->envelope($row, 'valid');
    }

    /**
     * Rotate the current bounded credential: next monotonic sequence, fresh refresh
     * window, same allowlist. Never imposes an access expiry on the posture and never
     * widens the family allowlist. Idempotent under the caller's idempotency key.
     */
    public function refresh(array $input): array
    {
        $posture = $this->requireActivePosture((string) ($input['posture_uuid'] ?? ''));
        $latest = $this->assertions->findLatestByPosture((string) $posture['posture_uuid']);
        if ($latest === null) {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        $refreshAt = (string) ($input['refresh_at'] ?? '');
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($refreshAt);
        $families = json_decode((string) $posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
        $nextSequence = (int) $latest['sequence'] + 1;
        $signature = $this->signer->sign(FocusaSpec172LimitedAssertionPayload::build([
            'posture_uuid' => $posture['posture_uuid'],
            'account_uuid' => $posture['account_uuid'],
            'identity_uuid' => $posture['identity_uuid'],
            'product_scope' => $posture['product_scope'],
            'node_uuid' => $posture['node_uuid'],
            'family_allowlist' => $families,
            'sequence' => $nextSequence,
            'issued_at' => (string) $latest['issued_at'],
            'refresh_at' => $refreshAt,
            'signer' => self::SIGNER_REFRESH,
        ]));
        $row = $this->assertions->refreshAssertion([
            'posture_uuid' => $posture['posture_uuid'],
            'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
            'signature' => $signature,
            'refresh_at' => $refreshAt,
            'idempotency_key' => (string) ($input['idempotency_key'] ?? ''),
            'migration_provenance' => $this->provenance($input),
        ]);
        return $this->envelope($row, 'valid');
    }

    /** Preservation-only revoke for lost device / account abuse; rows are never deleted. */
    public function revoke(array $input): array
    {
        $reason = (string) ($input['reason'] ?? '');
        if (!in_array($reason, ['lost_device', 'account_abuse', 'compromised', 'operator_request'], true)) {
            throw new InvalidArgumentException('bounded revoke reason required');
        }
        $occurredAt = (string) ($input['occurred_at'] ?? '');
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($occurredAt);
        $row = $this->assertions->revokeAssertion(
            (string) ($input['posture_uuid'] ?? ''),
            $reason,
            $occurredAt,
            $this->provenance($input),
        );
        return $this->envelope($row, 'revoked');
    }

    /**
     * Recover from verified identity: the account must already have a verified
     * posture for the product scope, that posture must be revoked/superseded, the
     * recovery proof must be a fresh canonical verified_at after the account's
     * latest posture change, and the node must be a fresh registration (max three
     * per account). The recovered posture starts at a strictly higher monotonic
     * account sequence with the same canonical limited-mode allowlist; it never
     * widens into paid families and never creates an EDD key.
     */
    public function recover(array $input): array
    {
        $accountUuid = $this->assertUuid((string) ($input['account_uuid'] ?? ''), 'account');
        $productScope = (string) ($input['product_scope'] ?? '');
        if (!in_array($productScope, FocusaSpec172VerifiedAccessPostureState::PRODUCT_SCOPES, true)) {
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        $verifiedAt = (string) ($input['recovery_verified_at'] ?? '');
        FocusaSpec172VerifiedAccessPostureMigration::assertTimestamp($verifiedAt);
        $nodeUuid = $this->assertNodeUuid((string) ($input['node_uuid'] ?? ''));
        $nodeDigest = $this->assertNodeDigest((string) ($input['node_digest'] ?? ''));
        $provenance = $this->provenance($input);

        $accountPostures = $this->posturesForAccount($accountUuid);
        if ($accountPostures === []) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        $latestForScope = null;
        foreach ($accountPostures as $candidate) {
            if ((string) $candidate['product_scope'] === $productScope
                && ($latestForScope === null || (int) $candidate['sequence'] > (int) $latestForScope['sequence'])) {
                $latestForScope = $candidate;
            }
        }
        if ($latestForScope === null) {
            // The account never held this product scope: recovery must not widen.
            throw new DomainException('PRODUCT_NOT_INCLUDED');
        }
        if (!in_array((string) $latestForScope['status'], ['revoked', 'superseded'], true)) {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        $latestAccountChange = (string) $accountPostures[0]['updated_at'];
        foreach ($accountPostures as $candidate) {
            if ((string) $candidate['updated_at'] > $latestAccountChange) {
                $latestAccountChange = (string) $candidate['updated_at'];
            }
        }
        if ($verifiedAt <= $latestAccountChange) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if ($this->isNodeRegisteredForAccount($accountUuid, $nodeUuid)) {
            throw new DomainException('NODE_LIMIT_REACHED');
        }
        if ($this->countRegisteredNodes($accountUuid) >= self::NODE_LIMIT) {
            throw new DomainException('NODE_LIMIT_REACHED');
        }
        $nextSequence = (int) $latestForScope['sequence'] + 1;
        foreach ($accountPostures as $candidate) {
            if ((int) $candidate['sequence'] >= $nextSequence) {
                $nextSequence = (int) $candidate['sequence'] + 1;
            }
        }

        $newPosture = $this->postures->recordPosture([
            'account_uuid' => $accountUuid,
            'identity_uuid' => (string) $latestForScope['identity_uuid'],
            'registration_uuid' => (string) $latestForScope['registration_uuid'],
            'verification_state' => 'account_promoted',
            'verified_at' => $verifiedAt,
            'product_scope' => $productScope,
            'node_uuid' => $nodeUuid,
            'node_digest' => $nodeDigest,
            'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor($productScope),
            'signer' => self::SIGNER_RECOVERY,
            'sequence' => $nextSequence,
            'issued_at' => $verifiedAt,
            'refresh_at' => $verifiedAt,
            'migration_provenance' => $provenance,
        ]);
        $signature = $this->signer->sign(FocusaSpec172LimitedAssertionPayload::build([
            'posture_uuid' => $newPosture['posture_uuid'],
            'account_uuid' => $accountUuid,
            'identity_uuid' => (string) $newPosture['identity_uuid'],
            'product_scope' => $productScope,
            'node_uuid' => $nodeUuid,
            'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor($productScope),
            'sequence' => $nextSequence,
            'issued_at' => $verifiedAt,
            'refresh_at' => $verifiedAt,
            'signer' => self::SIGNER_RECOVERY,
        ]));
        $row = $this->assertions->recordAssertion([
            'posture_uuid' => $newPosture['posture_uuid'],
            'product_scope' => $productScope,
            'node_uuid' => $nodeUuid,
            'family_allowlist' => FocusaSpec172VerifiedAccessPostureState::allowlistFor($productScope),
            'sequence' => $nextSequence,
            'signature_algorithm' => FocusaSpec172SignedAccessAssertionRepository::SIGNATURE_ALGORITHM,
            'signature' => $signature,
            'issued_at' => $verifiedAt,
            'refresh_at' => $verifiedAt,
            'signer' => self::SIGNER_RECOVERY,
            'migration_provenance' => $provenance,
        ]);
        return $this->envelope($row, 'valid');
    }

    /**
     * Verify a presented assertion. Signature first, then stored-row binding, then
     * posture status/product/node/family/sequence, then the bounded credential
     * window. Fails closed: SIGNATURE_INVALID, ASSERTION_UNKNOWN, ASSERTION_TAMPERED,
     * EMAIL_VERIFICATION_REQUIRED, VERIFIED_LIMITED_ACCESS, ENTITLEMENT_PRODUCT_MISMATCH,
     * NODE_LIMIT_REACHED, CAPABILITY_FAMILY_NOT_INCLUDED, ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED,
     * CREDENTIAL_REFRESH_REQUIRED.
     */
    public function verify(array $presented, ?string $at = null): array
    {
        try {
            $payload = FocusaSpec172LimitedAssertionPayload::build($presented);
        } catch (InvalidArgumentException $error) {
            return $this->denied('ASSERTION_UNKNOWN');
        }
        if (!$this->signer->verify($payload, (string) ($presented['signature'] ?? ''))) {
            return $this->denied('SIGNATURE_INVALID');
        }
        $sequence = filter_var($presented['sequence'] ?? null, FILTER_VALIDATE_INT);
        if ($sequence === false || $sequence < 1) {
            return $this->denied('ASSERTION_UNKNOWN');
        }
        $row = $this->assertions->findByPostureSequence((string) $payload['posture_uuid'], $sequence);
        if ($row === null) {
            return $this->denied('ASSERTION_UNKNOWN');
        }
        if (!in_array((string) $row['status'], ['issued', 'refreshed'], true)) {
            return $this->denied('VERIFIED_LIMITED_ACCESS');
        }
        if ($this->bindingMismatch($row, $presented)) {
            return $this->denied('ASSERTION_TAMPERED');
        }
        $posture = $this->postures->findByUuid((string) $payload['posture_uuid']);
        if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
            return $this->denied('VERIFIED_LIMITED_ACCESS');
        }
        $verdict = self::policyVerdict($presented, $posture, $at);
        if ($verdict !== 'valid') {
            return $this->denied($verdict);
        }
        return $this->envelope($row, 'valid');
    }

    /** Bounded posture/credential status for the status route. */
    public function status(string $postureUuid): array
    {
        $this->assertUuid($postureUuid, 'posture');
        $posture = $this->postures->findByUuid($postureUuid);
        $latest = $this->assertions->findLatestByPosture($postureUuid);
        return [
            'verdict' => 'status',
            'posture_uuid' => $postureUuid,
            'product_scope' => $posture['product_scope'],
            'status' => $posture['status'],
            'status_reason' => $posture['status_reason'],
            'sequence' => (int) $posture['sequence'],
            'refresh_at' => $posture['refresh_at'],
            'latest_assertion_sequence' => $latest === null ? null : (int) $latest['sequence'],
            'latest_assertion_status' => $latest === null ? null : $latest['status'],
        ];
    }

    // ── policy verdict (shared with the client store) ────────────────────

    /** Fail-closed policy evaluation of a presented claim against authoritative posture state. */
    public static function policyVerdict(array $presented, array $posture, ?string $at): string
    {
        if (!hash_equals((string) $posture['product_scope'], (string) $presented['product_scope'])) {
            return 'ENTITLEMENT_PRODUCT_MISMATCH';
        }
        if (!hash_equals((string) $posture['node_uuid'], (string) $presented['node_uuid'])) {
            return 'NODE_LIMIT_REACHED';
        }
        if (!hash_equals((string) $posture['account_uuid'], (string) $presented['account_uuid'])) {
            return 'ASSERTION_TAMPERED';
        }
        $postureAllowlist = is_array($posture['family_allowlist'])
            ? $posture['family_allowlist']
            : json_decode((string) $posture['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
        $families = FocusaSpec172LimitedAssertionPayload::sortedFamilies($presented['family_allowlist'] ?? null);
        foreach ($families as $family) {
            if (!in_array($family, $postureAllowlist, true)
                || !FocusaSpec172VerifiedAccessPostureState::isRegisteredFamily((string) $posture['product_scope'], $family)) {
                return 'CAPABILITY_FAMILY_NOT_INCLUDED';
            }
        }
        if ((int) $presented['sequence'] !== (int) $posture['sequence']) {
            return 'ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED';
        }
        if ((string) $presented['issued_at'] > (string) $presented['refresh_at']) {
            return 'ASSERTION_TAMPERED';
        }
        if ($at !== null && $at > (string) $presented['refresh_at']) {
            return 'CREDENTIAL_REFRESH_REQUIRED';
        }
        return 'valid';
    }

    // ── helpers ──────────────────────────────────────────────────────────

    private function requireActivePosture(string $postureUuid): array
    {
        $this->assertUuid($postureUuid, 'posture');
        try {
            $posture = $this->postures->findByUuid($postureUuid);
        } catch (OutOfBoundsException $error) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        return $posture;
    }

    private function bindingMismatch(array $row, array $presented): bool
    {
        foreach (['product_scope', 'node_uuid', 'signer', 'account_uuid', 'identity_uuid'] as $field) {
            if (!hash_equals((string) $row[$field], (string) ($presented[$field] ?? ''))) {
                return true;
            }
        }
        $rowFamilies = json_decode((string) $row['family_allowlist'], true, 512, JSON_THROW_ON_ERROR);
        if ($rowFamilies !== FocusaSpec172LimitedAssertionPayload::sortedFamilies($presented['family_allowlist'] ?? null)) {
            return true;
        }
        return (int) $row['sequence'] !== (int) ($presented['sequence'] ?? -1);
    }

    private function posturesForAccount(string $accountUuid): array
    {
        $table = $this->postureSchema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE account_uuid = :account ORDER BY sequence ASC");
        $statement->execute([':account' => $accountUuid]);
        return $statement->fetchAll(PDO::FETCH_ASSOC);
    }

    private function isNodeRegisteredForAccount(string $accountUuid, string $nodeUuid): bool
    {
        $table = $this->postureSchema->table('wpuiai_verified_access_postures');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table}
            WHERE account_uuid = :account AND node_uuid = :node");
        $statement->execute([':account' => $accountUuid, ':node' => $nodeUuid]);
        return (int) $statement->fetchColumn() > 0;
    }

    private function countRegisteredNodes(string $accountUuid): int
    {
        $table = $this->postureSchema->table('wpuiai_verified_access_nodes');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE account_uuid = :account");
        $statement->execute([':account' => $accountUuid]);
        return (int) $statement->fetchColumn();
    }

    private function envelope(array $row, string $verdict): array
    {
        return [
            'verdict' => $verdict,
            'schema' => FocusaSpec172LimitedAssertionPayload::SCHEMA,
            'assertion_uuid' => $row['assertion_uuid'],
            'posture_uuid' => $row['posture_uuid'],
            'product_scope' => $row['product_scope'],
            'node_uuid' => $row['node_uuid'],
            'family_allowlist' => json_decode((string) $row['family_allowlist'], true, 512, JSON_THROW_ON_ERROR),
            'sequence' => (int) $row['sequence'],
            'issued_at' => $row['issued_at'],
            'refresh_at' => $row['refresh_at'],
            'signer' => $row['signer'],
            'status' => $row['status'],
            'signature' => $row['signature'],
        ];
    }

    private function denied(string $code): array
    {
        return ['verdict' => 'denied', 'code' => $code];
    }

    private function provenance(array $input): array
    {
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        return $provenance;
    }

    private function assertUuid(string $uuid, string $kind): string
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
        return $uuid;
    }

    private function assertNodeUuid(string $nodeUuid): string
    {
        if (preg_match('/^[A-Za-z0-9._:-]{8,64}$/D', $nodeUuid) !== 1) {
            throw new InvalidArgumentException('bounded opaque node identifier required');
        }
        return $nodeUuid;
    }

    private function assertNodeDigest(string $nodeDigest): string
    {
        if (preg_match('/^[0-9a-f]{64}$/D', $nodeDigest) !== 1) {
            throw new InvalidArgumentException('canonical node digest required');
        }
        return $nodeDigest;
    }
}

/** WPUIAI limited-access assertion route table (authority kernel routes). */
final class FocusaSpec172LimitedAssertionRoutes
{
    public const SCHEMA = 'focusa.spec172.limited_assertion_routes.v1';

    /** @var array<string, array{method: string, path: string, action: string}> */
    private const ROUTES = [
        'issue' => ['method' => 'POST', 'path' => '/wpuiai/v1/spec172/assertions/issue', 'action' => 'issue'],
        'refresh' => ['method' => 'POST', 'path' => '/wpuiai/v1/spec172/assertions/refresh', 'action' => 'refresh'],
        'revoke' => ['method' => 'POST', 'path' => '/wpuiai/v1/spec172/assertions/revoke', 'action' => 'revoke'],
        'recover' => ['method' => 'POST', 'path' => '/wpuiai/v1/spec172/assertions/recover', 'action' => 'recover'],
        'verify' => ['method' => 'POST', 'path' => '/wpuiai/v1/spec172/assertions/verify', 'action' => 'verify'],
        'status' => ['method' => 'GET', 'path' => '/wpuiai/v1/spec172/assertions/status', 'action' => 'status'],
    ];

    public function __construct(private FocusaSpec172LimitedAssertionService $service)
    {
    }

    public function route(string $method, string $path, array $input): array
    {
        $route = null;
        foreach (self::ROUTES as $candidate) {
            if ($candidate['path'] === $path) {
                $route = $candidate;
                break;
            }
        }
        if ($route === null || $route['method'] !== $method) {
            return ['verdict' => 'denied', 'code' => 'ROUTE_NOT_FOUND', 'schema' => self::SCHEMA];
        }
        try {
            return match ($route['action']) {
                'issue' => $this->service->issue($input),
                'refresh' => $this->service->refresh($input),
                'revoke' => $this->service->revoke($input),
                'recover' => $this->service->recover($input),
                'verify' => $this->service->verify($input, isset($input['at']) ? (string) $input['at'] : null),
                'status' => $this->service->status((string) ($input['posture_uuid'] ?? '')),
                default => ['verdict' => 'denied', 'code' => 'ROUTE_NOT_FOUND', 'schema' => self::SCHEMA],
            };
        } catch (DomainException $error) {
            return ['verdict' => 'denied', 'code' => $error->getMessage(), 'schema' => self::SCHEMA];
        } catch (InvalidArgumentException | OutOfBoundsException $error) {
            return ['verdict' => 'denied', 'code' => 'INVALID_REQUEST', 'schema' => self::SCHEMA];
        }
    }
}
