<?php
// Focusa authority client/store for Spec 172 limited-access assertions (atom
// focusa-vbcqu.20.15.11, 172.02.02). This is the client-side counterpart of the
// WPUIAI issuer/verifier: it verifies presented assertions against authoritative
// posture state and persists ONLY verified assertions in a local mirror. It can
// never issue, never self-grant, never create an EDD key, and never widen a
// family allowlist. Anonymous claims (no posture), tampered signatures, stale
// sequences, wrong nodes, unknown families, and paid families all fail closed.
// The Python vector test (tests/spec172_limited_assertion_vector_test.py)
// reimplements this exact evaluation natively against the shared fixture.
declare(strict_types=1);

final class FocusaSpec172LimitedAssertionClientStore
{
    public const SCHEMA = 'focusa.spec172.focusa_authority_client_store.v1';

    public function __construct(
        private PDO $db,
        private FocusaSpec172LimitedAssertionSigner $signer,
        private string $prefix = 'wp_',
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$table} (
            record_uuid TEXT NOT NULL PRIMARY KEY,
            posture_uuid TEXT NOT NULL,
            sequence BIGINT NOT NULL,
            product_scope VARCHAR(32) NOT NULL,
            node_uuid VARCHAR(64) NOT NULL,
            family_allowlist TEXT NOT NULL,
            issued_at VARCHAR(32) NOT NULL,
            refresh_at VARCHAR(32) NOT NULL,
            signer VARCHAR(64) NOT NULL,
            signature TEXT NOT NULL,
            stored_at VARCHAR(32) NOT NULL,
            UNIQUE (posture_uuid, sequence)
        )");
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    /**
     * Evaluate a presented assertion against authoritative posture state. Pure:
     * no side effects. Fails closed with SIGNATURE_INVALID (unverified or
     * tampered), ASSERTION_UNKNOWN (malformed), EMAIL_VERIFICATION_REQUIRED
     * (no posture), VERIFIED_LIMITED_ACCESS (revoked/superseded),
     * ENTITLEMENT_PRODUCT_MISMATCH, NODE_LIMIT_REACHED (wrong node),
     * ASSERTION_TAMPERED (binding mismatch), CAPABILITY_FAMILY_NOT_INCLUDED
     * (unknown or paid family), ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED (stale
     * sequence), CREDENTIAL_REFRESH_REQUIRED (bounded window elapsed).
     */
    public function evaluate(array $presented, ?array $posture, ?string $at = null): array
    {
        try {
            $payload = FocusaSpec172LimitedAssertionPayload::build($presented);
        } catch (InvalidArgumentException $error) {
            return $this->denied('ASSERTION_UNKNOWN');
        }
        if (!$this->signer->verify($payload, (string) ($presented['signature'] ?? ''))) {
            return $this->denied('SIGNATURE_INVALID');
        }
        if ($posture === null) {
            return $this->denied('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
            return $this->denied('VERIFIED_LIMITED_ACCESS');
        }
        $verdict = FocusaSpec172LimitedAssertionService::policyVerdict($presented, $posture, $at);
        if ($verdict !== 'valid') {
            return $this->denied($verdict);
        }
        return $this->validEnvelope($presented);
    }

    /** Verify and, ONLY for a valid assertion, persist it in the local mirror. */
    public function verifyAndStore(array $presented, ?array $posture, ?string $at = null, ?string $storedAt = null): array
    {
        $result = $this->evaluate($presented, $posture, $at);
        if ($result['verdict'] === 'valid') {
            $this->store($presented, $storedAt);
        }
        return $result;
    }

    public function storeCount(): int
    {
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function storedCountForPosture(string $postureUuid): int
    {
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        $statement = $this->db->prepare("SELECT COUNT(*) FROM {$table} WHERE posture_uuid = :posture");
        $statement->execute([':posture' => $postureUuid]);
        return (int) $statement->fetchColumn();
    }

    /** @return list<string> posture_uuid values present in the local mirror */
    public function storedPostures(): array
    {
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        $rows = $this->db->query("SELECT DISTINCT posture_uuid FROM {$table} ORDER BY posture_uuid ASC")->fetchAll(PDO::FETCH_COLUMN);
        return array_map('strval', $rows);
    }

    /** @return list<string> sorted family allowlist of the newest stored record for a posture */
    public function storedAllowlistForPosture(string $postureUuid): array
    {
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        $statement = $this->db->prepare("SELECT family_allowlist FROM {$table}
            WHERE posture_uuid = :posture ORDER BY sequence DESC LIMIT 1");
        $statement->execute([':posture' => $postureUuid]);
        $value = $statement->fetchColumn();
        if ($value === false) {
            return [];
        }
        return json_decode((string) $value, true, 512, JSON_THROW_ON_ERROR);
    }

    // ── helpers ──────────────────────────────────────────────────────────

    private function store(array $presented, ?string $storedAt): void
    {
        $table = $this->table('wpuiai_focusa_client_assertion_store');
        $recordUuid = self::uuid();
        $statement = $this->db->prepare("INSERT OR IGNORE INTO {$table}
            (record_uuid, posture_uuid, sequence, product_scope, node_uuid, family_allowlist,
             issued_at, refresh_at, signer, signature, stored_at)
            VALUES (:record, :posture, :sequence, :product, :node, :allowlist,
                    :issued, :refresh, :signer, :signature, :stored)");
        $statement->execute([
            ':record' => $recordUuid,
            ':posture' => (string) $presented['posture_uuid'],
            ':sequence' => (int) $presented['sequence'],
            ':product' => (string) $presented['product_scope'],
            ':node' => (string) $presented['node_uuid'],
            ':allowlist' => FocusaSpec172LimitedAssertionPayload::encodeCanonical(
                FocusaSpec172LimitedAssertionPayload::sortedFamilies($presented['family_allowlist'] ?? null),
            ),
            ':issued' => (string) $presented['issued_at'],
            ':refresh' => (string) $presented['refresh_at'],
            ':signer' => (string) $presented['signer'],
            ':signature' => (string) $presented['signature'],
            ':stored' => $storedAt ?? '2026-08-08T00:00:00Z',
        ]);
    }

    private function validEnvelope(array $presented): array
    {
        return [
            'verdict' => 'valid',
            'schema' => FocusaSpec172LimitedAssertionPayload::SCHEMA,
            'posture_uuid' => (string) $presented['posture_uuid'],
            'product_scope' => (string) $presented['product_scope'],
            'node_uuid' => (string) $presented['node_uuid'],
            'family_allowlist' => FocusaSpec172LimitedAssertionPayload::sortedFamilies($presented['family_allowlist'] ?? null),
            'sequence' => (int) $presented['sequence'],
            'issued_at' => (string) $presented['issued_at'],
            'refresh_at' => (string) $presented['refresh_at'],
            'signer' => (string) $presented['signer'],
        ];
    }

    private function denied(string $code): array
    {
        return ['verdict' => 'denied', 'code' => $code];
    }

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }
}
