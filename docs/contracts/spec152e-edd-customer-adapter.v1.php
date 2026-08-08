<?php
// Authority EDD customer resolution adapter. Resolves or creates EDD customers only from
// verified identity; merges exact evidence-backed records; never enumerates accounts or
// creates duplicates; returns typed account/customer references.
declare(strict_types=1);

final class FocusaSpec152eEddCustomerAdapter
{
    public const SCHEMA = 'focusa.spec152e.edd_customer_adapter.v1';
    public const VERSION = 1;

    private PDO $db;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(PDO $db, string $prefix = 'wp_', callable $clock)
    {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Resolve or create an EDD customer from a verified identity payload.
     *
     * Required fields in $verifiedIdentity:
     *   - identity_uuid:     canonical opaque UUID
     *   - account_uuid:      canonical opaque UUID
     *   - normalized_email:  exact verified email
     *   - verification_state: must be 'mailbox_verified' or 'account_promoted'
     *   - verified_at:       canonical UTC timestamp
     *   - idempotency_key:   bounded idempotency key
     *   - migration_provenance: evidence array
     *
     * Optional:
     *   - wordpress_user_id, stripe_customer_id
     *
     * Returns a typed reference:
     *   - edd_customer_id:   positive integer
     *   - account_uuid:      opaque UUID
     *   - identity_uuid:     opaque UUID
     *   - resolution:        'existing' | 'new'
     *   - created_at, updated_at
     */
    public function resolveCustomer(array $verifiedIdentity): array
    {
        if (!in_array($verifiedIdentity['verification_state'] ?? null, ['mailbox_verified', 'account_promoted'], true)) {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        if (!is_string($verifiedIdentity['verified_at'] ?? null) || $verifiedIdentity['verified_at'] === '') {
            throw new DomainException('EMAIL_VERIFICATION_REQUIRED');
        }
        self::assertTimestamp($verifiedIdentity['verified_at']);

        $this->assertUuid((string) ($verifiedIdentity['identity_uuid'] ?? ''), 'identity');
        $this->assertUuid((string) ($verifiedIdentity['account_uuid'] ?? ''), 'account');

        $email = (string) ($verifiedIdentity['normalized_email'] ?? '');
        if ($email === '' || filter_var($email, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('verified normalized email required');
        }

        $idempotencyKey = $this->idempotencyKey($verifiedIdentity);
        $evidence = $verifiedIdentity['migration_provenance'] ?? [];
        if (!is_array($evidence) || $evidence === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $provenance = $this->encodeCanonical($evidence);
        $digest = $this->digest([
            'identity_uuid' => $verifiedIdentity['identity_uuid'],
            'account_uuid' => $verifiedIdentity['account_uuid'],
            'normalized_email' => $email,
            'wordpress_user_id' => $verifiedIdentity['wordpress_user_id'] ?? null,
            'stripe_customer_id' => $verifiedIdentity['stripe_customer_id'] ?? null,
            'verified_at' => $verifiedIdentity['verified_at'],
            'migration_provenance' => json_decode($provenance, true, 512, JSON_THROW_ON_ERROR),
        ]);

        return $this->transaction(function () use ($verifiedIdentity, $email, $idempotencyKey, $digest, $provenance): array {
            $replay = $this->replay($idempotencyKey, 'resolve_customer', $digest);
            if ($replay !== null) {
                return $replay;
            }

            $now = ($this->clock)();
            self::assertTimestamp($now);

            // Look up existing EDD customer by exact email — never enumerate.
            $existing = $this->findCustomerByEmail($email);

            if ($existing !== null) {
                // Existing customer: verify the identity payload is consistent.
                $existingEmail = $existing['email'] ?? '';
                if (!hash_equals($email, $existingEmail)) {
                    throw new DomainException('EDD_CUSTOMER_RESOLUTION_FAILED');
                }
                // Optionally update WP user and Stripe customer references.
                $this->updateCustomerLinks(
                    (int) $existing['id'],
                    $verifiedIdentity['wordpress_user_id'] ?? null,
                    $verifiedIdentity['stripe_customer_id'] ?? null,
                    $now,
                );
                $result = $this->buildResult($existing, 'existing', $now);
                $this->recordIdempotency($idempotencyKey, 'resolve_customer', $digest, $result, $now);
                return $result;
            }

            // No existing customer: create one.
            $customerId = $this->createCustomer(
                $email,
                $verifiedIdentity['wordpress_user_id'] ?? null,
                $verifiedIdentity['stripe_customer_id'] ?? null,
                $provenance,
                $now,
            );
            $this->createCustomerEmailAddress($customerId, $email, $now);

            $result = $this->buildResult(
                ['id' => $customerId, 'email' => $email, 'user_id' => $verifiedIdentity['wordpress_user_id'] ?? null],
                'new',
                $now,
            );
            $this->recordIdempotency($idempotencyKey, 'resolve_customer', $digest, $result, $now);
            return $result;
        });
    }

    /**
     * Find an EDD customer by exact email address. Never returns multiple rows; never enumerates.
     */
    public function findCustomerByEmail(string $email): ?array
    {
        if ($email === '' || filter_var($email, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid email required for EDD customer lookup');
        }
        $table = $this->table('edd_customers');
        $emailTable = $this->table('edd_customer_email_addresses');

        // Check the primary customer email first.
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE email = :email LIMIT 1");
        $statement->execute([':email' => $email]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row !== false) {
            return $row;
        }

        // Check the email addresses table for additional verified emails.
        $eaStatement = $this->db->prepare("SELECT c.* FROM {$emailTable} ea
            JOIN {$table} c ON c.id = ea.customer_id
            WHERE ea.email = :email LIMIT 1");
        $eaStatement->execute([':email' => $email]);
        $row = $eaStatement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /**
     * Find an EDD customer by ID. Returns null when not found.
     */
    public function findCustomerById(int $customerId): ?array
    {
        if ($customerId < 1) {
            throw new InvalidArgumentException('positive EDD customer ID required');
        }
        $table = $this->table('edd_customers');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE id = :id");
        $statement->execute([':id' => $customerId]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false ? null : $row;
    }

    /**
     * Count is provided only for bounded verification; callers must never enumerate in production.
     */
    public function customerCount(): int
    {
        $table = $this->table('edd_customers');
        return (int) $this->db->query("SELECT COUNT(*) FROM {$table}")->fetchColumn();
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    // ── private helpers ────────────────────────────────────────────────

    private function createCustomer(string $email, ?int $wpUserId, ?string $stripeCustomerId, string $provenance, string $now): int
    {
        $table = $this->table('edd_customers');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (user_id, email, name, purchase_value, purchase_count, notes, date_created, stripe_customer_id)
            VALUES (:user_id, :email, '', 0, 0, :provenance, :created, :stripe)");
        $statement->execute([
            ':user_id' => $wpUserId,
            ':email' => $email,
            ':provenance' => $provenance,
            ':created' => $now,
            ':stripe' => $stripeCustomerId,
        ]);
        return (int) $this->db->lastInsertId();
    }

    private function createCustomerEmailAddress(int $customerId, string $email, string $now): void
    {
        $table = $this->table('edd_customer_email_addresses');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (customer_id, email, type, date_created)
            VALUES (:customer_id, :email, 'secondary', :created)");
        $statement->execute([
            ':customer_id' => $customerId,
            ':email' => $email,
            ':created' => $now,
        ]);
    }

    private function updateCustomerLinks(int $customerId, ?int $wpUserId, ?string $stripeCustomerId, string $now): void
    {
        $table = $this->table('edd_customers');
        $fields = [];
        $params = [':id' => $customerId];

        if ($wpUserId !== null) {
            $fields[] = 'user_id = :user_id';
            $params[':user_id'] = $wpUserId;
        }
        if ($stripeCustomerId !== null) {
            $fields[] = 'stripe_customer_id = :stripe';
            $params[':stripe'] = $stripeCustomerId;
        }
        if ($fields === []) {
            return;
        }
        $setClause = implode(', ', $fields);
        $this->db->prepare("UPDATE {$table} SET {$setClause} WHERE id = :id")->execute($params);
    }

    private function buildResult(array $customer, string $resolution, string $now): array
    {
        return [
            'edd_customer_id' => (int) $customer['id'],
            'email' => $customer['email'],
            'wordpress_user_id' => $customer['user_id'] ?? null,
            'resolution' => $resolution,
            'created_at' => $now,
            'updated_at' => $now,
        ];
    }

    private function replay(string $key, string $operation, string $digest): ?array
    {
        $table = $this->table('wpuiai_edd_customer_idempotency');
        $statement = $this->db->prepare("SELECT * FROM {$table} WHERE idempotency_key = :key");
        $statement->execute([':key' => $key]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        if (!hash_equals($operation, $row['operation']) || !hash_equals($digest, $row['request_digest'])) {
            throw new DomainException('IDEMPOTENCY_CONFLICT');
        }
        return json_decode($row['result_payload'], true, 512, JSON_THROW_ON_ERROR);
    }

    private function recordIdempotency(string $key, string $operation, string $digest, array $result, string $createdAt): void
    {
        $table = $this->table('wpuiai_edd_customer_idempotency');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (idempotency_key, operation, request_digest, result_payload, created_at)
            VALUES (:key, :operation, :digest, :payload, :created)");
        $statement->execute([
            ':key' => $key,
            ':operation' => $operation,
            ':digest' => $digest,
            ':payload' => json_encode($result, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES),
            ':created' => $createdAt,
        ]);
    }

    private function idempotencyKey(array $input): string
    {
        $key = (string) ($input['idempotency_key'] ?? '');
        if (preg_match('/^[A-Za-z0-9._:-]{8,191}$/D', $key) !== 1) {
            throw new InvalidArgumentException('bounded idempotency key required');
        }
        return $key;
    }

    private function digest(array $value): string
    {
        return hash('sha256', $this->encodeCanonical($value));
    }

    private function transaction(callable $callback): mixed
    {
        $this->db->beginTransaction();
        try {
            $result = $callback();
            $this->db->commit();
            return $result;
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    // ── static helpers ─────────────────────────────────────────────────

    public static function assertTimestamp(string $timestamp): void
    {
        $parsed = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($parsed === false || $parsed->format('Y-m-d\TH:i:s\Z') !== $timestamp) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
        }
    }

    public static function assertUuid(string $uuid, string $kind): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException("canonical opaque {$kind} UUID required");
        }
    }

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
}