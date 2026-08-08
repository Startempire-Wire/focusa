<?php
// Enumeration-resistant rate limiter. Counts are per facade + opaque client key + route
// and never disclose whether an email or account exists. The opaque client key is derived
// from the caller (e.g., session ID or IP hash) without receiving email/customer data.
declare(strict_types=1);

final class FocusaSpec152eRateLimiter
{
    public const SCHEMA = 'focusa.spec152e.rate_limiter.v1';
    public const WINDOW_SECONDS = 60;
    public const DEFAULT_MAX_PER_WINDOW = 5;
    public const CONSECUTIVE_MAX = 3;

    private PDO $db;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;
    private int $windowSeconds;
    private int $maxPerWindow;
    private int $consecutiveMax;

    public function __construct(
        PDO $db,
        string $prefix,
        callable $clock,
        int $windowSeconds = self::WINDOW_SECONDS,
        int $maxPerWindow = self::DEFAULT_MAX_PER_WINDOW,
        int $consecutiveMax = self::CONSECUTIVE_MAX,
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        if ($windowSeconds < 1 || $maxPerWindow < 1 || $consecutiveMax < 1) {
            throw new InvalidArgumentException('positive rate limit parameters required');
        }
        $this->db = $db;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
        $this->windowSeconds = $windowSeconds;
        $this->maxPerWindow = $maxPerWindow;
        $this->consecutiveMax = $consecutiveMax;
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        $this->ensureTable();
    }

    /**
     * Check whether a request is allowed. Returns true only when the rate limit is not exceeded.
     *
     * Parameters:
     *   - facade_id:       registered facade ID
     *   - opaque_client_key:  opaque key derived from the caller (session hash, IP hash, etc.)
     *   - route:           route being called (e.g., 'activation_start')
     *
     * The call is always counted (even when denied) so enumeration is indistinguishable.
     * The only difference between allowed and denied is the boolean return value.
     */
    public function allow(string $facadeId, string $opaqueClientKey, string $route): bool
    {
        if ($facadeId === '' || strlen($facadeId) > 96 || preg_match('/[\r\n]/', $facadeId)) {
            return false;
        }
        if (!preg_match('/^[a-f0-9]{64}$/D', $opaqueClientKey)) {
            return false;
        }
        if ($route === '' || strlen($route) > 128 || preg_match('/[\r\n]/', $route)) {
            return false;
        }

        $now = ($this->clock)();
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($now);
        $windowStart = self::minusSeconds($now, $this->windowSeconds);

        $table = $this->table('wpuiai_rate_limit_entries');
        $this->db->beginTransaction();
        try {
            // Count entries in the current window.
            $statement = $this->db->prepare(
                "SELECT COUNT(*), MAX(consecutive_count) FROM {$table}
                 WHERE facade_id = :facade AND opaque_client_key = :client_key
                   AND route = :route AND window_start >= :window_start"
            );
            $statement->execute([
                ':facade' => $facadeId,
                ':client_key' => $opaqueClientKey,
                ':route' => $route,
                ':window_start' => $windowStart,
            ]);
            $row = $statement->fetch(PDO::FETCH_NUM);
            $count = (int) ($row[0] ?? 0);
            $lastConsecutive = (int) ($row[1] ?? 0);

            if ($count >= $this->maxPerWindow || $lastConsecutive >= $this->consecutiveMax) {
                // Record the attempt and deny.
                $this->insertEntry($facadeId, $opaqueClientKey, $route, $now, $lastConsecutive + 1);
                $this->db->commit();
                return false;
            }

            $nextConsecutive = $lastConsecutive + 1;
            $this->insertEntry($facadeId, $opaqueClientKey, $route, $now, $nextConsecutive);
            $this->db->commit();
            return $nextConsecutive <= $this->consecutiveMax && ($count + 1) <= $this->maxPerWindow;
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            throw $error;
        }
    }

    /**
     * Reset is not exposed; rate limit state is window-based and self-expiring.
     */
    public function cleanup(string $now): int
    {
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($now);
        $cutoff = self::minusSeconds($now, $this->windowSeconds * 2);
        $table = $this->table('wpuiai_rate_limit_entries');
        $statement = $this->db->prepare("DELETE FROM {$table} WHERE window_start < :cutoff");
        $statement->execute([':cutoff' => $cutoff]);
        return $statement->rowCount();
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    private function insertEntry(string $facadeId, string $opaqueClientKey, string $route, string $now, int $consecutive): void
    {
        $table = $this->table('wpuiai_rate_limit_entries');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (facade_id, opaque_client_key, route, window_start, consecutive_count, created_at)
            VALUES (:facade, :client_key, :route, :window_start, :consecutive, :created)");
        $statement->execute([
            ':facade' => $facadeId,
            ':client_key' => $opaqueClientKey,
            ':route' => $route,
            ':window_start' => $now,
            ':consecutive' => $consecutive,
            ':created' => $now,
        ]);
    }

    private function ensureTable(): void
    {
        $table = $this->table('wpuiai_rate_limit_entries');
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$table} (
            facade_id {$key} NOT NULL,
            opaque_client_key VARCHAR(64) NOT NULL,
            route VARCHAR(128) NOT NULL,
            window_start VARCHAR(32) NOT NULL,
            consecutive_count BIGINT NOT NULL DEFAULT 1,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_rate_limit_window
            ON {$table} (facade_id, opaque_client_key, route, window_start)");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_rate_limit_cleanup
            ON {$table} (window_start)");
    }

    private static function minusSeconds(string $timestamp, int $seconds): string
    {
        $dt = DateTimeImmutable::createFromFormat('!Y-m-d\TH:i:s\Z', $timestamp, new DateTimeZone('UTC'));
        if ($dt === false) {
            throw new InvalidArgumentException('canonical timestamp required');
        }
        return $dt->modify("-{$seconds} seconds")->format('Y-m-d\TH:i:s\Z');
    }
}