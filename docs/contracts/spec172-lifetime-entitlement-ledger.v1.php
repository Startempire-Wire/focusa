<?php
// Spec 172 §14 lifetime entitlement ledger + bounded device credential issuer
// (atom focusa-vbcqu.20.15.22, 172.03.05, lane spec152f / WPUIAI).
//
// The WPUIAI authority keeps the perpetual License Type entitlement (term
// `lifetime`) in a ledger that is SEPARATE from the bounded signed device
// leases used for execution:
//
//   - wp_wpuiai_lifetime_entitlements: one row per product; status
//     entitled|revoked; the highest authority sequence; server-owned price
//     version, family digest, node/seat limits. Refund/revoke/chargeback at a
//     strictly higher authority sequence flips status to revoked — nothing
//     else can, and no stale or offline device credential can override it.
//   - wp_wpuiai_device_credentials: bounded signed leases with a 90-day
//     refresh window and a 30-day offline grace. Rotation (refresh), verified
//     recovery issuance after expiry, lost-device revoke, and key rotation
//     re-sign a fresh bounded lease. A device credential never carries
//     product, price, License Type, family, feature, limit, node, or
//     commercial right — those live in the perpetual entitlement record.
//
// Credential expiry never erases lifetime rights (verified recovery issues a
// replacement lease); a revoked lifetime entitlement defeats every stale and
// offline device credential; Offline Grace stays bounded. All identifiers are
// synthetic; no raw email, key, token, customer row, or card data appears.
declare(strict_types=1);

require_once __DIR__ . '/spec172-limited-access-assertion-service.v1.php';

final class FocusaSpec172LifetimeEntitlementMigration
{
    public const SCHEMA = 'focusa.spec172.lifetime_entitlement.v1';
    public const TERM = 'lifetime';
    public const REFRESH_WINDOW_DAYS = 90;
    public const OFFLINE_GRACE_DAYS = 30;

    public function __construct(private PDO $db, private string $prefix = 'wp_')
    {
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        self::assertTimestamp($appliedAt);
        $p = $this->prefix;
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$p}wpuiai_lifetime_entitlements (
            product VARCHAR(64) NOT NULL,
            license_type VARCHAR(64) NOT NULL,
            term VARCHAR(16) NOT NULL,
            status VARCHAR(16) NOT NULL,
            sequence BIGINT NOT NULL,
            price_version VARCHAR(32) NOT NULL,
            family_digest VARCHAR(96) NOT NULL,
            node_limit INTEGER NOT NULL,
            operator_seats INTEGER NOT NULL,
            updated_at VARCHAR(32) NOT NULL,
            PRIMARY KEY (product)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$p}wpuiai_device_credentials (
            lease_id VARCHAR(96) NOT NULL,
            product VARCHAR(64) NOT NULL,
            node_id VARCHAR(96) NOT NULL,
            sequence BIGINT NOT NULL,
            issued_at VARCHAR(32) NOT NULL,
            expires_at VARCHAR(32) NOT NULL,
            offline_grace_until VARCHAR(32) NOT NULL,
            authority_key_id VARCHAR(64) NOT NULL,
            status VARCHAR(16) NOT NULL,
            PRIMARY KEY (lease_id),
            UNIQUE (product, node_id)
        )");
    }

    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        return [
            'schema' => 'focusa.spec172.lifetime_rollback_journal.v1',
            'occurred_at' => $occurredAt,
            'preservation_only' => true,
            'entitlements_preserved' => $this->countRows('wpuiai_lifetime_entitlements'),
            'credentials_preserved' => $this->countRows('wpuiai_device_credentials'),
            'provenance' => $provenance,
        ];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    public function countRows(string $table): int
    {
        return (int) $this->db->query("SELECT COUNT(*) FROM {$this->prefix}{$table}")->fetchColumn();
    }

    public static function assertTimestamp(?string $timestamp, bool $nullable = false): void
    {
        if ($timestamp === null && $nullable) {
            return;
        }
        if (preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/D', (string) $timestamp) !== 1) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }
}

/**
 * WPUIAI lifetime entitlement ledger and bounded device credential issuer.
 * Issue/record only from canonical projections; refresh rotates the bounded
 * credential; verified recovery issues a replacement lease after expiry;
 * refund/revoke at a higher authority sequence revokes the lifetime
 * entitlement; verification fails closed on revoked, stale, unknown, and
 * expired-without-recovery postures. Signatures are real RFC 8032 Ed25519
 * (pure PHP, shared with the limited-access assertion service) over canonical
 * payload bytes so Python and Rust can independently verify the same bytes.
 */
final class FocusaSpec172LifetimeEntitlementLedger
{
    public const SCHEMA = 'focusa.spec172.lifetime_entitlement.v1';
    public const DEVICE_CREDENTIAL_SCHEMA = 'focusa.spec172.device_credential.v1';
    public const TERM = 'lifetime';
    public const REFRESH_WINDOW_DAYS = 90;
    public const OFFLINE_GRACE_DAYS = 30;
    public const PRODUCT = 'focusa';
    public const LICENSE_TYPE = 'focusa_operator_lifetime_v1';
    public const NODE_LIMIT = 3;
    public const OPERATOR_SEATS = 1;
    public const NODE_ID = 'node-operator-lt-001';
    public const AUTHORITY_KEY_ID = 'authority-lease-2026-01';
    public const AUTHORITY_KEY_ID_ROTATED = 'authority-lease-2026-02';
    public const SEED_HEX = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
    public const ALGORITHM = 'ed25519.spec172.v1';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private string $prefix = 'wp_',
        ?callable $clock = null,
    ) {
        $this->clock = $clock ?? static fn (): string => '2026-08-09T06:00:00Z';
    }

    // ── canonical helpers ────────────────────────────────────────────────

    public function publicKeyHex(): string
    {
        return bin2hex(FocusaSpec172Ed25519::keypair(hex2bin(self::SEED_HEX))['public_key']);
    }

    public function signPayload(array $payload): string
    {
        $message = FocusaSpec172LimitedAssertionPayload::encodeCanonical($payload);
        $keypair = FocusaSpec172Ed25519::keypair(hex2bin(self::SEED_HEX));
        return bin2hex(FocusaSpec172Ed25519::sign($message, $keypair['public_key'], $keypair['secret_key']));
    }

    public function verifyPayload(array $payload, string $signatureHex): bool
    {
        if (preg_match('/^[0-9a-f]{128}$/D', $signatureHex) !== 1) {
            return false;
        }
        $message = FocusaSpec172LimitedAssertionPayload::encodeCanonical($payload);
        $keypair = FocusaSpec172Ed25519::keypair(hex2bin(self::SEED_HEX));
        return FocusaSpec172Ed25519::verify($message, $keypair['public_key'], hex2bin($signatureHex));
    }

    public static function leaseId(string $product, string $nodeId, int $sequence): string
    {
        return 'lease-' . hash('sha256', $product . "\0" . $nodeId . "\0" . $sequence);
    }

    private function plusDays(string $timestamp, int $days): string
    {
        return (new DateTimeImmutable($timestamp))->modify('+' . $days . ' days')->format('Y-m-d\TH:i:s\Z');
    }

    // ── lifetime entitlement ledger ──────────────────────────────────────

    /**
     * Record the perpetual License Type entitlement from exactly one canonical
     * Operator projection. Product, License Type, term, price version, family
     * digest, and limits are server-owned; a caller can never select them.
     * Re-recording at a strictly higher sequence updates the row (server
     * re-issuance); equal or lower sequences fail closed.
     */
    public function recordEntitlement(array $projection): array
    {
        if (($projection['decision'] ?? '') !== 'license_type_projected'
            || ($projection['status'] ?? '') !== 'active') {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        if (($projection['license_type'] ?? '') !== self::LICENSE_TYPE) {
            throw new DomainException('LICENSE_TYPE_NOT_INCLUDED');
        }
        if (($projection['term'] ?? '') !== self::TERM) {
            throw new DomainException('LICENSE_TYPE_NOT_INCLUDED');
        }
        $sequence = filter_var($projection['sequence'] ?? null, FILTER_VALIDATE_INT);
        if ($sequence === false || $sequence < 1) {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        $priceVersion = (string) ($projection['price_version'] ?? '');
        $familyDigest = (string) ($projection['family_digest'] ?? '');
        if ($priceVersion === '' || $familyDigest === '') {
            throw new DomainException('LICENSE_TYPE_PROJECTION_REQUIRED');
        }
        $existing = $this->entitlementRow(self::PRODUCT);
        if ($existing !== null && $sequence <= (int) $existing['sequence']) {
            throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
        }
        $updatedAt = (string) ($this->clock)();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($updatedAt);
        $stmt = $this->db->prepare(
            "INSERT INTO {$this->prefix}wpuiai_lifetime_entitlements
                (product, license_type, term, status, sequence, price_version, family_digest, node_limit, operator_seats, updated_at)
             VALUES (:product, :license_type, :term, 'entitled', :sequence, :price_version, :family_digest, :node_limit, :operator_seats, :updated_at)
             ON CONFLICT(product) DO UPDATE SET
                license_type = excluded.license_type, term = excluded.term, status = 'entitled',
                sequence = excluded.sequence, price_version = excluded.price_version,
                family_digest = excluded.family_digest, node_limit = excluded.node_limit,
                operator_seats = excluded.operator_seats, updated_at = excluded.updated_at",
        );
        $stmt->execute([
            ':product' => self::PRODUCT,
            ':license_type' => self::LICENSE_TYPE,
            ':term' => self::TERM,
            ':sequence' => $sequence,
            ':price_version' => $priceVersion,
            ':family_digest' => $familyDigest,
            ':node_limit' => self::NODE_LIMIT,
            ':operator_seats' => self::OPERATOR_SEATS,
            ':updated_at' => $updatedAt,
        ]);
        return $this->entitlementRow(self::PRODUCT);
    }

    public function entitlementRow(string $product): ?array
    {
        $stmt = $this->db->prepare(
            "SELECT product, license_type, term, status, sequence, price_version, family_digest,
                    node_limit, operator_seats, updated_at
             FROM {$this->prefix}wpuiai_lifetime_entitlements WHERE product = :product",
        );
        $stmt->execute([':product' => $product]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /**
     * Refund/revoke/chargeback: mark the lifetime entitlement Revoked at a
     * strictly higher authority sequence. Stale and offline device
     * credentials can never override this decision.
     */
    public function applyRefundRevoke(int $higherSequence, string $occurredAt): array
    {
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($occurredAt);
        $entitlement = $this->entitlementRow(self::PRODUCT);
        if ($entitlement === null) {
            throw new DomainException('LIFETIME_ENTITLEMENT_MISSING');
        }
        if ($higherSequence <= (int) $entitlement['sequence']) {
            throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
        }
        $stmt = $this->db->prepare(
            "UPDATE {$this->prefix}wpuiai_lifetime_entitlements
             SET status = 'revoked', sequence = :sequence, updated_at = :updated_at
             WHERE product = :product AND sequence < :sequence",
        );
        $stmt->execute([
            ':sequence' => $higherSequence,
            ':updated_at' => $occurredAt,
            ':product' => self::PRODUCT,
        ]);
        return $this->entitlementRow(self::PRODUCT);
    }

    // ── bounded device credential issuer ────────────────────────────────

    private function credentialPayload(int $sequence, string $nodeId, string $issuedAt, string $authorityKeyId, string $status = 'active'): array
    {
        $expiresAt = $this->plusDays($issuedAt, self::REFRESH_WINDOW_DAYS);
        $offlineGraceUntil = $this->plusDays($expiresAt, self::OFFLINE_GRACE_DAYS);
        return [
            'schema' => self::DEVICE_CREDENTIAL_SCHEMA,
            'lease_id' => self::leaseId(self::PRODUCT, $nodeId, $sequence),
            'product' => self::PRODUCT,
            'node_id' => $nodeId,
            'sequence' => $sequence,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
            'offline_grace_until' => $offlineGraceUntil,
            'authority_key_id' => $authorityKeyId,
            'status' => $status,
        ];
    }

    private function storeCredential(array $payload, string $signatureHex): array
    {
        $stmt = $this->db->prepare(
            "INSERT OR REPLACE INTO {$this->prefix}wpuiai_device_credentials
                (lease_id, product, node_id, sequence, issued_at, expires_at, offline_grace_until, authority_key_id, status)
             VALUES (:lease_id, :product, :node_id, :sequence, :issued_at, :expires_at, :offline_grace_until, :authority_key_id, :status)",
        );
        $stmt->execute([
            ':lease_id' => $payload['lease_id'],
            ':product' => $payload['product'],
            ':node_id' => $payload['node_id'],
            ':sequence' => $payload['sequence'],
            ':issued_at' => $payload['issued_at'],
            ':expires_at' => $payload['expires_at'],
            ':offline_grace_until' => $payload['offline_grace_until'],
            ':authority_key_id' => $payload['authority_key_id'],
            ':status' => $payload['status'],
        ]);
        return ['payload' => $payload, 'signature_hex' => $signatureHex];
    }

    public function credentialRow(string $product, string $nodeId): ?array
    {
        $stmt = $this->db->prepare(
            "SELECT lease_id, product, node_id, sequence, issued_at, expires_at, offline_grace_until, authority_key_id, status
             FROM {$this->prefix}wpuiai_device_credentials WHERE product = :product AND node_id = :node_id",
        );
        $stmt->execute([':product' => $product, ':node_id' => $nodeId]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        // The canonical signed payload always carries the schema field; the
        // ledger row mirrors it so Python/Rust verify the exact same bytes.
        $row['schema'] = self::DEVICE_CREDENTIAL_SCHEMA;
        return $row;
    }

    private function requireEntitled(): array
    {
        $entitlement = $this->entitlementRow(self::PRODUCT);
        if ($entitlement === null) {
            throw new DomainException('LIFETIME_ENTITLEMENT_MISSING');
        }
        if ($entitlement['status'] !== 'entitled') {
            throw new DomainException('VERIFIED_LIMITED_ACCESS');
        }
        return $entitlement;
    }

    /**
     * Issue the very first bounded lease for a node at the entitlement's
     * current sequence (no bump: the initial credential chains to the
     * entitlement as issued). Refused when a credential already exists.
     */
    public function issueInitialCredential(string $nodeId, string $now): array
    {
        $entitlement = $this->requireEntitled();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($now);
        $current = $this->credentialRow(self::PRODUCT, $nodeId);
        if ($current !== null) {
            throw new DomainException('DEVICE_CREDENTIAL_ALREADY_ISSUED');
        }
        $sequence = (int) $entitlement['sequence'];
        $payload = $this->credentialPayload($sequence, $nodeId, $now, self::AUTHORITY_KEY_ID);
        return $this->storeCredential($payload, $this->signPayload($payload));
    }

    /**
     * Rotate (refresh) the bounded device credential: a fresh bounded window
     * at entitlement.sequence + 1 under the current authority key. Refused
     * when the lifetime entitlement is revoked. Families, limits, License
     * Type, and product never change — they live in the entitlement record.
     */
    public function rotateCredential(string $nodeId, string $now): array
    {
        $entitlement = $this->requireEntitled();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($now);
        $current = $this->credentialRow(self::PRODUCT, $nodeId);
        $sequence = (int) $entitlement['sequence'] + 1;
        if ($current !== null && $sequence <= (int) $current['sequence']) {
            throw new DomainException('ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED');
        }
        $payload = $this->credentialPayload($sequence, $nodeId, $now, self::AUTHORITY_KEY_ID);
        $record = $this->storeCredential($payload, $this->signPayload($payload));
        $this->advanceEntitlementSequence($sequence, $now);
        return $record;
    }

    /**
     * Verified recovery issuance: after credential expiry the lifetime
     * entitlement persists and a replacement bounded lease is issued at a
     * strictly higher sequence. Refused when the entitlement is revoked or
     * missing — recovery never reactivates a revoked entitlement.
     */
    public function recoverCredential(string $nodeId, string $now): array
    {
        $entitlement = $this->requireEntitled();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($now);
        $sequence = (int) $entitlement['sequence'] + 1;
        $payload = $this->credentialPayload($sequence, $nodeId, $now, self::AUTHORITY_KEY_ID);
        $record = $this->storeCredential($payload, $this->signPayload($payload));
        $this->advanceEntitlementSequence($sequence, $now);
        return $record;
    }

    /**
     * Key rotation: rotate the bounded credential under the next authority key
     * without widening anything; the lifetime entitlement is untouched except
     * for its highest-sequence advance so older credentials become stale.
     */
    public function rotateKey(string $nodeId, string $now, string $newAuthorityKeyId): array
    {
        $entitlement = $this->requireEntitled();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($now);
        $sequence = (int) $entitlement['sequence'] + 1;
        $payload = $this->credentialPayload($sequence, $nodeId, $now, $newAuthorityKeyId);
        $record = $this->storeCredential($payload, $this->signPayload($payload));
        $this->advanceEntitlementSequence($sequence, $now);
        return $record;
    }

    /**
     * Lost-device revoke: preservation-only. The current device credential row
     * is flipped to revoked (never deleted); verification of it fails closed
     * while the lifetime entitlement itself stays intact.
     */
    public function revokeDeviceCredential(string $nodeId, string $now): array
    {
        $entitlement = $this->requireEntitled();
        FocusaSpec172LifetimeEntitlementMigration::assertTimestamp($now);
        $current = $this->credentialRow(self::PRODUCT, $nodeId);
        if ($current === null) {
            throw new DomainException('DEVICE_CREDENTIAL_MISSING');
        }
        $payload = $this->credentialPayload(
            (int) $current['sequence'],
            $nodeId,
            (string) $current['issued_at'],
            (string) $current['authority_key_id'],
            'revoked',
        );
        $stmt = $this->db->prepare(
            "UPDATE {$this->prefix}wpuiai_device_credentials SET status = 'revoked'
             WHERE product = :product AND node_id = :node_id",
        );
        $stmt->execute([':product' => self::PRODUCT, ':node_id' => $nodeId]);
        return ['payload' => $payload, 'signature_hex' => $this->signPayload($payload)];
    }

    private function advanceEntitlementSequence(int $sequence, string $occurredAt): void
    {
        $stmt = $this->db->prepare(
            "UPDATE {$this->prefix}wpuiai_lifetime_entitlements
             SET sequence = :sequence, updated_at = :updated_at
             WHERE product = :product AND sequence < :sequence",
        );
        $stmt->execute([
            ':sequence' => $sequence,
            ':updated_at' => $occurredAt,
            ':product' => self::PRODUCT,
        ]);
    }

    // ── the joint lifecycle state machine ────────────────────────────────

    /**
     * Stateless joint decision (mirrors the focusa-license Rust machine):
     * lifetime entitlement first, bounded device credential second. A revoked
     * entitlement defeats every stale and offline credential; a missing
     * entitlement denies; an expired credential with a live entitlement
     * resolves to recovery_only (lifetime preserved, replacement lease
     * issuable).
     *
     * @param array|null $entitlement ledger row
     * @param array|null $credential credential row
     */
    public function resolveState(?array $entitlement, ?array $credential, string $now): string
    {
        if ($entitlement === null) {
            return 'denied_unknown';
        }
        if (($entitlement['status'] ?? '') !== 'entitled') {
            return 'denied_revoked';
        }
        if ($credential === null) {
            return 'recovery_only';
        }
        if ((int) $credential['sequence'] < (int) $entitlement['sequence']) {
            return 'denied_stale';
        }
        if (($credential['status'] ?? '') === 'revoked') {
            return 'denied_revoked';
        }
        if ($now <= (string) $credential['expires_at']) {
            return 'active';
        }
        if ($now <= (string) $credential['offline_grace_until']) {
            return 'offline_grace';
        }
        return 'recovery_only';
    }

    /** Current joint posture for the product/node from durable rows. */
    public function resolve(string $nodeId, string $now): array
    {
        $entitlement = $this->entitlementRow(self::PRODUCT);
        $credential = $this->credentialRow(self::PRODUCT, $nodeId);
        $state = $this->resolveState($entitlement, $credential, $now);
        return [
            'schema' => 'focusa.spec172.lifetime_resolution.v1',
            'product' => self::PRODUCT,
            'license_type' => $entitlement['license_type'] ?? null,
            'term' => $entitlement['term'] ?? null,
            'entitlement_status' => $entitlement['status'] ?? null,
            'entitlement_sequence' => $entitlement !== null ? (int) $entitlement['sequence'] : null,
            'credential_sequence' => $credential !== null ? (int) $credential['sequence'] : null,
            'state' => $state,
        ];
    }

    // ── cross-language vector fixture ────────────────────────────────────

    /**
     * Deterministically regenerate the cross-language vector fixture. The
     * scenario drives one linear lifetime timeline (issue seq1 -> rotate seq2
     * -> recovery seq3 -> key rotation seq4 -> lost-device revoke -> refund/
     * revoke at seq5) and snapshots each record so every vector is provable.
     * All signatures are real Ed25519 over canonical payload bytes.
     */
    public function exportVectors(): array
    {
        $issuedAt1 = '2026-08-09T06:00:00Z';
        $issuedAt2 = '2026-08-09T07:00:00Z';
        $issuedAt3 = '2026-12-10T00:00:00Z';
        $issuedAt4 = '2026-12-12T00:00:00Z';

        $projection = [
            'schema' => 'focusa.spec172.focusa_operator_lifetime_projection.v1',
            'decision' => 'license_type_projected',
            'status' => 'active',
            'license_type' => self::LICENSE_TYPE,
            'term' => self::TERM,
            'sequence' => 1,
            'price_version' => 'v1.0.0',
            'family_digest' => 'sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae',
        ];
        $this->recordEntitlement($projection);
        $entitledSeq1 = $this->entitlementRow(self::PRODUCT);
        $credentialSeq1 = $this->issueInitialCredential(self::NODE_ID, $issuedAt1);
        $credentialSeq2 = $this->rotateCredential(self::NODE_ID, $issuedAt2);
        $entitledSeq2 = $this->entitlementRow(self::PRODUCT);
        $credentialSeq3 = $this->recoverCredential(self::NODE_ID, $issuedAt3);
        $entitledSeq3 = $this->entitlementRow(self::PRODUCT);
        $credentialSeq4 = $this->rotateKey(self::NODE_ID, $issuedAt4, self::AUTHORITY_KEY_ID_ROTATED);
        $revokedDevice = $this->revokeDeviceCredential(self::NODE_ID, '2026-12-13T00:00:00Z');
        $credentialSeq4Revoked = $this->credentialRow(self::PRODUCT, self::NODE_ID);
        $revokedSeq5 = $this->applyRefundRevoke(5, '2026-12-14T00:00:00Z');

        $credentials = [
            ['id' => 'credential_v1_seq1', 'payload' => $credentialSeq1['payload'], 'signature_hex' => $credentialSeq1['signature_hex']],
            ['id' => 'credential_v2_seq2_rotated', 'payload' => $credentialSeq2['payload'], 'signature_hex' => $credentialSeq2['signature_hex']],
            ['id' => 'credential_v3_seq3_recovered', 'payload' => $credentialSeq3['payload'], 'signature_hex' => $credentialSeq3['signature_hex']],
            ['id' => 'credential_v4_seq4_key_rotated', 'payload' => $credentialSeq4['payload'], 'signature_hex' => $credentialSeq4['signature_hex']],
            ['id' => 'credential_v4_seq4_device_revoked', 'payload' => $credentialSeq4Revoked, 'signature_hex' => $revokedDevice['signature_hex']],
        ];

        $entitlements = [
            ['id' => 'entitled_v1_seq1', ...$entitledSeq1],
            ['id' => 'entitled_v2_seq2', ...$entitledSeq2],
            ['id' => 'entitled_v3_seq3', ...$entitledSeq3],
            ['id' => 'revoked_v5_seq5', ...$revokedSeq5],
        ];

        $vectors = $this->vectors($entitlements, $credentials);
        return [
            'schema' => 'focusa.spec172.lifetime_credential_vectors.v1',
            'fixture_kind' => 'public_synthetic_nonproduction',
            'algorithm' => self::ALGORITHM,
            'seed_hex' => self::SEED_HEX,
            'public_key_hex' => $this->publicKeyHex(),
            'product' => self::PRODUCT,
            'license_type' => self::LICENSE_TYPE,
            'term' => self::TERM,
            'refresh_window_days' => self::REFRESH_WINDOW_DAYS,
            'offline_grace_days' => self::OFFLINE_GRACE_DAYS,
            'authority_key_id' => self::AUTHORITY_KEY_ID,
            'authority_key_id_rotated' => self::AUTHORITY_KEY_ID_ROTATED,
            'entitlements' => $entitlements,
            'credentials' => $credentials,
            'vectors' => $vectors,
        ];
    }

    /** The explicit vector set; expected labels are computed by the machine. */
    private function vectors(array $entitlements, array $credentials): array
    {
        $byEntitlement = [];
        foreach ($entitlements as $entry) {
            $byEntitlement[$entry['id']] = $entry;
        }
        $byCredential = [];
        foreach ($credentials as $entry) {
            $byCredential[$entry['id']] = $entry;
        }

        $cases = [
            ['id' => 'lifetime_active_within_window', 'entitlement' => 'entitled_v1_seq1', 'credential' => 'credential_v1_seq1', 'now' => '2026-09-01T00:00:00Z'],
            ['id' => 'lifetime_offline_grace_bounded', 'entitlement' => 'entitled_v1_seq1', 'credential' => 'credential_v1_seq1', 'now' => '2026-11-20T00:00:00Z'],
            ['id' => 'lifetime_survives_credential_expiry', 'entitlement' => 'entitled_v1_seq1', 'credential' => 'credential_v1_seq1', 'now' => '2026-12-20T00:00:00Z'],
            ['id' => 'lifetime_survives_missing_credential', 'entitlement' => 'entitled_v1_seq1', 'credential' => null, 'now' => '2026-12-20T00:00:00Z'],
            ['id' => 'credential_rotation_preserves_lifetime', 'entitlement' => 'entitled_v2_seq2', 'credential' => 'credential_v2_seq2_rotated', 'now' => '2026-09-01T00:00:00Z'],
            ['id' => 'rotation_makes_old_credential_stale', 'entitlement' => 'entitled_v2_seq2', 'credential' => 'credential_v1_seq1', 'now' => '2026-09-01T00:00:00Z'],
            ['id' => 'recovery_issuance_replaces_expired_lease', 'entitlement' => 'entitled_v3_seq3', 'credential' => 'credential_v3_seq3_recovered', 'now' => '2027-01-10T00:00:00Z'],
            ['id' => 'key_rotation_preserves_lifetime', 'entitlement' => 'entitled_v3_seq3', 'credential' => 'credential_v4_seq4_key_rotated', 'now' => '2026-12-20T00:00:00Z'],
            ['id' => 'stale_recovered_credential_never_trusted', 'entitlement' => 'entitled_v3_seq3', 'credential' => 'credential_v2_seq2_rotated', 'now' => '2026-12-20T00:00:00Z'],
            ['id' => 'refund_defeats_active_credential', 'entitlement' => 'revoked_v5_seq5', 'credential' => 'credential_v4_seq4_key_rotated', 'now' => '2027-01-10T00:00:00Z'],
            ['id' => 'revoke_defeats_offline_credential', 'entitlement' => 'revoked_v5_seq5', 'credential' => 'credential_v1_seq1', 'now' => '2026-11-20T00:00:00Z'],
            ['id' => 'revoke_defeats_stale_credential', 'entitlement' => 'revoked_v5_seq5', 'credential' => 'credential_v2_seq2_rotated', 'now' => '2026-09-01T00:00:00Z'],
            ['id' => 'revoked_device_credential_denied', 'entitlement' => 'entitled_v1_seq1', 'credential' => 'credential_v4_seq4_device_revoked', 'now' => '2026-12-20T00:00:00Z'],
            ['id' => 'missing_entitlement_denies', 'entitlement' => null, 'credential' => 'credential_v1_seq1', 'now' => '2026-09-01T00:00:00Z'],
        ];

        $vectors = [];
        foreach ($cases as $case) {
            $entitlement = $case['entitlement'] !== null ? $byEntitlement[$case['entitlement']] : null;
            $credential = $case['credential'] !== null ? $byCredential[$case['credential']] : null;
            $entitlementRow = $entitlement !== null ? $this->rowForEntitlement($entitlement) : null;
            $credentialRow = $credential !== null ? $credential['payload'] : null;
            $vectors[] = [
                'id' => $case['id'],
                'entitlement' => $case['entitlement'],
                'credential' => $case['credential'],
                'now' => $case['now'],
                'expected' => $this->resolveState($entitlementRow, $credentialRow, $case['now']),
            ];
        }
        return $vectors;
    }

    /** Ledger-row shape from a fixture entitlement snapshot (same columns). */
    private function rowForEntitlement(array $entitlement): array
    {
        return [
            'product' => $entitlement['product'],
            'license_type' => $entitlement['license_type'],
            'term' => $entitlement['term'],
            'status' => $entitlement['status'],
            'sequence' => $entitlement['sequence'],
            'price_version' => $entitlement['price_version'],
            'family_digest' => $entitlement['family_digest'],
            'node_limit' => $entitlement['node_limit'],
            'operator_seats' => $entitlement['operator_seats'],
            'updated_at' => $entitlement['updated_at'],
        ];
    }
}
