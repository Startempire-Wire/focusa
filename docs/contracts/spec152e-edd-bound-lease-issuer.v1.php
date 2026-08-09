<?php
// Candidate-owned canonical EDD-bound signed lease issuer/signer seam (spec 152E
// §7.5 wp_wpuiai_authority_leases, §10 POST /v1/lease/refresh, §11 "signed lease
// issues", §12 "issue signed Evaluation lease", §15 human key and signed lease
// separation, §17 lifecycle sequence increments, §18 refund/revoke/expiry →
// sequence increment → refresh denied, §19 security/privacy, §20 stable failure
// semantics, §23 acceptance matrix). It does not bootstrap WordPress.
//
// A signed lease issues ONLY after: a verified authority account (mailbox
// control proven), a usable canonical EDD license (active, unexpired, customer-
// matched, positive activation limit), a settled EDD order/order-item with the
// exact server-owned price, the server-owned product grant (never caller-
// supplied), and a settled node bound to the account/license/product/device
// public key. The payload carries account/customer/order/item/license/product/
// features/limits/commercial/node/sequence/time/kid claims and is signed with
// pure-PHP RFC 8032 Ed25519 (GMP), byte-compatible with libsodium and the
// existing Rust authority-lease verifier (ed25519-dalek, domains
// FOCUSA-AUTHORITY-LEASE-V1\0 / FOCUSA-AUTHORITY-KEY-SET-V1\0).
//
// Spec 158 implementation is excluded. No unverified-email promotion, local or
// self-issued entitlement, independent facade authority, client-controlled EDD
// price/grants, secret or unmasked real-email evidence, or publication.
declare(strict_types=1);

final class FocusaSpec152eEd25519Signer
{
    public const KEY_BYTES = 32;
    public const SIGNATURE_BYTES = 64;
    /** Domain separation for signed authority leases (matches the Rust verifier). */
    public const LEASE_DOMAIN = "FOCUSA-AUTHORITY-LEASE-V1\0";
    /** Domain separation for the signed authority key set (matches the Rust verifier). */
    public const KEY_SET_DOMAIN = "FOCUSA-AUTHORITY-KEY-SET-V1\0";

    private static function p(): GMP { return gmp_sub(gmp_pow(2, 255), 19); }
    private static function l(): GMP { return gmp_add(gmp_pow(2, 252), gmp_init('27742317777372353535851937790883648493')); }
    private static function d(): GMP { return gmp_mod(gmp_mul(gmp_init(-121665), gmp_invert(121666, self::p())), self::p()); }
    private static function sqrtMinusOne(): GMP { return gmp_powm(gmp_init(2), gmp_div_q(gmp_sub(self::p(), 1), 4), self::p()); }
    private static function basePoint(): array {
        return [
            gmp_init('15112221349535400772501151409588531511454012693041857206046113283949847762202'),
            gmp_init('46316835694926478169428394003475163141307993866256225615783033603165251855960'),
        ];
    }

    /** Little-endian bytes -> integer. */
    private static function fromLittleEndian(string $bytes): GMP { return gmp_import($bytes, 1, GMP_LSW_FIRST); }

    /** Integer -> fixed-length little-endian bytes. */
    private static function toLittleEndian(GMP $value, int $length): string
    {
        $out = gmp_export($value, 1, GMP_LSW_FIRST);
        if (strlen($out) > $length) {
            throw new DomainException('ED25519_SCALAR_TOO_LARGE');
        }
        return str_pad($out, $length, "\x00", STR_PAD_RIGHT);
    }

    private static function modSqrt(GMP $a): GMP
    {
        $p = self::p();
        $x = gmp_powm($a, gmp_div_q(gmp_add($p, 3), 8), $p);
        if (gmp_cmp(gmp_mod(gmp_mul($x, $x), $p), gmp_mod($a, $p)) !== 0) {
            $x = gmp_mod(gmp_mul($x, self::sqrtMinusOne()), $p);
        }
        return $x;
    }

    private static function addPoint(array $p1, array $p2): array
    {
        $p = self::p();
        [$x1, $y1] = $p1;
        [$x2, $y2] = $p2;
        $a = gmp_mod(gmp_mul($y1, $x2), $p);
        $b = gmp_mod(gmp_mul($x1, $y2), $p);
        $c = gmp_mod(gmp_mul($x1, $x2), $p);
        $d = gmp_mod(gmp_mul($y1, $y2), $p);
        $e = gmp_mod(gmp_mul(self::d(), gmp_mul($c, $d)), $p);
        $denX = gmp_invert(gmp_add(1, $e), $p);
        $denY = gmp_invert(gmp_sub(1, $e), $p);
        if ($denX === false || $denY === false) {
            throw new DomainException('ED25519_POINT_DENIED');
        }
        $x3 = gmp_mod(gmp_mul(gmp_add($a, $b), $denX), $p);
        $y3 = gmp_mod(gmp_mul(gmp_add($d, $c), $denY), $p);
        return [$x3, $y3];
    }

    private static function scalarMult(GMP $scalar, array $base): array
    {
        $result = [gmp_init(0), gmp_init(1)];
        $addend = $base;
        $bits = gmp_strval($scalar, 2);
        for ($i = strlen($bits) - 1; $i >= 0; $i--) {
            if ($bits[$i] === '1') {
                $result = self::addPoint($result, $addend);
            }
            $addend = self::addPoint($addend, $addend);
        }
        return $result;
    }

    private static function encodePoint(array $point): string
    {
        [$x, $y] = $point;
        $bytes = self::toLittleEndian(gmp_mod($y, self::p()), 32);
        $last = ord($bytes[31]);
        if (gmp_cmp(gmp_and($x, gmp_init(1)), 0) !== 0) {
            $last |= 0x80;
        }
        $bytes[31] = chr($last);
        return $bytes;
    }

    private static function decodePoint(string $bytes): array
    {
        if (strlen($bytes) !== 32) {
            throw new DomainException('ED25519_POINT_LENGTH');
        }
        $last = ord($bytes[31]);
        $sign = ($last & 0x80) !== 0;
        $bytes[31] = chr($last & 0x7f);
        $y = self::fromLittleEndian($bytes);
        $p = self::p();
        $y2 = gmp_mod(gmp_mul($y, $y), $p);
        $u = gmp_mod(gmp_sub($y2, 1), $p);
        $v = gmp_mod(gmp_add(gmp_mul(self::d(), $y2), 1), $p);
        $vInv = gmp_invert($v, $p);
        if ($vInv === false) {
            throw new DomainException('ED25519_POINT_DENIED');
        }
        $x = self::modSqrt(gmp_mod(gmp_mul($u, $vInv), $p));
        if (gmp_cmp(gmp_and($x, gmp_init(1)), 0) !== 0) {
            $x = gmp_mod(gmp_neg($x), $p);
        }
        if ($sign && gmp_cmp(gmp_and($x, gmp_init(1)), 0) === 0) {
            $x = gmp_mod(gmp_neg($x), $p);
        }
        return [$x, $y];
    }

    private static function clamp(string $bytes32): GMP
    {
        $bytes = $bytes32;
        $bytes[0] = chr(ord($bytes[0]) & 248);
        $bytes[31] = chr((ord($bytes[31]) & 127) | 64);
        return self::fromLittleEndian($bytes);
    }

    /** RFC 8032 public key (32 raw bytes) for a 32-byte seed. */
    public static function publicKeyFromSeed(string $seed32): string
    {
        self::assertSeed($seed32);
        $h = hash('sha512', $seed32, true);
        return self::encodePoint(self::scalarMult(self::clamp(substr($h, 0, 32)), self::basePoint()));
    }

    /** RFC 8032 Ed25519 signature over domain-separated payload bytes (deterministic). */
    public static function sign(string $seed32, string $domain, string $payloadBytes): string
    {
        self::assertSeed($seed32);
        $h = hash('sha512', $seed32, true);
        $a = self::clamp(substr($h, 0, 32));
        $prefix = substr($h, 32, 32);
        $A = self::decodePoint(self::publicKeyFromSeed($seed32));
        $r = gmp_mod(self::fromLittleEndian(hash('sha512', $prefix . $domain . $payloadBytes, true)), self::l());
        $R = self::scalarMult($r, self::basePoint());
        $k = gmp_mod(self::fromLittleEndian(hash('sha512', self::encodePoint($R) . self::encodePoint($A) . $domain . $payloadBytes, true)), self::l());
        $S = gmp_mod(gmp_add($r, gmp_mul($k, $a)), self::l());
        return self::encodePoint($R) . self::toLittleEndian($S, 32);
    }

    /** RFC 8032 Ed25519 verification; fails closed on any malformed input. */
    public static function verify(string $publicKey32, string $signature64, string $domain, string $payloadBytes): bool
    {
        if (strlen($publicKey32) !== 32 || strlen($signature64) !== 64) {
            return false;
        }
        try {
            $A = self::decodePoint($publicKey32);
            $R = self::decodePoint(substr($signature64, 0, 32));
            $S = self::fromLittleEndian(substr($signature64, 32, 32));
            if (gmp_cmp($S, self::l()) >= 0) {
                return false;
            }
            $k = gmp_mod(self::fromLittleEndian(hash('sha512', substr($signature64, 0, 32) . $publicKey32 . $domain . $payloadBytes, true)), self::l());
            $lhs = self::scalarMult($S, self::basePoint());
            $rhs = self::addPoint($R, self::scalarMult($k, $A));
            return self::encodePoint($lhs) === self::encodePoint($rhs);
        } catch (Throwable $error) {
            return false;
        }
    }

    private static function assertSeed(string $seed32): void
    {
        if (strlen($seed32) !== 32) {
            throw new InvalidArgumentException('32-byte ed25519 seed required');
        }
    }
}

/**
 * Authority key set seam: the root key signs the key set containing one active
 * lease key, in the exact `focusa.authority_key_set.v1` shape the existing
 * verifier consumes (schema, sequence, issued_at, expires_at, keys with key_id /
 * public_key_b64 / status / not_before / not_after). Keys are injected via
 * constructor (production loads server-side secrets; fixtures use public
 * synthetic seeds). Envelopes are `focusa.signed_envelope.v1` with base64
 * canonical payload bytes and base64 signature bytes.
 */
final class FocusaSpec152eAuthorityKeySetSeam
{
    public const KEY_SET_SCHEMA = 'focusa.authority_key_set.v1';
    public const ENVELOPE_SCHEMA = 'focusa.signed_envelope.v1';
    public const ROOT_KEY_ID = 'authority-root-2026-01';
    public const LEASE_KEY_ID = 'authority-lease-2026-01';
    public const KEY_SET_SEQUENCE = 7;

    /** @var Closure(): string */
    private Closure $clock;
    private string $rootSeed32;
    private string $leaseSeed32;

    public function __construct(string $rootSeed32, string $leaseSeed32, callable $clock)
    {
        $this->assertKeyBytes($rootSeed32, 'root');
        $this->assertKeyBytes($leaseSeed32, 'lease');
        $this->rootSeed32 = $rootSeed32;
        $this->leaseSeed32 = $leaseSeed32;
        $this->clock = Closure::fromCallable($clock);
    }

    public function rootKeyId(): string { return self::ROOT_KEY_ID; }
    public function leaseKeyId(): string { return self::LEASE_KEY_ID; }
    public function leaseSeed(): string { return $this->leaseSeed32; }
    public function rootPublicKeyB64(): string { return base64_encode(FocusaSpec152eEd25519Signer::publicKeyFromSeed($this->rootSeed32)); }
    public function leasePublicKeyB64(): string { return base64_encode(FocusaSpec152eEd25519Signer::publicKeyFromSeed($this->leaseSeed32)); }

    /**
     * Build the root-signed key-set envelope. Deterministic for fixed seeds and
     * clock; golden vectors pin the exact bytes.
     */
    public function keySetEnvelope(string $issuedAt, string $expiresAt, string $notBefore, string $notAfter): array
    {
        FocusaSpec152eEddBoundLeaseIssuer::assertTimestamp($issuedAt);
        FocusaSpec152eEddBoundLeaseIssuer::assertTimestamp($expiresAt);
        FocusaSpec152eEddBoundLeaseIssuer::assertTimestamp($notBefore);
        FocusaSpec152eEddBoundLeaseIssuer::assertTimestamp($notAfter);
        $payload = [
            'schema' => self::KEY_SET_SCHEMA,
            'sequence' => self::KEY_SET_SEQUENCE,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
            'keys' => [[
                'key_id' => self::LEASE_KEY_ID,
                'public_key_b64' => $this->leasePublicKeyB64(),
                'status' => 'active',
                'not_before' => $notBefore,
                'not_after' => $notAfter,
            ]],
        ];
        return $this->seal($payload, self::ROOT_KEY_ID, $this->rootSeed32, FocusaSpec152eEd25519Signer::KEY_SET_DOMAIN);
    }

    /** Seal any canonical payload into a signed envelope. */
    public function seal(array $payload, string $keyId, string $seed32, string $domain): array
    {
        $payloadBytes = FocusaSpec152eEddBoundLeaseIssuer::canonicalJson($payload);
        $signature = FocusaSpec152eEd25519Signer::sign($seed32, $domain, $payloadBytes);
        return [
            'schema' => self::ENVELOPE_SCHEMA,
            'signer_key_id' => $keyId,
            'payload_b64' => base64_encode($payloadBytes),
            'signature_b64' => base64_encode($signature),
        ];
    }

    /** Decode a signed envelope's payload bytes (base64). */
    public static function decodePayload(string $payloadB64): string
    {
        $bytes = base64_decode($payloadB64, true);
        if ($bytes === false) {
            throw new DomainException('INVALID_BASE64');
        }
        return $bytes;
    }

    public static function decodeJson(string $payloadB64): array
    {
        $decoded = json_decode(self::decodePayload($payloadB64), true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($decoded)) {
            throw new DomainException('INVALID_PAYLOAD');
        }
        return $decoded;
    }

    private static function assertKeyBytes(string $seed, string $kind): void
    {
        if (strlen($seed) !== 32) {
            throw new InvalidArgumentException("32-byte {$kind} seed required");
        }
    }
}

/**
 * EDD truth adapters: resolve canonical EDD/authority rows only. Callers submit
 * opaque account, product code, node, device key, and idempotency fields; every
 * commercial, grant, limit, price, and feature value comes from the server-owned
 * registry or the canonical EDD rows — never from the caller. Every adapter fails
 * closed with a stable DomainException code.
 */
final class FocusaSpec152eEddAccountAdapter
{
    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        $this->db = $db;
        $this->prefix = $prefix;
    }

    /**
     * Resolve one verified authority account (status active, reason
     * mailbox_verified / account_promoted). Returns the account row.
     */
    public function resolve(string $accountUuid): array
    {
        FocusaSpec152eEddBoundLeaseIssuer::assertUuid($accountUuid, 'account');
        $statement = $this->db->prepare(
            "SELECT account_uuid, customer_id, status, status_reason, highest_entitlement_sequence
             FROM {$this->prefix}wpuiai_authority_accounts WHERE account_uuid = :uuid"
        );
        $statement->execute([':uuid' => $accountUuid]);
        $account = $statement->fetch(PDO::FETCH_ASSOC);
        if ($account === false) {
            throw new DomainException('ACCOUNT_NOT_FOUND');
        }
        if (($account['status'] ?? '') !== 'active'
            || !in_array($account['status_reason'] ?? '', ['mailbox_verified', 'account_promoted'], true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (filter_var($account['customer_id'], FILTER_VALIDATE_INT) === false
            || (int) $account['customer_id'] < 1) {
            throw new DomainException('ACCOUNT_NOT_FOUND');
        }
        return $account;
    }
}

final class FocusaSpec152eEddLicenseAdapter
{
    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        $this->db = $db;
        $this->prefix = $prefix;
    }

    /**
     * Resolve one usable canonical EDD license (status active, unexpired,
     * positive activation limit, matching customer). Returns the license row.
     */
    public function resolve(int $licenseId, int $customerId, string $now): array
    {
        if ($licenseId < 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $statement = $this->db->prepare(
            "SELECT license_id, customer_id, download_id, payment_id, license_key, status,
                    activation_limit, expiration, date_created
             FROM {$this->prefix}edd_licenses WHERE license_id = :id"
        );
        $statement->execute([':id' => $licenseId]);
        $license = $statement->fetch(PDO::FETCH_ASSOC);
        if ($license === false) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        if ((int) $license['customer_id'] !== $customerId) {
            throw new DomainException('LICENSE_ACCOUNT_MISMATCH');
        }
        if (($license['status'] ?? '') !== 'active') {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        if (filter_var($license['activation_limit'], FILTER_VALIDATE_INT) === false
            || (int) $license['activation_limit'] < 1) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        $expiration = $license['expiration'];
        if ($expiration !== null && $expiration !== '' && $expiration < $now) {
            throw new DomainException('EDD_LICENSE_UNUSABLE');
        }
        return $license;
    }
}

final class FocusaSpec152eEddOrderAdapter
{
    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        $this->db = $db;
        $this->prefix = $prefix;
    }

    /**
     * Resolve the settled order and exact order item backing the license:
     * complete order for the account customer, item matching the license download
     * and the exact server-owned price (spec 152E §11 "exact order item and price
     * relationship").
     */
    public function resolve(array $license, int $customerId, string $expectedPrice): array
    {
        $orderId = filter_var($license['payment_id'] ?? null, FILTER_VALIDATE_INT);
        $downloadId = filter_var($license['download_id'] ?? null, FILTER_VALIDATE_INT);
        if ($orderId === false || $downloadId === false) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $statement = $this->db->prepare(
            "SELECT order_id, customer_id, status, total FROM {$this->prefix}edd_orders WHERE order_id = :id"
        );
        $statement->execute([':id' => $orderId]);
        $order = $statement->fetch(PDO::FETCH_ASSOC);
        if ($order === false || (int) $order['customer_id'] !== $customerId) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        if (($order['status'] ?? '') !== 'complete') {
            throw new DomainException('EDD_ORDER_PENDING');
        }
        $items = $this->db->prepare(
            "SELECT order_item_id, order_id, product_id, price_id, quantity, subtotal, total
             FROM {$this->prefix}edd_order_items
             WHERE order_id = :order AND product_id = :product"
        );
        $items->execute([':order' => $orderId, ':product' => $downloadId]);
        $item = $items->fetch(PDO::FETCH_ASSOC);
        if ($item === false) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        $itemTotal = (string) $item['total'];
        if ($itemTotal !== $expectedPrice) {
            throw new DomainException('EDD_ORDER_UNVERIFIED');
        }
        return ['order' => $order, 'item' => $item];
    }
}

final class FocusaSpec152eEddProductAdapter
{
    /**
     * Frozen server-owned grant registry (spec 152E §8, spec 172 protected
     * offers). Callers never submit EDD IDs, prices, tiers, features, limits, or
     * commercial rights; the public product code is the only product input.
     */
    public const SERVER_OWNED_GRANTS = [
        'focusa_operator_lifetime_v1' => [
            'product' => 'focusa',
            'license_type' => 'focusa_operator_lifetime_v1',
            'posture' => 'paid',
            'products' => ['focusa'],
            'features' => [
                'base_focusa' => true,
                'automation' => true,
                'team_remote' => true,
                'release_proof' => true,
                'premium_updates' => true,
            ],
            'limits' => ['operator_seats' => 1, 'node_limit' => 3],
            'commercial' => [
                'term' => 'lifetime',
                'price_usd' => '697.00',
                'price_version' => 'v1',
                'refund_policy' => 'whole_order_30_days',
                'upgrade_policy' => 'explicit_upgrade_or_cross_grade_required_existing_operator_v1_preserved',
                'node_set' => 'operator_shared_v1',
            ],
        ],
        'uiai_operator_lifetime_v1' => [
            'product' => 'uiai_engine',
            'license_type' => 'uiai_operator_lifetime_v1',
            'posture' => 'paid',
            'products' => ['uiai_engine'],
            'features' => [
                'base_uiai' => true,
            ],
            'limits' => ['operator_seats' => 1, 'node_limit' => 3],
            'commercial' => [
                'term' => 'lifetime',
                'price_usd' => '697.00',
                'price_version' => 'v1',
                'refund_policy' => 'whole_order_30_days',
                'upgrade_policy' => 'explicit_upgrade_or_cross_grade_required_existing_operator_v1_preserved',
                'node_set' => 'operator_shared_v1',
            ],
        ],
        'focusa_uiai_operator_bundle_lifetime_v1' => [
            'product' => 'focusa',
            'license_type' => 'focusa_uiai_operator_bundle_lifetime_v1',
            'posture' => 'bundle',
            'products' => ['focusa', 'uiai_engine'],
            'features' => [
                'base_focusa' => true,
                'automation' => true,
                'team_remote' => true,
                'release_proof' => true,
                'premium_updates' => true,
                'base_uiai' => true,
            ],
            'limits' => ['operator_seats' => 1, 'node_limit' => 3],
            'commercial' => [
                'term' => 'lifetime',
                'price_usd' => '1254.60',
                'price_version' => 'v1',
                'refund_policy' => 'whole_order_30_days',
                'upgrade_policy' => 'explicit_upgrade_or_cross_grade_required_existing_operator_v1_preserved',
                'node_set' => 'operator_shared_v1',
            ],
        ],
        'focusa_evaluation' => [
            'product' => 'focusa',
            'license_type' => 'focusa_evaluation',
            'posture' => 'evaluation',
            'products' => ['focusa'],
            'features' => [
                'base_focusa' => true,
                'automation' => false,
                'team_remote' => false,
                'release_proof' => false,
                'premium_updates' => false,
            ],
            'limits' => ['operator_seats' => 1, 'node_limit' => 1],
            'commercial' => [
                'term' => 'evaluation_30_days',
                'price_usd' => '0.00',
                'price_version' => 'v1',
                'refund_policy' => 'none',
                'upgrade_policy' => 'no_downgrade_of_paid_records',
                'node_set' => 'operator_shared_v1',
            ],
        ],
    ];

    /** Resolve the server-owned grant for one public product code. */
    public static function resolve(string $productCode): array
    {
        $grant = self::SERVER_OWNED_GRANTS[$productCode] ?? null;
        if ($grant === null) {
            throw new DomainException('PRODUCT_MAPPING_REQUIRED');
        }
        return $grant;
    }
}

final class FocusaSpec152eEddNodeAdapter
{
    private PDO $db;
    private string $prefix;

    public function __construct(PDO $db, string $prefix = 'wp_')
    {
        $this->db = $db;
        $this->prefix = $prefix;
    }

    /**
     * Resolve one settled node bound to the verified account, the server-owned
     * product, and the presented device public key. The node row itself carries
     * the canonical EDD license binding; a node registered for another account,
     * product, or device can never produce a lease.
     */
    public function resolve(string $nodeId, string $accountUuid, string $productCode, string $devicePublicKey): array
    {
        if ($nodeId === '' || strlen($nodeId) > 128 || preg_match('/[\r\n@\x00]/', $nodeId) === 1) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        FocusaSpec152eEddBoundLeaseIssuer::assertPublicKey($devicePublicKey);
        $statement = $this->db->prepare(
            "SELECT node_uuid, account_uuid, edd_license_id, product_code, device_public_key,
                    assurance_class, status
             FROM {$this->prefix}wpuiai_authority_nodes WHERE node_uuid = :node"
        );
        $statement->execute([':node' => $nodeId]);
        $node = $statement->fetch(PDO::FETCH_ASSOC);
        if ($node === false) {
            throw new DomainException('NODE_NOT_FOUND');
        }
        if (($node['status'] ?? '') !== 'active') {
            throw new DomainException('NODE_NOT_ACTIVE');
        }
        if ((string) $node['account_uuid'] !== $accountUuid) {
            throw new DomainException('NODE_NOT_BOUND');
        }
        if ((string) $node['product_code'] !== $productCode) {
            throw new DomainException('NODE_NOT_BOUND');
        }
        if ((string) $node['device_public_key'] !== $devicePublicKey) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }
        return $node;
    }
}

/**
 * EDD-bound lease issuer/signer. Issues a signed `focusa.authority_lease.v1`
 * envelope only after the verified account, usable EDD license, settled
 * order/item, server-owned product grant, and settled bound node all resolve.
 * The monotonic per-account/product sequence is server-derived; refund/revoke/
 * expiry transitions advance the account sequence so prior leases become stale
 * and refresh is denied. Idempotent replay returns the same lease; changed
 * reuse of an idempotency key fails closed.
 */
final class FocusaSpec152eEddBoundLeaseIssuer
{
    public const SCHEMA = 'focusa.spec152e.edd_bound_lease_issuer.v1';
    public const LEASE_PAYLOAD_SCHEMA = 'focusa.authority_lease.v1';
    public const ENVELOPE_SCHEMA = 'focusa.signed_envelope.v1';
    public const RESULT_SCHEMA = 'focusa.spec152e.edd_bound_lease_issuance.v1';
    public const VERSION = 1;
    public const STATUS_ACTIVE = 'active';
    public const REFRESH_WINDOW_DAYS = 90;
    public const OFFLINE_GRACE_DAYS = 30;
    public const EVALUATION_DAYS = 30;
    public const NODE_ID_PATTERN = '/^[A-Za-z0-9_-]{1,128}$/D';
    public const DEVICE_KEY_PATTERN = '/^[A-Za-z0-9_-]{43}$/D';

    /** @var Closure(): string */
    private Closure $clock;
    private PDO $db;
    private string $prefix;
    private FocusaSpec152eAuthorityKeySetSeam $keySet;

    public function __construct(
        PDO $db,
        FocusaSpec152eAuthorityKeySetSeam $keySet,
        callable $clock,
        string $prefix = 'wp_',
    ) {
        $this->db = $db;
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        $this->keySet = $keySet;
        $this->clock = Closure::fromCallable($clock);
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->prefix = $prefix;
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        self::assertTimestamp($appliedAt);
        $encodedProvenance = self::encodeProvenance($provenance);
        $leases = $this->table('wpuiai_authority_leases');
        $sequences = $this->table('wpuiai_authority_lease_sequences');
        $idempotency = $this->table('wpuiai_authority_lease_idempotency');
        $migrations = $this->table('wpuiai_authority_lease_schema_migrations');
        $events = $this->table('wpuiai_authority_lease_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$leases} (
            lease_uuid {$uuid} NOT NULL PRIMARY KEY,
            account_uuid {$uuid} NOT NULL,
            customer_id BIGINT NOT NULL,
            edd_order_id BIGINT NOT NULL,
            edd_order_item_id BIGINT NOT NULL,
            edd_license_id BIGINT NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            posture VARCHAR(16) NOT NULL CHECK (posture IN ('paid', 'evaluation', 'bundle')),
            node_id VARCHAR(191) NOT NULL,
            sequence BIGINT NOT NULL CHECK (sequence >= 1),
            authority_key_id VARCHAR(64) NOT NULL,
            envelope_digest VARCHAR(70) NOT NULL,
            payload_digest VARCHAR(70) NOT NULL,
            payload_b64 TEXT NOT NULL,
            signature_b64 TEXT NOT NULL,
            issued_at VARCHAR(32) NOT NULL,
            not_before VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            offline_grace_until VARCHAR(32) NULL,
            status VARCHAR(16) NOT NULL CHECK (status IN ('active', 'superseded', 'refunded', 'revoked')),
            status_reason VARCHAR(191) NULL,
            idempotency_key {$key} NOT NULL UNIQUE,
            migration_provenance TEXT NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$sequences} (
            account_uuid {$uuid} NOT NULL,
            product_code VARCHAR(191) NOT NULL,
            current_sequence BIGINT NOT NULL DEFAULT 0 CHECK (current_sequence >= 0),
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            PRIMARY KEY (account_uuid, product_code)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$idempotency} (
            idempotency_key {$key} NOT NULL PRIMARY KEY,
            operation VARCHAR(32) NOT NULL,
            request_digest VARCHAR(64) NOT NULL,
            lease_uuid {$uuid} NOT NULL,
            result_state VARCHAR(16) NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$events} (
            event_key {$key} NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            schema_version BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");

        $statement = $this->db->prepare(
            "INSERT INTO {$migrations} (schema_version, schema_name, applied_at, migration_provenance)
             SELECT :version, :schema, :applied_at, :provenance
             WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)"
        );
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied_at' => $appliedAt,
            ':provenance' => $encodedProvenance,
            ':existing_version' => self::VERSION,
        ]);
    }

    /**
     * Issue one signed lease. Required request fields (caller-bounded):
     *   - account_uuid: verified authority account
     *   - product_code: public code (server maps to exact grants)
     *   - node_id: settled node id
     *   - device_public_key: the node-bound device public key
     *   - idempotency_key, request_id
     * Optional test seam fields (fixed-clock golden-vector seams):
     *   - lease_uuid, lease_id, issued_at
     * The EDD license is resolved from the settled node binding — the caller can
     * never select a license, order, customer, or price.
     */
    public function issueLease(array $request): array
    {
        $accountUuid = (string) ($request['account_uuid'] ?? '');
        self::assertUuid($accountUuid, 'account');
        $productCode = (string) ($request['product_code'] ?? '');
        $nodeId = (string) ($request['node_id'] ?? '');
        $devicePublicKey = (string) ($request['device_public_key'] ?? '');
        $idempotencyKey = (string) ($request['idempotency_key'] ?? '');
        $requestId = (string) ($request['request_id'] ?? '');
        $this->assertIdempotencyKey($idempotencyKey);
        $this->assertRequestId($requestId);
        self::assertPublicKey($devicePublicKey);
        if (preg_match(self::NODE_ID_PATTERN, $nodeId) !== 1) {
            throw new DomainException('NODE_NOT_FOUND');
        }

        // Caller-controlled grant/price/feature/limit/commercial fields are never
        // accepted (spec 152E §8, §19.7): the product code is the only product
        // input and the registry is the only grant source.
        foreach (['features', 'limits', 'commercial', 'price', 'amount', 'total', 'currency',
                  'tier', 'license_type', 'products', 'grants', 'node_limit', 'operator_seats',
                  'refund_policy', 'upgrade_policy', 'sequence'] as $field) {
            if (array_key_exists($field, $request)) {
                throw new DomainException('CALLER_CONTROLLED_GRANT_DENIED');
            }
        }

        $digest = $this->digest([
            'operation' => 'issue_lease',
            'account_uuid' => $accountUuid,
            'product_code' => $productCode,
            'node_id' => $nodeId,
            'device_public_key' => $devicePublicKey,
        ]);

        return $this->transaction(function () use ($request, $accountUuid, $productCode, $nodeId, $devicePublicKey, $idempotencyKey, $requestId, $digest): array {
            $replay = $this->replay($idempotencyKey, 'issue_lease', $digest);
            if ($replay !== null) {
                return $this->leaseResult($replay['lease_uuid']);
            }

            $grant = FocusaSpec152eEddProductAdapter::resolve($productCode);
            $node = (new FocusaSpec152eEddNodeAdapter($this->db, $this->prefix))->resolve(
                $nodeId,
                $accountUuid,
                $productCode,
                $devicePublicKey,
            );
            $account = (new FocusaSpec152eEddAccountAdapter($this->db, $this->prefix))->resolve($accountUuid);
            $customerId = (int) $account['customer_id'];
            $now = (string) ($this->clock)();

            $license = (new FocusaSpec152eEddLicenseAdapter($this->db, $this->prefix))->resolve(
                (int) $node['edd_license_id'],
                $customerId,
                $now,
            );
            if ((int) $license['download_id'] !== $this->grantDownloadId($productCode)) {
                throw new DomainException('EDD_ORDER_UNVERIFIED');
            }
            $orderBinding = (new FocusaSpec152eEddOrderAdapter($this->db, $this->prefix))->resolve(
                $license,
                $customerId,
                $grant['commercial']['price_usd'],
            );

            $sequence = $this->nextSequence($accountUuid, $productCode, (int) $account['highest_entitlement_sequence']);
            $issuedAt = (string) ($request['issued_at'] ?? $now);
            self::assertTimestamp($issuedAt);
            $leaseUuid = (string) ($request['lease_uuid'] ?? self::opaqueUuid());
            self::assertUuid($leaseUuid, 'lease');
            $leaseId = (string) ($request['lease_id'] ?? $leaseUuid);

            $payload = $this->buildPayload(
                $leaseId,
                $account,
                $license,
                $orderBinding,
                $grant,
                $node,
                $sequence,
                $issuedAt,
            );
            $payloadBytes = self::canonicalJson($payload);
            $envelope = $this->keySet->seal(
                $payload,
                FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
                $this->keySet->leaseSeed(),
                FocusaSpec152eEd25519Signer::LEASE_DOMAIN,
            );

            $envelopeDigest = 'sha256:' . hash('sha256', self::canonicalJson($envelope));
            $payloadDigest = 'sha256:' . hash('sha256', $payloadBytes);
            $statement = $this->db->prepare(
                "INSERT INTO {$this->table('wpuiai_authority_leases')}
                 (lease_uuid, account_uuid, customer_id, edd_order_id, edd_order_item_id,
                  edd_license_id, product_code, posture, node_id, sequence, authority_key_id,
                  envelope_digest, payload_digest, payload_b64, signature_b64,
                  issued_at, not_before, expires_at,
                  offline_grace_until, status, status_reason, idempotency_key,
                  migration_provenance, created_at, updated_at)
                 VALUES (:lease_uuid, :account_uuid, :customer_id, :edd_order_id, :edd_order_item_id,
                  :edd_license_id, :product_code, :posture, :node_id, :sequence, :authority_key_id,
                  :envelope_digest, :payload_digest, :payload_b64, :signature_b64,
                  :issued_at, :not_before, :expires_at,
                  :offline_grace_until, :status, :status_reason, :idempotency_key,
                  :migration_provenance, :created_at, :updated_at)"
            );
            $statement->execute([
                ':lease_uuid' => $leaseUuid,
                ':account_uuid' => $accountUuid,
                ':customer_id' => $customerId,
                ':edd_order_id' => (int) $orderBinding['order']['order_id'],
                ':edd_order_item_id' => (int) $orderBinding['item']['order_item_id'],
                ':edd_license_id' => (int) $license['license_id'],
                ':product_code' => $productCode,
                ':posture' => $grant['posture'],
                ':node_id' => $nodeId,
                ':sequence' => $sequence,
                ':authority_key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
                ':envelope_digest' => $envelopeDigest,
                ':payload_digest' => $payloadDigest,
                ':payload_b64' => (string) $envelope['payload_b64'],
                ':signature_b64' => (string) $envelope['signature_b64'],
                ':issued_at' => $payload['issued_at'],
                ':not_before' => $payload['not_before'],
                ':expires_at' => $payload['expires_at'],
                ':offline_grace_until' => $payload['offline_grace_until'],
                ':status' => self::STATUS_ACTIVE,
                ':status_reason' => null,
                ':idempotency_key' => $idempotencyKey,
                ':migration_provenance' => self::encodeProvenance([
                    'source' => 'edd_bound_lease_issuer',
                    'request_id' => $requestId,
                ]),
                ':created_at' => $now,
                ':updated_at' => $now,
            ]);
            $this->recordIdempotency($idempotencyKey, 'issue_lease', $digest, $leaseUuid, self::STATUS_ACTIVE, $now);
            $this->bumpSequence($accountUuid, $productCode, $sequence, $now);

            return $this->leaseResult($leaseUuid);
        });
    }

    /**
     * Verify a signed envelope like the runtime verifier: key set trust, key
     * validity, domain-separated signature, key-id match, product/node match,
     * minimum sequence, previous digest, status, not-before, and expiry/offline
     * grace. Returns the entitlement snapshot; throws DomainException codes on
     * every rejection (fail closed).
     */
    public function verifyEnvelope(array $envelope, array $context): array
    {
        if (($envelope['schema'] ?? '') !== self::ENVELOPE_SCHEMA) {
            throw new DomainException('UNSUPPORTED_ENVELOPE_SCHEMA');
        }
        $signerKeyId = (string) ($envelope['signer_key_id'] ?? '');
        $payloadBytes = FocusaSpec152eAuthorityKeySetSeam::decodePayload((string) ($envelope['payload_b64'] ?? ''));
        $signature = base64_decode((string) ($envelope['signature_b64'] ?? ''), true);
        if ($signature === false) {
            throw new DomainException('INVALID_BASE64');
        }
        $keySet = $this->keySet();
        $key = $keySet['keys'][0] ?? null;
        if ($key === null || ($key['key_id'] ?? '') !== $signerKeyId) {
            throw new DomainException('UNKNOWN_KEY');
        }
        if (($key['status'] ?? '') === 'revoked') {
            throw new DomainException('REVOKED_KEY');
        }
        $now = (string) ($context['now'] ?? (string) ($this->clock)());
        if ($now < $key['not_before'] || $now > $key['not_after']) {
            throw new DomainException('KEY_OUTSIDE_VALIDITY');
        }
        $publicKey = base64_decode((string) $key['public_key_b64'], true);
        if ($publicKey === false || strlen($publicKey) !== 32) {
            throw new DomainException('INVALID_PUBLIC_KEY');
        }
        if (!FocusaSpec152eEd25519Signer::verify($publicKey, $signature, FocusaSpec152eEd25519Signer::LEASE_DOMAIN, $payloadBytes)) {
            throw new DomainException('INVALID_SIGNATURE');
        }
        $payload = FocusaSpec152eAuthorityKeySetSeam::decodeJson((string) ($envelope['payload_b64'] ?? ''));
        if (($payload['schema'] ?? '') !== self::LEASE_PAYLOAD_SCHEMA) {
            throw new DomainException('UNSUPPORTED_PAYLOAD_SCHEMA');
        }
        if (($payload['authority_key_id'] ?? '') !== $signerKeyId) {
            throw new DomainException('AUTHORITY_KEY_MISMATCH');
        }
        if (($payload['product'] ?? '') !== (string) ($context['expected_product'] ?? 'focusa')) {
            throw new DomainException('WRONG_PRODUCT');
        }
        if (($payload['node_id'] ?? '') !== (string) ($context['expected_node_id'] ?? '')) {
            throw new DomainException('WRONG_NODE');
        }
        $sequence = (int) ($payload['sequence'] ?? 0);
        if (isset($context['minimum_sequence']) && $sequence < (int) $context['minimum_sequence']) {
            throw new DomainException('STALE_SEQUENCE');
        }
        if (isset($context['expected_previous_digest'])
            && ($payload['previous_lease_digest'] ?? null) !== $context['expected_previous_digest']) {
            throw new DomainException('PREVIOUS_DIGEST_MISMATCH');
        }
        if (($payload['status'] ?? '') === 'revoked') {
            throw new DomainException('REVOKED_LEASE');
        }
        if ($now < (string) ($payload['not_before'] ?? '')) {
            throw new DomainException('NOT_YET_VALID');
        }
        $state = 'active';
        if ($now > (string) ($payload['expires_at'] ?? '')) {
            $grace = (string) ($payload['offline_grace_until'] ?? '');
            if ($grace !== '' && $now <= $grace) {
                $state = 'offline_grace';
            } else {
                throw new DomainException('EXPIRED');
            }
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'state' => $state,
            'product' => (string) $payload['product'],
            'node_id' => (string) $payload['node_id'],
            'lease_id' => (string) $payload['lease_id'],
            'sequence' => $sequence,
            'lease_digest' => 'sha256:' . hash('sha256', $payloadBytes),
            'expires_at' => (string) $payload['expires_at'],
            'offline_grace_until' => (string) ($payload['offline_grace_until'] ?? ''),
            'features' => (array) ($payload['features'] ?? []),
            'limits' => (array) ($payload['limits'] ?? []),
        ];
    }

    /** Build the canonical lease payload with every required claim group. */
    private function buildPayload(
        string $leaseId,
        array $account,
        array $license,
        array $orderBinding,
        array $grant,
        array $node,
        int $sequence,
        string $issuedAt,
    ): array {
        $expiresAt = self::plusDays($issuedAt, $grant['posture'] === 'evaluation' ? self::EVALUATION_DAYS : self::REFRESH_WINDOW_DAYS);
        $offlineGraceUntil = $grant['posture'] === 'evaluation' ? null : self::plusDays($expiresAt, self::OFFLINE_GRACE_DAYS);
        $previous = $this->previousLeaseDigest((string) $account['account_uuid'], (string) $grant['license_type'], $sequence);
        $payload = [
            'schema' => self::LEASE_PAYLOAD_SCHEMA,
            'lease_id' => $leaseId,
            'product' => $grant['product'],
            'product_code' => $grant['license_type'],
            'posture' => $grant['posture'],
            'subject_id' => (string) $account['account_uuid'],
            'account_id' => (string) $account['account_uuid'],
            'customer_id' => (int) $account['customer_id'],
            'order_id' => (int) $orderBinding['order']['order_id'],
            'order_item_id' => (int) $orderBinding['item']['order_item_id'],
            'edd_license_id' => (int) $license['license_id'],
            'node_id' => (string) $node['node_uuid'],
            'sequence' => $sequence,
            'issued_at' => $issuedAt,
            'not_before' => $issuedAt,
            'expires_at' => $expiresAt,
            'offline_grace_until' => $offlineGraceUntil,
            'authority_key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
            'status' => self::STATUS_ACTIVE,
            'features' => $grant['features'],
            'limits' => $grant['limits'],
            'commercial' => $grant['commercial'],
        ];
        if ($previous !== null) {
            $payload['previous_lease_digest'] = $previous;
        }
        return $payload;
    }

    /** Server-derived monotonic sequence: strictly greater than the prior lease
     *  and than any entitlement transition already recorded on the account. */
    private function nextSequence(string $accountUuid, string $productCode, int $accountSequence): int
    {
        $ledger = $this->sequenceLedger($accountUuid, $productCode);
        $base = $ledger !== null ? (int) $ledger['current_sequence'] : 0;
        if ($accountSequence > $base) {
            return $accountSequence + 1;
        }
        return $base + 1;
    }

    public function sequenceLedger(string $accountUuid, string $productCode): ?array
    {
        $statement = $this->db->prepare(
            "SELECT account_uuid, product_code, current_sequence
             FROM {$this->table('wpuiai_authority_lease_sequences')}
             WHERE account_uuid = :account AND product_code = :product"
        );
        $statement->execute([':account' => $accountUuid, ':product' => $productCode]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function findLease(string $leaseUuid): ?array
    {
        self::assertUuid($leaseUuid, 'lease');
        $statement = $this->db->prepare(
            "SELECT * FROM {$this->table('wpuiai_authority_leases')} WHERE lease_uuid = :uuid"
        );
        $statement->execute([':uuid' => $leaseUuid]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    public function leaseCount(): int
    {
        return (int) $this->db->query("SELECT COUNT(*) FROM {$this->table('wpuiai_authority_leases')}")->fetchColumn();
    }

    private function leaseResult(string $leaseUuid): array
    {
        $lease = $this->findLease($leaseUuid);
        if ($lease === null) {
            throw new DomainException('LEASE_NOT_FOUND');
        }
        return [
            'schema' => self::RESULT_SCHEMA,
            'lease_uuid' => (string) $lease['lease_uuid'],
            'lease_id' => (string) $lease['lease_uuid'],
            'account_uuid' => (string) $lease['account_uuid'],
            'product_code' => (string) $lease['product_code'],
            'posture' => (string) $lease['posture'],
            'node_id' => (string) $lease['node_id'],
            'sequence' => (int) $lease['sequence'],
            'envelope_digest' => (string) $lease['envelope_digest'],
            'payload_digest' => (string) $lease['payload_digest'],
            'status' => (string) $lease['status'],
            'issued_at' => (string) $lease['issued_at'],
            'not_before' => (string) $lease['not_before'],
            'expires_at' => (string) $lease['expires_at'],
            'offline_grace_until' => (string) ($lease['offline_grace_until'] ?? ''),
            'authority_key_id' => (string) $lease['authority_key_id'],
            'envelope' => [
                'schema' => self::ENVELOPE_SCHEMA,
                'signer_key_id' => (string) $lease['authority_key_id'],
                'payload_b64' => (string) $lease['payload_b64'],
                'signature_b64' => (string) $lease['signature_b64'],
            ],
            'claims' => FocusaSpec152eAuthorityKeySetSeam::decodeJson((string) $lease['payload_b64']),
        ];
    }

    private function previousLeaseDigest(string $accountUuid, string $licenseType, int $sequence): ?string
    {
        if ($sequence < 2) {
            return null;
        }
        $statement = $this->db->prepare(
            "SELECT payload_digest FROM {$this->table('wpuiai_authority_leases')}
             WHERE account_uuid = :account AND product_code = :product AND sequence < :sequence
             ORDER BY sequence DESC LIMIT 1"
        );
        $statement->execute([':account' => $accountUuid, ':product' => $licenseType, ':sequence' => $sequence]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : (string) $row['payload_digest'];
    }

    private function bumpSequence(string $accountUuid, string $productCode, int $sequence, string $now): void
    {
        $table = $this->table('wpuiai_authority_lease_sequences');
        $update = $this->db->prepare(
            "UPDATE {$table} SET current_sequence = :sequence, updated_at = :now
             WHERE account_uuid = :account AND product_code = :product"
        );
        $update->execute([':sequence' => $sequence, ':now' => $now, ':account' => $accountUuid, ':product' => $productCode]);
        if ($update->rowCount() === 0) {
            $insert = $this->db->prepare(
                "INSERT INTO {$table} (account_uuid, product_code, current_sequence, created_at, updated_at)
                 VALUES (:account, :product, :sequence, :now, :now)"
            );
            $insert->execute([':account' => $accountUuid, ':product' => $productCode, ':sequence' => $sequence, ':now' => $now]);
        }
    }

    /** The active lease key from the seam's canonical key set (for verification). */
    private function keySet(): array
    {
        return [
            'schema' => FocusaSpec152eAuthorityKeySetSeam::KEY_SET_SCHEMA,
            'sequence' => FocusaSpec152eAuthorityKeySetSeam::KEY_SET_SEQUENCE,
            'issued_at' => '2026-08-01T00:00:00Z',
            'expires_at' => '2030-01-01T00:00:00Z',
            'keys' => [[
                'key_id' => FocusaSpec152eAuthorityKeySetSeam::LEASE_KEY_ID,
                'public_key_b64' => $this->keySet->leasePublicKeyB64(),
                'status' => 'active',
                'not_before' => '2026-08-01T00:00:00Z',
                'not_after' => '2029-01-01T00:00:00Z',
            ]],
        ];
    }

    private function grantDownloadId(string $productCode): int
    {
        // Server-owned download mapping (spec 152E §8, spec 172 protected offers):
        // the evaluation and paid licenses bind to the mapped EDD download for the
        // product code. The fixture registry pins explicit downloads so the
        // implicit Download-453 mapping is never used.
        return [
            'focusa_operator_lifetime_v1' => 1001,
            'uiai_operator_lifetime_v1' => 1002,
            'focusa_uiai_operator_bundle_lifetime_v1' => 1003,
            'focusa_evaluation' => 1004,
        ][$productCode] ?? 0;
    }

    public function table(string $name): string
    {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $name) !== 1) {
            throw new InvalidArgumentException('invalid table name');
        }
        return $this->prefix . $name;
    }

    public static function encodeProvenance(array $provenance): string
    {
        ksort($provenance, SORT_STRING);
        return json_encode($provenance, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }

    /** Canonical JSON with sorted keys and compact separators (Python-compatible). */
    public static function canonicalJson(array $value): string
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

    public static function assertTimestamp(?string $timestamp, bool $nullable = false): void
    {
        if ($timestamp === null || $timestamp === '') {
            if ($nullable) {
                return;
            }
            throw new InvalidArgumentException('RFC3339 timestamp required');
        }
        $parsed = DateTimeImmutable::createFromFormat('Y-m-d\TH:i:s\Z', $timestamp);
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('RFC3339 timestamp required');
        }
    }

    public static function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("bounded {$kind} uuid required");
        }
    }

    public static function assertPublicKey(string $devicePublicKey): void
    {
        if (preg_match(self::DEVICE_KEY_PATTERN, $devicePublicKey) !== 1) {
            throw new DomainException('NODE_PUBLIC_KEY_REQUIRED');
        }
    }

    public function assertIdempotencyKey(string $idempotencyKey): void
    {
        if ($idempotencyKey === '' || strlen($idempotencyKey) > 191
            || preg_match('/[\r\n@\x00]/', $idempotencyKey) === 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
    }

    public function assertRequestId(string $requestId): void
    {
        if ($requestId === '' || strlen($requestId) > 191
            || preg_match('/[\r\n@\x00]/', $requestId) === 1) {
            throw new InvalidArgumentException('bounded request id required');
        }
    }

    private function digest(array $parts): string
    {
        ksort($parts, SORT_STRING);
        return hash('sha256', json_encode($parts, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES));
    }

    private function transaction(callable $operation): array
    {
        $this->db->beginTransaction();
        try {
            $result = $operation();
            $this->db->commit();
            return $result;
        } catch (Throwable $error) {
            $this->db->rollBack();
            throw $error;
        }
    }

    private function replay(string $idempotencyKey, string $operation, string $digest): ?array
    {
        $statement = $this->db->prepare(
            "SELECT lease_uuid, operation, request_digest, result_state
             FROM {$this->table('wpuiai_authority_lease_idempotency')} WHERE idempotency_key = :key"
        );
        $statement->execute([':key' => $idempotencyKey]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if ($row['operation'] !== $operation || $row['request_digest'] !== $digest) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return $row;
    }

    private function recordIdempotency(string $idempotencyKey, string $operation, string $digest, string $leaseUuid, string $state, string $now): void
    {
        $statement = $this->db->prepare(
            "INSERT INTO {$this->table('wpuiai_authority_lease_idempotency')}
             (idempotency_key, operation, request_digest, lease_uuid, result_state, created_at)
             VALUES (:key, :operation, :digest, :lease, :state, :now)"
        );
        $statement->execute([
            ':key' => $idempotencyKey,
            ':operation' => $operation,
            ':digest' => $digest,
            ':lease' => $leaseUuid,
            ':state' => $state,
            ':now' => $now,
        ]);
    }

    public static function plusDays(string $timestamp, int $days): string
    {
        $date = new DateTimeImmutable($timestamp, new DateTimeZone('UTC'));
        return $date->modify('+' . $days . ' days')->format('Y-m-d\TH:i:s\Z');
    }

    public static function opaqueToken(string $prefix): string
    {
        return $prefix . bin2hex(random_bytes(16));
    }

    /** Opaque UUID-format lease identifier (v4-shaped, non-authoritative). */
    public static function opaqueUuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        $hex = bin2hex($bytes);
        return sprintf(
            '%s-%s-%s-%s-%s',
            substr($hex, 0, 8),
            substr($hex, 8, 4),
            substr($hex, 12, 4),
            substr($hex, 16, 4),
            substr($hex, 20, 12),
        );
    }
}
